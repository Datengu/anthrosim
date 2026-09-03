use anthrosim_core::{
    FocalRegion, FocalRegionSource, ParameterProvenance, TemporaryTravelModel,
    TemporaryTravelResolution, World, WorldConfig,
    ids::CellId,
    rng::RngFactory,
};

const WIDTH: u32 = 7;
const HEIGHT: u32 = 5;
const DEST_X: u32 = 5;
const DEST_Y: u32 = 2;

fn cell(x: u32, y: u32) -> CellId {
    CellId::new(u64::from(y * WIDTH + x + 1))
}

fn mirror_cell(source_cell: CellId) -> CellId {
    let index = u32::try_from(source_cell.0 - 1).unwrap();
    let x = index % WIDTH;
    let y = index / WIDTH;
    cell(WIDTH - 1 - x, y)
}

fn movement_field(pattern: u32) -> Vec<u16> {
    let mut values = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = (x * 137 + y * 211 + pattern * 73 + x * y * 29 + pattern * x * 17)
                % 1_600;
            values.push(u16::try_from(1_000 + offset).unwrap());
        }
    }
    values[usize::try_from(DEST_Y * WIDTH + DEST_X).unwrap()] = 1_000;
    values
}

fn horizontal_reflection(values: &[u16]) -> Vec<u16> {
    let mut reflected = vec![0_u16; values.len()];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let source = usize::try_from(y * WIDTH + x).unwrap();
            let target = usize::try_from(y * WIDTH + (WIDTH - 1 - x)).unwrap();
            reflected[target] = values[source];
        }
    }
    reflected
}

fn world(values: &[u16]) -> World {
    World::generate(WorldConfig::new(WIDTH, HEIGHT), RngFactory::new(94_001))
        .unwrap()
        .with_model_field_overlay(Some(values), None, None)
        .unwrap()
}

fn model() -> TemporaryTravelModel {
    TemporaryTravelModel::new(
        "audit-v4-m9-route-reflection",
        ParameterProvenance::SyntheticValidation,
        2_400,
        2_200,
    )
    .unwrap()
}

#[test]
fn m9_unique_destination_routes_are_equivariant_under_horizontal_reflection() {
    let model = model();
    let canonical_destination = cell(DEST_X, DEST_Y);
    let reflected_destination = mirror_cell(canonical_destination);
    let mut comparisons = 0_u64;

    for pattern in 0..64_u32 {
        let canonical_values = movement_field(pattern);
        let reflected_values = horizontal_reflection(&canonical_values);
        let canonical_world = world(&canonical_values);
        let reflected_world = world(&reflected_values);
        let canonical_region = FocalRegion::new(
            "audit-v4-route-reflection-canonical",
            FocalRegionSource::Synthetic,
            vec![canonical_destination],
        )
        .unwrap();
        let reflected_region = FocalRegion::new(
            "audit-v4-route-reflection-reflected",
            FocalRegionSource::Synthetic,
            vec![reflected_destination],
        )
        .unwrap();
        let canonical = model
            .derive_table(&canonical_region, &canonical_world)
            .unwrap();
        let reflected = model
            .derive_table(&reflected_region, &reflected_world)
            .unwrap();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let origin = cell(x, y);
                let mirrored_origin = mirror_cell(origin);
                let canonical_resolution = canonical.resolution(origin).unwrap();
                let reflected_resolution = reflected.resolution(mirrored_origin).unwrap();

                match (canonical_resolution, reflected_resolution) {
                    (TemporaryTravelResolution::Unreachable, TemporaryTravelResolution::Unreachable) => {
                        assert_eq!(canonical.accumulated_cost_units(origin), None);
                        assert_eq!(reflected.accumulated_cost_units(mirrored_origin), None);
                    }
                    (
                        TemporaryTravelResolution::Reachable {
                            destination,
                            outbound_travel_days,
                            return_travel_days,
                        },
                        TemporaryTravelResolution::Reachable {
                            destination: reflected_destination_observed,
                            outbound_travel_days: reflected_outbound,
                            return_travel_days: reflected_return,
                        },
                    ) => {
                        assert_eq!(destination, canonical_destination, "pattern={pattern} origin={origin:?}");
                        assert_eq!(
                            reflected_destination_observed,
                            reflected_destination,
                            "pattern={pattern} origin={origin:?}"
                        );
                        assert_eq!(
                            outbound_travel_days, reflected_outbound,
                            "pattern={pattern} origin={origin:?}"
                        );
                        assert_eq!(
                            return_travel_days, reflected_return,
                            "pattern={pattern} origin={origin:?}"
                        );
                        assert_eq!(
                            canonical.accumulated_cost_units(origin),
                            reflected.accumulated_cost_units(mirrored_origin),
                            "pattern={pattern} origin={origin:?}"
                        );
                        assert_eq!(
                            canonical.route_distance_edges(origin, canonical_destination),
                            reflected.route_distance_edges(mirrored_origin, reflected_destination),
                            "pattern={pattern} origin={origin:?}"
                        );
                        assert_eq!(canonical.equal_cost_destination_count(origin), Some(1));
                        assert_eq!(reflected.equal_cost_destination_count(mirrored_origin), Some(1));
                    }
                    (left, right) => panic!(
                        "reachability changed under pure reflection: pattern={pattern} origin={origin:?} mirrored_origin={mirrored_origin:?} canonical={left:?} reflected={right:?}"
                    ),
                }
                comparisons += 1;
            }
        }
    }

    println!("audit_v4_m9_route_reflection_comparisons={comparisons}");
    assert_eq!(comparisons, 64 * u64::from(WIDTH) * u64::from(HEIGHT));
}
