use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography(probability_per_million: u32) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = probability_per_million;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
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
        condition_permille: 1_000,
    }
}

fn two_household_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        if swapped_labels {
            "mortality-relabel-b"
        } else {
            "mortality-relabel-a"
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
    death_cells: Vec<CellId>,
    background_rng: anthrosim_core::RngStreamPosition,
    condition_rng: anthrosim_core::RngStreamPosition,
}

fn run_one_year(seed: u64, swapped_labels: bool) -> OneYearOutcome {
    let mut resources =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(500_000))
        .with_resources(resources)
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
    OneYearOutcome {
        death_cells,
        background_rng: recorded.checkpoint.rng.demography_mortality,
        condition_rng: recorded.checkpoint.rng.resource_scarcity_mortality,
    }
}

#[test]
fn background_mortality_cells_and_rng_positions_are_person_label_invariant() {
    for seed in 1..=1_000 {
        assert_eq!(
            run_one_year(seed, false),
            run_one_year(seed, true),
            "background mortality diverged under pure PersonId relabelling at seed {seed}"
        );
    }
}

fn three_household_founders(rotation: u64) -> FounderPopulationDefinition {
    let household_for_label = |label: u64| ((label - 1 + rotation) % 3) + 1;
    FounderPopulationDefinition::new(
        format!("mortality-cycle-{rotation}"),
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
    let mut resources =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(three_household_founders(rotation))
        .with_demography(demography(350_000))
        .with_resources(resources)
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
fn background_mortality_is_invariant_to_three_person_cyclic_relabelling() {
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

#[derive(Debug, PartialEq, Eq)]
struct DownstreamOutcome {
    death_cells: Vec<CellId>,
    living_by_cell: [u64; 2],
    final_food_stock: [u64; 2],
}

fn run_downstream(seed: u64, swapped_labels: bool) -> DownstreamOutcome {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(100)
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(250_000))
        .with_resources(resources)
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
    let resources = &recorded.checkpoint.resources;
    DownstreamOutcome {
        death_cells,
        living_by_cell,
        final_food_stock: [
            resources.cell_food_stock(CellId::new(1)).unwrap(),
            resources.cell_food_stock(CellId::new(2)).unwrap(),
        ],
    }
}

#[test]
fn background_mortality_relabelling_does_not_propagate_into_resource_state() {
    for seed in 1..=256 {
        assert_eq!(
            run_downstream(seed, false),
            run_downstream(seed, true),
            "downstream state diverged under pure PersonId relabelling at seed {seed}"
        );
    }
}
