from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def patch(path: str, transforms):
    p = ROOT / path
    text = p.read_text()
    for old, new in transforms:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
        text = text.replace(old, new, 1)
    p.write_text(text)


patch(
    "crates/anthrosim-core/src/config.rs",
    [
        (
            "use crate::evidence::EvidenceCatalog;\n",
            "use crate::{evidence::EvidenceCatalog, temporary_mobility::TemporaryMobilityConfig};\n",
        ),
        (
            "    pub migration: MigrationConfig,\n    /// Optional machine-readable evidence catalogue.",
            "    pub migration: MigrationConfig,\n    /// Optional world-independent M9 definition. The authoritative resolved travel table is\n    /// derived from each run's actual world rather than copied across seeds.\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub temporary_mobility: Option<TemporaryMobilityConfig>,\n    /// Optional machine-readable evidence catalogue.",
        ),
        (
            "    pub const CURRENT_SCHEMA_VERSION: u32 = 7;\n",
            "    pub const CURRENT_SCHEMA_VERSION: u32 = 8;\n",
        ),
        (
            "            resources: ResourceConfig::synthetic_validation_v1(),\n            migration: MigrationConfig::synthetic_validation_v1(),\n            evidence: None,\n",
            "            resources: ResourceConfig::synthetic_validation_v1(),\n            migration: MigrationConfig::synthetic_validation_v1(),\n            temporary_mobility: None,\n            evidence: None,\n",
        ),
        (
            "    pub fn with_migration(mut self, migration: MigrationConfig) -> Self {\n        self.migration = migration;\n        self\n    }\n\n    #[must_use]\n    pub fn with_evidence",
            "    pub fn with_migration(mut self, migration: MigrationConfig) -> Self {\n        self.migration = migration;\n        self\n    }\n\n    #[must_use]\n    pub fn with_temporary_mobility(mut self, temporary_mobility: TemporaryMobilityConfig) -> Self {\n        self.temporary_mobility = Some(temporary_mobility);\n        self\n    }\n\n    #[must_use]\n    pub fn with_evidence",
        ),
    ],
)

patch(
    "crates/anthrosim-core/src/focal_region.rs",
    [
        (
            "    fn validate_structure(&self) -> Result<(), FocalRegionError> {\n",
            "    pub(crate) fn validate_structure(&self) -> Result<(), FocalRegionError> {\n",
        )
    ],
)

p = ROOT / "crates/anthrosim-core/src/temporary_mobility.rs"
text = p.read_text()
old = "    temporary_travel::TemporaryTravelModel,\n"
new = "    temporary_travel::{TemporaryTravelModel, TemporaryTravelModelError},\n"
if text.count(old) != 1:
    raise SystemExit("temporary_mobility.rs: travel import boundary changed")
text = text.replace(old, new, 1)
marker = "\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(tag = \"status\", rename_all = \"snake_case\")]\npub enum TemporaryTravelResolution"
if text.count(marker) != 1:
    raise SystemExit("temporary_mobility.rs: resolution marker changed")
