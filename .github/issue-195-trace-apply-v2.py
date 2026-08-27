from pathlib import Path

source_path = Path('.github/issue-195-trace-apply.py')
source = source_path.read_text()
old = '''if text.count(field_anchor) != 1:\n    raise SystemExit("expected trace construction weight fields exactly once")\ntext = text.replace(field_anchor, field_replacement, 1)'''
new = '''trace_start = text.index("self.recorded_decision_traces.push(MigrationDecisionTrace {")\nevent_start = text.index("events.push_authoritative", trace_start)\ntrace_segment = text[trace_start:event_start]\nif trace_segment.count(field_anchor) != 1:\n    raise SystemExit("expected trace construction weight fields exactly once inside retained trace")\ntrace_segment = trace_segment.replace(field_anchor, field_replacement, 1)\ntext = text[:trace_start] + trace_segment + text[event_start:]'''
if source.count(old) != 1:
    raise SystemExit('expected trace-script ambiguity block exactly once')
exec(compile(source.replace(old, new), str(source_path), 'exec'))

# The integration test hands ExperimentConfig into Simulation::new. Preserve the one threshold
# needed by the post-run audit assertions before that ownership transfer.
test = Path('crates/anthrosim-core/tests/migration_behavior.rs')
test_text = test.read_text()
old_setup = '''    let radius = config.migration.candidate_radius_cells;\n    let manifest = Simulation::new(config).unwrap().run().unwrap();'''
new_setup = '''    let radius = config.migration.candidate_radius_cells;\n    let minimum_utility_improvement = config.migration.minimum_utility_improvement;\n    let manifest = Simulation::new(config).unwrap().run().unwrap();'''
if test_text.count(old_setup) != 1:
    raise SystemExit('expected migration integration setup exactly once')
test_text = test_text.replace(old_setup, new_setup, 1)
old_use = 'i32::try_from(config.migration.minimum_utility_improvement).unwrap_or(i32::MAX)'
new_use = 'i32::try_from(minimum_utility_improvement).unwrap_or(i32::MAX)'
if test_text.count(old_use) != 1:
    raise SystemExit('expected moved config threshold use exactly once')
test.write_text(test_text.replace(old_use, new_use, 1))