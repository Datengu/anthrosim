use anthrosim_core::{
    CellId, FocalRegion, FocalRegionSource, HouseholdId, RngFactory, TemporaryTravelModel,
    TemporaryTravelResolution, World, WorldConfig,
};

fn flat_world(width: u32, height: u32) -> World {
    let world = World::generate(WorldConfig::new(width, height), RngFactory::new(190)).unwrap();
    world
        .with_model_field_overlay(Some(&vec![1_000; world.cell_count()]), None, None)
        .unwrap()
}

fn region(world: &World, cells: Vec<CellId>) -> FocalRegion {
    FocalRegion::new("equal-cost-test", FocalRegionSource::Synthetic, cells)
        .and_then(|region| {
            region.validate(world)?;
            Ok(region)
        })
        .unwrap()
}

fn destination(resolution: TemporaryTravelResolution) -> CellId {
    match resolution {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => panic!("expected reachable travel resolution"),
    }
}

#[test]
fn flat_three_by_one_retains_both_equal_minima_and_keyed_choice_is_reproducible() {
    let world = flat_world(3, 1);
    let focal = region(&world, vec![CellId::new(1), CellId::new(3)]);
    let table = TemporaryTravelModel::default()
        .derive_table_with_tie_seed(&focal, &world, 190)
        .unwrap();
    let origin = CellId::new(2);

    let candidates = table.equal_cost_destinations(origin).unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].destination, CellId::new(1));
    assert_eq!(candidates[1].destination, CellId::new(3));
    assert_eq!(candidates[0].route_distance_edges, 1);
    assert_eq!(candidates[1].route_distance_edges, 1);

    let first = table
        .resolution_for(origin, HouseholdId::new(17), 4)
        .unwrap();
    for _ in 0..20 {
        assert_eq!(
            table
                .resolution_for(origin, HouseholdId::new(17), 4)
                .unwrap(),
            first
        );
    }
}

#[test]
fn tied_households_do_not_systematically_choose_lower_cell_id() {
    let world = flat_world(3, 1);
    let focal = region(&world, vec![CellId::new(1), CellId::new(3)]);
    let table = TemporaryTravelModel::default()
        .derive_table_with_tie_seed(&focal, &world, 190)
        .unwrap();
    let origin = CellId::new(2);
    let mut lower = 0_u32;
    let mut upper = 0_u32;

    for household in 1..=512 {
        match destination(
            table
                .resolution_for(origin, HouseholdId::new(household), 0)
                .unwrap(),
        ) {
            cell if cell == CellId::new(1) => lower += 1,
            cell if cell == CellId::new(3) => upper += 1,
            cell => panic!("unexpected tied destination {cell:?}"),
        }
    }

    assert!(lower > 0 && upper > 0);
    assert!(lower.abs_diff(upper) < 128, "lower={lower}, upper={upper}");
}

#[test]
fn horizontal_and_vertical_symmetries_have_no_fixed_directional_winner() {
    let world = flat_world(3, 3);
    let horizontal = region(&world, vec![CellId::new(4), CellId::new(6)]);
    let vertical = region(&world, vec![CellId::new(2), CellId::new(8)]);
    let horizontal_table = TemporaryTravelModel::default()
        .derive_table_with_tie_seed(&horizontal, &world, 190)
        .unwrap();
    let vertical_table = TemporaryTravelModel::default()
        .derive_table_with_tie_seed(&vertical, &world, 190)
        .unwrap();
    let origin = CellId::new(5);
    let mut left = 0_u32;
    let mut right = 0_u32;
    let mut top = 0_u32;
    let mut bottom = 0_u32;

    for household in 1..=512 {
        match destination(
            horizontal_table
                .resolution_for(origin, HouseholdId::new(household), 1)
                .unwrap(),
        ) {
            cell if cell == CellId::new(4) => left += 1,
            cell if cell == CellId::new(6) => right += 1,
            cell => panic!("unexpected horizontal destination {cell:?}"),
        }
        match destination(
            vertical_table
                .resolution_for(origin, HouseholdId::new(household), 1)
                .unwrap(),
        ) {
            cell if cell == CellId::new(2) => top += 1,
            cell if cell == CellId::new(8) => bottom += 1,
            cell => panic!("unexpected vertical destination {cell:?}"),
        }
    }

    assert!(left > 0 && right > 0 && top > 0 && bottom > 0);
    assert!(left.abs_diff(right) < 128, "left={left}, right={right}");
    assert!(top.abs_diff(bottom) < 128, "top={top}, bottom={bottom}");
}

#[test]
fn varying_seed_can_select_either_symmetric_destination_for_same_household() {
    let world = flat_world(3, 1);
    let focal = region(&world, vec![CellId::new(1), CellId::new(3)]);
    let origin = CellId::new(2);
    let mut seen_lower = false;
    let mut seen_upper = false;

    for seed in 0..64 {
        let table = TemporaryTravelModel::default()
            .derive_table_with_tie_seed(&focal, &world, seed)
            .unwrap();
        match destination(
            table
                .resolution_for(origin, HouseholdId::new(1), 0)
                .unwrap(),
        ) {
            cell if cell == CellId::new(1) => seen_lower = true,
            cell if cell == CellId::new(3) => seen_upper = true,
            cell => panic!("unexpected seeded destination {cell:?}"),
        }
    }

    assert!(seen_lower && seen_upper);
}

#[test]
fn non_tied_minimum_is_identical_for_all_tie_keys() {
    let world = flat_world(3, 1);
    let focal = region(&world, vec![CellId::new(3)]);
    let origin = CellId::new(1);

    for seed in [0_u64, 1, 190, u32::MAX as u64] {
        let table = TemporaryTravelModel::default()
            .derive_table_with_tie_seed(&focal, &world, seed)
            .unwrap();
        let base = table.resolution(origin).unwrap();
        assert_eq!(table.equal_cost_destination_count(origin), Some(1));
        for household in [1_u64, 2, 99, 10_000] {
            for trigger in [0_u32, 1, 17] {
                assert_eq!(
                    table
                        .resolution_for(origin, HouseholdId::new(household), trigger)
                        .unwrap(),
                    base
                );
            }
        }
    }
}
