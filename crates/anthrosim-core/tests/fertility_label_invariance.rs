use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography(spacing_days: u32) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 500_000;
    }
    config.minimum_birth_spacing_days = spacing_days;
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn person(
    id: u64,
    reproductive_sex: ReproductiveSex,
    household: u64,
    age_years: i64,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(age_years * 365),
        reproductive_sex,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn simple_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let people = if swapped_labels {
        vec![
            person(1, ReproductiveSex::Female, 2, 30),
            person(2, ReproductiveSex::Male, 2, 30),
            person(3, ReproductiveSex::Female, 1, 30),
            person(4, ReproductiveSex::Male, 1, 30),
        ]
    } else {
        vec![
            person(1, ReproductiveSex::Female, 1, 30),
            person(2, ReproductiveSex::Male, 1, 30),
            person(3, ReproductiveSex::Female, 2, 30),
            person(4, ReproductiveSex::Male, 2, 30),
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

#[derive(Debug, PartialEq, Eq)]
struct SimpleOutcome {
    birth_cells: Vec<CellId>,
    demography_rng: [anthrosim_core::RngStreamPosition; 4],
}

fn run_simple(seed: u64, swapped_labels: bool) -> SimpleOutcome {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(4)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(simple_founders(swapped_labels))
        .with_demography(demography(0))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let birth_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth { cell, .. } => Some(cell),
            _ => None,
        })
        .collect();
    let rng = &recorded.checkpoint.rng;
    SimpleOutcome {
        birth_cells,
        demography_rng: [
            rng.demography_mortality,
            rng.demography_fertility,
            rng.demography_parentage,
            rng.demography_newborn_sex,
        ],
    }
}

#[test]
fn fertility_birth_cells_and_demography_rng_positions_are_person_label_invariant() {
    for seed in 1..=1_000 {
        assert_eq!(
            run_simple(seed, false),
            run_simple(seed, true),
            "scientifically identical founder states diverged under PersonId relabelling at seed {seed}"
        );
    }
}

fn propagation_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    let household_for_group_a = if swapped_labels { 2 } else { 1 };
    let household_for_group_b = if swapped_labels { 1 } else { 2 };
    FounderPopulationDefinition::new(
        if swapped_labels {
            "area-n-relabel-b"
        } else {
            "area-n-relabel-a"
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
        vec![
            person(1, ReproductiveSex::Female, household_for_group_a, 30),
            person(2, ReproductiveSex::Male, household_for_group_a, 30),
            person(3, ReproductiveSex::Female, household_for_group_a, 70),
            person(4, ReproductiveSex::Male, household_for_group_a, 80),
            person(5, ReproductiveSex::Female, household_for_group_b, 30),
            person(6, ReproductiveSex::Male, household_for_group_b, 30),
            person(7, ReproductiveSex::Female, household_for_group_b, 70),
            person(8, ReproductiveSex::Male, household_for_group_b, 80),
        ],
    )
}

#[derive(Debug, PartialEq, Eq)]
struct PropagationOutcome {
    birth_cells: Vec<CellId>,
    fission_cells: Vec<CellId>,
    living_by_cell: [u64; 2],
    sorted_household_locations: Vec<CellId>,
    final_food_stock: [u64; 2],
    demography_rng: [anthrosim_core::RngStreamPosition; 4],
}

fn run_propagation(swapped_labels: bool) -> PropagationOutcome {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(100)
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(1, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(8)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(64),
        )
        .with_founder_population(propagation_founders(swapped_labels))
        .with_demography(demography(10_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(4, 18),
        );
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let birth_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth { cell, .. } => Some(cell),
            _ => None,
        })
        .collect();
    let fission_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::HouseholdFission { residence, .. } => Some(residence),
            _ => None,
        })
        .collect();
    let population = &recorded.checkpoint.population;
    let mut living_by_cell = [0_u64; 2];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if person.death_day.is_none() {
            living_by_cell[usize::try_from(person.location.0 - 1).unwrap()] += 1;
        }
    }
    let mut sorted_household_locations = (1..=population.household_count() as u64)
        .map(|id| population.household_location(HouseholdId::new(id)).unwrap())
        .collect::<Vec<_>>();
    sorted_household_locations.sort();
    let resources = &recorded.checkpoint.resources;
    let rng = &recorded.checkpoint.rng;
    PropagationOutcome {
        birth_cells,
        fission_cells,
        living_by_cell,
        sorted_household_locations,
        final_food_stock: [
            resources.cell_food_stock(CellId::new(1)).unwrap(),
            resources.cell_food_stock(CellId::new(2)).unwrap(),
        ],
        demography_rng: [
            rng.demography_mortality,
            rng.demography_fertility,
            rng.demography_parentage,
            rng.demography_newborn_sex,
        ],
    }
}

#[test]
fn fertility_relabelling_no_longer_propagates_into_household_or_resource_state() {
    assert_eq!(run_propagation(false), run_propagation(true));
}
