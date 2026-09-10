use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, SpatialFieldTransform, SpatialLandscapeSimulation,
    SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TemporaryJourneyIneligibility, TemporaryMobilityConfig, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const ORIGIN: CellId = CellId::new(1);
const MIGRATION_DESTINATION: CellId = CellId::new(2);
const FOCAL_DESTINATION: CellId = CellId::new(3);
const FIRST_M4_BOUNDARY: u64 = 91;
const TARGET_ARRIVAL_DAY: u64 = 100;
const NORMALIZED: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn layer(id: &str, role: LandscapeLayerRole, values: [i32; 3]) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(NORMALIZED),
        evidence_input_id: None,
        values: values.into_iter().map(Some).collect(),
    }
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "model_cell".to_owned(),
            spatial_reference: "LOCAL_CS[av6-001-scheduler-boundary]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                [1_000, 0, 0],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                [0, 1_000, 1_000],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "av6-001-scheduler-boundary",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                NORMALIZED,
                1_000,
                65_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                NORMALIZED,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
    .with_run_realization(SpatialRunRealization::new(71_001, 72_001))
}

fn founders(origin: CellId) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "av6-001-scheduler-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: origin,
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 0,
        }],
    )
}

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn migration(enabled: bool) -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(enabled)
        .with_decision_periods_per_year(4)
        .with_candidate_radius_cells(1);
    config.condition_pressure_threshold_permille = 1_000;
    config.resource_pressure_threshold_permille = 0;
    config.minimum_utility_improvement = 0;
    config.resource_weight = 0;
    config.water_security_weight = 100;
    config.kin_weight = 0;
    config.travel_cost_weight = 0;
    config.max_uncertainty_penalty_permille = 0;
    config.relocation_risk_base_penalty_permille = 0;
    config.relocation_risk_per_cell_permille = 0;
    config.travel_condition_cost_per_cell = 0;
    config
}

fn temporary_mobility(target_day: u64, capacity_per_day: u64) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "av6-001-target",
        FocalRegionSource::Synthetic,
        vec![FOCAL_DESTINATION],
    )
    .expect("focal region");
    let schedule = TemporaryMobilitySchedule::new(
        "av6-001-target-arrival",
        TemporaryTriggerTiming::TargetArrivalDay,
        vec![target_day],
        1,
    )
    .expect("target-arrival schedule");
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::new(
            "av6-001-travel",
            ParameterProvenance::SyntheticValidation,
            capacity_per_day,
            10_000,
        )
        .expect("travel model"),
    )
    .expect("temporary mobility")
}

fn experiment(
    target_day: u64,
    capacity_per_day: u64,
    founder_origin: CellId,
    migration_enabled: bool,
    duration_years: u64,
) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(73_001, duration_years)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders(founder_origin))
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(migration(migration_enabled))
        .with_temporary_mobility(temporary_mobility(target_day, capacity_per_day))
}

fn run_after_m4(capacity_per_day: u64) -> anthrosim_core::SpatialLandscapeRecordedRun {
    SpatialLandscapeSimulation::new(
        experiment(TARGET_ARRIVAL_DAY, capacity_per_day, ORIGIN, true, 1),
        landscape(),
        mechanisms(),
    )
    .expect("controlled spatial simulation")
    .run_recorded()
    .expect("controlled run")
}

fn migration_event(
    run: &anthrosim_core::SpatialLandscapeRecordedRun,
) -> &anthrosim_core::EventRecord {
    run.events()
        .events
        .iter()
        .find(|record| {
            record.day == FIRST_M4_BOUNDARY
                && matches!(
                    record.event,
                    EventKind::HouseholdMigration {
                        household,
                        origin,
                        destination,
                        ..
                    } if household == HouseholdId::new(1)
                        && origin == ORIGIN
                        && destination == MIGRATION_DESTINATION
                )
        })
        .expect("M4 must relocate the controlled household on day 91")
}

