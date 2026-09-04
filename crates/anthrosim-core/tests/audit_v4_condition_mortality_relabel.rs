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
        band.annual_probability_per_million = 0;
    }
    config
}

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 0;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 150_000;
    config
}

fn person(id: u64, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 0,
    }
}

fn founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let people = if swapped_labels {
        vec![person(1, 2), person(2, 1)]
    } else {
        vec![person(1, 1), person(2, 2)]
    };
    FounderPopulationDefinition::new(
        "condition-mortality-person-relabel",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold { id: HouseholdId::new(1), location: CellId::new(1) },
            FounderHousehold { id: HouseholdId::new(2), location: CellId::new(2) },
        ],
        people,
    )
}

fn run(seed: u64, swapped_labels: bool) -> Vec<CellId> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(founders(swapped_labels))
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut death_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::Death { cell, .. } => Some(cell),
            _ => None,
        })
        .collect::<Vec<_>>();
    death_cells.sort_unstable_by_key(|cell| cell.0);
    death_cells
}

#[test]
fn same_seed_condition_mortality_is_invariant_to_pure_founder_person_relabelling() {
    let mut informative_seeds = 0_u32;
    for seed in 1..=1_000 {
        let a = run(seed, false);
        let b = run(seed, true);
        if a.len() == 1 || b.len() == 1 {
            informative_seeds += 1;
        }
        assert_eq!(
            a, b,
            "scientifically identical unlabeled founder states diverged under person-label permutation for condition-mediated mortality at seed {seed}: A={a:?}, B={b:?}"
        );
    }
    assert!(informative_seeds > 0, "adversary did not exercise asymmetric condition-mortality outcomes");
}
