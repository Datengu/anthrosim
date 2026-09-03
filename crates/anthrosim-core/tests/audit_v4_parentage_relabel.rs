use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn founder(
    id: u64,
    age_years: i64,
    sex: ReproductiveSex,
    male_parent: Option<u64>,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(age_years * 365),
        reproductive_sex: sex,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: male_parent.map(PersonId::new),
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(swapped_male_labels: bool) -> (FounderPopulationDefinition, PersonId) {
    // The two arms preserve the same unlabeled genealogy: one 35-year-old male is the father of
    // the 10-year-old founder and the other 35-year-old male is unrelated. Only those males'
    // canonical PersonIds are exchanged, and the child's parent link is updated consistently.
    let (father_id, unrelated_id) = if swapped_male_labels { (3, 2) } else { (2, 3) };
    let people = vec![
        founder(1, 30, ReproductiveSex::Female, None),
        if father_id == 2 {
            founder(2, 35, ReproductiveSex::Male, None)
        } else {
            founder(2, 35, ReproductiveSex::Male, None)
        },
        if unrelated_id == 3 {
            founder(3, 35, ReproductiveSex::Male, None)
        } else {
            founder(3, 35, ReproductiveSex::Male, None)
        },
        founder(4, 10, ReproductiveSex::Male, Some(father_id)),
    ];

    (
        FounderPopulationDefinition::new(
            "parentage-person-relabel",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::Unspecified,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            people,
        ),
        PersonId::new(father_id),
    )
}

fn selected_existing_father(seed: u64, swapped_male_labels: bool) -> bool {
    let (founders, existing_father) = founders(swapped_male_labels);
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(4)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(founders)
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let births = recorded
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::Birth { male_parent, .. } => Some(male_parent),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(births.len(), 1, "forced fertility must produce exactly one adult-female birth");
    births[0] == existing_father
}

#[test]
fn same_seed_parentage_is_invariant_to_pure_male_person_relabelling() {
    for seed in 1..=1_000 {
        let a = selected_existing_father(seed, false);
        let b = selected_existing_father(seed, true);
        assert_eq!(
            a, b,
            "scientifically identical unlabeled genealogy changed whether the newborn selected the already-parent male after pure male PersonId relabelling at seed {seed}: A_existing_father={a}, B_existing_father={b}"
        );
    }
}
