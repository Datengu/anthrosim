from pathlib import Path

path = Path(__file__).resolve().parent / "issue-207-bootstrap.py"
text = path.read_text(encoding="utf-8")
replacements = {
    "temporary resource household count mismatch: ledger {ledger}, expected {expected}":
        "temporary resource ledger has {ledger} households but expected {expected}",
    "usize::from(larger_group_count > 0)":
        "if larger_group_count > 0 { 1 } else { 0 }",
    "usize::from(group_index < larger_group_count)":
        "if group_index < larger_group_count { 1 } else { 0 }",
    '"""        InvalidHousehold {\\n            household: HouseholdId,\\n        },"""':
        '"""    InvalidHousehold {\\n        household: HouseholdId,\\n    },"""',
    '"""        InvalidHousehold { household: HouseholdId },"""':
        '"""    InvalidHousehold { household: HouseholdId },"""',
}
for old, new in replacements.items():
    if old not in text:
        raise RuntimeError(f"preflight anchor not found: {old}")
    text = text.replace(old, new)

# The core Simulation host has a blank line before `match outcome`; the spatial host does not.
# Patch only the first bootstrap occurrence so each replacement remains exact to live main.
old = "            self.record_metric_snapshot();\\n            match outcome {"
new = "            self.record_metric_snapshot();\\n\\n            match outcome {"
if old not in text:
    raise RuntimeError("simulation annual-loop bootstrap anchor not found")
text = text.replace(old, new, 1)

path.write_text(text, encoding="utf-8")
print("issue 207 bootstrap preflight complete")
