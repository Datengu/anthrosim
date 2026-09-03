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
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config.male_birth_permille = 500;
    config
}

fn female(id: u64, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn male(id: u64, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let people = if swapped_labels {
        vec![female(1, 2), male(2, 2), female(3, 1), male(4, 1)]
    } else {
        vec![female(1, 1), male(2, 1), female(3, 2), male(4, 2)]
    };

    FounderPopulationDefinition::new(
        "newborn-sex-person-relabel",
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

fn run(seed: u64, swapped_labels: bool) -> Vec<(CellId, ReproductiveSex)> {
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
    let mut births = recorded
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::Birth {
                cell,
                reproductive_sex,
                ..
            } => Some((cell, reproductive_sex)),
            _ => None,
        })
        .collect::<Vec<_>>();
    births.sort_unstable_by_key(|(cell, sex)| {
        let sex_key = match sex {
            ReproductiveSex::Female => 0_u8,
            ReproductiveSex::Male => 1_u8,
        };
        (cell.0, sex_key)
    });
    births
}

#[test]
fn same_seed_newborn_sex_is_invariant_to_pure_founder_person_relabelling() {
    for seed in 1..=1_000 {
        let a = run(seed, false);
        let b = run(seed, true);
        assert_eq!(a.len(), 2, "forced fertility must produce two births in arm A at seed {seed}");
        assert_eq!(b.len(), 2, "forced fertility must produce two births in arm B at seed {seed}");
        assert_eq!(
            a, b,
            "scientifically identical unlabeled founder states attached different newborn-sex realizations to fixed birth cells under person-label permutation at seed {seed}: A={a:?}, B={b:?}"
        );
    }
}
