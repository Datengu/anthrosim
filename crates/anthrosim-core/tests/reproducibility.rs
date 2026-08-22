use anthrosim_core::{ExperimentConfig, PopulationConfig, Simulation, WorldConfig};

#[test]
fn identical_runs_emit_identical_resource_demographic_trajectories() {
    let config = ExperimentConfig::new(18_472_910, 20)
        .with_world(WorldConfig::new(32, 24))
        .with_population(PopulationConfig::new(2_000).with_max_person_records(100_000));

    let a = Simulation::new(config.clone()).unwrap().run().unwrap();
    let b = Simulation::new(config).unwrap().run().unwrap();

    assert_eq!(a, b);
    assert_eq!(a.population.digest64, b.population.digest64);
    assert_eq!(a.resources.digest64, b.resources.digest64);
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
}

#[test]
fn different_seeds_change_world_resource_and_population_trajectory() {
    let a = Simulation::new(ExperimentConfig::new(1, 10))
        .unwrap()
        .run()
        .unwrap();
    let b = Simulation::new(ExperimentConfig::new(2, 10))
        .unwrap()
        .run()
        .unwrap();
    assert_ne!(a.world.digest64, b.world.digest64);
    assert_ne!(a.resources.digest64, b.resources.digest64);
    assert_ne!(a.population.digest64, b.population.digest64);
}
