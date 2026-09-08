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
                "modelVersion": "0.3.5",
                "modelSemanticsId": "anthrosim-model-semantics-v33",
                "gitCommit": "synthetic-fixture",
            },
            "researchRelativeDir": "research",
            "runCounts": {"completed": 40, "failed": 0},
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


def guarded_mean_sequential_demo():
    plan = _legacy.make_plan("mean", [[1, 2, 3, 4], list(range(5, 41))], 3.0)
    small = _legacy.sample([("mean", [(1, 0.0), (2, 20.0), (3, -10.0), (4, 10.0)])])
    first = mc.derive(plan, small, None)
    validity = first["precision"]["normalApproximationValidity"]
    assert validity["validForStopping"] is False
    assert "replicate_count_below_asymptotic_floor" in validity["reasons"]
    assert first["decision"] == "insufficient_continue_with_declared_next_batch"

    more_rows = [(1, 0.0), (2, 20.0), (3, -10.0), (4, 10.0)] + [
        (seed, 5.0 + ((seed % 3) - 1) * 0.2) for seed in range(5, 41)
    ]
    larger = _legacy.sample([("mean", more_rows)])
    second = mc.derive(plan, larger, None)
    assert second["precision"]["normalApproximationValidity"]["validForStopping"] is True
    assert second["precision"]["sufficient"] is True
    assert second["decision"] == "sufficient_stop"
    assert second["replicateCount"] == 40
    assert mc.derive(plan, larger, None) == second

    partial = _legacy.sample([("mean", more_rows[:5])])
    _legacy.assert_raises(
        "predeclared cumulative batch boundary", lambda: mc.derive(plan, partial, None)
    )
    return plan, small, larger, first, second


def guarded_independent_difference_demo():
    left_seeds = list(range(1, 41))
    right_seeds = list(range(101, 141))
    plan = _legacy.make_plan(
        "difference_in_means",
        [],
        1.0,
        group_batches=[[left_seeds], [right_seeds]],
    )
    values = [float((index % 5) - 2) for index in range(40)]
    result = mc.derive(
        plan,
        _legacy.sample([
            ("left", list(zip(left_seeds, values))),
            ("right", list(zip(right_seeds, [-value for value in values]))),
        ]),
        None,
    )
    assert result["precision"]["precisionMethod"] == "normal_clt_independent_difference_in_means"
    assert result["precision"]["normalApproximationValidity"]["validForStopping"] is True
    assert result["precision"]["sufficient"] is True
    assert result["pairingSemantics"] == "independent"

    same_seed_plan = dict(plan)
    same_seed_plan["design"] = {
        "mode": "fixed",
        "groupSeedBatches": [[left_seeds], [left_seeds]],
    }
    same_seed_plan["planIdentity"] = mc.plan_identity(same_seed_plan)
    _legacy.assert_raises("must be disjoint", lambda: mc.validate_plan(same_seed_plan))


def guarded_paired_covariance_adversaries():
    seeds = list(range(1, 41))
    values = [float(index) - 19.5 for index in range(40)]
    plan = _legacy.make_plan(
        "paired_mean_difference", [seeds], 6.0, pairing="paired_by_seed"
    )
    negative = mc.derive(
        plan,
        _legacy.sample([
            ("left", list(zip(seeds, values))),
            ("right", list(zip(seeds, [-value for value in values]))),
        ]),
        None,
    )
    assert negative["precision"]["normalApproximationValidity"]["validForStopping"] is True
    assert negative["precision"]["sufficient"] is False

    positive = mc.derive(
        plan,
        _legacy.sample([
            ("left", list(zip(seeds, values))),
            ("right", list(zip(seeds, values))),
        ]),
        None,
    )
    validity = positive["precision"]["normalApproximationValidity"]
    assert positive["precision"]["halfWidth"] == 0.0
    assert validity["validForStopping"] is False
    assert "zero_observed_variance_cannot_establish_determinism" in validity["reasons"]
    assert positive["precision"]["sufficient"] is False


def guarded_paired_demo():
    seeds = list(range(201, 241))
    left = [(seed, 10.0 + (index % 5) * 0.1) for index, seed in enumerate(seeds)]
    right = [
        (seed, value - (0.9 + (index % 3) * 0.1))
        for index, (seed, value) in enumerate(left)
    ]
    plan = _legacy.make_plan(
        "paired_mean_difference", [seeds], 0.1, pairing="paired_by_seed"
    )
    result = mc.derive(
        plan, _legacy.sample([("treatment", left), ("control", right)]), None
    )
    assert result["precision"]["precisionMethod"] == "normal_clt_paired_seed_difference"
    assert result["precision"]["normalApproximationValidity"]["validForStopping"] is True
    assert result["precision"]["sufficient"] is True


def small_n_zero_variance_adversary():
    plan = _legacy.make_plan(
        "mean", [[1, 2], list(range(3, 31))], 0.1
    )
    result = mc.derive(
        plan, _legacy.sample([("mean", [(1, 0.0), (2, 0.0)])]), None
    )
    validity = result["precision"]["normalApproximationValidity"]
    assert result["precision"]["intervalLower"] == 0.0
    assert result["precision"]["intervalUpper"] == 0.0
    assert result["precision"]["halfWidth"] == 0.0
    assert result["precision"]["sufficient"] is False
    assert result["decision"] == "insufficient_continue_with_declared_next_batch"
    assert result["nextDeclaredBatchSeeds"] == list(range(3, 31))
    assert validity["validForStopping"] is False
    assert validity["minimumReplicatesPerGroup"] == 30
    assert validity["observedReplicatesPerGroup"] == [2]
    assert "replicate_count_below_asymptotic_floor" in validity["reasons"]
    assert "zero_observed_variance_cannot_establish_determinism" in validity["reasons"]
    assert "not a finite-sample distribution-free coverage guarantee" in result["precision"]["confidenceSemantics"]


_legacy.frozen_study_binding_demo = frozen_study_binding_demo
_legacy.mean_sequential_demo = guarded_mean_sequential_demo
_legacy.independent_difference_demo = guarded_independent_difference_demo
_legacy.paired_covariance_adversaries = guarded_paired_covariance_adversaries
_legacy.paired_demo = guarded_paired_demo

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"frozen_study_binding_demo", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


def main():
    _legacy.main()
    small_n_zero_variance_adversary()
    print("guarded normal-CLT validity regressions passed")


if __name__ == "__main__":
    main()
