use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig, validate_recorded_run_invariants,
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

fn resources(initial_stock_units_per_productivity: u32) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 4;
    config.annual_need_units_per_person = 4;
    config.annual_regeneration_units_per_productivity = 0;
    config.initial_stock_units_per_productivity = initial_stock_units_per_productivity;
    config.cell_stock_capacity_years = 100;
    config.seasonality_scale_permille = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn run(seed: u64, initial_stock_units_per_productivity: u32) -> anthrosim_core::RecordedRun {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(quiet_demography())
        .with_resources(resources(initial_stock_units_per_productivity))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();
    run
}

#[test]
fn excess_nonbinding_initial_stock_does_not_change_condition_when_realized_supply_is_identical() {
    let mut exercised = 0_u32;

    for seed in 81_000..81_200 {
        let low = run(seed, 1);
        let high = run(seed, 10);
        let low_summary = low.checkpoint.resources.summary(&low.checkpoint.population);
        let high_summary = high.checkpoint.resources.summary(&high.checkpoint.population);

        if low_summary.unmet_need != 0 || high_summary.unmet_need != 0 {
            continue;
        }

        exercised += 1;
        assert_eq!(low_summary.consumed_food, high_summary.consumed_food);
        assert_eq!(low_summary.unmet_need, high_summary.unmet_need);
        assert_eq!(
            low_summary.mean_living_condition_permille,
            high_summary.mean_living_condition_permille,
            "once both arms realize the same complete supply, extra nonbinding initial stock must not alter the condition response"
        );
    }

    assert!(
        exercised >= 20,
        "fixture must exercise at least 20 seeds where both initial-stock arms fully satisfy realized demand; observed {exercised}"
    );
}
