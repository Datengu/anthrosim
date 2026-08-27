use anthrosim_core::ids::{CellId, HouseholdId, PersonId};
use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    DemographyConfig, EvidenceCatalog, EvidenceRecord, EvidenceSource, ExperimentConfig,
    FocalRegion, FocalRegionSource, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig,
    ReproductiveSex, ResourceConfig, SpatialAnalysisDomain, SpatialAnalysisExtent,
    SpatialBoundaryDeclaration, SpatialBoundaryError, SpatialBoundaryInterpretation,
    SpatialExtentAdequacyCriterion, SpatialExtentMetricTolerance, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTravelResolution, TemporaryTriggerTiming, TransformDirection, World, WorldConfig,
    assess_spatial_boundary, bounded_candidate_cells, derive_temporary_mobility_observability,
    transform_landscape,
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

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

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values,
    }
}

fn rectangular_landscape(
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    terrain: impl Fn(i64, i64) -> i32,
    resources: impl Fn(i64, i64) -> i32,
) -> LandscapeBundle {
    let width = u32::try_from(max_x - min_x).unwrap();
    let height = u32::try_from(max_y - min_y).unwrap();
    let mut terrain_values = Vec::with_capacity((u64::from(width) * u64::from(height)) as usize);
    let mut resource_values = Vec::with_capacity(terrain_values.capacity());
    for grid_y in 0..height {
        let cell_min_y = max_y - i64::from(grid_y) - 1;
        for grid_x in 0..width {
            let cell_min_x = min_x + i64::from(grid_x);
            terrain_values.push(Some(terrain(cell_min_x, cell_min_y)));
            resource_values.push(Some(resources(cell_min_x, cell_min_y)));
        }
    }
    LandscapeBundle::new(
        width,
        height,
        GridGeometry {
            origin_x: min_x,
            origin_y: max_y,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:BOUNDARY-TEST".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                terrain_values,
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                resource_values,
            ),
        ],
    )
}

fn cell_at(landscape: &LandscapeBundle, min_x: i64, min_y: i64) -> CellId {
    let cell_count = u64::from(landscape.width) * u64::from(landscape.height);
    (1..=cell_count)
        .map(CellId::new)
        .find(|&cell| {
            let extent = landscape.cell_extent(cell).unwrap();
            extent.min_x == min_x && extent.min_y == min_y
        })
        .expect("physical cell must exist in landscape")
}

fn transformed_world(landscape: &LandscapeBundle, maximum_traversable_cost: u16) -> World {
    let mechanisms = SpatialMechanismConfig::new(
        "boundary-travel-transform-v1",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "terrain",
            "normalized_index",
            DOMAIN,
            1_000,
            maximum_traversable_cost,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    );
    let overlay = transform_landscape(landscape, &mechanisms).unwrap();
    World::generate(
        WorldConfig::new(landscape.width, landscape.height),
        RngFactory::new(211_001),
    )
    .unwrap()
    .with_model_field_overlay(overlay.movement_cost.as_deref(), None, None)
    .unwrap()
}

fn empirical_boundary_record() -> EvidenceRecord {
    EvidenceRecord {
        schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
        evidence_id: "boundary-barrier-evidence".to_owned(),
        provenance: ParameterProvenance::EmpiricalDirect,
        source: EvidenceSource {
            source_id: "boundary-source".to_owned(),
            citation: "Boundary fixture source".to_owned(),
            persistent_id: Some("fixture:boundary-source-v1".to_owned()),
            dataset_version: Some("v1".to_owned()),
            licence: Some("test".to_owned()),
            spatial_coverage: Some("fixture landscape".to_owned()),
            temporal_coverage: Some("fixture period".to_owned()),
        },
        original_variable: "closed barrier".to_owned(),
        original_units: "presence".to_owned(),
        transformation: None,
        simulation_units: "closed simulation boundary".to_owned(),
        uncertainty: None,
        applicability: "boundary contract fixture only".to_owned(),
        competing_estimates: Vec::new(),
    }
}

