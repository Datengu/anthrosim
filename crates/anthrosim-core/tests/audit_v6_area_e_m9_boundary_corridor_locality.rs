use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    FocalRegion, FocalRegionSource, ParameterProvenance, TemporaryTravelModel,
    TemporaryTravelResolution, World, WorldConfig,
};

fn world(width: u32, movement_cost: &[u16]) -> World {
    World::generate(WorldConfig::new(width, 1), RngFactory::new(713_001))
        .unwrap()
        .with_model_field_overlay(Some(movement_cost), None, None)
        .unwrap()
}

fn travel_model() -> TemporaryTravelModel {
    TemporaryTravelModel::new(
        "audit_v6_area_e_boundary_corridor_v1",
        ParameterProvenance::SyntheticValidation,
        1_500,
        5_000,
    )
    .unwrap()
}

#[test]
fn unique_m9_boundary_corridor_is_invariant_to_impassable_embedding_padding() {
    let baseline = world(3, &[1_000, 1_000, 1_000]);
    let padded = world(5, &[6_000, 1_000, 1_000, 1_000, 6_000]);

    let baseline_origin = baseline.cell_id(0, 0).unwrap();
    let baseline_destination = baseline.cell_id(2, 0).unwrap();
    let padded_origin = padded.cell_id(1, 0).unwrap();
    let padded_destination = padded.cell_id(3, 0).unwrap();

    let baseline_region = FocalRegion::new(
        "audit-v6-area-e-boundary-corridor",
        FocalRegionSource::Synthetic,
        vec![baseline_destination],
    )
    .unwrap();
    let padded_region = FocalRegion::new(
        "audit-v6-area-e-boundary-corridor",
        FocalRegionSource::Synthetic,
        vec![padded_destination],
    )
    .unwrap();

    let model = travel_model();
    let baseline_table = model
        .derive_table_with_tie_seed(&baseline_region, &baseline, 713_002)
        .unwrap();
    let padded_table = model
        .derive_table_with_tie_seed(&padded_region, &padded, 713_002)
        .unwrap();

    let left_padding = padded.cell_id(0, 0).unwrap();
    let right_padding = padded.cell_id(4, 0).unwrap();
    assert!(!model.is_traversable(&padded, left_padding));
    assert!(!model.is_traversable(&padded, right_padding));

    assert_eq!(baseline_table.accumulated_cost_units(baseline_origin), Some(2_000));
    assert_eq!(padded_table.accumulated_cost_units(padded_origin), Some(2_000));
    assert_eq!(
        baseline_table.equal_cost_destination_count(baseline_origin),
        Some(1)
    );
    assert_eq!(
        padded_table.equal_cost_destination_count(padded_origin),
        Some(1)
    );
    assert_eq!(
        baseline_table.route_distance_edges(baseline_origin, baseline_destination),
        Some(2)
    );
    assert_eq!(
        padded_table.route_distance_edges(padded_origin, padded_destination),
        Some(2)
    );

    let baseline_resolution =
        baseline_table.resolution_for_coupling_key(baseline_origin, 47, 0);
    let padded_resolution = padded_table.resolution_for_coupling_key(padded_origin, 47, 0);

    eprintln!(
        "M9 boundary-corridor locality: baseline={baseline_resolution:?} padded={padded_resolution:?}"
    );

    assert_eq!(
        baseline_resolution,
        Some(TemporaryTravelResolution::Reachable {
            destination: baseline_destination,
            outbound_travel_days: 2,
            return_travel_days: 2,
        })
    );
    assert_eq!(
        padded_resolution,
        Some(TemporaryTravelResolution::Reachable {
            destination: padded_destination,
            outbound_travel_days: 2,
            return_travel_days: 2,
        })
    );
}
