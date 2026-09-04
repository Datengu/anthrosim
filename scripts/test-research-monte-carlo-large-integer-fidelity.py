#!/usr/bin/env python3
"""Permanent AV4-010 regressions for Monte Carlo numeric fidelity."""

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


def make_plan(kind: str, *, paired: bool = False, independent_groups: bool = False) -> dict:
    if independent_groups:
        design = {
            "mode": "fixed",
            "groupSeedBatches": [[[1, 2, 3, 4]], [[101, 102, 103, 104]]],
        }
    else:
        design = {"mode": "fixed", "seedBatches": [[1, 2, 3, 4]]}
    plan = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": f"av4-010-{kind}",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": kind,
            "confidenceLevel": 0.95,
            "maxHalfWidth": 0.1,
        },
        "design": design,
        "pairing": "paired_by_seed" if paired else "independent",
        "rationale": "Permanent AV4-010 exact-integer numeric-fidelity regression.",
    }
    plan["planIdentity"] = mc.plan_identity(plan)
    mc.validate_plan(plan)
    return plan


def sample(groups) -> dict:
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


def exact_half(values: list[int]) -> float:
    n = len(values)
    mean = Fraction(sum(values), n)
    variance = sum((Fraction(value) - mean) ** 2 for value in values) / (n - 1)
    return NormalDist().inv_cdf(0.975) * math.sqrt(float(variance / n))


def assert_raises(fragment: str, fn) -> None:
    try:
        fn()
    except ValueError as error:
        assert fragment in str(error), (fragment, str(error))
    else:
        raise AssertionError(f"expected ValueError containing {fragment!r}")


def mean_large_integer_regression() -> None:
    base = 1 << 53
    values = [base, base + 1, base, base + 1]
    result = mc.derive(
        make_plan("mean"),
        sample([("counter", list(zip([1, 2, 3, 4], values)))]),
        None,
    )
    expected = exact_half(values)
    assert abs(result["precision"]["halfWidth"] - expected) < 1e-12
    assert result["precision"]["sufficient"] is False
    assert result["decision"] == "insufficient_no_predeclared_additional_batch"
    assert result["precision"]["numericFidelity"]["momentArithmetic"] == "exact_rational"


def independent_difference_large_integer_regression() -> None:
    base = 1 << 53
    left = [base, base + 1, base, base + 1]
    right = [0, 0, 0, 0]
    result = mc.derive(
        make_plan("difference_in_means", independent_groups=True),
        sample([
            ("left", list(zip([1, 2, 3, 4], left))),
            ("right", list(zip([101, 102, 103, 104], right))),
        ]),
        None,
    )
    expected = exact_half(left)
    assert abs(result["precision"]["halfWidth"] - expected) < 1e-12
    assert result["precision"]["sufficient"] is False
    assert result["decision"] == "insufficient_no_predeclared_additional_batch"


def paired_difference_large_integer_regression() -> None:
    base = 1 << 53
    left = [base, base + 1, base, base + 1]
    right = [base, base, base, base]
    result = mc.derive(
        make_plan("paired_mean_difference", paired=True),
        sample([
            ("left", list(zip([1, 2, 3, 4], left))),
            ("right", list(zip([1, 2, 3, 4], right))),
        ]),
        None,
    )
    expected = exact_half([0, 1, 0, 1])
    assert abs(result["precision"]["halfWidth"] - expected) < 1e-12
    assert result["precision"]["sufficient"] is False
    assert result["decision"] == "insufficient_no_predeclared_additional_batch"


def numeric_input_contract_regression() -> None:
    base = 1 << 53
    large_float = sample([("float", [(1, float(base)), (2, float(base)), (3, float(base)), (4, float(base))])])
    assert_raises(
        "large floating replicate",
        lambda: mc.derive(make_plan("mean"), large_float, None),
    )

    # Ordinary finite binary64 inputs remain accepted and retain the legacy path.
    ordinary = mc.derive(
        make_plan("mean"),
        sample([("float", [(1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0)])]),
        None,
    )
    assert "numericFidelity" not in ordinary["precision"]


if __name__ == "__main__":
    mean_large_integer_regression()
    independent_difference_large_integer_regression()
    paired_difference_large_integer_regression()
    numeric_input_contract_regression()
    print("AV4-010 Monte Carlo numeric-fidelity regressions passed")
