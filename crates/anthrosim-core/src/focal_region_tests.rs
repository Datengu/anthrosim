use super::*;
use crate::{
    config::{ParameterProvenance, PopulationConfig, WorldConfig},
    evidence::{EvidenceRecord, EvidenceSource, ExternalInputEvidence},
    landscape::{GridGeometry, LandscapeLayer},
    rng::RngFactory,
};

fn world() -> World {
    World::generate(WorldConfig::new(4, 4), RngFactory::new(7)).unwrap()
}

fn evidence_catalog() -> EvidenceCatalog {
    EvidenceCatalog::new(vec![EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: "mask-source".to_owned(),
        provenance: ParameterProvenance::EmpiricalDerived,
        source: EvidenceSource {
            source_id: "mask-dataset".to_owned(),
            citation: "Example focal-region evidence".to_owned(),
            persistent_id: None,
            dataset_version: Some("v1".to_owned()),
            licence: Some("example".to_owned()),
            spatial_coverage: Some("validation grid".to_owned()),
            temporal_coverage: None,
        },
        original_variable: "region membership".to_owned(),
        original_units: "binary".to_owned(),
        transformation: None,
        simulation_units: "binary mask".to_owned(),
        uncertainty: None,
        applicability: "M9 focal-region validation".to_owned(),
        competing_estimates: Vec::new(),
    }])
    .with_external_inputs(vec![ExternalInputEvidence {
        input_id: "region-mask-input".to_owned(),
        evidence_id: "mask-source".to_owned(),
        format: "normalized-json-grid".to_owned(),
        spatial_reference: Some("EPSG:27700".to_owned()),
        content_digest: Some("sha256:example".to_owned()),
    }])
}

fn landscape(values: Vec<Option<i32>>) -> LandscapeBundle {
    LandscapeBundle::new(
        4,
        4,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 10,
            cell_size_y: 10,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "region-mask".to_owned(),
            role: LandscapeLayerRole::Auxiliary,
            unit: "binary".to_owned(),
            value_domain: Some(LandscapeValueDomain { min: 0, max: 1 }),
            evidence_input_id: Some("region-mask-input".to_owned()),
            values,
        }],
    )
}

fn bound_mask_fixture() -> (LandscapeBundle, EvidenceCatalog, World, FocalRegion) {
    let mut values = vec![Some(0); 16];
    values[1] = Some(1);
    values[5] = Some(1);
    values[14] = Some(1);
    let landscape = landscape(values);
    let evidence = evidence_catalog();
    let world = world();
    let region =
        FocalRegion::from_landscape_mask("region", &landscape, "region-mask", &evidence, &world)
            .unwrap();
    (landscape, evidence, world, region)
}

#[test]
fn synthetic_region_is_canonical_and_duplicate_fail_closed() {
    let region = FocalRegion::new(
        "region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4), CellId::new(2), CellId::new(3)],
    )
    .unwrap();
    assert_eq!(
        region.member_cells(),
        &[CellId::new(2), CellId::new(3), CellId::new(4)]
    );
    assert_eq!(
        FocalRegion::new(
            "duplicate",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4), CellId::new(2), CellId::new(4)]
        ),
        Err(FocalRegionError::DuplicateCell {
            cell: CellId::new(4)
        })
    );
}

#[test]
fn identity_is_order_independent_and_serialization_stable() {
    let first = FocalRegion::new(
        "region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(1), CellId::new(4), CellId::new(2)],
    )
    .unwrap();
    let second = FocalRegion::new(
        "region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4), CellId::new(2), CellId::new(1)],
    )
    .unwrap();
    assert_eq!(first.identity(), second.identity());
    let restored: FocalRegion =
        serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
    assert_eq!(restored.identity(), first.identity());
}

#[test]
fn evidence_bound_binary_mask_derives_region_and_provenance() {
    let (landscape, evidence, world, region) = bound_mask_fixture();
    assert_eq!(
        region.member_cells(),
        &[CellId::new(2), CellId::new(6), CellId::new(15)]
    );
    assert_eq!(
        region.source,
        FocalRegionSource::LandscapeMask {
            layer_id: "region-mask".to_owned(),
            evidence_input_id: "region-mask-input".to_owned(),
        }
    );
    region
        .validate_landscape_binding(&landscape, &evidence, &world)
        .unwrap();
}

