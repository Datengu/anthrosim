use crate::{
    config::{
        AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig,
        ParameterProvenance, PopulationConfig, ResourceConfig, WorldConfig,
    },
    demography_observability::derive_demography_observability,
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
    simulation::Simulation,
    time::DAYS_PER_YEAR,
};

#[test]
fn synthetic_run_replays_exact_opportunity_funnel_and_spacing_semantics() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(91_337, 8)
        .with_world(WorldConfig::new(8, 8))
        .with_population(PopulationConfig::new(240).with_max_person_records(10_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let simulation = Simulation::new(config).expect("fixture should initialize");
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().expect("fixture should complete");
    let report = derive_demography_observability(&initial_population, &recorded.checkpoint)
        .expect("authoritative artifacts should replay exactly");

    assert_eq!(report.requested_birth_spacing_days, 1_278);
    assert_eq!(report.effective_birth_spacing_days, 1_460);
    assert!(report.fertility_probability_is_conditional_on_m2_survival);
    assert!(report.parentage_uses_pre_same_day_m4_residence);
    assert_eq!(
        report.summary.successful_births,
        recorded.checkpoint.population.summary().births_since_start
    );
    assert_eq!(
        report.summary.fertility_draws_attempted,
        report
            .summary
            .fertility_draw_successes
            .saturating_add(report.summary.stochastic_draw_failures)
    );
    assert_eq!(
        report.summary.fertility_draw_successes,
        report
            .summary
            .successful_births
            .saturating_add(report.summary.record_limit_blocked_births)
    );
    assert!(!report.fertility_stage_truncated_by_record_limit);
}

#[test]
fn partial_year_resource_extinction_replays_without_inventing_m2_exposure() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.productivity_scale_permille = 0;
    resources.annual_need_units_per_person = u32::MAX;
    resources.max_condition_loss_per_period = 1_000;
    resources.max_scarcity_mortality_probability_per_million = 1_000_000;
    let config = ExperimentConfig::new(91_338, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(4).with_max_person_records(100))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let simulation = Simulation::new(config).expect("fixture should initialize");
    let initial_population = simulation.population().clone();
    let recorded = simulation
        .run_recorded()
        .expect("resource-extinction fixture should terminate cleanly");

    assert!(recorded.checkpoint.time.days() < DAYS_PER_YEAR);
    assert!(
        !recorded
            .checkpoint
            .time
            .days()
            .is_multiple_of(DAYS_PER_YEAR)
    );
    assert_eq!(recorded.checkpoint.population.living_count(), 0);

    let report = derive_demography_observability(&initial_population, &recorded.checkpoint)
        .expect("partial-year terminal events should replay exactly");
    assert_eq!(report.simulated_days, recorded.checkpoint.time.days());
    assert_eq!(report.annual_boundaries_observed, 0);
    assert_eq!(report.summary.mortality_exposures, 0);
    assert_eq!(report.summary.demographic_deaths, 0);
    assert_eq!(report.summary.fertility_draws_attempted, 0);
    assert_eq!(report.summary.final_living_population, 0);
}

#[test]
fn total_mortality_removes_fertility_stage_exposure_under_declared_contract() {
    let household = HouseholdId::new(1);
    let founders = FounderPopulationDefinition::new(
        "observability-total-mortality",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(20_i64 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Female,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(20_i64 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    );
    let demography = DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "total-mortality-total-fertility".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 1_000_000)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 1_000_000)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 0,
        male_parent_max_age_years_exclusive: u32::MAX,
    };
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(7, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(2).with_max_person_records(100))
        .with_founder_population(founders)
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let simulation = Simulation::new(config).expect("fixture should initialize");
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().expect("fixture should complete");
    let report = derive_demography_observability(&initial_population, &recorded.checkpoint)
        .expect("authoritative artifacts should replay exactly");

    assert_eq!(report.summary.mortality_exposures, 2);
    assert_eq!(report.summary.demographic_deaths, 2);
    assert_eq!(report.summary.surviving_females_entering_fertility, 0);
    assert_eq!(report.summary.fertility_draws_attempted, 0);
    assert_eq!(report.summary.successful_births, 0);
    assert_eq!(report.summary.final_living_population, 0);
}
