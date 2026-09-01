#!/usr/bin/env python3
"""Audit-v3 Area H adversary for independent difference-in-means seed semantics."""

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

seeds = list(range(1, 21))
threshold = 4.5
plan = {
    "schemaVersion": 1,
    "planIdentity": "",
    "planId": "audit-v3-independent-difference-pairing",
    "uncertaintyCategory": "process_stochastic_monte_carlo",
    "estimand": {
        "kind": "difference_in_means",
        "confidenceLevel": 0.95,
        "maxHalfWidth": threshold,
    },
    "design": {"mode": "fixed", "seedBatches": [seeds]},
    "pairing": "independent",
    "rationale": "Audit-v3 falsification of independent-sample uncertainty under the exact seed layout required by the gate.",
}
plan["planIdentity"] = mc.plan_identity(plan)
mc.validate_plan(plan)

# The two groups use the exact same declared seed identities because validate_samples
# requires that for every multi-group estimand, including difference_in_means.
# Values are perfectly anti-correlated by seed. This makes the covariance term
# scientifically material and gives a hand-computable adversary.
left_values = [float(index) - 9.5 for index in range(20)]
right_values = [-value for value in left_values]
sample = {
    "schemaVersion": 1,
    "groups": [
        {
            "id": "left",
            "replicates": [
                {"seed": seed, "value": value}
                for seed, value in zip(seeds, left_values)
            ],
        },
        {
            "id": "right",
            "replicates": [
                {"seed": seed, "value": value}
                for seed, value in zip(seeds, right_values)
            ],
        },
    ],
}

result = mc.derive(plan, sample, None)
reported = result["precision"]["halfWidth"]
assert result["precision"]["precisionMethod"] == "normal_clt_independent_difference_in_means"

# Because the gate itself forces the same seed identity in both arms, the actual
# estimator produced by this accepted design is a same-seed contrast. Its sampling
# variance is Var(L-R)/n, not Var(L)/n + Var(R)/n unless covariance is zero.
differences = [left - right for left, right in zip(left_values, right_values)]
mean_difference = sum(differences) / len(differences)
variance_difference = sum(
    (value - mean_difference) ** 2 for value in differences
) / (len(differences) - 1)
z = NormalDist().inv_cdf(0.975)
covariance_aware_half_width = z * math.sqrt(variance_difference / len(differences))

print(f"reported independent half-width: {reported:.12f}")
print(f"same-seed covariance-aware half-width: {covariance_aware_half_width:.12f}")
print(f"predeclared threshold: {threshold:.12f}")
print(f"gate decision: {result['decision']}")

assert reported < threshold
assert covariance_aware_half_width > threshold
assert result["precision"]["sufficient"] is False, (
    "difference_in_means is declared independent but the gate requires identical seed "
    "identities in both arms and ignored their non-zero covariance, producing a false "
    "precision pass"
)
