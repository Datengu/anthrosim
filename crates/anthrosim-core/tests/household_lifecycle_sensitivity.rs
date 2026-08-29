use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    HouseholdLifecycleConfig, MigrationConfig, Population, PopulationConfig, ResourceConfig,
    Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, World, WorldConfig, derive_household_observability,
    derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId},
    rng::RngFactory,
    validate_temporary_mobility_history,
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

fn base_config(seed: u64, duration_years: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, duration_years)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(12).with_target_household_size(12))
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn deterministic_size_fission_balances_caps_and_records_household_ages() {
    let config = base_config(20701, 1).with_household_lifecycle(
        HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18),
    );
    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    let report = derive_household_observability(
        &run.checkpoint.population,
        &run.checkpoint.experiment,
        &run.checkpoint.events,
        run.checkpoint.time.days(),
    )
    .unwrap();
    assert_eq!(report.active_households, 3);
    assert_eq!(report.largest_living_household_size, 4);
    assert_eq!(
        report
            .living_household_size_distribution
            .iter()
            .map(|bin| (bin.living_members, bin.household_count))
            .collect::<Vec<_>>(),
        vec![(4, 3)]
    );
    assert_eq!(
        report
            .living_household_age_distribution
            .iter()
            .map(|bin| (bin.age_days, bin.household_count))
            .collect::<Vec<_>>(),
        vec![(0, 2), (365, 1)]
    );
    assert_eq!(run.manifest.population.living_population, 12);
    assert_eq!(run.manifest.population.births_since_start, 0);
    assert_eq!(run.manifest.population.deaths_since_start, 0);
    assert_eq!(
        run.checkpoint
            .events
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::HouseholdFission { .. }))
            .count(),
        2
    );
}

#[test]
fn lifecycle_is_exactly_deterministic_and_checkpoint_resumable() {
    let config = base_config(20702, 3).with_household_lifecycle(
        HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18),
    );
    let first = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let duplicate = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    assert_eq!(
        first.checkpoint.state_digest64,
        duplicate.checkpoint.state_digest64
    );
    assert_eq!(first.checkpoint.population, duplicate.checkpoint.population);
    assert_eq!(first.checkpoint.events, duplicate.checkpoint.events);

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    assert_eq!(
        first.checkpoint.state_digest64,
        resumed.checkpoint.state_digest64
    );
    assert_eq!(first.checkpoint.population, resumed.checkpoint.population);
    assert_eq!(first.checkpoint.events, resumed.checkpoint.events);
}

#[test]
fn fissioned_households_are_auditable_independent_future_m9_participants() {
    let seed = 20703;
    let base = base_config(seed, 2);
    let factory = RngFactory::new(seed);
    let world = World::generate(base.world, factory).unwrap();
    let initial_population = Population::initialize(base.population, &world, factory).unwrap();
    let residence = initial_population
        .household_location(HouseholdId::new(1))
        .unwrap();
    let destination = (1..=world.cell_count() as u64)
        .map(CellId::new)
        .find(|&cell| cell != residence)
        .unwrap();
    let mobility = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "issue-207-test-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "issue-207-two-year-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![100, 465],
            3,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();

    let baseline = Simulation::new(base.clone().with_temporary_mobility(mobility.clone()))
        .unwrap()
        .run_recorded()
        .unwrap();
    let fission = Simulation::new(
        base.with_temporary_mobility(mobility)
            .with_household_lifecycle(
                HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18),
            ),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let departures = |events: &anthrosim_core::EventLog| {
        events
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
            .count()
    };
    assert_eq!(departures(&baseline.checkpoint.events), 2);
    assert_eq!(departures(&fission.checkpoint.events), 4);
    assert_eq!(fission.checkpoint.population.household_count(), 3);

    let observability =
        derive_temporary_mobility_observability(&world, &initial_population, &fission.checkpoint)
            .unwrap();
    assert_eq!(observability.summary.journeys_started, 4);
    assert!(observability.summary.visitor_person_days > 0);
    assert!(observability.summary.peak_visitors > 0);
    validate_temporary_mobility_history(&world, &fission.checkpoint).unwrap();
}
