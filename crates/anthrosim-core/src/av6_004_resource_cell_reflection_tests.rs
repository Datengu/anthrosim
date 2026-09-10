use crate::{
    config::{
        DemographyConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
        ResourceConfig, WorldConfig,
    },
    events::EventLog,
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    population::{Population, ReproductiveSex},
    resources::{ResourcePeriodContext, ResourceRngs, ResourceSystem},
    rng::RngFactory,
    world::World,
};

const PERSON_A: PersonId = PersonId::new(1);
const PERSON_B: PersonId = PersonId::new(2);
const HOUSEHOLD_A: HouseholdId = HouseholdId::new(1);
const HOUSEHOLD_B: HouseholdId = HouseholdId::new(2);

fn reflected_resource_outcome(width: u32, height: u32, residence: CellId) -> (u16, u16) {
    let world = World::generate(WorldConfig::new(width, height), RngFactory::new(73_001))
        .unwrap()
        .with_model_field_overlay(None, None, Some(&[1, 1]))
        .unwrap();

    let founders = FounderPopulationDefinition::new(
        "audit-v6-area-d-resource-cell-reflection",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HOUSEHOLD_A,
                location: residence,
            },
            FounderHousehold {
                id: HOUSEHOLD_B,
                location: residence,
            },
        ],
        vec![
            FounderPerson {
                id: PERSON_A,
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HOUSEHOLD_A,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PERSON_B,
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HOUSEHOLD_B,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    );

    let population_config = PopulationConfig::new(2)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
        .with_max_person_records(4);
    let mut population = Population::initialize_declared_founder_state_v1(
        population_config,
        &founders,
        &world,
        &DemographyConfig::synthetic_validation_v1(),
    )
    .unwrap();

    let mut resources_config = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(1)
        .with_annual_regeneration_units_per_productivity(1)
        .with_annual_need_units_per_person(1)
        .with_seasonality_scale_permille(0);
    resources_config.cell_stock_capacity_years = 1;
    resources_config.periods_per_year = 1;
    resources_config.condition_recovery_per_period = 0;
    resources_config.max_condition_loss_per_period = 1_000;
    resources_config.max_scarcity_mortality_probability_per_million = 0;

    let mut resources = ResourceSystem::initialize(&world, &resources_config).unwrap();
    let mut rngs = ResourceRngs::new(RngFactory::new(73_002));
    let mut events = EventLog::new();
    resources
        .process_period_recorded(
            &mut population,
            &ResourcePeriodContext {
                world: &world,
                config: &resources_config,
                period_index_in_year: 0,
                day: 365,
            },
            &mut rngs.scarcity_mortality,
            &mut events,
        )
        .unwrap();

    let observation = resources.period_observations().last().unwrap();
    assert_eq!(observation.total_need, 2);
    assert_eq!(observation.supplied, 1);
    assert_eq!(observation.unmet, 1);
    assert_eq!(observation.households_with_unmet_need, 1);

    (
        population.person(PERSON_A).unwrap().condition_permille,
        population.person(PERSON_B).unwrap().condition_permille,
    )
}

#[test]
fn exact_705_horizontal_reflection_adversary_is_repaired() {
    let canonical = reflected_resource_outcome(2, 1, CellId::new(1));
    let reflected = reflected_resource_outcome(2, 1, CellId::new(2));
    assert_eq!(
        canonical, reflected,
        "the preserved #705 scarcity fixture must allocate to the same abstract household under horizontal reflection"
    );
}

#[test]
fn scarce_resource_remainder_is_equivariant_under_vertical_cell_reflection() {
    let canonical = reflected_resource_outcome(1, 2, CellId::new(1));
    let reflected = reflected_resource_outcome(1, 2, CellId::new(2));
    assert_eq!(
        canonical, reflected,
        "the same scarcity fixture must remain equivariant under vertical reflection"
    );
}
