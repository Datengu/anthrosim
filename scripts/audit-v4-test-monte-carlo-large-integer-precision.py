#!/usr/bin/env python3
"""Fresh Audit-v4 Area-H adversary for large exact-integer Monte Carlo samples."""

from __future__ import annotations

from fractions import Fraction
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


def plan() -> dict:
    value = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": "audit-v4-large-integer-mean",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": "mean",
            "confidenceLevel": 0.95,
            "maxHalfWidth": 0.1,
        },
        "design": {"mode": "fixed", "seedBatches": [[1, 2, 3, 4]]},
        "pairing": "independent",
        "rationale": "Fresh Audit-v4 exact-integer numeric-fidelity adversary.",
    }
    value["planIdentity"] = mc.plan_identity(value)
    mc.validate_plan(value)
    return value


def sample(values: list[int]) -> dict:
    return {
        "schemaVersion": 1,
        "groups": [
            {
                "id": "large-exact-counter",
                "replicates": [
                    {"seed": seed, "value": value}
                    for seed, value in zip([1, 2, 3, 4], values)
                ],
            }
        ],
    }


def exact_normal_half_width(values: list[int]) -> float:
    n = len(values)
    mean = Fraction(sum(values), n)
    variance = sum((Fraction(value) - mean) ** 2 for value in values) / (n - 1)
    z = NormalDist().inv_cdf(0.975)
    return z * math.sqrt(float(variance / n))


def main() -> None:
    base = 1 << 53
    exact_values = [base, base + 1, base, base + 1]
    result = mc.derive(plan(), sample(exact_values), None)
    reported = result["precision"]
    exact_half = exact_normal_half_width(exact_values)

    converted = [float(value) for value in exact_values]
    print(f"exact_values={exact_values}")
    print(f"float_values={converted}")
    print(f"distinct_exact_values={len(set(exact_values))}")
    print(f"distinct_float_values={len(set(converted))}")
    print(f"reported_half_width={reported['halfWidth']:.12f}")
    print(f"exact_integer_half_width={exact_half:.12f}")
    print(f"declared_threshold={result['estimand']['maxHalfWidth']:.12f}")
    print(f"decision={result['decision']}")

    assert len(set(exact_values)) == 2
    assert len(set(converted)) == 1
    assert exact_half > result["estimand"]["maxHalfWidth"]

    # Scientific invariant under attack: accepting exact integer Monte Carlo values must not erase
    # their replicate variation and certify precision that the same declared normal-CLT estimator
    # rejects when evaluated on those exact values.
    assert reported["halfWidth"] >= exact_half - 1e-12, (
        "Monte Carlo gate lost exact-integer replicate variation before variance estimation: "
        f"reported={reported['halfWidth']} exact={exact_half} decision={result['decision']}"
    )
    assert result["decision"] != "sufficient_stop", (
        "Monte Carlo gate falsely certified sufficient precision after exact integers collapsed "
        "during float conversion"
    )


if __name__ == "__main__":
    main()
