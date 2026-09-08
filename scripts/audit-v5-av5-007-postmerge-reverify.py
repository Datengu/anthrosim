#!/usr/bin/env python3
"""Independent post-merge adversarial re-verification for Audit-v5 AV5-007.

This restores the controlled construction from discovery evidence PR #650 against
merged production main. The original defect oracle expected binary64 collapse;
this post-merge oracle requires the exact scientific geometry to survive.
"""

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


def require_half_width_not_identified(levels: list[int], analysis_id: str) -> dict:
    exact_full_width = levels[2] - levels[0]
    exact_compatible_width = levels[1] - levels[0]
    exact_normalized_width = Fraction(exact_compatible_width, exact_full_width)
    assert exact_normalized_width == Fraction(1, 2)
    assert exact_normalized_width > Fraction(1, 4)

    result = analyse(levels, analysis_id)
    diagnostic = theta_diagnostic(result)

    assert diagnostic["fullRange"] == [levels[0], levels[2]]
    assert diagnostic["compatibleRange"] == [levels[0], levels[1]]
    assert diagnostic["normalizedCompatibleWidth"] == 0.5
    assert diagnostic["exploredLevelCount"] == 3
    assert diagnostic["coordinateRepresentation"] == "exact_json_integer"
    assert diagnostic["identified"] is False
    assert diagnostic["reason"] == "compatible_region_too_wide"
    assert result["researchGate"]["passes"] is False
    assert result["researchGate"]["executedDesignBound"] is True
    assert result["parameterCoordinateSemantics"] == {
        "integerCoordinates": "exact_json_integer",
        "floatingCoordinates": "binary64_approximate",
        "unsafeMixedIntegerFloatCoordinates": "fail_closed",
    }
    return result


def main() -> None:
    # Matched exactly representable control from the original discovery adversary.
    control = require_half_width_not_identified(
        [0, 1, 2], "audit-v5-av5-007-postmerge-small-integer-control"
    )

    # Original discovery geometry: these are three distinct exact JSON integers.
    base = 2**53
    levels = [base, base + 1, base + 2]
    adversary = require_half_width_not_identified(
        levels, "audit-v5-av5-007-postmerge-large-integer-adversary"
    )
    diagnostic = theta_diagnostic(adversary)

    print(
        "reverified=true; exact_levels={}; exact_normalized_width=0.5; "
        "analyzer_full_range={}; analyzer_compatible_range={}; "
        "analyzer_normalized_width={}; explored_level_count={}; "
        "coordinate_representation={}; identified={}; gate={}; control_gate={}".format(
            levels,
            diagnostic["fullRange"],
            diagnostic["compatibleRange"],
            diagnostic["normalizedCompatibleWidth"],
            diagnostic["exploredLevelCount"],
            diagnostic["coordinateRepresentation"],
            diagnostic["identified"],
            adversary["researchGate"]["passes"],
            control["researchGate"]["passes"],
        )
    )


if __name__ == "__main__":
    main()
