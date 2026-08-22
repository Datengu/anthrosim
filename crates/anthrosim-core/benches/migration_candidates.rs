use std::hint::black_box;

use anthrosim_core::{World, WorldConfig, bounded_candidate_cells};
use criterion::{Criterion, criterion_group, criterion_main};

fn migration_candidate_lookup(c: &mut Criterion) {
    let world = World::generate(
        WorldConfig::new(128, 128),
        anthrosim_core::rng::RngFactory::new(7),
    )
    .unwrap();
    let origin = world.cell_id(64, 64).unwrap();
    c.bench_function("m4_candidate_lookup_radius_3", |b| {
        b.iter(|| {
            black_box(bounded_candidate_cells(
                &world,
                black_box(origin),
                black_box(3),
            ))
        })
    });
}

criterion_group!(benches, migration_candidate_lookup);
criterion_main!(benches);
