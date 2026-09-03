use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig,
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

fn run(periods_per_year: u16) -> (u64, u64, Option<u16>) {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = periods_per_year;
    resources.initial_stock_units_per_productivity = 0;
    resources.annual_regeneration_units_per_productivity = 0;
    resources.annual_need_units_per_person = 365;
    resources.seasonality_scale_permille = 0;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 200;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(78_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(20)
                .with_target_household_size(5)
                .with_max_person_records(100),
        )
        .with_demography(quiet_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    (
        run.manifest.resources.unmet_need,
        run.manifest.resources.final_food_stock,
        run.manifest.resources.mean_living_condition_permille,
    )
}

#[test]
fn annual_resource_and_condition_response_is_stable_across_m3_partitions() {
    let annual = run(1);
    let quarterly = run(4);
    let monthly = run(12);
    let daily = run(365);

    println!(
        "periods=1 unmet={} stock={} mean_condition={:?}",
        annual.0, annual.1, annual.2
    );
    println!(
        "periods=4 unmet={} stock={} mean_condition={:?}",
        quarterly.0, quarterly.1, quarterly.2
    );
    println!(
        "periods=12 unmet={} stock={} mean_condition={:?}",
        monthly.0, monthly.1, monthly.2
    );
    println!(
        "periods=365 unmet={} stock={} mean_condition={:?}",
        daily.0, daily.1, daily.2
    );

    assert_eq!(annual.0, quarterly.0);
    assert_eq!(annual.0, monthly.0);
    assert_eq!(annual.0, daily.0);
    assert_eq!(annual.1, quarterly.1);
    assert_eq!(annual.1, monthly.1);
    assert_eq!(annual.1, daily.1);
    assert_eq!(annual.2, quarterly.2);
    assert_eq!(annual.2, monthly.2);
    assert_eq!(annual.2, daily.2);
}
