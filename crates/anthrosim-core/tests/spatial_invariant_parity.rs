use anthrosim_core::{
    DemographyConfig, ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig,
    ResourceConfig, SpatialFieldTransform, SpatialInvariantError, SpatialLandscapeSimulation,
    SpatialMechanismConfig, SpatialTargetField, TransformDirection, WorldConfig,
    validate_spatial_landscape_recorded_run,
};

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values,
    }
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 100,
            cell_size_y: 100,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(500), Some(1_000)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(500), Some(0)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(250), Some(500), Some(750)],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "spatial-invariant-parity",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                LandscapeValueDomain { min: 0, max: 1_000 },
                1_000,
                3_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                LandscapeValueDomain { min: 0, max: 1_000 },
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                LandscapeValueDomain { min: 0, max: 1_000 },
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
}

fn config(seed: u64) -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 4)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(30).with_max_person_records(1_000))
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn run(seed: u64) -> anthrosim_core::SpatialLandscapeRecordedRun {
    SpatialLandscapeSimulation::new(config(seed), landscape(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap()
}

fn assert_core_invariant_failure(result: Result<(), SpatialInvariantError>, context: &str) {
    assert!(
        matches!(result, Err(SpatialInvariantError::Core(_))),
        "{context} should fail the shared core invariant suite, got {result:?}"
    );
}

#[test]
fn valid_transformed_run_passes_full_shared_core_invariants() {
    let run = run(14_201);
    validate_spatial_landscape_recorded_run(&run, &landscape()).unwrap();
}

#[test]
fn transformed_run_rejects_tampered_authoritative_event_history() {
    let mut run = run(14_202);
    let first = run
        .checkpoint
        .core_checkpoint
        .events
        .events
        .first_mut()
        .expect("forced mortality fixture must emit authoritative events");
    first.sequence = first.sequence.saturating_add(1);
    run.checkpoint.core_checkpoint = run.checkpoint.core_checkpoint.seal_continuation_identity();

    assert_core_invariant_failure(
        validate_spatial_landscape_recorded_run(&run, &landscape()),
        "tampered event sequence",
    );
}

#[test]
fn transformed_run_rejects_tampered_derived_metrics() {
    let mut run = run(14_203);
    let terminal = run
        .checkpoint
        .core_checkpoint
        .metrics
        .snapshots
        .last_mut()
        .expect("completed transformed run must have a terminal metric snapshot");
    terminal.state_digest64 ^= 1;
    run.checkpoint.core_checkpoint = run.checkpoint.core_checkpoint.seal_continuation_identity();

    assert_core_invariant_failure(
        validate_spatial_landscape_recorded_run(&run, &landscape()),
        "tampered terminal metric",
    );
}

#[test]
fn transformed_run_rejects_tampered_manifest_statistics() {
    let mut run = run(14_204);
    run.manifest
        .core_manifest
        .statistics
        .authoritative_event_count = run
        .manifest
        .core_manifest
        .statistics
        .authoritative_event_count
        .saturating_add(1);

    assert_core_invariant_failure(
        validate_spatial_landscape_recorded_run(&run, &landscape()),
        "tampered manifest statistics",
    );
}
