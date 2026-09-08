#!/usr/bin/env python3
"""Regression coverage for the held-out discrimination-tolerance domain."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("identifiability", ANALYZER)
assert SPEC and SPEC.loader
identifiability = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(identifiability)


def point(point_id: str, structure: str, theta: int, held_out: float) -> dict:
    outputs = {"calibration": 0.0, "held_out": held_out}
    return {
        "id": point_id,
        "structure": structure,
        "parameters": {"theta": theta},
        "outputs": outputs,
        "outputEvidence": {
            "calibration": {"kind": "deterministic"},
            "held_out": {"kind": "deterministic"},
        },
    }


def analyse(discrimination_tolerance: float, analysis_id: str) -> dict:
    points = [
        point("a-low", "A", 0, 0.0),
        point("a-high", "A", 1, 10.0),
        point("b-low", "B", 2, 9.0),
        point("b-high", "B", 3, 11.0),
    ]
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [
            {"observable": "calibration", "target": 0.0, "tolerance": 0.0}
        ],
        "corroborationObservables": ["held_out"],
        "corroborationDiscriminationTolerance": discrimination_tolerance,
        "claim": {"parameterIds": [], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": points,
    }
    return identifiability.analyse(
        plan,
        identifiability.bind_synthetic_fixture(data, analysis_id),
    )


def prediction(result: dict) -> dict:
    predictions = result["discriminatingPredictions"]
    assert len(predictions) == 1, predictions
    return predictions[0]


def main() -> None:
    for tolerance, analysis_id in [
        (0.0, "discrimination-zero-tolerance-control"),
        (1.0, "discrimination-positive-tolerance-control"),
    ]:
        result = analyse(tolerance, analysis_id)
        held_out = prediction(result)
        assert result["structuralDiagnostic"]["equifinal"] is True
        assert held_out["leftSimulationIntervalEnvelope"] == [0.0, 10.0]
        assert held_out["rightSimulationIntervalEnvelope"] == [9.0, 11.0]
        assert held_out["minimumIntervalSeparation"] == 0.0
        assert held_out["discriminating"] is False

    try:
        analyse(-1.0, "discrimination-negative-tolerance-rejected")
    except identifiability.IdentifiabilityError as error:
        assert str(error) == "corroborationDiscriminationTolerance must be >= 0", error
    else:
        raise AssertionError("negative corroboration discrimination tolerance was accepted")

    print("research-identifiability discrimination tolerance regression: ok")


if __name__ == "__main__":
    main()
