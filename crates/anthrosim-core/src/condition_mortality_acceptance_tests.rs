use serde_json::json;

use crate::{
    EventLog, PopulationConfig, ResourceConfig, ResourceSummary, WorldConfig,
    config::{MigrationConfig, ParameterProvenance, PopulationInitialization},
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    migration::{MigrationBoundaryContext, MigrationRngs, MigrationSystem},
    population::{Population, ReproductiveSex},
    resources::{
        ResourcePeriodContext, ResourceRngs, ResourceSystem, fixed_annual_quantity_for_period,
        resource_period_day_bounds,
    },
    rng::RngFactory,
    world::World,
};

fn one_person_world(seed: u64) -> (World, Population) {
    let world = World::generate(WorldConfig::new(8, 8), RngFactory::new(seed)).unwrap();
    let population = Population::initialize(
        PopulationConfig::new(1).with_target_household_size(1),
        &world,
        RngFactory::new(seed),
    )
    .unwrap();
    (world, population)
}

fn full_supply_destination(
    population: &Population,
    resources: &ResourceSystem,
    world: &World,
) -> CellId {
    let origin = population.household_location(HouseholdId::new(1)).unwrap();
    (1..=world.cell_count())
        .filter_map(|index| u64::try_from(index).ok().map(CellId::new))
        .find(|&cell| cell != origin && resources.cell_food_stock(cell).unwrap_or(0) >= 1)
        .expect("controlled world must expose a stocked non-origin cell")
}

fn process_one_year(
    population: &mut Population,
    world: &World,
    resources: &mut ResourceSystem,
    config: &ResourceConfig,
    seed: u64,
) -> EventLog {
    let mut events = EventLog::new();
    let mut rngs = ResourceRngs::new(RngFactory::new(seed));
    resources
        .process_period_recorded(
            population,
            &ResourcePeriodContext {
                world,
                config,
                period_index_in_year: 0,
                day: 365,
            },
            &mut rngs.scarcity_mortality,
            &mut events,
        )
        .unwrap();
    events
}

fn assert_general_condition_death(events: &EventLog) {
    assert_eq!(events.events.len(), 1);
    let serialized = serde_json::to_value(events).unwrap();
    assert_eq!(
        serialized["events"][0]["event"]["cause"],
        json!("condition_mediated")
    );
    assert_ne!(
        serialized["events"][0]["event"]["cause"],
        json!("resource_scarcity")
    );
}

fn assert_condition_mortality_summary_wire(summary: &ResourceSummary) {
    let serialized = serde_json::to_value(summary).unwrap();
    assert_eq!(serialized["conditionMortalityDeaths"], json!(1));
    assert!(serialized.get("scarcityDeaths").is_none());
}

#[test]
fn travel_only_low_condition_with_full_supply_is_not_labelled_resource_scarcity() {
    let seed = 801;
    let (world, mut population) = one_person_world(seed);
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 1;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 1_000_000;

    let mut resources = ResourceSystem::initialize(&world, &config).unwrap();
    let destination = full_supply_destination(&population, &resources, &world);
    let relocation = population
        .apply_household_relocations(&[destination], &[1_000], &world)
        .unwrap();
    assert_eq!(relocation.people_moved, 1);
    assert_eq!(relocation.condition_loss_total, 1_000);
    assert_eq!(population.condition_at_index(0), Some(0));

    let events = process_one_year(&mut population, &world, &mut resources, &config, seed);
    let summary = resources.summary(&population);

    assert_eq!(summary.unmet_need, 0, "food need must be fully supplied");
    assert_eq!(summary.scarcity_deaths, 1);
    assert_general_condition_death(&events);
    assert_condition_mortality_summary_wire(&summary);
}

#[test]
fn resource_only_condition_loss_still_directionally_generates_condition_mortality() {
    let seed = 802;
    let (world, mut population) = one_person_world(seed);
    let mut config = ResourceConfig::synthetic_validation_v1().with_productivity_scale_permille(0);
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 1;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 1_000;
    config.max_scarcity_mortality_probability_per_million = 1_000_000;

    let mut resources = ResourceSystem::initialize(&world, &config).unwrap();
    let events = process_one_year(&mut population, &world, &mut resources, &config, seed);
    let summary = resources.summary(&population);

    assert_eq!(summary.unmet_need, 1);
    assert_eq!(summary.scarcity_deaths, 1);
    assert_general_condition_death(&events);
    assert_condition_mortality_summary_wire(&summary);
}

