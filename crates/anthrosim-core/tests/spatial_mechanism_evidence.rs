use anthrosim_core::{
    EvidenceCatalog, EvidenceRecord, EvidenceSource, LandscapeValueDomain, NoDataPolicy,
    ParameterProvenance, SpatialFieldTransform, SpatialMechanismConfig, SpatialMechanismError,
    SpatialTargetField, TransformDirection,
};

fn transform() -> SpatialFieldTransform {
    SpatialFieldTransform::new(
        SpatialTargetField::WaterAccess,
        "water",
        "normalized_index",
        LandscapeValueDomain { min: 0, max: 1_000 },
        0,
        1_000,
        TransformDirection::Direct,
        NoDataPolicy::Reject,
    )
}

fn catalog(evidence_id: &str) -> EvidenceCatalog {
    EvidenceCatalog::new(vec![EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: evidence_id.to_owned(),
        provenance: ParameterProvenance::EvidenceInformed,
        source: EvidenceSource {
            source_id: "generic_source".to_owned(),
            citation: "Generic published evidence used only for provenance-contract testing"
                .to_owned(),
            persistent_id: None,
            dataset_version: None,
            licence: None,
            spatial_coverage: None,
            temporal_coverage: None,
        },
        original_variable: "water opportunity index".to_owned(),
        original_units: "normalized_index".to_owned(),
        transformation: None,
        simulation_units: "permille".to_owned(),
        uncertainty: None,
        applicability: "Generic test fixture; not a real-site estimate".to_owned(),
        competing_estimates: Vec::new(),
    }])
}

#[test]
fn declared_transform_evidence_must_exist_in_experiment_catalogue() {
    let config = SpatialMechanismConfig::new(
        "evidence_link_test",
        vec![transform().with_evidence_id("water_assumption")],
    );
    config
        .validate_evidence_links(Some(&catalog("water_assumption")))
        .unwrap();

    assert!(matches!(
        config.validate_evidence_links(None),
        Err(SpatialMechanismError::MissingEvidenceCatalog { .. })
    ));
    assert!(matches!(
        config.validate_evidence_links(Some(&catalog("different_assumption"))),
        Err(SpatialMechanismError::UnknownEvidenceReference { .. })
    ));
}

#[test]
fn evidence_link_is_part_of_transformation_identity() {
    let without_link = SpatialMechanismConfig::new("evidence_link_test", vec![transform()]);
    let with_link = SpatialMechanismConfig::new(
        "evidence_link_test",
        vec![transform().with_evidence_id("water_assumption")],
    );

    assert_ne!(without_link.identity(), with_link.identity());
}
