use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    checkpoint::{
        RngCheckpoint, SimulationCheckpoint, continuation_digest64,
        state_digest64_with_temporary_mobility,
    },
    config::{ExperimentConfig, PopulationInitialization},
    demography::{
        DemographyConfigError, DemographyRngs, DemographyStepOutcome,
        process_demographic_year_after_competing_mortality_recorded, validate_demography_config,
    },
    events::EventLog,
    focal_region::{FocalRegionBindingError, FocalRegionSource},
    founder_initialization::FounderGenealogyStatus,
    household_lifecycle::{
        HouseholdLifecycleError, apply_household_lifecycle_at_annual_boundary,
        validate_household_lifecycle_config,
    },
    landscape::LandscapeBundle,
    landscape_binding::{LandscapeBinding, LandscapeBindingError},
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
    research_readiness::EvidenceClosureAssessment,
    research_readiness::{assess_evidence_closure, assess_spatial_evidence_closure},
    resources::{
        BackgroundMortalityContext, ResourceConfigError, ResourceError, ResourcePeriodContext,
        ResourceRngs, ResourceStepOutcome, ResourceSystem, validate_resource_config,
    },
    rng::RngFactory,
    spatial_mechanisms::{
        SPATIAL_MODEL_SEMANTICS_ID, SpatialMechanismConfig, SpatialMechanismError,
        transform_landscape,
    },
    spatial_realization::SpatialEnvironmentProvenance,
    temporary_mobility::{
        TemporaryMobilityConfigError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,
        TemporaryMobilityProgramError, TemporaryMobilityState, TemporaryMobilityValidationError,
    },
    time::{DAYS_PER_YEAR, MAX_SUPPORTED_DURATION_YEARS, SimTime},
    world::{World, WorldError},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialMechanismBinding {
    pub schema_version: u32,
    pub spatial_model_semantics_id: String,
    pub config_identity: String,
    pub config: SpatialMechanismConfig,
    /// Exact resolved seed roles plus the residual synthetic fields still present after M8
    /// transformation. This is deliberately separate from `config_identity`.
    pub environment: SpatialEnvironmentProvenance,
    pub transformed_world_digest64: u64,
}

