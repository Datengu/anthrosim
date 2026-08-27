use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    FocalRegion, FocalRegionSource, GridGeometry, LandscapeBinding, LandscapeBindingError,
    LandscapeBundle, MigrationConfig, ResourceConfig, ResourceSystem, SpatialM4DistanceBasis,
    SpatialM9TravelCostBasis, SpatialResourceQuantityBasis, SpatialScaleSemantics,
    SpatialScaleStatus, TemporaryTravelModel, TemporaryTravelResolution, World, WorldConfig,
    bounded_candidate_cells,
};

fn landscape(width: u32, height: u32, cell_size: u64) -> LandscapeBundle {
    LandscapeBundle::new(
        width,
        height,
        GridGeometry {
            origin_x: 0,
            origin_y: i64::try_from(u64::from(height) * cell_size).unwrap(),
            cell_size_x: cell_size,
            cell_size_y: cell_size,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:RESOLUTION-TEST".to_owned(),
        },
        Vec::new(),
    )
}

fn constant_world(width: u32, height: u32, productivity: u16) -> World {
    let count = usize::try_from(u64::from(width) * u64::from(height)).unwrap();
    World::generate(WorldConfig::new(width, height), RngFactory::new(203_001))
        .unwrap()
        .with_model_field_overlay(
            Some(&vec![1_000; count]),
            None,
            Some(&vec![productivity; count]),
        )
        .unwrap()
}

#[test]
fn landscape_binding_declares_resolution_dependence_and_rejects_scale_tampering() {
    let coarse = landscape(2, 2, 100);
    let binding = LandscapeBinding::from_bundle(&coarse).unwrap();

    assert_eq!(
        binding.schema_version,
        LandscapeBinding::CURRENT_SCHEMA_VERSION
    );
    assert_eq!(
        binding.scale.semantics,
        SpatialScaleSemantics::CellSpaceResolutionDependentV1
    );
    assert_eq!(
        binding.scale.status,
        SpatialScaleStatus::ResolutionDependent
    );
    assert_eq!(binding.scale.cell_size_x, 100);
    assert_eq!(binding.scale.cell_size_y, 100);
    assert_eq!(binding.scale.coordinate_unit, "metre");
    assert_eq!(
        binding.scale.resource_quantity_basis,
        SpatialResourceQuantityBasis::PerCellTotal
    );
    assert_eq!(
        binding.scale.m4_distance_basis,
        SpatialM4DistanceBasis::GridSteps
    );
    assert_eq!(
        binding.scale.m9_travel_cost_basis,
        SpatialM9TravelCostBasis::GridEdges
    );
    assert!(binding.scale.requires_resolution_sensitivity);

    let mut forged = binding;
    forged.scale.cell_size_x = 50;
    assert!(matches!(
        forged.validate_bundle(&coarse),
        Err(LandscapeBindingError::BindingMismatch { .. })
    ));
}

#[test]
fn equal_physical_area_gains_four_times_resource_stock_when_resolution_is_doubled() {
    // Both landscapes cover 200 m x 200 m. Current M3 semantics attach resource opportunity to
    // each cell total, so subdividing every 100 m cell into four 50 m cells multiplies aggregate
    // opportunity rather than conserving it by area.
    let coarse_landscape = landscape(2, 2, 100);
    let fine_landscape = landscape(4, 4, 50);
    assert_eq!(
        u64::from(coarse_landscape.width) * coarse_landscape.geometry.cell_size_x,
        u64::from(fine_landscape.width) * fine_landscape.geometry.cell_size_x
    );
    assert_eq!(
        u64::from(coarse_landscape.height) * coarse_landscape.geometry.cell_size_y,
        u64::from(fine_landscape.height) * fine_landscape.geometry.cell_size_y
    );

    let coarse_world = constant_world(2, 2, 500);
    let fine_world = constant_world(4, 4, 500);
    let resources = ResourceConfig::synthetic_validation_v1();
    let coarse_stock = ResourceSystem::initialize(&coarse_world, &resources)
        .unwrap()
        .total_food_stock()
        .unwrap();
    let fine_stock = ResourceSystem::initialize(&fine_world, &resources)
        .unwrap()
        .total_food_stock()
        .unwrap();

    assert_eq!(fine_stock, coarse_stock * 4);
}

