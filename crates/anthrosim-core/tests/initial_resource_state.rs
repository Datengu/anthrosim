use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, ResourceConfig, Simulation, WorldConfig,
};

fn no_event_demography() -> DemographyConfig {
    DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "issue-216-no-events".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    }
}

fn run_with(resources: ResourceConfig) -> anthrosim_core::RecordedRun {
    Simulation::new(
        ExperimentConfig::new(216_001, 1)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(1).with_target_household_size(1))
            .with_demography(no_event_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .unwrap()
    .run_recorded()
    .unwrap()
}

#[test]
fn synthetic_baseline_declares_the_historical_full_stock_assumption() {
    let resources = ResourceConfig::synthetic_validation_v1();
    assert_eq!(resources.schema_version, 5);
    assert_eq!(resources.initial_stock_units_per_productivity, 10);

    let encoded = serde_json::to_value(ExperimentConfig::new(216_002, 1)).unwrap();
    assert_eq!(encoded["resources"]["initialStockUnitsPerProductivity"], 10);
}

#[test]
fn storage_capacity_does_not_create_additional_initial_stock() {
    let mut ten_year_capacity = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(5);
    ten_year_capacity.cell_stock_capacity_years = 10;

    let mut twenty_year_capacity = ten_year_capacity.clone();
    twenty_year_capacity.cell_stock_capacity_years = 20;

    let ten = run_with(ten_year_capacity);
    let twenty = run_with(twenty_year_capacity);
    assert_eq!(
        ten.manifest.resources.initial_food_stock,
        twenty.manifest.resources.initial_food_stock,
        "capacity may cap starting stock, but increasing capacity must not manufacture historical stock"
    );
}

#[test]
fn plausible_starting_stock_changes_early_scarcity_with_other_resource_rules_fixed() {
    let mut depleted = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(0)
        .with_annual_regeneration_units_per_productivity(0)
        .with_annual_need_units_per_person(100);
    depleted.periods_per_year = 1;
    depleted.max_scarcity_mortality_probability_per_million = 0;

    let stocked = depleted
        .clone()
        .with_initial_stock_units_per_productivity(10);

    let depleted_run = run_with(depleted);
    let stocked_run = run_with(stocked);

    assert_eq!(depleted_run.manifest.resources.initial_food_stock, 0);
    assert!(stocked_run.manifest.resources.initial_food_stock > 0);
    assert!(
        depleted_run.manifest.resources.unmet_need > stocked_run.manifest.resources.unmet_need,
        "early scarcity must remain sensitive to the explicitly declared starting stock when regeneration and demand are held fixed"
    );
}
