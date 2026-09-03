use anthrosim_core::ids::{CellId, HouseholdId};
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    ParameterProvenance, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
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

fn one_period_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 365;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn base_config() -> ExperimentConfig {
    ExperimentConfig::new(96_401, 1)
        .with_world(WorldConfig::new(5, 5))
        .with_population(PopulationConfig::new(2).with_target_household_size(1))
        .with_demography(no_event_demography())
        .with_resources(one_period_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn shared_destination_program(config: &ExperimentConfig) -> (TemporaryMobilityConfig, CellId) {
    let probe = Simulation::new(config.clone()).unwrap();
    let residences = [HouseholdId::new(1), HouseholdId::new(2)]
        .map(|household| probe.population().household_location(household).unwrap());
    let destination = probe
        .world()
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            let candidate = CellId::new(index as u64 + 1);
            (!residences.contains(&candidate) && cell.food_stock >= 100).then_some(candidate)
        })
        .next()
        .expect("synthetic fixture should provide a stocked shared destination");

    let region = FocalRegion::new(
        "audit-v4-area-f-shared-destination",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "audit-v4-area-f-fast-travel",
        ParameterProvenance::SyntheticValidation,
        1_000_000,
        u16::MAX,
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v4-area-f-five-day-aggregation",
        TemporaryTriggerTiming::DepartureDay,
        vec![20],
        5,
    )
    .unwrap();

    (
        TemporaryMobilityConfig::new(region, schedule, model).unwrap(),
        destination,
    )
}

#[test]
fn simultaneous_two_household_aggregation_preserves_exact_combined_resource_pressure() {
    let config = base_config();
    let (temporary_mobility, destination) = shared_destination_program(&config);

    let disabled = Simulation::new(config.clone())
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let enabled = Simulation::new(config.with_temporary_mobility(temporary_mobility))
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();

    disabled.validate_invariants().unwrap();
    enabled.validate_invariants().unwrap();

    let disabled_stock = disabled.resources.cell_food_stock(destination).unwrap();
    let enabled_stock = enabled.resources.cell_food_stock(destination).unwrap();
    assert_eq!(
        disabled_stock.checked_sub(enabled_stock),
        Some(10),
        "two one-person households aggregated for five days should exert exactly ten person-days of extra demand at the shared destination"
    );

    let observation = enabled
        .resources
        .period_observations()
        .last()
        .expect("one-period resource configuration should preserve one observation");
    assert_eq!(observation.visitor_need, 10);
    assert_eq!(observation.home_need, 720);
    assert_eq!(observation.total_need, 730);
}
