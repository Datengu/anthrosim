#!/usr/bin/env python3
"""Regression coverage for AV4-011 executed-design identifiability binding."""

from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
BINDER = ROOT / "scripts" / "research-identifiability-bind-design.py"
BENCHMARK_PLAN = ROOT / "research" / "identifiability-benchmark-v1" / "plan.json"
BENCHMARK_DATA = ROOT / "research" / "identifiability-benchmark-v1" / "data.json"
BASE_DEFINITION = ROOT / "research" / "general-demography-baseline-v1" / "confirmatory-definition.json"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


analyzer = load_module(ANALYZER, "anthrosim_identifiability_av4_011_test")
binder = load_module(BINDER, "anthrosim_identifiability_binder_av4_011_test")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def synthetic_regression() -> None:
    plan = load(BENCHMARK_PLAN)
    data = load(BENCHMARK_DATA)
    baseline = analyzer.analyse(plan, data)
    assert baseline["researchGate"]["passes"] is True
    assert baseline["researchGate"]["executedDesignBound"] is True
    assert baseline["executedDesignBinding"]["sourceKind"] == "synthetic_fixture"
    assert baseline["compatibleRegion"]["pointIds"] == ["o2-n2"]

    fabricated = copy.deepcopy(data)
    for point in fabricated["points"]:
        point["parameters"]["fabricated_theta"] = (
            0 if point["id"] == "o2-n2" else 1
        )
    fabricated_plan = copy.deepcopy(plan)
    fabricated_plan["analysisId"] = "av4-011-fabricated-parameter-regression"
    fabricated_plan["claim"] = {
        "parameterIds": ["fabricated_theta"],
        "structuralHypothesis": False,
    }
    rejected = analyzer.analyse(fabricated_plan, fabricated)
    diagnostic = next(
        item
        for item in rejected["parameterDiagnostics"]
        if item["parameter"] == "fabricated_theta"
    )
    assert rejected["researchGate"]["passes"] is False
    assert rejected["researchGate"]["reason"] == "executed_design_binding_invalid"
    assert rejected["researchGate"]["executedDesignBound"] is False
    assert diagnostic["identified"] is False
    assert diagnostic["reason"] == "parameter_not_bound_to_executed_design"
    assert diagnostic["fullRange"] is None
    assert diagnostic["compatibleRange"] is None
    assert rejected["profiles"] == {}
    assert rejected["pairwiseInteractionSurfaces"] == []

    altered = copy.deepcopy(data)
    altered["points"][0]["parameters"]["opportunity_scale"] = 999
    rejected = analyzer.analyse(plan, altered)
    assert rejected["researchGate"]["passes"] is False
    assert "synthetic_fixture_design_digest_mismatch" in rejected["executedDesignBinding"]["validationErrors"]

    structure = copy.deepcopy(data)
    structure["points"][0]["structure"] = "fabricated_structure"
    rejected = analyzer.analyse(plan, structure)
    assert rejected["researchGate"]["passes"] is False

    rebound = copy.deepcopy(data)
    rebound["points"][0]["id"] = "fabricated-point-id"
    rejected = analyzer.analyse(plan, rebound)
    assert rejected["researchGate"]["passes"] is False

    covariate = copy.deepcopy(data)
    for point in covariate["points"]:
        point["covariates"] = {"fabricated_theta": point["parameters"]["opportunity_scale"]}
    covariate_plan = copy.deepcopy(plan)
    covariate_plan["analysisId"] = "av4-011-derived-covariate-role"
    covariate_plan["claim"] = {
        "parameterIds": ["fabricated_theta"],
        "structuralHypothesis": False,
    }
    try:
        analyzer.analyse(covariate_plan, covariate)
    except analyzer.IdentifiabilityError as error:
        assert "claimed parameters are not present" in str(error)
    else:
        raise AssertionError("a derived covariate must not be promotable to a model-parameter claim")


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


