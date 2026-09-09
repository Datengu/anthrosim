#!/usr/bin/env python3
"""Audit-v6 Area J control: a narrow identified claim must not erase broader equifinality."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER_PATH = ROOT / "scripts" / "research-identifiability.py"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


analyzer = load_module(ANALYZER_PATH, "audit_v6_area_j_equifinality")


def main() -> None:
    points = []
    for theta in [0, 1]:
        for nuisance in [0, 1]:
            for structure in ["structure_a", "structure_b"]:
                output = 0.0 if theta == 0 else 10.0
                points.append(
                    {
                        "id": f"theta-{theta}-nuisance-{nuisance}-{structure}",
                        "parameters": {"theta": theta, "nuisance": nuisance},
                        "structure": structure,
                        "outputs": {"calibration": output},
                        "outputEvidence": {"calibration": {"kind": "deterministic"}},
                    }
                )

    data = analyzer.bind_synthetic_fixture(
        {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points},
        "audit-v6-area-j-narrow-claim-equifinality",
    )
    plan = {
        "schemaVersion": 2,
        "analysisId": "audit-v6-area-j-narrow-claim-equifinality",
        "calibrationTargets": [
            {"observable": "calibration", "target": 0.0, "tolerance": 0.0}
        ],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.0,
    }

    result = analyzer.analyse(plan, data)
    theta = next(item for item in result["parameterDiagnostics"] if item["parameter"] == "theta")
    nuisance = next(
        item for item in result["parameterDiagnostics"] if item["parameter"] == "nuisance"
    )

    print(
        f"gate={str(result['researchGate']['passes']).lower()} "
        f"compatible={result['compatibleRegion']['pointCount']} "
        f"theta_identified={str(theta['identified']).lower()} "
        f"nuisance_identified={str(nuisance['identified']).lower()}"
    )
    print(
        f"equifinality_present={str(result['equifinality']['present']).lower()} "
        f"parameter_combination_equifinality={str(result['equifinality']['parameterCombinationEquifinality']).lower()} "
        f"structural_equifinality={str(result['equifinality']['structuralEquifinality']).lower()} "
        f"nuisance_compensation={result['equifinality']['nuisanceParameterCompensation']['parameterIds']}"
    )

    assert result["researchGate"]["passes"] is True
    assert result["compatibleRegion"]["pointCount"] == 4
    assert theta["identified"] is True
    assert theta["compatibleRange"] == [0, 0]
    assert nuisance["identified"] is False
    assert nuisance["compatibleRange"] == [0, 1]
    assert result["structuralDiagnostic"]["equifinal"] is True
    assert result["structuralDiagnostic"]["compatibleStructures"] == [
        "structure_a",
        "structure_b",
    ]
    assert result["equifinality"]["present"] is True
    assert result["equifinality"]["parameterCombinationEquifinality"] is True
    assert result["equifinality"]["structuralEquifinality"] is True
    assert result["equifinality"]["nuisanceParameterCompensation"] == {
        "present": True,
        "parameterIds": ["nuisance"],
    }


if __name__ == "__main__":
    main()
