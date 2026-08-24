use anthrosim_core::ids::{CellId, HouseholdId};
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    ParameterProvenance, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityProgram,
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

fn config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(no_event_demography())
        .with_resources(one_period_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn program_for_stay(
    config: &ExperimentConfig,
    stay_duration_days: u32,
) -> (TemporaryMobilityProgram, CellId) {
    let probe = Simulation::new(config.clone()).unwrap();
    let household = HouseholdId::new(1);
    let residence = probe.population().household_location(household).unwrap();
    let destination = probe
        .world()
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            let candidate = CellId::new(index as u64 + 1);
            (candidate != residence && cell.food_stock >= 100).then_some(candidate)
        })
        .next()
        .expect("synthetic fixture should contain a stocked non-residence destination");
    let region = FocalRegion::new(
        format!("m9-5-stay-{stay_duration_days}"),
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        format!("m9-5-fast-travel-{stay_duration_days}"),
        ParameterProvenance::SyntheticValidation,
        1_000_000,
        u16::MAX,
    )
    .unwrap();
    let travel = model.derive_table(&region, probe.world()).unwrap();
    let program = TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            format!("m9-5-stay-schedule-{stay_duration_days}"),
            TemporaryTriggerTiming::DepartureDay,
            vec![20],
            stay_duration_days,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap();
    (program, destination)
}

fn destination_stock_shift(stay_duration_days: u32) -> (u64, u64) {
    let config = config(95_000 + u64::from(stay_duration_days));
    let (program, destination) = program_for_stay(&config, stay_duration_days);

    let disabled = Simulation::new(config.clone())
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let enabled = Simulation::new_with_temporary_mobility(config.clone(), program.clone())
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let replay = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();

    disabled.validate_invariants().unwrap();
    enabled.validate_invariants().unwrap();
    replay.validate_invariants().unwrap();
    assert_eq!(enabled.state_digest64, replay.state_digest64);

    let disabled_stock = disabled.resources.cell_food_stock(destination).unwrap();
    let enabled_stock = enabled.resources.cell_food_stock(destination).unwrap();
    (disabled_stock, enabled_stock)
}

#[test]
fn one_day_visit_between_resource_boundaries_exerts_exact_destination_demand() {
    let (disabled_stock, enabled_stock) = destination_stock_shift(1);
    assert_eq!(disabled_stock.checked_sub(enabled_stock), Some(1));
}

#[test]
fn five_day_visit_between_resource_boundaries_exerts_five_days_of_destination_demand() {
    let (disabled_stock, enabled_stock) = destination_stock_shift(5);
    assert_eq!(disabled_stock.checked_sub(enabled_stock), Some(5));
}
