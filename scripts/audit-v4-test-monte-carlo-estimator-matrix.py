#!/usr/bin/env python3
"""Fresh Audit-v4 Area-H estimator and seed-contract matrix."""

from __future__ import annotations

import importlib.util
import math
from pathlib import Path
from statistics import NormalDist

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
SPEC = importlib.util.spec_from_file_location("mc_sufficiency", SCRIPT)
assert SPEC and SPEC.loader
mc = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mc)

CONFIDENCE = 0.95
Z = NormalDist().inv_cdf(0.5 + CONFIDENCE / 2.0)


def make_plan(kind, *, seeds=None, group_seeds=None, pairing="independent", threshold=1_000_000.0, quantile=None, mode="fixed"):
    estimand = {
        "kind": kind,
        "confidenceLevel": CONFIDENCE,
        "maxHalfWidth": threshold,
    }
    if quantile is not None:
        estimand["quantileProbability"] = quantile
    design = {"mode": mode}
    if group_seeds is None:
        design["seedBatches"] = seeds
    else:
        design["groupSeedBatches"] = group_seeds
    plan = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": f"audit-v4-{kind}",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": estimand,
        "design": design,
        "pairing": pairing,
        "rationale": "Fresh Audit-v4 estimator and seed-contract matrix.",
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


def mean_variance(values):
    n = len(values)
    mean = math.fsum(values) / n
    variance = math.fsum((value - mean) ** 2 for value in values) / (n - 1)
    return mean, variance


def normal_half(values):
    _, variance = mean_variance(values)
    return Z * math.sqrt(variance / len(values))


def independent_half(left, right):
    _, left_variance = mean_variance(left)
    _, right_variance = mean_variance(right)
    return Z * math.sqrt(left_variance / len(left) + right_variance / len(right))


def wilson(values):
    n = len(values)
    estimate = math.fsum(values) / n
    denominator = 1.0 + Z * Z / n
    center = (estimate + Z * Z / (2.0 * n)) / denominator
    raw_half = Z * math.sqrt(
        estimate * (1.0 - estimate) / n + Z * Z / (4.0 * n * n)
    ) / denominator
    lower = max(0.0, center - raw_half)
    upper = min(1.0, center + raw_half)
    half = max(estimate - lower, upper - estimate)
    return estimate, lower, upper, half


def exact_binomial_interval_coverage(n, probability, lower_rank, upper_rank):
    lower_index = lower_rank - 1
    upper_index = upper_rank - 1
    return math.fsum(
        math.comb(n, k) * probability**k * (1.0 - probability) ** (n - k)
        for k in range(lower_index + 1, upper_index + 1)
    )


def assert_close(actual, expected, *, tolerance=1e-12, label="value"):
    if not math.isclose(actual, expected, rel_tol=tolerance, abs_tol=tolerance):
        raise AssertionError(f"{label}: actual={actual!r} expected={expected!r}")


def assert_raises(fragment, fn):
    try:
        fn()
    except ValueError as error:
        if fragment not in str(error):
            raise AssertionError(f"expected {fragment!r}, got {str(error)!r}") from error
    else:
        raise AssertionError(f"expected ValueError containing {fragment!r}")


