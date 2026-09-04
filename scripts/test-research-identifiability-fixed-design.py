#!/usr/bin/env python3
"""Regression tests for fixed-by-design identifiability (AV3-011 / #419)."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
spec = importlib.util.spec_from_file_location("anthrosim_research_identifiability", ANALYZER)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def evidence(outputs: dict[str, float]) -> dict[str, dict[str, str]]:
    return {name: {"kind": "deterministic"} for name in outputs}


def plan(analysis_id: str) -> dict:
    return {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }


def point(point_id: str, theta, nuisance: int, score: float = 0.0) -> dict:
    outputs = {"score": score}
    return {
        "id": point_id,
        "parameters": {"theta": theta, "nuisance": nuisance},
        "outputs": outputs,
        "outputEvidence": evidence(outputs),
    }


def analyse(points: list[dict], analysis_id: str) -> dict:
    data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
    return module.analyse(
        plan(analysis_id),
        module.bind_synthetic_fixture(data, analysis_id),
    )


def main() -> None:
    fixed_numeric = analyse(
        [point("numeric-a", 7, 0), point("numeric-b", 7, 1)],
        "av3-011-fixed-numeric",
    )
    theta = next(item for item in fixed_numeric["parameterDiagnostics"] if item["parameter"] == "theta")
    assert theta["fullRange"] == [7.0, 7.0]
    assert theta["compatibleRange"] == [7.0, 7.0]
    assert theta["exploredLevelCount"] == 1
    assert theta["normalizedCompatibleWidth"] is None
    assert theta["reason"] == "insufficient_explored_variation"
    assert theta["identified"] is False
    assert fixed_numeric["researchGate"]["passes"] is False
    assert fixed_numeric["researchGate"]["reason"] == "declared_claim_not_identified"
    assert fixed_numeric["profiles"]["theta"] == [{"value": 7, "compatible": 2, "evaluated": 2}]

    fixed_categorical = analyse(
        [point("category-a", "same", 0), point("category-b", "same", 1)],
        "av3-011-fixed-categorical",
    )
    theta = next(item for item in fixed_categorical["parameterDiagnostics"] if item["parameter"] == "theta")
    assert theta["fullValues"] == ["same"]
    assert theta["compatibleValues"] == ["same"]
    assert theta["exploredLevelCount"] == 1
    assert theta["reason"] == "insufficient_explored_variation"
    assert theta["identified"] is False
    assert fixed_categorical["researchGate"]["passes"] is False

    varied_numeric = analyse(
        [point("numeric-selected", 0, 0), point("numeric-rejected", 1, 0, score=1.0)],
        "av3-011-varied-numeric-control",
    )
    theta = next(item for item in varied_numeric["parameterDiagnostics"] if item["parameter"] == "theta")
    assert theta["exploredLevelCount"] == 2
    assert theta["normalizedCompatibleWidth"] == 0.0
    assert theta["reason"] == "compatible_region_within_threshold"
    assert theta["identified"] is True
    assert varied_numeric["researchGate"]["passes"] is True

    varied_categorical = analyse(
        [point("category-selected", "selected", 0), point("category-rejected", "rejected", 0, score=1.0)],
        "av3-011-varied-categorical-control",
    )
    theta = next(item for item in varied_categorical["parameterDiagnostics"] if item["parameter"] == "theta")
    assert theta["exploredLevelCount"] == 2
    assert theta["compatibleValues"] == ["selected"]
    assert theta["reason"] == "single_compatible_value"
    assert theta["identified"] is True
    assert varied_categorical["researchGate"]["passes"] is True

    print("fixed-design identifiability regression: ok")


if __name__ == "__main__":
    main()
