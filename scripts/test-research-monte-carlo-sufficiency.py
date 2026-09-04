#!/usr/bin/env python3
"""Monte Carlo sufficiency regressions with a producer-valid synthetic study binding."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_test_research_monte_carlo_sufficiency_legacy",
    HERE / "test-research-monte-carlo-sufficiency-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
mc = _legacy.mc


def frozen_study_binding_demo(plan, larger):
    with tempfile.TemporaryDirectory() as temporary:
        study_dir = Path(temporary)
        protocol = {
            "schemaVersion": 1,
            "protocolRevision": 1,
            "studyId": "synthetic-mc-study",
            "status": "confirmatory",
            "researchQuestion": "Does the declared stochastic estimand meet its predeclared Monte Carlo precision target?",
            "applicabilityDomain": "Synthetic verification only",
            "hypotheses": [],
            "analysisWindows": [],
            "observables": [],
            "comparisons": [],
            "evidenceRoles": [],
            "uncertainty": {"parameterUncertainty": [], "structuralUncertainty": []},
            "ensemblePolicy": {
                "seedPolicy": "Exact ordered seeds are declared by the bound Monte Carlo precision plan.",
                "pairingPolicy": "Independent for this synthetic mean.",
                "replicationPolicy": mc.PLAN_PREFIX + plan["planIdentity"],
            },
            "runHandling": {"stoppingRules": [], "exclusionRules": [], "censoringRules": []},
            "sensitivityPlan": [],
            "equifinalityPlan": [],
            "manipulationChecks": [],
            "analysisMethod": "Synthetic mean precision diagnostic",
            "multiplicityPolicy": "One estimand",
            "heldOutCorroboration": [],
            "permittedInterpretations": [],
            "prohibitedInterpretations": [],
        }
        protocol_identity = mc.study_protocol_identity(protocol)
        binding = {
            "schemaVersion": 1,
            "resultIdentity": "pending",
            "studyExecutionId": "study-execution-v1-synthetic-mc",
            "protocolIdentity": protocol_identity,
            "protocolRevision": 1,
            "studyId": "synthetic-mc-study",
            "scientificStatus": "confirmatory",
            "boundBeforeExecution": True,
            "confirmatoryPreResultClaimEligible": True,
            "definitionIdentity": "research-definition-v1-synthetic-mc",
            "researchId": "research-execution-v1-synthetic-mc",
            "source": {
                "modelVersion": "0.3.4",
                "modelSemanticsId": "anthrosim-model-semantics-v33",
                "gitCommit": "synthetic-fixture",
            },
            "researchRelativeDir": "research",
            "runCounts": {"completed": 20, "failed": 0},
            "resultArtifacts": [],
        }
        binding["resultIdentity"] = _binding.result_identity(binding)
        (study_dir / "study-protocol.json").write_text(
            json.dumps(protocol), encoding="utf-8"
        )
        (study_dir / "study-result-binding.json").write_text(
            json.dumps(binding), encoding="utf-8"
        )
        result = mc.derive(plan, larger, study_dir)
        assert result["studyLineage"]["protocolIdentity"] == protocol_identity
        assert result["studyLineage"]["boundBeforeExecution"] is True
        assert result["studyLineage"]["studyResultIdentity"] == binding["resultIdentity"]

        changed = dict(plan)
        changed["rationale"] = "Post-result changed rule"
        changed["planIdentity"] = mc.plan_identity(changed)
        _legacy.assert_raises(
            "does not bind this Monte Carlo precision plan",
            lambda: mc.derive(changed, larger, study_dir),
        )


_legacy.frozen_study_binding_demo = frozen_study_binding_demo

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"frozen_study_binding_demo", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    _legacy.main()
