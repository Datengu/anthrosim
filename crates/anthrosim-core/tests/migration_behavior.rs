use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig, candidate_count_upper_bound,
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

fn migration_experiment(enabled: bool) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(300)
        .with_annual_need_units_per_person(1_000);
    resources.max_scarcity_mortality_probability_per_million = 0;

    let mut migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(enabled)
        .with_candidate_radius_cells(2);
    migration.condition_pressure_threshold_permille = 1_000;
    migration.resource_pressure_threshold_permille = 1_000;
    migration.minimum_utility_improvement = 0;
    migration.resource_weight = 10;
    migration.water_security_weight = 3;
    migration.kin_weight = 0;
    migration.travel_cost_weight = 0;
    migration.max_uncertainty_penalty_permille = 0;
    migration.relocation_risk_base_penalty_permille = 0;
    migration.relocation_risk_per_cell_permille = 0;
    migration.travel_condition_cost_per_cell = 10;
    migration.max_recorded_decision_traces = 512;

    ExperimentConfig::new(4_204_204, 1)
        .with_world(WorldConfig::new(32, 32))
        .with_population(PopulationConfig::new(1_000).with_max_person_records(2_000))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(migration)
}

#[test]
fn pressure_driven_migration_completes_local_auditable_moves() {
    let config = migration_experiment(true);
    let radius = config.migration.candidate_radius_cells;
    let manifest = Simulation::new(config).unwrap().run().unwrap();

    assert!(manifest.migration.moves_completed > 0);
    assert!(manifest.migration.people_moved >= manifest.migration.moves_completed);
    assert!(manifest.migration.total_distance_cells >= manifest.migration.moves_completed);
    assert!(manifest.migration.travel_condition_cost_total > 0);
    assert!(!manifest.migration.recorded_decision_traces.is_empty());

    for trace in &manifest.migration.recorded_decision_traces {
        assert!(trace.distance_cells > 0);
        assert!(trace.distance_cells <= radius);
        assert!(usize::from(trace.candidate_count) <= candidate_count_upper_bound(radius));
        assert!(trace.destination_utility.total_utility > trace.origin_utility.total_utility);
        assert_eq!(trace.decision_day, trace.completed_day);
        assert!(trace.people_moved > 0);
    }
}

#[test]
fn enabling_migration_changes_spatial_history_under_otherwise_equal_inputs() {
    let enabled = Simulation::new(migration_experiment(true))
        .unwrap()
        .run()
        .unwrap();
    let disabled = Simulation::new(migration_experiment(false))
        .unwrap()
        .run()
        .unwrap();

    assert!(enabled.migration.moves_completed > 0);
    assert_eq!(disabled.migration.moves_completed, 0);
    assert_eq!(
        enabled.population.living_population,
        disabled.population.living_population
    );
    assert_ne!(enabled.population.digest64, disabled.population.digest64);
    assert_ne!(enabled.resources.digest64, disabled.resources.digest64);
}
