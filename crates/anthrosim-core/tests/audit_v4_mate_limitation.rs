use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.schedule_id = "audit-v4-mate-limitation".to_owned();
    config.mortality_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 50, 1_000_000),
        AgeProbabilityBand::new(50, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founder_population(male_age_days: Option<i64>) -> FounderPopulationDefinition {
    let mut people = vec![FounderPerson {
        id: PersonId::new(1),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }];
    if let Some(age_days) = male_age_days {
        people.push(FounderPerson {
            id: PersonId::new(2),
            birth_day: -age_days,
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        });
    }

    FounderPopulationDefinition::new(
        "audit-v4-mate-limitation",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        people,
    )
}

fn births(male_age_days: Option<i64>) -> u64 {
    let initial_population = if male_age_days.is_some() { 2 } else { 1 };
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(75_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(initial_population)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(founder_population(male_age_days))
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .population
        .births_since_start
}

#[test]
fn fertility_requires_a_local_male_inside_exact_configured_age_bounds() {
    let cases = [
        (None, 0_u64, "no male"),
        (Some(18 * 365 - 1), 0, "one day below male lower bound"),
        (Some(18 * 365), 1, "exact male lower bound"),
        (Some(70 * 365 - 1), 1, "one day below male upper bound"),
        (Some(70 * 365), 0, "exact male upper bound"),
    ];

    for (male_age_days, expected_births, label) in cases {
        let observed = births(male_age_days);
        println!("case={label} male_age_days={male_age_days:?} births={observed}");
        assert_eq!(observed, expected_births, "mate-limitation mismatch at {label}");
    }
}
