#!/usr/bin/env python3
"""Regression coverage for AV4-011 design binding plus AV6-012 output binding."""

from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_TEST = ROOT / "scripts" / "test-research-identifiability-executed-design-binding-core.py"
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
BINDER = ROOT / "scripts" / "research-identifiability-bind-design.py"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


core = load_module(CORE_TEST, "anthrosim_identifiability_av4_011_core_test")
analyzer = load_module(ANALYZER, "anthrosim_identifiability_av6_012_test")
binder = load_module(BINDER, "anthrosim_identifiability_binder_av6_012_test")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def completion_data(binding: dict, values: list[float]) -> dict:
    assert len(binding["points"]) == len(values)
    points = []
    for bound, value in zip(binding["points"], values, strict=True):
        points.append(
            {
                "id": bound["id"],
                "parameters": copy.deepcopy(bound["parameters"]),
                "structure": bound["structure"],
                "executionIds": list(bound["executionIds"]),
                "covariates": {
                    "duration_bucket": "short"
                    if bound["parameters"]["duration_years"] == 1
                    else "long"
                },
                "outputs": {"all_executions_completed": value},
                "outputEvidence": {
                    "all_executions_completed": {"kind": "deterministic"}
                },
            }
        )
    return {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}


def completion_plan() -> dict:
    return {
        "schemaVersion": 2,
        "analysisId": "av6-012-authoritative-completion-binding",
        "calibrationTargets": [
            {
                "observable": "all_executions_completed",
                "target": 1.0,
                "tolerance": 0.0,
            }
        ],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["duration_years"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.0,
    }


def manifest_scalar_data(binding: dict, value: float) -> dict:
    points = []
    for bound in binding["points"]:
        points.append(
            {
                "id": bound["id"],
                "parameters": copy.deepcopy(bound["parameters"]),
                "structure": bound["structure"],
                "executionIds": list(bound["executionIds"]),
                "outputs": {"initial_population": value},
                "outputEvidence": {
                    "initial_population": {
                        "kind": "deterministic",
                        "derivation": {
                            "kind": "run_manifest_scalar_v1",
                            "jsonPointer": "/population/initialPopulation",
                            "reducer": "require_equal_v1",
                        },
                    }
                },
            }
        )
    return {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}


def manifest_scalar_plan() -> dict:
    return {
        "schemaVersion": 2,
        "analysisId": "av6-012-run-manifest-scalar-binding",
        "calibrationTargets": [
            {"observable": "initial_population", "target": 20.0, "tolerance": 0.0}
        ],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["duration_years"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.0,
    }


def real_runner_regression(research_binary: Path) -> None:
    binary = research_binary.resolve(strict=True)
    with tempfile.TemporaryDirectory() as temp:
        temp_root = Path(temp)
        definition_path = temp_root / "definition.json"
        research_root = temp_root / "research-run"
        write(definition_path, core.tiny_definition())
        subprocess.run(
            [str(binary), "--definition", str(definition_path), "--run-dir", str(research_root)],
            check=True,
        )

        binding = binder.derive_from_root(research_root)
        assert binding["sourceKind"] == "anthrosim_research_manifest_v1"
        assert len(binding["points"]) == 2
        assert all(len(point["executionIds"]) == 2 for point in binding["points"])
        assert [point["parameters"]["duration_years"] for point in binding["points"]] == [1, 2]

        plan = completion_plan()
        truthful_data = completion_data(binding, [1.0, 1.0])
        truthful = analyzer.analyse_with_research_root(plan, truthful_data, research_root)
        assert truthful["researchGate"]["passes"] is False
        assert truthful["researchGate"]["executedDesignBound"] is True
        assert truthful["researchGate"]["authoritativeOutputsBound"] is True
        assert truthful["authoritativeOutputBinding"]["valid"] is True
        assert truthful["compatibleRegion"]["pointCount"] == 2
        duration = next(
            item for item in truthful["parameterDiagnostics"] if item["parameter"] == "duration_years"
        )
        assert duration["identified"] is False

        # Exact AV6-012 attack: same immutable root/design/executions; one downstream value changes.
        contradictory_data = completion_data(binding, [1.0, 0.0])
        contradictory = analyzer.analyse_with_research_root(
            plan, contradictory_data, research_root
        )
        assert contradictory["researchGate"]["passes"] is False
        assert contradictory["researchGate"]["reason"] == "authoritative_output_binding_invalid"
        assert contradictory["researchGate"]["executedDesignBound"] is True
        assert contradictory["researchGate"]["authoritativeOutputsBound"] is False
        assert contradictory["authoritativeOutputBinding"]["valid"] is False
        assert any(
            error.endswith("authoritative_value_mismatch")
            for error in contradictory["authoritativeOutputBinding"]["validationErrors"]
        )
        duration = next(
            item
            for item in contradictory["parameterDiagnostics"]
            if item["parameter"] == "duration_years"
        )
        assert duration["identified"] is False
        assert contradictory["profiles"] == {}
        assert contradictory["pairwiseInteractionSurfaces"] == []

        # General v1 deterministic derivation: the value is copied from every exact bound child
        # manifest and require_equal_v1 prevents a multi-seed table from silently selecting one run.
        scalar_data = manifest_scalar_data(binding, 20.0)
        scalar = analyzer.analyse_with_research_root(
            manifest_scalar_plan(), scalar_data, research_root
        )
        assert scalar["researchGate"]["authoritativeOutputsBound"] is True
        assert scalar["authoritativeOutputBinding"]["valid"] is True
        derivations = scalar["authoritativeOutputBinding"]["derivations"]
        assert len(derivations) == 2
        assert all(item["authoritativeValue"] == 20 for item in derivations)
        assert all(len(item["artifacts"]) == 2 for item in derivations)
        assert all(
            item["derivation"]["jsonPointer"] == "/population/initialPopulation"
            for item in derivations
        )

        scalar_tampered = copy.deepcopy(scalar_data)
        scalar_tampered["points"][1]["outputs"]["initial_population"] = 21.0
        rejected_scalar = analyzer.analyse_with_research_root(
            manifest_scalar_plan(), scalar_tampered, research_root
        )
        assert rejected_scalar["researchGate"]["passes"] is False
        assert rejected_scalar["researchGate"]["reason"] == "authoritative_output_binding_invalid"

        # Bare deterministic evidence is retained only for the canonical completion observable.
        unbound = copy.deepcopy(scalar_data)
        for point in unbound["points"]:
            point["outputEvidence"]["initial_population"] = {"kind": "deterministic"}
        rejected_unbound = analyzer.analyse_with_research_root(
            manifest_scalar_plan(), unbound, research_root
        )
        assert rejected_unbound["researchGate"]["passes"] is False
        assert any(
            error.endswith("deterministic_output_missing_authoritative_derivation")
            for error in rejected_unbound["authoritativeOutputBinding"]["validationErrors"]
        )

        # Preserve the AV4-011 coordinate/structure/execution firewall independently of output
        # authority. These must fail before a downstream output is interpreted.
        altered = copy.deepcopy(truthful_data)
        altered["points"][0]["parameters"]["duration_years"] = 999
        rejected = analyzer.analyse_with_research_root(plan, altered, research_root)
        assert rejected["researchGate"]["reason"] == "executed_design_binding_invalid"
        assert rejected["researchGate"]["executedDesignBound"] is False

        rebound = copy.deepcopy(truthful_data)
        rebound["points"][0]["executionIds"] = rebound["points"][1]["executionIds"]
        rejected = analyzer.analyse_with_research_root(plan, rebound, research_root)
        assert rejected["researchGate"]["reason"] == "executed_design_binding_invalid"

        structural = copy.deepcopy(truthful_data)
        structural["points"][0]["structure"] = "fabricated_structure"
        rejected = analyzer.analyse_with_research_root(plan, structural, research_root)
        assert rejected["researchGate"]["reason"] == "executed_design_binding_invalid"

        covariate_plan = copy.deepcopy(plan)
        covariate_plan["analysisId"] = "av4-011-real-derived-covariate-role"
        covariate_plan["claim"] = {
            "parameterIds": ["duration_bucket"],
            "structuralHypothesis": False,
        }
        try:
            analyzer.analyse_with_research_root(covariate_plan, truthful_data, research_root)
        except analyzer.IdentifiabilityError as error:
            assert "claimed parameters are not present" in str(error)
        else:
            raise AssertionError("real-run derived covariate must not become a model parameter")

        # Coordinated edits to redundant immutable metadata remain insufficient to create a new
        # accepted design because the executed-design binder independently reproduces identities.
        manifest = load(research_root / "research-manifest.json")
        manifest["points"][0]["point"]["coordinates"][0]["value"] = 999
        write(research_root / "research-manifest.json", manifest)
        write(research_root / "research-plan.json", manifest)
        try:
            binder.derive_from_root(research_root)
        except binder.BindingError:
            pass
        else:
            raise AssertionError("coordinated immutable-metadata tampering must fail identity validation")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--research-binary", type=Path)
    args = parser.parse_args()
    core.synthetic_regression()
    if args.research_binary is not None:
        real_runner_regression(args.research_binary)
        print("AV4-011/AV6-012 binding regression: synthetic + real runner ok")
    else:
        print("AV4-011/AV6-012 binding regression: synthetic ok")


if __name__ == "__main__":
    main()
