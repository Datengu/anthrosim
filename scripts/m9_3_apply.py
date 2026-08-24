from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Keep M9.1 test-only active-state fixtures within representable u32 travel durations.
path = "crates/anthrosim-core/src/temporary_mobility.rs"
replace_once(path, "                0,\n                u64::MAX - 3,\n                u64::MAX - 2,\n                u64::MAX - 1,", "                0,\n                999_998,\n                999_999,\n                1_000_000,")
replace_once(path, "                0,\n                0,\n                u64::MAX - 2,\n                u64::MAX - 1,", "                0,\n                0,\n                999_999,\n                1_000_000,")
replace_once(path, "                0,\n                0,\n                0,\n                u64::MAX - 1,", "                0,\n                0,\n                1,\n                1_000_000,")

# Core Simulation integration.
path = "crates/anthrosim-core/src/simulation.rs"
replace_once(
    path,
    "    temporary_mobility::{TemporaryMobilityState, TemporaryMobilityValidationError},",
    "    temporary_mobility::{\n        TemporaryMobilityExecutionError, TemporaryMobilityProgram, TemporaryMobilityProgramError,\n        TemporaryMobilityState, TemporaryMobilityValidationError,\n    },",
)
replace_once(
    path,
    '''    pub fn new(config: ExperimentConfig) -> Result<Self, SimulationError> {\n        validate_experiment(&config)?;\n\n        let rng_factory = RngFactory::new(config.seed);\n        let world = World::generate(config.world, rng_factory)?;\n        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let temporary_mobility = TemporaryMobilityState::at_residence(&population);\n        temporary_mobility.validate(&population, &world)?;\n        let resources = ResourceSystem::initialize(&world, &config.resources)?;\n        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;\n\n        Ok(Self {\n            demography_rngs: DemographyRngs::new(rng_factory),\n            resource_rngs: ResourceRngs::new(rng_factory),\n            migration_rngs: MigrationRngs::new(rng_factory),\n            config,\n            time: SimTime::ZERO,\n            terminal_stop_reason: None,\n            resume_lineage: ResumeLineage::new(),\n            world,\n            population,\n            temporary_mobility,\n            resources,\n            migration,\n            events: EventLog::new(),\n            metrics: MetricSeries::annual(),\n        })\n    }''',
    '''    pub fn new(config: ExperimentConfig) -> Result<Self, SimulationError> {\n        Self::new_internal(config, None)\n    }\n\n    /// Construct a core simulation with an explicit M9 temporary-mobility program.\n    pub fn new_with_temporary_mobility(\n        config: ExperimentConfig,\n        program: TemporaryMobilityProgram,\n    ) -> Result<Self, SimulationError> {\n        Self::new_internal(config, Some(program))\n    }\n\n    fn new_internal(\n        config: ExperimentConfig,\n        program: Option<TemporaryMobilityProgram>,\n    ) -> Result<Self, SimulationError> {\n        validate_experiment(&config)?;\n\n        let rng_factory = RngFactory::new(config.seed);\n        let world = World::generate(config.world, rng_factory)?;\n        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let temporary_mobility = match program {\n            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,\n            None => TemporaryMobilityState::at_residence(&population),\n        };\n        temporary_mobility.validate_at_day(0, &population, &world)?;\n        let resources = ResourceSystem::initialize(&world, &config.resources)?;\n        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;\n\n        Ok(Self {\n            demography_rngs: DemographyRngs::new(rng_factory),\n            resource_rngs: ResourceRngs::new(rng_factory),\n            migration_rngs: MigrationRngs::new(rng_factory),\n            config,\n            time: SimTime::ZERO,\n            terminal_stop_reason: None,\n            resume_lineage: ResumeLineage::new(),\n            world,\n            population,\n            temporary_mobility,\n            resources,\n            migration,\n            events: EventLog::new(),\n            metrics: MetricSeries::annual(),\n        })\n    }''',
)
replace_once(
    path,
    '''        checkpoint\n            .temporary_mobility\n            .validate(&checkpoint.population, &world)?;''',
    '''        checkpoint.temporary_mobility.validate_at_day(\n            checkpoint.time.days(),\n            &checkpoint.population,\n            &world,\n        )?;''',
)
replace_once(
    path,
    '''                self.time = SimTime::from_days(day);\n                let outcome = self.resources.process_period_recorded(''',
    '''                self.process_temporary_boundaries_before(day)?;\n                self.time = SimTime::from_days(day);\n                let outcome = self.resources.process_period_recorded(''',
)
replace_once(
    path,
    '''                if outcome == ResourceStepOutcome::PopulationExtinct {\n                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);\n                    self.record_metric_snapshot();\n                    return Ok(self.terminal_stop_reason);\n                }\n                self.migration.process_boundary_recorded_with_presence(''',
    '''                if outcome == ResourceStepOutcome::PopulationExtinct {\n                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);\n                    self.record_metric_snapshot();\n                    return Ok(self.terminal_stop_reason);\n                }\n                self.temporary_mobility.process_day(\n                    day,\n                    &self.population,\n                    &self.world,\n                    &mut self.events,\n                )?;\n                self.migration.process_boundary_recorded_with_presence(''',
)
replace_once(
    path,
    '''    fn completed_years(&self) -> Result<u64, SimulationError> {''',
    '''    fn process_temporary_boundaries_before(\n        &mut self,\n        fixed_day: u64,\n    ) -> Result<(), SimulationError> {\n        let Some(end_day) = fixed_day.checked_sub(1) else {\n            return Ok(());\n        };\n        loop {\n            let current_day = self.time.days();\n            let Some(day) = self.temporary_mobility.next_boundary_day(\n                current_day,\n                end_day,\n                &self.population,\n            )? else {\n                break;\n            };\n            self.time = SimTime::from_days(day);\n            self.temporary_mobility.process_day(\n                day,\n                &self.population,\n                &self.world,\n                &mut self.events,\n            )?;\n        }\n        Ok(())\n    }\n\n    fn completed_years(&self) -> Result<u64, SimulationError> {''',
)
replace_once(
    path,
    '''        self.temporary_mobility\n            .validate(&self.population, &self.world)?;''',
    '''        self.temporary_mobility\n            .validate_at_day(self.time.days(), &self.population, &self.world)?;''',
)
replace_once(
    path,
    '''    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),''',
    '''    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n    #[error(transparent)]\n    TemporaryMobilityProgram(#[from] TemporaryMobilityProgramError),\n    #[error(transparent)]\n    TemporaryMobilityExecution(#[from] TemporaryMobilityExecutionError),''',
)
replace_once(
    path,
    '''            SimulationCheckpoint::PRE_LINEAGE_SCHEMA_VERSION,\n            SimulationCheckpoint::PRE_TEMPORARY_MOBILITY_SCHEMA_VERSION,''',
    '''            SimulationCheckpoint::PRE_LINEAGE_SCHEMA_VERSION,\n            SimulationCheckpoint::PRE_TEMPORARY_MOBILITY_SCHEMA_VERSION,\n            SimulationCheckpoint::PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION,''',
)

