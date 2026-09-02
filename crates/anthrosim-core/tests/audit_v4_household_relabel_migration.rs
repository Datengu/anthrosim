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

fn founders(swapped_household_labels: bool) -> FounderPopulationDefinition {
    let (households, people) = if swapped_household_labels {
        (
            vec![
                FounderHousehold { id: HouseholdId::new(1), location: CellId::new(4) },
                FounderHousehold { id: HouseholdId::new(2), location: CellId::new(1) },
            ],
            vec![
                FounderPerson {
                    id: PersonId::new(1), birth_day: -(30 * 365),
                    reproductive_sex: ReproductiveSex::Male, household: HouseholdId::new(2),
                    female_parent: None, male_parent: None, last_birth_day: None,
                    condition_permille: 500,
                },
                FounderPerson {
                    id: PersonId::new(2), birth_day: -(30 * 365),
                    reproductive_sex: ReproductiveSex::Male, household: HouseholdId::new(1),
                    female_parent: None, male_parent: None, last_birth_day: None,
                    condition_permille: 500,
                },
            ],
        )
    } else {
        (
            vec![
                FounderHousehold { id: HouseholdId::new(1), location: CellId::new(1) },
                FounderHousehold { id: HouseholdId::new(2), location: CellId::new(4) },
            ],
            vec![
                FounderPerson {
                    id: PersonId::new(1), birth_day: -(30 * 365),
                    reproductive_sex: ReproductiveSex::Male, household: HouseholdId::new(1),
                    female_parent: None, male_parent: None, last_birth_day: None,
                    condition_permille: 500,
                },
                FounderPerson {
                    id: PersonId::new(2), birth_day: -(30 * 365),
                    reproductive_sex: ReproductiveSex::Male, household: HouseholdId::new(2),
                    female_parent: None, male_parent: None, last_birth_day: None,
                    condition_permille: 500,
                },
            ],
        )
    };

    FounderPopulationDefinition::new(
        "migration-household-relabel",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
    )
}

fn migration() -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_candidate_radius_cells(3)
        .with_decision_periods_per_year(1);
    config.condition_pressure_threshold_permille = 1_000;
    config.resource_pressure_threshold_permille = 1_000;
    config.minimum_utility_improvement = 0;
    config.resource_weight = 10;
    config.water_security_weight = 2;
    config.kin_weight = 0;
    config.travel_cost_weight = 0;
    config.max_uncertainty_penalty_permille = 100;
    config.relocation_risk_base_penalty_permille = 0;
    config.relocation_risk_per_cell_permille = 0;
    config.travel_condition_cost_per_cell = 0;
    config
}

fn run(seed: u64, swapped_household_labels: bool) -> Vec<(CellId, CellId)> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(founders(swapped_household_labels))
        .with_demography(demography())
        .with_resources(ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(1_000))
        .with_migration(migration());

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut moves = recorded
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::HouseholdMigration { origin, destination, .. } => Some((origin, destination)),
            _ => None,
        })
        .collect::<Vec<_>>();
    moves.sort_unstable_by_key(|(origin, destination)| (origin.0, destination.0));
    moves
}

#[test]
fn same_seed_migration_is_invariant_to_pure_household_relabelling() {
    let mut informative_seeds = 0_u32;
    for seed in 1..=1_000 {
        let a = run(seed, false);
        let b = run(seed, true);
        if !a.is_empty() || !b.is_empty() {
            informative_seeds += 1;
        }
        assert_eq!(
            a, b,
            "scientifically identical unlabeled household states diverged under household-label permutation at seed {seed}: A={a:?}, B={b:?}"
        );
    }
    assert!(informative_seeds > 0, "adversary produced no migration decisions across the seed sweep");
}
