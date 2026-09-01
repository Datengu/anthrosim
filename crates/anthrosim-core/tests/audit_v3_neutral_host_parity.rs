use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, NoDataPolicy, PopulationConfig, Simulation, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField, TransformDirection,
    WorldConfig,
};

fn layer(
    id: &str,
    role: LandscapeLayerRole,
    domain: LandscapeValueDomain,
    values: Vec<Option<i32>>,
) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "identity_units".to_owned(),
        value_domain: Some(domain),
        evidence_input_id: None,
        values,
    }
}

fn identity_landscape(world: &anthrosim_core::world::World) -> LandscapeBundle {
    let movement_domain = LandscapeValueDomain {
        min: 1_000,
        max: 3_500,
    };
    let permille_domain = LandscapeValueDomain { min: 0, max: 1_000 };

    LandscapeBundle::new(
        world.width(),
        world.height(),
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 100,
            cell_size_y: 100,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![
            layer(
                "movement",
                LandscapeLayerRole::TerrainTraversal,
                movement_domain,
                world
                    .cells()
                    .iter()
                    .map(|cell| Some(i32::from(cell.movement_cost)))
                    .collect(),
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                permille_domain,
                world
                    .cells()
                    .iter()
                    .map(|cell| Some(i32::from(cell.water_access)))
                    .collect(),
            ),
            layer(
                "productivity",
                LandscapeLayerRole::ResourceOpportunity,
                permille_domain,
                world
                    .cells()
                    .iter()
                    .map(|cell| Some(i32::from(cell.base_productivity)))
                    .collect(),
            ),
        ],
    )
}

fn identity_mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "audit-v3-causally-neutral-identity-overlay",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "movement",
                "identity_units",
                LandscapeValueDomain {
                    min: 1_000,
                    max: 3_500,
                },
                1_000,
                3_500,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "identity_units",
                LandscapeValueDomain { min: 0, max: 1_000 },
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "productivity",
                "identity_units",
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
        .with_world(WorldConfig::new(8, 6))
        .with_population(PopulationConfig::new(120).with_max_person_records(20_000))
}

#[test]
fn causally_neutral_spatial_host_matches_core_scheduler_exactly() {
    for seed in 73_100..73_110 {
        let config = config(seed);
        let core = Simulation::new(config.clone()).expect("core simulation must initialize");
        let expected_world_digest = core.world().digest64();
        let landscape = identity_landscape(core.world());
        let core_run = core.run_recorded().expect("core run must complete");

        let spatial = SpatialLandscapeSimulation::new(config, landscape, identity_mechanisms())
            .expect("neutral spatial simulation must initialize");
        assert_eq!(
            spatial.world().digest64(),
            expected_world_digest,
            "identity overlay changed the authoritative world for seed {seed}"
        );
        let spatial_run = spatial
            .run_recorded()
            .expect("neutral spatial run must complete");

        assert_eq!(
            spatial_run.core_manifest(),
            &core_run.manifest,
            "core manifest diverged solely because execution used the spatial host for seed {seed}"
        );
        assert_eq!(
            spatial_run.core_checkpoint(),
            &core_run.checkpoint,
            "core checkpoint diverged solely because execution used the spatial host for seed {seed}"
        );
    }
}
