use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, GridGeometry,
    LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig,
    NoDataPolicy, ParameterProvenance, PopulationConfig, ResourceConfig, Simulation,
    SpatialFieldTransform, SpatialMechanismConfig, SpatialTargetField, TemporaryMobilityProgram,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTravelResolution,
    TemporaryTriggerTiming, TransformDirection, World, WorldConfig, transform_landscape,
};
use anthrosim_core::{
    ids::{CellId, HouseholdId},
    rng::RngFactory,
};

fn no_event_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn terrain_bundle() -> LandscapeBundle {
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
        vec![LandscapeLayer {
            layer_id: "terrain".to_owned(),
            role: LandscapeLayerRole::TerrainTraversal,
            unit: "normalized_index".to_owned(),
            value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
            evidence_input_id: None,
            values: vec![Some(0), Some(1_000), Some(0)],
        }],
    )
}

#[test]
fn m8_movement_transform_changes_m9_4_cost_and_duration_through_public_boundary() {
    let transform = SpatialFieldTransform::new(
        SpatialTargetField::MovementCost,
        "terrain",
        "normalized_index",
        LandscapeValueDomain { min: 0, max: 1_000 },
        1_000,
        5_000,
        TransformDirection::Direct,
        NoDataPolicy::Reject,
    );
    let overlay = transform_landscape(
        &terrain_bundle(),
        &SpatialMechanismConfig::new("m9_4_m8_boundary", vec![transform]),
    )
    .unwrap();
    assert_eq!(overlay.movement_cost, Some(vec![1_000, 5_000, 1_000]));

    let base = World::generate(WorldConfig::new(3, 1), RngFactory::new(44))
        .unwrap()
        .with_model_field_overlay(Some(&[1_000, 1_000, 1_000]), None, None)
        .unwrap();
    let transformed = World::generate(WorldConfig::new(3, 1), RngFactory::new(44))
        .unwrap()
        .with_model_field_overlay(overlay.movement_cost.as_deref(), None, None)
        .unwrap();
    let base_region = FocalRegion::new(
        "target",
        FocalRegionSource::Synthetic,
        vec![CellId::new(3)],
    )
    .unwrap();
    let transformed_region = base_region.clone();
    let model = TemporaryTravelModel::new(
        "m9_4_public_boundary",
        ParameterProvenance::SyntheticValidation,
        2_000,
        u16::MAX,
    )
    .unwrap();

    let base_table = model.derive_table(&base_region, &base).unwrap();
    let transformed_table = model
        .derive_table(&transformed_region, &transformed)
        .unwrap();

    assert_eq!(base_table.accumulated_cost_units(CellId::new(1)), Some(2_000));
    assert_eq!(
        transformed_table.accumulated_cost_units(CellId::new(1)),
        Some(6_000)
    );
    assert!(matches!(
        base_table.resolution(CellId::new(1)),
        Some(TemporaryTravelResolution::Reachable {
            outbound_travel_days: 1,
            return_travel_days: 1,
            ..
        })
    ));
    assert!(matches!(
        transformed_table.resolution(CellId::new(1)),
        Some(TemporaryTravelResolution::Reachable {
            outbound_travel_days: 3,
            return_travel_days: 3,
            ..
        })
    ));
}

#[test]
fn derived_cost_and_model_identity_are_authoritative_in_active_journey_and_departure_event() {
    let seed = 9_104;
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(20).with_target_household_size(5))
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let probe = Simulation::new(config.clone()).unwrap();
    let household = HouseholdId::new(1);
    let residence = probe.population().household_location(household).unwrap();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| *cell != residence)
        .unwrap();
    let region = FocalRegion::new(
        "authoritative-cost-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "authoritative-cost-model",
        ParameterProvenance::SyntheticValidation,
        3_000,
        u16::MAX,
    )
    .unwrap();
    let expected_model_identity = model.identity();
    let travel = model.derive_table(&region, probe.world()).unwrap();
    let expected_cost = travel.accumulated_cost_units(residence).unwrap();
    let program = TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "active-cost-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![360],
            30,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap();

    let checkpoint = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let active = checkpoint
        .temporary_mobility
        .active_journey(household)
        .expect("household 1 should still have an active journey at day 365");
    assert_eq!(
        active.travel_model_identity.as_deref(),
        Some(expected_model_identity.as_str())
    );
    assert_eq!(active.accumulated_travel_cost_units, Some(expected_cost));

    let departure = checkpoint
        .events
        .events
        .iter()
        .find_map(|record| match &record.event {
            EventKind::TemporaryJourneyDeparted {
                household: event_household,
                travel_model_identity,
                accumulated_travel_cost_units,
                ..
            } if *event_household == household => Some((
                travel_model_identity.as_deref(),
                *accumulated_travel_cost_units,
            )),
            _ => None,
        })
        .expect("departure event must be present");
    assert_eq!(departure.0, Some(expected_model_identity.as_str()));
    assert_eq!(departure.1, Some(expected_cost));
    checkpoint.validate_invariants().unwrap();
}
