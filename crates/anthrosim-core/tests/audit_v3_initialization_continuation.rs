use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, ResourceConfig, ResumeLineage, Simulation, WorldConfig,
};

fn no_event_demography() -> DemographyConfig {
    DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "audit-v3-area-g-no-events".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    }
}

fn config(initial_stock: u32) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(initial_stock)
        .with_annual_regeneration_units_per_productivity(0)
        .with_annual_need_units_per_person(0);
    resources.periods_per_year = 1;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(307_001, 5)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(8).with_target_household_size(2))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn without_resume_lineage(
    mut run: anthrosim_core::RecordedRun,
) -> anthrosim_core::RecordedRun {
    run.manifest.resume_lineage = ResumeLineage::new();
    run.checkpoint.resume_lineage = ResumeLineage::new();
    run.checkpoint = run.checkpoint.seal_continuation_identity();
    run
}

#[test]
fn alternative_initial_resource_states_remain_causal_and_resume_exactly() {
    let mut terminal_stocks = Vec::new();

    for initial_stock in [0_u32, 10_u32] {
        let cfg = config(initial_stock);
        let initial = Simulation::new(cfg.clone()).unwrap();
        let day_zero_stock = initial.resources().total_food_stock().unwrap();

        let uninterrupted = Simulation::new(cfg.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        let terminal_stock = uninterrupted
            .checkpoint
            .resources
            .total_food_stock()
            .unwrap();

        let checkpoint = Simulation::new(cfg)
            .unwrap()
            .checkpoint_at_year(2)
            .unwrap();
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();

        assert_eq!(
            without_resume_lineage(resumed),
            without_resume_lineage(uninterrupted.clone()),
            "checkpoint/resume must preserve the complete authoritative trajectory for each alternative initial state"
        );
        assert_eq!(
            terminal_stock, day_zero_stock,
            "with zero demand, zero regeneration and no demographic or migration events, elapsed time must not silently erase the declared initial resource state"
        );

        println!(
            "initialStockUnitsPerProductivity={initial_stock} dayZeroStock={day_zero_stock} terminalStock={terminal_stock} stateDigest64={}",
            uninterrupted.checkpoint.state_digest64
        );
        terminal_stocks.push(terminal_stock);
    }

    assert_eq!(terminal_stocks[0], 0);
    assert!(
        terminal_stocks[1] > terminal_stocks[0],
        "the stocked and depleted initial states must remain scientifically distinguishable after five years when no mechanism can erase their difference"
    );
}
