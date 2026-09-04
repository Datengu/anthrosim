use anthrosim_core::{
    DeathCause, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, RngStreamPosition, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography(background_probability_per_million: u32) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = background_probability_per_million;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn resources(condition_probability_per_million: u32, annual_need: u32) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(annual_need)
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    config.periods_per_year = 1;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = condition_probability_per_million;
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

fn two_household_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        if swapped_labels {
            "condition-relabel-b"
        } else {
            "condition-relabel-a"
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
        if swapped_labels {
            vec![person(1, 2), person(2, 1)]
        } else {
            vec![person(1, 1), person(2, 2)]
        },
    )
}

#[derive(Debug, PartialEq, Eq)]
struct OneYearOutcome {
    deaths: Vec<(CellId, DeathCause)>,
    background_rng: RngStreamPosition,
    condition_rng: RngStreamPosition,
}

fn run_one_year(
    seed: u64,
    swapped_labels: bool,
    condition_probability: u32,
    background_probability: u32,
) -> OneYearOutcome {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(background_probability))
        .with_resources(resources(condition_probability, 0))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut deaths = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Death { cell, cause, .. } => Some((cell, cause)),
            _ => None,
        })
        .collect::<Vec<_>>();
    deaths.sort_unstable_by_key(|(cell, _)| cell.0);
    OneYearOutcome {
        deaths,
        background_rng: recorded.checkpoint.rng.demography_mortality,
        condition_rng: recorded.checkpoint.rng.resource_scarcity_mortality,
    }
}

#[test]
fn condition_mortality_cells_and_rng_positions_are_person_label_invariant() {
    let mut informative = 0_u32;
    for seed in 1..=1_000 {
        let a = run_one_year(seed, false, 150_000, 0);
        let b = run_one_year(seed, true, 150_000, 0);
        if a.deaths.len() == 1 || b.deaths.len() == 1 {
            informative += 1;
        }
        assert_eq!(
            a, b,
            "condition mortality diverged under pure PersonId relabelling at seed {seed}"
        );
    }
    assert!(
        informative > 0,
        "condition-mortality adversary never produced an asymmetric outcome"
    );
}

fn three_household_founders(rotation: u64) -> FounderPopulationDefinition {
    let household_for_label = |label: u64| ((label - 1 + rotation) % 3) + 1;
    FounderPopulationDefinition::new(
        format!("condition-cycle-{rotation}"),
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
        vec![
            person(1, household_for_label(1)),
            person(2, household_for_label(2)),
            person(3, household_for_label(3)),
        ],
    )
}

fn three_cell_deaths(seed: u64, rotation: u64) -> Vec<CellId> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(three_household_founders(rotation))
        .with_demography(demography(0))
        .with_resources(resources(350_000, 0))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Death { cell, .. } => Some(cell),
            _ => None,
        })
        .collect::<Vec<_>>();
    cells.sort_unstable_by_key(|cell| cell.0);
    cells
}

#[test]
fn condition_mortality_is_invariant_to_three_person_cyclic_relabelling() {
    for seed in 1..=256 {
        let baseline = three_cell_deaths(seed, 0);
        assert_eq!(
            baseline,
            three_cell_deaths(seed, 1),
            "rotation 1 diverged at seed {seed}"
        );
        assert_eq!(
            baseline,
            three_cell_deaths(seed, 2),
            "rotation 2 diverged at seed {seed}"
        );
    }
}

#[test]
fn simultaneous_condition_and_background_cause_attribution_is_label_invariant() {
    let mut saw_condition = false;
    let mut saw_background = false;
    for seed in 1..=256 {
        let a = run_one_year(seed, false, 1_000_000, 1_000_000);
        let b = run_one_year(seed, true, 1_000_000, 1_000_000);
        assert_eq!(
            a, b,
            "dual-trigger cause attribution diverged under relabelling at seed {seed}"
        );
        for (_, cause) in a.deaths {
            saw_condition |= cause == DeathCause::ResourceScarcity;
            saw_background |= cause == DeathCause::DemographicMortality;
        }
    }
    assert!(
        saw_condition && saw_background,
        "dual-trigger regression did not exercise both cause attributions"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct DownstreamOutcome {
    death_cells: Vec<CellId>,
    living_by_cell: [u64; 2],
    final_food_stock: [u64; 2],
    condition_rng: RngStreamPosition,
}

fn run_downstream(seed: u64, swapped_labels: bool) -> DownstreamOutcome {
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(0))
        .with_resources(resources(250_000, 100))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut death_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Death { cell, .. } => Some(cell),
            _ => None,
        })
        .collect::<Vec<_>>();
    death_cells.sort_unstable_by_key(|cell| cell.0);
    let population = &recorded.checkpoint.population;
    let mut living_by_cell = [0_u64; 2];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if person.death_day.is_none() {
            living_by_cell[usize::try_from(person.location.0 - 1).unwrap()] += 1;
        }
    }
    let resource_state = &recorded.checkpoint.resources;
    DownstreamOutcome {
        death_cells,
        living_by_cell,
        final_food_stock: [
            resource_state.cell_food_stock(CellId::new(1)).unwrap(),
            resource_state.cell_food_stock(CellId::new(2)).unwrap(),
        ],
        condition_rng: recorded.checkpoint.rng.resource_scarcity_mortality,
    }
}

#[test]
fn condition_mortality_relabelling_does_not_propagate_into_resource_state() {
    for seed in 1..=256 {
        assert_eq!(
            run_downstream(seed, false),
            run_downstream(seed, true),
            "downstream state diverged under pure PersonId relabelling at seed {seed}"
        );
    }
}