#[test]
fn same_m4_candidate_radius_has_half_the_physical_horizon_at_fifty_metres() {
    // Both rasters cover 700 m x 700 m. M4 keeps the same three-cell radius, so its physical
    // information horizon changes from 300 m to 150 m when resolution doubles.
    let coarse_landscape = landscape(7, 7, 100);
    let fine_landscape = landscape(14, 14, 50);
    let coarse_world = constant_world(7, 7, 500);
    let fine_world = constant_world(14, 14, 500);
    let radius = MigrationConfig::synthetic_validation_v1().candidate_radius_cells;
    assert_eq!(radius, 3);

    let coarse_origin = coarse_world.cell_id(3, 3).unwrap();
    let fine_origin = fine_world.cell_id(7, 7).unwrap();
    let coarse_candidates = bounded_candidate_cells(&coarse_world, coarse_origin, radius);
    let fine_candidates = bounded_candidate_cells(&fine_world, fine_origin, radius);

    let max_grid_steps = |world: &World, origin, candidates: &[anthrosim_core::ids::CellId]| {
        let (origin_x, origin_y) = world.coordinates(origin).unwrap();
        candidates
            .iter()
            .map(|&cell| {
                let (x, y) = world.coordinates(cell).unwrap();
                origin_x.abs_diff(x) + origin_y.abs_diff(y)
            })
            .max()
            .unwrap()
    };
    assert_eq!(
        max_grid_steps(&coarse_world, coarse_origin, &coarse_candidates),
        3
    );
    assert_eq!(
        max_grid_steps(&fine_world, fine_origin, &fine_candidates),
        3
    );

    let coarse_horizon = u64::from(radius) * coarse_landscape.geometry.cell_size_x;
    let fine_horizon = u64::from(radius) * fine_landscape.geometry.cell_size_x;
    assert_eq!(coarse_horizon, 300);
    assert_eq!(fine_horizon, 150);
}

#[test]
fn equal_physical_m9_route_accumulates_more_cost_at_finer_resolution() {
    // Uniform movement cost isolates edge count. The compared routes are both 200 m long:
    // two 100 m edges versus four 50 m edges. M9 currently charges one model movement-cost unit
    // per grid edge, so the finer raster doubles accumulated cost and changes travel duration.
    let coarse_landscape = landscape(3, 1, 100);
    let fine_landscape = landscape(6, 1, 50);
    let coarse_world = constant_world(3, 1, 500);
    let fine_world = constant_world(6, 1, 500);
    let coarse_origin = coarse_world.cell_id(0, 0).unwrap();
    let coarse_destination = coarse_world.cell_id(2, 0).unwrap();
    let fine_origin = fine_world.cell_id(0, 0).unwrap();
    let fine_destination = fine_world.cell_id(4, 0).unwrap();

    let coarse_extent = coarse_landscape.cell_centre_2x(coarse_origin).unwrap();
    let coarse_destination_extent = coarse_landscape.cell_centre_2x(coarse_destination).unwrap();
    let fine_extent = fine_landscape.cell_centre_2x(fine_origin).unwrap();
    let fine_destination_extent = fine_landscape.cell_centre_2x(fine_destination).unwrap();
    assert_eq!(
        (coarse_destination_extent.x_twice - coarse_extent.x_twice).unsigned_abs(),
        400
    );
    assert_eq!(
        (fine_destination_extent.x_twice - fine_extent.x_twice).unsigned_abs(),
        400
    );

    let coarse_region = FocalRegion::new(
        "coarse-destination",
        FocalRegionSource::Synthetic,
        vec![coarse_destination],
    )
    .unwrap();
    let fine_region = FocalRegion::new(
        "fine-destination",
        FocalRegionSource::Synthetic,
        vec![fine_destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::synthetic_validation_v1();
    let coarse_table = model.derive_table(&coarse_region, &coarse_world).unwrap();
    let fine_table = model.derive_table(&fine_region, &fine_world).unwrap();

    assert_eq!(
        coarse_table.accumulated_cost_units(coarse_origin),
        Some(2_000)
    );
    assert_eq!(fine_table.accumulated_cost_units(fine_origin), Some(4_000));
    assert_eq!(
        coarse_table.resolution(coarse_origin),
        Some(TemporaryTravelResolution::Reachable {
            destination: coarse_destination,
            outbound_travel_days: 1,
            return_travel_days: 1,
        })
    );
    assert_eq!(
        fine_table.resolution(fine_origin),
        Some(TemporaryTravelResolution::Reachable {
            destination: fine_destination,
            outbound_travel_days: 2,
            return_travel_days: 2,
        })
    );
}
