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
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn definition(m1_id: u64, m2_id: u64, id: &str) -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    let mut people = vec![
        FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
        FounderPerson {
            id: PersonId::new(2),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        },
    ];

    let male = |person_id: u64| FounderPerson {
        id: PersonId::new(person_id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    };
    people.push(male(3));
    people.push(male(4));

    for child_id in 5..=7 {
        people.push(FounderPerson {
            id: PersonId::new(child_id),
            birth_day: -(10 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: Some(PersonId::new(1)),
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        });
    }
    people.push(FounderPerson {
        id: PersonId::new(8),
        birth_day: -(10 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household,
        female_parent: Some(PersonId::new(1)),
        male_parent: Some(PersonId::new(m1_id)),
        last_birth_day: None,
        condition_permille: 1_000,
    });

    // `m1_id` and `m2_id` identify two otherwise identical adult males. Their only scientific
    // difference is that M1 is the declared father of child 8. Reassigning which canonical ID
    // names M1 is therefore a pure consistent relabelling of the same unlabeled founder graph.
    assert_eq!([m1_id, m2_id].into_iter().min(), Some(3));
    assert_eq!([m1_id, m2_id].into_iter().max(), Some(4));

    FounderPopulationDefinition::new(
        id,
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        people,
    )
}

fn run(definition: FounderPopulationDefinition) -> anthrosim_core::RecordedRun {
    let config = ExperimentConfig::new(75_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(8).with_max_person_records(100))
        .with_founder_population(definition)
        .with_demography(quiet_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(4, 18),
        );
    Simulation::new(config).unwrap().run_recorded().unwrap()
}

fn co_resident_parent_count(run: &anthrosim_core::RecordedRun) -> usize {
    let population = &run.checkpoint.population;
    let child = population.person(PersonId::new(8)).unwrap();
    let child_household = child.household;
    [child.female_parent, child.male_parent]
        .into_iter()
        .filter(|&parent| parent != PersonId::INVALID)
        .filter(|&parent| {
            population
                .person(parent)
                .is_some_and(|person| person.household == child_household)
        })
        .count()
}

#[test]
fn dependency_fission_is_invariant_to_consistent_same_age_same_sex_parent_relabelling() {
    let m1_is_3 = run(definition(3, 4, "audit-v3-kin-relabel-a"));
    let m1_is_4 = run(definition(4, 3, "audit-v3-kin-relabel-b"));

    assert_eq!(m1_is_3.checkpoint.population.household_count(), 2);
    assert_eq!(m1_is_4.checkpoint.population.household_count(), 2);

    let a = co_resident_parent_count(&m1_is_3);
    let b = co_resident_parent_count(&m1_is_4);
    assert_eq!(
        a, b,
        "pure canonical-ID relabelling of otherwise identical adult males must not change the dependent child's retained parent co-residence"
    );
}
