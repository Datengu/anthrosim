use anthrosim_core::{
    EventKind, EventLog, EventProvenance, ExperimentConfig, MetricProvenance, MetricSeries,
    PopulationConfig, Simulation, SimulationCheckpoint, WorldConfig,
};

fn experiment() -> ExperimentConfig {
    ExperimentConfig::new(7_777, 8)
        .with_world(WorldConfig::new(24, 24))
        .with_population(PopulationConfig::new(1_500).with_max_person_records(100_000))
}

#[test]
fn json_round_trip_checkpoint_resumes_to_identical_final_run() {
    let config = experiment();
    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(3)
        .unwrap();
    let json = serde_json::to_string_pretty(&checkpoint).unwrap();
    let restored: SimulationCheckpoint = serde_json::from_str(&json).unwrap();
    let resumed = Simulation::from_checkpoint(restored)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(resumed.manifest, uninterrupted.manifest);
    assert_eq!(resumed.checkpoint, uninterrupted.checkpoint);
}

#[test]
fn authoritative_events_and_derived_metrics_are_explicit_and_reconcile() {
    let run = Simulation::new(experiment())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        run.events().schema_version,
        EventLog::CURRENT_SCHEMA_VERSION
    );
    assert_eq!(
        run.metrics().schema_version,
        MetricSeries::CURRENT_SCHEMA_VERSION
    );
    assert!(
        run.events()
            .events
            .iter()
            .all(|event| event.provenance == EventProvenance::Authoritative)
    );
    assert!(
        run.metrics()
            .snapshots
            .iter()
            .all(|snapshot| snapshot.provenance == MetricProvenance::Derived)
    );

    let births = run
        .events()
        .events
        .iter()
        .filter(|event| matches!(event.event, EventKind::Birth { .. }))
        .count() as u64;
    let deaths = run
        .events()
        .events
        .iter()
        .filter(|event| matches!(event.event, EventKind::Death { .. }))
        .count() as u64;
    let moves = run
        .events()
        .events
        .iter()
        .filter(|event| matches!(event.event, EventKind::HouseholdMigration { .. }))
        .count() as u64;

    assert_eq!(births, run.manifest.population.births_since_start);
    assert_eq!(deaths, run.manifest.population.deaths_since_start);
    assert_eq!(moves, run.manifest.migration.moves_completed);
    assert_eq!(
        births + deaths + moves,
        run.manifest.statistics.authoritative_event_count
    );
    assert_eq!(
        run.manifest.statistics.authoritative_event_count,
        run.events().events.len() as u64
    );

    let final_metrics = run.metrics().snapshots.last().unwrap();
    assert_eq!(
        final_metrics.population.living_population,
        run.manifest.population.living_population
    );
    assert_eq!(
        final_metrics.resources.unmet_need,
        run.manifest.resources.unmet_need
    );
    assert_eq!(
        final_metrics.migration.moves_completed,
        run.manifest.migration.moves_completed
    );
    assert_eq!(final_metrics.state_digest64, run.manifest.state_digest64);
}