fn assert_missed_after_m4(capacity_per_day: u64, expected_required_departure: u64) {
    let run = run_after_m4(capacity_per_day);
    let migration = migration_event(&run);
    assert!(run.events().events.iter().all(|record| {
        !matches!(
            record.event,
            EventKind::TemporaryJourneyDeparted {
                household,
                trigger_day,
                ..
            } if household == HouseholdId::new(1) && trigger_day == TARGET_ARRIVAL_DAY
        )
    }));

    let missed = run
        .events()
        .events
        .iter()
        .find(|record| {
            record.day == TARGET_ARRIVAL_DAY
                && matches!(
                    record.event,
                    EventKind::TemporaryJourneyNotStarted {
                        household,
                        trigger_day,
                        reason: TemporaryJourneyIneligibility::DepartureWindowMissed,
                        ..
                    } if household == HouseholdId::new(1)
                        && trigger_day == TARGET_ARRIVAL_DAY
                )
        })
        .expect("an already-passed post-M4 departure must be reported as a missed window");
    assert!(missed.sequence > migration.sequence);

    assert_eq!(
        TARGET_ARRIVAL_DAY - expected_required_departure,
        if capacity_per_day == 100 { 10 } else { 9 }
    );
}

#[test]
fn post_m4_departure_before_boundary_is_not_executed_retroactively() {
    // ceil(1000/100) = 10 travel days => required departure day 90, before day-91 M4.
    assert_missed_after_m4(100, 90);
}

#[test]
fn post_m4_departure_equal_to_boundary_is_not_executed_after_m4() {
    // This is the exact AV6-001/#686 reproduction: ceil(1000/112) = 9 => departure day 91.
    assert_missed_after_m4(112, 91);
}

#[test]
fn post_m4_future_departure_remains_eligible() {
    // ceil(1000/125) = 8 travel days => departure day 92, which must preserve #197 semantics.
    let run = run_after_m4(125);
    let migration = migration_event(&run);
    let departure = run
        .events()
        .events
        .iter()
        .find(|record| {
            record.day == 92
                && matches!(
                    record.event,
                    EventKind::TemporaryJourneyDeparted {
                        household,
                        residence,
                        destination,
                        departure_day,
                        arrival_day,
                        outbound_travel_days,
                        ..
                    } if household == HouseholdId::new(1)
                        && residence == MIGRATION_DESTINATION
                        && destination == FOCAL_DESTINATION
                        && departure_day == 92
                        && arrival_day == TARGET_ARRIVAL_DAY
                        && outbound_travel_days == 8
                )
        })
        .expect("post-M4 future departure must remain eligible");
    assert!(departure.sequence > migration.sequence);
    assert!(run.events().events.iter().all(|record| {
        !(record.day == FIRST_M4_BOUNDARY
            && matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    }));
}

#[test]
fn fresh_simulation_still_executes_a_legitimate_day_zero_departure() {
    let run = SpatialLandscapeSimulation::new(
        experiment(9, 112, MIGRATION_DESTINATION, false, 1),
        landscape(),
        mechanisms(),
    )
    .expect("day-zero controlled simulation")
    .run_recorded()
    .expect("day-zero controlled run");

    assert!(run.events().events.iter().any(|record| {
        record.day == 0
            && matches!(
                record.event,
                EventKind::TemporaryJourneyDeparted {
                    household,
                    residence,
                    departure_day,
                    arrival_day,
                    outbound_travel_days,
                    ..
                } if household == HouseholdId::new(1)
                    && residence == MIGRATION_DESTINATION
                    && departure_day == 0
                    && arrival_day == 9
                    && outbound_travel_days == 9
            )
    }));
}

#[test]
fn annual_checkpoint_resume_preserves_next_day_target_arrival_departure() {
    // From Cell 2, target day 375 with nine travel days means departure day 366. The year-1
    // checkpoint is day 365, whose M9 fixed-day phase is already complete; resume must begin its
    // between-boundary scan at day 366 without re-entering day 365 or skipping the future journey.
    let config = experiment(375, 112, MIGRATION_DESTINATION, false, 2);
    let uninterrupted = SpatialLandscapeSimulation::new(config.clone(), landscape(), mechanisms())
        .expect("uninterrupted spatial simulation")
        .run_recorded()
        .expect("uninterrupted run");

    let checkpoint = SpatialLandscapeSimulation::new(config, landscape(), mechanisms())
        .expect("checkpoint spatial simulation")
        .checkpoint_at_year(1)
        .expect("year-1 checkpoint");
    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, landscape())
        .expect("resume spatial simulation")
        .run_recorded()
        .expect("resumed run");

    assert_eq!(resumed.events(), uninterrupted.events());
    assert!(resumed.events().events.iter().any(|record| {
        record.day == 366
            && matches!(
                record.event,
                EventKind::TemporaryJourneyDeparted {
                    departure_day: 366,
                    arrival_day: 375,
                    ..
                }
            )
    }));
}
