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

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.seasonality_scale_permille = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn founders(genealogy_status: FounderGenealogyStatus) -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    let people = vec![
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
            birth_day: -(40 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(3),
            birth_day: -(10 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(4),
            birth_day: -(10 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(5),
            birth_day: -(8 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(6),
            birth_day: -(8 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
    ];

    FounderPopulationDefinition::new(
        "audit-v6-area-g-unspecified-genealogy-lifecycle",
        ParameterProvenance::SyntheticValidation,
        genealogy_status,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        people,
    )
}

fn config(
    genealogy_status: FounderGenealogyStatus,
    dependency_fission: bool,
) -> ExperimentConfig {
    let mut config = ExperimentConfig::new(720_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(6).with_max_person_records(20))
        .with_founder_population(founders(genealogy_status))
        .with_demography(quiet_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    if dependency_fission {
        config = config.with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(3, 18),
        );
    }
    config
}

#[test]
fn dependency_fission_must_not_treat_unspecified_founder_parent_links_as_known_absence() {
    // Negative control: with no relationship-sensitive household lifecycle active, the existing
    // founder contract deliberately permits epistemically incomplete direct-parent state.
    assert!(
        Simulation::new(config(FounderGenealogyStatus::Unspecified, false)).is_ok(),
        "unspecified founder genealogy should remain admissible when no configured mechanism consumes missing parent links"
    );

    // Positive control: the same parent-link pattern is scientifically interpretable when its
    // completeness metadata explicitly states that omitted living direct parents are known absent.
    let complete = Simulation::new(config(
        FounderGenealogyStatus::CompleteLivingDirectParents,
        true,
    ))
    .unwrap()
    .run_recorded()
    .unwrap();
    assert_eq!(complete.checkpoint.population.household_count(), 2);

    // Scientific oracle: dependency-aware fission explicitly uses parent relationships to define
    // anchor/dependent relationship roles and allocation. Under `Unspecified`, omitted links are
    // documented as *not* meaning absence of living direct kin, so this configuration must fail
    // closed instead of silently executing the relationship-sensitive rule on unknown genealogy.
    assert!(
        Simulation::new(config(FounderGenealogyStatus::Unspecified, true)).is_err(),
        "dependency-aware household fission must require founder genealogy completeness before missing parent links can be interpreted as structural absence"
    );
}
