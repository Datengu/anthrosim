use anthrosim_core::{
    EvidenceCatalog, EvidenceRecord, EvidenceSource, ExternalInputEvidence, GridGeometry,
    LandscapeBundle, LandscapeError, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    ParameterProvenance,
};

fn geometry() -> GridGeometry {
    GridGeometry {
        origin_x: 500_000,
        origin_y: 200_000,
        cell_size_x: 25,
        cell_size_y: 25,
        coordinate_unit: "metre".to_owned(),
        spatial_reference: "EPSG:27700".to_owned(),
    }
}

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "permille".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values,
    }
}

fn bundle() -> LandscapeBundle {
    LandscapeBundle::new(
        2,
        2,
        geometry(),
        vec![
            layer(
                "terrain_traversal",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(100), Some(200), None, Some(400)],
            ),
            layer(
                "water_accessibility",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(800), Some(700), Some(600), Some(500)],
            ),
        ],
    )
}

#[test]
fn valid_bundle_round_trips_and_keeps_identity() {
    let bundle = bundle();
    bundle.validate().expect("fixture should be valid");

    let json = serde_json::to_string_pretty(&bundle).expect("serialize landscape");
    let decoded: LandscapeBundle = serde_json::from_str(&json).expect("deserialize landscape");

    assert_eq!(decoded, bundle);
    assert_eq!(decoded.identity(), bundle.identity());
    assert!(json.contains("EPSG:27700"));
    assert!(json.contains("null"));
}

#[test]
fn identity_changes_when_authoritative_normalized_input_changes() {
    let first = bundle();
    let mut second = first.clone();
    second.layers[0].values[1] = Some(201);

    assert_ne!(first.digest64(), second.digest64());
    assert_ne!(first.identity(), second.identity());
}

#[test]
fn rejects_duplicate_layers_and_invalid_lengths() {
    let mut duplicate = bundle();
    duplicate.layers.push(duplicate.layers[0].clone());
    assert!(matches!(
        duplicate.validate(),
        Err(LandscapeError::DuplicateLayerId(id)) if id == "terrain_traversal"
    ));

    let mut short = bundle();
    short.layers[0].values.pop();
    assert!(matches!(
        short.validate(),
        Err(LandscapeError::LayerLengthMismatch {
            expected: 4,
            actual: 3,
            ..
        })
    ));
}

#[test]
fn rejects_out_of_domain_values_but_allows_explicit_nodata() {
    let mut invalid = bundle();
    invalid.layers[1].values[2] = Some(1_001);
    assert!(matches!(
        invalid.validate(),
        Err(LandscapeError::ValueOutOfDomain {
            cell_index: 2,
            value: 1_001,
            ..
        })
    ));

    let mut nodata = bundle();
    nodata.layers[1].values[2] = None;
    nodata.validate().expect("explicit nodata should be valid");
}

#[test]
fn evidence_input_links_resolve_against_experiment_catalogue() {
    let record = EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: "terrain_source".to_owned(),
        provenance: ParameterProvenance::EmpiricalDerived,
        source: EvidenceSource {
            source_id: "source".to_owned(),
            citation: "Publishable example terrain source".to_owned(),
            persistent_id: None,
            dataset_version: Some("1".to_owned()),
            licence: Some("example".to_owned()),
            spatial_coverage: Some("generic test fixture".to_owned()),
            temporal_coverage: None,
        },
        original_variable: "elevation".to_owned(),
        original_units: "metre".to_owned(),
        transformation: None,
        simulation_units: "permille".to_owned(),
        uncertainty: None,
        applicability: "M8.1 contract test only".to_owned(),
        competing_estimates: Vec::new(),
    };
    let catalog = EvidenceCatalog::new(vec![record]).with_external_inputs(vec![
        ExternalInputEvidence {
            input_id: "terrain_normalized".to_owned(),
            evidence_id: "terrain_source".to_owned(),
            format: "anthrosim-landscape-v1".to_owned(),
            spatial_reference: Some("EPSG:27700".to_owned()),
            content_digest: Some("fixture".to_owned()),
        },
    ]);
    catalog.validate().expect("catalog should be valid");

    let mut linked = bundle();
    linked.layers[0].evidence_input_id = Some("terrain_normalized".to_owned());
    linked
        .validate_evidence_links(&catalog)
        .expect("known external input should resolve");

    linked.layers[0].evidence_input_id = Some("missing".to_owned());
    assert!(matches!(
        linked.validate_evidence_links(&catalog),
        Err(LandscapeError::UnknownEvidenceInput { input_id, .. }) if input_id == "missing"
    ));
}

#[test]
fn invalid_geometry_is_rejected_explicitly() {
    let mut invalid = bundle();
    invalid.geometry.cell_size_x = 0;
    assert_eq!(invalid.validate(), Err(LandscapeError::InvalidCellSize));

    let mut invalid = bundle();
    invalid.geometry.spatial_reference.clear();
    assert_eq!(invalid.validate(), Err(LandscapeError::EmptySpatialReference));
}
