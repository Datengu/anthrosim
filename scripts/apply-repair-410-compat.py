#!/usr/bin/env python3
from pathlib import Path

path = Path('scripts/research-monte-carlo-sufficiency.py')
text = path.read_text()
text = text.replace('DIAGNOSTIC_SCHEMA = 3', 'DIAGNOSTIC_SCHEMA = 2')
text = text.replace(
    '''        "seedIdentities": groups[0]["seeds"] if len(groups) == 1 else None,
        "groupSeedIdentities": {group["id"]: group["seeds"] for group in groups},
        "pairingSemantics": plan["pairing"],
        "groupIds": [group["id"] for group in groups],
''',
    '''        "seedIdentities": groups[0]["seeds"],
        "groupIds": [group["id"] for group in groups],
''',
)
needle = '''    if lineage is not None:
        result["studyLineage"] = lineage
'''
replacement = '''    if plan["estimand"]["kind"] == "difference_in_means":
        result["groupSeedIdentities"] = {group["id"]: group["seeds"] for group in groups}
        result["pairingSemantics"] = plan["pairing"]
    if lineage is not None:
        result["studyLineage"] = lineage
'''
assert needle in text
text = text.replace(needle, replacement)
path.write_text(text)

path = Path('scripts/test-research-monte-carlo-sufficiency.py')
text = path.read_text().replace(
    '    assert result["seedIdentities"] is None\n',
    '    assert result["seedIdentities"] == left_seeds\n',
)
path.write_text(text)

path = Path('docs/research/monte-carlo-sufficiency-v1.md')
text = path.read_text().replace(
    'The emitted diagnostic uses schema v3 while retaining precision-plan schema/identity v1. Schema v3 records actual pairing semantics and per-group seed identities. This analysis-layer change does not change `MODEL_SEMANTICS_ID`.',
    'The emitted diagnostic remains schema v2 and retains precision-plan schema/identity v1. For `difference_in_means`, v2 diagnostics additionally record `pairingSemantics=independent` and the exact per-group seed identities so the independent-arm contract is machine-visible. Existing diagnostic shapes for the other estimands remain unchanged. This analysis-layer change does not change `MODEL_SEMANTICS_ID`.',
)
path.write_text(text)
