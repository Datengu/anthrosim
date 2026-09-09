use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, SpatialFieldTransform, SpatialLandscapeSimulation,
    SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection,
    WorldConfig, derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId, PersonId},
};

const LEFT_DESTINATION: CellId = CellId::new(1);
const RESIDENCE: CellId = CellId::new(2);
const RIGHT_DESTINATION: CellId = CellId::new(3);
const PADDING: CellId = CellId::new(4);
const VISIT_DAYS: u64 = 5;
const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<i32>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values: values.into_iter().map(Some).collect(),
    }
}

fn landscape(padded: bool) -> LandscapeBundle {
    let width = if padded { 4 } else { 3 };
    let mut terrain = vec![0, 0, 0];
    let mut resources = vec![1_000, 1_000, 1_000];
    if padded {
        // The extra cell is outside the focal region and transforms to movement cost 6,000,
        // above the M9 traversal ceiling of 5,000. It therefore cannot enter any local route.
        terrain.push(1_000);
        resources.push(0);
    }
    LandscapeBundle::new(
        width,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "model_cell".to_owned(),
            spatial_reference: "LOCAL_CS[audit-v6-area-n-padding-resource]".to_owned(),
        },
        vec![
            layer("terrain", LandscapeLayerRole::TerrainTraversal, terrain),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                resources,
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "audit-v6-area-n-padding-resource",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                DOMAIN,
                1_000,
                6_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                DOMAIN,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
    .with_run_realization(SpatialRunRealization::new(714_001, 714_002))
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-n-padding-resource",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: RESIDENCE,
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
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

fn temporary_mobility() -> TemporaryMobilityConfig {
    TemporaryMobilityConfig::new(
        FocalRegion::new(
            "audit-v6-area-n-local-tie",
            FocalRegionSource::Synthetic,
            vec![LEFT_DESTINATION, RIGHT_DESTINATION],
        )
        .expect("focal region"),
        TemporaryMobilitySchedule::new(
            "audit-v6-area-n-one-visit",
            TemporaryTriggerTiming::DepartureDay,
            vec![20],
            u32::try_from(VISIT_DAYS).unwrap(),
        )
        .expect("temporary schedule"),
        TemporaryTravelModel::new(
            "audit-v6-area-n-padding-resource-travel",
            ParameterProvenance::SyntheticValidation,
            3_000,
            5_000,
        )
        .expect("travel model"),
    )
    .expect("temporary mobility")
}

fn experiment(process_seed: u64, width: u32) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 365;
    resources.annual_regeneration_units_per_productivity = 0;
    resources.seasonality_scale_permille = 0;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 0;
    resources.max_condition_mortality_probability_per_million = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(process_seed, 1)
        .with_world(WorldConfig::new(width, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

#[derive(Debug)]
struct ArmResult {
    destination: CellId,
    visitor_person_days: [u64; 3],
    food_stock: [u64; 3],
}

fn run_arm(process_seed: u64, padded: bool) -> ArmResult {
    let source = landscape(padded);
    let simulation = SpatialLandscapeSimulation::new(
        experiment(process_seed, source.width),
        source,
        mechanisms(),
    )
    .expect("controlled spatial simulation");
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();

    for cell in [LEFT_DESTINATION, RESIDENCE, RIGHT_DESTINATION] {
        let model_facing = world.cell(cell).expect("unchanged local cell");
        assert_eq!(model_facing.movement_cost, 1_000);
        assert_eq!(model_facing.base_productivity, 1_000);
    }
    if padded {
        let model = TemporaryTravelModel::new(
            "audit-v6-area-n-padding-resource-travel",
            ParameterProvenance::SyntheticValidation,
            3_000,
            5_000,
        )
        .unwrap();
        assert_eq!(world.cell(PADDING).unwrap().movement_cost, 6_000);
        assert!(!model.is_traversable(&world, PADDING));
    }

    let run = simulation.run_recorded().expect("controlled run");
    let destination = run
        .events()
        .events
        .iter()
        .find_map(|record| match record.event {
            EventKind::TemporaryJourneyDeparted {
                household,
                residence,
                destination,
                ..
            } if household == HouseholdId::new(1) && residence == RESIDENCE => Some(destination),
            _ => None,
        })
        .expect("controlled household must depart once");

    let temporary = derive_temporary_mobility_observability(
        &world,
        &initial_population,
        run.core_checkpoint(),
    )
    .expect("temporary observability");
    assert_eq!(temporary.summary.journeys_started, 1);
    assert_eq!(temporary.summary.journeys_completed, 1);
    assert_eq!(temporary.summary.visitor_person_days, VISIT_DAYS);

    let visitor_person_days = [LEFT_DESTINATION, RESIDENCE, RIGHT_DESTINATION].map(|cell| {
        temporary
            .cells
            .iter()
            .find(|row| row.cell == cell)
            .expect("local cell observability")
            .visitor_person_days
    });
    let food_stock = [LEFT_DESTINATION, RESIDENCE, RIGHT_DESTINATION].map(|cell| {
        run.core_checkpoint()
            .resources
            .cell_food_stock(cell)
            .expect("local resource stock")
    });

    ArmResult {
        destination,
        visitor_person_days,
        food_stock,
    }
}

#[test]
fn impassable_padding_m9_locality_failure_propagates_into_aggregation_and_resources() {
    // Area E / AV6-006 established that an unreachable impassable padding cell can relabel the
    // local equal-cost M9 ambiguity. Area N composes that defect with actual M9 presence and M3
    // duration-aware resource accounting rather than stopping at destination identity.
    let mut demonstrated = None;
    for process_seed in 0..128_u64 {
        let baseline = run_arm(process_seed, false);
        let padded = run_arm(process_seed, true);
        if baseline.destination != padded.destination {
            demonstrated = Some((process_seed, baseline, padded));
            break;
        }
    }

    let (process_seed, baseline, padded) =
        demonstrated.expect("AV6-006 locality failure must reproduce in the full integrated host");
    assert!([
        LEFT_DESTINATION,
        RIGHT_DESTINATION,
    ]
    .contains(&baseline.destination));
    assert!([
        LEFT_DESTINATION,
        RIGHT_DESTINATION,
    ]
    .contains(&padded.destination));

    let baseline_destination_index = usize::try_from(baseline.destination.0 - 1).unwrap();
    let padded_destination_index = usize::try_from(padded.destination.0 - 1).unwrap();
    assert_eq!(baseline.visitor_person_days[baseline_destination_index], VISIT_DAYS);
    assert_eq!(padded.visitor_person_days[padded_destination_index], VISIT_DAYS);
    assert_eq!(baseline.visitor_person_days[padded_destination_index], 0);
    assert_eq!(padded.visitor_person_days[baseline_destination_index], 0);
    assert_eq!(baseline.visitor_person_days[1], 0);
    assert_eq!(padded.visitor_person_days[1], 0);

    // The one-person fixture has one unit of need per model day. Five visiting days therefore
    // remove exactly five units from whichever unchanged local destination was selected.
    assert_eq!(baseline.food_stock[1], padded.food_stock[1]);
    assert_eq!(
        padded.food_stock[baseline_destination_index]
            .checked_sub(baseline.food_stock[baseline_destination_index]),
        Some(VISIT_DAYS),
    );
    assert_eq!(
        baseline.food_stock[padded_destination_index]
            .checked_sub(padded.food_stock[padded_destination_index]),
        Some(VISIT_DAYS),
    );

    eprintln!(
        "process_seed={process_seed} baseline_destination={:?} padded_destination={:?} baseline_visitor_person_days={:?} padded_visitor_person_days={:?} baseline_food_stock={:?} padded_food_stock={:?}",
        baseline.destination,
        padded.destination,
        baseline.visitor_person_days,
        padded.visitor_person_days,
        baseline.food_stock,
        padded.food_stock,
    );
}
