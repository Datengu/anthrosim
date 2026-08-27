use crate::{
    config::{
        AgeProbabilityBand, DemographyConfig, MigrationConfig, ParameterProvenance,
        PopulationConfig, PopulationInitialization, ResourceConfig, WorldConfig,
    },
    demography::{DemographyRngs, process_demographic_year_recorded},
    events::{DeathCause, EventKind, EventLog},
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    migration::migration_pressure_permille,
    population::{Population, ReproductiveSex},
    resources::{ResourcePeriodContext, ResourceRngs, ResourceSystem},
    rng::RngFactory,
    time::DAYS_PER_YEAR,
    world::World,
};

const FEMALE_PARENT: PersonId = PersonId::new(1);
const MALE_PARENT: PersonId = PersonId::new(2);
const NEWBORN: PersonId = PersonId::new(3);
const HOUSEHOLD: HouseholdId = HouseholdId::new(1);
const RESIDENCE: CellId = CellId::new(1);

fn birth_population(
    female_condition_permille: u16,
    male_condition_permille: u16,
    seed: u64,
) -> (World, Population, EventLog) {
    let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(seed)).unwrap();
    let definition = FounderPopulationDefinition::new(
        "newborn-condition-acceptance-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HOUSEHOLD,
            location: RESIDENCE,
        }],
        vec![
            FounderPerson {
                id: FEMALE_PARENT,
                birth_day: -((25 * DAYS_PER_YEAR) as i64),
                reproductive_sex: ReproductiveSex::Female,
                household: HOUSEHOLD,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: female_condition_permille,
            },
            FounderPerson {
                id: MALE_PARENT,
                birth_day: -((25 * DAYS_PER_YEAR) as i64),
                reproductive_sex: ReproductiveSex::Male,
                household: HOUSEHOLD,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: male_condition_permille,
            },
        ],
    );
    let population_config = PopulationConfig::new(2)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
        .with_max_person_records(16);
    let mut population =
        Population::initialize_declared_founder_state_v1(population_config, &definition, &world)
            .unwrap();

    let mut demography = DemographyConfig::synthetic_validation_v1();
    demography.schedule_id = "certain-birth-newborn-condition-v1".to_owned();
    demography.mortality_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];
    demography.fertility_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 1_000_000)];
    demography.minimum_birth_spacing_days = 0;

    let mut rngs = DemographyRngs::new(RngFactory::new(seed));
    let mut events = EventLog::new();
    process_demographic_year_recorded(
        &mut population,
        &world,
        &demography,
        DAYS_PER_YEAR,
        &mut rngs,
        &mut events,
    )
    .unwrap();

    assert_eq!(population.person_count(), 3);
    assert_eq!(
        population.person(NEWBORN).unwrap().female_parent,
        FEMALE_PARENT
    );
    assert_eq!(population.person(NEWBORN).unwrap().male_parent, MALE_PARENT);
    assert_eq!(
        events
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::Birth { .. }))
            .count(),
        1
    );

    (world, population, events)
}

#[test]
fn newborn_inherits_high_medium_and_very_low_female_parent_condition_exactly() {
    for (seed, condition) in [(20_101, 900_u16), (20_102, 500_u16), (20_103, 100_u16)] {
        let (_world, population, _events) = birth_population(condition, 1_000, seed);
        assert_eq!(
            population.person(FEMALE_PARENT).unwrap().condition_permille,
            condition
        );
        assert_eq!(
            population.person(NEWBORN).unwrap().condition_permille,
            condition
        );
    }
}

#[test]
fn severe_scarcity_does_not_leave_newborn_with_a_hidden_condition_advantage() {
    let (world, mut population, _birth_events) = birth_population(100, 100, 20_201);
    assert_eq!(
        population.person(FEMALE_PARENT).unwrap().condition_permille,
        100
    );
    assert_eq!(population.person(NEWBORN).unwrap().condition_permille, 100);

    let mut resource_config =
        ResourceConfig::synthetic_validation_v1().with_productivity_scale_permille(0);
    resource_config.periods_per_year = 1;
    resource_config.annual_need_units_per_person = 1;
    resource_config.condition_recovery_per_period = 0;
    resource_config.max_condition_loss_per_period = 1_000;
    resource_config.max_scarcity_mortality_probability_per_million = 1_000_000;

    let mut resources = ResourceSystem::initialize(&world, &resource_config).unwrap();
    assert_eq!(resources.total_food_stock().unwrap(), 0);
    let mut rngs = ResourceRngs::new(RngFactory::new(20_202));
    let mut events = EventLog::new();
    let resource_day = 2 * DAYS_PER_YEAR;
    resources
        .process_period_recorded(
            &mut population,
            &ResourcePeriodContext {
                world: &world,
                config: &resource_config,
                period_index_in_year: 0,
                day: resource_day,
            },
            &mut rngs.scarcity_mortality,
            &mut events,
        )
        .unwrap();

    for person in [FEMALE_PARENT, NEWBORN] {
        assert_eq!(
            population.person(person).unwrap().death_day,
            Some(resource_day)
        );
        let death = events.events.iter().find(|record| {
            matches!(
                record.event,
                EventKind::Death {
                    person: event_person,
                    cause: DeathCause::ResourceScarcity,
                    condition_permille: 0,
                    probability_per_million: 1_000_000,
                    ..
                } if event_person == person
            )
        });
        assert!(
            death.is_some(),
            "missing deterministic condition death for {person:?}"
        );
    }

    let summary = resources.summary(&population);
    assert_eq!(summary.unmet_need, 3);
    assert_eq!(summary.scarcity_deaths, 3);
    assert_eq!(summary.mean_living_condition_permille, None);
}

#[test]
fn newborn_inheritance_preserves_household_mean_and_expected_m4_condition_pressure() {
    let (_world, population, _events) = birth_population(200, 200, 20_301);
    let mean_condition = population.mean_living_condition_permille().unwrap();
    assert_eq!(mean_condition, 200);

    let mut migration = MigrationConfig::synthetic_validation_v1();
    migration.condition_pressure_threshold_permille = 500;
    migration.resource_pressure_threshold_permille = 0;

    let actual_pressure = migration_pressure_permille(mean_condition, 1_000, &migration);
    assert_eq!(actual_pressure, 300);

    // Under the removed hard reset, this same household would have contained conditions
    // [200, 200, 1000], raising the mean to 466 and suppressing M4 condition pressure to 34.
    let removed_reset_mean = (200_u16 + 200 + 1_000) / 3;
    assert_eq!(removed_reset_mean, 466);
    let removed_reset_pressure = migration_pressure_permille(removed_reset_mean, 1_000, &migration);
    assert_eq!(removed_reset_pressure, 34);
    assert!(actual_pressure > removed_reset_pressure);
}
