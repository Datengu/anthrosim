#!/usr/bin/env python3
"""Independent post-merge reverification for Audit-v3 AV3-012 / #421."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("anthrosim_research_identifiability_av3_012_post_merge", ANALYZER)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


def point(point_id: str, theta: float, nuisance: float, score: float) -> dict:
    return {
        "id": point_id,
        "parameters": {"theta": theta, "nuisance": nuisance},
        "structure": "same",
        "outputs": {"score": score},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    }


def analyse(points: list[dict], analysis_id: str) -> dict:
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    return module.analyse(plan, {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points})


def diagnostic_map(result: dict) -> dict[str, dict]:
    return {item["parameter"]: item for item in result["parameterDiagnostics"]}


def main() -> None:
    # Exact AV3-012 / PR #420 scientific adversary.
    result = analyse(
        [
            point("compatible-a", 0.0, 0.0, 0.0),
            point("compatible-b", 0.1, 100.0, 0.0),
            point("rejected-extreme", 1.0, 50.0, 10.0),
        ],
        "av3-012-post-merge-original-adversary",
    )
    diagnostics = diagnostic_map(result)

    assert result["researchGate"]["passes"] is True, result["researchGate"]
    assert result["compatibleRegion"]["pointIds"] == ["compatible-a", "compatible-b"]
    assert diagnostics["theta"]["identified"] is True
    assert abs(diagnostics["theta"]["normalizedCompatibleWidth"] - 0.1) < 1e-12
    assert diagnostics["nuisance"]["identified"] is False
    assert abs(diagnostics["nuisance"]["normalizedCompatibleWidth"] - 1.0) < 1e-12

    equifinality = result["equifinality"]
    assert equifinality["present"] is True, equifinality
    assert equifinality["parameterCombinationEquifinality"] is True, equifinality
    assert equifinality["distinctCompatibleParameterCombinationCount"] == 2, equifinality
    assert equifinality["structuralEquifinality"] is False, equifinality
    assert equifinality["nuisanceParameterCompensation"] == {
        "present": True,
        "parameterIds": ["nuisance"],
    }, equifinality

    print("researchGate.passes:", result["researchGate"]["passes"])
    print("compatiblePointIds:", result["compatibleRegion"]["pointIds"])
    print("theta.normalizedCompatibleWidth:", diagnostics["theta"]["normalizedCompatibleWidth"])
    print("theta.identified:", diagnostics["theta"]["identified"])
    print("nuisance.normalizedCompatibleWidth:", diagnostics["nuisance"]["normalizedCompatibleWidth"])
    print("nuisance.identified:", diagnostics["nuisance"]["identified"])
    print("equifinality.present:", equifinality["present"])
    print("parameterCombinationEquifinality:", equifinality["parameterCombinationEquifinality"])
    print("distinctCompatibleParameterCombinationCount:", equifinality["distinctCompatibleParameterCombinationCount"])
    print("nuisanceParameterCompensation:", equifinality["nuisanceParameterCompensation"])

    # Genuine unique-compatible-point control.
    unique = analyse(
        [
            point("unique-compatible", 0.0, 0.0, 0.0),
            point("rejected-mid", 0.1, 100.0, 10.0),
            point("rejected-extreme", 1.0, 50.0, 10.0),
        ],
        "av3-012-post-merge-unique-control",
    )
    assert unique["researchGate"]["passes"] is True
    assert unique["compatibleRegion"]["pointIds"] == ["unique-compatible"]
    assert unique["equifinality"]["present"] is False
    assert unique["equifinality"]["distinctCompatibleParameterCombinationCount"] == 1
    assert unique["equifinality"]["nuisanceParameterCompensation"] == {
        "present": False,
        "parameterIds": [],
    }
    print("unique control equifinality.present:", unique["equifinality"]["present"])
    print("AV3-012 post-merge adversary passed: original defect no longer demonstrates")


if __name__ == "__main__":
    main()
