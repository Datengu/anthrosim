use anthrosim_core::{ExperimentConfig, ResourceSystem, Simulation, WorldConfig};

#[test]
fn restored_aggregate_stock_immediately_below_u64_max_is_representable() {
    let simulation = Simulation::new(
        ExperimentConfig::new(176, 0).with_world(WorldConfig::new(2, 1)),
    )
    .expect("baseline simulation should construct");
    let mut value = serde_json::to_value(simulation.resources())
        .expect("resource state should serialize");
    value["cellFoodStock"] = serde_json::json!([u64::MAX - 2, 1_u64]);
    let restored: ResourceSystem =
        serde_json::from_value(value).expect("boundary resource state should deserialize");

    assert_eq!(restored.total_food_stock().unwrap(), u64::MAX - 1);
}
