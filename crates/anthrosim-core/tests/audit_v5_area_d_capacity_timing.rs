use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig, validate_recorded_run_invariants,
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

fn capacity_stress_resources(periods_per_year: u16) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = periods_per_year;
    config.annual_need_units_per_person = 1_000;
    config.annual_regeneration_units_per_productivity = 1;
    config.initial_stock_units_per_productivity = 1;
    config.productivity_scale_permille = 1_000;
    config.seasonality_scale_permille = 0;
    config.cell_stock_capacity_years = 1;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn run(periods_per_year: u16) -> anthrosim_core::RecordedSimulationRun {
    let config = ExperimentConfig::new(50_403, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(no_event_demography())
        .with_resources(capacity_stress_resources(periods_per_year))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();
    run
}

#[test]
fn finite_capacity_timing_effect_is_explicit_and_accounting_conserving() {
    let annual = run(1);
    let daily = run(365);

    let annual_summary = annual
        .checkpoint
        .resources
        .summary(&annual.checkpoint.population);
    let daily_summary = daily
        .checkpoint
        .resources
        .summary(&daily.checkpoint.population);

    let annual_periods = annual.checkpoint.resources.period_observations();
    let daily_periods = daily.checkpoint.resources.period_observations();

    assert_eq!(annual_periods.len(), 1);
    assert_eq!(daily_periods.len(), 365);
    assert_eq!(annual_summary.initial_food_stock, daily_summary.initial_food_stock);
    assert!(annual_summary.initial_food_stock > 0);

    let annual_need = annual_periods.iter().map(|period| period.total_need).sum::<u64>();
    let daily_need = daily_periods.iter().map(|period| period.total_need).sum::<u64>();
    assert_eq!(annual_need, 1_000);
    assert_eq!(daily_need, 1_000);

    // With a one-year capacity and day-zero stock already at that cap, the one-period arm
    // attempts all regeneration before its only harvest and therefore clips it all. In the
    // daily arm, early harvest creates storage room for later regeneration. This trajectory
    // difference is explicitly permitted by the v20 response-time contract; the adversary
    // quantifies it while requiring exact resource accounting in both arms.
    assert_eq!(annual_summary.regenerated_food, 0);
    assert!(daily_summary.regenerated_food > 0);

    for (label, summary) in [("annual", &annual_summary), ("daily", &daily_summary)] {
        assert_eq!(
            summary.initial_food_stock + summary.regenerated_food,
            summary.harvested_food + summary.final_food_stock,
            "{label} arm must conserve stock + regeneration = harvest + terminal stock"
        );
        assert_eq!(
            summary.harvested_food + summary.unmet_need,
            1_000,
            "{label} arm must conserve annual demand = supplied + unmet"
        );
    }

    eprintln!(
        "finite-capacity timing: initial_stock={}; P=1 regenerated={}, harvested={}, unmet={}, final_stock={}; P=365 regenerated={}, harvested={}, unmet={}, final_stock={}",
        annual_summary.initial_food_stock,
        annual_summary.regenerated_food,
        annual_summary.harvested_food,
        annual_summary.unmet_need,
        annual_summary.final_food_stock,
        daily_summary.regenerated_food,
        daily_summary.harvested_food,
        daily_summary.unmet_need,
        daily_summary.final_food_stock,
    );
}
