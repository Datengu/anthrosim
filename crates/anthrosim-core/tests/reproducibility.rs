use anthrosim_core::{ExperimentConfig, Simulation};

#[test]
fn identical_skeleton_runs_emit_identical_manifests() {
    let config = ExperimentConfig::new(18_472_910, 10_000);

    let a = Simulation::new(config.clone()).run();
    let b = Simulation::new(config).run();

    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
}
