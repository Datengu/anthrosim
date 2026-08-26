use serde_json::json;

use crate::{
    EventLog, PopulationConfig, ResourceConfig, ResourceSummary, WorldConfig,
    ids::{CellId, HouseholdId},
    population::Population,
    resources::{ResourcePeriodContext, ResourceRngs, ResourceSystem},
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
