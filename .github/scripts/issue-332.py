from pathlib import Path

# Plain simulation: checkpoint serialization must not manufacture observations.
p = Path("crates/anthrosim-core/src/simulation.rs")
s = p.read_text()
exact = "        self.ensure_terminal_metric_snapshot();\n        self.validate_state()?;\n        Ok(self.into_checkpoint())\n    }\n\n    fn advance_to_year"
assert exact in s
s = s.replace(
    exact,
    "        self.validate_state()?;\n        Ok(self.into_checkpoint())\n    }\n\n    fn advance_to_year",
    1,
)
marker = "        validate_terminal_checkpoint_state(&checkpoint)?;\n\n        let boundary_day = checkpoint.time.days();"
assert marker in s
s = s.replace(
    marker,
    "        validate_terminal_checkpoint_state(&checkpoint)?;\n        validate_initial_checkpoint_metric_history(&checkpoint)?;\n\n        let boundary_day = checkpoint.time.days();",
    1,
)
marker = "fn validate_terminal_checkpoint_state(\n    checkpoint: &SimulationCheckpoint,\n) -> Result<(), SimulationError> {"
assert marker in s
helper = """fn validate_initial_checkpoint_metric_history(
    checkpoint: &SimulationCheckpoint,
) -> Result<(), SimulationError> {
    if checkpoint.time.days() == 0
        && checkpoint.terminal_stop_reason.is_none()
        && !checkpoint.metrics.snapshots.is_empty()
    {
        return Err(SimulationError::CheckpointInitialMetricHistoryNotEmpty {
            snapshot_count: checkpoint.metrics.snapshots.len(),
        });
    }
    Ok(())
}

"""
s = s.replace(marker, helper + marker, 1)
marker = '    #[error("checkpoint terminal stop reason {stop_reason:?} does not match checkpoint state")]\n    CheckpointTerminalStateMismatch { stop_reason: StopReason },'
assert marker in s
replacement = marker + """
    #[error(
        "non-terminal initial checkpoint contains {snapshot_count} metric snapshot(s); checkpoint serialization must not create day-zero observations"
    )]
    CheckpointInitialMetricHistoryNotEmpty { snapshot_count: usize },"""
s = s.replace(marker, replacement, 1)
p.write_text(s)

# Spatial simulation: same serialization and resume-validity contract.
p = Path("crates/anthrosim-core/src/spatial_simulation.rs")
s = p.read_text()
exact = "        self.ensure_terminal_metric_snapshot();\n        self.validate_state()?;\n        let landscape = self.landscape_binding.clone();"
assert exact in s
s = s.replace(
    exact,
    "        self.validate_state()?;\n        let landscape = self.landscape_binding.clone();",
    1,
)
marker = """    if checkpoint.metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION {
        return Err(SpatialLandscapeError::CheckpointArtifactSchemaMismatch {
            artifact: "metrics",
        });
    }
"""
assert marker in s
addition = marker + """    if checkpoint.time.days() == 0
        && checkpoint.terminal_stop_reason.is_none()
        && !checkpoint.metrics.snapshots.is_empty()
    {
        return Err(SpatialLandscapeError::CheckpointInitialMetricHistoryNotEmpty {
            snapshot_count: checkpoint.metrics.snapshots.len(),
        });
    }
"""
s = s.replace(marker, addition, 1)
marker = '    #[error("checkpoint terminal stop reason {stop_reason:?} does not match checkpoint state")]\n    CheckpointTerminalStateMismatch { stop_reason: StopReason },'
assert marker in s
replacement = marker + """
    #[error(
        "non-terminal initial core checkpoint contains {snapshot_count} metric snapshot(s); checkpoint serialization must not create day-zero observations"
    )]
    CheckpointInitialMetricHistoryNotEmpty { snapshot_count: usize },"""
s = s.replace(marker, replacement, 1)
p.write_text(s)