#[test]
fn serialized_landscape_region_round_trip_revalidates_against_mask() {
    let (landscape, evidence, world, region) = bound_mask_fixture();
    let restored: FocalRegion =
        serde_json::from_str(&serde_json::to_string(&region).unwrap()).unwrap();
    restored
        .validate_landscape_binding(&landscape, &evidence, &world)
        .unwrap();
    assert_eq!(restored, region);
}

#[test]
fn changed_member_cells_cannot_retain_landscape_mask_provenance() {
    let (landscape, evidence, world, region) = bound_mask_fixture();
    let tampered = FocalRegion::new(
        region.region_id,
        region.source,
        vec![CellId::new(2), CellId::new(6), CellId::new(16)],
    )
    .unwrap();
    assert!(matches!(
        tampered.validate_landscape_binding(&landscape, &evidence, &world),
        Err(FocalRegionBindingError::MaskMembershipMismatch { .. })
    ));
}

#[test]
fn changed_evidence_input_cannot_retain_landscape_mask_provenance() {
    let (landscape, evidence, world, region) = bound_mask_fixture();
    let tampered = FocalRegion::new(
        region.region_id,
        FocalRegionSource::LandscapeMask {
            layer_id: "region-mask".to_owned(),
            evidence_input_id: "different-input".to_owned(),
        },
        region.member_cells().to_vec(),
    )
    .unwrap();
    assert_eq!(
        tampered.validate_landscape_binding(&landscape, &evidence, &world),
        Err(FocalRegionBindingError::MaskEvidenceInputMismatch {
            layer_id: "region-mask".to_owned(),
            declared: "different-input".to_owned(),
            bound: "region-mask-input".to_owned(),
        })
    );
}

#[test]
fn mask_nodata_and_empty_region_are_rejected() {
    let mut nodata = vec![Some(0); 16];
    nodata[3] = None;
    assert!(matches!(
        FocalRegion::from_landscape_mask(
            "region",
            &landscape(nodata),
            "region-mask",
            &evidence_catalog(),
            &world(),
        ),
        Err(FocalRegionBindingError::MaskContainsNoData { cell_index: 3, .. })
    ));
    assert_eq!(
        FocalRegion::from_landscape_mask(
            "empty",
            &landscape(vec![Some(0); 16]),
            "region-mask",
            &evidence_catalog(),
            &world(),
        ),
        Err(FocalRegionBindingError::Region(
            FocalRegionError::EmptyRegion
        ))
    );
}

#[test]
fn missing_evidence_link_is_rejected() {
    let mut ungrounded = landscape(vec![Some(1); 16]);
    ungrounded.layers[0].evidence_input_id = None;
    assert_eq!(
        FocalRegion::from_landscape_mask(
            "region",
            &ungrounded,
            "region-mask",
            &evidence_catalog(),
            &world(),
        ),
        Err(FocalRegionBindingError::MissingEvidenceInput {
            layer_id: "region-mask".to_owned()
        })
    );
}

#[test]
fn residents_are_identifiable_without_degenerate_visits() {
    let world = world();
    let population = Population::initialize(
        PopulationConfig::new(20).with_target_household_size(5),
        &world,
        RngFactory::new(11),
    )
    .unwrap();
    let household = HouseholdId::new(1);
    let residence = population.household_location(household).unwrap();
    let region = FocalRegion::new("home", FocalRegionSource::Synthetic, vec![residence]).unwrap();
    assert_eq!(
        region.contains_residence(household, &population),
        Some(true)
    );
}

#[test]
fn invalid_world_cells_fail_closed() {
    assert_eq!(
        FocalRegion::new(
            "invalid",
            FocalRegionSource::Synthetic,
            vec![CellId::INVALID]
        ),
        Err(FocalRegionError::InvalidCellId)
    );
    let outside = FocalRegion::new(
        "outside",
        FocalRegionSource::Synthetic,
        vec![CellId::new(99)],
    )
    .unwrap();
    assert_eq!(
        outside.validate(&world()),
        Err(FocalRegionError::CellOutsideWorld {
            cell: CellId::new(99)
        })
    );
}
