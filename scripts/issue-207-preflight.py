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
}
for old, new in replacements.items():
    if old not in text:
        raise RuntimeError(f"preflight anchor not found: {old}")
    text = text.replace(old, new)
path.write_text(text, encoding="utf-8")
print("issue 207 bootstrap preflight complete")
