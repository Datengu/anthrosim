from pathlib import Path

# The legacy combined resolver remains useful to its unit tests, but production now calls the
# trigger-level helper after independent cause-specific trigger assignment.
p = Path('crates/anthrosim-core/src/mortality.rs')
text = p.read_text(encoding='utf-8')
needle = 'pub(crate) fn resolve_two_cause_competing_mortality(\n'
if '#[cfg(test)]\n' + needle not in text:
    if needle not in text:
        raise SystemExit('combined mortality resolver not found')
    text = text.replace(needle, '#[cfg(test)]\n' + needle, 1)
p.write_text(text, encoding='utf-8')

p = Path('crates/anthrosim-core/src/resources.rs')
text = p.read_text(encoding='utf-8')
text = text.replace(
    '        probability_fraction_per_million_ceil, resolve_two_cause_competing_mortality,\n'
    '        resolve_two_cause_competing_mortality_from_triggers,\n',
    '        probability_fraction_per_million_ceil,\n'
    '        resolve_two_cause_competing_mortality_from_triggers,\n',
    1,
)
p.write_text(text, encoding='utf-8')

# The three standards-facing header lines already used Markdown hard-break whitespace. Because the
# semantics-label substitution touches those lines, remove the trailing spaces rather than creating
# a new diff-check violation. This changes no scientific prose.
for name in ['docs/research/odd.md', 'docs/research/odd-d.md', 'docs/research/trace.md']:
    p = Path(name)
    lines = p.read_text(encoding='utf-8').splitlines()
    lines = [line.rstrip() if 'current model semantics v27' in line else line for line in lines]
    p.write_text('\n'.join(lines) + '\n', encoding='utf-8')
