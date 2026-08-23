use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MetricProvenance, MigrationConfig, NoDataPolicy, PopulationConfig,
    ResourceConfig, SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
    SpatialObservabilityError, SpatialTargetField, TransformDirection, WorldConfig,
    derive_spatial_observability,
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
        "generic_observability_fixture_v1",
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
    ExperimentConfig::new(seed, 3)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(30).with_max_person_records(1_000))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn spatial_observability_is_deterministic_and_reconciles_terminal_state() {
    let source = landscape();
    let simulation =
        SpatialLandscapeSimulation::new(config(9501), source.clone(), mechanisms()).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let run = simulation.run_recorded().unwrap();

    let first = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .unwrap();
    let second = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.provenance, MetricProvenance::Derived);
    assert_eq!(first.cells.len(), 3);
    assert_eq!(first.normalized_layers.len(), 3);
    assert_eq!(first.source.landscape_identity, source.identity());
    assert_eq!(
        first.source.spatial_config_identity.as_deref(),
        Some(run.checkpoint.spatial.config_identity.as_str())
    );
    assert_eq!(
        first.summary.terminal_living_population,
        run.core_checkpoint().population.summary().living_population
    );
    assert_eq!(
        first.summary.terminal_occupied_cells,
        run.core_checkpoint()
            .population
            .summary()
            .living_occupied_cell_count
    );
    assert_eq!(
        first.summary.migration_moves,
        run.core_checkpoint().migration.moves_completed
    );
    assert!(
        first
            .unavailable_observables
            .iter()
            .any(|value| value.contains("historical per-cell food stock"))
    );
}

#[test]
fn report_keeps_normalized_inputs_distinct_from_model_facing_fields() {
    let source = landscape();
    let simulation =
        SpatialLandscapeSimulation::new(config(9502), source.clone(), mechanisms()).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let run = simulation.run_recorded().unwrap();
    let report = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .unwrap();

    assert_eq!(source.layers[0].values[1], Some(500));
    assert_eq!(report.cells[1].model_facing.movement_cost, 2_000);
    assert_eq!(source.layers[1].values[1], Some(500));
    assert_eq!(report.cells[1].model_facing.water_access, 500);
    assert_eq!(source.layers[2].values[1], Some(500));
    assert_eq!(report.cells[1].model_facing.base_productivity, 500);
    assert_eq!(report.cells[1].derived.provenance, MetricProvenance::Derived);
}

#[test]
fn report_rejects_checkpoint_from_another_world() {
    let source = landscape();
    let simulation =
        SpatialLandscapeSimulation::new(config(9503), source.clone(), mechanisms()).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let run = simulation.run_recorded().unwrap();
    let mut checkpoint = run.core_checkpoint().clone();
    checkpoint.world_digest64 ^= 1;

    assert!(matches!(
        derive_spatial_observability(
            &source,
            &world,
            &initial_population,
            &checkpoint,
            Some(&run.checkpoint.spatial),
        ),
        Err(SpatialObservabilityError::WorldDigestMismatch { .. })
    ));
}
