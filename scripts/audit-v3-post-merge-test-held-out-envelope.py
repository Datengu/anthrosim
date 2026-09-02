#!/usr/bin/env python3
"""Independent post-merge reverification for Audit-v3 AV3-010 / #418."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("anthrosim_research_identifiability_av3_010", ANALYZER)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


def point(point_id: str, structure: str, heldout: float) -> dict:
    outputs = {"score": 0.0, "heldout": heldout}
    return {
        "id": point_id,
        "parameters": {},
        "structure": structure,
        "outputs": outputs,
        "outputEvidence": {name: {"kind": "deterministic"} for name in outputs},
    }


def analyse(heldout_values: tuple[float, float, float, float], analysis_id: str) -> dict:
    a0, a1, b0, b1 = heldout_values
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": ["heldout"],
        "corroborationDiscriminationTolerance": 1.0,
        "claim": {"parameterIds": [], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": [
            point("a-low", "A", a0),
            point("a-high", "A", a1),
            point("b-low", "B", b0),
            point("b-high", "B", b1),
        ],
    }
    return module.analyse(plan, data)


def prediction(result: dict) -> dict:
    matches = [item for item in result["discriminatingPredictions"] if item["observable"] == "heldout"]
    assert len(matches) == 1, matches
    return matches[0]


def main() -> None:
    # Exact AV3-010 frozen adversary from issue #418.
    result = analyse((0.0, 10.0, 9.0, 11.0), "av3-010-post-merge-overlap")
    observed = prediction(result)

    # The frozen defective calculation averaged deterministic point intervals:
    # A -> [5, 5], B -> [10, 10], false minimum gap 5 > tolerance 1.
    legacy_a_average = (0.0 + 10.0) / 2.0
    legacy_b_average = (9.0 + 11.0) / 2.0
    legacy_false_gap = legacy_b_average - legacy_a_average
    assert legacy_false_gap == 5.0

    assert observed["leftSimulationIntervalEnvelope"] == [0.0, 10.0], observed
    assert observed["rightSimulationIntervalEnvelope"] == [9.0, 11.0], observed
    assert observed["minimumIntervalSeparation"] == 0.0, observed
    assert observed["discriminating"] is False, observed

    print("legacy averaged A interval: [5.0, 5.0]")
    print("legacy averaged B interval: [10.0, 10.0]")
    print("legacy false minimum separation:", legacy_false_gap)
    print("merged-main A conservative envelope:", observed["leftSimulationIntervalEnvelope"])
    print("merged-main B conservative envelope:", observed["rightSimulationIntervalEnvelope"])
    print("merged-main minimum separation:", observed["minimumIntervalSeparation"])
    print("merged-main discriminating:", observed["discriminating"])

    # Positive control: genuinely separated compatible structural ranges must
    # remain discriminating under the same tolerance.
    control = prediction(analyse((0.0, 2.0, 10.0, 12.0), "av3-010-post-merge-separated-control"))
    assert control["leftSimulationIntervalEnvelope"] == [0.0, 2.0], control
    assert control["rightSimulationIntervalEnvelope"] == [10.0, 12.0], control
    assert control["minimumIntervalSeparation"] == 8.0, control
    assert control["discriminating"] is True, control
    print("separated control minimum separation:", control["minimumIntervalSeparation"])
    print("separated control discriminating:", control["discriminating"])
    print("AV3-010 post-merge adversary passed: original defect no longer demonstrates")


if __name__ == "__main__":
    main()
