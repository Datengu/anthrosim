use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, SimulationCheckpoint, WorldConfig, validate_recorded_run_invariants,
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

fn run(periods_per_year: u16) -> SimulationCheckpoint {
    let config = ExperimentConfig::new(50_409, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(no_event_demography())
        .with_resources(capacity_stress_resources(periods_per_year))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();
    run.checkpoint
}

#[test]
fn finite_capacity_resolution_curve_is_conserving_and_monotone_toward_daily_endpoint() {
    let periods = [1_u16, 2, 4, 12, 52, 365];
    let mut curve = Vec::new();

    for p in periods {
        let checkpoint = run(p);
        let summary = checkpoint.resources.summary(&checkpoint.population);
        let annual_need = checkpoint
            .resources
            .period_observations()
            .iter()
            .map(|period| period.total_need)
            .sum::<u64>();

        assert_eq!(annual_need, 1_000);
        assert_eq!(summary.harvested_food + summary.unmet_need, 1_000);
        assert_eq!(
            summary.initial_food_stock + summary.regenerated_food,
            summary.harvested_food + summary.final_food_stock
        );
        assert_eq!(summary.final_food_stock, 0);

        curve.push((
            p,
            summary.initial_food_stock,
            summary.regenerated_food,
            summary.harvested_food,
            summary.unmet_need,
        ));
    }

    assert_eq!(curve[0].1, 350);
    assert_eq!(curve[0].3, 350);
    assert_eq!(curve.last().unwrap().3, 700);

    for pair in curve.windows(2) {
        assert!(
            pair[1].2 >= pair[0].2,
            "regeneration should not reverse downward as this controlled capacity-stress clock is refined: {pair:?}"
        );
        assert!(
            pair[1].3 >= pair[0].3,
            "harvest should not reverse downward as this controlled capacity-stress clock is refined: {pair:?}"
        );
        assert!(
            pair[1].4 <= pair[0].4,
            "unmet need should not reverse upward as this controlled capacity-stress clock is refined: {pair:?}"
        );
    }

    let daily_harvest = curve.last().unwrap().3;
    eprintln!("finite-capacity cadence curve:");
    for (p, initial, regenerated, harvested, unmet) in &curve {
        let gap_to_daily = daily_harvest - harvested;
        let gap_permille = gap_to_daily * 1_000 / daily_harvest;
        eprintln!(
            "P={p}: initial={initial}, regenerated={regenerated}, harvested={harvested}, unmet={unmet}, gap_to_daily={gap_to_daily} ({gap_permille} permille)"
        );
    }
}
