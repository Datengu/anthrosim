use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig,
    PopulationInitialization, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, TransformDirection, WorldConfig,
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
            // ORIGIN becomes movement cost 65_000 and is deliberately above the M9 traversal
            // ceiling. Cells 2/3 become the base movement cost 1_000.
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                [1_000, 0, 0],
            ),
            // Force M4 to prefer Cell 2 over the origin. With radius 1, Cell 2 is the only
            // alternative available from Cell 1 at the first decision boundary.
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
            // Keep permanent-migration pressure positive even after the day-91 M3 boundary.
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

fn temporary_mobility() -> TemporaryMobilityConfig {
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

    // Cell 2 -> Cell 3 costs exactly 1_000 units. A capacity of 112 units/day therefore
    // produces ceil(1000/112) = 9 outbound days, making day 91 the required departure day.
    // Cell 1 is above the 10_000 traversal ceiling and is initially unreachable.
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::new(
            "audit-v6-area-a-nine-day-travel",
            ParameterProvenance::SyntheticValidation,
            112,
            10_000,
        )
        .expect("travel model"),
    )
    .expect("temporary mobility")
}

fn experiment() -> ExperimentConfig {
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
        .with_temporary_mobility(temporary_mobility())
}

#[test]
fn target_arrival_reconsideration_cannot_run_m9_after_same_day_m4_boundary() {
    // Fresh Audit-v6 Area-A scheduler adversary.
    //
    // The declared fixed-day order is M3 -> M9 -> M4. Before day 91 the household's origin is
    // unreachable under M9, so the target-arrival trigger has no due day-91 departure. M4 then
    // deterministically relocates the household to Cell 2 on day 91. At Cell 2 the same predeclared
    // target arrival day 100 implies a newly computed departure day of exactly 91.
    //
    // If the event-driven outer loop immediately re-enters M9 at current_day=91 after M4 has
    // already executed, the authoritative event stream will contain M4 migration followed by M9
    // departure on the same day. That is the ordering inversion under attack.
    let run = SpatialLandscapeSimulation::new(experiment(), landscape(), mechanisms())
        .expect("controlled spatial simulation")
        .run_recorded()
        .expect("controlled run");

    let migration = run
        .events()
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
        .expect("M4 must relocate the controlled household from Cell 1 to Cell 2 on day 91");

    let departure = run
        .events()
        .events
        .iter()
        .find(|record| {
            record.day == FIRST_M4_BOUNDARY
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
                        && departure_day == FIRST_M4_BOUNDARY
                        && arrival_day == TARGET_ARRIVAL_DAY
                        && outbound_travel_days == 9
                )
        })
        .expect("post-M4 residence must make the target-arrival journey due on day 91");

    assert!(
        departure.sequence < migration.sequence,
        "same-day scheduler inversion: M4 migration sequence {} ran before newly due M9 departure sequence {} on day {}; declared fixed-day order requires M9 before M4",
        migration.sequence,
        departure.sequence,
        FIRST_M4_BOUNDARY,
    );
}