def real_runner_regression(research_binary: Path) -> None:
    binary = research_binary.resolve(strict=True)
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
        assert all(len(point["executionIds"]) == 2 for point in binding["points"])
        assert [point["parameters"]["duration_years"] for point in binding["points"]] == [1, 2]

        points = []
        for index, bound in enumerate(binding["points"]):
            outputs = {"score": float(index)}
            points.append(
                {
                    "id": bound["id"],
                    "parameters": copy.deepcopy(bound["parameters"]),
                    "structure": bound["structure"],
                    "executionIds": list(bound["executionIds"]),
                    "covariates": {"duration_bucket": "short" if index == 0 else "long"},
                    "outputs": outputs,
                    "outputEvidence": {"score": {"kind": "deterministic"}},
                }
            )
        data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
        plan = {
            "schemaVersion": 2,
            "analysisId": "av4-011-real-runner-binding-control",
            "calibrationTargets": [
                {"observable": "score", "target": 0.0, "tolerance": 0.0}
            ],
            "corroborationObservables": [],
            "claim": {
                "parameterIds": ["duration_years"],
                "structuralHypothesis": False,
            },
            "maxNormalizedAcceptableWidth": 0.0,
        }
        result = analyzer.analyse_with_research_root(plan, data, research_root)
        assert result["researchGate"]["passes"] is True
        assert result["researchGate"]["executedDesignBound"] is True
        assert result["executedDesignBinding"]["researchId"] == binding["researchId"]
        assert result["executedDesignBinding"]["definitionIdentity"] == binding["definitionIdentity"]
        assert result["executedDesignBinding"]["executionCount"] == 4
        duration = next(
            item
            for item in result["parameterDiagnostics"]
            if item["parameter"] == "duration_years"
        )
        assert duration["identified"] is True
        assert duration["fullRange"] == [1.0, 2.0]
        assert duration["compatibleRange"] == [1.0, 1.0]

        altered = copy.deepcopy(data)
        altered["points"][0]["parameters"]["duration_years"] = 999
        rejected = analyzer.analyse_with_research_root(plan, altered, research_root)
        assert rejected["researchGate"]["passes"] is False
        assert any(
            error.endswith("parameter_coordinate_mismatch")
            for error in rejected["executedDesignBinding"]["validationErrors"]
        )

        rebound = copy.deepcopy(data)
        rebound["points"][0]["executionIds"] = rebound["points"][1]["executionIds"]
        rejected = analyzer.analyse_with_research_root(plan, rebound, research_root)
        assert rejected["researchGate"]["passes"] is False
        assert any(
            error.endswith("execution_identity_mismatch")
            for error in rejected["executedDesignBinding"]["validationErrors"]
        )

        structural = copy.deepcopy(data)
        structural["points"][0]["structure"] = "fabricated_structure"
        rejected = analyzer.analyse_with_research_root(plan, structural, research_root)
        assert rejected["researchGate"]["passes"] is False
        assert any(
            error.endswith("structure_coordinate_mismatch")
            for error in rejected["executedDesignBinding"]["validationErrors"]
        )

        covariate_plan = copy.deepcopy(plan)
        covariate_plan["analysisId"] = "av4-011-real-derived-covariate-role"
        covariate_plan["claim"] = {
            "parameterIds": ["duration_bucket"],
            "structuralHypothesis": False,
        }
        try:
            analyzer.analyse_with_research_root(covariate_plan, data, research_root)
        except analyzer.IdentifiabilityError as error:
            assert "claimed parameters are not present" in str(error)
        else:
            raise AssertionError("real-run derived covariate must not become a model parameter")

        # A coordinated edit of both redundant metadata copies is not enough:
        # the binder independently recomputes point/run/definition/execution identities.
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
    synthetic_regression()
    if args.research_binary is not None:
        real_runner_regression(args.research_binary)
        print("AV4-011 executed-design binding regression: synthetic + real runner ok")
    else:
        print("AV4-011 executed-design binding regression: synthetic ok")


if __name__ == "__main__":
    main()
