#!/usr/bin/env python3
"""Audit-v5 Area-J control for valid held-out discrimination and calibration firewall."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("identifiability", ANALYZER)
assert SPEC and SPEC.loader
identifiability = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(identifiability)

DISCRIMINATION_TOLERANCE = 1.0


def point(point_id: str, structure: str, held_out: float) -> dict:
    outputs = {"calibration": 0.0, "held_out": held_out}
    return {
        "id": point_id,
        "parameters": {},
        "structure": structure,
        "outputs": outputs,
        "outputEvidence": {
            "calibration": {"kind": "deterministic"},
            "held_out": {"kind": "deterministic"},
        },
    }


def analyse(held_out_values: list[float], analysis_id: str) -> dict:
    assert len(held_out_values) == 4
    data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": [
            point("a-low", "A", held_out_values[0]),
            point("a-high", "A", held_out_values[1]),
            point("b-low", "B", held_out_values[2]),
            point("b-high", "B", held_out_values[3]),
        ],
    }
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [
            {"observable": "calibration", "target": 0.0, "tolerance": 0.0}
        ],
        "corroborationObservables": ["held_out"],
        "corroborationDiscriminationTolerance": DISCRIMINATION_TOLERANCE,
        "claim": {"parameterIds": [], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    return identifiability.analyse(
        plan,
        identifiability.bind_synthetic_fixture(data, analysis_id),
    )


def only_prediction(result: dict) -> dict:
    predictions = result["discriminatingPredictions"]
    assert len(predictions) == 1, predictions
    return predictions[0]


def main() -> None:
    # Calibration is deliberately identical for both structures. Held-out values
    # are also made non-discriminating in the matched control.
    nondiscriminating = analyse(
        [5.0, 5.0, 5.0, 5.0],
        "audit-v5-area-j-heldout-firewall-nondiscriminating-control",
    )
    nondiscriminating_prediction = only_prediction(nondiscriminating)

    assert nondiscriminating["compatibleRegion"]["pointIds"] == [
        "a-low",
        "a-high",
        "b-low",
        "b-high",
    ]
    assert nondiscriminating["structuralDiagnostic"]["compatibleStructures"] == ["A", "B"]
    assert nondiscriminating["structuralDiagnostic"]["equifinal"] is True
    assert nondiscriminating["researchGate"]["passes"] is False
    assert nondiscriminating_prediction["leftSimulationIntervalEnvelope"] == [5.0, 5.0]
    assert nondiscriminating_prediction["rightSimulationIntervalEnvelope"] == [5.0, 5.0]
    assert nondiscriminating_prediction["minimumIntervalSeparation"] == 0.0
    assert nondiscriminating_prediction["discriminating"] is False

    # Change only the held-out/corroboration values. Calibration evidence and all
    # calibration-compatible structures remain identical, but the held-out
    # structural envelopes are now genuinely separated by 8 units.
    discriminating = analyse(
        [0.0, 2.0, 10.0, 12.0],
        "audit-v5-area-j-heldout-firewall-positive-control",
    )
    discriminating_prediction = only_prediction(discriminating)

    assert discriminating["compatibleRegion"] == nondiscriminating["compatibleRegion"]
    assert discriminating["structuralDiagnostic"] == nondiscriminating["structuralDiagnostic"]
    assert discriminating["researchGate"] == nondiscriminating["researchGate"]
    assert discriminating["researchGate"]["passes"] is False

    assert discriminating_prediction["leftSimulationIntervalEnvelope"] == [0.0, 2.0]
    assert discriminating_prediction["rightSimulationIntervalEnvelope"] == [10.0, 12.0]
    assert discriminating_prediction["minimumIntervalSeparation"] == 8.0
    assert discriminating_prediction["discriminating"] is True

    print(
        "compatible_points={}; compatible_structures={}; calibration_gate={}; "
        "control_discriminating={}; separated_left={}; separated_right={}; "
        "minimum_gap={}; tolerance={}; separated_discriminating={}".format(
            discriminating["compatibleRegion"]["pointIds"],
            discriminating["structuralDiagnostic"]["compatibleStructures"],
            discriminating["researchGate"]["passes"],
            nondiscriminating_prediction["discriminating"],
            discriminating_prediction["leftSimulationIntervalEnvelope"],
            discriminating_prediction["rightSimulationIntervalEnvelope"],
            discriminating_prediction["minimumIntervalSeparation"],
            DISCRIMINATION_TOLERANCE,
            discriminating_prediction["discriminating"],
        )
    )


if __name__ == "__main__":
    main()
