use thiserror::Error;

use crate::{
    checkpoint::{
        RngCheckpoint, SimulationCheckpoint, continuation_digest64,
        state_digest64_with_temporary_mobility,
    },
    config::{ExperimentConfig, PopulationInitialization},
    demography::{
        DemographyConfigError, DemographyRngs, DemographyStepOutcome,
        process_demographic_year_recorded_with_founder_history, validate_demography_config,
    },
    events::EventLog,
    evidence::EvidenceError,
    founder_initialization::FounderGenealogyStatus,
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
    temporary_mobility::{
        TemporaryMobilityConfigError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,
        TemporaryMobilityProgramError, TemporaryMobilityState, TemporaryMobilityValidationError,
    },
    time::{DAYS_PER_YEAR, MAX_SUPPORTED_DURATION_YEARS, SimTime},
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
        Self::new_internal(config, None)
    }

    #[cfg(test)]
    pub(crate) fn new_with_temporary_mobility(
        config: ExperimentConfig,
        program: TemporaryMobilityProgram,
    ) -> Result<Self, SimulationError> {
        Self::new_internal(config, Some(program))
    }

    fn new_internal(
        config: ExperimentConfig,
        program: Option<TemporaryMobilityProgram>,
    ) -> Result<Self, SimulationError> {
        validate_experiment(&config)?;

        let rng_factory = RngFactory::new(config.seed);
        let world = World::generate(config.world, rng_factory)?;
        let population = match config.population.initialization {
            PopulationInitialization::SyntheticValidationV1 => {
                Population::initialize(config.population, &world, rng_factory)?
            }
            PopulationInitialization::DeclaredFounderStateV1 => {
                let definition = config
                    .founder_population
                    .as_ref()
                    .ok_or(SimulationError::MissingFounderPopulationDefinition)?;
                Population::initialize_declared_founder_state_v1(
                    config.population,
                    definition,
                    &world,
                )?
            }
        };
        let configured_program = config
            .temporary_mobility
            .as_ref()
            .map(|definition| definition.derive_program(&world))
            .transpose()?;
        let temporary_mobility = match (program, configured_program) {
            (Some(_), Some(_)) => {
                return Err(SimulationError::AmbiguousTemporaryMobilityConfiguration);
            }
            (Some(program), None) | (None, Some(program)) => {
                TemporaryMobilityState::with_program(&population, program, &world)?
            }
            (None, None) => TemporaryMobilityState::at_residence(&population),
        };
        temporary_mobility.validate_at_day(0, &population, &world)?;
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
        let actual_continuation_digest64 = continuation_digest64(&checkpoint);
        if actual_continuation_digest64 != checkpoint.continuation_digest64 {
            return Err(SimulationError::CheckpointContinuationDigestMismatch {
                expected: checkpoint.continuation_digest64,
                actual: actual_continuation_digest64,
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
        validate_founder_population_against_world(&checkpoint.experiment, &world)?;
        validate_configured_temporary_mobility(
            &checkpoint.experiment,
            &checkpoint.temporary_mobility,
            &world,
        )?;

        checkpoint
            .population
            .validate(&world)
            .map_err(PopulationError::from)?;
        checkpoint.temporary_mobility.validate_at_day(
            checkpoint.time.days(),
            &checkpoint.population,
            &world,
        )?;
        checkpoint
            .resources
            .validate_checkpoint_state(&world, &checkpoint.experiment.resources)?;
        validate_terminal_checkpoint_state(&checkpoint)?;

        let boundary_day = checkpoint.time.days();
        let boundary_completed_years = checkpoint.completed_years;
        let source_state_digest64 = checkpoint.state_digest64;
        let source_continuation_digest64 = checkpoint.continuation_digest64;
        let continuation_identity = SourceRevisionIdentity::current();
        let mut resume_lineage = checkpoint.resume_lineage;
        resume_lineage.boundaries.push(ResumeBoundary {
            source: source_identity,
            continuation: continuation_identity,
            boundary_day,
            boundary_completed_years,
            source_state_digest64,
            source_continuation_digest64,
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

    pub fn run(self) -> Result<RunManifest, SimulationError> {
        Ok(self.run_recorded()?.manifest)
    }

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
            let year_start_day = (year - 1).saturating_mul(DAYS_PER_YEAR);
            let mut resource_index = 0_u16;
            let mut migration_index = 0_u16;

            loop {
                let resource_day = fixed_schedule_boundary_day(
                    year_start_day,
                    resource_index,
                    self.config.resources.periods_per_year,
                );
                let migration_day = if self.config.migration.enabled {
                    fixed_schedule_boundary_day(
                        year_start_day,
                        migration_index,
                        self.config.migration.decision_periods_per_year,
                    )
                } else {
                    None
                };
                let next_day = match (resource_day, migration_day) {
                    (Some(resource_day), Some(migration_day)) => {
                        Some(resource_day.min(migration_day))
                    }
                    (Some(resource_day), None) => Some(resource_day),
                    (None, Some(migration_day)) => Some(migration_day),
                    (None, None) => None,
                };
                let Some(day) = next_day else {
                    break;
                };

                self.process_temporary_boundaries_before(day)?;
                self.time = SimTime::from_days(day);

                if resource_day == Some(day) {
                    let temporary_resource_period = self
                        .temporary_mobility
                        .resource_period_snapshot(day, &self.world)?;
                    let outcome = self.resources.process_period_recorded_with_presence(
                        &mut self.population,
                        &ResourcePeriodContext {
                            world: &self.world,
                            config: &self.config.resources,
                            period_index_in_year: resource_index,
                            day,
                        },
                        &mut self.resource_rngs.scarcity_mortality,
                        &mut self.events,
                        temporary_resource_period.as_ref(),
                    )?;
                    resource_index = resource_index.saturating_add(1);
                    self.temporary_mobility.complete_resource_period(day)?;
                    self.temporary_mobility
                        .reconcile_after_population_change(&self.population);
                    if outcome == ResourceStepOutcome::PopulationExtinct {
                        self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
                        self.record_metric_snapshot();
                        return Ok(self.terminal_stop_reason);
                    }
                }

                self.temporary_mobility.process_day(
                    day,
                    &self.population,
                    &self.world,
                    &mut self.events,
                )?;

                if migration_day == Some(day) {
                    self.migration.process_boundary_recorded_with_presence(
                        &mut self.population,
                        &MigrationBoundaryContext {
                            world: &self.world,
                            resources: &self.resources,
                            migration: &self.config.migration,
                            annual_food_need: self.config.resources.annual_need_units_per_person,
                            decision_periods_per_year: self
                                .config
                                .migration
                                .decision_periods_per_year,
                            decision_index_in_year: migration_index,
                            day,
                        },
                        &mut self.migration_rngs,
                        &mut self.events,
                        Some(&self.temporary_mobility),
                    )?;
                    migration_index = migration_index.saturating_add(1);
                }
            }

            self.time = SimTime::from_years(year);
            let outcome = process_demographic_year_recorded_with_founder_history(
                &mut self.population,
                &self.world,
                &self.config.demography,
                self.time.days(),
                &mut self.demography_rngs,
                &mut self.events,
                self.config.founder_population.as_ref(),
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

    fn process_temporary_boundaries_before(
        &mut self,
        fixed_day: u64,
    ) -> Result<(), SimulationError> {
        let Some(end_day) = fixed_day.checked_sub(1) else {
            return Ok(());
        };
        loop {
            let current_day = self.time.days();
            let Some(day) = self.temporary_mobility.next_boundary_day(
                current_day,
                end_day,
                &self.population,
            )?
            else {
                break;
            };
            self.time = SimTime::from_days(day);
            self.temporary_mobility.process_day(
                day,
                &self.population,
                &self.world,
                &mut self.events,
            )?;
        }
        Ok(())
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
            .validate_at_day(self.time.days(), &self.population, &self.world)?;
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
            continuation_digest64: 0,
            state_digest64: state_digest,
        }
        .seal_continuation_identity()
    }
}

fn fixed_schedule_boundary_day(year_start_day: u64, index: u16, periods: u16) -> Option<u64> {
    if periods == 0 || index >= periods {
        return None;
    }
    let offset = (u64::from(index) + 1)
        .checked_mul(DAYS_PER_YEAR)?
        .checked_div(u64::from(periods))?;
    year_start_day.checked_add(offset)
}

fn validate_experiment(config: &ExperimentConfig) -> Result<(), SimulationError> {
    if config.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return Err(SimulationError::UnsupportedExperimentSchema {
            found: config.schema_version,
            supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.duration_years > MAX_SUPPORTED_DURATION_YEARS {
        return Err(SimulationError::DurationOutOfRange {
            duration_years: config.duration_years,
            maximum_years: MAX_SUPPORTED_DURATION_YEARS,
        });
    }
    validate_founder_population_binding(config)?;
    validate_demography_config(&config.demography)?;
    validate_resource_config(&config.resources)?;
    validate_migration_config(&config.migration)?;
    if let Some(temporary_mobility) = &config.temporary_mobility {
        temporary_mobility.validate_evidence_context(config.evidence.as_ref())?;
    }
    if let Some(evidence) = &config.evidence {
        evidence.validate_against_experiment(config)?;
    }
    Ok(())
}

fn validate_founder_population_binding(config: &ExperimentConfig) -> Result<(), SimulationError> {
    match (
        config.population.initialization,
        config.founder_population.as_ref(),
    ) {
        (PopulationInitialization::SyntheticValidationV1, None) => Ok(()),
        (PopulationInitialization::SyntheticValidationV1, Some(_)) => {
            Err(SimulationError::UnexpectedFounderPopulationDefinition)
        }
        (PopulationInitialization::DeclaredFounderStateV1, None) => {
            Err(SimulationError::MissingFounderPopulationDefinition)
        }
        (PopulationInitialization::DeclaredFounderStateV1, Some(definition)) => {
            if config.migration.enabled
                && config.migration.kin_weight > 0
                && definition.genealogy_status
                    != FounderGenealogyStatus::CompleteLivingDirectParents
            {
                return Err(SimulationError::FounderKinStateUnspecified);
            }
            Ok(())
        }
    }
}

fn validate_founder_population_against_world(
    config: &ExperimentConfig,
    world: &World,
) -> Result<(), SimulationError> {
    if let Some(definition) = &config.founder_population {
        definition
            .validate(
                config.population.initial_population,
                config.population.max_person_records,
                world,
            )
            .map_err(PopulationError::from)?;
    }
    Ok(())
}

fn validate_configured_temporary_mobility(
    config: &ExperimentConfig,
    state: &TemporaryMobilityState,
    world: &World,
) -> Result<(), SimulationError> {
    let Some(definition) = &config.temporary_mobility else {
        if state.is_disabled() {
            return Ok(());
        }
        return Err(SimulationError::ConfiguredTemporaryMobilityMismatch {
            expected: "temporary-mobility-disabled".to_owned(),
            actual: state
                .program()
                .map(TemporaryMobilityProgram::identity)
                .or_else(|| Some("unconfigured-temporary-mobility-state".to_owned())),
        });
    };
    let expected = definition.derive_program(world)?;
    if state.program() != Some(&expected) {
        return Err(SimulationError::ConfiguredTemporaryMobilityMismatch {
            expected: expected.identity(),
            actual: state.program().map(TemporaryMobilityProgram::identity),
        });
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
    #[error(
        "experiment duration {duration_years} years exceeds supported signed chronology limit {maximum_years} years"
    )]
    DurationOutOfRange {
        duration_years: u64,
        maximum_years: u64,
    },
    #[error("declared founder initialization requires founderPopulation in experiment config")]
    MissingFounderPopulationDefinition,
    #[error("synthetic founder initialization cannot carry a founderPopulation definition")]
    UnexpectedFounderPopulationDefinition,
    #[error(
        "declared founder genealogy is unspecified while the active migration model gives kin non-zero weight"
    )]
    FounderKinStateUnspecified,
    #[error("checkpoint schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedCheckpointSchema { found: u32, supported: u32 },
    #[error("checkpoint model version {found} does not match current model version {expected}")]
    CheckpointModelVersionMismatch { found: String, expected: String },
    #[error(
        "checkpoint model semantics identity {found} does not match current model semantics identity {expected}"
    )]
    CheckpointModelSemanticsMismatch { found: String, expected: String },
    #[error("checkpoint continuation digest mismatch: stored {expected}, reconstructed {actual}")]
    CheckpointContinuationDigestMismatch { expected: u64, actual: u64 },
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
    #[error("both ExperimentConfig and an explicit constructor supplied temporary mobility")]
    AmbiguousTemporaryMobilityConfiguration,
    #[error(
        "configured temporary-mobility program mismatch: expected {expected}, found {actual:?}"
    )]
    ConfiguredTemporaryMobilityMismatch {
        expected: String,
        actual: Option<String>,
    },
    #[error(transparent)]
    TemporaryMobilityConfig(#[from] TemporaryMobilityConfigError),
    #[error(transparent)]
    TemporaryMobility(#[from] TemporaryMobilityValidationError),
    #[error(transparent)]
    TemporaryMobilityProgram(#[from] TemporaryMobilityProgramError),
    #[error(transparent)]
    TemporaryMobilityExecution(#[from] TemporaryMobilityExecutionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EvidenceCatalog, ParameterEvidenceLink,
        config::{
            DemographyConfig, MigrationConfig, PROBABILITY_PER_MILLION, ParameterProvenance,
            PopulationConfig, PopulationInitialization, ResourceConfig, WorldConfig,
        },
        events::EventProvenance,
        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
        },
        ids::{CellId, HouseholdId, PersonId},
        population::ReproductiveSex,
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

    fn declared_founder_definition(
        genealogy_status: FounderGenealogyStatus,
        last_birth_day: Option<i64>,
    ) -> FounderPopulationDefinition {
        FounderPopulationDefinition::new(
            "simulation-declared-founder-test-v1",
            ParameterProvenance::SyntheticValidation,
            genealogy_status,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -(25 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day,
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -(30 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
            ],
        )
    }

    fn declared_config(
        seed: u64,
        duration_years: u64,
        definition: FounderPopulationDefinition,
    ) -> ExperimentConfig {
        ExperimentConfig::new(seed, duration_years)
            .with_world(WorldConfig::new(1, 1))
            .with_population(
                PopulationConfig::new(2)
                    .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
            )
            .with_founder_population(definition)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration())
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

    fn assert_continuation_tamper_rejected(checkpoint: SimulationCheckpoint) {
        assert!(matches!(
            Simulation::from_checkpoint(checkpoint),
            Err(SimulationError::CheckpointContinuationDigestMismatch { .. })
        ));
    }

    #[test]
    fn duration_beyond_signed_chronology_domain_is_rejected_before_execution() {
        let duration_years = MAX_SUPPORTED_DURATION_YEARS + 1;
        assert!(matches!(
            Simulation::new(ExperimentConfig::new(5, duration_years)),
            Err(SimulationError::DurationOutOfRange {
                duration_years: found,
                maximum_years,
            }) if found == duration_years && maximum_years == MAX_SUPPORTED_DURATION_YEARS
        ));
    }

    #[test]
    fn declared_founder_mode_requires_a_definition() {
        let config = ExperimentConfig::new(51, 1).with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
        );
        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::MissingFounderPopulationDefinition)
        ));
    }

    #[test]
    fn synthetic_mode_rejects_an_unbound_founder_definition() {
        let mut config = ExperimentConfig::new(52, 1);
        config.founder_population = Some(declared_founder_definition(
            FounderGenealogyStatus::CompleteLivingDirectParents,
            None,
        ));
        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::UnexpectedFounderPopulationDefinition)
        ));
    }

    #[test]
    fn declared_founder_kin_state_fails_closed_when_unspecified() {
        let definition = declared_founder_definition(FounderGenealogyStatus::Unspecified, None);
        let mut config = ExperimentConfig::new(53, 1)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(2))
            .with_founder_population(definition)
            .with_resources(no_pressure_resources());
        config.migration.enabled = true;
        config.migration.kin_weight = 1;

        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::FounderKinStateUnspecified)
        ));
    }

    #[test]
    fn unspecified_founder_genealogy_is_allowed_when_kin_cannot_affect_behavior() {
        let definition = declared_founder_definition(FounderGenealogyStatus::Unspecified, None);
        let mut no_kin = ExperimentConfig::new(54, 1)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(2))
            .with_founder_population(definition.clone())
            .with_resources(no_pressure_resources());
        no_kin.migration.kin_weight = 0;
        Simulation::new(no_kin).unwrap();

        let disabled = declared_config(55, 1, definition);
        Simulation::new(disabled).unwrap();
    }

    #[test]
    fn declared_pre_run_birth_history_controls_first_boundary_in_full_lifecycle() {
        let mut demography = no_event_demography();
        for band in &mut demography.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        demography.male_parent_min_age_years = 0;
        demography.male_parent_max_age_years_exclusive = 100;

        let recent = declared_config(
            56,
            1,
            declared_founder_definition(FounderGenealogyStatus::Unspecified, Some(-100)),
        )
        .with_demography(demography.clone());
        let recent_run = Simulation::new(recent).unwrap().run_recorded().unwrap();
        assert_eq!(recent_run.manifest.population.births_since_start, 0);

        let distant = declared_config(
            56,
            1,
            declared_founder_definition(FounderGenealogyStatus::Unspecified, Some(-2_000)),
        )
        .with_demography(demography);
        let distant_run = Simulation::new(distant).unwrap().run_recorded().unwrap();
        assert_eq!(distant_run.manifest.population.births_since_start, 1);
    }

    #[test]
    fn declared_founder_history_survives_checkpoint_resume_via_experiment_identity() {
        let definition =
            declared_founder_definition(FounderGenealogyStatus::Unspecified, Some(-100));
        let mut demography = no_event_demography();
        for band in &mut demography.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        demography.male_parent_min_age_years = 0;
        demography.male_parent_max_age_years_exclusive = 100;
        let config = declared_config(57, 4, definition).with_demography(demography);

        let uninterrupted = Simulation::new(config.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        let checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(2)
            .unwrap();
        assert!(checkpoint.experiment.founder_population.is_some());
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();

        assert_eq!(
            resumed.checkpoint.population,
            uninterrupted.checkpoint.population
        );
        assert_eq!(resumed.checkpoint.events, uninterrupted.checkpoint.events);
        assert_eq!(
            resumed.checkpoint.state_digest64,
            uninterrupted.checkpoint.state_digest64
        );
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
        checkpoint = checkpoint.seal_continuation_identity();
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
        assert_eq!(manifest.migration.decision_boundaries, 40);
        assert!(manifest.resume_lineage.boundaries.is_empty());
    }

    #[test]
    fn resource_partition_no_longer_changes_migration_opportunity_count() {
        for resource_periods in [1_u16, 4, 12, 365] {
            let mut resources = no_pressure_resources();
            resources.periods_per_year = resource_periods;
            let migration =
                MigrationConfig::synthetic_validation_v1().with_decision_periods_per_year(4);
            let config = ExperimentConfig::new(70 + u64::from(resource_periods), 2)
                .with_world(WorldConfig::new(4, 4))
                .with_population(PopulationConfig::new(64))
                .with_demography(no_event_demography())
                .with_resources(resources)
                .with_migration(migration);
            let manifest = Simulation::new(config).unwrap().run().unwrap();
            assert_eq!(
                manifest.resources.periods_processed,
                u64::from(resource_periods) * 2
            );
            assert_eq!(manifest.migration.decision_boundaries, 8);
        }
    }

    #[test]
    fn migration_decision_clock_is_explicit_and_independent() {
        for decision_periods in [1_u16, 4, 12] {
            let migration = MigrationConfig::synthetic_validation_v1()
                .with_decision_periods_per_year(decision_periods);
            let config = ExperimentConfig::new(90 + u64::from(decision_periods), 2)
                .with_world(WorldConfig::new(4, 4))
                .with_population(PopulationConfig::new(64))
                .with_demography(no_event_demography())
                .with_resources(no_pressure_resources())
                .with_migration(migration);
            let manifest = Simulation::new(config).unwrap().run().unwrap();
            assert_eq!(manifest.resources.periods_processed, 8);
            assert_eq!(
                manifest.migration.decision_boundaries,
                u64::from(decision_periods) * 2
            );
        }
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
                .all(|event| event.provenance == EventProvenance::Authoritative)
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
        assert!(checkpoint.continuation_identity_is_valid());
        let source_digest = checkpoint.state_digest64;
        let source_continuation_digest = checkpoint.continuation_digest64;
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
        assert_eq!(
            boundary.source_continuation_digest64,
            source_continuation_digest
        );
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
        assert!(resumed.checkpoint.continuation_identity_is_valid());
        assert!(uninterrupted.checkpoint.continuation_identity_is_valid());
        assert_eq!(
            resumed.manifest.resume_lineage,
            resumed.checkpoint.resume_lineage
        );
    }

    #[test]
    fn checkpoint_continuation_identity_rejects_future_defining_tampering() {
        let checkpoint = Simulation::new(
            ExperimentConfig::new(2033, 4)
                .with_world(WorldConfig::new(4, 4))
                .with_population(PopulationConfig::new(64)),
        )
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();

        let mut rng_changed = checkpoint.clone();
        rng_changed.rng.migration_choice.low ^= 1;
        assert_continuation_tamper_rejected(rng_changed);

        let mut config_changed = checkpoint.clone();
        config_changed.experiment.duration_years += 1;
        assert_continuation_tamper_rejected(config_changed);

        let mut migration_changed = checkpoint.clone();
        migration_changed.migration.northward_steps ^= 1;
        assert_continuation_tamper_rejected(migration_changed);

        let mut explanatory_total_changed = checkpoint.clone();
        explanatory_total_changed
            .migration
            .origin_resource_score_total ^= 1;
        assert_continuation_tamper_rejected(explanatory_total_changed);

        let mut retained_output_changed = checkpoint;
        retained_output_changed.metrics.snapshots[0].state_digest64 ^= 1;
        assert_continuation_tamper_rejected(retained_output_changed);
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
        checkpoint = checkpoint.seal_continuation_identity();
        let source_continuation_digest64 = checkpoint.continuation_digest64;

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
            boundary.source_continuation_digest64,
            source_continuation_digest64
        );
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
        let first_continuation_digest = first.continuation_digest64;
        let second = Simulation::from_checkpoint(first)
            .unwrap()
            .checkpoint_at_year(3)
            .unwrap();
        let second_digest = second.state_digest64;
        let second_continuation_digest = second.continuation_digest64;
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
            final_run.manifest.resume_lineage.boundaries[0].source_continuation_digest64,
            first_continuation_digest
        );
        assert_eq!(
            final_run.manifest.resume_lineage.boundaries[1].source_state_digest64,
            second_digest
        );
        assert_eq!(
            final_run.manifest.resume_lineage.boundaries[1].source_continuation_digest64,
            second_continuation_digest
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
        second = second.seal_continuation_identity();

        assert!(matches!(
            Simulation::from_checkpoint(second),
            Err(SimulationError::CheckpointResumeLineageInvalid { .. })
        ));
    }

    #[test]
    fn pre_current_checkpoint_schemas_are_rejected_fail_closed() {
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
            SimulationCheckpoint::PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION,
            SimulationCheckpoint::PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION,
            SimulationCheckpoint::PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION,
            SimulationCheckpoint::PRE_CONDITION_MORTALITY_SCHEMA_VERSION,
            SimulationCheckpoint::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION,
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
    fn checkpoint_state_digest_detects_tampering_after_continuation_reseal() {
        let config = ExperimentConfig::new(505, 6)
            .with_world(WorldConfig::new(16, 16))
            .with_population(PopulationConfig::new(500));
        let mut checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(3)
            .unwrap();
        checkpoint.state_digest64 ^= 1;
        checkpoint = checkpoint.seal_continuation_identity();
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
        tampered = tampered.seal_continuation_identity();
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
