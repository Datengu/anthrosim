from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match for {old!r}, found {count}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "docs/scientific-model.md",
    "**Status:** working specification for the v0.3.4 release baseline / current model semantics v25 (immutable v0.3.3 release baseline: v21)",
    "**Status:** working specification for the post-v0.3.4 Audit-v4 remediation line / current model semantics v26 (immutable v0.3.4 release baseline: v25; immutable v0.3.3 release baseline: v21)",
)
replace_once(
    "docs/research/odd.md",
    "**AnthroSim baseline:** v0.3.4 release baseline / current model semantics v25 (immutable v0.3.3 release baseline: v21)",
    "**AnthroSim baseline:** post-v0.3.4 Audit-v4 remediation line / current model semantics v26 (immutable v0.3.4 release baseline: v25; immutable v0.3.3 release baseline: v21)",
)
replace_once(
    "docs/research/odd-d.md",
    "**AnthroSim baseline:** completed M9 / v0.3.4 release baseline / current model semantics v25 (immutable v0.3.3 release baseline: v21)",
    "**AnthroSim baseline:** completed M9 / post-v0.3.4 Audit-v4 remediation line / current model semantics v26 (immutable v0.3.4 release baseline: v25; immutable v0.3.3 release baseline: v21)",
)
replace_once(
    "docs/research/trace.md",
    "**AnthroSim baseline:** v0.3.4 / current model semantics v25 / Scientific Audit v3 remediation complete (immutable v0.3.3 release baseline: v21)",
    "**AnthroSim baseline:** post-v0.3.4 Audit-v4 remediation line / current model semantics v26 / Scientific Audit v4 remediation in progress (immutable v0.3.4 release baseline: v25; immutable v0.3.3 release baseline: v21)",
)

path = Path("scripts/test-current-model-semantics-docs.py")
text = path.read_text(encoding="utf-8")
text = text.replace(
    'V034_SEMANTICS_ID = "anthrosim-model-semantics-v25"\nV034_SHORT = "v25"\n',
    'V034_SEMANTICS_ID = "anthrosim-model-semantics-v25"\nV034_SHORT = "v25"\nCURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v26"\nCURRENT_SHORT = "v26"\n',
    1,
)
text = text.replace(
    '    if current_id != V034_SEMANTICS_ID:\n        raise AssertionError(\n            f"v0.3.4 living-release guard expects {V034_SEMANTICS_ID}, got {current_id}"\n        )\n\n    current_phrase = f"current model semantics {current_short}"\n    release_phrase = f"immutable v0.3.3 release baseline: {V033_SHORT}"\n',
    '    if current_id != CURRENT_SEMANTICS_ID:\n        raise AssertionError(\n            f"post-v0.3.4 remediation guard expects {CURRENT_SEMANTICS_ID}, got {current_id}"\n        )\n    if current_short != CURRENT_SHORT:\n        raise AssertionError(f"current short semantics should be {CURRENT_SHORT}, got {current_short}")\n\n    current_phrase = f"current model semantics {current_short}"\n    release_phrase = f"immutable v0.3.4 release baseline: {V034_SHORT}"\n    prior_release_phrase = f"immutable v0.3.3 release baseline: {V033_SHORT}"\n',
    1,
)
text = text.replace(
    '        if release_phrase not in text:\n            raise AssertionError(\n                f"{path.relative_to(ROOT)} does not distinguish the immutable v0.3.3 "\n                f"release baseline ({V033_SHORT}) from the v0.3.4/current line"\n            )\n',
    '        if release_phrase not in text:\n            raise AssertionError(\n                f"{path.relative_to(ROOT)} does not distinguish the immutable v0.3.4 "\n                f"release baseline ({V034_SHORT}) from the current remediation line"\n            )\n        if prior_release_phrase not in text:\n            raise AssertionError(\n                f"{path.relative_to(ROOT)} does not preserve the immutable v0.3.3 "\n                f"release baseline ({V033_SHORT}) distinction"\n            )\n',
    1,
)
if text.count('CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v26"') != 1:
    raise SystemExit("failed to update current model-semantics guard")
path.write_text(text, encoding="utf-8")

status = Path("docs/research/audit-v4/STATUS.md")
text = status.read_text(encoding="utf-8")
text = text.replace(
    "| Repair state | **discovery complete; all AV4 findings remain open and unrepaired; remediation may begin after this ledger closure is merged** |",
    "| Repair state | **post-discovery remediation in progress; AV4-001/#486 has production PR #553 under exact-head validation; all findings remain open until independent merged-main re-verification** |",
    1,
)
text = text.replace(
    "| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #486 |",
    "| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **remediation in progress; production PR #553; independent post-merge re-verification required** | #486 |",
    1,
)
marker = "## Current discovery handoff\n"
insert = (
    "## Post-discovery remediation state\n\n"
    "- **AV4-001/#486:** production repair is under review in PR #553. The proposed repair removes canonical PersonId record order from annual fertility RNG assignment using a relabelling-invariant scientific-role ordering, while preserving the named fertility stream and exact RNG-position accounting.\n"
    "- The causal same-seed assignment change advances the current remediation line to `anthrosim-model-semantics-v26`; immutable discovery target `v0.3.4` remains `anthrosim-model-semantics-v25`.\n"
    "- #486 remains open until PR #553 is merged from an exact green head and the original Audit-v4 adversary is independently rerun against merged `main` in a separate evidence PR.\n\n"
    "## Current discovery handoff\n"
)
if text.count(marker) != 1:
    raise SystemExit("STATUS remediation insertion marker mismatch")
text = text.replace(marker, insert, 1)
status.write_text(text, encoding="utf-8")
