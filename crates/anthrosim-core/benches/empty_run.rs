use std::hint::black_box;

use anthrosim_core::{ExperimentConfig, PopulationConfig, Simulation, WorldConfig};
use criterion::{Criterion, criterion_group, criterion_main};

fn demographic_run(c: &mut Criterion) {
    c.bench_function("m2_10k_people_25_year_demography_run", |b| {
        b.iter(|| {
            let config = ExperimentConfig::new(black_box(1_847_291), black_box(25))
                .with_world(WorldConfig::new(64, 64))
                .with_population(
                    PopulationConfig::new(10_000).with_max_person_records(250_000),
                );
            black_box(Simulation::new(config).unwrap().run().unwrap())
        });
    });
}

criterion_group!(benches, demographic_run);
criterion_main!(benches);
