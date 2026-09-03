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

fn migration() -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_candidate_radius_cells(4)
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

fn person(id: u64, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 500,
    }
}

fn two_household_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let (household_one_cell, household_two_cell, person_one_household, person_two_household) =
        if swapped_labels { (4, 1, 2, 1) } else { (1, 4, 1, 2) };
    FounderPopulationDefinition::new(
        if swapped_labels {
            "migration-relabel-b"
        } else {
            "migration-relabel-a"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(household_one_cell),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(household_two_cell),
            },
        ],
        vec![
            person(1, person_one_household),
            person(2, person_two_household),
        ],
    )
}

fn base_resources() -> ResourceConfig {
    ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(1_000)
}

#[derive(Debug, PartialEq, Eq)]
struct OneYearOutcome {
    moves: Vec<(CellId, CellId)>,
    choice_rng: anthrosim_core::RngStreamPosition,
    uncertainty_rng: anthrosim_core::RngStreamPosition,
}

fn run_two_household_one_year(seed: u64, swapped_labels: bool) -> OneYearOutcome {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography())
        .with_resources(base_resources())
        .with_migration(migration());
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut moves = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::HouseholdMigration {
                origin,
                destination,
                ..
            } => Some((origin, destination)),
            _ => None,
        })
        .collect::<Vec<_>>();
    moves.sort_unstable_by_key(|(origin, destination)| (origin.0, destination.0));
    OneYearOutcome {
        moves,
        choice_rng: recorded.checkpoint.rng.migration_choice,
        uncertainty_rng: recorded.checkpoint.rng.migration_uncertainty,
    }
}

#[test]
fn migration_moves_and_rng_positions_are_household_label_invariant() {
    let mut informative = 0_u32;
    for seed in 1..=1_000 {
        let a = run_two_household_one_year(seed, false);
        let b = run_two_household_one_year(seed, true);
        if !a.moves.is_empty() || !b.moves.is_empty() {
            informative += 1;
        }
        assert_eq!(
            a, b,
            "M4 migration diverged under pure HouseholdId relabelling at seed {seed}"
        );
    }
    assert!(informative > 0, "regression did not exercise a migration outcome");
}

fn three_household_founders(rotation: u64) -> FounderPopulationDefinition {
    let physical_cells = [1_u64, 3, 5];
    let mut locations_by_label = [CellId::INVALID; 3];
    let mut people = Vec::with_capacity(3);
    for (physical_index, &cell) in physical_cells.iter().enumerate() {
        let physical = u64::try_from(physical_index).unwrap();
        let label = ((physical + rotation) % 3) + 1;
        locations_by_label[usize::try_from(label - 1).unwrap()] = CellId::new(cell);
        people.push(person(physical + 1, label));
    }
    FounderPopulationDefinition::new(
        format!("migration-household-cycle-{rotation}"),
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        (1..=3)
            .map(|label| FounderHousehold {
                id: HouseholdId::new(label),
                location: locations_by_label[usize::try_from(label - 1).unwrap()],
            })
            .collect(),
        people,
    )
}

fn run_three_households(seed: u64, rotation: u64) -> Vec<(CellId, CellId)> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(5, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(three_household_founders(rotation))
        .with_demography(demography())
        .with_resources(base_resources())
        .with_migration(migration());
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut moves = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::HouseholdMigration {
                origin,
                destination,
                ..
            } => Some((origin, destination)),
            _ => None,
        })
        .collect::<Vec<_>>();
    moves.sort_unstable_by_key(|(origin, destination)| (origin.0, destination.0));
    moves
}

#[test]
fn migration_is_invariant_to_three_household_cyclic_relabelling() {
    for seed in 1..=256 {
        let baseline = run_three_households(seed, 0);
        assert_eq!(
            baseline,
            run_three_households(seed, 1),
            "rotation 1 diverged at seed {seed}"
        );
        assert_eq!(
            baseline,
            run_three_households(seed, 2),
            "rotation 2 diverged at seed {seed}"
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DownstreamOutcome {
    moves: Vec<(u64, CellId, CellId)>,
    living_by_cell: [u64; 4],
    food_stock: [u64; 4],
}

fn run_downstream(seed: u64, swapped_labels: bool) -> DownstreamOutcome {
    let resources = base_resources()
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(4, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(migration());
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut moves = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::HouseholdMigration {
                origin,
                destination,
                ..
            } => Some((record.day, origin, destination)),
            _ => None,
        })
        .collect::<Vec<_>>();
    moves.sort_unstable_by_key(|(day, origin, destination)| (*day, origin.0, destination.0));

    let population = &recorded.checkpoint.population;
    let mut living_by_cell = [0_u64; 4];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if person.death_day.is_none() {
            living_by_cell[usize::try_from(person.location.0 - 1).unwrap()] += 1;
        }
    }
    let resources = &recorded.checkpoint.resources;
    let food_stock = std::array::from_fn(|index| {
        resources
            .cell_food_stock(CellId::new(u64::try_from(index).unwrap() + 1))
            .unwrap()
    });
    DownstreamOutcome {
        moves,
        living_by_cell,
        food_stock,
    }
}

#[test]
fn household_relabelling_does_not_propagate_into_later_spatial_resource_state() {
    for seed in 1..=256 {
        assert_eq!(
            run_downstream(seed, false),
            run_downstream(seed, true),
            "downstream state diverged under pure HouseholdId relabelling at seed {seed}"
        );
    }
}
