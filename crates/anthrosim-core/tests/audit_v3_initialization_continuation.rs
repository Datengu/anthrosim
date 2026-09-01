use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, ResumeLineage,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
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

fn founder(condition_permille: u16) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        format!("audit-v3-area-g-condition-{condition_permille}"),
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(0),
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30_i64 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille,
        }],
    )
}

fn config(condition_permille: u16) -> ExperimentConfig {
    let mut resources =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(307_001, 5)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_founder_population(founder(condition_permille))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn without_resume_lineage(mut run: anthrosim_core::RecordedRun) -> anthrosim_core::RecordedRun {
    run.manifest.resume_lineage = ResumeLineage::new();
    run.checkpoint.resume_lineage = ResumeLineage::new();
    run.checkpoint = run.checkpoint.seal_continuation_identity();
    run
}

#[test]
fn alternative_founder_conditions_remain_causal_and_resume_exactly() {
    let mut terminal_conditions = Vec::new();

    for initial_condition in [400_u16, 900_u16] {
        let cfg = config(initial_condition);
        let initial = Simulation::new(cfg.clone()).unwrap();
        let day_zero_condition = initial
            .population()
            .mean_living_condition_permille()
            .unwrap();

        let uninterrupted = Simulation::new(cfg.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        let terminal_condition = uninterrupted
            .checkpoint
            .population
            .mean_living_condition_permille()
            .unwrap();

        let checkpoint = Simulation::new(cfg).unwrap().checkpoint_at_year(2).unwrap();
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();

        assert_eq!(
            without_resume_lineage(resumed),
            without_resume_lineage(uninterrupted.clone()),
            "checkpoint/resume must preserve the complete authoritative trajectory for each alternative founder condition"
        );
        assert_eq!(
            terminal_condition, day_zero_condition,
            "with mortality, fertility, migration, resource need, condition loss and condition recovery disabled, elapsed time must not silently erase founder condition"
        );

        println!(
            "initialConditionPermille={initial_condition} dayZeroCondition={day_zero_condition} terminalCondition={terminal_condition} stateDigest64={}",
            uninterrupted.checkpoint.state_digest64
        );
        terminal_conditions.push(terminal_condition);
    }

    assert_eq!(terminal_conditions, vec![400, 900]);
}
