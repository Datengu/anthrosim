from pathlib import Path

source_path = Path('.github/issue-195-trace-apply.py')
source = source_path.read_text()
old = '''if text.count(field_anchor) != 1:\n    raise SystemExit("expected trace construction weight fields exactly once")\ntext = text.replace(field_anchor, field_replacement, 1)'''
new = '''trace_start = text.index("self.recorded_decision_traces.push(MigrationDecisionTrace {")\nevent_start = text.index("events.push_authoritative", trace_start)\ntrace_segment = text[trace_start:event_start]\nif trace_segment.count(field_anchor) != 1:\n    raise SystemExit("expected trace construction weight fields exactly once inside retained trace")\ntrace_segment = trace_segment.replace(field_anchor, field_replacement, 1)\ntext = text[:trace_start] + trace_segment + text[event_start:]'''
if source.count(old) != 1:
    raise SystemExit('expected trace-script ambiguity block exactly once')
# Rerun after correcting the scientific-model anchor in the base patch script.
exec(compile(source.replace(old, new), str(source_path), 'exec'))