impl SpatialMechanismBinding {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    fn new(
        config: SpatialMechanismConfig,
        world: &World,
        experiment: &ExperimentConfig,
    ) -> Result<Self, SpatialLandscapeError> {
        config.validate()?;
        let environment = SpatialEnvironmentProvenance::resolve(experiment, &config);
        Ok(Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            spatial_model_semantics_id: SPATIAL_MODEL_SEMANTICS_ID.to_owned(),
            config_identity: config.identity(),
            config,
            environment,
            transformed_world_digest64: world.digest64(),
        })
    }

    fn validate(
        &self,
        world: &World,
        experiment: &ExperimentConfig,
    ) -> Result<(), SpatialLandscapeError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(SpatialLandscapeError::UnsupportedSpatialBindingSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.spatial_model_semantics_id != SPATIAL_MODEL_SEMANTICS_ID {
            return Err(SpatialLandscapeError::SpatialSemanticsMismatch {
                found: self.spatial_model_semantics_id.clone(),
                expected: SPATIAL_MODEL_SEMANTICS_ID.to_owned(),
            });
        }
        self.config.validate()?;
        let actual_identity = self.config.identity();
        if self.config_identity != actual_identity {
            return Err(SpatialLandscapeError::SpatialConfigIdentityMismatch {
                expected: self.config_identity.clone(),
                actual: actual_identity,
            });
        }
        let expected_environment = SpatialEnvironmentProvenance::resolve(experiment, &self.config);
        if self.environment != expected_environment {
            return Err(SpatialLandscapeError::SpatialEnvironmentProvenanceMismatch);
        }
        if self.transformed_world_digest64 != world.digest64() {
            return Err(SpatialLandscapeError::TransformedWorldDigestMismatch {
                expected: self.transformed_world_digest64,
                actual: world.digest64(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialLandscapeRunManifest {
    pub schema_version: u32,
    pub landscape: LandscapeBinding,
    pub spatial: SpatialMechanismBinding,
    /// Closure assessment composed from the exact core experiment plus the causally used
    /// landscape-layer and spatial-transform evidence claims.
    pub evidence_closure: EvidenceClosureAssessment,
    pub core_manifest: RunManifest,
}

impl SpatialLandscapeRunManifest {
    pub const PRE_COMPOSED_EVIDENCE_CLOSURE_SCHEMA_VERSION: u32 = 1;
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialLandscapeCheckpoint {
    pub schema_version: u32,
    pub landscape: LandscapeBinding,
    pub spatial: SpatialMechanismBinding,
    pub core_checkpoint: SimulationCheckpoint,
}

impl SpatialLandscapeCheckpoint {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialLandscapeRecordedRun {
    pub manifest: SpatialLandscapeRunManifest,
    pub checkpoint: SpatialLandscapeCheckpoint,
}

impl SpatialLandscapeRecordedRun {
    #[must_use]
    pub const fn core_manifest(&self) -> &RunManifest {
        &self.manifest.core_manifest
    }

    #[must_use]
    pub const fn core_checkpoint(&self) -> &SimulationCheckpoint {
        &self.checkpoint.core_checkpoint
    }

    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.checkpoint.core_checkpoint.events
    }

    #[must_use]
    pub const fn metrics(&self) -> &MetricSeries {
        &self.checkpoint.core_checkpoint.metrics
    }
}

/// Authoritative M8.4 host for an explicit normalized-landscape transformation configuration.
///
/// The pre-existing `Simulation` path remains unchanged for synthetic execution. This host uses
/// the same M2-M4 mechanisms and RNG stream names, but constructs the immutable `World` by applying
/// a declared M8.4 overlay before population/resource/migration initialization.
#[derive(Debug)]
pub struct SpatialLandscapeSimulation {
    config: ExperimentConfig,
    time: SimTime,
    terminal_stop_reason: Option<StopReason>,
    resume_lineage: ResumeLineage,
    landscape: LandscapeBundle,
    landscape_binding: LandscapeBinding,
    spatial_binding: SpatialMechanismBinding,
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

impl SpatialLandscapeSimulation {
    pub fn new(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
    ) -> Result<Self, SpatialLandscapeError> {
        Self::new_internal(config, landscape, mechanisms)
    }

    fn new_internal(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
    ) -> Result<Self, SpatialLandscapeError> {
        validate_experiment(&config)?;
        mechanisms.validate()?;
        let landscape_binding = LandscapeBinding::from_bundle(&landscape)?;
        validate_grid_match(&config, &landscape_binding)?;
        validate_movement_grid_geometry(&config, &landscape)?;
        if let Some(evidence) = &config.evidence {
            evidence.validate()?;
            landscape.validate_evidence_links(evidence)?;
        }

        let environment = SpatialEnvironmentProvenance::resolve(&config, &mechanisms);
        let world = reconstruct_world(
            &config,
            &landscape,
            &mechanisms,
            environment.realization.environment_seed,
        )?;
        validate_spatial_temporary_mobility_definition(&config, &landscape, &world)?;
        let spatial_binding = SpatialMechanismBinding::new(mechanisms, &world, &config)?;
        let population_rng_factory =
            RngFactory::new(spatial_binding.environment.realization.population_seed);
        let process_rng_factory =
            RngFactory::new(spatial_binding.environment.realization.process_seed);
        let population = match config.population.initialization {
            PopulationInitialization::SyntheticValidationV1 => {
                Population::initialize(config.population, &world, population_rng_factory)?
            }
            PopulationInitialization::DeclaredFounderStateV1 => {
                let definition = config
                    .founder_population
                    .as_ref()
                    .ok_or(SpatialLandscapeError::MissingFounderPopulationDefinition)?;
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
            .map(|definition| {
                definition.derive_program_with_seed(
                    &world,
                    spatial_binding.environment.realization.process_seed,
                )
            })
            .transpose()?;
        let temporary_mobility = match configured_program {
            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,
            None => TemporaryMobilityState::at_residence(&population),
        };
        temporary_mobility.validate_at_day(0, &population, &world)?;
        let resources = ResourceSystem::initialize(&world, &config.resources)?;
        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;

        Ok(Self {
            config,
            time: SimTime::ZERO,
            terminal_stop_reason: None,
            resume_lineage: ResumeLineage::new(),
            landscape,
            landscape_binding,
            spatial_binding,
            world,
            population,
            temporary_mobility,
            resources,
            migration,
            demography_rngs: DemographyRngs::new(process_rng_factory),
            resource_rngs: ResourceRngs::new(process_rng_factory),
            migration_rngs: MigrationRngs::new(process_rng_factory),
            events: EventLog::new(),
            metrics: MetricSeries::annual(),
        })
    }

    pub fn from_checkpoint(
        checkpoint: SpatialLandscapeCheckpoint,
        landscape: LandscapeBundle,
    ) -> Result<Self, SpatialLandscapeError> {
        if checkpoint.schema_version != SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION {
            return Err(SpatialLandscapeError::UnsupportedCheckpointWrapperSchema {
                found: checkpoint.schema_version,
                supported: SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
            });
        }
        checkpoint.landscape.validate_bundle(&landscape)?;
        validate_core_checkpoint_header(&checkpoint.core_checkpoint)?;
        validate_experiment(&checkpoint.core_checkpoint.experiment)?;
        validate_grid_match(
            &checkpoint.core_checkpoint.experiment,
            &checkpoint.landscape,
        )?;
        validate_movement_grid_geometry(&checkpoint.core_checkpoint.experiment, &landscape)?;
        if let Some(evidence) = &checkpoint.core_checkpoint.experiment.evidence {
            evidence.validate()?;
            landscape.validate_evidence_links(evidence)?;
        }

        let world = reconstruct_world(
            &checkpoint.core_checkpoint.experiment,
            &landscape,
            &checkpoint.spatial.config,
            checkpoint.spatial.environment.realization.environment_seed,
        )?;
        validate_founder_population_against_world(&checkpoint.core_checkpoint.experiment, &world)?;
        checkpoint
            .spatial
            .validate(&world, &checkpoint.core_checkpoint.experiment)?;
        if checkpoint.core_checkpoint.world_digest64 != world.digest64() {
            return Err(SpatialLandscapeError::CoreWorldDigestMismatch {
                expected: checkpoint.core_checkpoint.world_digest64,
                actual: world.digest64(),
            });
        }

        checkpoint
            .core_checkpoint
            .population
            .validate(&world)
            .map_err(PopulationError::from)?;
        validate_spatial_temporary_mobility(
            &checkpoint.core_checkpoint,
            &landscape,
            &world,
            checkpoint.spatial.environment.realization.process_seed,
        )?;
        checkpoint
            .core_checkpoint
            .resources
            .validate_checkpoint_state(&world, &checkpoint.core_checkpoint.experiment.resources)?;
        validate_terminal_checkpoint_state(&checkpoint.core_checkpoint)?;
        let migration = MigrationSystem::from_checkpoint_state(
            &checkpoint.core_checkpoint.population,
            &world,
            &checkpoint.core_checkpoint.experiment.migration,
            checkpoint.core_checkpoint.migration.clone(),
        )?;

        let rng_factory = RngFactory::new(checkpoint.spatial.environment.realization.process_seed);
        let mut demography_rngs = DemographyRngs::new(rng_factory);
        demography_rngs.restore_positions([
            checkpoint.core_checkpoint.rng.demography_mortality,
            checkpoint.core_checkpoint.rng.demography_fertility,
            checkpoint.core_checkpoint.rng.demography_parentage,
            checkpoint.core_checkpoint.rng.demography_newborn_sex,
        ]);
        let mut resource_rngs = ResourceRngs::new(rng_factory);
        resource_rngs.restore_position(checkpoint.core_checkpoint.rng.resource_scarcity_mortality);
        let mut migration_rngs = MigrationRngs::new(rng_factory);
        migration_rngs.restore_positions([
            checkpoint.core_checkpoint.rng.migration_choice,
            checkpoint.core_checkpoint.rng.migration_uncertainty,
        ]);
        let expected_state_digest = checkpoint.core_checkpoint.state_digest64;
        let source_identity = SourceRevisionIdentity {
            model_version: checkpoint.core_checkpoint.model_version.clone(),
            model_semantics_id: checkpoint.core_checkpoint.model_semantics_id.clone(),
            git_commit: checkpoint.core_checkpoint.git_commit.clone(),
        };
        let mut resume_lineage = checkpoint.core_checkpoint.resume_lineage.clone();
        resume_lineage.boundaries.push(ResumeBoundary {
            source: source_identity,
            continuation: SourceRevisionIdentity::current(),
            boundary_day: checkpoint.core_checkpoint.time.days(),
            boundary_completed_years: checkpoint.core_checkpoint.completed_years,
            source_state_digest64: checkpoint.core_checkpoint.state_digest64,
            source_continuation_digest64: checkpoint.core_checkpoint.continuation_digest64,
        });

        let simulation = Self {
            config: checkpoint.core_checkpoint.experiment,
            time: checkpoint.core_checkpoint.time,
            terminal_stop_reason: checkpoint.core_checkpoint.terminal_stop_reason,
            resume_lineage,
            landscape,
            landscape_binding: checkpoint.landscape,
            spatial_binding: checkpoint.spatial,
            world,
            population: checkpoint.core_checkpoint.population,
            temporary_mobility: checkpoint.core_checkpoint.temporary_mobility,
            resources: checkpoint.core_checkpoint.resources,
            migration,
            demography_rngs,
            resource_rngs,
            migration_rngs,
            events: checkpoint.core_checkpoint.events,
            metrics: checkpoint.core_checkpoint.metrics,
        };
        let actual_digest = simulation.state_digest64();
        if actual_digest != expected_state_digest {
            return Err(SpatialLandscapeError::CheckpointStateDigestMismatch {
                expected: expected_state_digest,
                actual: actual_digest,
            });
        }
        Ok(simulation)
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
    pub const fn landscape(&self) -> &LandscapeBundle {
        &self.landscape
    }

    #[must_use]
    pub const fn spatial_binding(&self) -> &SpatialMechanismBinding {
        &self.spatial_binding
    }

    pub fn run_recorded(mut self) -> Result<SpatialLandscapeRecordedRun, SpatialLandscapeError> {
        let target_year = self.config.duration_years;
        let stop_reason = self
            .advance_to_year(target_year)?
            .unwrap_or(StopReason::DurationReached);
        self.terminal_stop_reason = Some(stop_reason);
        self.ensure_terminal_metric_snapshot();
        self.validate_state()?;
        let core_manifest = self.build_manifest(stop_reason);
        let evidence_closure = assess_spatial_evidence_closure(
            &self.config,
            &self.landscape,
            &self.spatial_binding.config,
        );
        let source_landscape = self.landscape.clone();
        let landscape = self.landscape_binding.clone();
        let spatial = self.spatial_binding.clone();
        let core_checkpoint = self.into_checkpoint();
        let run = SpatialLandscapeRecordedRun {
            manifest: SpatialLandscapeRunManifest {
                schema_version: SpatialLandscapeRunManifest::CURRENT_SCHEMA_VERSION,
                landscape: landscape.clone(),
                spatial: spatial.clone(),
                evidence_closure,
                core_manifest,
            },
            checkpoint: SpatialLandscapeCheckpoint {
                schema_version: SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
                landscape,
                spatial,
                core_checkpoint,
            },
        };
        validate_spatial_landscape_recorded_run(&run, &source_landscape)?;
        Ok(run)
    }

    pub fn checkpoint_at_year(
        mut self,
        target_year: u64,
    ) -> Result<SpatialLandscapeCheckpoint, SpatialLandscapeError> {
        let current_year = self.completed_years()?;
        if target_year < current_year || target_year > self.config.duration_years {
            return Err(SpatialLandscapeError::InvalidCheckpointTarget {
                current_year,
                target_year,
                duration_years: self.config.duration_years,
            });
        }
        if let Some(stop_reason) = self.advance_to_year(target_year)? {
            let expected_day = target_year.saturating_mul(DAYS_PER_YEAR);
            if self.time.days() != expected_day {
                return Err(SpatialLandscapeError::CheckpointTargetUnreachable {
                    target_year,
                    stop_reason,
                    stopped_day: self.time.days(),
                });
            }
        }
        self.ensure_terminal_metric_snapshot();
        self.validate_state()?;
        let landscape = self.landscape_binding.clone();
        let spatial = self.spatial_binding.clone();
        Ok(SpatialLandscapeCheckpoint {
            schema_version: SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
            landscape,
            spatial,
            core_checkpoint: self.into_checkpoint(),
        })
    }

    fn advance_to_year(
        &mut self,
        target_year: u64,
    ) -> Result<Option<StopReason>, SpatialLandscapeError> {
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
                    let outcome = self
                        .resources
                        .process_period_recorded_with_presence_and_background(
                            &mut self.population,
                            &ResourcePeriodContext {
                                world: &self.world,
                                config: &self.config.resources,
                                period_index_in_year: resource_index,
                                day,
                            },
                            &mut self.resource_rngs.scarcity_mortality,
                            Some(BackgroundMortalityContext {
                                config: &self.config.demography,
                                mortality_rng: self.demography_rngs.mortality_rng_mut(),
                            }),
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
            let outcome = process_demographic_year_after_competing_mortality_recorded(
                &mut self.population,
                &self.world,
                &self.config.demography,
                self.time.days(),
                &mut self.demography_rngs,
                &mut self.events,
            )?;
            self.temporary_mobility
                .reconcile_after_population_change(&self.population);
            if let Some(household_lifecycle) = self.config.household_lifecycle.clone() {
                apply_household_lifecycle_at_annual_boundary(
                    &mut self.population,
                    &mut self.temporary_mobility,
                    &mut self.events,
                    &household_lifecycle,
                    self.time.days(),
                )?;
            }
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
    ) -> Result<(), SpatialLandscapeError> {
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

    fn completed_years(&self) -> Result<u64, SpatialLandscapeError> {
        if !self.time.days().is_multiple_of(DAYS_PER_YEAR) {
            return Err(SpatialLandscapeError::UnsupportedCheckpointBoundary {
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

    fn validate_state(&self) -> Result<(), SpatialLandscapeError> {
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
            evidence_closure: assess_evidence_closure(&self.config),
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

pub fn validate_spatial_landscape_recorded_run(
    run: &SpatialLandscapeRecordedRun,
    landscape: &LandscapeBundle,
) -> Result<(), SpatialLandscapeError> {
    if run.manifest.schema_version != SpatialLandscapeRunManifest::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::UnsupportedManifestWrapperSchema {
            found: run.manifest.schema_version,
            supported: SpatialLandscapeRunManifest::CURRENT_SCHEMA_VERSION,
        });
    }
    if run.checkpoint.schema_version != SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::UnsupportedCheckpointWrapperSchema {
            found: run.checkpoint.schema_version,
            supported: SpatialLandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
        });
    }
    if run.manifest.landscape != run.checkpoint.landscape {
        return Err(SpatialLandscapeError::CrossArtifactLandscapeMismatch);
    }
    if run.manifest.spatial != run.checkpoint.spatial {
        return Err(SpatialLandscapeError::CrossArtifactSpatialMismatch);
    }
    run.manifest.landscape.validate_bundle(landscape)?;
    validate_core_checkpoint_header(&run.checkpoint.core_checkpoint)?;
    validate_experiment(&run.checkpoint.core_checkpoint.experiment)?;
    validate_movement_grid_geometry(&run.checkpoint.core_checkpoint.experiment, landscape)?;

    let world = reconstruct_world(
        &run.checkpoint.core_checkpoint.experiment,
        landscape,
        &run.checkpoint.spatial.config,
        run.checkpoint
            .spatial
            .environment
            .realization
            .environment_seed,
    )?;
    validate_founder_population_against_world(&run.checkpoint.core_checkpoint.experiment, &world)?;
    run.checkpoint
        .spatial
        .validate(&world, &run.checkpoint.core_checkpoint.experiment)?;
    if run.checkpoint.core_checkpoint.world_digest64 != world.digest64() {
        return Err(SpatialLandscapeError::CoreWorldDigestMismatch {
            expected: run.checkpoint.core_checkpoint.world_digest64,
            actual: world.digest64(),
        });
    }
    let expected_core_evidence_closure =
        assess_evidence_closure(&run.checkpoint.core_checkpoint.experiment);
    if run.manifest.core_manifest.evidence_closure != expected_core_evidence_closure {
        return Err(SpatialLandscapeError::CoreEvidenceClosureMismatch);
    }
    let expected_spatial_evidence_closure = assess_spatial_evidence_closure(
        &run.checkpoint.core_checkpoint.experiment,
        landscape,
        &run.checkpoint.spatial.config,
    );
    if run.manifest.evidence_closure != expected_spatial_evidence_closure {
        return Err(SpatialLandscapeError::SpatialEvidenceClosureMismatch);
    }
    if run.manifest.core_manifest.experiment != run.checkpoint.core_checkpoint.experiment
        || run.manifest.core_manifest.resume_lineage
            != run.checkpoint.core_checkpoint.resume_lineage
        || run.manifest.core_manifest.state_digest64
            != run.checkpoint.core_checkpoint.state_digest64
        || run.manifest.core_manifest.world.digest64 != format!("{:016x}", world.digest64())
        || run.manifest.core_manifest.stop_reason
            != run
                .checkpoint
                .core_checkpoint
                .terminal_stop_reason
                .unwrap_or(StopReason::DurationReached)
    {
        return Err(SpatialLandscapeError::CrossArtifactCoreMismatch);
    }

    run.checkpoint
        .core_checkpoint
        .population
        .validate(&world)
        .map_err(PopulationError::from)?;
    validate_spatial_temporary_mobility(
        &run.checkpoint.core_checkpoint,
        landscape,
        &world,
        run.checkpoint.spatial.environment.realization.process_seed,
    )?;
    run.checkpoint
        .core_checkpoint
        .resources
        .validate_checkpoint_state(&world, &run.checkpoint.core_checkpoint.experiment.resources)?;
    let migration = MigrationSystem::from_checkpoint_state(
        &run.checkpoint.core_checkpoint.population,
        &world,
        &run.checkpoint.core_checkpoint.experiment.migration,
        run.checkpoint.core_checkpoint.migration.clone(),
    )?;
    let actual_state = state_digest64_with_temporary_mobility(
        run.checkpoint.core_checkpoint.time.days(),
        world.digest64(),
        run.checkpoint.core_checkpoint.population.digest64(),
        run.checkpoint.core_checkpoint.resources.digest64(),
        migration.digest64(),
        &run.checkpoint.core_checkpoint.temporary_mobility,
    );
    if actual_state != run.checkpoint.core_checkpoint.state_digest64 {
        return Err(SpatialLandscapeError::CheckpointStateDigestMismatch {
            expected: run.checkpoint.core_checkpoint.state_digest64,
            actual: actual_state,
        });
    }
    Ok(())
}

fn reconstruct_world(
    config: &ExperimentConfig,
    landscape: &LandscapeBundle,
    mechanisms: &SpatialMechanismConfig,
    environment_seed: u64,
) -> Result<World, SpatialLandscapeError> {
    landscape.validate_evidence_context(config.evidence.as_ref())?;
    mechanisms.validate_evidence_links(config.evidence.as_ref())?;
    let overlay = transform_landscape(landscape, mechanisms)?;
    let world = World::generate(config.world, RngFactory::new(environment_seed))?
        .with_model_field_overlay(
            overlay.movement_cost.as_deref(),
            overlay.water_access.as_deref(),
            overlay.base_productivity.as_deref(),
        )?;
    Ok(world)
}

fn validate_grid_match(
    config: &ExperimentConfig,
    binding: &LandscapeBinding,
) -> Result<(), SpatialLandscapeError> {
    if config.world.width != binding.width || config.world.height != binding.height {
        return Err(SpatialLandscapeError::GridMismatch {
            world_width: config.world.width,
            world_height: config.world.height,
            landscape_width: binding.width,
            landscape_height: binding.height,
        });
    }
    Ok(())
}

fn validate_movement_grid_geometry(
    config: &ExperimentConfig,
    landscape: &LandscapeBundle,
) -> Result<(), SpatialLandscapeError> {
    if landscape.geometry.has_square_cells()
        || (!config.migration.enabled && config.temporary_mobility.is_none())
    {
        return Ok(());
    }
    Err(SpatialLandscapeError::RectangularMovementGrid {
        cell_size_x: landscape.geometry.cell_size_x,
        cell_size_y: landscape.geometry.cell_size_y,
        coordinate_unit: landscape.geometry.coordinate_unit.clone(),
    })
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

fn validate_experiment(config: &ExperimentConfig) -> Result<(), SpatialLandscapeError> {
    if config.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::UnsupportedExperimentSchema {
            found: config.schema_version,
            supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.duration_years > MAX_SUPPORTED_DURATION_YEARS {
        return Err(SpatialLandscapeError::DurationOutOfRange {
            duration_years: config.duration_years,
            maximum_years: MAX_SUPPORTED_DURATION_YEARS,
        });
    }
    validate_founder_population_binding(config)?;
    validate_demography_config(&config.demography)?;
    if let Some(household_lifecycle) = &config.household_lifecycle {
        validate_household_lifecycle_config(household_lifecycle)?;
    }
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

fn validate_founder_population_binding(
    config: &ExperimentConfig,
) -> Result<(), SpatialLandscapeError> {
    match (
        config.population.initialization,
        config.founder_population.as_ref(),
    ) {
        (PopulationInitialization::SyntheticValidationV1, None) => Ok(()),
        (PopulationInitialization::SyntheticValidationV1, Some(_)) => {
            Err(SpatialLandscapeError::UnexpectedFounderPopulationDefinition)
        }
        (PopulationInitialization::DeclaredFounderStateV1, None) => {
            Err(SpatialLandscapeError::MissingFounderPopulationDefinition)
        }
        (PopulationInitialization::DeclaredFounderStateV1, Some(definition)) => {
            if config.migration.enabled
                && config.migration.kin_weight > 0
                && definition.genealogy_status
                    != FounderGenealogyStatus::CompleteLivingDirectParents
            {
                return Err(SpatialLandscapeError::FounderKinStateUnspecified);
            }
            Ok(())
        }
    }
}

fn validate_founder_population_against_world(
    config: &ExperimentConfig,
    world: &World,
) -> Result<(), SpatialLandscapeError> {
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

fn validate_spatial_temporary_mobility_definition(
    config: &ExperimentConfig,
    landscape: &LandscapeBundle,
    world: &World,
) -> Result<(), SpatialLandscapeError> {
    let Some(definition) = &config.temporary_mobility else {
        return Ok(());
    };
    let FocalRegionSource::LandscapeMask {
        evidence_input_id, ..
    } = &definition.region.source
    else {
        return Ok(());
    };
    let evidence = config.evidence.as_ref().ok_or_else(|| {
        TemporaryMobilityConfigError::MissingEvidenceCatalog {
            input_id: evidence_input_id.clone(),
        }
    })?;
    definition
        .region
        .validate_landscape_binding(landscape, evidence, world)?;
    Ok(())
}

fn validate_core_checkpoint_header(
    checkpoint: &SimulationCheckpoint,
) -> Result<(), SpatialLandscapeError> {
    if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::UnsupportedCoreCheckpointSchema {
            found: checkpoint.schema_version,
            supported: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,
        });
    }
    if checkpoint.model_version != env!("CARGO_PKG_VERSION") {
        return Err(SpatialLandscapeError::CheckpointModelVersionMismatch {
            found: checkpoint.model_version.clone(),
            expected: env!("CARGO_PKG_VERSION").to_owned(),
        });
    }
    if checkpoint.model_semantics_id != MODEL_SEMANTICS_ID {
        return Err(SpatialLandscapeError::CheckpointCoreSemanticsMismatch {
            found: checkpoint.model_semantics_id.clone(),
            expected: MODEL_SEMANTICS_ID.to_owned(),
        });
    }
    let actual_continuation_digest64 = continuation_digest64(checkpoint);
    if actual_continuation_digest64 != checkpoint.continuation_digest64 {
        return Err(
            SpatialLandscapeError::CheckpointContinuationDigestMismatch {
                expected: checkpoint.continuation_digest64,
                actual: actual_continuation_digest64,
            },
        );
    }
    let source_identity = SourceRevisionIdentity {
        model_version: checkpoint.model_version.clone(),
        model_semantics_id: checkpoint.model_semantics_id.clone(),
        git_commit: checkpoint.git_commit.clone(),
    };
    checkpoint
        .resume_lineage
        .validate_for_artifact(checkpoint.time.days(), &source_identity)
        .map_err(
            |error| SpatialLandscapeError::CheckpointResumeLineageInvalid {
                reason: error.to_string(),
            },
        )?;
    if checkpoint.events.schema_version != EventLog::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::CheckpointArtifactSchemaMismatch { artifact: "events" });
    }
    if checkpoint.metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::CheckpointArtifactSchemaMismatch {
            artifact: "metrics",
        });
    }
    if (!checkpoint.time.days().is_multiple_of(DAYS_PER_YEAR)
        && !matches!(
            checkpoint.terminal_stop_reason,
            Some(StopReason::PopulationExtinct)
        ))
        || checkpoint.completed_years != checkpoint.time.days() / DAYS_PER_YEAR
    {
        return Err(SpatialLandscapeError::UnsupportedCheckpointBoundary {
            day: checkpoint.time.days(),
        });
    }
    if checkpoint.completed_years > checkpoint.experiment.duration_years {
        return Err(SpatialLandscapeError::CheckpointBeyondDuration {
            completed_years: checkpoint.completed_years,
            duration_years: checkpoint.experiment.duration_years,
        });
    }
    Ok(())
}

fn validate_spatial_temporary_mobility(
    checkpoint: &SimulationCheckpoint,
    landscape: &LandscapeBundle,
    world: &World,
    process_seed: u64,
) -> Result<(), SpatialLandscapeError> {
    validate_spatial_temporary_mobility_definition(&checkpoint.experiment, landscape, world)?;
    checkpoint
        .temporary_mobility
        .validate_at_day(checkpoint.time.days(), &checkpoint.population, world)
        .map_err(|error| SpatialLandscapeError::TemporaryMobilityInvalid {
            reason: error.to_string(),
        })?;
    if let Some(definition) = &checkpoint.experiment.temporary_mobility {
        let expected = definition.derive_program_with_seed(world, process_seed)?;
        if checkpoint.temporary_mobility.program() != Some(&expected) {
            return Err(SpatialLandscapeError::ConfiguredTemporaryMobilityMismatch {
                expected: expected.identity(),
                actual: checkpoint
                    .temporary_mobility
                    .program()
                    .map(TemporaryMobilityProgram::identity),
            });
        }
    } else if !checkpoint.temporary_mobility.is_disabled() {
        return Err(SpatialLandscapeError::ConfiguredTemporaryMobilityMismatch {
            expected: "temporary-mobility-disabled".to_owned(),
            actual: checkpoint
                .temporary_mobility
                .program()
                .map(TemporaryMobilityProgram::identity)
                .or_else(|| Some("unconfigured-temporary-mobility-state".to_owned())),
        });
    }
    Ok(())
}

fn validate_terminal_checkpoint_state(
    checkpoint: &SimulationCheckpoint,
) -> Result<(), SpatialLandscapeError> {
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
        return Err(SpatialLandscapeError::CheckpointTerminalStateMismatch { stop_reason });
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum SpatialLandscapeError {
    #[error(transparent)]
    LandscapeBinding(#[from] LandscapeBindingError),
    #[error(transparent)]
    Landscape(#[from] crate::LandscapeError),
    #[error(transparent)]
    Evidence(#[from] crate::EvidenceError),
    #[error(transparent)]
    FocalRegionBinding(#[from] FocalRegionBindingError),
    #[error(transparent)]
    SpatialMechanism(#[from] SpatialMechanismError),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error(transparent)]
    HouseholdLifecycle(#[from] HouseholdLifecycleError),
    #[error(transparent)]
    DemographyConfig(#[from] DemographyConfigError),
    #[error(transparent)]
    ResourceConfig(#[from] ResourceConfigError),
    #[error(transparent)]
    Resources(#[from] ResourceError),
    #[error(transparent)]
    MigrationConfig(#[from] MigrationConfigError),
    #[error(transparent)]
    Migration(#[from] MigrationError),
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
    #[error(
        "grid-step M4/M9 movement requires square landscape cells; found {cell_size_x} by {cell_size_y} {coordinate_unit} cells"
    )]
    RectangularMovementGrid {
        cell_size_x: u64,
        cell_size_y: u64,
        coordinate_unit: String,
    },
    #[error("spatial binding schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSpatialBindingSchema { found: u32, supported: u32 },
    #[error(
        "spatial landscape manifest schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedManifestWrapperSchema { found: u32, supported: u32 },
    #[error(
        "spatial landscape checkpoint schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedCheckpointWrapperSchema { found: u32, supported: u32 },
    #[error("core checkpoint schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedCoreCheckpointSchema { found: u32, supported: u32 },
    #[error("spatial model semantics {found} does not match current semantics {expected}")]
    SpatialSemanticsMismatch { found: String, expected: String },
    #[error("spatial config identity mismatch: stored {expected}, reconstructed {actual}")]
    SpatialConfigIdentityMismatch { expected: String, actual: String },
    #[error(
        "spatial environment-realization provenance does not match the bound experiment/config"
    )]
    SpatialEnvironmentProvenanceMismatch,
    #[error("transformed world digest mismatch: stored {expected}, reconstructed {actual}")]
    TransformedWorldDigestMismatch { expected: u64, actual: u64 },
    #[error("core checkpoint world digest mismatch: stored {expected}, reconstructed {actual}")]
    CoreWorldDigestMismatch { expected: u64, actual: u64 },
    #[error("checkpoint model version {found} does not match current model version {expected}")]
    CheckpointModelVersionMismatch { found: String, expected: String },
    #[error("checkpoint core semantics {found} does not match current core semantics {expected}")]
    CheckpointCoreSemanticsMismatch { found: String, expected: String },
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
    #[error("spatial checkpoint temporary mobility state is invalid: {reason}")]
    TemporaryMobilityInvalid { reason: String },
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
    #[error("checkpoint terminal stop reason {stop_reason:?} does not match checkpoint state")]
    CheckpointTerminalStateMismatch { stop_reason: StopReason },
    #[error("checkpoint state digest mismatch: expected {expected}, reconstructed {actual}")]
    CheckpointStateDigestMismatch { expected: u64, actual: u64 },
    #[error(
        "simulation grid {world_width}x{world_height} does not match landscape {landscape_width}x{landscape_height}"
    )]
    GridMismatch {
        world_width: u32,
        world_height: u32,
        landscape_width: u32,
        landscape_height: u32,
    },
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
    #[error("spatial landscape manifest/checkpoint landscape bindings disagree")]
    CrossArtifactLandscapeMismatch,
    #[error("spatial landscape manifest/checkpoint mechanism bindings disagree")]
    CrossArtifactSpatialMismatch,
    #[error("spatial landscape core manifest evidence-closure provenance is inconsistent")]
    CoreEvidenceClosureMismatch,
    #[error("spatial landscape composed evidence-closure provenance is inconsistent")]
    SpatialEvidenceClosureMismatch,
    #[error("spatial landscape core manifest/checkpoint do not describe the same run")]
    CrossArtifactCoreMismatch,
}
