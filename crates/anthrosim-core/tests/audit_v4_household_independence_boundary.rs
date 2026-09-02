use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, HouseholdLifecycleConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn quiet_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn quiet_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn definition(second_anchor_age_at_day_365: i64) -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    let day_zero_age = second_anchor_age_at_day_365 - 365;
    let mut people = vec![
        FounderPerson {
            id: PersonId::new(1),
            birth_day: -(40 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(2),
            birth_day: -day_zero_age,
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
    ];

    for id in 3..=6 {
        people.push(FounderPerson {
            id: PersonId::new(id),
            birth_day: -(8 * 365),
            reproductive_sex: if id % 2 == 0 {
                ReproductiveSex::Female
            } else {
                ReproductiveSex::Male
            },
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        });
    }

    FounderPopulationDefinition::new(
        "audit-v4-household-independence-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        people,
    )
}

fn household_count(second_anchor_age_at_day_365: i64) -> usize {
    let config = ExperimentConfig::new(76_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(6).with_max_person_records(50))
        .with_founder_population(definition(second_anchor_age_at_day_365))
        .with_demography(quiet_demography())
        .with_resources(quiet_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(3, 18),
        );

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .checkpoint
        .population
        .household_count()
}

#[test]
fn dependency_fission_changes_only_at_exact_independent_age_boundary() {
    let below = 18 * 365 - 1;
    let exact = 18 * 365;
    let above = 18 * 365 + 1;

    let below_count = household_count(below);
    let exact_count = household_count(exact);
    let above_count = household_count(above);

    println!(
        "day365_second_anchor_age_days below={below}:{below_count} exact={exact}:{exact_count} above={above}:{above_count}"
    );

    assert_eq!(below_count, 1, "one independent anchor must defer fission");
    assert_eq!(exact_count, 2, "exactly age 18 must qualify as an independent anchor");
    assert_eq!(above_count, 2, "one day above age 18 must remain independently eligible");
}
