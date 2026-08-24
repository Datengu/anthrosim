use thiserror::Error;

use crate::{
    checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64_with_temporary_mobility},
    config::ExperimentConfig,
    demography::{
        DemographyConfigError, DemographyRngs, DemographyStepOutcome,
        process_demographic_year_recorded, validate_demography_config,
    },
    events::EventLog,
    evidence::EvidenceError,
    manifest::{ArtifactSchemas, RunManifest, RunStatistics, StopReason},
    metrics::{
        MetricProvenance, MetricSeries, MetricSnapshot, MigrationMetrics, PopulationMetrics,
        ResourceMetrics,
    },
    migration::{
        MigrationBoundaryContext, MigrationConfigError, MigrationError, MigrationRngs,
        MigrationSystem, validate_migration_config,
    },
    population::{Population, PopulationError},
    provenance::{MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, SourceRevisionIdentity},
    resources::{
        ResourceConfigError, ResourceError, ResourcePeriodContext, ResourceRngs,
        ResourceStepOutcome, ResourceSystem, validate_resource_config,
    },
    rng::RngFactory,
    temporary_mobility::{TemporaryMobilityState, TemporaryMobilityValidationError},
    time::{DAYS_PER_YEAR, SimTime},
    world::{World, WorldError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRun {
    pub manifest: RunManifest,
    pub checkpoint: SimulationCheckpoint,
}

impl RecordedRun {
    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.checkpoint.events
    }

    #[must_use]
    pub const fn metrics(&self) -> &MetricSeries {
        &self.checkpoint.metrics
    }
}

/// Authoritative headless simulation host.
#[derive(Debug)]
pub struct Simulation {
    config: ExperimentConfig,
    time: SimTime,
    terminal_stop_reason: Option<StopReason>,
    resume_lineage: ResumeLineage,
    world: World,
    population: Population,
    temporary_mobility: TemporaryMobilityState,
    resources: ResourceSystem,
    migration: MigrationSystem,
    demography_rngs: DemographyRngs,
    resource_rngs: ResourceRngs,
    migration_rngs: MigrationRngs,
    events: EventLog,
    metrics: MetricSeries,
}

impl Simulation {
    pub fn new(config: ExperimentConfig) -> Result<Self, SimulationError> {
        validate_experiment(&config)?;

        let rng_factory = RngFactory::new(config.seed);
        let world = World::generate(config.world, rng_factory)?;
        let population = Population::initialize(config.population, &world, rng_factory)?;
        let temporary_mobility = TemporaryMobilityState::at_residence(&population);
        temporary_mobility.validate(&population, &world)?;
        let resources = ResourceSystem::initialize(&world, &config.resources)?;
        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;

        Ok(Self {
            demography_rngs: DemographyRngs::new(rng_factory),
            resource_rngs: ResourceRngs::new(rng_factory),
            migration_rngs: MigrationRngs::new(rng_factory),
            config,
            time: SimTime::ZERO,
            terminal_stop_reason: None,
            resume_lineage: ResumeLineage::new(),
            world,
            population,
            temporary_mobility,
            resources,
            migration,
            events: EventLog::new(),
            metrics: MetricSeries::annual(),
        })
    }

