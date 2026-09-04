#!/usr/bin/env python3
"""Regression tests for AV3-012 / #421 nuisance-parameter equifinality reporting."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("anthrosim_research_identifiability_av3_012", ANALYZER)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


def point(point_id: str, theta: float, nuisance: float, score: float) -> dict:
    outputs = {"score": score}
    return {
        "id": point_id,
        "parameters": {"theta": theta, "nuisance": nuisance},
        "structure": "same",
        "outputs": outputs,
        "outputEvidence": {"score": {"kind": "deterministic"}},
    }


def analyse(points: list[dict], analysis_id: str, claimed: list[str] | None = None) -> dict:
    plan = {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"] if claimed is None else claimed, "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
    return module.analyse(plan, module.bind_synthetic_fixture(data, analysis_id))


def by_parameter(result: dict) -> dict[str, dict]:
    return {item["parameter"]: item for item in result["parameterDiagnostics"]}


def main() -> None:
    # Exact #420 / AV3-012 adversary. The narrow declared theta claim is
    # practically identified, but two distinct compatible parameter
    # combinations remain and nuisance spans the full explored range.
    adversary = analyse(
        [
            point("compatible-a", 0.0, 0.0, 0.0),
            point("compatible-b", 0.1, 100.0, 0.0),
            point("rejected-extreme", 1.0, 50.0, 10.0),
        ],
        "av3-012-nuisance-compensation",
    )
    diagnostics = by_parameter(adversary)
    assert adversary["researchGate"]["passes"] is True
    assert adversary["compatibleRegion"]["pointIds"] == ["compatible-a", "compatible-b"]
    assert diagnostics["theta"]["identified"] is True
    assert abs(diagnostics["theta"]["normalizedCompatibleWidth"] - 0.1) < 1e-12
    assert diagnostics["nuisance"]["identified"] is False
    assert abs(diagnostics["nuisance"]["normalizedCompatibleWidth"] - 1.0) < 1e-12
    assert adversary["equifinality"]["present"] is True
    assert adversary["equifinality"]["parameterCombinationEquifinality"] is True
    assert adversary["equifinality"]["distinctCompatibleParameterCombinationCount"] == 2
    assert adversary["equifinality"]["structuralEquifinality"] is False
    assert adversary["equifinality"]["nuisanceParameterCompensation"] == {
        "present": True,
        "parameterIds": ["nuisance"],
    }

    # Genuine unique-compatible-point control: the declared claim passes and
    # the compatible scientific state is unique, so no equifinality remains.
    unique = analyse(
        [
            point("unique-compatible", 0.0, 0.0, 0.0),
            point("rejected-mid", 0.1, 100.0, 10.0),
            point("rejected-extreme", 1.0, 50.0, 10.0),
        ],
        "av3-012-unique-control",
    )
    assert unique["researchGate"]["passes"] is True
    assert unique["compatibleRegion"]["pointIds"] == ["unique-compatible"]
    assert unique["equifinality"]["present"] is False
    assert unique["equifinality"]["parameterCombinationEquifinality"] is False
    assert unique["equifinality"]["distinctCompatibleParameterCombinationCount"] == 1
    assert unique["equifinality"]["structuralEquifinality"] is False
    assert unique["equifinality"]["nuisanceParameterCompensation"] == {
        "present": False,
        "parameterIds": [],
    }

    # Multiple rows with the same scientific state are not multiple parameter
    # combinations. This guards against equating row count with equifinality.
    duplicate_state = analyse(
        [
            point("replicate-a", 0.0, 0.0, 0.0),
            point("replicate-b", 0.0, 0.0, 0.0),
        ],
        "av3-012-duplicate-state-control",
        claimed=[],
    )
    assert duplicate_state["researchGate"]["passes"] is True
    assert duplicate_state["compatibleRegion"]["pointCount"] == 2
    assert duplicate_state["equifinality"]["distinctCompatibleParameterCombinationCount"] == 1
    assert duplicate_state["equifinality"]["present"] is False

    print("AV3-012 equifinality/nuisance-compensation regression: ok")


if __name__ == "__main__":
    main()