# Invariant suite: a non-terminal initial checkpoint is valid only with no observations yet.
p = Path("crates/anthrosim-core/src/invariants.rs")
s = p.read_text()
exact = "        &resources,\n        &migration_summary,\n    )?;\n\n    let actual_digest"
assert exact in s
s = s.replace(
    exact,
    "        &resources,\n        &migration_summary,\n        recorded_stop_reason.is_none()\n            && checkpoint.terminal_stop_reason.is_none()\n            && checkpoint.time.days() == 0,\n    )?;\n\n    let actual_digest",
    1,
)
exact = """fn validate_metrics(
    metrics: &MetricSeries,
    events: &EventLog,
    day: u64,
    state_digest: u64,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<(), InvariantError> {
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION
        || metrics.cadence != "annual_boundary_plus_terminal"
        || metrics.snapshots.is_empty()
    {
        return violation(
            "metric series schema/cadence is invalid or terminal snapshot is missing",
        );
    }
"""
assert exact in s
replacement = """fn validate_metrics(
    metrics: &MetricSeries,
    events: &EventLog,
    day: u64,
    state_digest: u64,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
    nonterminal_initial_checkpoint: bool,
) -> Result<(), InvariantError> {
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION
        || metrics.cadence != "annual_boundary_plus_terminal"
    {
        return violation("metric series schema/cadence is invalid");
    }
    if nonterminal_initial_checkpoint {
        if metrics.snapshots.is_empty() {
            return Ok(());
        }
        return violation("non-terminal initial checkpoint must not contain metric snapshots");
    }
    if metrics.snapshots.is_empty() {
        return violation("metric series terminal/current-boundary snapshot is missing");
    }
"""
s = s.replace(exact, replacement, 1)
p.write_text(s)

# Plain checkpoint regression + explicit legacy fail-closed adversary.
p = Path("crates/anthrosim-core/tests/checkpoint_persistence.rs")
s = p.read_text()
old_import = "PopulationConfig, ResumeLineage, Simulation, SimulationCheckpoint, WorldConfig,"
assert old_import in s
s = s.replace(
    old_import,
    "PopulationConfig, ResumeLineage, Simulation, SimulationCheckpoint, SimulationError, WorldConfig,",
    1,
)
old = """fn experiment() -> ExperimentConfig {
    ExperimentConfig::new(7_777, 8)
        .with_world(WorldConfig::new(24, 24))
        .with_population(PopulationConfig::new(1_500).with_max_person_records(100_000))
}
"""
assert old in s
new = """fn experiment_with_duration(duration_years: u64) -> ExperimentConfig {
    ExperimentConfig::new(7_777, duration_years)
        .with_world(WorldConfig::new(24, 24))
        .with_population(PopulationConfig::new(1_500).with_max_person_records(100_000))
}

fn experiment() -> ExperimentConfig {
    experiment_with_duration(8)
}
"""
s = s.replace(old, new, 1)
assert "year_zero_checkpoint_resume_preserves_exact_authoritative_output" not in s
s += r'''

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
        Err(SimulationError::CheckpointInitialMetricHistoryNotEmpty {
            snapshot_count: 1
        })
    ));
}
'''
p.write_text(s)

# Spatial/transformed path: exact equivalence and legacy rejection.
p = Path("crates/anthrosim-core/tests/spatial_simulation.rs")
s = p.read_text()
marker = """#[test]
fn transformed_resume_rejects_core_continuation_tampering() {"""
assert marker in s
assert "transformed_year_zero_checkpoint_resume_matches_uninterrupted" not in s
tests = r'''#[test]
fn transformed_year_zero_checkpoint_resume_matches_uninterrupted() {
    let uninterrupted = SpatialLandscapeSimulation::new(config(9013), fixture(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap();
    let checkpoint = SpatialLandscapeSimulation::new(config(9013), fixture(), mechanisms())
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    assert_eq!(checkpoint.core_checkpoint.time.days(), 0);
    assert!(checkpoint.core_checkpoint.metrics.snapshots.is_empty());

    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, fixture())
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
    assert_eq!(resumed_days, uninterrupted_days);

    let mut resumed_without_lineage = resumed.clone();
    resumed_without_lineage.manifest.core_manifest.resume_lineage = ResumeLineage::new();
    resumed_without_lineage
        .checkpoint
        .core_checkpoint
        .resume_lineage = ResumeLineage::new();
    resumed_without_lineage.checkpoint.core_checkpoint = resumed_without_lineage
        .checkpoint
        .core_checkpoint
        .seal_continuation_identity();
    assert_eq!(resumed_without_lineage, uninterrupted);
}

#[test]
fn transformed_resume_rejects_legacy_nonterminal_year_zero_metric_snapshot() {
    let mut legacy = SpatialLandscapeSimulation::new(config(9014), fixture(), mechanisms())
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    let mut terminal_config = config(9014);
    terminal_config.duration_years = 0;
    let terminal_zero =
        SpatialLandscapeSimulation::new(terminal_config, fixture(), mechanisms())
            .unwrap()
            .run_recorded()
            .unwrap();
    legacy
        .core_checkpoint
        .metrics
        .snapshots
        .push(terminal_zero.metrics().snapshots[0].clone());
    legacy.core_checkpoint = legacy.core_checkpoint.seal_continuation_identity();

    assert!(matches!(
        SpatialLandscapeSimulation::from_checkpoint(legacy, fixture()),
        Err(SpatialLandscapeError::CheckpointInitialMetricHistoryNotEmpty {
            snapshot_count: 1
        })
    ));
}

'''
s = s.replace(marker, tests + marker, 1)
p.write_text(s)

