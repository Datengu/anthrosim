#!/usr/bin/env python3
"""Regression tests for research-monte-carlo-sufficiency.py."""

from __future__ import annotations

import importlib.util
import json
import math
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
SPEC = importlib.util.spec_from_file_location("mc_sufficiency", SCRIPT)
assert SPEC and SPEC.loader
mc = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mc)


def make_plan(kind, batches, threshold, *, pairing="independent", quantile=None, group_batches=None):
    estimand = {
        "kind": kind,
        "confidenceLevel": 0.95,
        "maxHalfWidth": threshold,
    }
    if quantile is not None:
        estimand["quantileProbability"] = quantile
    plan = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": f"synthetic-{kind}",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": estimand,
        "design": (
            {
                "mode": "sequential" if len(group_batches[0]) > 1 else "fixed",
                "groupSeedBatches": group_batches,
            }
            if group_batches is not None
            else {
                "mode": "sequential" if len(batches) > 1 else "fixed",
                "seedBatches": batches,
            }
        ),
        "pairing": pairing,
        "rationale": "Controlled synthetic precision demonstration declared before result inspection.",
    }
    plan["planIdentity"] = mc.plan_identity(plan)
    mc.validate_plan(plan)
    return plan


def sample(groups):
    return {
        "schemaVersion": 1,
        "groups": [
            {
                "id": group_id,
                "replicates": [
                    {"seed": seed, "value": value}
                    for seed, value in rows
                ],
            }
            for group_id, rows in groups
        ],
    }


def assert_raises(fragment, fn):
    try:
        fn()
    except ValueError as error:
        assert fragment in str(error), (fragment, str(error))
    else:
        raise AssertionError(f"expected ValueError containing {fragment!r}")


def mean_sequential_demo():
    plan = make_plan("mean", [[1, 2, 3, 4], list(range(5, 21))], 3.0)
    small = sample([("mean", [(1, 0.0), (2, 20.0), (3, -10.0), (4, 10.0)])])
    first = mc.derive(plan, small, None)
    assert first["precision"]["sufficient"] is False
    assert first["decision"] == "insufficient_continue_with_declared_next_batch"
    assert first["nextDeclaredBatchSeeds"] == list(range(5, 21))

    more_rows = [(1, 0.0), (2, 20.0), (3, -10.0), (4, 10.0)] + [
        (seed, 5.0 + ((seed % 3) - 1) * 0.2) for seed in range(5, 21)
    ]
    larger = sample([("mean", more_rows)])
    second = mc.derive(plan, larger, None)
    assert second["precision"]["sufficient"] is True
    assert second["decision"] == "sufficient_stop"
    assert second["precision"]["halfWidth"] < first["precision"]["halfWidth"]
    assert second["replicateCount"] == 20

    # Exact seed sample + values deterministically reproduces the exact diagnostic.
    assert mc.derive(plan, larger, None) == second

    # Looking after an undeclared partial batch is forbidden: no post-hoc seed-by-seed peeking.
    partial = sample([("mean", more_rows[:5])])
    assert_raises("predeclared cumulative batch boundary", lambda: mc.derive(plan, partial, None))
    return plan, small, larger, first, second


def probability_demo():
    plan = make_plan("probability", [[101, 102, 103, 104], list(range(105, 141))], 0.16)
    first_rows = [(101, 1), (102, 0), (103, 1), (104, 0)]
    first = mc.derive(plan, sample([("persisted", first_rows)]), None)
    assert first["precision"]["precisionMethod"] == "wilson_score_probability"
    assert first["precision"]["sufficient"] is False

    later_rows = first_rows + [(seed, seed % 2) for seed in range(105, 141)]
    second = mc.derive(plan, sample([("persisted", later_rows)]), None)
    assert second["precision"]["sufficient"] is True
    assert second["precision"]["halfWidth"] < first["precision"]["halfWidth"]


def changed_seed_provenance_demo(base_plan, larger):
    changed = make_plan("mean", [[1001, 1002, 1003, 1004], list(range(1005, 1021))], 3.0)
    changed_rows = [
        (seed + 1000, row["value"])
        for seed, row in zip(
            range(1, 21),
            larger["groups"][0]["replicates"],
        )
    ]
    changed_result = mc.derive(changed, sample([("mean", changed_rows)]), None)
    original_result = mc.derive(base_plan, larger, None)
    assert changed["planIdentity"] != base_plan["planIdentity"]
    assert changed_result["seedIdentities"] != original_result["seedIdentities"]
    assert changed_result != original_result


