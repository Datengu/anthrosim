#!/usr/bin/env python3
"""Permanent regression coverage for AV5-007 exact parameter-coordinate fidelity."""

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


def point(point_id: str, theta: int | float, score: float) -> dict:
    return {
        "id": point_id,
        "parameters": {"theta": theta},
        "structure": "default",
        "outputs": {"score": score},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    }


def analyse(levels: list[int | float], analysis_id: str) -> dict:
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


def theta(result: dict) -> dict:
    matches = [item for item in result["parameterDiagnostics"] if item["parameter"] == "theta"]
    assert len(matches) == 1
    return matches[0]


def assert_half_width_not_identified(levels: list[int], analysis_id: str) -> None:
    exact_width = Fraction(levels[1] - levels[0], levels[2] - levels[0])
    assert exact_width == Fraction(1, 2)
    result = analyse(levels, analysis_id)
    diagnostic = theta(result)
    assert diagnostic["fullRange"] == [levels[0], levels[2]]
    assert diagnostic["compatibleRange"] == [levels[0], levels[1]]
    assert diagnostic["exploredLevelCount"] == 3
    assert diagnostic["normalizedCompatibleWidth"] == 0.5
    assert diagnostic["identified"] is False
    assert diagnostic["reason"] == "compatible_region_too_wide"
    assert result["researchGate"]["passes"] is False


def main() -> None:
    # Matched small-integer control from the Audit-v5 discovery adversary.
    assert_half_width_not_identified([0, 1, 2], "av5-007-small-integer-control")

    # Exact discovery adversary: 2**53+1 must remain distinct from 2**53.
    base = 2**53
    assert_half_width_not_identified(
        [base, base + 1, base + 2],
        "av5-007-binary64-boundary",
    )

    # Real wide-integer configuration domains include u64-valued coordinates.
    u64_max = 2**64 - 1
    assert_half_width_not_identified(
        [u64_max - 2, u64_max - 1, u64_max],
        "av5-007-u64-upper-boundary",
    )

    # Ordinary approximate floating coordinates remain supported and retain their
    # existing binary64 interpretation.
    floating = analyse([0.0, 0.5, 1.0], "av5-007-floating-control")
    floating_theta = theta(floating)
    assert floating_theta["fullRange"] == [0.0, 1.0]
    assert floating_theta["compatibleRange"] == [0.0, 0.5]
    assert floating_theta["exploredLevelCount"] == 3
    assert floating_theta["normalizedCompatibleWidth"] == 0.5
    assert floating_theta["identified"] is False
    assert floating["researchGate"]["passes"] is False

    # AV3-011: one genuinely explored level is never positive identification.
    fixed_points = [
        point("fixed-a", 7, 0.0),
        point("fixed-b", 7, 0.0),
    ]
    fixed_plan = {
        "schemaVersion": 2,
        "analysisId": "av5-007-fixed-control",
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    fixed_data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": fixed_points}
    fixed = identifiability.analyse(
        fixed_plan,
        identifiability.bind_synthetic_fixture(fixed_data, "av5-007-fixed-control"),
    )
    fixed_theta = theta(fixed)
    assert fixed_theta["exploredLevelCount"] == 1
    assert fixed_theta["identified"] is False
    assert fixed_theta["reason"] == "insufficient_explored_variation"
    assert fixed["researchGate"]["passes"] is False


if __name__ == "__main__":
    main()
