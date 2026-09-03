use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
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
    config.minimum_birth_spacing_days = 10_000;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
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

fn founders(swapped_labels: bool) -> FounderPopulationDefinition {
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
            // Group A: one fertile pair plus two independent-age non-fertile anchors.
            person(1, ReproductiveSex::Female, household_for_group_a, 30),
            person(2, ReproductiveSex::Male, household_for_group_a, 30),
            person(3, ReproductiveSex::Female, household_for_group_a, 70),
            person(4, ReproductiveSex::Male, household_for_group_a, 80),
            // Group B is scientifically identical after erasing canonical person labels.
            person(5, ReproductiveSex::Female, household_for_group_b, 30),
            person(6, ReproductiveSex::Male, household_for_group_b, 30),
            person(7, ReproductiveSex::Female, household_for_group_b, 70),
            person(8, ReproductiveSex::Male, household_for_group_b, 80),
        ],
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    birth_cells: Vec<CellId>,
    fission_cells: Vec<CellId>,
    living_by_cell: [u64; 2],
    household_locations: Vec<CellId>,
    final_food_stock: [u64; 2],
}

fn run(seed: u64, swapped_labels: bool) -> Snapshot {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(100)
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(8)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(64),
        )
        .with_founder_population(founders(swapped_labels))
        .with_demography(demography())
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
        .filter_map(|record| match &record.event {
            EventKind::Birth { cell, .. } => Some(*cell),
            _ => None,
        })
        .collect::<Vec<_>>();
    let fission_cells = recorded
        .events()
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::HouseholdFission { residence, .. } => Some(*residence),
            _ => None,
        })
        .collect::<Vec<_>>();

    let population = &recorded.checkpoint.population;
    let mut living_by_cell = [0_u64; 2];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if person.death_day.is_none() {
            let index = usize::try_from(person.location.0 - 1).unwrap();
            living_by_cell[index] += 1;
        }
    }
    let household_locations = (1..=population.household_count() as u64)
        .map(|id| population.household_location(HouseholdId::new(id)).unwrap())
        .collect::<Vec<_>>();
    let resources = &recorded.checkpoint.resources;
    let final_food_stock = [
        resources.cell_food_stock(CellId::new(1)).unwrap(),
        resources.cell_food_stock(CellId::new(2)).unwrap(),
    ];

    Snapshot {
        birth_cells,
        fission_cells,
        living_by_cell,
        household_locations,
        final_food_stock,
    }
}

#[test]
fn arbitrary_founder_labels_do_not_propagate_across_demography_households_and_resources() {
    for seed in 1..=1_000 {
        let control = run(seed, false);
        let relabelled = run(seed, true);

        let fertility_diverged = control.birth_cells != relabelled.birth_cells;
        let household_diverged = control.fission_cells != relabelled.fission_cells
            || control.household_locations != relabelled.household_locations;
        let resource_diverged = control.final_food_stock != relabelled.final_food_stock;

        if fertility_diverged && household_diverged && resource_diverged {
            println!("audit_v4_area_n_seed={seed}");
            println!("control_birth_cells={:?}", control.birth_cells);
            println!("relabelled_birth_cells={:?}", relabelled.birth_cells);
            println!("control_fission_cells={:?}", control.fission_cells);
            println!("relabelled_fission_cells={:?}", relabelled.fission_cells);
            println!("control_living_by_cell={:?}", control.living_by_cell);
            println!("relabelled_living_by_cell={:?}", relabelled.living_by_cell);
            println!("control_household_locations={:?}", control.household_locations);
            println!("relabelled_household_locations={:?}", relabelled.household_locations);
            println!("control_final_food_stock={:?}", control.final_food_stock);
            println!("relabelled_final_food_stock={:?}", relabelled.final_food_stock);
            panic!(
                "pure founder PersonId relabelling propagated from fertility into household topology and M3 resource state at seed {seed}"
            );
        }
    }
}