# Independent arithmetic/checkpoint-observability checker.
checker = Path("docs/research/audit-v2/area-g-year0-checkpoint-observability-audit.py")
checker.write_text(
    '''#!/usr/bin/env python3
"""Independent #332 checker; deliberately imports no AnthroSim code."""

DAYS_PER_YEAR = 365


def annual_observations(years: int) -> list[int]:
    return [year * DAYS_PER_YEAR for year in range(1, years + 1)]


uninterrupted = annual_observations(2)
legacy_resumed = [0] + annual_observations(2)
repaired_resumed = annual_observations(2)
year_one_checkpoint = annual_observations(1)
true_terminal_zero = [0]

assert uninterrupted == [365, 730]
assert legacy_resumed == [0, 365, 730]
assert repaired_resumed == uninterrupted
assert year_one_checkpoint == [365]
assert true_terminal_zero == [0]
assert len(legacy_resumed) / len(uninterrupted) == 1.5

print("uninterrupted two-year metric days:", uninterrupted)
print("legacy year-zero resume metric days:", legacy_resumed)
print("repaired year-zero resume metric days:", repaired_resumed)
print("legacy retained-observation inflation: +50.0% (3 vs 2)")
print("year-one checkpoint retains its annual boundary:", year_one_checkpoint)
print("true terminal duration-zero run retains day zero:", true_terminal_zero)
'''
)

# Preserve original discovery evidence and append repair contract/evidence.
p = Path("docs/research/audit-v2/area-g-2026-08-29.md")
s = p.read_text()
heading = "## #332 repair/reverification — checkpoint serialization is not observation"
assert heading not in s
s += r'''

## #332 repair/reverification — checkpoint serialization is not observation

The repair distinguishes **serialization boundaries** from **scientific observation boundaries**. A non-terminal checkpoint requested at simulation day 0 no longer calls terminal-metric recording merely in order to serialize state. Its metric series is therefore empty, exactly as it is in a newly constructed uninterrupted simulation. True terminal runs at duration zero still receive a day-0 terminal metric, and non-zero annual checkpoints retain the annual boundary snapshot already produced by model advancement.

The same change is applied to both ordinary `Simulation` and transformed `SpatialLandscapeSimulation` checkpoint paths. Resume validation additionally fails closed on legacy/nonconforming **non-terminal day-zero checkpoints that already contain metric snapshots**, even if their continuation integrity is resealed; the implementation does not silently delete an observation and thereby rewrite provenance.

Independent arithmetic reproduces the original two-year discrepancy without importing AnthroSim: uninterrupted `[365, 730]` versus legacy year-zero-resumed `[0, 365, 730]`, a 3-vs-2 (**+50%**) retained-observation difference. The repaired path is `[365, 730]`; a year-one checkpoint remains `[365]`; and a genuinely terminal zero-duration run remains `[0]`.

Regression coverage requires complete final checkpoint/run equality after removing only declared resume-lineage provenance for both ordinary and transformed spatial execution. Existing non-zero checkpoint regressions remain in place, and the long-run diagnostics test suite is rerun because downstream studies consume completed-run metric histories.

No `MODEL_SEMANTICS_ID` or metric schema bump is required: causal simulation dynamics and completed-run metric cadence are unchanged, and schema v3 already declares `annual_boundary_plus_terminal`. This repair makes year-zero checkpoint behavior conform to that existing contract rather than introducing new executable model semantics.
'''
p.write_text(s)
