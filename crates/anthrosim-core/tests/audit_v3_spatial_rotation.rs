use anthrosim_core::ids::CellId;
use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    FocalRegion, FocalRegionSource, GridGeometry, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, NoDataPolicy, ParameterProvenance,
    SpatialFieldTransform, SpatialMechanismConfig, SpatialTargetField, TemporaryTravelModel,
    TemporaryTravelResolution, TransformDirection, World, WorldConfig, transform_landscape,
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };
const SIZE: i64 = 5;

fn layer(values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: "terrain".to_owned(),
        role: LandscapeLayerRole::TerrainTraversal,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values,
    }
}

fn landscape(terrain: impl Fn(i64, i64) -> i32) -> LandscapeBundle {
    let mut values = Vec::with_capacity((SIZE * SIZE) as usize);
    for grid_y in 0..SIZE {
        let cell_min_y = SIZE - grid_y - 1;
        for grid_x in 0..SIZE {
            values.push(Some(terrain(grid_x, cell_min_y)));
        }
    }
    LandscapeBundle::new(
        SIZE as u32,
        SIZE as u32,
        GridGeometry {
            origin_x: 0,
            origin_y: SIZE,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:AUDIT-V3-ROTATION".to_owned(),
        },
        vec![layer(values)],
    )
}

fn cell_at(landscape: &LandscapeBundle, x: i64, y: i64) -> CellId {
    (1..=(SIZE * SIZE) as u64)
        .map(CellId::new)
        .find(|&cell| {
            let extent = landscape.cell_extent(cell).unwrap();
            extent.min_x == x && extent.min_y == y
        })
        .unwrap()
}

fn transformed_world(landscape: &LandscapeBundle) -> World {
    let mechanisms = SpatialMechanismConfig::new(
        "audit-v3-rotation-movement-cost-v1",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "terrain",
            "normalized_index",
            DOMAIN,
            1_000,
            5_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    );
    let overlay = transform_landscape(landscape, &mechanisms).unwrap();
    World::generate(
        WorldConfig::new(landscape.width, landscape.height),
        RngFactory::new(303_001),
    )
    .unwrap()
    .with_model_field_overlay(overlay.movement_cost.as_deref(), None, None)
    .unwrap()
}

fn resolve(
    landscape: &LandscapeBundle,
    origin_xy: (i64, i64),
    destination_xy: (i64, i64),
) -> (TemporaryTravelResolution, Option<u64>) {
    let world = transformed_world(landscape);
    let origin = cell_at(landscape, origin_xy.0, origin_xy.1);
    let destination = cell_at(landscape, destination_xy.0, destination_xy.1);
    let focal = FocalRegion::new(
        "audit-v3-rotated-destination",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "audit-v3-rotation-travel-v1",
        ParameterProvenance::SyntheticValidation,
        3_000,
        1_000,
    )
    .unwrap();
    let table = model.derive_table(&focal, &world).unwrap();
    (
        table.resolution(origin).unwrap(),
        table.accumulated_cost_units(origin),
    )
}

#[test]
fn m9_route_cost_and_duration_are_invariant_under_quarter_turn_rotation() {
    // Deliberately asymmetric non-flat terrain. The second arm is an exact 90-degree
    // rotation of the first, including origin and focal destination. This attacks
    // hidden row/column, north-up, tie-order and directional edge-cost dependence.
    let terrain = |x: i64, y: i64| -> i32 {
        match (x, y) {
            (1, 1) | (1, 2) | (2, 3) => 1_000,
            (3, 1) | (3, 2) => 700,
            (2, 1) => 300,
            _ => 0,
        }
    };
    let rotated_terrain = |x: i64, y: i64| -> i32 {
        // Clockwise inverse: source (sx, sy) = (y, SIZE - 1 - x).
        terrain(y, SIZE - 1 - x)
    };

    let base = landscape(terrain);
    let rotated = landscape(rotated_terrain);
    let base_result = resolve(&base, (0, 2), (4, 2));
    let rotated_result = resolve(&rotated, (2, 0), (2, 4));

    assert_eq!(base_result.1, rotated_result.1, "quarter-turn rotation changed minimum accumulated route cost");

    match (base_result.0, rotated_result.0) {
        (
            TemporaryTravelResolution::Reachable {
                outbound_travel_days: base_out,
                return_travel_days: base_back,
                ..
            },
            TemporaryTravelResolution::Reachable {
                outbound_travel_days: rotated_out,
                return_travel_days: rotated_back,
                ..
            },
        ) => {
            assert_eq!(base_out, rotated_out, "quarter-turn rotation changed outbound duration");
            assert_eq!(base_back, rotated_back, "quarter-turn rotation changed return duration");
        }
        pair => panic!("rotated-equivalent routes must have identical reachability: {pair:?}"),
    }
}