# Invariants: validate temporal state at artifact day and accept structurally valid M9.3 events.
path = "crates/anthrosim-core/src/invariants.rs"
replace_once(
    path,
    '''    checkpoint\n        .temporary_mobility\n        .validate(&checkpoint.population, &world)\n        .map_err(|error| {''',
    '''    checkpoint\n        .temporary_mobility\n        .validate_at_day(checkpoint.time.days(), &checkpoint.population, &world)\n        .map_err(|error| {''',
)
needle = '''            EventKind::HouseholdMigration {\n                household,\n                people_moved,\n                origin,\n                destination,\n                distance_cells,\n                pressure_permille,\n                selected_weight,\n                total_move_weight,\n                choice_draw,\n                travel_condition_cost_per_person,\n                ..\n            } => {\n                counts.migrations = counts.migrations.saturating_add(1);\n                counts.people_moved = counts.people_moved.saturating_add(u64::from(*people_moved));\n                counts.migration_distance = counts\n                    .migration_distance\n                    .saturating_add(u64::from(*distance_cells));\n                let distance =\n                    manhattan_distance(world, *origin, *destination).ok_or_else(|| {\n                        InvariantError::Violation("migration event references invalid cells".into())\n                    })?;\n                if household.0 == 0\n                    || household.0 > population.household_count\n                    || distance == 0\n                    || distance != *distance_cells\n                    || *pressure_permille > PERMILLE_MAX\n                    || *travel_condition_cost_per_person > PERMILLE_MAX\n                    || *selected_weight == 0\n                    || *selected_weight > *total_move_weight\n                    || *choice_draw >= *total_move_weight\n                {\n                    return violation("migration event accounting is invalid");\n                }\n            }'''
replacement = needle + '''\n            EventKind::TemporaryJourneyNotStarted {\n                event_schema_version,\n                household,\n                region_id,\n                region_identity,\n                ..\n            } => {\n                if *event_schema_version != 1\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || region_id.trim().is_empty()\n                    || region_identity.trim().is_empty()\n                {\n                    return violation("temporary journey skip event is invalid");\n                }\n            }\n            EventKind::TemporaryJourneyDeparted {\n                event_schema_version,\n                household,\n                journey,\n                region_id,\n                region_identity,\n                residence,\n                destination,\n                departure_day,\n                arrival_day,\n                return_departure_day,\n                completion_day,\n                outbound_travel_days,\n                return_travel_days,\n                ..\n            } => {\n                if *event_schema_version != 1\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || journey.0 == 0\n                    || region_id.trim().is_empty()\n                    || region_identity.trim().is_empty()\n                    || world.cell(*residence).is_none()\n                    || world.cell(*destination).is_none()\n                    || residence == destination\n                    || record.day != *departure_day\n                    || *arrival_day != departure_day.saturating_add(u64::from(*outbound_travel_days))\n                    || *return_departure_day <= *arrival_day\n                    || *completion_day\n                        != return_departure_day.saturating_add(u64::from(*return_travel_days))\n                {\n                    return violation("temporary journey departure event is invalid");\n                }\n            }\n            EventKind::TemporaryJourneyArrived {\n                event_schema_version,\n                household,\n                journey,\n                region_id,\n                region_identity,\n                destination,\n                ..\n            } => {\n                if *event_schema_version != 1\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || journey.0 == 0\n                    || region_id.trim().is_empty()\n                    || region_identity.trim().is_empty()\n                    || world.cell(*destination).is_none()\n                {\n                    return violation("temporary journey arrival event is invalid");\n                }\n            }\n            EventKind::TemporaryReturnDeparted {\n                event_schema_version,\n                household,\n                journey,\n                region_id,\n                region_identity,\n                destination,\n                residence,\n                ..\n            } => {\n                if *event_schema_version != 1\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || journey.0 == 0\n                    || region_id.trim().is_empty()\n                    || region_identity.trim().is_empty()\n                    || world.cell(*destination).is_none()\n                    || world.cell(*residence).is_none()\n                    || destination == residence\n                {\n                    return violation("temporary return-departure event is invalid");\n                }\n            }\n            EventKind::TemporaryJourneyCompleted {\n                event_schema_version,\n                household,\n                journey,\n                region_id,\n                region_identity,\n                residence,\n                ..\n            } => {\n                if *event_schema_version != 1\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || journey.0 == 0\n                    || region_id.trim().is_empty()\n                    || region_identity.trim().is_empty()\n                    || world.cell(*residence).is_none()\n                {\n                    return violation("temporary journey completion event is invalid");\n                }\n            }'''
replace_once(path, needle, replacement)

# Existing M8 spatial host must reject any enabled M9.3 program, even while nobody is away.
path = "crates/anthrosim-core/src/spatial_simulation.rs"
replace_once(
    path,
    '''    checkpoint\n        .temporary_mobility\n        .validate(&checkpoint.population, world)\n        .map_err(|error| SpatialLandscapeError::TemporaryMobilityInvalid {''',
    '''    checkpoint\n        .temporary_mobility\n        .validate_at_day(checkpoint.time.days(), &checkpoint.population, world)\n        .map_err(|error| SpatialLandscapeError::TemporaryMobilityInvalid {''',
)
replace_once(
    path,
    '''    if !checkpoint.temporary_mobility.all_at_residence() {\n        return Err(SpatialLandscapeError::ActiveTemporaryMobilityUnsupported);\n    }''',
    '''    if !checkpoint.temporary_mobility.is_disabled() {\n        return Err(SpatialLandscapeError::ActiveTemporaryMobilityUnsupported);\n    }''',
)

print("M9.3 integration substitutions applied")
