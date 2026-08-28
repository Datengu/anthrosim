use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, ResearchDimension, ResearchDimensionKind, ResearchExperimentDefinition,
    ResearchRunConfig, ResourceConfig, Simulation, WorldConfig,
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

fn simulation_with(seed: u64, resources: ResourceConfig) -> Simulation {
    Simulation::new(
        ExperimentConfig::new(seed, 1)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(1).with_target_household_size(1))
            .with_demography(no_event_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .unwrap()
}

fn positive_productivity_seed() -> u64 {
    (1..=1_000)
        .find(|seed| {
            simulation_with(*seed, ResourceConfig::synthetic_validation_v1())
                .resources()
                .total_food_stock()
                .is_ok_and(|stock| stock > 0)
        })
        .expect("controlled seed search must find a positive-productivity one-cell world")
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
    let seed = positive_productivity_seed();
    let mut ten_year_capacity =
        ResourceConfig::synthetic_validation_v1().with_initial_stock_units_per_productivity(5);
    ten_year_capacity.cell_stock_capacity_years = 10;

    let mut twenty_year_capacity = ten_year_capacity.clone();
    twenty_year_capacity.cell_stock_capacity_years = 20;

    let ten = simulation_with(seed, ten_year_capacity);
    let twenty = simulation_with(seed, twenty_year_capacity);
    assert_eq!(
        ten.resources().total_food_stock().unwrap(),
        twenty.resources().total_food_stock().unwrap(),
        "capacity may cap starting stock, but increasing capacity must not manufacture historical stock"
    );
}

#[test]
fn plausible_starting_stock_changes_early_scarcity_with_other_resource_rules_fixed() {
    let seed = positive_productivity_seed();
    let mut depleted = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(0)
        .with_annual_regeneration_units_per_productivity(0)
        .with_annual_need_units_per_person(100);
    depleted.periods_per_year = 1;
    depleted.max_scarcity_mortality_probability_per_million = 0;

    let stocked = depleted
        .clone()
        .with_initial_stock_units_per_productivity(10);

    let depleted_simulation = simulation_with(seed, depleted);
    let stocked_simulation = simulation_with(seed, stocked);
    assert_eq!(
        depleted_simulation.resources().total_food_stock().unwrap(),
        0
    );
    assert!(
        stocked_simulation.resources().total_food_stock().unwrap() > 0,
        "the controlled positive-productivity world must retain the declared stocked start"
    );

    let depleted_run = depleted_simulation.run_recorded().unwrap();
    let stocked_run = stocked_simulation.run_recorded().unwrap();
    assert!(
        depleted_run.manifest.resources.unmet_need > stocked_run.manifest.resources.unmet_need,
        "early scarcity must remain sensitive to the explicitly declared starting stock when regeneration and demand are held fixed"
    );
}

#[test]
fn research_definition_can_sweep_initial_stock_as_an_exact_numeric_dimension() {
    let base_experiment = ExperimentConfig::new(216_003, 1);
    let definition = ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![216_003],
        base: ResearchRunConfig {
            experiment: base_experiment.clone(),
            spatial: None,
        },
        dimensions: vec![ResearchDimension {
            id: "initial-stock".to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: "/experiment/resources/initialStockUnitsPerProductivity".to_owned(),
            values: vec![serde_json::json!(0), serde_json::json!(10)],
        }],
    };

    let points = definition.expand().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(
        points[0]
            .run_config
            .experiment
            .resources
            .initial_stock_units_per_productivity,
        0
    );
    assert_eq!(
        points[1]
            .run_config
            .experiment
            .resources
            .initial_stock_units_per_productivity,
        10
    );

    let mut expected_zero = base_experiment.clone();
    expected_zero.resources.initial_stock_units_per_productivity = 0;
    assert_eq!(points[0].run_config.experiment, expected_zero);
    assert_eq!(points[1].run_config.experiment, base_experiment);
    assert_ne!(points[0].point_id, points[1].point_id);
}
