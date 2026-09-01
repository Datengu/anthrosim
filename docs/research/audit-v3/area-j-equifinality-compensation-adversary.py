#!/usr/bin/env python3
"""Audit-v3 Area J adversary: nuisance-parameter compensation must remain explicit equifinality."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "scripts" / "research-identifiability.py"
SPEC = importlib.util.spec_from_file_location("research_identifiability", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)

plan = {
    "schemaVersion": 2,
    "analysisId": "audit-v3-area-j-equifinality-compensation",
    "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
    "corroborationObservables": [],
    "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
    "maxNormalizedAcceptableWidth": 0.25,
}

points = [
    {
        "id": "compatible-a",
        "parameters": {"theta": 0.0, "nuisance": 0.0},
        "structure": "same",
        "outputs": {"score": 0.0},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    },
    {
        "id": "compatible-b",
        "parameters": {"theta": 0.1, "nuisance": 100.0},
        "structure": "same",
        "outputs": {"score": 0.0},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    },
    {
        "id": "rejected-extreme",
        "parameters": {"theta": 1.0, "nuisance": 50.0},
        "structure": "same",
        "outputs": {"score": 10.0},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    },
]

data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
result = module.analyse(plan, data)
by_parameter = {item["parameter"]: item for item in result["parameterDiagnostics"]}

print("researchGate.passes=", result["researchGate"]["passes"])
print("compatiblePointIds=", result["compatibleRegion"]["pointIds"])
print("theta.normalizedCompatibleWidth=", by_parameter["theta"]["normalizedCompatibleWidth"])
print("theta.identified=", by_parameter["theta"]["identified"])
print("nuisance.normalizedCompatibleWidth=", by_parameter["nuisance"]["normalizedCompatibleWidth"])
print("nuisance.identified=", by_parameter["nuisance"]["identified"])
print("equifinality.present=", result["equifinality"]["present"])

assert result["researchGate"]["passes"] is True
assert result["compatibleRegion"]["pointIds"] == ["compatible-a", "compatible-b"]
assert by_parameter["theta"]["identified"] is True
assert abs(by_parameter["theta"]["normalizedCompatibleWidth"] - 0.1) < 1e-12
assert by_parameter["nuisance"]["identified"] is False
assert abs(by_parameter["nuisance"]["normalizedCompatibleWidth"] - 1.0) < 1e-12

# Contract under test: docs/research/identifiability-equifinality-v1.md states that
# when multiple parameter combinations remain compatible with the claim,
# equifinality.present is true. These two compatible points differ in both theta
# and an unconstrained nuisance parameter, so the compatible region is not unique.
assert result["equifinality"]["present"] is True, result["equifinality"]
