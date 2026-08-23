use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig, ResourceConfig,
    SpatialFieldTransform, SpatialLandscapeError, SpatialLandscapeSimulation,
    SpatialMechanismConfig, SpatialTargetField, TransformDirection, WorldConfig, ids::CellId,
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

fn fixture() -> LandscapeBundle {
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 100,
            origin_y: 200,
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

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "generic_spatial_null_v1",
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
    ExperimentConfig::new(seed, 4)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(60).with_max_person_records(10_000))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn transformed_world_uses_declared_model_facing_fields() {
    let source = fixture();
    let simulation =
        SpatialLandscapeSimulation::new(config(9001), source.clone(), mechanisms()).unwrap();

    assert_eq!(simulation.landscape(), &source);
    let cells = simulation.world().cells();
    assert_eq!(cells[0].movement_cost, 1_000);
    assert_eq!(cells[1].movement_cost, 2_000);
    assert_eq!(cells[2].movement_cost, 3_000);
    assert_eq!(cells[0].water_access, 0);
    assert_eq!(cells[1].water_access, 500);
    assert_eq!(cells[2].water_access, 1_000);
    assert_eq!(cells[0].base_productivity, 0);
    assert_eq!(cells[1].base_productivity, 500);
    assert_eq!(cells[2].base_productivity, 1_000);
    assert_eq!(cells[0].food_stock, 0);
    assert_eq!(cells[1].food_stock, 5_000);
    assert_eq!(cells[2].food_stock, 10_000);

    assert_eq!(
        simulation.resources().cell_food_stock(CellId::new(1)),
        Some(0)
    );
    assert_eq!(
        simulation.resources().cell_food_stock(CellId::new(2)),
        Some(5_000)
    );
    assert_eq!(
        simulation.resources().cell_food_stock(CellId::new(3)),
        Some(10_000)
    );
}

#[test]
fn same_inputs_produce_identical_spatial_runs() {
    let first = SpatialLandscapeSimulation::new(config(9002), fixture(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap();
    let second = SpatialLandscapeSimulation::new(config(9002), fixture(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(first, second);
    validate_spatial_landscape_recorded_run(&first, &fixture()).unwrap();
}

#[test]
fn transformed_checkpoint_resume_matches_uninterrupted() {
    let uninterrupted = SpatialLandscapeSimulation::new(config(9003), fixture(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap();
    let checkpoint = SpatialLandscapeSimulation::new(config(9003), fixture(), mechanisms())
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, fixture())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(resumed, uninterrupted);
}

#[test]
fn resume_rejects_modified_source_landscape() {
    let checkpoint = SpatialLandscapeSimulation::new(config(9004), fixture(), mechanisms())
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let mut modified = fixture();
    modified.layers[0].values[0] = Some(1);

    assert!(matches!(
        SpatialLandscapeSimulation::from_checkpoint(checkpoint, modified),
        Err(SpatialLandscapeError::LandscapeBinding(_))
    ));
}

#[test]
fn resume_rejects_tampered_transform_configuration() {
    let mut checkpoint = SpatialLandscapeSimulation::new(config(9005), fixture(), mechanisms())
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    checkpoint.spatial.config.transforms[0].target_max = 2_500;

    assert!(matches!(
        SpatialLandscapeSimulation::from_checkpoint(checkpoint, fixture()),
        Err(SpatialLandscapeError::SpatialConfigIdentityMismatch { .. })
            | Err(SpatialLandscapeError::TransformedWorldDigestMismatch { .. })
    ));
}

#[test]
fn transform_parameters_are_part_of_spatial_run_identity() {
    let first = SpatialLandscapeSimulation::new(config(9006), fixture(), mechanisms()).unwrap();
    let first_identity = first.spatial_binding().config_identity.clone();
    let first_world = first.world().digest64();

    let mut alternate = mechanisms();
    alternate.transforms[0].target_max = 4_000;
    let second = SpatialLandscapeSimulation::new(config(9006), fixture(), alternate).unwrap();

    assert_ne!(first_identity, second.spatial_binding().config_identity);
    assert_ne!(first_world, second.world().digest64());
}
