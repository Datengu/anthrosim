use anthrosim_core::{
    EventKind, EventLog, EventProvenance, ExperimentConfig, MetricProvenance, MetricSeries,
    PopulationConfig, ResumeLineage, Simulation, SimulationCheckpoint, SimulationError,
    WorldConfig,
};

fn experiment_with_duration(duration_years: u64) -> ExperimentConfig {
    ExperimentConfig::new(7_777, duration_years)
        .with_world(WorldConfig::new(24, 24))
        .with_population(PopulationConfig::new(1_500).with_max_person_records(100_000))
}

fn experiment() -> ExperimentConfig {
    experiment_with_duration(8)
}

#[test]
fn json_round_trip_checkpoint_resumes_to_identical_authoritative_final_state() {
    let config = experiment();
    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(3)
        .unwrap();
    let source_digest = checkpoint.state_digest64;
    let source_continuation_digest = checkpoint.continuation_digest64;
    assert!(checkpoint.continuation_identity_is_valid());
    let json = serde_json::to_string_pretty(&checkpoint).unwrap();
    let restored: SimulationCheckpoint = serde_json::from_str(&json).unwrap();
    assert!(restored.continuation_identity_is_valid());
    let resumed = Simulation::from_checkpoint(restored)
        .unwrap()
        .run_recorded()
        .unwrap();

    let mut resumed_manifest_without_lineage = resumed.manifest.clone();
    resumed_manifest_without_lineage.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_manifest_without_lineage, uninterrupted.manifest);

    let mut resumed_checkpoint_without_lineage = resumed.checkpoint.clone();
    resumed_checkpoint_without_lineage.resume_lineage = ResumeLineage::new();
    resumed_checkpoint_without_lineage =
        resumed_checkpoint_without_lineage.seal_continuation_identity();
    assert_eq!(resumed_checkpoint_without_lineage, uninterrupted.checkpoint);

    assert_eq!(resumed.manifest.resume_lineage.boundaries.len(), 1);
    assert_eq!(resumed.checkpoint.resume_lineage.boundaries.len(), 1);
    assert_eq!(
        resumed.manifest.resume_lineage.boundaries[0].source_state_digest64,
        source_digest
    );
    assert_eq!(
        resumed.manifest.resume_lineage.boundaries[0].source_continuation_digest64,
        source_continuation_digest
    );
    assert!(resumed.checkpoint.continuation_identity_is_valid());
    assert_eq!(
        resumed.manifest.resume_lineage,
        resumed.checkpoint.resume_lineage
    );
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

#[test]
fn year_zero_checkpoint_resume_preserves_exact_authoritative_output() {
    let config = experiment_with_duration(2);
    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    assert_eq!(checkpoint.time.days(), 0);
    assert!(checkpoint.metrics.snapshots.is_empty());
    checkpoint.validate_invariants().unwrap();

    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    let uninterrupted_days = uninterrupted
        .metrics()
        .snapshots
        .iter()
        .map(|snapshot| snapshot.day)
        .collect::<Vec<_>>();
    let resumed_days = resumed
        .metrics()
        .snapshots
        .iter()
        .map(|snapshot| snapshot.day)
        .collect::<Vec<_>>();
    assert_eq!(uninterrupted_days, vec![365, 730]);
    assert_eq!(resumed_days, uninterrupted_days);
    assert_eq!(resumed.events(), uninterrupted.events());

    let mut resumed_manifest = resumed.manifest.clone();
    resumed_manifest.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_manifest, uninterrupted.manifest);

    let mut resumed_checkpoint = resumed.checkpoint.clone();
    resumed_checkpoint.resume_lineage = ResumeLineage::new();
    resumed_checkpoint = resumed_checkpoint.seal_continuation_identity();
    assert_eq!(resumed_checkpoint, uninterrupted.checkpoint);
}

#[test]
fn year_zero_checkpoint_invariants_still_validate_metric_series_identity() {
    let mut checkpoint = Simulation::new(experiment_with_duration(2))
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    assert!(checkpoint.metrics.snapshots.is_empty());
    checkpoint.metrics.schema_version += 1;
    let error = checkpoint.validate_invariants().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("metric series schema/cadence is invalid")
    );
}

#[test]
fn legacy_nonterminal_year_zero_metric_snapshot_is_rejected_after_reseal() {
    let mut legacy = Simulation::new(experiment_with_duration(2))
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    let terminal_zero = Simulation::new(experiment_with_duration(0))
        .unwrap()
        .run_recorded()
        .unwrap();
    assert_eq!(terminal_zero.metrics().snapshots.len(), 1);
    assert_eq!(terminal_zero.metrics().snapshots[0].day, 0);

    legacy
        .metrics
        .snapshots
        .push(terminal_zero.metrics().snapshots[0].clone());
    legacy = legacy.seal_continuation_identity();
    assert!(legacy.validate_invariants().is_err());
    assert!(matches!(
        Simulation::from_checkpoint(legacy),
        Err(SimulationError::CheckpointInitialMetricHistoryNotEmpty { snapshot_count: 1 })
    ));
}
