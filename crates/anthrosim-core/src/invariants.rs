use thiserror::Error;

use crate::{
    checkpoint::{SimulationCheckpoint, state_digest64},
    config::{PROBABILITY_PER_MILLION, validate_demography_config},
    events::{EventKind, EventLog, EventProvenance},
    manifest::{RunManifest, StopReason},
    metrics::{MetricProvenance, MetricSeries, MigrationMetrics, PopulationMetrics, ResourceMetrics},
    migration::{MigrationCheckpointState, MigrationError, MigrationSystem, MigrationSummary, validate_migration_config},
    population::{PopulationSummary, PopulationValidationError},
    resources::{ResourceError, ResourceSummary, validate_resource_config},
    rng::RngFactory,
    simulation::RecordedRun,
    time::DAYS_PER_YEAR,
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
    DemographyConfig(#[from] crate::demography::DemographyConfigError),
    #[error(transparent)]
    ResourceConfig(#[from] crate::resources::ResourceConfigError),
    #[error(transparent)]
    MigrationConfig(#[from] crate::migration::MigrationConfigError),
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
    if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {
        return violation(format!(
            "checkpoint schema {} does not match current schema {}",
            checkpoint.schema_version,
            SimulationCheckpoint::CURRENT_SCHEMA_VERSION
        ));
    }
    if checkpoint.model_version != env!("CARGO_PKG_VERSION") {
        return violation(format!(
            "checkpoint model version {} does not match current model version {}",
            checkpoint.model_version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    if checkpoint.experiment.schema_version != crate::ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return violation("checkpoint experiment schema is not current");
    }
    validate_demography_config(&checkpoint.experiment.demography)?;
    validate_resource_config(&checkpoint.experiment.resources)?;
    validate_migration_config(&checkpoint.experiment.migration)?;

    let expected_completed_years = checkpoint.time.days() / DAYS_PER_YEAR;
    if checkpoint.completed_years != expected_completed_years {
        return violation(format!(
            "checkpoint completedYears {} does not match day-derived value {}",
            checkpoint.completed_years, expected_completed_years
        ));
    }
    let duration_days = u128::from(checkpoint.experiment.duration_years) * u128::from(DAYS_PER_YEAR);
    if u128::from(checkpoint.time.days()) > duration_days {
        return violation("checkpoint time exceeds configured experiment duration");
    }

    let rng_factory = RngFactory::new(checkpoint.experiment.seed);
    let world = World::generate(checkpoint.experiment.world, rng_factory)?;
    if world.digest64() != checkpoint.world_digest64 {
        return violation("checkpoint world digest does not match deterministic reconstruction");
    }

    checkpoint.population.validate(&world)?;
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

    validate_resource_accounting(&resources, checkpoint.time.days(), &checkpoint.experiment.resources)?;
    validate_migration_accounting(
        &checkpoint.migration,
        &migration_summary,
        &checkpoint.experiment.migration,
        &world,
        population.household_count,
        resources.periods_processed,
    )?;
    let event_counts = validate_events(
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

    let actual_digest = state_digest64(
        checkpoint.time.days(),
        world.digest64(),
        checkpoint.population.digest64(),
        checkpoint.resources.digest64(),
        migration.digest64(),
    );
    if actual_digest != checkpoint.state_digest64 {
        return violation(format!(
            "checkpoint state digest mismatch: stored {}, reconstructed {}",
            checkpoint.state_digest64, actual_digest
        ));
    }

    Ok(InvariantReport {
        event_count: u64::try_from(checkpoint.events.len()).unwrap_or(u64::MAX),
        metric_snapshot_count: u64::try_from(checkpoint.metrics.len()).unwrap_or(u64::MAX),
        births: event_counts.births,
        deaths: event_counts.deaths,
        migration_moves: event_counts.migrations,
        resource_periods: resources.periods_processed,
    })
}

pub fn validate_recorded_run_invariants(
    run: &RecordedRun,
) -> Result<InvariantReport, InvariantError> {
    let report = validate_checkpoint_invariants(&run.checkpoint)?;
    validate_manifest_against_checkpoint(&run.manifest, &run.checkpoint)?;
    Ok(report)
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

    if manifest.model_version != checkpoint.model_version
        || manifest.git_commit != checkpoint.git_commit
        || manifest.experiment != checkpoint.experiment
        || manifest.end_time != checkpoint.time
        || manifest.world != world.summary()
        || manifest.population != population
        || manifest.resources != resources
        || manifest.migration != migration
        || manifest.state_digest64 != checkpoint.state_digest64
    {
        return violation("run manifest does not reconcile with its final checkpoint");
    }
    if manifest.statistics.simulated_days != checkpoint.time.days()
        || manifest.statistics.authoritative_event_count
            != u64::try_from(checkpoint.events.len()).unwrap_or(u64::MAX)
        || manifest.statistics.metric_snapshot_count
            != u64::try_from(checkpoint.metrics.len()).unwrap_or(u64::MAX)
        || manifest.statistics.resource_periods_processed != resources.periods_processed
        || manifest.statistics.migration_decision_boundaries != migration.decision_boundaries
    {
        return violation("run manifest statistics do not reconcile with final artifacts");
    }

    match manifest.stop_reason {
        StopReason::DurationReached => {
            let expected = u128::from(manifest.experiment.duration_years) * u128::from(DAYS_PER_YEAR);
            if u128::from(manifest.end_time.days()) != expected {
                return violation("duration-reached run did not end at configured duration");
            }
        }
        StopReason::PopulationExtinct => {
            if manifest.population.living_population != 0 {
                return violation("population-extinct run still has living people");
            }
        }
        StopReason::PersonRecordLimitReached => {
            if manifest.population.person_records != manifest.experiment.population.max_person_records {
                return violation("record-limit stop does not end exactly at the configured record ceiling");
            }
        }
    }
    Ok(())
}

fn validate_resource_accounting(
    resources: &ResourceSummary,
    day: u64,
    config: &crate::ResourceConfig,
) -> Result<(), InvariantError> {
    let available = u128::from(resources.initial_food_stock) + u128::from(resources.regenerated_food);
    let accounted = u128::from(resources.harvested_food) + u128::from(resources.final_food_stock);
    if available != accounted {
        return violation(format!(
            "resource stock accounting drifted: available {available}, accounted {accounted}"
        ));
    }
    if resources.consumed_food != resources.harvested_food {
        return violation("resource consumed-food total does not match harvested allocation");
    }
    if resources.scarcity_deaths > resources.household_periods_with_unmet_need.saturating_mul(u64::MAX) {
        return violation("unreachable resource accounting state");
    }

    let full_years = day / DAYS_PER_YEAR;
    let remainder = day % DAYS_PER_YEAR;
    let periods = u64::from(config.periods_per_year);
    let partial_periods = (1..=periods)
        .filter(|period| period.saturating_mul(DAYS_PER_YEAR) / periods <= remainder)
        .count() as u64;
    let elapsed_period_boundaries = full_years
        .saturating_mul(periods)
        .saturating_add(partial_periods);
    if resources.periods_processed > elapsed_period_boundaries {
        return violation("resource periods processed exceed elapsed scheduled boundaries");
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
        return violation("migration checkpoint identity does not match configuration");
    }
    if state.households_under_pressure > state.households_evaluated
        || state.moves_completed > state.households_under_pressure
        || state.decision_boundaries > resource_periods
    {
        return violation("migration counter ordering is impossible");
    }
    if state.households_evaluated
        > state.decision_boundaries.saturating_mul(household_count)
    {
        return violation("migration evaluated more households than decision boundaries permit");
    }
    let directional_distance = u128::from(state.northward_steps)
        + u128::from(state.eastward_steps)
        + u128::from(state.southward_steps)
        + u128::from(state.westward_steps);
    if directional_distance != u128::from(state.total_distance_cells) {
        return violation("migration directional step totals do not reconcile with total distance");
    }
    let score_ceiling = u128::from(state.moves_completed) * u128::from(PERMILLE_MAX);
    for (name, total) in [
        ("origin resource", state.origin_resource_score_total),
        ("destination resource", state.destination_resource_score_total),
        ("origin water", state.origin_water_security_score_total),
        ("destination water", state.destination_water_security_score_total),
    ] {
        if u128::from(total) > score_ceiling {
            return violation(format!("migration {name} score total exceeds move-count ceiling"));
        }
    }
    if state.recorded_decision_traces.len()
        > usize::try_from(config.max_recorded_decision_traces).unwrap_or(usize::MAX)
        || state.recorded_decision_traces.len()
            > usize::try_from(state.moves_completed).unwrap_or(usize::MAX)
    {
        return violation("migration decision trace count exceeds configured or completed-move count");
    }
    if !config.enabled
        && (state.decision_boundaries != 0
            || state.households_evaluated != 0
            || state.moves_completed != 0
            || !state.recorded_decision_traces.is_empty())
    {
        return violation("disabled migration accumulated migration activity");
    }

    for trace in &state.recorded_decision_traces {
        validate_migration_trace(trace, world, household_count, config.candidate_radius_cells)?;
    }

    if summary.decision_boundaries != state.decision_boundaries
        || summary.moves_completed != state.moves_completed
        || summary.people_moved != state.people_moved
        || summary.total_distance_cells != state.total_distance_cells
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
    let distance = manhattan_distance(world, trace.origin, trace.destination)
        .ok_or_else(|| InvariantError::Violation("migration trace coordinates are invalid".to_owned()))?;
    if distance == 0 || distance != trace.distance_cells || distance > radius {
        return violation("migration trace distance violates bounded local movement");
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
    population_state: &crate::Population,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<EventCounts, InvariantError> {
    if events.schema_version != EventLog::CURRENT_SCHEMA_VERSION {
        return violation("event log schema is not current");
    }
    let mut counts = EventCounts::default();
    let mut previous_day = 0_u64;
    for (index, record) in events.events.iter().enumerate() {
        let expected_sequence = u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1);
        if record.sequence != expected_sequence
            || record.day > day
            || (index > 0 && record.day < previous_day)
            || record.provenance != EventProvenance::Authoritative
        {
            return violation("event log sequence, time ordering, or provenance is invalid");
        }
        previous_day = record.day;
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
                let snapshot = population_state
                    .person(*person)
                    .ok_or_else(|| InvariantError::Violation("birth event references missing person".to_owned()))?;
                if snapshot.birth_day != i64::try_from(record.day).unwrap_or(i64::MAX)
                    || snapshot.female_parent != *female_parent
                    || snapshot.male_parent != *male_parent
                    || snapshot.household != *household
                    || snapshot.reproductive_sex != *reproductive_sex
                    || world.cell(*cell).is_none()
                {
                    return violation("birth event does not reconcile with persistent person state");
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
                let snapshot = population_state
                    .person(*person)
                    .ok_or_else(|| InvariantError::Violation("death event references missing person".to_owned()))?;
                if snapshot.death_day != Some(record.day)
                    || snapshot.household != *household
                    || snapshot.location != *cell
                    || *condition_permille > PERMILLE_MAX
                    || *probability_per_million > PROBABILITY_PER_MILLION
                    || world.cell(*cell).is_none()
                {
                    return violation("death event does not reconcile with persistent person state");
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
                let distance = manhattan_distance(world, *origin, *destination)
                    .ok_or_else(|| InvariantError::Violation("migration event has invalid cells".to_owned()))?;
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
        return violation("authoritative event counts do not reconcile with subsystem summaries");
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
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION {
        return violation("metric series schema is not current");
    }
    if metrics.snapshots.is_empty() {
        return violation("checkpoint has no terminal metric snapshot");
    }
    let mut previous = None;
    for snapshot in &metrics.snapshots {
        if snapshot.schema_version != crate::MetricSnapshot::CURRENT_SCHEMA_VERSION
            || snapshot.provenance != MetricProvenance::Derived
            || snapshot.day > day
            || previous.is_some_and(|prior| snapshot.day <= prior)
        {
            return violation("metric snapshot schema, provenance, or ordering is invalid");
        }
        previous = Some(snapshot.day);
    }
    let final_snapshot = metrics
        .snapshots
        .last()
        .ok_or_else(|| InvariantError::Violation("terminal metric snapshot is missing".to_owned()))?;
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