def independent_difference_demo():
    left_seeds = list(range(1, 21))
    right_seeds = list(range(101, 121))
    plan = make_plan(
        "difference_in_means",
        [],
        4.5,
        group_batches=[[left_seeds], [right_seeds]],
    )
    values = [float(index) - 9.5 for index in range(20)]
    result = mc.derive(
        plan,
        sample([
            ("left", list(zip(left_seeds, values))),
            ("right", list(zip(right_seeds, [-value for value in values]))),
        ]),
        None,
    )
    assert result["precision"]["precisionMethod"] == "normal_clt_independent_difference_in_means"
    assert abs(result["precision"]["halfWidth"] - 3.666756860283) < 1e-12
    assert result["precision"]["sufficient"] is True
    assert result["pairingSemantics"] == "independent"
    assert result["seedIdentities"] == left_seeds
    assert result["groupSeedIdentities"]["left"] == left_seeds
    assert result["groupSeedIdentities"]["right"] == right_seeds

    same_seed_plan = dict(plan)
    same_seed_plan["design"] = {"mode": "fixed", "groupSeedBatches": [[left_seeds], [left_seeds]]}
    same_seed_plan["planIdentity"] = mc.plan_identity(same_seed_plan)
    assert_raises("must be disjoint", lambda: mc.validate_plan(same_seed_plan))

    overlapping = dict(plan)
    overlapping["design"] = {"mode": "fixed", "groupSeedBatches": [[left_seeds], [list(range(20, 40))]]}
    overlapping["planIdentity"] = mc.plan_identity(overlapping)
    assert_raises("overlapping seed", lambda: mc.validate_plan(overlapping))

    sequential = make_plan(
        "difference_in_means",
        [],
        0.1,
        group_batches=[[[1, 2], [3, 4]], [[101, 102], [103, 104]]],
    )
    mismatched = sample([
        ("left", [(1, 1.0), (2, 2.0)]),
        ("right", [(101, 1.0), (102, 2.0), (103, 3.0), (104, 4.0)]),
    ])
    assert_raises("same batch boundary", lambda: mc.derive(sequential, mismatched, None))


def paired_covariance_adversaries():
    seeds = list(range(1, 21))
    values = [float(index) - 9.5 for index in range(20)]
    plan = make_plan(
        "paired_mean_difference",
        [seeds],
        4.5,
        pairing="paired_by_seed",
    )
    negative = mc.derive(
        plan,
        sample([
            ("left", list(zip(seeds, values))),
            ("right", list(zip(seeds, [-value for value in values]))),
        ]),
        None,
    )
    assert abs(negative["precision"]["halfWidth"] - 5.185577281736) < 1e-12
    assert negative["precision"]["sufficient"] is False
    assert negative["decision"] == "insufficient_no_predeclared_additional_batch"

    positive = mc.derive(
        plan,
        sample([
            ("left", list(zip(seeds, values))),
            ("right", list(zip(seeds, values))),
        ]),
        None,
    )
    assert positive["precision"]["halfWidth"] == 0.0
    assert positive["precision"]["sufficient"] is True


def paired_demo():
    plan = make_plan(
        "paired_mean_difference",
        [[201, 202, 203, 204, 205, 206]],
        0.5,
        pairing="paired_by_seed",
    )
    left = [(201, 10.0), (202, 11.0), (203, 9.0), (204, 10.5), (205, 10.2), (206, 9.8)]
    right = [(201, 9.0), (202, 10.0), (203, 8.0), (204, 9.5), (205, 9.2), (206, 8.8)]
    result = mc.derive(plan, sample([("treatment", left), ("control", right)]), None)
    assert result["precision"]["precisionMethod"] == "normal_clt_paired_seed_difference"
    assert abs(result["precision"]["estimate"] - 1.0) < 1e-12
    assert result["precision"]["sufficient"] is True


def independent_quantile_coverage(n, probability, lower_rank, upper_rank):
    lower_index = lower_rank - 1
    upper_index = upper_rank - 1
    return math.fsum(
        math.comb(n, k) * probability**k * (1.0 - probability) ** (n - k)
        for k in range(lower_index + 1, upper_index + 1)
    )


def quantile_demo():
    plan = make_plan("quantile", [list(range(301, 321))], 10.0, quantile=0.5)
    rows = [(seed, float(seed - 300)) for seed in range(301, 321)]
    result = mc.derive(plan, sample([("median", rows)]), None)
    precision = result["precision"]
    assert precision["precisionMethod"] == "distribution_free_exact_binomial_order_statistic_interval"
    assert precision["coverageFeasible"] is True
    assert precision["achievedCoverage"] >= 0.95
    assert precision["intervalLower"] <= precision["estimate"] <= precision["intervalUpper"]
    exact = independent_quantile_coverage(
        20, 0.5, precision["lowerOrderStatisticRank"], precision["upperOrderStatisticRank"]
    )
    assert abs(exact - precision["achievedCoverage"]) < 1e-12


