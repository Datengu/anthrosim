#!/usr/bin/env python3
"""Keep living ODD/ODD+D mortality timing aligned with authoritative semantics."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ODD_PATH = ROOT / "docs" / "research" / "odd.md"
ODDD_PATH = ROOT / "docs" / "research" / "odd-d.md"
MODEL_PATH = ROOT / "docs" / "scientific-model.md"
M2_CONTRACT_PATH = ROOT / "docs" / "research" / "m2-demographic-time-contract-v1.md"

STALE_PHRASES = (
    "M2 mortality is drawn before fertility",
    "execute the M2 discrete transition for `[t-365,t)`: use interval-start age bands, draw mortality, then evaluate conditional fertility/parentage among survivors",
    "mortality has declared priority, and fertility is conditional on surviving that annual transition",
)

REQUIRED_CURRENT_CLAIMS = {
    ODD_PATH: (
        "background mortality is parameterized annually but executed across elapsed M3 intervals as an order-invariant competing risk",
        "year-end M2 stage performs fertility/parentage only after survival through the elapsed year",
    ),
    ODDD_PATH: (
        "annual background-mortality parameter is resolved across elapsed M3 intervals in order-invariant competition with condition-mediated mortality",
        "year-end M2 stage has no separate mortality priority",
    ),
}


def check() -> None:
    odd = ODD_PATH.read_text(encoding="utf-8")
    oddd = ODDD_PATH.read_text(encoding="utf-8")
    model = MODEL_PATH.read_text(encoding="utf-8")
    contract = M2_CONTRACT_PATH.read_text(encoding="utf-8")

    combined = odd + "\n" + oddd
    for phrase in STALE_PHRASES:
        assert phrase not in combined, f"stale pre-v15 mortality semantics remain: {phrase}"

    for path, claims in REQUIRED_CURRENT_CLAIMS.items():
        text = path.read_text(encoding="utf-8")
        for claim in claims:
            assert claim in text, f"missing current mortality semantics in {path}: {claim}"

    # Normative positive controls: living standards-facing docs must stay aligned
    # with the executable/normative v15+ competing-risk contract.
    assert "background mortality has already been resolved over the elapsed year" in model
    assert "year-end M2 stage then performs fertility/parentage only" in model
    assert "order-invariant competition between the M3 condition-mediated cause and the M2 background cause" in model
    assert "background mortality" in contract
    assert "fertility" in contract


if __name__ == "__main__":
    check()
