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
    migration::{MigrationBoundaryContext, MigrationRngs, MigrationSummary, MigrationSystem},
    population::{Population, ReproductiveSex},
    resources::{ResourceSystem, resource_period_day_bounds},
    rng::RngFactory,
    world::World,
};

fn run_two_cell_case(
    movement_cost: [u16; 2],
    relocation_risk_base_penalty_permille: u16,
    relocation_risk_per_cell_permille: u16,
    travel_cost_weight: u16,
) -> (MigrationSummary, CellId) {
    let seed = 1_860;
    let factory = RngFactory::new(seed);
    let water_access = [0, 1_000];
    let base_productivity = [1_000, 1_000];
    let world = World::generate(WorldConfig::new(2, 1), factory)
        .unwrap()
        .with_model_field_overlay(
            Some(&movement_cost),
            Some(&water_access),
            Some(&base_productivity),
        )
        .unwrap();

    let definition = FounderPopulationDefinition::new(
        "m4-stay-utility-acceptance-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(25 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 0,
        }],
    );
    let population_config = PopulationConfig::new(1)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
    let mut population =
        Population::initialize_declared_founder_state_v1(population_config, &definition, &world)
            .unwrap();

    let resources_config =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    let resources = ResourceSystem::initialize(&world, &resources_config).unwrap();
    let first_resource_boundary =
        resource_period_day_bounds(0, resources_config.periods_per_year)
            .expect("synthetic resource schedule has a first period")
            .1;
    let mut migration_config = MigrationConfig::synthetic_validation_v1();
    migration_config.enabled = true;
    migration_config.candidate_radius_cells = 1;
    migration_config.condition_pressure_threshold_permille = 1_000;
    migration_config.resource_pressure_threshold_permille = 0;
    migration_config.minimum_utility_improvement = 0;
    migration_config.resource_weight = 1;
    migration_config.water_security_weight = 1;
    migration_config.kin_weight = 0;
    migration_config.travel_cost_weight = travel_cost_weight;
    migration_config.max_uncertainty_penalty_permille = 0;
    migration_config.relocation_risk_base_penalty_permille = relocation_risk_base_penalty_permille;
    migration_config.relocation_risk_per_cell_permille = relocation_risk_per_cell_permille;
    migration_config.travel_condition_cost_per_cell = 0;
    migration_config.max_recorded_decision_traces = 8;

    let mut migration =
        MigrationSystem::initialize(&population, &world, &migration_config).unwrap();
    let mut rngs = MigrationRngs::new(factory);
    let mut events = EventLog::new();
    migration
        .process_boundary_recorded(
            &mut population,
            &MigrationBoundaryContext {
                world: &world,
                resources: &resources,
                migration: &migration_config,
                annual_food_need: 0,
                resource_periods_per_year: resources_config.periods_per_year,
                day: first_resource_boundary,
            },
            &mut rngs,
            &mut events,
        )
        .unwrap();

    (
        migration.summary(),
        population.household_location(HouseholdId::new(1)).unwrap(),
    )
}

#[test]
fn base_relocation_risk_reduces_move_eligibility_instead_of_cancelling_against_stay() {
    let (zero_risk, zero_risk_location) = run_two_cell_case([1_000, 1_000], 0, 0, 0);
    let (high_risk, high_risk_location) = run_two_cell_case([1_000, 1_000], 800, 0, 0);

    assert_eq!(zero_risk.moves_completed, 1);
    assert_eq!(zero_risk_location, CellId::new(2));
    assert_eq!(high_risk.moves_completed, 0);
    assert_eq!(high_risk_location, CellId::new(1));
}

#[test]
fn stay_action_never_pays_travel_uncertainty_or_relocation_risk() {
    let (summary, location) = run_two_cell_case([4_000, 1_000], 50, 25, 2);

    assert_eq!(summary.moves_completed, 1);
    assert_eq!(location, CellId::new(2));
    let trace = summary
        .recorded_decision_traces
        .first()
        .expect("controlled destination should remain preferable after real move costs");
    assert_eq!(trace.origin_utility.travel_penalty_permille, 0);
    assert_eq!(trace.origin_utility.uncertainty_penalty_permille, 0);
    assert_eq!(trace.origin_utility.relocation_risk_penalty_permille, 0);
    assert_eq!(trace.destination_utility.travel_penalty_permille, 120);
    assert_eq!(
        trace.destination_utility.relocation_risk_penalty_permille,
        75
    );
}

#[test]
fn m8_style_rough_origin_overlay_cannot_penalize_the_zero_distance_stay_action() {
    let (smooth_origin, smooth_location) = run_two_cell_case([1_000, 1_000], 50, 25, 2);
    let (rough_origin, rough_location) = run_two_cell_case([4_000, 1_000], 50, 25, 2);

    assert_eq!(smooth_location, rough_location);
    assert_eq!(smooth_origin, rough_origin);
    let trace = rough_origin.recorded_decision_traces.first().unwrap();
    assert_eq!(trace.origin_utility.travel_penalty_permille, 0);
}

#[test]
fn raising_only_candidate_movement_cost_cannot_make_relocation_more_attractive() {
    let (baseline, baseline_location) = run_two_cell_case([1_000, 1_000], 0, 0, 1);
    let (rough_candidate, rough_candidate_location) = run_two_cell_case([1_000, 4_000], 0, 0, 1);

    assert_eq!(baseline.moves_completed, 1);
    assert_eq!(baseline_location, CellId::new(2));
    assert_eq!(rough_candidate.moves_completed, 0);
    assert_eq!(rough_candidate_location, CellId::new(1));
}

#[test]
fn disabling_effective_relocation_costs_reduces_comparison_to_residence_terms() {
    let (summary, location) = run_two_cell_case([4_000, 4_000], 0, 0, 0);

    assert_eq!(summary.moves_completed, 1);
    assert_eq!(location, CellId::new(2));
    let trace = summary.recorded_decision_traces.first().unwrap();
    assert_eq!(trace.origin_utility.total_utility, 1_250);
    assert_eq!(trace.destination_utility.total_utility, 2_000);
    assert_eq!(trace.destination_utility.travel_penalty_permille, 1_000);
}
