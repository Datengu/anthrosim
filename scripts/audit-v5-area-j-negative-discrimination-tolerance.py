#!/usr/bin/env python3
"""Audit-v5 Area-J adversary for negative held-out discrimination tolerance."""

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
    # Calibration leaves both structures fully equifinal. Their held-out envelopes
    # overlap on [9, 10], so the scientifically meaningful minimum separation is 0.
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
    control = analyse(1.0, "audit-v5-area-j-positive-tolerance-control")
    control_prediction = prediction(control)
    assert control["structuralDiagnostic"]["equifinal"] is True
    assert control_prediction["leftSimulationIntervalEnvelope"] == [0.0, 10.0]
    assert control_prediction["rightSimulationIntervalEnvelope"] == [9.0, 11.0]
    assert control_prediction["minimumIntervalSeparation"] == 0.0
    assert control_prediction["discriminating"] is False

    adversary = analyse(-1.0, "audit-v5-area-j-negative-tolerance-adversary")
    adversary_prediction = prediction(adversary)

    print(
        "equifinal={}; left={}; right={}; minimum_gap={}; "
        "positive_tolerance_discriminating={}; negative_tolerance_discriminating={}; gate={}".format(
            adversary["structuralDiagnostic"]["equifinal"],
            adversary_prediction["leftSimulationIntervalEnvelope"],
            adversary_prediction["rightSimulationIntervalEnvelope"],
            adversary_prediction["minimumIntervalSeparation"],
            control_prediction["discriminating"],
            adversary_prediction["discriminating"],
            adversary["researchGate"]["passes"],
        )
    )

    # Frozen-v0.3.5 defect oracle: the analyzer accepts the physically/statistically
    # nonsensical negative tolerance. Since its discrimination rule is
    # minimum_gap > tolerance, zero-gap overlapping envelopes become labelled as
    # a positive discriminating prediction.
    assert adversary["structuralDiagnostic"]["equifinal"] is True
    assert adversary["researchGate"]["passes"] is False
    assert adversary_prediction["minimumIntervalSeparation"] == 0.0
    assert adversary_prediction["discriminating"] is True


if __name__ == "__main__":
    main()
