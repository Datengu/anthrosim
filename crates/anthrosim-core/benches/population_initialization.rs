use anthrosim_core::{Population, PopulationConfig, World, WorldConfig, rng::RngFactory};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn population_initialization(c: &mut Criterion) {
    let world = World::generate(WorldConfig::new(128, 128), RngFactory::new(91_337)).unwrap();
    let mut group = c.benchmark_group("population_initialization");
    group.sample_size(20);

    for population_size in [10_000_u32, 100_000, 1_000_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(population_size),
            &population_size,
            |b, &population_size| {
                b.iter(|| {
                    black_box(
                        Population::initialize(
                            PopulationConfig::new(black_box(population_size)),
                            &world,
                            RngFactory::new(black_box(91_337)),
                        )
                        .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, population_initialization);
criterion_main!(benches);
