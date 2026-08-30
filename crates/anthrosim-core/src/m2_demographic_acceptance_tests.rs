use crate::{
    config::{
        AgeProbabilityBand, DemographyConfig, ParameterProvenance, PopulationConfig,
        PopulationInitialization, WorldConfig,
    },
    demography::{DemographyRngs, process_demographic_year, process_demographic_year_recorded},
    events::{EventKind, EventLog},
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    migration::MigrationUtilityBreakdown,
    population::{Population, ReproductiveSex},
    rng::RngFactory,
    time::DAYS_PER_YEAR,
    world::World,
};

#[test]
fn model_born_child_receives_second_year_band_at_second_elapsed_interval() {
    let (world, definition, mut population) = two_adult_population(CellId::new(1), CellId::new(1));
    let child = population
        .append_birth(
            DAYS_PER_YEAR,
            ReproductiveSex::Male,
            CellId::new(1),
            HouseholdId::new(1),
            PersonId::new(1),
            PersonId::new(2),
        )
        .expect("child should fit record limit");
    let config = DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "second-year-boundary".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![
            AgeProbabilityBand::new(0, 1, 0),
            AgeProbabilityBand::new(1, 2, 1_000_000),
            AgeProbabilityBand::new(2, u32::MAX, 0),
        ],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    };
    let mut rngs = DemographyRngs::new(RngFactory::new(11));

    process_demographic_year(
        &mut population,
        &world,
        &config,
        2 * DAYS_PER_YEAR,
        &mut rngs,
    )
    .expect("first child interval should execute");
    assert_eq!(population.person(child).unwrap().death_day, None);

    process_demographic_year(
        &mut population,
        &world,
        &config,
        3 * DAYS_PER_YEAR,
        &mut rngs,
    )
    .expect("second child interval should execute");
    assert_eq!(
        population.person(child).unwrap().death_day,
        Some(3 * DAYS_PER_YEAR)
    );
    assert_eq!(definition.people.len(), 2);
}

#[test]
fn founder_age_band_boundary_uses_interval_start_age_exactly() {
    let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(1)).unwrap();
    let household = HouseholdId::new(1);
    let definition = FounderPopulationDefinition::new(
        "founder-age-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            founder(PersonId::new(1), -364, ReproductiveSex::Male, household),
            founder(PersonId::new(2), -365, ReproductiveSex::Male, household),
        ],
    );
    let population_config = declared_population_config(2);
    let mut population = Population::initialize_declared_founder_state_v1(
        population_config,
        &definition,
        &world,
        &crate::config::DemographyConfig::synthetic_validation_v1(),
    )
    .unwrap();
    let config = DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "founder-age-boundary".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![
            AgeProbabilityBand::new(0, 1, 0),
            AgeProbabilityBand::new(1, u32::MAX, 1_000_000),
        ],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    };
    let mut rngs = DemographyRngs::new(RngFactory::new(2));

    process_demographic_year(&mut population, &world, &config, DAYS_PER_YEAR, &mut rngs).unwrap();

    assert_eq!(population.person(PersonId::new(1)).unwrap().death_day, None);
    assert_eq!(
        population.person(PersonId::new(2)).unwrap().death_day,
        Some(DAYS_PER_YEAR)
    );
}

#[test]
fn fertility_band_boundary_uses_interval_start_age_exactly() {
    let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(3)).unwrap();
    let household = HouseholdId::new(1);
    let definition = FounderPopulationDefinition::new(
        "fertility-age-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            founder(
                PersonId::new(1),
                -((18 * DAYS_PER_YEAR) as i64 - 1),
                ReproductiveSex::Female,
                household,
            ),
            founder(
                PersonId::new(2),
                -((18 * DAYS_PER_YEAR) as i64),
                ReproductiveSex::Female,
                household,
            ),
            founder(
                PersonId::new(3),
                -((25 * DAYS_PER_YEAR) as i64),
                ReproductiveSex::Male,
                household,
            ),
        ],
    );
    let mut population = Population::initialize_declared_founder_state_v1(
        declared_population_config(3),
        &definition,
        &world,
        &crate::config::DemographyConfig::synthetic_validation_v1(),
    )
    .unwrap();
    let config = DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "fertility-age-boundary".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        fertility_bands: vec![
            AgeProbabilityBand::new(0, 18, 0),
            AgeProbabilityBand::new(18, 19, 1_000_000),
            AgeProbabilityBand::new(19, u32::MAX, 0),
        ],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    };
    let mut rngs = DemographyRngs::new(RngFactory::new(4));
    let mut events = EventLog::new();

    process_demographic_year_recorded(
        &mut population,
        &world,
        &config,
        DAYS_PER_YEAR,
        &mut rngs,
        &mut events,
    )
    .unwrap();

    let parents: Vec<_> = events
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth { female_parent, .. } => Some(female_parent),
            _ => None,
        })
        .collect();
    assert_eq!(parents, vec![PersonId::new(2)]);
}

