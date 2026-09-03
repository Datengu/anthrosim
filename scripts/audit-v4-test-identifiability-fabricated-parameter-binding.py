#!/usr/bin/env python3
"""Audit-v4 Area J adversary: fabricated parameter-coordinate binding.

The identifiability gate describes its point table as the exact parameter
coordinates of an already-executed research design. This adversary keeps the
checked-in benchmark point identities and outputs unchanged, injects a parameter
that does not exist in the original benchmark rows, and asks whether the gate
can nevertheless certify that invented coordinate as identified.
"""

from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
PLAN = ROOT / "research" / "identifiability-benchmark-v1" / "plan.json"
DATA = ROOT / "research" / "identifiability-benchmark-v1" / "data.json"
FABRICATED = "fabricated_theta"


def load_analyzer():
    spec = importlib.util.spec_from_file_location("research_identifiability", ANALYZER)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load identifiability analyzer")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    analyzer = load_analyzer()
    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    data = json.loads(DATA.read_text(encoding="utf-8"))

    assert data["points"], "benchmark must contain design points"
    assert all(
        FABRICATED not in point["parameters"] for point in data["points"]
    ), "adversary parameter must not already exist in the preserved benchmark"

    baseline = analyzer.analyse(plan, data)
    assert baseline["researchGate"]["passes"] is True
    compatible_ids = baseline["compatibleRegion"]["pointIds"]
    assert len(compatible_ids) == 1, compatible_ids
    compatible_id = compatible_ids[0]

    fabricated_data = copy.deepcopy(data)
    for point in fabricated_data["points"]:
        # The accepted benchmark point receives one invented label and every
        # other already-rejected point receives another. Nothing about model
        # execution, point identity, outputs, or evidence is changed.
        point["parameters"][FABRICATED] = 0 if point["id"] == compatible_id else 1

    fabricated_plan = copy.deepcopy(plan)
    fabricated_plan["analysisId"] = "audit-v4-area-j-fabricated-parameter-binding"
    fabricated_plan["claim"] = {
        "parameterIds": [FABRICATED],
        "structuralHypothesis": False,
    }
    fabricated_plan["maxNormalizedAcceptableWidth"] = 0.25

    result = analyzer.analyse(fabricated_plan, fabricated_data)
    diagnostic = next(
        item for item in result["parameterDiagnostics"] if item["parameter"] == FABRICATED
    )

    print(f"baseline_compatible_point={compatible_id}")
    print(f"original_parameter_present={any(FABRICATED in p['parameters'] for p in data['points'])}")
    print(f"fabricated_full_range={diagnostic['fullRange']}")
    print(f"fabricated_compatible_range={diagnostic['compatibleRange']}")
    print(f"fabricated_identified={diagnostic['identified']}")
    print(f"research_gate_passes={result['researchGate']['passes']}")
    print(f"research_gate_reason={result['researchGate']['reason']}")
    print(f"compatible_point_ids={result['compatibleRegion']['pointIds']}")

    if result["researchGate"]["passes"]:
        raise AssertionError(
            "identifiability gate certified an invented parameter coordinate that is absent "
            "from the preserved benchmark design; parameter rows are not semantically bound "
            "to executed ResearchPoint/run configuration coordinates"
        )


if __name__ == "__main__":
    main()
