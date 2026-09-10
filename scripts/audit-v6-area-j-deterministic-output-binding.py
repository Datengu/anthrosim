#!/usr/bin/env python3
"""Audit-v6 Area J adversary: bind identifiability outputs to real executions.

This evidence-only test uses a real ``anthrosim-research`` run and the production
executed-design binder. It then asks whether a deterministic analysis output can be
changed independently of those immutable executions strongly enough to reverse a
parameter-identification decision while the executed-design binding remains valid.
"""

from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER_PATH = ROOT / "scripts" / "research-identifiability.py"
BINDER_PATH = ROOT / "scripts" / "research-identifiability-bind-design.py"
BASE_DEFINITION = (
    ROOT / "research" / "general-demography-baseline-v1" / "confirmatory-definition.json"
)


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


analyzer = load_module(ANALYZER_PATH, "audit_v6_area_j_identifiability")
binder = load_module(BINDER_PATH, "audit_v6_area_j_binder")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def tiny_definition() -> dict:
    definition = load(BASE_DEFINITION)
    definition["seeds"] = [991, 992]
    experiment = definition["base"]["experiment"]
    experiment["seed"] = 991
    experiment["durationYears"] = 1
    experiment["population"]["initialPopulation"] = 20
    experiment["population"]["maxPersonRecords"] = 10_000
    experiment["world"] = {"schemaVersion": 1, "width": 4, "height": 4}
    definition["base"].pop("spatial", None)
    definition["dimensions"] = [
        {
            "id": "duration_years",
            "kind": "numeric",
            "path": "/experiment/durationYears",
            "values": [1, 2],
        }
    ]
    return definition


def data_from_binding(binding: dict, outputs: list[float]) -> dict:
    points = []
    assert len(binding["points"]) == len(outputs)
    for bound, output in zip(binding["points"], outputs, strict=True):
        points.append(
            {
                "id": bound["id"],
                "parameters": copy.deepcopy(bound["parameters"]),
                "structure": bound["structure"],
                "executionIds": list(bound["executionIds"]),
                "outputs": {"all_executions_completed": output},
                "outputEvidence": {
                    "all_executions_completed": {"kind": "deterministic"}
                },
            }
        )
    return {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--research-binary", required=True, type=Path)
    args = parser.parse_args()
    binary = args.research_binary.resolve(strict=True)

    with tempfile.TemporaryDirectory() as temp:
        temp_root = Path(temp)
        definition_path = temp_root / "definition.json"
        research_root = temp_root / "research-run"
        write(definition_path, tiny_definition())

        subprocess.run(
            [
                str(binary),
                "--definition",
                str(definition_path),
                "--run-dir",
                str(research_root),
            ],
            check=True,
        )

        binding = binder.derive_from_root(research_root)
        assert binding["sourceKind"] == "anthrosim_research_manifest_v1"
        assert len(binding["points"]) == 2
        assert [point["parameters"]["duration_years"] for point in binding["points"]] == [1, 2]
        assert all(len(point["executionIds"]) == 2 for point in binding["points"])

        plan = {
            "schemaVersion": 2,
            "analysisId": "audit-v6-area-j-deterministic-output-binding",
            "calibrationTargets": [
                {
                    "observable": "all_executions_completed",
                    "target": 1.0,
                    "tolerance": 0.0,
                }
            ],
            "corroborationObservables": [],
            "claim": {
                "parameterIds": ["duration_years"],
                "structuralHypothesis": False,
            },
            "maxNormalizedAcceptableWidth": 0.0,
        }

        truthful_data = data_from_binding(binding, [1.0, 1.0])
        truthful = analyzer.analyse_with_research_root(plan, truthful_data, research_root)
        assert truthful["researchGate"]["executedDesignBound"] is True
        assert truthful["researchGate"]["passes"] is False
        assert truthful["compatibleRegion"]["pointCount"] == 2
        truthful_duration = next(
            item
            for item in truthful["parameterDiagnostics"]
            if item["parameter"] == "duration_years"
        )
        assert truthful_duration["identified"] is False

        contradictory_data = data_from_binding(binding, [1.0, 0.0])
        contradictory = analyzer.analyse_with_research_root(
            plan, contradictory_data, research_root
        )
        contradictory_duration = next(
            item
            for item in contradictory["parameterDiagnostics"]
            if item["parameter"] == "duration_years"
        )

        print(f"research_id={binding['researchId']}")
        print(f"definition_identity={binding['definitionIdentity']}")
        print(
            "execution_ids="
            + ",".join(
                execution_id
                for point in binding["points"]
                for execution_id in point["executionIds"]
            )
        )
        print(
            "truthful_outputs=1.0,1.0 "
            f"gate={str(truthful['researchGate']['passes']).lower()} "
            f"compatible={truthful['compatibleRegion']['pointCount']} "
            f"identified={str(truthful_duration['identified']).lower()}"
        )
        print(
            "contradictory_outputs=1.0,0.0 "
            f"executed_design_bound={str(contradictory['researchGate']['executedDesignBound']).lower()} "
            f"gate={str(contradictory['researchGate']['passes']).lower()} "
            f"compatible={contradictory['compatibleRegion']['pointCount']} "
            f"identified={str(contradictory_duration['identified']).lower()}"
        )

        assert contradictory["researchGate"]["passes"] is False, (
            "predeclared Area-J oracle failed: the production real-study identifiability "
            "gate remained executed-design-bound but accepted a contradictory analyst-"
            "supplied deterministic output and converted an unidentifiable two-point "
            "compatible region into a positive duration_years identification claim"
        )


if __name__ == "__main__":
    main()