#[test]
fn annual_move_keeps_origin_only_male_in_parentage_pool() {
    let (world, _definition, mut population) = two_adult_population(CellId::new(1), CellId::new(1));
    let mut events = EventLog::new();
    move_female_household(
        &mut population,
        &world,
        &mut events,
        DAYS_PER_YEAR,
        CellId::new(1),
        CellId::new(2),
    );

    run_certain_fertility(&mut population, &world, &mut events, DAYS_PER_YEAR, 31);
    assert_eq!(birth_count(&events), 1);
}

#[test]
fn annual_move_does_not_create_destination_only_parentage() {
    let (world, _definition, mut population) = two_adult_population(CellId::new(1), CellId::new(2));
    let mut events = EventLog::new();
    move_female_household(
        &mut population,
        &world,
        &mut events,
        DAYS_PER_YEAR,
        CellId::new(1),
        CellId::new(2),
    );

    run_certain_fertility(&mut population, &world, &mut events, DAYS_PER_YEAR, 32);
    assert_eq!(birth_count(&events), 0);
}

#[test]
fn nonannual_move_uses_destination_after_elapsed_residence() {
    let (world, _definition, mut population) = two_adult_population(CellId::new(1), CellId::new(2));
    let mut events = EventLog::new();
    move_female_household(
        &mut population,
        &world,
        &mut events,
        DAYS_PER_YEAR / 2,
        CellId::new(1),
        CellId::new(2),
    );

    run_certain_fertility(&mut population, &world, &mut events, DAYS_PER_YEAR, 33);
    assert_eq!(birth_count(&events), 1);
}

fn run_certain_fertility(
    population: &mut Population,
    world: &World,
    events: &mut EventLog,
    day: u64,
    seed: u64,
) {
    let config = DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "certain-fertility".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 1_000_000)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    };
    let mut rngs = DemographyRngs::new(RngFactory::new(seed));
    process_demographic_year_recorded(population, world, &config, day, &mut rngs, events).unwrap();
}

fn two_adult_population(
    female_location: CellId,
    male_location: CellId,
) -> (World, FounderPopulationDefinition, Population) {
    let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(8)).unwrap();
    let female_household = HouseholdId::new(1);
    let male_household = HouseholdId::new(2);
    let definition = FounderPopulationDefinition::new(
        "m2-parentage-locality",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![
            FounderHousehold {
                id: female_household,
                location: female_location,
            },
            FounderHousehold {
                id: male_household,
                location: male_location,
            },
        ],
        vec![
            founder(
                PersonId::new(1),
                -((25 * DAYS_PER_YEAR) as i64),
                ReproductiveSex::Female,
                female_household,
            ),
            founder(
                PersonId::new(2),
                -((25 * DAYS_PER_YEAR) as i64),
                ReproductiveSex::Male,
                male_household,
            ),
        ],
    );
    let population = Population::initialize_declared_founder_state_v1(
        declared_population_config(2),
        &definition,
        &world,
        &crate::config::DemographyConfig::synthetic_validation_v1(),
    )
    .unwrap();
    (world, definition, population)
}

fn move_female_household(
    population: &mut Population,
    world: &World,
    events: &mut EventLog,
    day: u64,
    origin: CellId,
    destination: CellId,
) {
    population
        .apply_household_relocations(&[destination, CellId::INVALID], &[0, 0], world)
        .unwrap();
    let utility = MigrationUtilityBreakdown {
        resource_score_permille: 0,
        water_security_score_permille: 0,
        kin_score_permille: 0,
        travel_penalty_permille: 0,
        uncertainty_penalty_permille: 0,
        relocation_risk_penalty_permille: 0,
        total_utility: 0,
    };
    events.push_authoritative(
        day,
        EventKind::HouseholdMigration {
            household: HouseholdId::new(1),
            people_moved: 1,
            origin,
            destination,
            distance_cells: 1,
            pressure_permille: 0,
            origin_utility: utility,
            destination_utility: utility,
            best_candidate: destination,
            best_candidate_utility: 0,
            selected_weight: 1,
            total_move_weight: 1,
            choice_draw: 0,
            nominal_travel_condition_cost_per_person: 0,
            realized_travel_condition_loss_total: 0,
        },
    );
}

fn birth_count(events: &EventLog) -> usize {
    events
        .events
        .iter()
        .filter(|record| matches!(record.event, EventKind::Birth { .. }))
        .count()
}

fn declared_population_config(initial_population: u32) -> PopulationConfig {
    PopulationConfig::new(initial_population)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
        .with_max_person_records(100)
}

fn founder(
    id: PersonId,
    birth_day: i64,
    reproductive_sex: ReproductiveSex,
    household: HouseholdId,
) -> FounderPerson {
    FounderPerson {
        id,
        birth_day,
        reproductive_sex,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}
