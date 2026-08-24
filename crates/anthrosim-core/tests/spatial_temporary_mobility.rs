use anthrosim_core::ids::{CellId, HouseholdId};
use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, GridGeometry,
    LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig,
    NoDataPolicy, PopulationConfig, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField,
    TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryTravelResolution,
    TemporaryTravelTable, TemporaryTriggerTiming, TransformDirection, WorldConfig,
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
        4,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "cell".to_owned(),
            spatial_reference: "LOCAL_CS[generic]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(250), Some(500), Some(750)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(750), Some(500), Some(250)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(500), Some(500), Some(500), Some(500)],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "m9_6_spatial_host_fixture_v1",
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

fn config() -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    ExperimentConfig::new(96_001, 2)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(24).with_target_household_size(4))
        .with_demography(demography)
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn program() -> TemporaryMobilityProgram {
    let source = landscape();
    let mechanisms = mechanisms();
    let baseline = SpatialLandscapeSimulation::new(config(), source, mechanisms)
        .expect("baseline spatial host");
    let household = HouseholdId::new(1);
    let residence = baseline
        .population()
        .household_location(household)
        .expect("household residence");
    let destination = (1..=baseline.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| *cell != residence)
        .expect("world has another cell");
    let region = FocalRegion::new(
        "generic-m9-6-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .expect("region");
    let resolutions = (1..=baseline.world().cell_count() as u64)
        .map(|_| TemporaryTravelResolution::Reachable {
            destination,
            outbound_travel_days: 10,
            return_travel_days: 10,
        })
        .collect();
    let travel = TemporaryTravelTable::new(resolutions, &region, baseline.world()).expect("travel");
    let schedule = TemporaryMobilitySchedule::new(
        "annual-boundary-active-journey",
        TemporaryTriggerTiming::DepartureDay,
        vec![360],
        5,
    )
    .expect("schedule");
    TemporaryMobilityProgram::new(region, schedule, travel, baseline.world()).expect("program")
}

#[test]
fn transformed_spatial_host_executes_and_resumes_active_temporary_journeys_exactly() {
    let source = landscape();
    let mechanisms = mechanisms();
    let program = program();

    let uninterrupted = SpatialLandscapeSimulation::new_with_temporary_mobility(
        config(),
        source.clone(),
        mechanisms.clone(),
        program.clone(),
    )
    .expect("temporary spatial host")
    .run_recorded()
    .expect("uninterrupted run");

    assert!(
        uninterrupted
            .events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    );

    let paused = SpatialLandscapeSimulation::new_with_temporary_mobility(
        config(),
        source.clone(),
        mechanisms,
        program,
    )
    .expect("temporary spatial host")
    .checkpoint_at_year(1)
    .expect("annual checkpoint");
    assert!(
        paused
            .core_checkpoint
            .temporary_mobility
            .active_journey(HouseholdId::new(1))
            .is_some(),
        "household 1 should be in outbound transit across the annual checkpoint"
    );

    let resumed = SpatialLandscapeSimulation::from_checkpoint(paused, source)
        .expect("resume active spatial journey")
        .run_recorded()
        .expect("resumed run");

    let expected = uninterrupted.core_checkpoint();
    let actual = resumed.core_checkpoint();
    assert_eq!(actual.state_digest64, expected.state_digest64);
    assert_eq!(actual.population, expected.population);
    assert_eq!(actual.temporary_mobility, expected.temporary_mobility);
    assert_eq!(actual.resources, expected.resources);
    assert_eq!(actual.migration, expected.migration);
    assert_eq!(actual.rng, expected.rng);
    assert_eq!(actual.events, expected.events);
    assert_eq!(actual.metrics, expected.metrics);
}
