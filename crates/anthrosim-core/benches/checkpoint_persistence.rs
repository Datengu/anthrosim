use std::hint::black_box;

use anthrosim_core::{
    ExperimentConfig, PopulationConfig, Simulation, SimulationCheckpoint, WorldConfig,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn checkpoint_fixture() -> SimulationCheckpoint {
    let config = ExperimentConfig::new(42, 25)
        .with_world(WorldConfig::new(64, 64))
        .with_population(PopulationConfig::new(10_000).with_max_person_records(1_000_000));
    Simulation::new(config)
        .expect("benchmark config should initialize")
        .checkpoint_at_year(10)
        .expect("benchmark checkpoint should be reachable")
}

fn benchmark_checkpoint_persistence(c: &mut Criterion) {
    let checkpoint = checkpoint_fixture();
    let bytes = serde_json::to_vec(&checkpoint).expect("checkpoint serialization should succeed");
    eprintln!("m5_checkpoint_json_bytes={}", bytes.len());

    c.bench_function("m5_checkpoint_json_serialize_10k_year10", |b| {
        b.iter(|| {
            black_box(
                serde_json::to_vec(black_box(&checkpoint))
                    .expect("checkpoint serialization should succeed"),
            )
        });
    });

    c.bench_function("m5_checkpoint_json_deserialize_10k_year10", |b| {
        b.iter(|| {
            black_box(
                serde_json::from_slice::<SimulationCheckpoint>(black_box(&bytes))
                    .expect("checkpoint deserialization should succeed"),
            )
        });
    });
}

criterion_group!(benches, benchmark_checkpoint_persistence);
criterion_main!(benches);
