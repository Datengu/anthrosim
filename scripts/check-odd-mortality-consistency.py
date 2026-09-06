#!/usr/bin/env python3
"""Keep living ODD/ODD+D mortality timing aligned with authoritative semantics."""

from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ODD_PATH = ROOT / "docs" / "research" / "odd.md"
ODDD_PATH = ROOT / "docs" / "research" / "odd-d.md"
MODEL_PATH = ROOT / "docs" / "scientific-model.md"
M2_CONTRACT_PATH = ROOT / "docs" / "research" / "m2-demographic-time-contract-v1.md"

REPLACEMENTS = {
    ODD_PATH: {
        "- M2 mortality is drawn before fertility; the current fertility probability is therefore conditional on surviving the annual demographic mortality transition, subject also to spacing and parent-availability filters.":
            "- M2 background mortality is parameterized annually but executed across elapsed M3 intervals as an order-invariant competing risk with condition-mediated mortality. The year-end M2 stage performs fertility/parentage only after survival through the elapsed year; fertility remains conditional on survival, spacing and parent availability.",
        "6. after the year's subannual schedules complete, execute the M2 discrete transition for `[t-365,t)`: use interval-start age bands, draw mortality, then evaluate conditional fertility/parentage among survivors;":
            "6. after the year's subannual schedules complete, finalize M2 fertility/parentage for `[t-365,t)` among people who survived the elapsed-year competing-mortality process, using interval-start age bands for the annual demographic parameters;",
    },
    ODDD_PATH: {
        "M2 is likewise a coarse annual discrete transition, not continuous reproductive/death decision-making. Its schedule age is read at the start of `[t-365,t)`, mortality has declared priority, and fertility is conditional on surviving that annual transition. Those are model semantics rather than behavioural assertions.":
            "M2 reproduction/parentage is likewise a coarse annual discrete transition, not continuous reproductive decision-making. Its schedule age is read at the start of `[t-365,t)`. The annual background-mortality parameter is resolved across elapsed M3 intervals in order-invariant competition with condition-mediated mortality, so the year-end M2 stage has no separate mortality priority; fertility/parentage is conditional on having survived the elapsed year. Those are model semantics rather than behavioural assertions.",
    },
}

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


def apply_fix() -> None:
    for path, replacements in REPLACEMENTS.items():
        text = path.read_text(encoding="utf-8")
        for old, new in replacements.items():
            if old not in text:
                raise AssertionError(f"expected stale AV4-015 source text missing from {path}")
            text = text.replace(old, new, 1)
        path.write_text(text, encoding="utf-8")


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
    parser = argparse.ArgumentParser()
    parser.add_argument("--fix", action="store_true")
    args = parser.parse_args()
    if args.fix:
        apply_fix()
    check()
