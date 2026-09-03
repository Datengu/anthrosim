use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, RngStreamPosition,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography(male_birth_permille: u16) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config.minimum_birth_spacing_days = 0;
    // Founder males remain the only eligible fathers in the two-year propagation test. That keeps
    // this regression isolated from AV4-005 parentage candidate ordering while allowing a female
    // newborn from year one to propagate her sex assignment into year-two demography.
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 100;
    config.male_birth_permille = male_birth_permille;
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
        "newborn-sex-two-household-relabel",
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
    let households_by_pair = if rotated_labels {
        [2_u64, 3_u64, 1_u64]
    } else {
        [1_u64, 2_u64, 3_u64]
    };
    let mut people = Vec::with_capacity(6);
    for (pair_index, household) in households_by_pair.into_iter().enumerate() {
        let first_id = u64::try_from(pair_index * 2 + 1).unwrap();
        people.push(person(first_id, ReproductiveSex::Female, household));
        people.push(person(first_id + 1, ReproductiveSex::Male, household));
    }

    FounderPopulationDefinition::new(
        "newborn-sex-three-household-rotation",
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

#[derive(Debug, PartialEq, Eq)]
struct PhysicalOutcome {
    births: Vec<(u64, CellId, ReproductiveSex)>,
    living_by_cell_and_sex: Vec<[u64; 2]>,
    demography_rng: [RngStreamPosition; 4],
}

fn sex_key(sex: ReproductiveSex) -> usize {
    match sex {
        ReproductiveSex::Female => 0,
        ReproductiveSex::Male => 1,
    }
}

fn run(
    seed: u64,
    years: u32,
    width: u32,
    initial_population: u32,
    founders: FounderPopulationDefinition,
    male_birth_permille: u16,
) -> PhysicalOutcome {
    let config = ExperimentConfig::new(seed, years)
        .with_world(WorldConfig::new(width, 1))
        .with_population(
            PopulationConfig::new(initial_population)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(64),
        )
        .with_founder_population(founders)
        .with_demography(demography(male_birth_permille))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut births = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth {
                cell,
                reproductive_sex,
                ..
            } => Some((record.day, cell, reproductive_sex)),
            _ => None,
        })
        .collect::<Vec<_>>();
    births.sort_unstable_by_key(|(day, cell, sex)| (*day, cell.0, sex_key(*sex)));

    let population = &recorded.checkpoint.population;
    let mut living_by_cell_and_sex = vec![[0_u64; 2]; usize::try_from(width).unwrap()];
    for raw_id in 1..=u64::try_from(population.person_count()).unwrap() {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if !person.is_alive() {
            continue;
        }
        let cell_index = usize::try_from(person.location.0 - 1).unwrap();
        living_by_cell_and_sex[cell_index][sex_key(person.reproductive_sex)] += 1;
    }

    let rng = &recorded.checkpoint.rng;
    PhysicalOutcome {
        births,
        living_by_cell_and_sex,
        demography_rng: [
            rng.demography_mortality,
            rng.demography_fertility,
            rng.demography_parentage,
            rng.demography_newborn_sex,
        ],
    }
}

fn run_two_households(
    seed: u64,
    years: u32,
    swapped_labels: bool,
    male_birth_permille: u16,
) -> PhysicalOutcome {
    run(
        seed,
        years,
        2,
        4,
        two_household_founders(swapped_labels),
        male_birth_permille,
    )
}

fn run_three_households(seed: u64, rotated_labels: bool) -> PhysicalOutcome {
    run(seed, 1, 3, 6, three_household_founders(rotated_labels), 500)
}

#[test]
fn original_two_household_newborn_sex_relabel_sweep_is_invariant() {
    for seed in 1..=1_000 {
        let a = run_two_households(seed, 1, false, 500);
        let b = run_two_households(seed, 1, true, 500);
        assert_eq!(
            a.births.len(),
            2,
            "forced fertility must produce two births at seed {seed}"
        );
        assert_eq!(
            b.births.len(),
            2,
            "forced fertility must produce two births after relabelling at seed {seed}"
        );
        assert_eq!(
            a, b,
            "newborn-sex physical outcomes or demography RNG positions changed under pure founder PersonId relabelling at seed {seed}"
        );
    }
}

#[test]
fn three_household_cyclic_person_relabelling_preserves_newborn_sex_assignment() {
    for seed in 1..=256 {
        assert_eq!(
            run_three_households(seed, false),
            run_three_households(seed, true),
            "three-household newborn-sex assignment changed under cyclic founder PersonId relabelling at seed {seed}"
        );
    }
}

#[test]
fn newborn_sex_relabel_invariance_propagates_through_two_demographic_years() {
    for seed in 1..=256 {
        assert_eq!(
            run_two_households(seed, 2, false, 500),
            run_two_households(seed, 2, true, 500),
            "year-one newborn-sex relabelling changed downstream physical demographic state at seed {seed}"
        );
    }
}

#[test]
fn configured_newborn_sex_probability_endpoints_remain_exact_and_label_invariant() {
    for (male_birth_permille, expected_sex) in [
        (0_u16, ReproductiveSex::Female),
        (1_000_u16, ReproductiveSex::Male),
    ] {
        for seed in 1..=64 {
            let a = run_two_households(seed, 1, false, male_birth_permille);
            let b = run_two_households(seed, 1, true, male_birth_permille);
            assert_eq!(
                a, b,
                "probability endpoint changed under relabelling at seed {seed}"
            );
            assert_eq!(a.births.len(), 2);
            assert!(
                a.births.iter().all(|(_, _, sex)| *sex == expected_sex),
                "configured maleBirthPermille={male_birth_permille} produced an unexpected newborn sex at seed {seed}: {:?}",
                a.births
            );
        }
    }
}
