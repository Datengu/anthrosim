from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one exact replacement target, found {count}")
    target.write_text(text.replace(old, new, 1))


simulation = "crates/anthrosim-core/src/simulation.rs"
replace_once(
    simulation,
    """    /// Construct a core simulation with an explicit M9 temporary-mobility program.
    pub fn new_with_temporary_mobility(
        config: ExperimentConfig,
        program: TemporaryMobilityProgram,
    ) -> Result<Self, SimulationError> {
        Self::new_internal(config, Some(program))
    }
""",
    """    /// Test-only seam for isolated lifecycle mechanics that need a resolved M9 program.
    /// Production callers must configure temporary mobility through `ExperimentConfig`.
    #[cfg(test)]
    pub(crate) fn new_with_temporary_mobility(
        config: ExperimentConfig,
        program: TemporaryMobilityProgram,
    ) -> Result<Self, SimulationError> {
        Self::new_internal(config, Some(program))
    }
""",
)
replace_once(
    simulation,
    """    let Some(definition) = &config.temporary_mobility else {
        // Explicit `new_with_temporary_mobility` remains available for isolated lifecycle tests.
        // Ordinary experiment execution records a definition in ExperimentConfig and is checked
        // below.
        return Ok(());
    };
""",
    """    let Some(definition) = &config.temporary_mobility else {
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
""",
)

spatial = "crates/anthrosim-core/src/spatial_simulation.rs"
replace_once(
    spatial,
    """    pub fn new(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
    ) -> Result<Self, SpatialLandscapeError> {
        Self::new_internal(config, landscape, mechanisms, None)
    }

    /// Construct a transformed-landscape simulation with the same authoritative M9 temporary-
    /// mobility program supported by the core host.
    pub fn new_with_temporary_mobility(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
        program: TemporaryMobilityProgram,
    ) -> Result<Self, SpatialLandscapeError> {
        Self::new_internal(config, landscape, mechanisms, Some(program))
    }

    fn new_internal(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
        program: Option<TemporaryMobilityProgram>,
    ) -> Result<Self, SpatialLandscapeError> {
""",
    """    pub fn new(
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
""",
)
replace_once(
    spatial,
    """        let temporary_mobility = match (program, configured_program) {
            (Some(_), Some(_)) => {
                return Err(SpatialLandscapeError::AmbiguousTemporaryMobilityConfiguration);
            }
            (Some(program), None) | (None, Some(program)) => {
                TemporaryMobilityState::with_program(&population, program, &world)?
            }
            (None, None) => TemporaryMobilityState::at_residence(&population),
        };
""",
    """        let temporary_mobility = match configured_program {
            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,
            None => TemporaryMobilityState::at_residence(&population),
        };
""",
)
replace_once(
    spatial,
    """    if let Some(definition) = &checkpoint.experiment.temporary_mobility {
        let expected = definition.derive_program(world)?;
        if checkpoint.temporary_mobility.program() != Some(&expected) {
            return Err(SpatialLandscapeError::ConfiguredTemporaryMobilityMismatch {
                expected: expected.identity(),
                actual: checkpoint
                    .temporary_mobility
                    .program()
                    .map(TemporaryMobilityProgram::identity),
            });
        }
    }
""",
    """    if let Some(definition) = &checkpoint.experiment.temporary_mobility {
        let expected = definition.derive_program(world)?;
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
""",
)
replace_once(
    spatial,
    """    #[error("both ExperimentConfig and an explicit constructor supplied temporary mobility")]
    AmbiguousTemporaryMobilityConfiguration,
""",
    "",
)

integration = "crates/anthrosim-core/src/m9_integration_tests.rs"
replace_once(
    integration,
    """    simulation::Simulation,
""",
    """    simulation::{Simulation, SimulationError},
""",
)
replace_once(
    integration,
    """#[test]
fn active_presence_round_trips_through_checkpoint_integrity() {
    let checkpoint = active_checkpoint(9_001, 2);
    let source_presence = checkpoint.temporary_mobility.clone();
    let source_digest = checkpoint.state_digest64;

    checkpoint.validate_invariants().unwrap();
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();

    assert_eq!(resumed.temporary_mobility, source_presence);
    assert_eq!(resumed.state_digest64, source_digest);
    resumed.validate_invariants().unwrap();
}
""",
    """#[test]
fn unconfigured_temporary_presence_is_rejected_by_resume() {
    let checkpoint = active_checkpoint(9_001, 2);

    assert!(matches!(
        Simulation::from_checkpoint(checkpoint),
        Err(SimulationError::ConfiguredTemporaryMobilityMismatch { .. })
    ));
}
""",
)
replace_once(
    integration,
    """#[test]
fn active_temporary_household_is_excluded_from_m4_without_changing_residence() {
    let seed = 9_002;
    let baseline = Simulation::new(
        ExperimentConfig::new(seed, 1)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(20).with_target_household_size(5))
            .with_demography(stable_demography())
            .with_resources(stable_resources()),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let checkpoint = active_checkpoint(seed, 1);
    let household = HouseholdId::new(1);
    let residence = checkpoint.population.household_location(household).unwrap();
    let active = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        baseline.manifest.migration.households_evaluated
            - active.manifest.migration.households_evaluated,
        active.manifest.migration.decision_boundaries
    );
    assert_eq!(
        active.checkpoint.population.household_location(household),
        Some(residence)
    );
    assert_eq!(
        active
            .checkpoint
            .temporary_mobility
            .is_at_residence(household),
        Some(false)
    );
    active.validate_invariants().unwrap();
}
""",
    """#[test]
fn temporary_households_are_excluded_from_m4_without_changing_residence() {
    let seed = 9_002;
    let config = m9_config(seed, 1, 20, stable_demography());
    let probe = Simulation::new(config.clone()).unwrap();
    let household = HouseholdId::new(1);
    let residence = probe.population().household_location(household).unwrap();
    let household_count = probe.population().household_count() as u64;

    let baseline = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let program = temporary_program(
        &config,
        TemporaryTriggerTiming::DepartureDay,
        vec![0],
        400,
        0,
        true,
    );
    let active = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        baseline.manifest.migration.households_evaluated,
        household_count.saturating_mul(baseline.manifest.migration.decision_boundaries)
    );
    assert_eq!(active.manifest.migration.households_evaluated, 0);
    assert_eq!(
        active.checkpoint.population.household_location(household),
        Some(residence)
    );
    assert_eq!(
        active
            .checkpoint
            .temporary_mobility
            .is_at_residence(household),
        Some(false)
    );
}
""",
)
replace_once(
    integration,
    """    let config = m9_config(9_003, 2, 40, stable_demography());
    let program = temporary_program(
        &config,
        TemporaryTriggerTiming::DepartureDay,
        vec![360],
        20,
        10,
        true,
    );

    let uninterrupted = Simulation::new_with_temporary_mobility(config.clone(), program.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let checkpoint = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
""",
    """    let base_config = m9_config(9_003, 2, 40, stable_demography());
    let probe = Simulation::new(base_config.clone()).unwrap();
    let destination = unoccupied_destination(&probe);
    let definition = crate::TemporaryMobilityConfig::new(
        FocalRegion::new(
            "m9-integration-resume-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "m9-integration-resume-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![300],
            100,
        )
        .unwrap(),
        crate::TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();
    let config = base_config.with_temporary_mobility(definition);

    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
""",
)