config_block = r'''

/// World-independent immutable M9 experiment definition.
///
/// The focal region and schedule are fixed experiment inputs, while M9.4 routing is deliberately
/// resolved from each run's authoritative world. This prevents a travel table derived from one
/// synthetic seed from being silently reused against another seed's movement-cost field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityConfig {
    pub schema_version: u32,
    pub region: FocalRegion,
    pub schedule: TemporaryMobilitySchedule,
    pub travel_model: TemporaryTravelModel,
}

impl TemporaryMobilityConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        region: FocalRegion,
        schedule: TemporaryMobilitySchedule,
        travel_model: TemporaryTravelModel,
    ) -> Result<Self, TemporaryMobilityConfigError> {
        let config = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            region,
            schedule,
            travel_model,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), TemporaryMobilityConfigError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryMobilityConfigError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        self.region.validate_structure()?;
        self.schedule.validate()?;
        self.travel_model.validate()?;
        Ok(())
    }

    pub fn derive_program(
        &self,
        world: &World,
    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {
        self.validate()?;
        self.region.validate(world)?;
        let travel = self.travel_model.derive_table(&self.region, world)?;
        Ok(TemporaryMobilityProgram::new(
            self.region.clone(),
            self.schedule.clone(),
            travel,
            world,
        )?)
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_u64(&mut hash, self.region.digest64());
        self.schedule.digest_into(&mut hash);
        digest_str(&mut hash, &self.travel_model.identity());
        hash
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "temporary-mobility-config-v{}-{:016x}",
            self.schema_version,
            self.digest64()
        )
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityConfigError {
    #[error(
        "temporary-mobility configuration schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error(transparent)]
    Region(#[from] FocalRegionError),
    #[error(transparent)]
    Program(#[from] TemporaryMobilityProgramError),
    #[error(transparent)]
    TravelModel(#[from] TemporaryTravelModelError),
}
'''
text = text.replace(marker, config_block + marker, 1)
p.write_text(text)

patch(
    "crates/anthrosim-core/src/lib.rs",
    [
        (
            "    ActiveTemporaryJourney, HouseholdPresence, TemporaryJourneySkip, TemporaryMobilityDayOutcome,\n    TemporaryMobilityError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n",
            "    ActiveTemporaryJourney, HouseholdPresence, TemporaryJourneySkip, TemporaryMobilityConfig,\n    TemporaryMobilityConfigError, TemporaryMobilityDayOutcome, TemporaryMobilityError,\n    TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n",
        )
    ],
)

patch(
    "crates/anthrosim-core/src/simulation.rs",
    [
        (
            "        TemporaryMobilityExecutionError, TemporaryMobilityProgram, TemporaryMobilityProgramError,\n        TemporaryMobilityState, TemporaryMobilityValidationError,\n",
            "        TemporaryMobilityConfigError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n        TemporaryMobilityProgramError, TemporaryMobilityState, TemporaryMobilityValidationError,\n",
        ),
        (
            "        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let temporary_mobility = match program {\n            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,\n            None => TemporaryMobilityState::at_residence(&population),\n        };\n",
            "        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let configured_program = config\n            .temporary_mobility\n            .as_ref()\n            .map(|definition| definition.derive_program(&world))\n            .transpose()?;\n        let temporary_mobility = match (program, configured_program) {\n            (Some(_), Some(_)) => {\n                return Err(SimulationError::AmbiguousTemporaryMobilityConfiguration);\n            }\n            (Some(program), None) | (None, Some(program)) => {\n                TemporaryMobilityState::with_program(&population, program, &world)?\n            }\n            (None, None) => TemporaryMobilityState::at_residence(&population),\n        };\n",
        ),
        (
            "        if world.digest64() != checkpoint.world_digest64 {\n            return Err(SimulationError::CheckpointWorldDigestMismatch {\n                expected: checkpoint.world_digest64,\n                actual: world.digest64(),\n            });\n        }\n\n        checkpoint\n            .population\n",
            "        if world.digest64() != checkpoint.world_digest64 {\n            return Err(SimulationError::CheckpointWorldDigestMismatch {\n                expected: checkpoint.world_digest64,\n                actual: world.digest64(),\n            });\n        }\n        validate_configured_temporary_mobility(\n            &checkpoint.experiment,\n            &checkpoint.temporary_mobility,\n            &world,\n        )?;\n\n        checkpoint\n            .population\n",
        ),
        (
            "    validate_migration_config(&config.migration)?;\n    if let Some(evidence) = &config.evidence {\n",
            "    validate_migration_config(&config.migration)?;\n    if let Some(temporary_mobility) = &config.temporary_mobility {\n        temporary_mobility.validate()?;\n    }\n    if let Some(evidence) = &config.evidence {\n",
        ),
        (
            "    Ok(())\n}\n\nfn validate_terminal_checkpoint_state(\n",
            "    Ok(())\n}\n\nfn validate_configured_temporary_mobility(\n    config: &ExperimentConfig,\n    state: &TemporaryMobilityState,\n    world: &World,\n) -> Result<(), SimulationError> {\n    let Some(definition) = &config.temporary_mobility else {\n        // Explicit `new_with_temporary_mobility` remains available for isolated lifecycle tests.\n        // Ordinary experiment execution records a definition in ExperimentConfig and is checked\n        // below.\n        return Ok(());\n    };\n    let expected = definition.derive_program(world)?;\n    if state.program() != Some(&expected) {\n        return Err(SimulationError::ConfiguredTemporaryMobilityMismatch {\n            expected: expected.identity(),\n            actual: state.program().map(TemporaryMobilityProgram::identity),\n        });\n    }\n    Ok(())\n}\n\nfn validate_terminal_checkpoint_state(\n",
        ),
        (
            "    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n",
            "    #[error(\"both ExperimentConfig and an explicit constructor supplied temporary mobility\")]\n    AmbiguousTemporaryMobilityConfiguration,\n    #[error(\n        \"configured temporary-mobility program mismatch: expected {expected}, found {actual:?}\"\n    )]\n    ConfiguredTemporaryMobilityMismatch {\n        expected: String,\n        actual: Option<String>,\n    },\n    #[error(transparent)]\n    TemporaryMobilityConfig(#[from] TemporaryMobilityConfigError),\n    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n",
        ),
    ],
)