#[test]
fn closed_edge_interpretation_is_explicit_and_barrier_claims_require_evidence() {
    let unresolved = SpatialBoundaryDeclaration::unresolved();
    unresolved.validate(None).unwrap();
    assert_eq!(
        unresolved.interpretation,
        SpatialBoundaryInterpretation::UnresolvedExtent
    );
    assert!(unresolved.requires_extent_sensitivity());

    let crop = SpatialBoundaryDeclaration::analyst_defined_crop(
        "five-km-crop",
        "Extent was selected for analysis convenience rather than as a historical barrier.",
    );
    crop.validate(None).unwrap();
    assert!(crop.requires_extent_sensitivity());

    let unsupported_barrier = SpatialBoundaryDeclaration::declared_closed_barrier(
        "river-cliff-barrier",
        "Fixture treats the outer edge as impassable.",
        vec!["boundary-barrier-evidence".to_owned()],
    );
    assert!(matches!(
        unsupported_barrier.validate(None),
        Err(SpatialBoundaryError::MissingEvidenceCatalog)
    ));

    let evidence = EvidenceCatalog::new(vec![empirical_boundary_record()]);
    unsupported_barrier.validate(Some(&evidence)).unwrap();
    assert!(!unsupported_barrier.requires_extent_sensitivity());

    let criterion = SpatialExtentAdequacyCriterion {
        schema_version: SpatialExtentAdequacyCriterion::CURRENT_SCHEMA_VERSION,
        criterion_id: "study-specific-inner-domain-stability".to_owned(),
        required_consecutive_stable_extensions: 2,
        minimum_buffer_cells: Some(3),
        metric_tolerances: vec![
            SpatialExtentMetricTolerance {
                metric_id: "migration.moves_completed".to_owned(),
                max_absolute_difference: Some(0),
                max_relative_difference_permille: None,
            },
            SpatialExtentMetricTolerance {
                metric_id: "temporary.visitor_person_days".to_owned(),
                max_absolute_difference: Some(0),
                max_relative_difference_permille: Some(0),
            },
        ],
    };
    criterion.validate().unwrap();
}

#[test]
fn fixed_physical_inner_domain_exposes_m4_candidate_clipping_then_converges_with_buffer() {
    let terrain = |_: i64, _: i64| 0;
    let resources = |_: i64, _: i64| 500;
    let tight = rectangular_landscape(0, 0, 4, 4, terrain, resources);
    let buffered = rectangular_landscape(-2, -2, 6, 6, terrain, resources);
    let larger = rectangular_landscape(-3, -3, 7, 7, terrain, resources);
    let analysis = SpatialAnalysisDomain::new(
        "fixed-physical-inner-cell",
        SpatialAnalysisExtent {
            min_x: 2,
            min_y: 1,
            max_x: 3,
            max_y: 2,
        },
    );
    let migration = MigrationConfig::synthetic_validation_v1().with_candidate_radius_cells(2);

    let assess = |landscape: &LandscapeBundle| {
        let world = World::generate(
            WorldConfig::new(landscape.width, landscape.height),
            RngFactory::new(211_010),
        )
        .unwrap();
        assess_spatial_boundary(
            landscape,
            &world,
            &migration,
            SpatialBoundaryDeclaration::analyst_defined_crop(
                "test-crop",
                "Synthetic extent-convergence fixture.",
            ),
            analysis.clone(),
            None,
        )
        .unwrap()
    };

    let tight_assessment = assess(&tight);
    let buffered_assessment = assess(&buffered);
    let larger_assessment = assess(&larger);

    assert_eq!(tight_assessment.minimum_analysis_buffer_cells, 1);
    assert_eq!(
        tight_assessment.analysis_cells_with_truncated_m4_candidates,
        1
    );
    assert!(!tight_assessment.m4_analysis_horizon_clear_of_boundary);
    assert!(tight_assessment.cells[0].m4_candidate_set_truncated);
    assert!(tight_assessment.cells[0].m4_missing_candidate_count > 0);

    for assessment in [&buffered_assessment, &larger_assessment] {
        assert_eq!(assessment.analysis_cells_with_truncated_m4_candidates, 0);
        assert!(assessment.m4_analysis_horizon_clear_of_boundary);
        assert_eq!(
            assessment.cells[0].m4_candidate_count,
            assessment.m4_full_interior_candidate_count
        );
        assert!(assessment.minimum_analysis_buffer_cells >= 2);
        assert!(assessment.requires_extent_sensitivity);
        assert!(assessment.m9_routes_confined_to_simulation_domain);
        assert!(!assessment.m9_routes_may_leave_and_reenter_simulation_domain);
    }

    let physical_offsets = |landscape: &LandscapeBundle| {
        let world = World::generate(
            WorldConfig::new(landscape.width, landscape.height),
            RngFactory::new(211_011),
        )
        .unwrap();
        let origin = cell_at(landscape, 2, 1);
        let origin_extent = landscape.cell_extent(origin).unwrap();
        let mut offsets = bounded_candidate_cells(&world, origin, 2)
            .into_iter()
            .map(|cell| {
                let extent = landscape.cell_extent(cell).unwrap();
                (
                    extent.min_x - origin_extent.min_x,
                    extent.min_y - origin_extent.min_y,
                )
            })
            .collect::<Vec<_>>();
        offsets.sort_unstable();
        offsets
    };
    assert_eq!(physical_offsets(&buffered), physical_offsets(&larger));
    assert_ne!(physical_offsets(&tight), physical_offsets(&buffered));
}

