use anthrosim_core::{ExperimentConfig, Simulation};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn empty_run(c: &mut Criterion) {
    c.bench_function("milestone0_empty_10k_year_run", |b| {
        b.iter(|| {
            let config = ExperimentConfig::new(black_box(1_847_291), black_box(10_000));
            black_box(Simulation::new(config).run())
        });
    });
}

criterion_group!(benches, empty_run);
criterion_main!(benches);
