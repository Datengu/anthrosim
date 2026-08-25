use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig, ResourceConfig,
    ResumeLineage, SpatialFieldTransform, SpatialLandscapeError, SpatialLandscapeSimulation,
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

fn directional_config(seed: u64, water_weight: u16, travel_weight: u16) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 100;
    resources.annual_regeneration_units_per_productivity = 0;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let mut migration = MigrationConfig::synthetic_validation_v1();
    migration.enabled = true;
    migration.candidate_radius_cells = 2;
    migration.condition_pressure_threshold_permille = 1_000;
    migration.resource_pressure_threshold_permille = 1_000;
    migration.minimum_utility_improvement = 0;
    migration.resource_weight = 0;
    migration.water_security_weight = water_weight;
    migration.kin_weight = 0;
    migration.travel_cost_weight = travel_weight;
    migration.max_uncertainty_penalty_permille = 0;
    migration.relocation_risk_base_penalty_permille = 0;
    migration.relocation_risk_per_cell_permille = 0;
    migration.travel_condition_cost_per_cell = 0;
    migration.max_recorded_decision_traces = 1_000;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(60).with_max_person_records(10_000))
        .with_resources(resources)
        .with_migration(migration)
}

fn directional_mechanisms(
    movement_min: u16,
    movement_max: u16,
    water_min: u16,
    water_max: u16,
) -> SpatialMechanismConfig {
    let mut result = mechanisms();
    result.transforms[0].target_min = movement_min;
    result.transforms[0].target_max = movement_max;
    result.transforms[1].target_min = water_min;
    result.transforms[1].target_max = water_max;
    result.transforms[2].target_min = 0;
    result.transforms[2].target_max = 0;
    result
}

#[test]
fn transformed_simulation_rejects_duration_beyond_signed_chronology_domain() {
    let duration_years = anthrosim_core::time::MAX_SUPPORTED_DURATION_YEARS + 1;
    let config = ExperimentConfig::new(8_000, duration_years)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(60).with_max_person_records(10_000));
    assert!(matches!(
        SpatialLandscapeSimulation::new(config, fixture(), mechanisms()),
        Err(SpatialLandscapeError::DurationOutOfRange {
            duration_years: found,
            maximum_years,
        }) if found == duration_years
            && maximum_years == anthrosim_core::time::MAX_SUPPORTED_DURATION_YEARS
    ));
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
fn transformed_water_access_changes_migration_utility_in_declared_direction() {
    let mut traces = Vec::new();
    for seed in 9_100..9_120 {
        let run = SpatialLandscapeSimulation::new(
            directional_config(seed, 1, 0),
            fixture(),
            directional_mechanisms(1_000, 1_000, 0, 1_000),
        )
        .unwrap()
        .run_recorded()
        .unwrap();
        traces.extend(
            run.core_manifest()
                .migration
                .recorded_decision_traces
                .clone(),
        );
        if !traces.is_empty() {
            break;
        }
    }

    assert!(
        !traces.is_empty(),
        "controlled fixture should produce water-seeking moves"
    );
    assert!(traces.iter().all(|trace| {
        trace.destination_utility.water_security_score_permille
            > trace.origin_utility.water_security_score_permille
    }));
}

#[test]
fn transformed_movement_cost_changes_migration_utility_in_declared_direction() {
    let mut traces = Vec::new();
    for seed in 9_200..9_220 {
        let run = SpatialLandscapeSimulation::new(
            directional_config(seed, 1, 1),
            fixture(),
            directional_mechanisms(1_000, 3_000, 1_000, 1_000),
        )
        .unwrap()
        .run_recorded()
        .unwrap();
        traces.extend(
            run.core_manifest()
                .migration
                .recorded_decision_traces
                .clone(),
        );
        if !traces.is_empty() {
            break;
        }
    }

    assert!(
        !traces.is_empty(),
        "controlled fixture should produce lower-cost moves"
    );
    assert!(traces.iter().all(|trace| {
        trace.destination_utility.travel_penalty_permille
            < trace.origin_utility.travel_penalty_permille
    }));
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
    let source_day = checkpoint.core_checkpoint.time.days();
    let source_state_digest64 = checkpoint.core_checkpoint.state_digest64;
    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, fixture())
        .unwrap()
        .run_recorded()
        .unwrap();

    validate_spatial_landscape_recorded_run(&resumed, &fixture()).unwrap();

    let mut resumed_without_lineage = resumed.clone();
    resumed_without_lineage
        .manifest
        .core_manifest
        .resume_lineage = ResumeLineage::new();
    resumed_without_lineage
        .checkpoint
        .core_checkpoint
        .resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_without_lineage, uninterrupted);

    assert_eq!(
        resumed.manifest.core_manifest.resume_lineage,
        resumed.checkpoint.core_checkpoint.resume_lineage
    );
    let boundaries = &resumed.manifest.core_manifest.resume_lineage.boundaries;
    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];
    assert_eq!(boundary.source, boundary.continuation);
    assert_eq!(boundary.boundary_day, source_day);
    assert_eq!(boundary.boundary_completed_years, 2);
    assert_eq!(boundary.source_state_digest64, source_state_digest64);
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
