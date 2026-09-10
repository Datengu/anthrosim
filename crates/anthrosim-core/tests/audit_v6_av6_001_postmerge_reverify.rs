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
            spatial_reference: "LOCAL_CS[audit-v6-area-a-target-arrival-m4]".to_owned(),
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
        "audit-v6-area-a-target-arrival-m4",
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

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-a-target-arrival-m4",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: ORIGIN,
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

fn migration() -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
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

fn temporary_mobility(capacity_per_day: u32) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v6-area-a-target",
        FocalRegionSource::Synthetic,
        vec![FOCAL_DESTINATION],
    )
    .expect("focal region");
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v6-area-a-target-arrival",
        TemporaryTriggerTiming::TargetArrivalDay,
        vec![TARGET_ARRIVAL_DAY],
        1,
    )
    .expect("target-arrival schedule");

    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::new(
            "audit-v6-area-a-nine-day-travel",
            ParameterProvenance::SyntheticValidation,
            capacity_per_day,
            10_000,
        )
        .expect("travel model"),
    )
    .expect("temporary mobility")
}

fn experiment(capacity_per_day: u32) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(73_001, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(migration())
        .with_temporary_mobility(temporary_mobility(capacity_per_day))
}

fn run(capacity_per_day: u32) -> anthrosim_core::SpatialLandscapeRecordedRun {
    SpatialLandscapeSimulation::new(experiment(capacity_per_day), landscape(), mechanisms())
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
        .expect("M4 must preserve the original #686 controlled Cell 1 -> Cell 2 move on day 91")
}

#[test]
fn preserved_686_boundary_contract_no_longer_allows_m4_then_same_day_m9() {
    // Exact #686 scientific fixture: Cell 1 is initially M9-unreachable, M4 moves the household
    // to Cell 2 on day 91, and ceil(1000/112)=9 means target day 100 would require departure on
    // the already-entered day 91 after that move.
    let run = run(112);
    let migration = migration_event(&run);

    assert!(run.events().events.iter().all(|record| {
        !(record.day == FIRST_M4_BOUNDARY
            && matches!(
                record.event,
                EventKind::TemporaryJourneyDeparted {
                    household,
                    trigger_day,
                    ..
                } if household == HouseholdId::new(1) && trigger_day == TARGET_ARRIVAL_DAY
            ))
    }), "AV6-001 substantive oracle failed: the preserved #686 fixture still executed a target-arrival M9 departure on day 91 after the fixed M9 phase had already completed");

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
        .expect("the exact-boundary target-arrival window must be reported explicitly as missed");

    assert!(missed.sequence > migration.sequence);
    println!(
        "AV6-001 postmerge boundary: migration_day={} migration_sequence={} same_day_target_departure_present=false missed_day={} missed_sequence={}",
        migration.day, migration.sequence, missed.day, missed.sequence
    );
}

#[test]
fn neighbouring_future_departure_reconsideration_remains_valid() {
    // Same #686 fixture, changing only travel capacity. ceil(1000/125)=8, therefore the post-M4
    // residence implies a genuinely future departure on day 92 and preserves #197 semantics.
    let run = run(125);
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
                        trigger_day,
                        departure_day,
                        arrival_day,
                        outbound_travel_days,
                        ..
                    } if household == HouseholdId::new(1)
                        && residence == MIGRATION_DESTINATION
                        && destination == FOCAL_DESTINATION
                        && trigger_day == TARGET_ARRIVAL_DAY
                        && departure_day == 92
                        && arrival_day == TARGET_ARRIVAL_DAY
                        && outbound_travel_days == 8
                )
        })
        .expect("#197-style post-M4 future target-arrival reconsideration must remain eligible");

    assert!(departure.sequence > migration.sequence);
    assert!(run.events().events.iter().all(|record| {
        !(record.day == FIRST_M4_BOUNDARY
            && matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    }));
    println!(
        "AV6-001 future control: migration_day={} migration_sequence={} departure_day={} departure_sequence={} arrival_day={}",
        migration.day, migration.sequence, departure.day, departure.sequence, TARGET_ARRIVAL_DAY
    );
}
