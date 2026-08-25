use anthrosim_core::{
    EvidenceCatalog, EvidenceError, EvidenceRecord, EvidenceSource, EvidenceTransformation,
    EvidenceUncertainty, ExperimentConfig, ExternalInputEvidence, ParameterEvidenceLink,
    ParameterProvenance, Simulation, SimulationError,
};

fn evidence_record() -> EvidenceRecord {
    EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: "resource-estimate-a".to_owned(),
        provenance: ParameterProvenance::EmpiricalDerived,
        source: EvidenceSource {
            source_id: "example-dataset-v1".to_owned(),
            citation: "Example published dataset".to_owned(),
            persistent_id: Some("doi:10.example/example".to_owned()),
            dataset_version: Some("v1".to_owned()),
            licence: Some("example-licence".to_owned()),
            spatial_coverage: Some("example study area".to_owned()),
            temporal_coverage: Some("example archaeological period".to_owned()),
        },
        original_variable: "estimated annual yield".to_owned(),
        original_units: "kg/ha/year".to_owned(),
        transformation: Some(EvidenceTransformation {
            method: "documented unit conversion and spatial aggregation".to_owned(),
            source_units: Some("kg/ha/year".to_owned()),
            simulation_units: "abstract resource units/cell/year".to_owned(),
            notes: Some("Example only; not an AnthroSim empirical preset".to_owned()),
        }),
        simulation_units: "abstract resource units/cell/year".to_owned(),
        uncertainty: Some(EvidenceUncertainty {
            representation: "range".to_owned(),
            value: "source-defined low-high interval".to_owned(),
        }),
        applicability: "Example provenance wiring only".to_owned(),
        competing_estimates: vec!["alternative published estimate".to_owned()],
    }
}

fn evidence_catalog() -> EvidenceCatalog {
    EvidenceCatalog::new(vec![evidence_record()])
        .with_parameter_links(vec![ParameterEvidenceLink {
            parameter_path: "resources.annualRegenerationUnitsPerProductivity".to_owned(),
            evidence_id: "resource-estimate-a".to_owned(),
            note: Some("Parameter remains explicit in the serialized experiment".to_owned()),
        }])
        .with_external_inputs(vec![ExternalInputEvidence {
            input_id: "terrain-dem".to_owned(),
            evidence_id: "resource-estimate-a".to_owned(),
            format: "GeoTIFF".to_owned(),
            spatial_reference: Some("EPSG:27700".to_owned()),
            content_digest: Some("sha256:example".to_owned()),
        }])
}

#[test]
fn synthetic_experiment_remains_lightweight_and_omits_empty_evidence() {
    let config = ExperimentConfig::new(36_001, 10);
    let json = serde_json::to_value(config).unwrap();

    assert!(json.get("evidence").is_none());
}

#[test]
fn evidence_catalog_is_serialized_inside_reproducible_experiment_identity() {
    let catalog = evidence_catalog();
    catalog.validate().unwrap();

    let without_evidence = ExperimentConfig::new(36_002, 10);
    let with_evidence = without_evidence.clone().with_evidence(catalog.clone());
    catalog.validate_against_experiment(&with_evidence).unwrap();
    Simulation::new(with_evidence.clone()).expect("valid evidence path must pass run preflight");

    let plain = serde_json::to_string(&without_evidence).unwrap();
    let grounded = serde_json::to_string(&with_evidence).unwrap();
    assert_ne!(plain, grounded);

    let json = serde_json::to_value(with_evidence).unwrap();
    let evidence = json.get("evidence").expect("evidence catalogue serialized");
    assert_eq!(evidence.get("schemaVersion").unwrap(), 1);
    assert_eq!(evidence["records"][0]["evidenceId"], "resource-estimate-a");
    assert_eq!(
        evidence["parameterLinks"][0]["parameterPath"],
        "resources.annualRegenerationUnitsPerProductivity"
    );
    assert_eq!(evidence["externalInputs"][0]["format"], "GeoTIFF");
}

#[test]
fn authoritative_run_preflight_rejects_nonexistent_parameter_path() {
    let catalog = EvidenceCatalog::new(vec![evidence_record()]).with_parameter_links(vec![
        ParameterEvidenceLink {
            parameter_path: "resources.annualRegeneratonUnitsPerProductivity".to_owned(),
            evidence_id: "resource-estimate-a".to_owned(),
            note: None,
        },
    ]);
    assert_eq!(catalog.validate(), Ok(()));
    let config = ExperimentConfig::new(36_003, 1).with_evidence(catalog);

    assert!(matches!(
        Simulation::new(config),
        Err(SimulationError::Evidence(EvidenceError::UnknownParameterPath {
            parameter_path,
            ..
        })) if parameter_path == "resources.annualRegeneratonUnitsPerProductivity"
    ));
}
