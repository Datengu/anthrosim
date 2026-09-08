#!/usr/bin/env python3
"""Monte Carlo confirmatory regressions with producer-valid result bindings."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
ENGINE = HERE / "research-monte-carlo-sufficiency.py"
CONFIRMATORY = HERE / "research-monte-carlo-confirmatory.py"


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_test_research_monte_carlo_confirmatory_legacy",
    HERE / "test-research-monte-carlo-confirmatory-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_write_json = _legacy.write_json


def write_json(path: Path, value) -> None:
    if path.name == "study-result-binding.json" and isinstance(value, dict):
        value = dict(value)
        value.update(
            {
                "schemaVersion": 1,
                "studyExecutionId": value.get(
                    "studyExecutionId", "study-execution-v1-synthetic"
                ),
                "definitionIdentity": value.get(
                    "definitionIdentity", "research-definition-v1-synthetic"
                ),
                "source": value.get(
                    "source",
                    {
                        "modelVersion": "0.3.5",
                        "modelSemanticsId": "anthrosim-model-semantics-v33",
                        "gitCommit": "synthetic-fixture",
                    },
                ),
                "researchRelativeDir": value.get("researchRelativeDir", "research"),
                "runCounts": value.get("runCounts", {"completed": 40, "failed": 0}),
                "resultArtifacts": value.get("resultArtifacts", []),
            }
        )
        value["resultIdentity"] = "pending"
        value["resultIdentity"] = _binding.result_identity(value)
    _original_write_json(path, value)


_legacy.write_json = write_json

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"write_json", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


def guarded_main() -> None:
    """Keep the seed-binding regression scientifically valid under the repaired mean gate."""
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        study = root / "study"
        study.mkdir()
        plan_path = root / "plan.json"
        samples_path = root / "samples.json"
        output_path = root / "diagnostic.json"
        seeds = list(range(11, 51))

        plan = {
            "schemaVersion": 1,
            "planIdentity": "",
            "planId": "confirmatory-seed-binding",
            "uncertaintyCategory": "process_stochastic_monte_carlo",
            "estimand": {
                "kind": "mean",
                "confidenceLevel": 0.95,
                "maxHalfWidth": 20.0,
            },
            "design": {"mode": "fixed", "seedBatches": [seeds]},
            "pairing": "independent",
            "rationale": "Synthetic exact frozen seed binding test above the guarded normal-CLT sample floor.",
        }
        write_json(plan_path, plan)
        plan["planIdentity"] = _legacy.plan_identity(plan_path)
        write_json(plan_path, plan)

        protocol = {
            "schemaVersion": 1,
            "protocolRevision": 1,
            "studyId": "synthetic-seed-binding",
            "status": "confirmatory",
            "researchQuestion": "Is the planned Monte Carlo sample exactly the executed frozen sample?",
            "applicabilityDomain": "Synthetic verification",
            "hypotheses": [],
            "analysisWindows": [],
            "observables": [],
            "comparisons": [],
            "evidenceRoles": [],
            "uncertainty": {"parameterUncertainty": [], "structuralUncertainty": []},
            "ensemblePolicy": {
                "seedPolicy": "Exact ordered frozen seeds",
                "pairingPolicy": "Independent",
                "replicationPolicy": "monte-carlo-precision-plan-v1:" + plan["planIdentity"],
            },
            "runHandling": {
                "stoppingRules": [],
                "exclusionRules": [],
                "censoringRules": [],
            },
            "sensitivityPlan": [],
            "equifinalityPlan": [],
            "manipulationChecks": [],
            "analysisMethod": "Synthetic",
            "multiplicityPolicy": "One estimand",
            "heldOutCorroboration": [],
            "permittedInterpretations": [],
            "prohibitedInterpretations": [],
        }
        write_json(study / "study-protocol.json", protocol)
        write_json(
            study / "study-result-binding.json",
            {
                "protocolIdentity": _legacy.protocol_identity(protocol),
                "protocolRevision": 1,
                "studyId": "synthetic-seed-binding",
                "resultIdentity": "synthetic-result",
                "researchId": "synthetic-research",
                "scientificStatus": "confirmatory",
                "boundBeforeExecution": True,
                "confirmatoryPreResultClaimEligible": True,
            },
        )
        write_json(study / "research-definition.json", {"seeds": seeds})
        write_json(
            samples_path,
            {
                "schemaVersion": 1,
                "groups": [
                    {
                        "id": "mean",
                        "replicates": [
                            {
                                "seed": seed,
                                "value": 10.0 + ((index % 5) - 2) * 0.1,
                            }
                            for index, seed in enumerate(seeds)
                        ],
                    }
                ],
            },
        )

        accepted = subprocess.run(
            [
                sys.executable,
                str(CONFIRMATORY),
                str(plan_path),
                str(samples_path),
                str(output_path),
                "--study-dir",
                str(study),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert accepted.returncode == 0, (accepted.stdout, accepted.stderr)
        diagnostic = json.loads(output_path.read_text(encoding="utf-8"))
        assert diagnostic["seedIdentities"] == seeds
        assert diagnostic["precision"]["normalApproximationValidity"]["validForStopping"] is True

        changed_seeds = seeds[:-1] + [999]
        write_json(study / "research-definition.json", {"seeds": changed_seeds})
        rejected = subprocess.run(
            [
                sys.executable,
                str(CONFIRMATORY),
                str(plan_path),
                str(samples_path),
                str(output_path),
                "--study-dir",
                str(study),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert rejected.returncode == 1
        assert "do not exactly equal" in rejected.stderr

    print("confirmatory Monte Carlo frozen-seed binding suite passed")


if __name__ == "__main__":
    guarded_main()
