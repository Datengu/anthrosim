use std::hint::black_box;

use anthrosim_core::{World, WorldConfig};
use criterion::{Criterion, criterion_group, criterion_main};

fn world_generation(c: &mut Criterion) {
    c.bench_function("generate_synthetic_world_256x256", |b| {
        b.iter(|| {
            let config = WorldConfig::new(black_box(256), black_box(256));
            black_box(World::generate(
                config,
                anthrosim_core::rng::RngFactory::new(black_box(1_847_291)),
            ))
        });
    });
}

criterion_group!(benches, world_generation);
criterion_main!(benches);
