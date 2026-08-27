use crate::{
    config::{ExperimentConfig, ParameterProvenance, PopulationConfig, WorldConfig},
    evidence::{EvidenceCatalog, EvidenceRecord, EvidenceSource, ExternalInputEvidence},
    focal_region::{FocalRegion, FocalRegionBindingError, FocalRegionSource},
    ids::CellId,
    landscape::{
        GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    },
    rng::RngFactory,
    spatial_mechanisms::{
        NoDataPolicy, SpatialFieldTransform, SpatialMechanismConfig, SpatialTargetField,
        TransformDirection,
    },
    spatial_simulation::{SpatialLandscapeError, SpatialLandscapeSimulation},
    temporary_mobility::{
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTriggerTiming,
    },
    temporary_travel::TemporaryTravelModel,
    world::World,
};

fn evidence_catalog() -> EvidenceCatalog {
    EvidenceCatalog::new(vec![EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: "mask-source".to_owned(),
        provenance: ParameterProvenance::EmpiricalDerived,
        source: EvidenceSource {
            source_id: "mask-dataset".to_owned(),
            citation: "Spatial focal-region binding fixture".to_owned(),
            persistent_id: None,
            dataset_version: Some("v1".to_owned()),
            licence: Some("test".to_owned()),
            spatial_coverage: Some("2x2 fixture".to_owned()),
            temporal_coverage: None,
        },
        original_variable: "focal membership".to_owned(),
        original_units: "binary".to_owned(),
        transformation: None,
        simulation_units: "binary mask".to_owned(),
        uncertainty: None,
        applicability: "M9 focal-region binding test".to_owned(),
        competing_estimates: Vec::new(),
    }])
    .with_external_inputs(vec![ExternalInputEvidence {
        input_id: "mask-input".to_owned(),
        evidence_id: "mask-source".to_owned(),
        format: "normalized-json-grid".to_owned(),
        spatial_reference: Some("EPSG:27700".to_owned()),
        content_digest: Some("sha256:mask-fixture".to_owned()),
    }])
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        2,
        2,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 10,
            cell_size_y: 10,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![
            LandscapeLayer {
                layer_id: "movement".to_owned(),
                role: LandscapeLayerRole::TerrainTraversal,
                unit: "cost-index".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 1, max: 2 }),
                evidence_input_id: None,
                values: vec![Some(1), Some(2), Some(1), Some(2)],
            },
            LandscapeLayer {
                layer_id: "focal-mask".to_owned(),
                role: LandscapeLayerRole::Auxiliary,
                unit: "binary".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 1 }),
                evidence_input_id: Some("mask-input".to_owned()),
                values: vec![Some(0), Some(1), Some(0), Some(1)],
            },
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "focal-binding-test",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "movement",
            "cost-index",
            LandscapeValueDomain { min: 1, max: 2 },
            1_000,
            2_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    )
}

fn valid_config() -> ExperimentConfig {
    let landscape = landscape();
    let evidence = evidence_catalog();
    let world = World::generate(WorldConfig::new(2, 2), RngFactory::new(18_700)).unwrap();
    let region = FocalRegion::from_landscape_mask(
        "bound-region",
        &landscape,
        "focal-mask",
        &evidence,
        &world,
    )
    .unwrap();
    let temporary_mobility = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            "bound-region-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![0],
            1,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();
    ExperimentConfig::new(18_700, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(4).with_target_household_size(2))
        .with_evidence(evidence)
        .with_temporary_mobility(temporary_mobility)
}

fn tampered_region() -> FocalRegion {
    FocalRegion::new(
        "bound-region",
        FocalRegionSource::LandscapeMask {
            layer_id: "focal-mask".to_owned(),
            evidence_input_id: "mask-input".to_owned(),
        },
        vec![CellId::new(2), CellId::new(3)],
    )
    .unwrap()
}

#[test]
fn spatial_construction_accepts_exact_bound_mask() {
    SpatialLandscapeSimulation::new(valid_config(), landscape(), mechanisms()).unwrap();
}

#[test]
fn spatial_construction_rejects_edited_member_cells_with_mask_provenance() {
    let mut config = valid_config();
    config.temporary_mobility.as_mut().unwrap().region = tampered_region();
    assert!(matches!(
        SpatialLandscapeSimulation::new(config, landscape(), mechanisms()),
        Err(SpatialLandscapeError::FocalRegionBinding(
            FocalRegionBindingError::MaskMembershipMismatch { .. }
        ))
    ));
}

#[test]
fn spatial_resume_revalidates_focal_region_against_bound_landscape() {
    let mut checkpoint = SpatialLandscapeSimulation::new(valid_config(), landscape(), mechanisms())
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    checkpoint
        .core_checkpoint
        .experiment
        .temporary_mobility
        .as_mut()
        .unwrap()
        .region = tampered_region();
    checkpoint.core_checkpoint = checkpoint.core_checkpoint.seal_continuation_identity();

    assert!(matches!(
        SpatialLandscapeSimulation::from_checkpoint(checkpoint, landscape()),
        Err(SpatialLandscapeError::FocalRegionBinding(
            FocalRegionBindingError::MaskMembershipMismatch { .. }
        ))
    ));
}
