use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBindingError, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeSimulation, LandscapeValueDomain, PopulationConfig, WorldConfig,
    validate_landscape_recorded_run_invariants,
};

fn fixture() -> LandscapeBundle {
    LandscapeBundle::new(
        2,
        2,
        GridGeometry {
            origin_x: 100,
            origin_y: 200,
            cell_size_x: 25,
            cell_size_y: 25,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![
            LandscapeLayer {
                layer_id: "terrain".to_owned(),
                role: LandscapeLayerRole::TerrainTraversal,
                unit: "normalized_index".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
                evidence_input_id: None,
                values: vec![Some(100), Some(200), Some(300), Some(400)],
            },
            LandscapeLayer {
                layer_id: "water".to_owned(),
                role: LandscapeLayerRole::WaterAccessibility,
                unit: "normalized_index".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
                evidence_input_id: None,
                values: vec![Some(900), Some(700), Some(500), Some(300)],
            },
        ],
    )
}

fn config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 4)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(40).with_max_person_records(10_000))
}

#[test]
fn same_config_seed_and_landscape_are_identical() {
    let first = LandscapeSimulation::new(config(8001), fixture())
        .unwrap()
        .run_recorded()
        .unwrap();
    let second = LandscapeSimulation::new(config(8001), fixture())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(first, second);
    validate_landscape_recorded_run_invariants(&first).unwrap();
}

#[test]
fn checkpoint_resume_matches_uninterrupted_landscape_run() {
    let uninterrupted = LandscapeSimulation::new(config(8002), fixture())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = LandscapeSimulation::new(config(8002), fixture())
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let resumed = LandscapeSimulation::from_checkpoint(checkpoint, fixture())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(resumed, uninterrupted);
}

#[test]
fn modified_landscape_is_rejected_on_resume() {
    let checkpoint = LandscapeSimulation::new(config(8003), fixture())
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let mut modified = fixture();
    modified.layers[0].values[0] = Some(101);

    assert!(matches!(
        LandscapeSimulation::from_checkpoint(checkpoint, modified),
        Err(LandscapeBindingError::BindingMismatch { .. })
    ));
}

#[test]
fn landscape_grid_must_match_simulation_grid() {
    let mismatched = ExperimentConfig::new(8004, 1)
        .with_world(WorldConfig::new(3, 2))
        .with_population(PopulationConfig::new(10));

    assert!(matches!(
        LandscapeSimulation::new(mismatched, fixture()),
        Err(LandscapeBindingError::GridMismatch { .. })
    ));
}

#[test]
fn wrapper_artifacts_preserve_machine_readable_landscape_identity() {
    let run = LandscapeSimulation::new(config(8005), fixture())
        .unwrap()
        .run_recorded()
        .unwrap();
    let manifest_json = serde_json::to_value(&run.manifest).unwrap();
    let checkpoint_json = serde_json::to_value(&run.checkpoint).unwrap();

    assert_eq!(
        manifest_json["landscape"]["landscapeIdentity"],
        checkpoint_json["landscape"]["landscapeIdentity"]
    );
    assert_eq!(
        manifest_json["landscape"]["landscapeDigest64"],
        checkpoint_json["landscape"]["landscapeDigest64"]
    );
    assert_eq!(manifest_json["landscape"]["width"], 2);
    assert_eq!(manifest_json["landscape"]["height"], 2);
}