#!/usr/bin/env python3
"""Regression coverage for Audit-v3 AV3-010 / issue #418."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
spec = importlib.util.spec_from_file_location("anthrosim_research_identifiability", ANALYZER)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def point(point_id: str, structure: str, theta: int, held_out: float) -> dict:
    outputs = {"calibration": 0.0, "held_out": held_out}
    return {
        "id": point_id,
        "structure": structure,
        "parameters": {"theta": theta},
        "outputs": outputs,
        "outputEvidence": {
            observable: {"kind": "deterministic"} for observable in outputs
        },
    }


def analyse(points: list[dict], analysis_id: str) -> dict:
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [
            {"observable": "calibration", "target": 0.0, "tolerance": 0.0}
        ],
        "corroborationObservables": ["held_out"],
        "corroborationDiscriminationTolerance": 1.0,
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": points,
    }
    return module.analyse(plan, module.bind_synthetic_fixture(data, analysis_id))


def prediction(result: dict) -> dict:
    predictions = result["discriminatingPredictions"]
    assert len(predictions) == 1, predictions
    item = predictions[0]
    assert item["structures"] == ["A", "B"]
    assert item["observable"] == "held_out"
    assert item["role"] == "held_out_corroboration_prediction"
    return item


def main() -> None:
    # Exact AV3-010 adversary. All four points are calibration-compatible.
    # The scientifically relevant structural prediction ranges overlap:
    # A=[0,10], B=[9,11]. The minimum separation is therefore zero.
    overlap = analyse(
        [
            point("a-0", "A", 0, 0.0),
            point("a-10", "A", 1, 10.0),
            point("b-9", "B", 2, 9.0),
            point("b-11", "B", 3, 11.0),
        ],
        "av3-010-overlapping-compatible-envelopes",
    )
    item = prediction(overlap)
    assert overlap["compatibleRegion"]["pointCount"] == 4
    assert overlap["structuralDiagnostic"]["equifinal"] is True
    assert item["leftSimulationIntervalEnvelope"] == [0.0, 10.0], item
    assert item["rightSimulationIntervalEnvelope"] == [9.0, 11.0], item
    assert item["minimumIntervalSeparation"] == 0.0, item
    assert item["discriminating"] is False, item

    # Positive control: genuinely separated compatible structural envelopes
    # remain discriminating at the same tolerance.
    separated = analyse(
        [
            point("a-0", "A", 0, 0.0),
            point("a-2", "A", 1, 2.0),
            point("b-10", "B", 2, 10.0),
            point("b-12", "B", 3, 12.0),
        ],
        "av3-010-separated-compatible-envelopes",
    )
    item = prediction(separated)
    assert item["leftSimulationIntervalEnvelope"] == [0.0, 2.0], item
    assert item["rightSimulationIntervalEnvelope"] == [10.0, 12.0], item
    assert item["minimumIntervalSeparation"] == 8.0, item
    assert item["discriminating"] is True, item

    print("AV3-010 held-out structural envelope regression: ok")


if __name__ == "__main__":
    main()