patch(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    [
        (
            "        TemporaryMobilityExecutionError, TemporaryMobilityProgram, TemporaryMobilityProgramError,\n        TemporaryMobilityState, TemporaryMobilityValidationError,\n",
            "        TemporaryMobilityConfigError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n        TemporaryMobilityProgramError, TemporaryMobilityState, TemporaryMobilityValidationError,\n",
        ),
        (
            "        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let temporary_mobility = match program {\n            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,\n            None => TemporaryMobilityState::at_residence(&population),\n        };\n",
            "        let population = Population::initialize(config.population, &world, rng_factory)?;\n        let configured_program = config\n            .temporary_mobility\n            .as_ref()\n            .map(|definition| definition.derive_program(&world))\n            .transpose()?;\n        let temporary_mobility = match (program, configured_program) {\n            (Some(_), Some(_)) => {\n                return Err(SpatialLandscapeError::AmbiguousTemporaryMobilityConfiguration);\n            }\n            (Some(program), None) | (None, Some(program)) => {\n                TemporaryMobilityState::with_program(&population, program, &world)?\n            }\n            (None, None) => TemporaryMobilityState::at_residence(&population),\n        };\n",
        ),
        (
            "    validate_migration_config(&config.migration)?;\n    if let Some(evidence) = &config.evidence {\n",
            "    validate_migration_config(&config.migration)?;\n    if let Some(temporary_mobility) = &config.temporary_mobility {\n        temporary_mobility.validate()?;\n    }\n    if let Some(evidence) = &config.evidence {\n",
        ),
        (
            "    checkpoint\n        .temporary_mobility\n        .validate_at_day(\n            checkpoint.time.days(),\n            &checkpoint.population,\n            world,\n        )\n        .map_err(|error| SpatialLandscapeError::TemporaryMobilityInvalid {\n            reason: error.to_string(),\n        })?;\n    Ok(())\n",
            "    checkpoint\n        .temporary_mobility\n        .validate_at_day(\n            checkpoint.time.days(),\n            &checkpoint.population,\n            world,\n        )\n        .map_err(|error| SpatialLandscapeError::TemporaryMobilityInvalid {\n            reason: error.to_string(),\n        })?;\n    if let Some(definition) = &checkpoint.experiment.temporary_mobility {\n        let expected = definition.derive_program(world)?;\n        if checkpoint.temporary_mobility.program() != Some(&expected) {\n            return Err(SpatialLandscapeError::ConfiguredTemporaryMobilityMismatch {\n                expected: expected.identity(),\n                actual: checkpoint\n                    .temporary_mobility\n                    .program()\n                    .map(TemporaryMobilityProgram::identity),\n            });\n        }\n    }\n    Ok(())\n",
        ),
        (
            "    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n",
            "    #[error(\"both ExperimentConfig and an explicit constructor supplied temporary mobility\")]\n    AmbiguousTemporaryMobilityConfiguration,\n    #[error(\n        \"configured temporary-mobility program mismatch: expected {expected}, found {actual:?}\"\n    )]\n    ConfiguredTemporaryMobilityMismatch {\n        expected: String,\n        actual: Option<String>,\n    },\n    #[error(transparent)]\n    TemporaryMobilityConfig(#[from] TemporaryMobilityConfigError),\n    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n",
        ),
    ],
)

