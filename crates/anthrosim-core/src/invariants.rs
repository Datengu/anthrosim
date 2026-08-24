use thiserror::Error;

use crate::{
    checkpoint::{SimulationCheckpoint, state_digest64_with_temporary_mobility},
    config::PROBABILITY_PER_MILLION,
    demography::{DemographyConfigError, validate_demography_config},
    events::{EventKind, EventLog, EventProvenance},
    manifest::{ArtifactSchemas, RunManifest, StopReason},
    metrics::{
        MetricProvenance, MetricSeries, MetricSnapshot, MigrationMetrics, PopulationMetrics,
        ResourceMetrics,
    },
    migration::{
        MigrationCheckpointState, MigrationConfigError, MigrationError, MigrationSummary,
        MigrationSystem, validate_migration_config,
    },
    population::{Population, PopulationSummary, PopulationValidationError},
    provenance::{MODEL_SEMANTICS_ID, SourceRevisionIdentity},
    resources::{ResourceConfigError, ResourceError, ResourceSummary, validate_resource_config},
    rng::RngFactory,
    simulation::RecordedRun,
    time::{DAYS_PER_YEAR, SimTime},
    world::{PERMILLE_MAX, World, WorldError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantReport {
    pub event_count: u64,
    pub metric_snapshot_count: u64,
    pub births: u64,
    pub deaths: u64,
    pub migration_moves: u64,
    pub resource_periods: u64,
}

#[derive(Debug, Error)]
pub enum InvariantError {
    #[error("invariant violation: {0}")]
    Violation(String),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationValidationError),
    #[error(transparent)]
    Resources(#[from] ResourceError),
    #[error(transparent)]
    Migration(#[from] MigrationError),
    #[error(transparent)]
    DemographyConfig(#[from] DemographyConfigError),
    #[error(transparent)]
    ResourceConfig(#[from] ResourceConfigError),
    #[error(transparent)]
    MigrationConfig(#[from] MigrationConfigError),
}

impl SimulationCheckpoint {
    pub fn validate_invariants(&self) -> Result<InvariantReport, InvariantError> {
        validate_checkpoint_invariants(self)
    }
}

impl RecordedRun {
    pub fn validate_invariants(&self) -> Result<InvariantReport, InvariantError> {
        validate_recorded_run_invariants(self)
    }
}

pub fn validate_checkpoint_invariants(
    checkpoint: &SimulationCheckpoint,
) -> Result<InvariantReport, InvariantError> {
    validate_checkpoint_invariants_for_context(checkpoint, None)
}

fn validate_checkpoint_invariants_for_context(
    checkpoint: &SimulationCheckpoint,
    recorded_stop_reason: Option<StopReason>,
) -> Result<InvariantReport, InvariantError> {
    validate_checkpoint_identity(checkpoint, recorded_stop_reason)?;

    let rng_factory = RngFactory::new(checkpoint.experiment.seed);
    let world = World::generate(checkpoint.experiment.world, rng_factory)?;
    if world.digest64() != checkpoint.world_digest64 {
        return violation("world digest does not match deterministic reconstruction");
    }

    checkpoint.population.validate(&world)?;
    checkpoint
        .temporary_mobility
        .validate(&checkpoint.population, &world)
        .map_err(|error| {
            InvariantError::Violation(format!("temporary mobility state is invalid: {error}"))
        })?;
    checkpoint
        .resources
        .validate_checkpoint_state(&world, &checkpoint.experiment.resources)?;
    let migration = MigrationSystem::from_checkpoint_state(
        &checkpoint.population,
        &world,
        &checkpoint.experiment.migration,
        checkpoint.migration.clone(),
    )?;

    let population = checkpoint.population.summary();
    let resources = checkpoint.resources.summary(&checkpoint.population);
    let migration_summary = migration.summary();

    validate_resource_accounting(
        &resources,
        checkpoint.time.days(),
        checkpoint.experiment.resources.periods_per_year,
    )?;
    validate_migration_accounting(
        &checkpoint.migration,
        &migration_summary,
        &checkpoint.experiment.migration,
        &world,
        population.household_count,
        resources.periods_processed,
    )?;
    let counts = validate_events(
        &checkpoint.events,
        checkpoint.time.days(),
        &world,
        &checkpoint.population,
        &population,
        &resources,
        &migration_summary,
    )?;
    validate_metrics(
        &checkpoint.metrics,
        checkpoint.time.days(),
        checkpoint.state_digest64,
        &population,
        &resources,
        &migration_summary,
    )?;

    let actual_digest = state_digest64_with_temporary_mobility(
        checkpoint.time.days(),
        world.digest64(),
        checkpoint.population.digest64(),
        checkpoint.resources.digest64(),
        migration.digest64(),
        &checkpoint.temporary_mobility,
    );
    if actual_digest != checkpoint.state_digest64 {
        return violation(format!(
            "state digest mismatch: stored {}, reconstructed {}",
            checkpoint.state_digest64, actual_digest
        ));
    }

    Ok(InvariantReport {
        event_count: u64::try_from(checkpoint.events.len()).unwrap_or(u64::MAX),
        metric_snapshot_count: u64::try_from(checkpoint.metrics.len()).unwrap_or(u64::MAX),
        births: counts.births,
        deaths: counts.deaths,
        migration_moves: counts.migrations,
        resource_periods: resources.periods_processed,
    })
}

pub fn validate_recorded_run_invariants(
    run: &RecordedRun,
) -> Result<InvariantReport, InvariantError> {
    let report = validate_checkpoint_invariants_for_context(
        &run.checkpoint,
        Some(run.manifest.stop_reason),
    )?;
    validate_manifest_against_checkpoint(&run.manifest, &run.checkpoint)?;
    Ok(report)
}

fn validate_checkpoint_identity(
    checkpoint: &SimulationCheckpoint,
    recorded_stop_reason: Option<StopReason>,
) -> Result<(), InvariantError> {
    if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {
        return violation("checkpoint schema is not current");
    }
    if checkpoint.model_version != env!("CARGO_PKG_VERSION") {
        return violation("checkpoint model version does not match this build");
    }
    if checkpoint.model_semantics_id != MODEL_SEMANTICS_ID {
        return violation("checkpoint model semantics identity does not match this build");
    }
    let checkpoint_identity = SourceRevisionIdentity {
        model_version: checkpoint.model_version.clone(),
        model_semantics_id: checkpoint.model_semantics_id.clone(),
        git_commit: checkpoint.git_commit.clone(),
    };
    checkpoint
        .resume_lineage
        .validate_for_artifact(checkpoint.time.days(), &checkpoint_identity)
        .map_err(|error| {
            InvariantError::Violation(format!("checkpoint resume lineage is invalid: {error}"))
        })?;
    if checkpoint.experiment.schema_version != crate::ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return violation("experiment schema is not current");
    }
    validate_demography_config(&checkpoint.experiment.demography)?;
    validate_resource_config(&checkpoint.experiment.resources)?;
    validate_migration_config(&checkpoint.experiment.migration)?;

    if !checkpoint.time.days().is_multiple_of(DAYS_PER_YEAR)
        && !matches!(recorded_stop_reason, Some(StopReason::PopulationExtinct))
    {
        return violation("checkpoint day is not a completed annual boundary");
    }
    if checkpoint.completed_years != checkpoint.time.days() / DAYS_PER_YEAR {
        return violation("completedYears does not match the checkpoint day");
    }
    let duration_days =
        u128::from(checkpoint.experiment.duration_years) * u128::from(DAYS_PER_YEAR);
    if u128::from(checkpoint.time.days()) > duration_days {
        return violation("checkpoint time exceeds configured duration");
    }

    let population = checkpoint.population.summary();
    let terminal_state_matches = match checkpoint.terminal_stop_reason {
        None => true,
        Some(StopReason::DurationReached) => u128::from(checkpoint.time.days()) == duration_days,
        Some(StopReason::PopulationExtinct) => population.living_population == 0,
        Some(StopReason::PersonRecordLimitReached) => {
            population.person_records == checkpoint.experiment.population.max_person_records
        }
    };
    if !terminal_state_matches {
        return violation("checkpoint terminal stop reason does not reconcile with state");
    }
    Ok(())
}

fn validate_manifest_against_checkpoint(
    manifest: &RunManifest,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), InvariantError> {
    let rng_factory = RngFactory::new(checkpoint.experiment.seed);
    let world = World::generate(checkpoint.experiment.world, rng_factory)?;
    let population = checkpoint.population.summary();
    let resources = checkpoint.resources.summary(&checkpoint.population);
    let migration = MigrationSystem::from_checkpoint_state(
        &checkpoint.population,
        &world,
        &checkpoint.experiment.migration,
        checkpoint.migration.clone(),
    )?
    .summary();

    if manifest.schema_version != RunManifest::CURRENT_SCHEMA_VERSION
        || manifest.artifact_schemas != ArtifactSchemas::current()
        || manifest.model_version != checkpoint.model_version
        || manifest.model_semantics_id != checkpoint.model_semantics_id
        || manifest.git_commit != checkpoint.git_commit
        || manifest.resume_lineage != checkpoint.resume_lineage
        || manifest.experiment != checkpoint.experiment
        || manifest.start_time != SimTime::ZERO
        || manifest.end_time != checkpoint.time
        || checkpoint.terminal_stop_reason != Some(manifest.stop_reason)
        || manifest.world != world.summary()
        || manifest.population != population
        || manifest.resources != resources
        || manifest.migration != migration
        || manifest.state_digest64 != checkpoint.state_digest64
    {
        return violation("run manifest does not reconcile with the final checkpoint");
    }
    if manifest.statistics.simulated_days != checkpoint.time.days()
        || manifest.statistics.authoritative_event_count
            != u64::try_from(checkpoint.events.len()).unwrap_or(u64::MAX)
        || manifest.statistics.metric_snapshot_count
            != u64::try_from(checkpoint.metrics.len()).unwrap_or(u64::MAX)
        || manifest.statistics.resource_periods_processed != resources.periods_processed
        || manifest.statistics.migration_decision_boundaries != migration.decision_boundaries
    {
        return violation("run statistics do not reconcile with final artifacts");
    }

    match manifest.stop_reason {
        StopReason::DurationReached => {
            let duration_days =
                u128::from(manifest.experiment.duration_years) * u128::from(DAYS_PER_YEAR);
            if u128::from(manifest.end_time.days()) != duration_days {
                return violation("duration-reached run ended before or after configured duration");
            }
        }
        StopReason::PopulationExtinct => {
            if manifest.population.living_population != 0 {
                return violation("population-extinct run still has living people");
            }
        }
        StopReason::PersonRecordLimitReached => {
            if manifest.population.person_records
                != manifest.experiment.population.max_person_records
            {
                return violation("record-limit run did not end exactly at its record ceiling");
            }
        }
    }
    Ok(())
}

fn validate_resource_accounting(
    resources: &ResourceSummary,
    day: u64,
    periods_per_year: u16,
) -> Result<(), InvariantError> {
    let available =
        u128::from(resources.initial_food_stock) + u128::from(resources.regenerated_food);
    let accounted = u128::from(resources.harvested_food) + u128::from(resources.final_food_stock);
    if available != accounted {
        return violation(format!(
            "resource stock drift: available {available}, accounted {accounted}"
        ));
    }
    if resources.consumed_food != resources.harvested_food {
        return violation("consumed-food total differs from allocated harvest");
    }

    let periods = u64::from(periods_per_year);
    let full_years = day / DAYS_PER_YEAR;
    let remainder = day % DAYS_PER_YEAR;
    let partial_periods = (1..=periods)
        .filter(|period| period.saturating_mul(DAYS_PER_YEAR) / periods <= remainder)
        .count() as u64;
    let elapsed_boundaries = full_years
        .saturating_mul(periods)
        .saturating_add(partial_periods);
    if resources.periods_processed > elapsed_boundaries {
        return violation("resource periods exceed elapsed scheduled boundaries");
    }
    Ok(())
}

fn validate_migration_accounting(
    state: &MigrationCheckpointState,
    summary: &MigrationSummary,
    config: &crate::MigrationConfig,
    world: &World,
    household_count: u64,
    resource_periods: u64,
) -> Result<(), InvariantError> {
    if state.schema_version != MigrationCheckpointState::CURRENT_SCHEMA_VERSION
        || state.model_id != config.model_id
    {
        return violation("migration checkpoint identity does not match its configuration");
    }
    if state.households_under_pressure > state.households_evaluated
        || state.moves_completed > state.households_under_pressure
        || state.decision_boundaries > resource_periods
        || state.households_evaluated > state.decision_boundaries.saturating_mul(household_count)
    {
        return violation("migration counters have an impossible ordering");
    }

    let directional_distance = u128::from(state.northward_steps)
        + u128::from(state.eastward_steps)
        + u128::from(state.southward_steps)
        + u128::from(state.westward_steps);
    if directional_distance != u128::from(state.total_distance_cells) {
        return violation("migration directional steps do not reconcile with total distance");
    }
    let score_ceiling = u128::from(state.moves_completed) * u128::from(PERMILLE_MAX);
    for total in [
        state.origin_resource_score_total,
        state.destination_resource_score_total,
        state.origin_water_security_score_total,
        state.destination_water_security_score_total,
    ] {
        if u128::from(total) > score_ceiling {
            return violation("migration score total exceeds the move-count permille ceiling");
        }
    }

    let trace_limit = usize::try_from(config.max_recorded_decision_traces).unwrap_or(usize::MAX);
    let move_limit = usize::try_from(state.moves_completed).unwrap_or(usize::MAX);
    if state.recorded_decision_traces.len() > trace_limit
        || state.recorded_decision_traces.len() > move_limit
    {
        return violation("migration trace count exceeds its configured or completed-move limit");
    }
    if !config.enabled
        && (state.decision_boundaries != 0
            || state.households_evaluated != 0
            || state.households_under_pressure != 0
            || state.moves_completed != 0
            || state.people_moved != 0
            || !state.recorded_decision_traces.is_empty())
    {
        return violation("disabled migration accumulated migration activity");
    }
    for trace in &state.recorded_decision_traces {
        validate_migration_trace(trace, world, household_count, config.candidate_radius_cells)?;
    }

    if summary.decision_boundaries != state.decision_boundaries
        || summary.households_evaluated != state.households_evaluated
        || summary.households_under_pressure != state.households_under_pressure
        || summary.moves_completed != state.moves_completed
        || summary.people_moved != state.people_moved
        || summary.total_distance_cells != state.total_distance_cells
        || summary.travel_condition_cost_total != state.travel_condition_cost_total
    {
        return violation("migration summary does not reconcile with checkpoint counters");
    }
    Ok(())
}

fn validate_migration_trace(
    trace: &crate::MigrationDecisionTrace,
    world: &World,
    household_count: u64,
    radius: u16,
) -> Result<(), InvariantError> {
    if trace.household.0 == 0 || trace.household.0 > household_count {
        return violation("migration trace references an invalid household");
    }
    if world.cell(trace.origin).is_none()
        || world.cell(trace.destination).is_none()
        || world.cell(trace.best_candidate).is_none()
    {
        return violation("migration trace references a cell outside the world");
    }
    let distance = manhattan_distance(world, trace.origin, trace.destination).ok_or_else(|| {
        InvariantError::Violation("migration trace coordinates are invalid".into())
    })?;
    if distance == 0 || distance != trace.distance_cells || distance > radius {
        return violation("migration trace violates bounded local movement");
    }
    if trace.decision_day != trace.completed_day
        || trace.pressure_permille > PERMILLE_MAX
        || trace.travel_condition_cost_per_person > PERMILLE_MAX
        || trace.selected_weight == 0
        || trace.selected_weight > trace.total_move_weight
        || trace.choice_draw >= trace.total_move_weight
    {
        return violation("migration trace decision accounting is invalid");
    }
    Ok(())
}

#[derive(Default)]
struct EventCounts {
    births: u64,
    deaths: u64,
    scarcity_deaths: u64,
    migrations: u64,
    people_moved: u64,
    migration_distance: u64,
}

fn validate_events(
    events: &EventLog,
    day: u64,
    world: &World,
    population_state: &Population,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<EventCounts, InvariantError> {
    if events.schema_version != EventLog::CURRENT_SCHEMA_VERSION {
        return violation("event log schema is not current");
    }

    let mut counts = EventCounts::default();
    let mut previous_day = None;
    for (index, record) in events.events.iter().enumerate() {
        let expected_sequence = u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1);
        if record.sequence != expected_sequence
            || record.day > day
            || previous_day.is_some_and(|prior| record.day < prior)
            || record.provenance != EventProvenance::Authoritative
        {
            return violation("event sequence, time ordering, or provenance is invalid");
        }
        previous_day = Some(record.day);

        match &record.event {
            EventKind::Birth {
                person,
                female_parent,
                male_parent,
                household,
                cell,
                reproductive_sex,
            } => {
                counts.births = counts.births.saturating_add(1);
                let snapshot = population_state.person(*person).ok_or_else(|| {
                    InvariantError::Violation("birth event references a missing person".into())
                })?;
                let birth_day = i64::try_from(record.day)
                    .map_err(|_| InvariantError::Violation("birth event day exceeds i64".into()))?;
                if snapshot.birth_day != birth_day
                    || snapshot.female_parent != *female_parent
                    || snapshot.male_parent != *male_parent
                    || snapshot.household != *household
                    || snapshot.reproductive_sex != *reproductive_sex
                    || world.cell(*cell).is_none()
                {
                    return violation(
                        "birth event does not reconcile with persistent person state",
                    );
                }
            }
            EventKind::Death {
                person,
                household,
                cell,
                cause,
                condition_permille,
                probability_per_million,
            } => {
                counts.deaths = counts.deaths.saturating_add(1);
                if matches!(cause, crate::DeathCause::ResourceScarcity) {
                    counts.scarcity_deaths = counts.scarcity_deaths.saturating_add(1);
                }
                let snapshot = population_state.person(*person).ok_or_else(|| {
                    InvariantError::Violation("death event references a missing person".into())
                })?;
                if snapshot.death_day != Some(record.day)
                    || snapshot.household != *household
                    || snapshot.location != *cell
                    || *condition_permille > PERMILLE_MAX
                    || *probability_per_million > PROBABILITY_PER_MILLION
                    || world.cell(*cell).is_none()
                {
                    return violation(
                        "death event does not reconcile with persistent person state",
                    );
                }
            }
            EventKind::HouseholdMigration {
                household,
                people_moved,
                origin,
                destination,
                distance_cells,
                pressure_permille,
                selected_weight,
                total_move_weight,
                choice_draw,
                travel_condition_cost_per_person,
                ..
            } => {
                counts.migrations = counts.migrations.saturating_add(1);
                counts.people_moved = counts.people_moved.saturating_add(u64::from(*people_moved));
                counts.migration_distance = counts
                    .migration_distance
                    .saturating_add(u64::from(*distance_cells));
                let distance =
                    manhattan_distance(world, *origin, *destination).ok_or_else(|| {
                        InvariantError::Violation("migration event references invalid cells".into())
                    })?;
                if household.0 == 0
                    || household.0 > population.household_count
                    || distance == 0
                    || distance != *distance_cells
                    || *pressure_permille > PERMILLE_MAX
                    || *travel_condition_cost_per_person > PERMILLE_MAX
                    || *selected_weight == 0
                    || *selected_weight > *total_move_weight
                    || *choice_draw >= *total_move_weight
                {
                    return violation("migration event accounting is invalid");
                }
            }
        }
    }

    if counts.births != population.births_since_start
        || counts.deaths != population.deaths_since_start
        || counts.scarcity_deaths != resources.scarcity_deaths
        || counts.migrations != migration.moves_completed
        || counts.people_moved != migration.people_moved
        || counts.migration_distance != migration.total_distance_cells
    {
        return violation("authoritative event totals do not reconcile with subsystem summaries");
    }
    Ok(counts)
}

fn validate_metrics(
    metrics: &MetricSeries,
    day: u64,
    state_digest: u64,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<(), InvariantError> {
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION
        || metrics.snapshots.is_empty()
    {
        return violation("metric series schema is invalid or terminal snapshot is missing");
    }
    let mut previous_day = None;
    for snapshot in &metrics.snapshots {
        if snapshot.schema_version != MetricSnapshot::CURRENT_SCHEMA_VERSION
            || snapshot.provenance != MetricProvenance::Derived
            || snapshot.day > day
            || previous_day.is_some_and(|prior| snapshot.day <= prior)
        {
            return violation("metric snapshot schema, provenance, or ordering is invalid");
        }
        previous_day = Some(snapshot.day);
    }
    let final_snapshot = metrics.snapshots.last().expect("non-empty checked above");
    if final_snapshot.day != day
        || final_snapshot.state_digest64 != state_digest
        || final_snapshot.population != PopulationMetrics::from(population)
        || final_snapshot.resources != ResourceMetrics::from(resources)
        || final_snapshot.migration != MigrationMetrics::from(migration)
    {
        return violation("terminal derived metrics do not reconcile with authoritative state");
    }
    Ok(())
}

fn manhattan_distance(world: &World, a: crate::ids::CellId, b: crate::ids::CellId) -> Option<u16> {
    let (ax, ay) = world.coordinates(a)?;
    let (bx, by) = world.coordinates(b)?;
    u16::try_from(ax.abs_diff(bx).saturating_add(ay.abs_diff(by))).ok()
}

fn violation<T>(message: impl Into<String>) -> Result<T, InvariantError> {
    Err(InvariantError::Violation(message.into()))
}
