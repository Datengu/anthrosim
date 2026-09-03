use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, HouseholdLifecycleConfig,
    MigrationConfig, Population, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, World, WorldConfig,
    ids::{CellId, HouseholdId},
    rng::RngFactory,
};

fn quiet_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn quiet_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn base(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(12).with_target_household_size(12))
        .with_demography(quiet_demography())
        .with_resources(quiet_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18),
        )
}

fn mobility(
    config: &ExperimentConfig,
    departure_day: u64,
    duration_days: u32,
) -> TemporaryMobilityConfig {
    let factory = RngFactory::new(config.seed);
    let world = World::generate(config.world, factory).unwrap();
    let population = Population::initialize(config.population, &world, factory).unwrap();
    let residence = population
        .household_location(HouseholdId::new(1))
        .unwrap();
    let destination = (1..=world.cell_count() as u64)
        .map(CellId::new)
        .find(|&cell| cell != residence)
        .unwrap();

    TemporaryMobilityConfig::new(
        FocalRegion::new(
            "audit-v4-household-presence-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "audit-v4-household-presence-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![departure_day],
            duration_days,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn household_count_at_year_end(departure_day: u64, duration_days: u32) -> usize {
    let config = base(77_001);
    let mobility = mobility(&config, departure_day, duration_days);
    Simulation::new(config.with_temporary_mobility(mobility))
        .unwrap()
        .run_recorded()
        .unwrap()
        .checkpoint
        .population
        .household_count()
}

#[test]
fn temporary_absence_on_the_annual_boundary_defers_household_fission() {
    // Leave enough clearance for outbound travel, the three-day stay and return travel to finish
    // well before the annual boundary in the control arm.
    let returned_before_boundary = household_count_at_year_end(350, 3);
    let away_on_boundary = household_count_at_year_end(364, 3);

    println!(
        "households_at_day365 returned_before_boundary={returned_before_boundary} away_on_boundary={away_on_boundary}"
    );

    assert_eq!(returned_before_boundary, 3);
    assert_eq!(
        away_on_boundary, 1,
        "a household temporarily away at the annual lifecycle boundary is ineligible for fission"
    );
}
