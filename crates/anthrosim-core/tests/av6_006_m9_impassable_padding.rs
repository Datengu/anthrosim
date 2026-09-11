use anthrosim_core::{
    config::{ParameterProvenance, WorldConfig},
    focal_region::{FocalRegion, FocalRegionSource},
    ids::CellId,
    rng::RngFactory,
    temporary_mobility::TemporaryTravelResolution,
    temporary_travel::TemporaryTravelModel,
    world::World,
};

fn world(movement_cost: &[u16]) -> World {
    let world = World::generate(
        WorldConfig::new(u32::try_from(movement_cost.len()).unwrap(), 1),
        RngFactory::new(711_001),
    )
    .unwrap();
    world
        .with_model_field_overlay(Some(movement_cost), None, None)
        .unwrap()
}

fn region(world: &World) -> FocalRegion {
    FocalRegion::new(
        "v6-area-e-local-tie",
        FocalRegionSource::Synthetic,
        vec![CellId::new(1), CellId::new(3)],
    )
    .and_then(|region| {
        region.validate(world)?;
        Ok(region)
    })
    .unwrap()
}

fn destination(resolution: TemporaryTravelResolution) -> CellId {
    match resolution {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => {
            panic!("expected local M9 route to remain reachable")
        }
    }
}

#[test]
fn unreachable_impassable_padding_cannot_change_local_equal_cost_m9_destination() {
    let baseline = world(&[1_000, 1_000, 1_000]);
    let padded = world(&[1_000, 1_000, 1_000, 6_000]);
    let baseline_region = region(&baseline);
    let padded_region = region(&padded);
    let model = TemporaryTravelModel::new(
        "v6-area-e-impassable-padding",
        ParameterProvenance::SyntheticValidation,
        3_000,
        5_000,
    )
    .unwrap();
    let origin = CellId::new(2);
    let coupling_key = 711_101_u64;
    let trigger_index = 0_u32;

    for seed in 0..128_u64 {
        let baseline_table = model
            .derive_table_with_tie_seed(&baseline_region, &baseline, seed)
            .unwrap();
        let padded_table = model
            .derive_table_with_tie_seed(&padded_region, &padded, seed)
            .unwrap();

        assert_eq!(baseline_table.accumulated_cost_units(origin), Some(1_000));
        assert_eq!(padded_table.accumulated_cost_units(origin), Some(1_000));
        assert_eq!(baseline_table.equal_cost_destination_count(origin), Some(2));
        assert_eq!(padded_table.equal_cost_destination_count(origin), Some(2));
        for destination in [CellId::new(1), CellId::new(3)] {
            assert_eq!(
                baseline_table.route_distance_edges(origin, destination),
                Some(1)
            );
            assert_eq!(
                padded_table.route_distance_edges(origin, destination),
                Some(1)
            );
        }
        assert!(!model.is_traversable(&padded, CellId::new(4)));

        let baseline_destination = destination(
            baseline_table
                .resolution_for_coupling_key(origin, coupling_key, trigger_index)
                .unwrap(),
        );
        let padded_destination = destination(
            padded_table
                .resolution_for_coupling_key(origin, coupling_key, trigger_index)
                .unwrap(),
        );

        assert_eq!(
            padded_destination, baseline_destination,
            "adding one unreachable impassable cell outside the unchanged local M9 problem must not change the selected equal-cost destination at seed {seed}"
        );
    }
}
