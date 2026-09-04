# Fail-closed helper for synchronizing the M9 temporary-event schema invariant.
from pathlib import Path

mobility = Path("crates/anthrosim-core/src/temporary_mobility.rs")
invariants = Path("crates/anthrosim-core/src/invariants.rs")

mobility_text = mobility.read_text()
old_const = "const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 3;"
new_const = "pub(crate) const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 3;"
if mobility_text.count(old_const) != 1:
    raise SystemExit(f"temporary_mobility.rs: expected one private event-schema constant, found {mobility_text.count(old_const)}")
mobility.write_text(mobility_text.replace(old_const, new_const, 1))

text = invariants.read_text()
old_import = "    temporary_history::validate_temporary_mobility_history,\n"
new_import = (
    "    temporary_history::validate_temporary_mobility_history,\n"
    "    temporary_mobility::TEMPORARY_EVENT_SCHEMA_VERSION,\n"
)
if text.count(old_import) != 1:
    raise SystemExit(f"invariants.rs: expected one temporary-history import target, found {text.count(old_import)}")
text = text.replace(old_import, new_import, 1)

needle = "*event_schema_version != 2"
count = text.count(needle)
if count != 5:
    raise SystemExit(f"invariants.rs: expected five stale temporary-event schema checks, found {count}")
text = text.replace(needle, "*event_schema_version != TEMPORARY_EVENT_SCHEMA_VERSION")
invariants.write_text(text)
