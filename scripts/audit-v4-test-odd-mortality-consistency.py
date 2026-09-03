#!/usr/bin/env python3
"""Fresh Audit-v4 Area M ODD/current-semantics consistency adversary."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
odd = (ROOT / "docs" / "research" / "odd.md").read_text(encoding="utf-8")
model = (ROOT / "docs" / "scientific-model.md").read_text(encoding="utf-8")
demography = (ROOT / "crates" / "anthrosim-core" / "src" / "demography.rs").read_text(encoding="utf-8")

# Positive controls establish that the frozen target is the v25 current-facing
# documentation/source set and that the authoritative executable/normative model
# agree on elapsed competing mortality + year-end fertility/parentage finalization.
assert "current model semantics v25" in odd
assert "current model semantics v25" in model
assert "partition that same annual risk over elapsed" in demography
assert "before calling the annual\n/// fertility/parentage finalizer below" in demography
assert "background mortality has already been resolved over the elapsed year" in model
assert "year-end M2 stage then performs fertility/parentage only" in model

stale_phrases = [
    "M2 mortality is drawn before fertility",
    "execute the M2 discrete transition for `[t-365,t)`: use interval-start age bands, draw mortality, then evaluate conditional fertility/parentage among survivors",
]
stale_present = [phrase for phrase in stale_phrases if phrase in odd]

# A current ODD may still describe the annual probability parameter and interval-start
# age selection, but must not describe background mortality as being executed/drawn at
# the year-end M2 boundary after v15 competing-risk semantics.
print(f"audit_v4_area_m_odd_semantics_v25={('current model semantics v25' in odd)}")
print(f"audit_v4_area_m_normative_competing_mortality=True")
print(f"audit_v4_area_m_stale_annual_mortality_claims={len(stale_present)}")
for index, phrase in enumerate(stale_present, start=1):
    print(f"audit_v4_area_m_stale_claim_{index}={phrase}")

assert not stale_present, (
    "living v0.3.4/v25 ODD still describes pre-v15 annual-boundary background "
    "mortality execution even though authoritative hosts partition that annual risk "
    "over M3 intervals and year-end M2 only finalizes fertility/parentage"
)
