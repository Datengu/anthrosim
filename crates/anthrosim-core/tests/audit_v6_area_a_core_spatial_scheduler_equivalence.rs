use anthrosim_core::{
    ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig, ResourceConfig,
    Simulation, SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
    SpatialRunRealization, SpatialTargetField, TransformDirection, World, WorldConfig,
};

const MOVEMENT_DOMAIN: LandscapeValueDomain = LandscapeValueDomain {
    min: 1_000,
    max: 65_000,
};

fn identity_movement_landscape(world: &World) -> LandscapeBundle {
    LandscapeBundle::new(
        world.width(),
        world.height(),
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "model_cell".to_owned(),
            spatial_reference: "LOCAL_CS[audit-v6-area-a-host-equivalence]".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "movement".to_owned(),
            role: LandscapeLayerRole::TerrainTraversal,
            unit: "movement_cost_units".to_owned(),
            value_domain: Some(MOVEMENT_DOMAIN),
            evidence_input_id: None,
            values: world
                .cells()
                .iter()
                .map(|cell| Some(i32::from(cell.movement_cost)))
                .collect(),
        }],
    )
}

fn identity_movement_mechanisms(seed: u64) -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "audit-v6-area-a-host-equivalence",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "movement",
            "movement_cost_units",
            MOVEMENT_DOMAIN,
            1_000,
            65_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    )
    .with_run_realization(SpatialRunRealization::new(seed, seed))
}

fn experiment(seed: u64, resource_periods: u16, migration_periods: u16) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = resource_periods;

    let migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_decision_periods_per_year(migration_periods);

    ExperimentConfig::new(seed, 3)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(32).with_max_person_records(2_000))
        .with_resources(resources)
        .with_migration(migration)
}

#[test]
fn identity_spatial_wrapper_preserves_core_scheduler_trajectory_across_clock_collisions() {
    // Fresh Audit-v6 Area-A adversary: the core and spatial hosts duplicate the authoritative
    // scheduler. If the spatial transform is an exact identity and all environment/population/
    // process seed roles are the same, merely selecting the spatial host must not change M2/M3/M4
    // execution. Exercise both coincident and non-coincident M3/M4 clocks, including the maximum
    // supported daily cadence.
    let cadences = [(1_u16, 1_u16), (4, 4), (12, 4), (4, 12), (365, 365)];

    for seed in 91_000..91_004_u64 {
        for (resource_periods, migration_periods) in cadences {
            let config = experiment(seed, resource_periods, migration_periods);

            let core = Simulation::new(config.clone()).expect("core simulation must initialize");
            let core_world = core.world().clone();
            let landscape = identity_movement_landscape(&core_world);
            let mechanisms = identity_movement_mechanisms(seed);
            let core_run = core.run_recorded().expect("core run must complete");

            let spatial = SpatialLandscapeSimulation::new(config, landscape, mechanisms)
                .expect("identity spatial simulation must initialize");
            assert_eq!(
                spatial.world(),
                &core_world,
                "identity movement transform changed the authoritative world at seed {seed}, resource cadence {resource_periods}, migration cadence {migration_periods}"
            );
            let spatial_run = spatial
                .run_recorded()
                .expect("identity spatial run must complete");

            assert_eq!(
                spatial_run.core_checkpoint(),
                &core_run.checkpoint,
                "core and identity-spatial hosts diverged at seed {seed}, resource cadence {resource_periods}, migration cadence {migration_periods}"
            );
        }
    }
}
