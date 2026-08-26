use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeSimulation, LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig,
    ResourceConfig, ResourceSystem, SpatialFieldTransform, SpatialLandscapeSimulation,
    SpatialMechanismConfig, SpatialObservabilityReport, SpatialTargetField, TransformDirection,
    World, WorldConfig, derive_spatial_observability,
    ids::CellId,
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

fn landscape() -> LandscapeBundle {
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
                vec![Some(250), Some(500), Some(750)],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "generic_initial_resource_observability_fixture_v1",
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

fn config(seed: u64, resources: ResourceConfig) -> ExperimentConfig {
    ExperimentConfig::new(seed, 0)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(30).with_max_person_records(1_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn control_report(
    seed: u64,
    resources: ResourceConfig,
) -> (World, ResourceSystem, SpatialObservabilityReport) {
    let source = landscape();
    let simulation = LandscapeSimulation::new(config(seed, resources.clone()), source.clone())
        .expect("landscape simulation");
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let expected = ResourceSystem::initialize(&world, &resources).expect("M3 initial resources");
    let run = simulation.run_recorded().expect("recorded run");
    let report = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        None,
    )
    .expect("spatial observability");
    (world, expected, report)
}

fn transformed_report(
    seed: u64,
    resources: ResourceConfig,
) -> (World, ResourceSystem, SpatialObservabilityReport) {
    let source = landscape();
    let simulation = SpatialLandscapeSimulation::new(
        config(seed, resources.clone()),
        source.clone(),
        mechanisms(),
    )
    .expect("spatial landscape simulation");
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let expected = ResourceSystem::initialize(&world, &resources).expect("M3 initial resources");
    let run = simulation.run_recorded().expect("recorded run");
    let report = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .expect("spatial observability");
    (world, expected, report)
}

fn assert_report_matches_m3_initialization(
    world: &World,
    expected: &ResourceSystem,
    report: &SpatialObservabilityReport,
) {
    assert_eq!(report.cells.len(), world.cell_count());
    for row in &report.cells {
        assert_eq!(
            row.model_facing.initial_food_stock,
            expected
                .cell_food_stock(row.cell)
                .expect("M3 stock for report cell"),
            "cell {:?}",
            row.cell
        );
    }
}

#[test]
fn initial_food_stock_applies_m3_productivity_scaling() {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(0)
        .with_productivity_scale_permille(500);
    let (world, expected, report) = control_report(22_401, resources);

    assert_report_matches_m3_initialization(&world, &expected, &report);
    assert!(report.cells.iter().any(|row| {
        let raw_world_stock = u64::from(world.cell(row.cell).expect("world cell").food_stock);
        row.model_facing.initial_food_stock != raw_world_stock
    }));
}

#[test]
fn initial_food_stock_applies_m3_capacity_clipping() {
    let mut resources =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.cell_stock_capacity_years = 1;
    let (world, expected, report) = control_report(22_402, resources);

    assert_report_matches_m3_initialization(&world, &expected, &report);
    assert!(report.cells.iter().any(|row| {
        let raw_world_stock = u64::from(world.cell(row.cell).expect("world cell").food_stock);
        row.model_facing.initial_food_stock < raw_world_stock
    }));
}

#[test]
fn initial_food_stock_uses_transformed_m8_productivity_before_m3_initialization() {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(0)
        .with_productivity_scale_permille(500);
    let (world, expected, report) = transformed_report(22_403, resources);

    assert_report_matches_m3_initialization(&world, &expected, &report);
    let middle = report
        .cells
        .iter()
        .find(|row| row.cell == CellId::new(2))
        .expect("middle cell");
    assert_eq!(middle.model_facing.base_productivity, 500);
    assert_ne!(
        middle.model_facing.initial_food_stock,
        u64::from(world.cell(CellId::new(2)).expect("middle world cell").food_stock)
    );
}

#[test]
fn default_m3_initial_stock_can_coincide_with_raw_world_stock_without_conflating_them() {
    let resources =
        ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    let (world, expected, report) = control_report(22_404, resources);

    assert_report_matches_m3_initialization(&world, &expected, &report);
    assert!(report.cells.iter().all(|row| {
        row.model_facing.initial_food_stock
            == u64::from(world.cell(row.cell).expect("world cell").food_stock)
    }));
    assert!(report.cells.iter().all(|row| {
        row.model_facing.provenance == "authoritative_world_m3_initialization_and_checkpoint"
    }));
}
