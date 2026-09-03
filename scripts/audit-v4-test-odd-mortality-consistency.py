#!/usr/bin/env python3
"""Fresh Audit-v4 Area M ODD/ODD+D current-semantics consistency adversary."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
odd = (ROOT / "docs" / "research" / "odd.md").read_text(encoding="utf-8")
oddd = (ROOT / "docs" / "research" / "odd-d.md").read_text(encoding="utf-8")
model = (ROOT / "docs" / "scientific-model.md").read_text(encoding="utf-8")
demography = (ROOT / "crates" / "anthrosim-core" / "src" / "demography.rs").read_text(encoding="utf-8")

# Positive controls establish that the frozen target is the v25 current-facing
# documentation/source set and that the authoritative executable/normative model
# agree on elapsed competing mortality + year-end fertility/parentage finalization.
assert "current model semantics v25" in odd
assert "current model semantics v25" in oddd
assert "current model semantics v25" in model
assert "partition that same annual risk over elapsed" in demography
assert "before calling the annual\n/// fertility/parentage finalizer below" in demography
assert "background mortality has already been resolved over the elapsed year" in model
assert "year-end M2 stage then performs fertility/parentage only" in model
assert "order-invariant competition between the M3 condition-mediated cause and the M2 background cause" in model

odd_stale_phrases = [
    "M2 mortality is drawn before fertility",
    "execute the M2 discrete transition for `[t-365,t)`: use interval-start age bands, draw mortality, then evaluate conditional fertility/parentage among survivors",
]
oddd_stale_phrases = [
    "M2 is likewise a coarse annual discrete transition, not continuous reproductive/death decision-making. Its schedule age is read at the start of `[t-365,t)`, mortality has declared priority, and fertility is conditional on surviving that annual transition.",
]
odd_stale = [phrase for phrase in odd_stale_phrases if phrase in odd]
oddd_stale = [phrase for phrase in oddd_stale_phrases if phrase in oddd]

# Current standards-facing docs may describe the annual probability parameter and
# interval-start age selection, but must not describe background mortality as being
# executed/drawn with priority at the year-end M2 boundary after v15 competing risks.
print(f"audit_v4_area_m_odd_semantics_v25={('current model semantics v25' in odd)}")
print(f"audit_v4_area_m_oddd_semantics_v25={('current model semantics v25' in oddd)}")
print("audit_v4_area_m_normative_competing_mortality=True")
print(f"audit_v4_area_m_odd_stale_claims={len(odd_stale)}")
print(f"audit_v4_area_m_oddd_stale_claims={len(oddd_stale)}")
print(f"audit_v4_area_m_total_stale_claims={len(odd_stale) + len(oddd_stale)}")
for index, phrase in enumerate(odd_stale, start=1):
    print(f"audit_v4_area_m_odd_stale_claim_{index}={phrase}")
for index, phrase in enumerate(oddd_stale, start=1):
    print(f"audit_v4_area_m_oddd_stale_claim_{index}={phrase}")

assert not odd_stale and not oddd_stale, (
    "living v0.3.4/v25 ODD/ODD+D still describe pre-v15 annual-boundary background "
    "mortality execution/priority even though authoritative hosts partition that annual risk "
    "over M3 intervals as order-invariant competing mortality and year-end M2 only finalizes "
    "fertility/parentage"
)