def main():
    comparisons = 0
    maximum_error = 0.0
    n = 64
    shared_seeds = list(range(1, n + 1))
    right_seeds = list(range(10_001, 10_001 + n))

    for pattern in range(32):
        mean_values = [
            float(((index * 17 + pattern * 11 + index * pattern * 3) % 101) - 50) / 4.0
            for index in range(n)
        ]
        mean_plan = make_plan("mean", seeds=[shared_seeds])
        mean_result = mc.derive(
            mean_plan,
            sample([("mean", list(zip(shared_seeds, mean_values)))]),
            None,
        )
        expected_mean, _ = mean_variance(mean_values)
        expected_half = normal_half(mean_values)
        assert_close(mean_result["precision"]["estimate"], expected_mean, label=f"mean estimate pattern {pattern}")
        assert_close(mean_result["precision"]["halfWidth"], expected_half, label=f"mean half pattern {pattern}")
        maximum_error = max(maximum_error, abs(mean_result["precision"]["halfWidth"] - expected_half))
        comparisons += 1

        left = [
            float(((index * 19 + pattern * 7 + index * pattern) % 89) - 44) / 5.0
            for index in range(n)
        ]
        right = [
            float(((index * 13 + pattern * 17 + index * pattern * 2) % 83) - 41) / 6.0
            for index in range(n)
        ]
        independent_plan = make_plan(
            "difference_in_means",
            group_seeds=[[shared_seeds], [right_seeds]],
        )
        independent_result = mc.derive(
            independent_plan,
            sample([
                ("left", list(zip(shared_seeds, left))),
                ("right", list(zip(right_seeds, right))),
            ]),
            None,
        )
        expected_difference = mean_variance(left)[0] - mean_variance(right)[0]
        expected_independent_half = independent_half(left, right)
        assert_close(independent_result["precision"]["estimate"], expected_difference, label=f"independent estimate pattern {pattern}")
        assert_close(independent_result["precision"]["halfWidth"], expected_independent_half, label=f"independent half pattern {pattern}")
        assert independent_result["pairingSemantics"] == "independent"
        assert independent_result["groupSeedIdentities"]["left"] == shared_seeds
        assert independent_result["groupSeedIdentities"]["right"] == right_seeds
        maximum_error = max(maximum_error, abs(independent_result["precision"]["halfWidth"] - expected_independent_half))
        comparisons += 1

        paired_plan = make_plan(
            "paired_mean_difference",
            seeds=[shared_seeds],
            pairing="paired_by_seed",
        )
        paired_result = mc.derive(
            paired_plan,
            sample([
                ("left", list(zip(shared_seeds, left))),
                ("right", list(zip(shared_seeds, right))),
            ]),
            None,
        )
        differences = [a - b for a, b in zip(left, right)]
        expected_paired_mean, _ = mean_variance(differences)
        expected_paired_half = normal_half(differences)
        assert_close(paired_result["precision"]["estimate"], expected_paired_mean, label=f"paired estimate pattern {pattern}")
        assert_close(paired_result["precision"]["halfWidth"], expected_paired_half, label=f"paired half pattern {pattern}")
        maximum_error = max(maximum_error, abs(paired_result["precision"]["halfWidth"] - expected_paired_half))
        comparisons += 1

        probability_values = [1 if ((index * 7 + pattern * 5) % 13) < (2 + pattern % 9) else 0 for index in range(n)]
        probability_plan = make_plan("probability", seeds=[shared_seeds], threshold=1.0)
        probability_result = mc.derive(
            probability_plan,
            sample([("probability", list(zip(shared_seeds, probability_values)))]),
            None,
        )
        expected_probability = wilson(probability_values)
        for field, expected in zip(
            ("estimate", "intervalLower", "intervalUpper", "halfWidth"),
            expected_probability,
        ):
            assert_close(
                probability_result["precision"][field],
                expected,
                label=f"Wilson {field} pattern {pattern}",
            )
        maximum_error = max(maximum_error, abs(probability_result["precision"]["halfWidth"] - expected_probability[3]))
        comparisons += 1

    # Fresh fail-closed seed-layout adversaries.
    overlapping_plan = {
        **make_plan(
            "difference_in_means",
            group_seeds=[[shared_seeds], [right_seeds]],
        )
    }
    overlapping_plan["design"] = {
        "mode": "fixed",
        "groupSeedBatches": [[shared_seeds], [shared_seeds]],
    }
    overlapping_plan["planIdentity"] = mc.plan_identity(overlapping_plan)
    assert_raises("must be disjoint", lambda: mc.validate_plan(overlapping_plan))
    comparisons += 1

    paired_plan = make_plan(
        "paired_mean_difference",
        seeds=[shared_seeds],
        pairing="paired_by_seed",
    )
    left_rows = list(zip(shared_seeds, [float(i) for i in range(n)]))
    reversed_rows = list(zip(reversed(shared_seeds), [float(i) for i in range(n)]))
    assert_raises(
        "same exact declared seed identities and order",
        lambda: mc.derive(paired_plan, sample([("left", left_rows), ("right", reversed_rows)]), None),
    )
    comparisons += 1

    sequential_plan = make_plan(
        "mean",
        seeds=[[1, 2, 3, 4], [5, 6, 7, 8]],
        mode="sequential",
    )
    partial = sample([("mean", [(1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0), (5, 5.0)])])
    assert_raises(
        "predeclared cumulative batch boundary",
        lambda: mc.derive(sequential_plan, partial, None),
    )
    comparisons += 1

    # Exact quantile rank coverage at central/tail probabilities, including a deliberately
    # unsupported p=.99 sample that must fail closed.
    quantile_cases = [
        (0.50, 20, True),
        (0.90, 40, True),
        (0.95, 64, True),
        (0.99, 128, False),
        (0.99, 299, True),
    ]
    for probability, count, feasible in quantile_cases:
        seeds = list(range(20_000, 20_000 + count))
        values = [float((index * 37 + 11) % 503) for index in range(count)]
        plan = make_plan("quantile", seeds=[seeds], threshold=1_000_000.0, quantile=probability)
        result = mc.derive(plan, sample([("quantile", list(zip(seeds, values)))]), None)
        precision = result["precision"]
        assert precision["coverageFeasible"] is feasible
        if feasible:
            lower_rank = precision["lowerOrderStatisticRank"]
            upper_rank = precision["upperOrderStatisticRank"]
            coverage = exact_binomial_interval_coverage(count, probability, lower_rank, upper_rank)
            assert_close(precision["achievedCoverage"], coverage, tolerance=2e-12, label=f"quantile coverage p={probability}")
            assert coverage + 1e-12 >= CONFIDENCE
            ordered = sorted(values)
            assert precision["intervalLower"] == ordered[lower_rank - 1]
            assert precision["intervalUpper"] == ordered[upper_rank - 1]
            assert result["decision"] == "sufficient_stop"
        else:
            assert precision["halfWidth"] is None
            assert precision["intervalLower"] is None
            assert precision["intervalUpper"] is None
            assert result["decision"] == "insufficient_quantile_coverage_no_predeclared_additional_batch"
        comparisons += 1

    print(f"audit_v4_area_h_estimator_matrix_comparisons={comparisons}")
    print(f"audit_v4_area_h_estimator_matrix_max_half_width_error={maximum_error:.18g}")
    assert comparisons == 32 * 4 + 3 + len(quantile_cases)


if __name__ == "__main__":
    main()
