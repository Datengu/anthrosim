use crate::{
    config::{
        MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
        ResourceConfig, WorldConfig,
    },
    events::EventLog,
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    migration::{MigrationBoundaryContext, MigrationRngs, MigrationSystem},
    population::{Population, ReproductiveSex},
    resources::ResourceSystem,
    rng::RngFactory,
    world::World,
};

#[test]
fn declared_founder_kin_is_active_on_first_migration_boundary() {
    let seed = 801;
    let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(seed)).unwrap();
    let definition = FounderPopulationDefinition::new(
        "first-boundary-kin-acceptance-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(2),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(1),
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(50 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(25 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: Some(PersonId::new(1)),
                last_birth_day: None,
                condition_permille: 0,
            },
        ],
    );
    let population_config = PopulationConfig::new(2)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
    let mut population =
        Population::initialize_declared_founder_state_v1(population_config, &definition, &world)
            .unwrap();

    let resources_config = ResourceConfig::synthetic_validation_v1();
    let resources = ResourceSystem::initialize(&world, &resources_config).unwrap();
    let mut migration_config = MigrationConfig::synthetic_validation_v1();
    migration_config.enabled = true;
    migration_config.candidate_radius_cells = 1;
    migration_config.condition_pressure_threshold_permille = 1_000;
    migration_config.resource_pressure_threshold_permille = 0;
    migration_config.minimum_utility_improvement = 0;
    migration_config.resource_weight = 0;
    migration_config.water_security_weight = 0;
    migration_config.kin_weight = 4;
    migration_config.travel_cost_weight = 0;
    migration_config.max_uncertainty_penalty_permille = 0;
    migration_config.relocation_risk_base_penalty_permille = 0;
    migration_config.relocation_risk_per_cell_permille = 0;
    migration_config.travel_condition_cost_per_cell = 0;
    migration_config.max_recorded_decision_traces = 8;

    let mut migration =
        MigrationSystem::initialize(&population, &world, &migration_config).unwrap();
    let mut rngs = MigrationRngs::new(RngFactory::new(seed));
    let mut events = EventLog::new();
    migration
        .process_boundary_recorded(
            &mut population,
            &MigrationBoundaryContext {
                world: &world,
                resources: &resources,
                migration: &migration_config,
                annual_food_need: 0,
                resource_periods_per_year: 4,
                day: 1,
            },
            &mut rngs,
            &mut events,
        )
        .unwrap();

    let summary = migration.summary();
    assert_eq!(summary.decision_boundaries, 1);
    assert_eq!(summary.moves_completed, 1);
    let trace = summary
        .recorded_decision_traces
        .iter()
        .find(|trace| trace.household == HouseholdId::new(2))
        .expect("declared child household should move toward its living founder parent");
    assert_eq!(trace.decision_day, 1);
    assert_eq!(trace.origin, CellId::new(1));
    assert_eq!(trace.destination, CellId::new(2));
    assert_eq!(trace.origin_utility.kin_score_permille, 0);
    assert_eq!(trace.destination_utility.kin_score_permille, 250);
    assert_eq!(
        population.household_location(HouseholdId::new(2)),
        Some(CellId::new(2))
    );
}
