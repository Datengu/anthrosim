use anthrosim_core::rng::RngFactory;
use anthrosim_core::{World, WorldConfig, bounded_candidate_cells};

fn world(width: u32, height: u32) -> World {
    World::generate(WorldConfig::new(width, height), RngFactory::new(712_001)).unwrap()
}

fn physical_offsets(world: &World, origin_x: u32, origin_y: u32, radius: u16) -> Vec<(i32, i32)> {
    let origin = world.cell_id(origin_x, origin_y).unwrap();
    let mut offsets = bounded_candidate_cells(world, origin, radius)
        .into_iter()
        .map(|cell| {
            let (x, y) = world.coordinates(cell).unwrap();
            (
                i32::try_from(x).unwrap() - i32::try_from(origin_x).unwrap(),
                i32::try_from(y).unwrap() - i32::try_from(origin_y).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    offsets
}

#[test]
fn m4_local_candidate_geometry_is_invariant_to_padding_outside_radius() {
    let baseline = world(5, 5);
    let right_padded = world(8, 5);
    let bottom_padded = world(5, 8);
    let both_padded = world(8, 8);

    let origin = (2, 2);
    let radius = 2;
    let baseline_offsets = physical_offsets(&baseline, origin.0, origin.1, radius);

    assert_eq!(baseline_offsets.len(), 12);
    assert!(baseline_offsets.iter().all(|&(dx, dy)| {
        dx.unsigned_abs() + dy.unsigned_abs() <= u32::from(radius) && (dx != 0 || dy != 0)
    }));

    for (label, padded) in [
        ("right", &right_padded),
        ("bottom", &bottom_padded),
        ("right+bottom", &both_padded),
    ] {
        let padded_offsets = physical_offsets(padded, origin.0, origin.1, radius);
        eprintln!(
            "M4 padding-locality control {label}: baseline={baseline_offsets:?} padded={padded_offsets:?}"
        );
        assert_eq!(
            padded_offsets, baseline_offsets,
            "adding simulation-domain padding strictly outside the M4 candidate radius must not alter the physical local candidate set"
        );
    }
}
