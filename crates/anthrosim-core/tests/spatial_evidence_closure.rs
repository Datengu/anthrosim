use anthrosim_core::{
    EvidenceCatalog, EvidenceClosureFailureClass, EvidenceClosureStatus, ExperimentConfig,
    LandscapeBundle, SpatialMechanismConfig, assess_spatial_evidence_closure,
};

fn fixture() -> (EvidenceCatalog, LandscapeBundle, SpatialMechanismConfig) {
    let evidence = serde_json::from_str(include_str!(
        "../../../examples/m8-first-evidence-grounded-benchmark/evidence.json"
    ))
    .expect("committed M8 evidence fixture");
    let landscape = serde_json::from_str(include_str!(
        "../../../examples/m8-first-evidence-grounded-benchmark/landscape.json"
    ))
    .expect("committed M8 landscape fixture");
    let mechanisms = serde_json::from_str(include_str!(
        "../../../examples/m8-first-evidence-grounded-benchmark/spatial-mechanisms-strong.json"
    ))
    .expect("committed M8 mechanism fixture");
    (evidence, landscape, mechanisms)
}

#[test]
fn committed_m8_evidence_grounded_terrain_fixture_closes_its_declared_claims() {
    let (evidence, landscape, mechanisms) = fixture();
    let experiment = ExperimentConfig::default().with_evidence(evidence);

    let assessment = assess_spatial_evidence_closure(&experiment, &landscape, &mechanisms);
    assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
    assert!(assessment.failures.is_empty());
}

#[test]
fn missing_content_identity_on_the_used_m8_layer_fails_closed() {
    let (mut evidence, landscape, mechanisms) = fixture();
    evidence.external_inputs[0].content_digest = None;
    let experiment = ExperimentConfig::default().with_evidence(evidence);

    let assessment = assess_spatial_evidence_closure(&experiment, &landscape, &mechanisms);
    assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
    assert!(assessment.failures.iter().any(|failure| {
        failure.subject == "landscape.layers.terrain_contrast"
            && failure.class == EvidenceClosureFailureClass::ExternalInputMissingContentIdentity
    }));
}