def quantile_coverage_adversarial_demo():
    infeasible = [(0.50, 2), (0.90, 8), (0.95, 8), (0.95, 20), (0.99, 20), (0.99, 100)]
    for probability, n in infeasible:
        seeds = list(range(10_000, 10_000 + n))
        plan = make_plan("quantile", [seeds], 1_000_000.0, quantile=probability)
        rows = [(seed, 7.0) for seed in seeds]
        result = mc.derive(plan, sample([("tail", rows)]), None)
        precision = result["precision"]
        assert precision["coverageFeasible"] is False
        assert precision["sufficient"] is False
        assert precision["intervalLower"] is None
        assert precision["intervalUpper"] is None
        assert precision["halfWidth"] is None
        assert precision["maximumAchievableCoverage"] < 0.95
        assert result["decision"] == "insufficient_quantile_coverage_no_predeclared_additional_batch"

    minimum_feasible = [(0.50, 6), (0.90, 29), (0.95, 59), (0.99, 299)]
    for probability, n in minimum_feasible:
        seeds = list(range(20_000, 20_000 + n))
        plan = make_plan("quantile", [seeds], 0.1, quantile=probability)
        rows = [(seed, 7.0) for seed in seeds]
        result = mc.derive(plan, sample([("tail", rows)]), None)
        precision = result["precision"]
        assert precision["coverageFeasible"] is True
        assert precision["achievedCoverage"] >= 0.95
        assert precision["sufficient"] is True
        exact = independent_quantile_coverage(
            n, probability, precision["lowerOrderStatisticRank"], precision["upperOrderStatisticRank"]
        )
        assert abs(exact - precision["achievedCoverage"]) < 1e-12


def quantile_large_n_numeric_stability_demo():
    # The former direct comb(n,k)*float evaluation overflows around this scale.
    support = mc.exact_quantile_rank_interval(1500, 0.5, 0.95)
    assert support["feasible"] is True
    assert support["achievedCoverage"] >= 0.95
    assert support["maximumAchievableCoverage"] > 0.999


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
            "resultIdentity": "synthetic-result-v1",
            "studyExecutionId": "synthetic-study-execution-v1",
            "protocolIdentity": protocol_identity,
            "protocolRevision": 1,
            "studyId": "synthetic-mc-study",
            "scientificStatus": "confirmatory",
            "boundBeforeExecution": True,
            "confirmatoryPreResultClaimEligible": True,
            "definitionIdentity": "synthetic-definition-v1",
            "researchId": "synthetic-research-v1",
            "source": {},
            "researchRelativeDir": "research",
            "runCounts": {"completed": 20, "failed": 0},
            "resultArtifacts": [],
        }
        (study_dir / "study-protocol.json").write_text(json.dumps(protocol), encoding="utf-8")
        (study_dir / "study-result-binding.json").write_text(json.dumps(binding), encoding="utf-8")
        result = mc.derive(plan, larger, study_dir)
        assert result["studyLineage"]["protocolIdentity"] == protocol_identity
        assert result["studyLineage"]["boundBeforeExecution"] is True

        # A different unbound plan cannot be retrofitted after seeing the result.
        changed = dict(plan)
        changed["rationale"] = "Post-result changed rule"
        changed["planIdentity"] = mc.plan_identity(changed)
        assert_raises("does not bind this Monte Carlo precision plan", lambda: mc.derive(changed, larger, study_dir))


def fixed_failure_has_no_posthoc_escape():
    plan = make_plan("mean", [[401, 402, 403, 404]], 0.1)
    rows = [(401, 0.0), (402, 10.0), (403, -10.0), (404, 5.0)]
    result = mc.derive(plan, sample([("mean", rows)]), None)
    assert result["precision"]["sufficient"] is False
    assert result["decision"] == "insufficient_no_predeclared_additional_batch"
    assert result["nextDeclaredBatchSeeds"] == []


def main():
    plan, _small, larger, _first, _second = mean_sequential_demo()
    probability_demo()
    changed_seed_provenance_demo(plan, larger)
    independent_difference_demo()
    paired_covariance_adversaries()
    paired_demo()
    quantile_demo()
    quantile_coverage_adversarial_demo()
    quantile_large_n_numeric_stability_demo()
    frozen_study_binding_demo(plan, larger)
    fixed_failure_has_no_posthoc_escape()
    print("research Monte Carlo sufficiency regression suite passed")


if __name__ == "__main__":
    main()
