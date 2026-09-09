use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, MigrationConfig, ParameterProvenance, PopulationConfig,
    ReproductiveSex, ResourceConfig, Simulation, World, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
    validate_recorded_run_invariants,
};

const HOUSEHOLD_COUNT: u64 = 3;
const DAYS_PER_YEAR: i64 = 365;

fn quiet_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn resource_config(annual_need: u32) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = annual_need;
    config.annual_regeneration_units_per_productivity = 1;
    config.cell_stock_capacity_years = 10;
    config.seasonality_scale_permille = 0;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 1_000;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn find_productive_cell_with_three_way_remainder() -> (u64, CellId, u64) {
    for seed in 78_101..78_501 {
        let world = World::generate(WorldConfig::new(2, 2), RngFactory::new(seed)).unwrap();
        if let Some((index, cell)) = world
            .cells()
            .iter()
            .enumerate()
            .find(|(_, cell)| cell.food_stock > 0 && u64::from(cell.food_stock) % HOUSEHOLD_COUNT != 0)
        {
            return (
                seed,
                CellId::new(u64::try_from(index).unwrap() + 1),
                u64::from(cell.food_stock),
            );
        }
    }
    panic!("audit seed search found no productive cell with a three-way allocation remainder");
}

fn physical_profile(physical_household: u64) -> (i64, ReproductiveSex) {
    match physical_household {
        1 => (-(25 * DAYS_PER_YEAR), ReproductiveSex::Female),
        2 => (-(35 * DAYS_PER_YEAR), ReproductiveSex::Male),
        3 => (-(45 * DAYS_PER_YEAR), ReproductiveSex::Female),
        _ => unreachable!(),
    }
}

fn physical_household_for_person_id(person_id: u64, relabel_person_ids: bool) -> u64 {
    if relabel_person_ids {
        // Pure cyclic PersonId relabelling of the three fixed physical founder profiles:
        // H1/person-profile-A -> P2, H2/profile-B -> P3, H3/profile-C -> P1.
        (person_id + 1) % HOUSEHOLD_COUNT + 1
    } else {
        person_id
    }
}

fn person_id_for_physical_household(physical_household: u64, relabel_person_ids: bool) -> PersonId {
    if relabel_person_ids {
        PersonId::new(physical_household % HOUSEHOLD_COUNT + 1)
    } else {
        PersonId::new(physical_household)
    }
}

fn founders(residence: CellId, relabel_person_ids: bool) -> FounderPopulationDefinition {
    let households = (1..=HOUSEHOLD_COUNT)
        .map(|id| FounderHousehold {
            id: HouseholdId::new(id),
            location: residence,
        })
        .collect::<Vec<_>>();

    let people = (1..=HOUSEHOLD_COUNT)
        .map(|person_id| {
            let physical_household =
                physical_household_for_person_id(person_id, relabel_person_ids);
            let (birth_day, reproductive_sex) = physical_profile(physical_household);
            FounderPerson {
                id: PersonId::new(person_id),
                birth_day,
                reproductive_sex,
                household: HouseholdId::new(physical_household),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            }
        })
        .collect::<Vec<_>>();

    FounderPopulationDefinition::new(
        if relabel_person_ids {
            "audit-v6-area-d-person-label-rotated"
        } else {
            "audit-v6-area-d-person-label-canonical"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
    )
}

fn conditions_by_physical_household(
    seed: u64,
    residence: CellId,
    annual_need: u32,
    relabel_person_ids: bool,
) -> Vec<u16> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(3).with_max_person_records(100))
        .with_founder_population(founders(residence, relabel_person_ids))
        .with_demography(quiet_demography())
        .with_resources(resource_config(annual_need))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();

    (1..=HOUSEHOLD_COUNT)
        .map(|physical_household| {
            let person = person_id_for_physical_household(physical_household, relabel_person_ids);
            run.checkpoint
                .population
                .person(person)
                .unwrap()
                .condition_permille
        })
        .collect()
}

#[test]
fn scarce_resource_tie_is_invariant_to_pure_person_id_relabelling() {
    let (seed, residence, stock) = find_productive_cell_with_three_way_remainder();
    let annual_need = u32::try_from(stock / HOUSEHOLD_COUNT + 1).unwrap();

    let canonical = conditions_by_physical_household(seed, residence, annual_need, false);
    let relabelled = conditions_by_physical_household(seed, residence, annual_need, true);

    eprintln!(
        "seed={seed} residence={residence:?} cell_stock={stock} annual_need={annual_need} canonical_by_physical_household={canonical:?} relabelled_by_physical_household={relabelled:?}"
    );

    assert_eq!(
        canonical, relabelled,
        "a pure PersonId permutation must not change which fixed physical household profile receives indivisible scarce-resource remainder units"
    );
}