#[test]
fn mixed_travel_and_resource_loss_remains_general_in_cause_attribution() {
    let seed = 803;
    let (world, mut population) = one_person_world(seed);
    let mut config = ResourceConfig::synthetic_validation_v1().with_productivity_scale_permille(0);
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 1;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 500;
    config.max_scarcity_mortality_probability_per_million = 1_000_000;

    let mut resources = ResourceSystem::initialize(&world, &config).unwrap();
    let origin = population.household_location(HouseholdId::new(1)).unwrap();
    let destination = (1..=world.cell_count())
        .filter_map(|index| u64::try_from(index).ok().map(CellId::new))
        .find(|&cell| cell != origin)
        .unwrap();
    let relocation = population
        .apply_household_relocations(&[destination], &[500], &world)
        .unwrap();
    assert_eq!(relocation.condition_loss_total, 500);
    assert_eq!(population.condition_at_index(0), Some(500));

    let events = process_one_year(&mut population, &world, &mut resources, &config, seed);
    let summary = resources.summary(&population);

    assert_eq!(summary.unmet_need, 1);
    assert_eq!(summary.scarcity_deaths, 1);
    assert_general_condition_death(&events);
    assert_condition_mortality_summary_wire(&summary);
}

fn run_full_support_migration_case(enabled: bool) -> (u64, CellId, u16) {
    const ANNUAL_NEED: u64 = 100;
    let current_need = fixed_annual_quantity_for_period(ANNUAL_NEED, 0, 4).unwrap();
    let (seed, world, resources) = (1_900_u64..2_000)
        .find_map(|seed| {
            let factory = RngFactory::new(seed);
            let world = World::generate(WorldConfig::new(2, 1), factory)
                .unwrap()
                .with_model_field_overlay(
                    Some(&[1_000, 1_000]),
                    Some(&[0, 1_000]),
                    Some(&[1_000, 1_000]),
                )
                .unwrap();
            let resource_config = ResourceConfig::synthetic_validation_v1()
                .with_annual_need_units_per_person(u32::try_from(ANNUAL_NEED).unwrap());
            let resources = ResourceSystem::initialize(&world, &resource_config).unwrap();
            (resources.cell_food_stock(CellId::new(1)).unwrap_or(0) >= current_need
                && resources.cell_food_stock(CellId::new(2)).unwrap_or(0) >= current_need)
                .then_some((seed, world, resources))
        })
        .expect("controlled search must find a fully supported two-cell world");

    let definition = FounderPopulationDefinition::new(
        "condition-mortality-migration-switch-v1",
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
            condition_permille: 500,
        }],
    );
    let population_config = PopulationConfig::new(1)
        .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
    let mut population =
        Population::initialize_declared_founder_state_v1(population_config, &definition, &world)
            .unwrap();

    let mut migration_config = MigrationConfig::synthetic_validation_v1();
    migration_config.enabled = enabled;
    migration_config.candidate_radius_cells = 1;
    migration_config.condition_pressure_threshold_permille = 1_000;
    migration_config.resource_pressure_threshold_permille = 0;
    migration_config.minimum_utility_improvement = 0;
    migration_config.resource_weight = 1;
    migration_config.water_security_weight = 1;
    migration_config.kin_weight = 0;
    migration_config.travel_cost_weight = 0;
    migration_config.max_uncertainty_penalty_permille = 0;
    migration_config.relocation_risk_base_penalty_permille = 0;
    migration_config.relocation_risk_per_cell_permille = 0;
    migration_config.travel_condition_cost_per_cell = 100;
    migration_config.max_recorded_decision_traces = 8;
    let decision_day = resource_period_day_bounds(0, migration_config.decision_periods_per_year)
        .unwrap()
        .1;

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
                annual_food_need: ANNUAL_NEED,
                decision_periods_per_year: migration_config.decision_periods_per_year,
                decision_index_in_year: 0,
                day: decision_day,
            },
            &mut rngs,
            &mut events,
        )
        .unwrap();

    (
        migration.summary().moves_completed,
        population.household_location(HouseholdId::new(1)).unwrap(),
        population.condition_at_index(0).unwrap(),
    )
}

#[test]
fn migration_switch_is_the_only_source_of_travel_condition_loss_under_full_support() {
    let (enabled_moves, enabled_location, enabled_condition) =
        run_full_support_migration_case(true);
    let (disabled_moves, disabled_location, disabled_condition) =
        run_full_support_migration_case(false);

    assert_eq!(enabled_moves, 1);
    assert_eq!(enabled_location, CellId::new(2));
    assert_eq!(enabled_condition, 400);
    assert_eq!(disabled_moves, 0);
    assert_eq!(disabled_location, CellId::new(1));
    assert_eq!(disabled_condition, 500);
}

#[test]
fn v10_resource_config_requires_general_condition_mortality_wire_name() {
    let config = ResourceConfig::synthetic_validation_v1();
    let serialized = serde_json::to_value(&config).unwrap();
    assert_eq!(serialized["schemaVersion"], json!(4));
    assert_eq!(
        serialized["maxConditionMortalityProbabilityPerMillion"],
        json!(200_000)
    );
    assert!(
        serialized
            .get("maxScarcityMortalityProbabilityPerMillion")
            .is_none()
    );

    let mut old_wire = serialized;
    let object = old_wire.as_object_mut().unwrap();
    let value = object
        .remove("maxConditionMortalityProbabilityPerMillion")
        .unwrap();
    object.insert(
        "maxScarcityMortalityProbabilityPerMillion".to_owned(),
        value,
    );
    assert!(serde_json::from_value::<ResourceConfig>(old_wire).is_err());
}
