from pathlib import Path

p = Path("crates/anthrosim-core/src/invariants.rs")
s = p.read_text()

old_call = """    validate_metrics(
        &checkpoint.metrics,
        &checkpoint.events,
        checkpoint.time.days(),
        checkpoint.state_digest64,
        &population,
        &resources,
        &migration_summary,
        recorded_stop_reason.is_none()
            && checkpoint.terminal_stop_reason.is_none()
            && checkpoint.time.days() == 0,
    )?;
"""
assert old_call in s
new_call = """    let nonterminal_initial_checkpoint = recorded_stop_reason.is_none()
        && checkpoint.terminal_stop_reason.is_none()
        && checkpoint.time.days() == 0;
    if nonterminal_initial_checkpoint {
        if !checkpoint.metrics.snapshots.is_empty() {
            return violation("non-terminal initial checkpoint must not contain metric snapshots");
        }
    } else {
        validate_metrics(
            &checkpoint.metrics,
            &checkpoint.events,
            checkpoint.time.days(),
            checkpoint.state_digest64,
            &population,
            &resources,
            &migration_summary,
        )?;
    }
"""
s = s.replace(old_call, new_call, 1)

old_header = """fn validate_metrics(
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
assert old_header in s
new_header = """fn validate_metrics(
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
    {
        return violation("metric series schema/cadence is invalid");
    }
    if metrics.snapshots.is_empty() {
        return violation("metric series terminal/current-boundary snapshot is missing");
    }
"""
s = s.replace(old_header, new_header, 1)
p.write_text(s)
