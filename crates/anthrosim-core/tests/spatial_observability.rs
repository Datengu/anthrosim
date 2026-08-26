use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, GridGeometry,
    LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MetricProvenance,
    MigrationConfig, NoDataPolicy, PopulationConfig, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialLocationAttribution, SpatialMechanismConfig,
    SpatialObservabilityError, SpatialTargetField, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection,
    WorldConfig, derive_spatial_observability, derive_temporary_mobility_observability,
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

fn visiting_mortality_config(seed: u64) -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    let temporary = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "generic-observability-m9-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(3)],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "generic-observability-m9-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![100],
            1_000,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(30).with_max_person_records(1_000))
        .with_demography(demography)
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary)
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
    assert_eq!(first.schema_version, 3);
    assert_eq!(first.provenance, MetricProvenance::Derived);
    assert_eq!(
        first.semantics.population_location_basis,
        SpatialLocationAttribution::PersistentResidence
    );
    assert!(!first.semantics.occupancy_includes_temporary_visitors);
    assert!(!first.semantics.occupancy_includes_transit);
    assert_eq!(
        first.semantics.birth_cell_attribution,
        SpatialLocationAttribution::PersistentResidence
    );
    assert_eq!(
        first.semantics.death_cell_attribution,
        SpatialLocationAttribution::PersistentResidence
    );
    assert_eq!(first.semantics.physical_presence_companion_artifact, None);
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
    assert!(
        first
            .unavailable_observables
            .iter()
            .any(|value| value.contains("persistent residence"))
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
    assert_eq!(
        report.cells[1].derived.provenance,
        MetricProvenance::Derived
    );
}

#[test]
fn m9_visitors_and_deaths_cannot_be_mistaken_for_residence_occupancy() {
    let source = landscape();
    let simulation = SpatialLandscapeSimulation::new(
        visiting_mortality_config(9504),
        source.clone(),
        mechanisms(),
    )
    .unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let run = simulation.run_recorded().unwrap();

    let (household, residence, destination) = run
        .core_checkpoint()
        .events
        .events
        .iter()
        .find_map(|record| match &record.event {
            EventKind::TemporaryJourneyDeparted {
                household,
                residence,
                destination,
                ..
            } => Some((*household, *residence, *destination)),
            _ => None,
        })
        .expect("fixture must contain a temporary departure");
    let death_cell = run
        .core_checkpoint()
        .events
        .events
        .iter()
        .find_map(|record| match &record.event {
            EventKind::Death {
                household: death_household,
                cell,
                ..
            } if *death_household == household => Some(*cell),
            _ => None,
        })
        .expect("visiting household must contain an annual mortality event");

    assert_ne!(residence, destination);
    assert_eq!(death_cell, residence);

    let spatial = derive_spatial_observability(
        &source,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .unwrap();
    let temporary =
        derive_temporary_mobility_observability(&world, &initial_population, run.core_checkpoint())
            .unwrap();

    assert_eq!(
        spatial.semantics.population_location_basis,
        SpatialLocationAttribution::PersistentResidence
    );
    assert_eq!(
        spatial.semantics.death_cell_attribution,
        SpatialLocationAttribution::PersistentResidence
    );
    assert_eq!(
        spatial
            .semantics
            .physical_presence_companion_artifact
            .as_deref(),
        Some("temporary-observability.json")
    );
    let residence_row = spatial
        .cells
        .iter()
        .find(|row| row.cell == residence)
        .unwrap();
    assert!(residence_row.derived.deaths > 0);
    let destination_presence = temporary
        .cells
        .iter()
        .find(|row| row.cell == destination)
        .unwrap();
    assert!(destination_presence.visitor_person_days > 0);
    assert!(temporary.summary.visitor_person_days > 0);
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
