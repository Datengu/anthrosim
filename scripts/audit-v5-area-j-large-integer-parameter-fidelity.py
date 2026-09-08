#!/usr/bin/env python3
"""Audit-v5 Area-J adversary for binary64 loss in parameter-width diagnostics."""

from __future__ import annotations

from fractions import Fraction
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("identifiability", ANALYZER)
assert SPEC and SPEC.loader
identifiability = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(identifiability)


def point(point_id: str, theta: int, score: float) -> dict:
    return {
        "id": point_id,
        "parameters": {"theta": theta},
        "structure": "default",
        "outputs": {"score": score},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    }


def analyse(levels: list[int], analysis_id: str) -> dict:
    points = [
        point("compatible-low", levels[0], 0.0),
        point("compatible-high", levels[1], 0.0),
        point("rejected-high", levels[2], 10.0),
    ]
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
    return identifiability.analyse(
        plan,
        identifiability.bind_synthetic_fixture(data, analysis_id),
    )


def theta_diagnostic(result: dict) -> dict:
    matches = [item for item in result["parameterDiagnostics"] if item["parameter"] == "theta"]
    assert len(matches) == 1
    return matches[0]


def main() -> None:
    # Matched exactly representable control. The compatible region spans 1/2 of
    # the explored range, so a 0.25 identification threshold must fail.
    control = analyse([0, 1, 2], "audit-v5-area-j-small-integer-control")
    control_diag = theta_diagnostic(control)
    assert control_diag["normalizedCompatibleWidth"] == 0.5
    assert control_diag["identified"] is False
    assert control["researchGate"]["passes"] is False

    # All three coordinates are distinct exact JSON integers. 2**53+1 is not
    # exactly representable in binary64 and rounds to 2**53 when _number() casts
    # parameter coordinates to float.
    base = 2**53
    levels = [base, base + 1, base + 2]
    exact_full_width = levels[2] - levels[0]
    exact_compatible_width = levels[1] - levels[0]
    exact_normalized_width = Fraction(exact_compatible_width, exact_full_width)
    assert exact_normalized_width == Fraction(1, 2)
    assert exact_normalized_width > Fraction(1, 4)

    adversary = analyse(levels, "audit-v5-area-j-large-integer-adversary")
    adversary_diag = theta_diagnostic(adversary)

    print(
        "exact_levels={}; exact_normalized_width={}; analyzer_full_range={}; "
        "analyzer_compatible_range={}; analyzer_normalized_width={}; "
        "explored_level_count={}; identified={}; gate={}".format(
            levels,
            float(exact_normalized_width),
            adversary_diag["fullRange"],
            adversary_diag["compatibleRange"],
            adversary_diag["normalizedCompatibleWidth"],
            adversary_diag["exploredLevelCount"],
            adversary_diag["identified"],
            adversary["researchGate"]["passes"],
        )
    )

    # Frozen-v0.3.5 defect oracle: the exact compatible span is 50% of the
    # explored range, but binary64 collapse of 2**53 and 2**53+1 makes the
    # analyzer report zero compatible width and positively identify theta.
    assert adversary_diag["fullRange"] == [float(base), float(base + 2)]
    assert adversary_diag["compatibleRange"] == [float(base), float(base)]
    assert adversary_diag["normalizedCompatibleWidth"] == 0.0
    assert adversary_diag["exploredLevelCount"] == 2
    assert adversary_diag["identified"] is True
    assert adversary["researchGate"]["passes"] is True


if __name__ == "__main__":
    main()
