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
        band.annual_probability_per_million = 500_000;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn person(id: u64, reproductive_sex: ReproductiveSex, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn two_household_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let people = if swapped_labels {
        vec![
            person(1, ReproductiveSex::Female, 2),
            person(2, ReproductiveSex::Male, 2),
            person(3, ReproductiveSex::Female, 1),
            person(4, ReproductiveSex::Male, 1),
        ]
    } else {
        vec![
            person(1, ReproductiveSex::Female, 1),
            person(2, ReproductiveSex::Male, 1),
            person(3, ReproductiveSex::Female, 2),
            person(4, ReproductiveSex::Male, 2),
        ]
    };

    FounderPopulationDefinition::new(
        if swapped_labels {
            "relabel-b"
        } else {
            "relabel-a"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(2),
            },
        ],
        people,
    )
}

fn three_household_founders(rotated_labels: bool) -> FounderPopulationDefinition {
    let physical_households = if rotated_labels {
        [2_u64, 3, 1]
    } else {
        [1_u64, 2, 3]
    };
    let mut people = Vec::new();
    for (pair_index, household) in physical_households.into_iter().enumerate() {
        let female_id = u64::try_from(pair_index * 2 + 1).unwrap();
        let male_id = female_id + 1;
        people.push(person(female_id, ReproductiveSex::Female, household));
        people.push(person(male_id, ReproductiveSex::Male, household));
    }
    FounderPopulationDefinition::new(
        if rotated_labels {
            "relabel-three-b"
        } else {
            "relabel-three-a"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(2),
            },
            FounderHousehold {
                id: HouseholdId::new(3),
                location: CellId::new(3),
            },
        ],
        people,
    )
}

fn run(
    seed: u64,
    founders: FounderPopulationDefinition,
    population: u32,
    width: u32,
) -> Vec<CellId> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(width, 1))
        .with_population(
            PopulationConfig::new(population)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(32),
        )
        .with_founder_population(founders)
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::Birth { cell, .. } => Some(cell),
            _ => None,
        })
        .collect()
}

#[test]
fn same_seed_fertility_is_invariant_to_pure_founder_person_relabelling() {
    for seed in 1..=1_000 {
        let a = run(seed, two_household_founders(false), 4, 2);
        let b = run(seed, two_household_founders(true), 4, 2);
        assert_eq!(
            a, b,
            "scientifically identical unlabeled founder states diverged under person-label permutation at seed {seed}: A={a:?}, B={b:?}"
        );
    }
}

#[test]
fn fertility_assignment_survives_a_three_household_cyclic_person_relabelling() {
    for seed in 1..=256 {
        let a = run(seed, three_household_founders(false), 6, 3);
        let b = run(seed, three_household_founders(true), 6, 3);
        assert_eq!(
            a, b,
            "three-household fertility attribution diverged under cyclic person relabelling at seed {seed}: A={a:?}, B={b:?}"
        );
    }
}
