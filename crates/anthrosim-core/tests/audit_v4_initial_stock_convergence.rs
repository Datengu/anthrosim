use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, ResourceConfig, Simulation, WorldConfig,
};

fn no_event_demography() -> DemographyConfig {
    DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "audit-v4-area-g-no-events".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    }
}

fn run(seed: u64, years: u64, initial_stock: u32) -> (u64, u64, u64) {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(initial_stock)
        .with_annual_regeneration_units_per_productivity(1)
        .with_annual_need_units_per_person(0)
        .with_productivity_scale_permille(1_000);
    resources.periods_per_year = 1;
    resources.cell_stock_capacity_years = 10;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let record = Simulation::new(
        ExperimentConfig::new(seed, years)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(1).with_target_household_size(1))
            .with_demography(no_event_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    (
        record.manifest.resources.initial_food_stock,
        record.manifest.resources.regenerated_food,
        record.manifest.resources.final_food_stock,
    )
}

fn positive_productivity_seed() -> u64 {
    (1..=1_000)
        .find(|seed| run(*seed, 1, 10).0 > 0)
        .expect("controlled seed search must find a positive-productivity one-cell world")
}

#[test]
fn plausible_initial_stock_transient_converges_only_after_the_declared_regeneration_time() {
    let seed = positive_productivity_seed();

    let (empty_initial, empty_regen_one, empty_final_one) = run(seed, 1, 0);
    let (stocked_initial, stocked_regen_one, stocked_final_one) = run(seed, 1, 10);

    assert_eq!(empty_initial, 0);
    assert!(stocked_initial > 0);
    assert_eq!(empty_regen_one, stocked_regen_one);
    assert!(
        empty_final_one < stocked_final_one,
        "after one year the otherwise-identical runs must still retain the causal day-zero stock contrast"
    );

    let (_, empty_regen_ten, empty_final_ten) = run(seed, 10, 0);
    let (_, stocked_regen_ten, stocked_final_ten) = run(seed, 10, 10);

    assert_eq!(empty_regen_ten, stocked_regen_ten);
    assert_eq!(
        empty_final_ten, stocked_final_ten,
        "with zero demand, one unit of annual regeneration per productivity, and ten years of capacity, the empty and fully stocked starts should converge exactly only after the full ten-year refill horizon"
    );

    assert_eq!(
        empty_final_ten, stocked_initial,
        "the converged stock should equal the declared fully stocked day-zero state under this controlled capacity/regeneration construction"
    );
}
