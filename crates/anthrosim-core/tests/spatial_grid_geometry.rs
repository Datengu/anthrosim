use anthrosim_core::{
    ExperimentConfig, FocalRegion, FocalRegionSource, GridGeometry, LandscapeBundle,
    LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy,
    PopulationConfig, ResourceConfig, SpatialFieldTransform, SpatialLandscapeError,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, TransformDirection, WorldConfig, ids::CellId,
};

fn layer(id: &str, role: LandscapeLayerRole) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values: vec![Some(500), Some(500)],
    }
}

fn landscape(cell_size_x: u64, cell_size_y: u64) -> LandscapeBundle {
    LandscapeBundle::new(
        2,
        1,
        GridGeometry {
            origin_x: 1_000,
            origin_y: 2_000,
            cell_size_x,
            cell_size_y,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[grid-geometry-test]".to_owned(),
        },
        vec![
            layer("terrain", LandscapeLayerRole::TerrainTraversal),
            layer("water", LandscapeLayerRole::WaterAccessibility),
            layer("resources", LandscapeLayerRole::ResourceOpportunity),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "grid-geometry-test-v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                2_000,
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
    )
}

fn base_config() -> ExperimentConfig {
    ExperimentConfig::new(18_500, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(8).with_max_person_records(100))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn rectangular_grid_allows_nonmovement_spatial_execution() {
    SpatialLandscapeSimulation::new(base_config(), landscape(10, 20), mechanisms())
        .expect("resource-only spatial execution does not interpret cardinal steps as distance");
}

#[test]
fn rectangular_grid_rejects_active_permanent_migration() {
    let config =
        base_config().with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(true));
    assert!(matches!(
        SpatialLandscapeSimulation::new(config, landscape(10, 20), mechanisms()),
        Err(SpatialLandscapeError::RectangularMovementGrid {
            cell_size_x: 10,
            cell_size_y: 20,
            ..
        })
    ));
}

#[test]
fn rectangular_grid_rejects_temporary_travel() {
    let region = FocalRegion::new(
        "grid-geometry-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .expect("region");
    let schedule = TemporaryMobilitySchedule::new(
        "grid-geometry-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![0],
        1,
    )
    .expect("schedule");
    let temporary = TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility");
    let config = base_config().with_temporary_mobility(temporary);

    assert!(matches!(
        SpatialLandscapeSimulation::new(config, landscape(10, 20), mechanisms()),
        Err(SpatialLandscapeError::RectangularMovementGrid {
            cell_size_x: 10,
            cell_size_y: 20,
            ..
        })
    ));
}

#[test]
fn square_grid_preserves_active_grid_step_movement() {
    let config =
        base_config().with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(true));
    SpatialLandscapeSimulation::new(config, landscape(10, 10), mechanisms())
        .expect("square cells preserve existing grid-step movement semantics");
}
