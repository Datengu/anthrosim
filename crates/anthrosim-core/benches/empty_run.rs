use std::hint::black_box;

use anthrosim_core::{ExperimentConfig, Simulation, WorldConfig};
use criterion::{Criterion, criterion_group, criterion_main};

fn empty_run(c: &mut Criterion) {
    c.bench_function("m1_128x128_10k_year_skeleton_run", |b| {
        b.iter(|| {
            let config = ExperimentConfig::new(black_box(1_847_291), black_box(10_000))
                .with_world(WorldConfig::new(128, 128));
            black_box(Simulation::new(config).unwrap().run())
        });
    });
}

criterion_group!(benches, empty_run);
criterion_main!(benches);
