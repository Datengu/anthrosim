use anthrosim_core::{ExperimentConfig, Simulation, WorldConfig};

#[test]
fn identical_runs_emit_identical_manifests() {
    let config = ExperimentConfig::new(18_472_910, 10_000).with_world(WorldConfig::new(96, 64));

    let a = Simulation::new(config.clone()).unwrap().run();
    let b = Simulation::new(config).unwrap().run();

    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
}

#[test]
fn different_seeds_change_world_digest() {
    let a = Simulation::new(ExperimentConfig::new(1, 100))
        .unwrap()
        .run();
    let b = Simulation::new(ExperimentConfig::new(2, 100))
        .unwrap()
        .run();
    assert_ne!(a.world.digest64, b.world.digest64);
}
