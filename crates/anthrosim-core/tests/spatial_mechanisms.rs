use anthrosim_core::{
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    NoDataPolicy, SpatialFieldTransform, SpatialMechanismConfig, SpatialMechanismError,
    SpatialTargetField, TransformDirection, transform_landscape,
};

fn layer(
    id: &str,
    role: LandscapeLayerRole,
    values: Vec<Option<i32>>,
) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values,
    }
}

fn fixture() -> LandscapeBundle {
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 25,
            cell_size_y: 25,
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
                vec![Some(0), Some(500), Some(1_000)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(0), Some(500), Some(1_000)],
            ),
        ],
    )
}

fn transform(
    target: SpatialTargetField,
    layer_id: &str,
    min: u16,
    max: u16,
    direction: TransformDirection,
) -> SpatialFieldTransform {
    SpatialFieldTransform::new(
        target,
        layer_id,
        "normalized_index",
        LandscapeValueDomain { min: 0, max: 1_000 },
        min,
        max,
        direction,
        NoDataPolicy::Reject,
    )
}

#[test]
fn direct_transforms_preserve_expected_ordering() {
    let config = SpatialMechanismConfig::new(
        "directional_fixture_v1",
        vec![
            transform(
                SpatialTargetField::MovementCost,
                "terrain",
                1_000,
                3_000,
                TransformDirection::Direct,
            ),
            transform(
                SpatialTargetField::WaterAccess,
                "water",
                0,
                1_000,
                TransformDirection::Direct,
            ),
            transform(
                SpatialTargetField::BaseProductivity,
                "resources",
                0,
                1_000,
                TransformDirection::Direct,
            ),
        ],
    );

    let output = transform_landscape(&fixture(), &config).unwrap();
    assert_eq!(output.movement_cost, Some(vec![1_000, 2_000, 3_000]));
    assert_eq!(output.water_access, Some(vec![0, 500, 1_000]));
    assert_eq!(output.base_productivity, Some(vec![0, 500, 1_000]));
}

#[test]
fn inverse_transform_reverses_source_order_explicitly() {
    let config = SpatialMechanismConfig::new(
        "inverse_fixture_v1",
        vec![transform(
            SpatialTargetField::MovementCost,
            "terrain",
            1_000,
            3_000,
            TransformDirection::Inverse,
        )],
    );

    let output = transform_landscape(&fixture(), &config).unwrap();
    assert_eq!(output.movement_cost, Some(vec![3_000, 2_000, 1_000]));
}

#[test]
fn nodata_reject_policy_fails_explicitly() {
    let mut landscape = fixture();
    landscape.layers[1].values[1] = None;
    let config = SpatialMechanismConfig::new(
        "nodata_reject_v1",
        vec![transform(
            SpatialTargetField::WaterAccess,
            "water",
            0,
            1_000,
            TransformDirection::Direct,
        )],
    );

    assert!(matches!(
        transform_landscape(&landscape, &config),
        Err(SpatialMechanismError::NoDataRejected {
            layer_id,
            cell_index: 1
        }) if layer_id == "water"
    ));
}

#[test]
fn nodata_constant_is_declared_and_deterministic() {
    let mut landscape = fixture();
    landscape.layers[2].values[1] = None;
    let mut mapping = transform(
        SpatialTargetField::BaseProductivity,
        "resources",
        0,
        1_000,
        TransformDirection::Direct,
    );
    mapping.nodata = NoDataPolicy::Constant { value: 250 };
    let config = SpatialMechanismConfig::new("nodata_constant_v1", vec![mapping]);

    let output = transform_landscape(&landscape, &config).unwrap();
    assert_eq!(output.base_productivity, Some(vec![0, 250, 1_000]));
}

#[test]
fn layer_role_unit_and_domain_are_part_of_the_contract() {
    let config = SpatialMechanismConfig::new(
        "strict_contract_v1",
        vec![transform(
            SpatialTargetField::WaterAccess,
            "terrain",
            0,
            1_000,
            TransformDirection::Direct,
        )],
    );
    assert!(matches!(
        transform_landscape(&fixture(), &config),
        Err(SpatialMechanismError::UnexpectedLayerRole { .. })
    ));

    let mut wrong_unit = fixture();
    wrong_unit.layers[1].unit = "percent".to_owned();
    let config = SpatialMechanismConfig::new(
        "strict_unit_v1",
        vec![transform(
            SpatialTargetField::WaterAccess,
            "water",
            0,
            1_000,
            TransformDirection::Direct,
        )],
    );
    assert!(matches!(
        transform_landscape(&wrong_unit, &config),
        Err(SpatialMechanismError::UnexpectedLayerUnit { .. })
    ));
}

#[test]
fn duplicate_targets_are_rejected() {
    let config = SpatialMechanismConfig::new(
        "duplicate_target_v1",
        vec![
            transform(
                SpatialTargetField::WaterAccess,
                "water",
                0,
                1_000,
                TransformDirection::Direct,
            ),
            transform(
                SpatialTargetField::WaterAccess,
                "water",
                0,
                900,
                TransformDirection::Direct,
            ),
        ],
    );

    assert!(matches!(
        config.validate(),
        Err(SpatialMechanismError::DuplicateTarget(
            SpatialTargetField::WaterAccess
        ))
    ));
}

#[test]
fn transformation_parameters_change_spatial_identity() {
    let first = SpatialMechanismConfig::new(
        "identity_v1",
        vec![transform(
            SpatialTargetField::MovementCost,
            "terrain",
            1_000,
            2_000,
            TransformDirection::Direct,
        )],
    );
    let second = SpatialMechanismConfig::new(
        "identity_v1",
        vec![transform(
            SpatialTargetField::MovementCost,
            "terrain",
            1_000,
            3_000,
            TransformDirection::Direct,
        )],
    );

    assert_ne!(first.identity(), second.identity());
    assert_eq!(first.identity(), first.clone().identity());
}

#[test]
fn permille_targets_cannot_exceed_world_domain() {
    let invalid = SpatialMechanismConfig::new(
        "invalid_target_v1",
        vec![transform(
            SpatialTargetField::WaterAccess,
            "water",
            0,
            1_001,
            TransformDirection::Direct,
        )],
    );
    assert!(matches!(
        invalid.validate(),
        Err(SpatialMechanismError::InvalidTargetDomain {
            target: SpatialTargetField::WaterAccess,
            ..
        })
    ));
}
