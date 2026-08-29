use anthrosim_core::{
    DemographyConfig, ExperimentConfig, HouseholdLifecycleConfig, MigrationConfig, PopulationConfig,
    ResourceConfig, Simulation, WorldConfig,
    ids::HouseholdId,
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

fn migration_pressure_config() -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(300)
        .with_annual_need_units_per_person(1_000);
    resources.max_scarcity_mortality_probability_per_million = 0;

    let mut migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
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
    migration.travel_condition_cost_per_cell = 0;
    migration.max_recorded_decision_traces = 2_048;

    ExperimentConfig::new(324_004, 3)
        .with_world(WorldConfig::new(32, 32))
        .with_population(PopulationConfig::new(12).with_target_household_size(12))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(migration)
        .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_dependency_fission_v2(
            5, 18,
        ))
}

#[test]
fn daughter_households_enter_future_m4_decisions_after_dependency_aware_fission() {
    let run = Simulation::new(migration_pressure_config())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert!(run.checkpoint.population.household_count() >= 2);
    assert!(run.manifest.migration.moves_completed > 0);
    assert!(
        run.manifest
            .migration
            .recorded_decision_traces
            .iter()
            .any(|trace| trace.household != HouseholdId::new(1)),
        "at least one daughter household must participate in a recorded M4 move after fission"
    );
}