    pub fn from_checkpoint(checkpoint: SimulationCheckpoint) -> Result<Self, SimulationError> {
        if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {
            return Err(SimulationError::UnsupportedCheckpointSchema {
                found: checkpoint.schema_version,
                supported: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,
            });
        }
        if checkpoint.model_version != env!("CARGO_PKG_VERSION") {
            return Err(SimulationError::CheckpointModelVersionMismatch {
                found: checkpoint.model_version,
                expected: env!("CARGO_PKG_VERSION").to_owned(),
            });
        }
        if checkpoint.model_semantics_id != MODEL_SEMANTICS_ID {
            return Err(SimulationError::CheckpointModelSemanticsMismatch {
                found: checkpoint.model_semantics_id,
                expected: MODEL_SEMANTICS_ID.to_owned(),
            });
        }
        validate_experiment(&checkpoint.experiment)?;
        if checkpoint.events.schema_version != EventLog::CURRENT_SCHEMA_VERSION {
            return Err(SimulationError::CheckpointArtifactSchemaMismatch { artifact: "events" });
        }
        if checkpoint.metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION {
            return Err(SimulationError::CheckpointArtifactSchemaMismatch {
                artifact: "metrics",
            });
        }
        if !checkpoint.time.days().is_multiple_of(DAYS_PER_YEAR)
            || checkpoint.completed_years != checkpoint.time.days() / DAYS_PER_YEAR
        {
            return Err(SimulationError::UnsupportedCheckpointBoundary {
                day: checkpoint.time.days(),
            });
        }
        if checkpoint.completed_years > checkpoint.experiment.duration_years {
            return Err(SimulationError::CheckpointBeyondDuration {
                completed_years: checkpoint.completed_years,
                duration_years: checkpoint.experiment.duration_years,
            });
        }

        let source_identity = SourceRevisionIdentity {
            model_version: checkpoint.model_version.clone(),
            model_semantics_id: checkpoint.model_semantics_id.clone(),
            git_commit: checkpoint.git_commit.clone(),
        };
        checkpoint
            .resume_lineage
            .validate_for_artifact(checkpoint.time.days(), &source_identity)
            .map_err(|error| SimulationError::CheckpointResumeLineageInvalid {
                reason: error.to_string(),
            })?;

        let rng_factory = RngFactory::new(checkpoint.experiment.seed);
        let world = World::generate(checkpoint.experiment.world, rng_factory)?;
        if world.digest64() != checkpoint.world_digest64 {
            return Err(SimulationError::CheckpointWorldDigestMismatch {
                expected: checkpoint.world_digest64,
                actual: world.digest64(),
            });
        }

        checkpoint
            .population
            .validate(&world)
            .map_err(PopulationError::from)?;
        checkpoint
            .temporary_mobility
            .validate(&checkpoint.population, &world)?;
        checkpoint
            .resources
            .validate_checkpoint_state(&world, &checkpoint.experiment.resources)?;
        validate_terminal_checkpoint_state(&checkpoint)?;

        let boundary_day = checkpoint.time.days();
        let boundary_completed_years = checkpoint.completed_years;
        let source_state_digest64 = checkpoint.state_digest64;
        let continuation_identity = SourceRevisionIdentity::current();
        let mut resume_lineage = checkpoint.resume_lineage;
        resume_lineage.boundaries.push(ResumeBoundary {
            source: source_identity,
            continuation: continuation_identity,
            boundary_day,
            boundary_completed_years,
            source_state_digest64,
        });

        let migration = MigrationSystem::from_checkpoint_state(
            &checkpoint.population,
            &world,
            &checkpoint.experiment.migration,
            checkpoint.migration,
        )?;

        let mut demography_rngs = DemographyRngs::new(rng_factory);
        demography_rngs.restore_positions([
            checkpoint.rng.demography_mortality,
            checkpoint.rng.demography_fertility,
            checkpoint.rng.demography_parentage,
            checkpoint.rng.demography_newborn_sex,
        ]);
        let mut resource_rngs = ResourceRngs::new(rng_factory);
        resource_rngs.restore_position(checkpoint.rng.resource_scarcity_mortality);
        let mut migration_rngs = MigrationRngs::new(rng_factory);
        migration_rngs.restore_positions([
            checkpoint.rng.migration_choice,
            checkpoint.rng.migration_uncertainty,
        ]);

        let simulation = Self {
            config: checkpoint.experiment,
            time: checkpoint.time,
            terminal_stop_reason: checkpoint.terminal_stop_reason,
            resume_lineage,
            world,
            population: checkpoint.population,
            temporary_mobility: checkpoint.temporary_mobility,
            resources: checkpoint.resources,
            migration,
            demography_rngs,
            resource_rngs,
            migration_rngs,
            events: checkpoint.events,
            metrics: checkpoint.metrics,
        };
        let actual_digest = simulation.state_digest64();
        if actual_digest != source_state_digest64 {
            return Err(SimulationError::CheckpointStateDigestMismatch {
                expected: source_state_digest64,
                actual: actual_digest,
            });
        }
        Ok(simulation)
    }

    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.time
    }

    #[must_use]
    pub const fn config(&self) -> &ExperimentConfig {
        &self.config
    }

    #[must_use]
    pub const fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub const fn population(&self) -> &Population {
        &self.population
    }

    #[must_use]
    pub const fn temporary_mobility(&self) -> &TemporaryMobilityState {
        &self.temporary_mobility
    }

    #[must_use]
    pub const fn resources(&self) -> &ResourceSystem {
        &self.resources
    }

    #[must_use]
    pub const fn migration(&self) -> &MigrationSystem {
        &self.migration
    }

    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.events
    }

    #[must_use]
    pub const fn metrics(&self) -> &MetricSeries {
        &self.metrics
    }

    /// Run the configured lifecycle while preserving the legacy manifest-only API.
    pub fn run(self) -> Result<RunManifest, SimulationError> {
        Ok(self.run_recorded()?.manifest)
    }

    /// Run to the configured duration or an earlier model stop and retain all M5 artifacts.
    pub fn run_recorded(mut self) -> Result<RecordedRun, SimulationError> {
        let target_year = self.config.duration_years;
        let stop_reason = self
            .advance_to_year(target_year)?
            .unwrap_or(StopReason::DurationReached);
        self.terminal_stop_reason = Some(stop_reason);
        self.ensure_terminal_metric_snapshot();
        self.validate_state()?;
        let manifest = self.build_manifest(stop_reason);
        let checkpoint = self.into_checkpoint();
        Ok(RecordedRun {
            manifest,
            checkpoint,
        })
    }

    /// Advance to a completed annual boundary and return a resumable deterministic checkpoint.
    pub fn checkpoint_at_year(
        mut self,
        target_year: u64,
    ) -> Result<SimulationCheckpoint, SimulationError> {
        let current_year = self.completed_years()?;
        if target_year < current_year || target_year > self.config.duration_years {
            return Err(SimulationError::InvalidCheckpointTarget {
                current_year,
                target_year,
                duration_years: self.config.duration_years,
            });
        }
        if let Some(stop_reason) = self.advance_to_year(target_year)? {
            let expected_day = target_year.saturating_mul(DAYS_PER_YEAR);
            if self.time.days() != expected_day {
                return Err(SimulationError::CheckpointTargetUnreachable {
                    target_year,
                    stop_reason,
                    stopped_day: self.time.days(),
                });
            }
        }
        self.ensure_terminal_metric_snapshot();
        self.validate_state()?;
        Ok(self.into_checkpoint())
    }

    fn advance_to_year(&mut self, target_year: u64) -> Result<Option<StopReason>, SimulationError> {
        if let Some(stop_reason) = self.terminal_stop_reason {
            return Ok(Some(stop_reason));
        }

        let current_year = self.completed_years()?;
        if self.population.living_count() == 0 {
            self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
            self.record_metric_snapshot();
            return Ok(self.terminal_stop_reason);
        }

        for year in current_year.saturating_add(1)..=target_year {
            let periods = u64::from(self.config.resources.periods_per_year);
            let year_start_day = (year - 1).saturating_mul(DAYS_PER_YEAR);

            for period_index in 0..self.config.resources.periods_per_year {
                let period_number = u64::from(period_index) + 1;
                let day = year_start_day
                    .saturating_add(period_number.saturating_mul(DAYS_PER_YEAR) / periods);
                self.time = SimTime::from_days(day);
                let outcome = self.resources.process_period_recorded(
                    &mut self.population,
                    &ResourcePeriodContext {
                        world: &self.world,
                        config: &self.config.resources,
                        period_index_in_year: period_index,
                        day,
                    },
                    &mut self.resource_rngs.scarcity_mortality,
                    &mut self.events,
                )?;
                self.temporary_mobility
                    .reconcile_after_population_change(&self.population);
                if outcome == ResourceStepOutcome::PopulationExtinct {
                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
                    self.record_metric_snapshot();
                    return Ok(self.terminal_stop_reason);
                }
                self.migration.process_boundary_recorded_with_presence(
                    &mut self.population,
                    &MigrationBoundaryContext {
                        world: &self.world,
                        resources: &self.resources,
                        migration: &self.config.migration,
                        annual_food_need: self.config.resources.annual_need_units_per_person,
                        resource_periods_per_year: self.config.resources.periods_per_year,
                        day,
                    },
                    &mut self.migration_rngs,
                    &mut self.events,
                    Some(&self.temporary_mobility),
                )?;
            }

            self.time = SimTime::from_years(year);
            let outcome = process_demographic_year_recorded(
                &mut self.population,
                &self.world,
                &self.config.demography,
                self.time.days(),
                &mut self.demography_rngs,
                &mut self.events,
            )?;
            self.temporary_mobility
                .reconcile_after_population_change(&self.population);
            self.record_metric_snapshot();

            match outcome {
                DemographyStepOutcome::Continue => {}
                DemographyStepOutcome::PopulationExtinct => {
                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
                    return Ok(self.terminal_stop_reason);
                }
                DemographyStepOutcome::PersonRecordLimitReached => {
                    self.terminal_stop_reason = Some(StopReason::PersonRecordLimitReached);
                    return Ok(self.terminal_stop_reason);
                }
            }
        }
        Ok(None)
    }

    fn completed_years(&self) -> Result<u64, SimulationError> {
        if !self.time.days().is_multiple_of(DAYS_PER_YEAR) {
            return Err(SimulationError::UnsupportedCheckpointBoundary {
                day: self.time.days(),
            });
        }
        Ok(self.time.days() / DAYS_PER_YEAR)
    }

    fn record_metric_snapshot(&mut self) {
        let population = self.population.summary();
        let resources = self.resources.summary(&self.population);
        let migration = self.migration.summary();
        let snapshot = MetricSnapshot {
            schema_version: MetricSnapshot::CURRENT_SCHEMA_VERSION,
            day: self.time.days(),
            provenance: MetricProvenance::Derived,
            population: PopulationMetrics::from(&population),
            resources: ResourceMetrics::from(&resources),
            migration: MigrationMetrics::from(&migration),
            state_digest64: self.state_digest64(),
        };
        if self
            .metrics
            .snapshots
            .last()
            .is_some_and(|last| last.day == snapshot.day)
        {
            let _ = self.metrics.snapshots.pop();
        }
        self.metrics.snapshots.push(snapshot);
    }

    fn ensure_terminal_metric_snapshot(&mut self) {
        if self
            .metrics
            .snapshots
            .last()
            .is_none_or(|snapshot| snapshot.day != self.time.days())
        {
            self.record_metric_snapshot();
        }
    }

    fn state_digest64(&self) -> u64 {
        state_digest64_with_temporary_mobility(
            self.time.days(),
            self.world.digest64(),
            self.population.digest64(),
            self.resources.digest64(),
            self.migration.digest64(),
            &self.temporary_mobility,
        )
    }

    fn validate_state(&self) -> Result<(), SimulationError> {
        self.population
            .validate(&self.world)
            .map_err(PopulationError::from)?;
        self.temporary_mobility
            .validate(&self.population, &self.world)?;
        self.resources
            .validate_checkpoint_state(&self.world, &self.config.resources)?;
        Ok(())
    }

    fn rng_checkpoint(&self) -> RngCheckpoint {
        let demography = self.demography_rngs.positions();
        let migration = self.migration_rngs.positions();
        RngCheckpoint {
            demography_mortality: demography[0],
            demography_fertility: demography[1],
            demography_parentage: demography[2],
            demography_newborn_sex: demography[3],
            resource_scarcity_mortality: self.resource_rngs.position(),
            migration_choice: migration[0],
            migration_uncertainty: migration[1],
        }
    }

    fn build_manifest(&self, stop_reason: StopReason) -> RunManifest {
        let resources = self.resources.summary(&self.population);
        let migration = self.migration.summary();
        RunManifest {
            schema_version: RunManifest::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            model_semantics_id: MODEL_SEMANTICS_ID.to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            resume_lineage: self.resume_lineage.clone(),
            experiment: self.config.clone(),
            artifact_schemas: ArtifactSchemas::current(),
            world: self.world.summary(),
            population: self.population.summary(),
            resources: resources.clone(),
            migration: migration.clone(),
            state_digest64: self.state_digest64(),
            statistics: RunStatistics {
                simulated_days: self.time.days(),
                authoritative_event_count: u64::try_from(self.events.len()).unwrap_or(u64::MAX),
                metric_snapshot_count: u64::try_from(self.metrics.len()).unwrap_or(u64::MAX),
                resource_periods_processed: resources.periods_processed,
                migration_decision_boundaries: migration.decision_boundaries,
            },
            start_time: SimTime::ZERO,
            end_time: self.time,
            stop_reason,
        }
    }

    fn into_checkpoint(self) -> SimulationCheckpoint {
        let state_digest = self.state_digest64();
        let rng = self.rng_checkpoint();
        let completed_years = self.time.days() / DAYS_PER_YEAR;
        SimulationCheckpoint {
            schema_version: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            model_semantics_id: MODEL_SEMANTICS_ID.to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            resume_lineage: self.resume_lineage,
            experiment: self.config,
            time: self.time,
            completed_years,
            terminal_stop_reason: self.terminal_stop_reason,
            world_digest64: self.world.digest64(),
            population: self.population,
            temporary_mobility: self.temporary_mobility,
            resources: self.resources,
            migration: self.migration.checkpoint_state(),
            rng,
            events: self.events,
            metrics: self.metrics,
            state_digest64: state_digest,
        }
    }
}

