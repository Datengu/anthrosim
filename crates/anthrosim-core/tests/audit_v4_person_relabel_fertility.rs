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

fn person(
    id: u64,
    reproductive_sex: ReproductiveSex,
    household: u64,
) -> FounderPerson {
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

fn founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let people = if swapped_labels {
        // Same unlabeled population as the control, with the two household-local female/male
        // pairs exchanged across canonical person labels 1..4.
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
        if swapped_labels { "relabel-b" } else { "relabel-a" },
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

fn run(seed: u64, swapped_labels: bool) -> Vec<CellId> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(4)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(founders(swapped_labels))
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    recorded
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
        let a = run(seed, false);
        let b = run(seed, true);
        assert_eq!(
            a, b,
            "scientifically identical unlabeled founder states diverged under person-label permutation at seed {seed}: A={a:?}, B={b:?}"
        );
    }
}