TEST = ROOT / "crates/anthrosim-core/tests/temporary_mobility_experiment_identity.rs"
TEST.write_text(r'''use anthrosim_core::ids::CellId;
use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    PopulationConfig, ResourceConfig, Simulation, SimulationError, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
};

fn no_events_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn definition() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "generic-experiment-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4)],
    )
    .expect("region");
    let schedule = TemporaryMobilitySchedule::new(
        "generic-experiment-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        5,
    )
    .expect("schedule");
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility definition")
}

fn config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(24).with_target_household_size(4))
        .with_demography(no_events_demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(definition())
}

#[test]
fn ordinary_experiment_derives_and_preserves_world_specific_temporary_program() {
    let configured = config(96_101);
    let expected_definition = configured
        .temporary_mobility
        .clone()
        .expect("configured temporary mobility");
    let simulation = Simulation::new(configured.clone()).expect("configured simulation");
    let expected_program = expected_definition
        .derive_program(simulation.world())
        .expect("world-specific program");
    assert_eq!(
        simulation.temporary_mobility().program(),
        Some(&expected_program)
    );

    let run = simulation.run_recorded().expect("configured run");
    assert_eq!(run.manifest.experiment, configured);
    assert_eq!(run.checkpoint.experiment, configured);
    assert!(run.events().events.iter().any(|record| matches!(
        record.event,
        EventKind::TemporaryJourneyDeparted { .. }
    )));
}

#[test]
fn configured_program_is_rederived_for_each_seed_world() {
    let definition = definition();
    for seed in [96_102, 96_103] {
        let configured = ExperimentConfig::new(seed, 0)
            .with_world(WorldConfig::new(4, 1))
            .with_population(PopulationConfig::new(8).with_target_household_size(4))
            .with_temporary_mobility(definition.clone());
        let simulation = Simulation::new(configured).expect("configured simulation");
        let expected = definition
            .derive_program(simulation.world())
            .expect("program derived from this seed's world");
        assert_eq!(simulation.temporary_mobility().program(), Some(&expected));
    }
}

#[test]
fn resume_rejects_config_definition_that_no_longer_matches_authoritative_program() {
    let mut checkpoint = Simulation::new(config(96_104))
        .expect("configured simulation")
        .checkpoint_at_year(1)
        .expect("checkpoint");
    checkpoint
        .experiment
        .temporary_mobility
        .as_mut()
        .expect("definition")
        .schedule
        .stay_duration_days += 1;

    assert!(matches!(
        Simulation::from_checkpoint(checkpoint),
        Err(SimulationError::ConfiguredTemporaryMobilityMismatch { .. })
    ));
}

#[test]
fn explicit_program_and_config_definition_are_rejected_as_ambiguous() {
    let configured = config(96_105);
    let world_probe = Simulation::new(configured.clone()).expect("world probe");
    let program = configured
        .temporary_mobility
        .as_ref()
        .expect("definition")
        .derive_program(world_probe.world())
        .expect("program");

    assert!(matches!(
        Simulation::new_with_temporary_mobility(configured, program),
        Err(SimulationError::AmbiguousTemporaryMobilityConfiguration)
    ));
}
''')

print("patched M9.6 experiment identity and wrote acceptance tests")