fn validate_experiment(config: &ExperimentConfig) -> Result<(), SimulationError> {
    if config.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return Err(SimulationError::UnsupportedExperimentSchema {
            found: config.schema_version,
            supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    validate_demography_config(&config.demography)?;
    validate_resource_config(&config.resources)?;
    validate_migration_config(&config.migration)?;
    if let Some(evidence) = &config.evidence {
        evidence.validate()?;
    }
    Ok(())
}

fn validate_terminal_checkpoint_state(
    checkpoint: &SimulationCheckpoint,
) -> Result<(), SimulationError> {
    let Some(stop_reason) = checkpoint.terminal_stop_reason else {
        return Ok(());
    };

    let matches_state = match stop_reason {
        StopReason::DurationReached => {
            checkpoint.completed_years == checkpoint.experiment.duration_years
        }
        StopReason::PopulationExtinct => checkpoint.population.living_count() == 0,
        StopReason::PersonRecordLimitReached => {
            checkpoint.population.summary().person_records
                == checkpoint.experiment.population.max_person_records
        }
    };
    if !matches_state {
        return Err(SimulationError::CheckpointTerminalStateMismatch { stop_reason });
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("experiment schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedExperimentSchema { found: u32, supported: u32 },
    #[error("checkpoint schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedCheckpointSchema { found: u32, supported: u32 },
    #[error("checkpoint model version {found} does not match current model version {expected}")]
    CheckpointModelVersionMismatch { found: String, expected: String },
    #[error(
        "checkpoint model semantics identity {found} does not match current model semantics identity {expected}"
    )]
    CheckpointModelSemanticsMismatch { found: String, expected: String },
    #[error("checkpoint resume lineage is invalid: {reason}")]
    CheckpointResumeLineageInvalid { reason: String },
    #[error("checkpoint {artifact} artifact schema is incompatible with this build")]
    CheckpointArtifactSchemaMismatch { artifact: &'static str },
    #[error("checkpoint day {day} is not a completed annual boundary")]
    UnsupportedCheckpointBoundary { day: u64 },
    #[error(
        "checkpoint completed year {completed_years} exceeds experiment duration {duration_years}"
    )]
    CheckpointBeyondDuration {
        completed_years: u64,
        duration_years: u64,
    },
    #[error("checkpoint terminal stop reason {stop_reason:?} does not match checkpoint state")]
    CheckpointTerminalStateMismatch { stop_reason: StopReason },
    #[error("checkpoint world digest mismatch: expected {expected}, reconstructed {actual}")]
    CheckpointWorldDigestMismatch { expected: u64, actual: u64 },
    #[error("checkpoint state digest mismatch: expected {expected}, reconstructed {actual}")]
    CheckpointStateDigestMismatch { expected: u64, actual: u64 },
    #[error(
        "checkpoint target year {target_year} is outside current year {current_year}..={duration_years}"
    )]
    InvalidCheckpointTarget {
        current_year: u64,
        target_year: u64,
        duration_years: u64,
    },
    #[error(
        "checkpoint target year {target_year} was not reached: stopped at day {stopped_day} because {stop_reason:?}"
    )]
    CheckpointTargetUnreachable {
        target_year: u64,
        stop_reason: StopReason,
        stopped_day: u64,
    },
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
    #[error(transparent)]
    Demography(#[from] DemographyConfigError),
    #[error(transparent)]
    ResourceConfig(#[from] ResourceConfigError),
    #[error(transparent)]
    Resources(#[from] ResourceError),
    #[error(transparent)]
    MigrationConfig(#[from] MigrationConfigError),
    #[error(transparent)]
    Migration(#[from] MigrationError),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error(transparent)]
    TemporaryMobility(#[from] TemporaryMobilityValidationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EvidenceCatalog, ParameterEvidenceLink,
        config::{
            DemographyConfig, MigrationConfig, PROBABILITY_PER_MILLION, PopulationConfig,
            ResourceConfig, WorldConfig,
        },
        events::EventProvenance,
    };

    fn no_event_demography() -> DemographyConfig {
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }
        config
    }

    fn no_pressure_resources() -> ResourceConfig {
        let mut config = ResourceConfig::synthetic_validation_v1();
        config.annual_need_units_per_person = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        config
    }

    fn disabled_migration() -> MigrationConfig {
        MigrationConfig::synthetic_validation_v1().with_enabled(false)
    }

    fn record_limit_config(seed: u64) -> ExperimentConfig {
        let mut demography = no_event_demography();
        for band in &mut demography.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        demography.minimum_birth_spacing_days = 0;
        demography.male_parent_min_age_years = 0;
        demography.male_parent_max_age_years_exclusive = 100;

        ExperimentConfig::new(seed, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100).with_max_person_records(101))
            .with_demography(demography)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration())
    }

    fn dangling_evidence_config(seed: u64) -> ExperimentConfig {
        ExperimentConfig::new(seed, 1).with_evidence(
            EvidenceCatalog::new(Vec::new()).with_parameter_links(vec![ParameterEvidenceLink {
                parameter_path: "resources.annualNeedUnitsPerPerson".to_owned(),
                evidence_id: "missing".to_owned(),
                note: None,
            }]),
        )
    }

    #[test]
    fn malformed_evidence_is_rejected_by_core_construction() {
        let mut catalog = EvidenceCatalog::new(Vec::new());
        catalog.schema_version = 999;
        let config = ExperimentConfig::new(6, 1).with_evidence(catalog);
        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::Evidence(
                EvidenceError::UnsupportedCatalogSchema { .. }
            ))
        ));
    }

    #[test]
    fn dangling_evidence_reference_is_rejected_by_core_construction() {
        assert!(matches!(
            Simulation::new(dangling_evidence_config(7)),
            Err(SimulationError::Evidence(
                EvidenceError::UnknownEvidenceReference { evidence_id }
            )) if evidence_id == "missing"
        ));
    }

    #[test]
    fn checkpoint_resume_revalidates_embedded_evidence() {
        let mut checkpoint = Simulation::new(ExperimentConfig::new(8, 2))
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        checkpoint.experiment = dangling_evidence_config(8).with_world(checkpoint.experiment.world);
        assert!(matches!(
            Simulation::from_checkpoint(checkpoint),
            Err(SimulationError::Evidence(
                EvidenceError::UnknownEvidenceReference { evidence_id }
            )) if evidence_id == "missing"
        ));
    }

    #[test]
    fn run_reaches_configured_duration_when_no_stop_condition_occurs() {
        let config = ExperimentConfig::new(7, 10)
            .with_demography(no_event_demography())
            .with_resources(no_pressure_resources());
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.model_semantics_id, MODEL_SEMANTICS_ID);
        assert_eq!(manifest.end_time, SimTime::from_years(10));
        assert_eq!(manifest.stop_reason, StopReason::DurationReached);
        assert_eq!(manifest.world.cell_count, 128 * 128);
        assert_eq!(manifest.population.initial_population, 10_000);
        assert_eq!(manifest.population.living_population, 10_000);
        assert_eq!(manifest.population.births_since_start, 0);
        assert_eq!(manifest.population.deaths_since_start, 0);
        assert_eq!(manifest.resources.periods_processed, 40);
        assert_eq!(manifest.resources.unmet_need, 0);
        assert_eq!(manifest.migration.moves_completed, 0);
        assert!(manifest.resume_lineage.boundaries.is_empty());
    }

    #[test]
    fn default_schedule_produces_resource_demographic_and_migration_state() {
        let config = ExperimentConfig::new(81, 5)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(2_000).with_max_person_records(100_000));
        let run = Simulation::new(config).unwrap().run_recorded().unwrap();
        let manifest = &run.manifest;

        assert!(manifest.population.deaths_since_start > 0);
        assert!(manifest.resources.periods_processed > 0);
        assert!(manifest.resources.harvested_food > 0 || manifest.resources.unmet_need > 0);
        assert!(manifest.migration.decision_boundaries > 0);
        assert_eq!(
            u64::from(manifest.population.initial_population)
                + manifest.population.births_since_start
                - manifest.population.deaths_since_start,
            manifest.population.living_population
        );
        assert!(
            run.events()
                .events
                .iter()
                .all(|event| { event.provenance == EventProvenance::Authoritative })
        );
    }

    #[test]
    fn checkpoint_resume_matches_uninterrupted_execution() {
        let config = ExperimentConfig::new(2026, 12)
            .with_world(WorldConfig::new(32, 32))
            .with_population(PopulationConfig::new(2_000).with_max_person_records(100_000));

        let uninterrupted = Simulation::new(config.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        let checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(5)
            .unwrap();
        assert_eq!(checkpoint.model_semantics_id, MODEL_SEMANTICS_ID);
        assert_eq!(checkpoint.terminal_stop_reason, None);
        let source_digest = checkpoint.state_digest64;
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();

        let mut resumed_manifest_without_lineage = resumed.manifest.clone();
        resumed_manifest_without_lineage.resume_lineage = ResumeLineage::new();
        assert_eq!(resumed_manifest_without_lineage, uninterrupted.manifest);
        assert_eq!(resumed.manifest.resume_lineage.boundaries.len(), 1);
        let boundary = &resumed.manifest.resume_lineage.boundaries[0];
        assert_eq!(boundary.boundary_day, SimTime::from_years(5).days());
        assert_eq!(boundary.boundary_completed_years, 5);
        assert_eq!(boundary.source_state_digest64, source_digest);
        assert_eq!(boundary.source, boundary.continuation);
        assert_eq!(
            resumed.checkpoint.population,
            uninterrupted.checkpoint.population
        );
        assert_eq!(
            resumed.checkpoint.temporary_mobility,
            uninterrupted.checkpoint.temporary_mobility
        );
        assert_eq!(
            resumed.checkpoint.resources,
            uninterrupted.checkpoint.resources
        );
        assert_eq!(
            resumed.checkpoint.migration,
            uninterrupted.checkpoint.migration
        );
        assert_eq!(resumed.checkpoint.events, uninterrupted.checkpoint.events);
        assert_eq!(resumed.checkpoint.metrics, uninterrupted.checkpoint.metrics);
        assert_eq!(
            resumed.checkpoint.state_digest64,
            uninterrupted.checkpoint.state_digest64
        );
        assert_eq!(
            resumed.checkpoint.terminal_stop_reason,
            uninterrupted.checkpoint.terminal_stop_reason
        );
        assert_eq!(
            resumed.manifest.resume_lineage,
            resumed.checkpoint.resume_lineage
        );
    }

    #[test]
    fn checkpoint_rejects_incompatible_model_semantics() {
        let config = ExperimentConfig::new(2034, 4)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(32));
        let mut checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        checkpoint.model_semantics_id = "anthrosim-model-semantics-incompatible-test".to_owned();

        assert!(matches!(
            Simulation::from_checkpoint(checkpoint),
            Err(SimulationError::CheckpointModelSemanticsMismatch { found, expected })
                if found == "anthrosim-model-semantics-incompatible-test"
                    && expected == MODEL_SEMANTICS_ID
        ));
    }

    #[test]
    fn checkpoint_source_revision_is_provenance_not_resume_compatibility() {
        let config = ExperimentConfig::new(2035, 4)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(32));
        let mut checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let boundary_day = checkpoint.time.days();
        let source_state_digest64 = checkpoint.state_digest64;
        checkpoint.git_commit = Some("source-neutral-revision-test".to_owned());

        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();
        let boundary = &resumed.manifest.resume_lineage.boundaries[0];
        assert_eq!(
            boundary.source.git_commit.as_deref(),
            Some("source-neutral-revision-test")
        );
        assert_eq!(boundary.continuation, SourceRevisionIdentity::current());
        assert_eq!(boundary.boundary_day, boundary_day);
        assert_eq!(boundary.source_state_digest64, source_state_digest64);
        assert_eq!(
            resumed.manifest.git_commit,
            boundary.continuation.git_commit
        );
    }

    #[test]
    fn multi_stage_resume_preserves_full_source_lineage() {
        let config = ExperimentConfig::new(2036, 5)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(64));
        let first = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let first_digest = first.state_digest64;
        let second = Simulation::from_checkpoint(first)
            .unwrap()
            .checkpoint_at_year(3)
            .unwrap();
        let second_digest = second.state_digest64;
        assert_eq!(second.resume_lineage.boundaries.len(), 1);

        let final_run = Simulation::from_checkpoint(second)
            .unwrap()
            .run_recorded()
            .unwrap();
        assert_eq!(final_run.manifest.resume_lineage.boundaries.len(), 2);
        assert_eq!(
            final_run.manifest.resume_lineage.boundaries[0].source_state_digest64,
            first_digest
        );
        assert_eq!(
            final_run.manifest.resume_lineage.boundaries[1].source_state_digest64,
            second_digest
        );
        assert_eq!(
            final_run.manifest.resume_lineage.boundaries[0].continuation,
            final_run.manifest.resume_lineage.boundaries[1].source
        );
        assert_eq!(
            final_run.manifest.resume_lineage,
            final_run.checkpoint.resume_lineage
        );
    }

    #[test]
    fn tampered_resume_lineage_is_rejected() {
        let config = ExperimentConfig::new(2037, 4)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(32));
        let first = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let mut second = Simulation::from_checkpoint(first)
            .unwrap()
            .checkpoint_at_year(2)
            .unwrap();
        second.resume_lineage.boundaries[0].continuation.git_commit =
            Some("tampered-continuation".to_owned());

        assert!(matches!(
            Simulation::from_checkpoint(second),
            Err(SimulationError::CheckpointResumeLineageInvalid { .. })
        ));
    }

    #[test]
    fn pre_temporary_mobility_checkpoint_schemas_are_rejected_fail_closed() {
        let config = ExperimentConfig::new(2038, 3)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(32));
        let checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();

        for schema_version in [
            SimulationCheckpoint::PRE_LINEAGE_SCHEMA_VERSION,
            SimulationCheckpoint::PRE_TEMPORARY_MOBILITY_SCHEMA_VERSION,
        ] {
            let mut old = checkpoint.clone();
            old.schema_version = schema_version;
            assert!(matches!(
                Simulation::from_checkpoint(old),
                Err(SimulationError::UnsupportedCheckpointSchema { found, supported })
                    if found == schema_version
                        && supported == SimulationCheckpoint::CURRENT_SCHEMA_VERSION
            ));
        }
    }

    #[test]
    fn final_derived_metrics_reconcile_with_authoritative_state() {
        let run = Simulation::new(ExperimentConfig::new(404, 4))
            .unwrap()
            .run_recorded()
            .unwrap();
        let final_metrics = run.metrics().snapshots.last().unwrap();
        assert_eq!(
            final_metrics.population.living_population,
            run.manifest.population.living_population
        );
        assert_eq!(
            final_metrics.population.person_records,
            run.manifest.population.person_records
        );
        assert_eq!(
            final_metrics.resources.final_food_stock,
            run.manifest.resources.final_food_stock
        );
        assert_eq!(
            final_metrics.resources.unmet_need,
            run.manifest.resources.unmet_need
        );
        assert_eq!(
            final_metrics.migration.moves_completed,
            run.manifest.migration.moves_completed
        );
        assert_eq!(final_metrics.state_digest64, run.manifest.state_digest64);
    }

    #[test]
    fn checkpoint_state_digest_detects_tampering() {
        let config = ExperimentConfig::new(505, 6)
            .with_world(WorldConfig::new(16, 16))
            .with_population(PopulationConfig::new(500));
        let mut checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(3)
            .unwrap();
        checkpoint.state_digest64 ^= 1;
        assert!(matches!(
            Simulation::from_checkpoint(checkpoint),
            Err(SimulationError::CheckpointStateDigestMismatch { .. })
        ));
    }

    #[test]
    fn certain_demographic_mortality_records_population_extinction() {
        let mut demography = no_event_demography();
        for band in &mut demography.mortality_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        let config = ExperimentConfig::new(91, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100))
            .with_demography(demography)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration());

        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.population.living_population, 0);
        assert_eq!(manifest.population.deaths_since_start, 100);
    }

    #[test]
    fn record_limit_is_an_explicit_operational_stop() {
        let run = Simulation::new(record_limit_config(101))
            .unwrap()
            .run_recorded()
            .unwrap();

        assert_eq!(
            run.manifest.stop_reason,
            StopReason::PersonRecordLimitReached
        );
        assert_eq!(run.manifest.end_time, SimTime::from_years(1));
        assert_eq!(run.manifest.population.person_records, 101);
        assert_eq!(run.manifest.population.births_since_start, 1);
        assert_eq!(
            run.checkpoint.terminal_stop_reason,
            Some(StopReason::PersonRecordLimitReached)
        );
    }

    #[test]
    fn terminal_record_limit_checkpoint_does_not_advance_when_resumed() {
        let config = record_limit_config(102);
        let checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();

        assert_eq!(checkpoint.time, SimTime::from_years(1));
        assert_eq!(
            checkpoint.terminal_stop_reason,
            Some(StopReason::PersonRecordLimitReached)
        );

        let resumed = Simulation::from_checkpoint(checkpoint.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        assert_eq!(
            resumed.manifest.stop_reason,
            StopReason::PersonRecordLimitReached
        );
        assert_eq!(resumed.manifest.end_time, SimTime::from_years(1));
        assert_eq!(resumed.checkpoint.state_digest64, checkpoint.state_digest64);
        assert_eq!(resumed.checkpoint.population, checkpoint.population);
        assert_eq!(
            resumed.checkpoint.temporary_mobility,
            checkpoint.temporary_mobility
        );
        assert_eq!(resumed.checkpoint.resources, checkpoint.resources);
        assert_eq!(resumed.checkpoint.migration, checkpoint.migration);
        assert_eq!(resumed.checkpoint.events, checkpoint.events);
        assert_eq!(resumed.checkpoint.metrics, checkpoint.metrics);
        assert_eq!(resumed.checkpoint.resume_lineage.boundaries.len(), 1);

        let mut tampered = checkpoint;
        tampered.terminal_stop_reason = Some(StopReason::DurationReached);
        assert!(matches!(
            Simulation::from_checkpoint(tampered),
            Err(SimulationError::CheckpointTerminalStateMismatch {
                stop_reason: StopReason::DurationReached
            })
        ));
    }

    #[test]
    fn severe_resource_scarcity_can_extinguish_before_migration_and_annual_demography() {
        let mut resources = ResourceConfig::synthetic_validation_v1()
            .with_productivity_scale_permille(0)
            .with_annual_need_units_per_person(100);
        resources.periods_per_year = 1;
        resources.max_condition_loss_per_period = 1_000;
        resources.max_scarcity_mortality_probability_per_million = PROBABILITY_PER_MILLION;
        let config = ExperimentConfig::new(111, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100))
            .with_demography(no_event_demography())
            .with_resources(resources);

        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.resources.scarcity_deaths, 100);
        assert!(manifest.resources.unmet_need > 0);
        assert_eq!(manifest.migration.moves_completed, 0);
    }

    #[test]
    fn empty_initial_population_is_extinct_at_epoch() {
        let config = ExperimentConfig::new(121, 10).with_population(PopulationConfig::new(0));
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::ZERO);
        assert_eq!(manifest.resources.periods_processed, 0);
        assert_eq!(manifest.migration.decision_boundaries, 0);
        assert_eq!(manifest.statistics.metric_snapshot_count, 1);
    }

    #[test]
    fn rejects_unsupported_experiment_schema() {
        let mut config = ExperimentConfig::new(7, 100);
        config.schema_version = 999;
        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::UnsupportedExperimentSchema { .. })
        ));
    }
}
