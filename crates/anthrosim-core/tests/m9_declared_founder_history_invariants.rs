use anthrosim_core::ids::{CellId, HouseholdId, PersonId, TemporaryJourneyId};
use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, RecordedRun, ReproductiveSex,
    ResourceConfig, Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
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

fn founders(home_a: CellId, home_b: CellId) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "m9-declared-founder-history-replay-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: home_a,
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: home_b,
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(32 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn recorded_declared_founder_m9_run() -> RecordedRun {
    let home_a = CellId::new(1);
    let home_b = CellId::new(2);
    let region = FocalRegion::new(
        "m9-declared-founder-history-region",
        FocalRegionSource::Synthetic,
        vec![home_a],
    )
    .unwrap();
    let temporary_mobility = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            "m9-declared-founder-history-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![50],
            10,
        )
        .unwrap(),
        TemporaryTravelModel::new(
            "m9-declared-founder-history-travel",
            ParameterProvenance::SyntheticValidation,
            100_000,
            u16::MAX,
        )
        .unwrap(),
    )
    .unwrap();

    let config = ExperimentConfig::new(39_200, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(2).with_max_person_records(16))
        .with_founder_population(founders(home_a, home_b))
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility);

    Simulation::new(config).unwrap().run_recorded().unwrap()
}

#[test]
fn declared_founder_m9_history_replays_through_checkpoint_and_recorded_run_invariants() {
    let run = recorded_declared_founder_m9_run();
    assert!(
        run.events()
            .events
            .iter()
            .any(|record| { matches!(record.event, EventKind::TemporaryJourneyArrived { .. }) })
    );
    run.checkpoint.validate_invariants().unwrap();
    run.validate_invariants().unwrap();
}

#[test]
fn declared_founder_m9_replay_still_rejects_tampered_journey_identity() {
    let mut run = recorded_declared_founder_m9_run();
    let arrival = run
        .checkpoint
        .events
        .events
        .iter_mut()
        .find(|record| matches!(record.event, EventKind::TemporaryJourneyArrived { .. }))
        .expect("fixture must contain a temporary arrival");
    let EventKind::TemporaryJourneyArrived { journey, .. } = &mut arrival.event else {
        unreachable!();
    };
    *journey = TemporaryJourneyId::new(journey.0.saturating_add(10_000));

    let error = run
        .validate_invariants()
        .expect_err("tampered declared-founder M9 history must fail closed");
    assert!(
        error
            .to_string()
            .contains("temporary mobility event history is invalid"),
        "unexpected invariant error: {error}"
    );
}