fn wall_landscape(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> LandscapeBundle {
    rectangular_landscape(
        min_x,
        min_y,
        max_x,
        max_y,
        |x, y| {
            if x == 2 && (0..3).contains(&y) {
                1_000
            } else {
                0
            }
        },
        |_, _| 1_000,
    )
}

#[test]
fn m9_hard_wall_changes_reachability_until_the_same_inner_landscape_has_a_route_buffer() {
    let tight = wall_landscape(0, 0, 5, 3);
    let buffered = wall_landscape(-1, -1, 6, 4);
    let larger = wall_landscape(-2, -2, 7, 5);
    let model = TemporaryTravelModel::new(
        "closed-boundary-route-test",
        ParameterProvenance::SyntheticValidation,
        3_000,
        1_000,
    )
    .unwrap();

    let resolution = |landscape: &LandscapeBundle| {
        let world = transformed_world(landscape, 5_000);
        let origin = cell_at(landscape, 0, 1);
        let destination = cell_at(landscape, 4, 1);
        let region = FocalRegion::new(
            "fixed-physical-destination",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap();
        let table = model.derive_table(&region, &world).unwrap();
        (
            table.resolution(origin).unwrap(),
            table.accumulated_cost_units(origin),
        )
    };

    let tight_resolution = resolution(&tight);
    assert_eq!(tight_resolution.0, TemporaryTravelResolution::Unreachable);
    assert_eq!(tight_resolution.1, None);
    let buffered_resolution = resolution(&buffered);
    let larger_resolution = resolution(&larger);
    assert_eq!(buffered_resolution, larger_resolution);
    assert!(matches!(
        buffered_resolution.0,
        TemporaryTravelResolution::Reachable {
            outbound_travel_days: 3,
            return_travel_days: 3,
            ..
        }
    ));
    assert_eq!(buffered_resolution.1, Some(8_000));
}

fn single_founder(cell: CellId) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "boundary-single-founder-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: cell,
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

fn resource_mechanisms(model_id: &str) -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        model_id,
        vec![SpatialFieldTransform::new(
            SpatialTargetField::BaseProductivity,
            "resources",
            "normalized_index",
            DOMAIN,
            100,
            1_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    )
}

fn m4_extent_landscape(max_x: i64) -> LandscapeBundle {
    rectangular_landscape(
        0,
        0,
        max_x,
        3,
        |_, _| 0,
        |x, y| {
            if x == 4 && y == 1 { 1_000 } else { 0 }
        },
    )
}

fn forced_extent_migration(landscape: LandscapeBundle) -> (u64, u64, CellId) {
    let origin = cell_at(&landscape, 3, 1);
    let mut migration = MigrationConfig::synthetic_validation_v1()
        .with_candidate_radius_cells(1)
        .with_decision_periods_per_year(1);
    migration.resource_pressure_threshold_permille = 1_000;
    migration.minimum_utility_improvement = 500;
    migration.resource_weight = 1;
    migration.water_security_weight = 0;
    migration.kin_weight = 0;
    migration.travel_cost_weight = 0;
    migration.max_uncertainty_penalty_permille = 0;
    migration.relocation_risk_base_penalty_permille = 0;
    migration.relocation_risk_per_cell_permille = 0;
    migration.travel_condition_cost_per_cell = 0;

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 10_000;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(211_020, 1)
        .with_world(WorldConfig::new(landscape.width, landscape.height))
        .with_population(PopulationConfig::new(1).with_max_person_records(1))
        .with_founder_population(single_founder(origin))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(migration);
    let checkpoint = SpatialLandscapeSimulation::new(
        config,
        landscape.clone(),
        resource_mechanisms("boundary-m4-resource-gradient-v1"),
    )
    .unwrap()
    .checkpoint_at_year(1)
    .unwrap();
    let destination = checkpoint
        .core_checkpoint
        .population
        .household_location(HouseholdId::new(1))
        .unwrap();
    (
        checkpoint.core_checkpoint.migration.moves_completed,
        checkpoint.core_checkpoint.migration.eastward_steps,
        destination,
    )
}

#[test]
fn m4_migration_frequency_and_direction_change_when_the_crop_removes_the_only_better_cell() {
    let tight = m4_extent_landscape(4);
    let buffered = m4_extent_landscape(5);
    let larger = m4_extent_landscape(6);

    let tight_result = forced_extent_migration(tight.clone());
    let buffered_result = forced_extent_migration(buffered.clone());
    let larger_result = forced_extent_migration(larger.clone());

    assert_eq!((tight_result.0, tight_result.1), (0, 0));
    assert_eq!((buffered_result.0, buffered_result.1), (1, 1));
    assert_eq!((larger_result.0, larger_result.1), (1, 1));
    assert_eq!(
        buffered.cell_extent(buffered_result.2).unwrap().min_x,
        4,
        "the buffered run must move east into the newly represented physical cell"
    );
    assert_eq!(
        larger.cell_extent(larger_result.2).unwrap().min_x,
        4,
        "adding another outer column must preserve the converged first migration"
    );
}

fn travel_resource_mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "boundary-travel-resource-v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                DOMAIN,
                1_000,
                5_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                DOMAIN,
                1_000,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
}

fn m9_experiment(landscape: &LandscapeBundle, with_temporary_mobility: bool) -> ExperimentConfig {
    let origin = cell_at(landscape, 0, 1);
    let destination = cell_at(landscape, 4, 1);
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 365;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let base = ExperimentConfig::new(211_030, 1)
        .with_world(WorldConfig::new(landscape.width, landscape.height))
        .with_population(PopulationConfig::new(1).with_max_person_records(1))
        .with_founder_population(single_founder(origin))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    if !with_temporary_mobility {
        return base;
    }

    let region = FocalRegion::new(
        "boundary-fixed-focal-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "boundary-one-visit",
        TemporaryTriggerTiming::DepartureDay,
        vec![20],
        5,
    )
    .unwrap();
    let travel = TemporaryTravelModel::new(
        "boundary-closed-grid-travel",
        ParameterProvenance::SyntheticValidation,
        10_000,
        1_000,
    )
    .unwrap();
    base.with_temporary_mobility(TemporaryMobilityConfig::new(region, schedule, travel).unwrap())
}

fn m9_resource_and_focal_result(landscape: LandscapeBundle) -> (u64, u64, u64, u64) {
    let destination = cell_at(&landscape, 4, 1);
    let mechanisms = travel_resource_mechanisms();

    let disabled = SpatialLandscapeSimulation::new(
        m9_experiment(&landscape, false),
        landscape.clone(),
        mechanisms.clone(),
    )
    .unwrap()
    .checkpoint_at_year(1)
    .unwrap();
    let disabled_destination_stock = disabled
        .core_checkpoint
        .resources
        .cell_food_stock(destination)
        .unwrap();

    let enabled_simulation =
        SpatialLandscapeSimulation::new(m9_experiment(&landscape, true), landscape, mechanisms)
            .unwrap();
    let world = enabled_simulation.world().clone();
    let initial_population = enabled_simulation.population().clone();
    let enabled = enabled_simulation.checkpoint_at_year(1).unwrap();
    let enabled_destination_stock = enabled
        .core_checkpoint
        .resources
        .cell_food_stock(destination)
        .unwrap();
    let observability = derive_temporary_mobility_observability(
        &world,
        &initial_population,
        &enabled.core_checkpoint,
    )
    .unwrap();

    (
        disabled_destination_stock - enabled_destination_stock,
        observability.summary.journeys_started,
        observability.summary.not_started_unreachable,
        observability.summary.visitor_person_days,
    )
}

#[test]
fn m9_boundary_reachability_propagates_into_focal_visits_and_destination_resource_pressure() {
    let tight = m9_resource_and_focal_result(wall_landscape(0, 0, 5, 3));
    let buffered = m9_resource_and_focal_result(wall_landscape(-1, -1, 6, 4));
    let larger = m9_resource_and_focal_result(wall_landscape(-2, -2, 7, 5));

    assert_eq!(tight, (0, 0, 1, 0));
    assert_eq!(buffered, (5, 1, 0, 5));
    assert_eq!(larger, buffered);
}
