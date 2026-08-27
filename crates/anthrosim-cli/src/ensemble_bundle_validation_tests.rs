use std::time::{SystemTime, UNIX_EPOCH};

use anthrosim_core::{
    GridGeometry, LandscapeBinding, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, NoDataPolicy, SpatialFieldTransform, SpatialMechanismConfig,
    SpatialTargetField, TransformDirection,
};

use super::*;

fn settings() -> EnsembleRunSettings {
    EnsembleRunSettings {
        years: 0,
        world_width: 4,
        world_height: 4,
        population: 12,
        household_size: 4,
        max_person_records: 100,
        resource_productivity_scale_permille: 1_000,
        resource_seasonality_scale_permille: 1_000,
        annual_food_need: 100,
        disable_migration: false,
        migration_radius: 3,
        temporary_mobility: None,
        spatial: None,
    }
}

fn temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-audit-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn status(root: &Path, seed: u64) -> RunStatus {
    read_json(&root.join(status_relative_path(&run_id(seed)))).expect("status")
}

fn mutate_schema(path: &Path) {
    let mut value: serde_json::Value = read_json(path).expect("artifact JSON");
    value["schemaVersion"] = serde_json::json!(999_999_u64);
    write_json(path, &value).expect("tampered artifact");
}

fn assert_retry_rejects_integrity_tamper(root: &Path, settings: EnsembleRunSettings, seed: u64) {
    assert!(execute_ensemble(root, settings, vec![seed], true).is_err());
    let status = status(root, seed);
    assert_eq!(status.state, RunLifecycle::Incomplete);
    assert_eq!(status.attempt, 1);
    assert!(
        status
            .message
            .as_deref()
            .is_some_and(|message| message.contains("bundle integrity error"))
    );
}

#[test]
fn retry_rejects_tampered_external_events_artifact() {
    let root = temp_path("events");
    let seed = 14_301;
    execute_ensemble(&root, settings(), vec![seed], false).expect("fresh ensemble");
    mutate_schema(&root.join(run_relative_dir(seed)).join("events.json"));

    assert_retry_rejects_integrity_tamper(&root, settings(), seed);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn retry_rejects_tampered_external_metrics_artifact() {
    let root = temp_path("metrics");
    let seed = 14_302;
    execute_ensemble(&root, settings(), vec![seed], false).expect("fresh ensemble");
    mutate_schema(&root.join(run_relative_dir(seed)).join("metrics.json"));

    assert_retry_rejects_integrity_tamper(&root, settings(), seed);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn retry_rejects_tampered_world_and_founder_artifacts() {
    for (seed, artifact) in [(14_303, "world.json"), (14_304, "initial-population.json")] {
        let root = temp_path(artifact);
        execute_ensemble(&root, settings(), vec![seed], false).expect("fresh ensemble");
        mutate_schema(&root.join(run_relative_dir(seed)).join(artifact));

        assert_retry_rejects_integrity_tamper(&root, settings(), seed);
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn spatial_settings() -> (EnsembleRunSettings, PathBuf) {
    let source_root = temp_path("spatial-source");
    fs::create_dir_all(&source_root).expect("source root");
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    let layer = |id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>| LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(domain),
        evidence_input_id: None,
        values,
    };
    let landscape = LandscapeBundle::new(
        2,
        2,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[generic]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(250), Some(500), Some(1_000)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(750), Some(500), Some(250)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(250), Some(500), Some(750), Some(1_000)],
            ),
        ],
    );
    let landscape_path = source_root.join("landscape.json");
    write_json(&landscape_path, &landscape).expect("landscape");
    let mechanisms = SpatialMechanismConfig::new(
        "ensemble_bundle_validation_v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                3_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    );
    let mut settings = settings();
    settings.world_width = 2;
    settings.world_height = 2;
    settings.spatial = Some(SpatialRunSettings {
        spatial_model_semantics_id: SPATIAL_MODEL_SEMANTICS_ID.to_owned(),
        landscape_binding: LandscapeBinding::from_bundle(&landscape).expect("binding"),
        mechanisms,
        evidence: None,
        founder_population: None,
        runtime_landscape_path: Some(landscape_path),
    });
    (settings, source_root)
}

#[test]
fn retry_rejects_tampered_transformed_spatial_wrapper() {
    let root = temp_path("spatial-wrapper");
    let seed = 14_305;
    let (settings, source_root) = spatial_settings();
    execute_ensemble(&root, settings.clone(), vec![seed], false).expect("fresh spatial ensemble");
    mutate_schema(
        &root
            .join(run_relative_dir(seed))
            .join("landscape-checkpoint.json"),
    );

    assert_retry_rejects_integrity_tamper(&root, settings, seed);
    fs::remove_dir_all(root).expect("cleanup");
    fs::remove_dir_all(source_root).expect("cleanup source");
}
