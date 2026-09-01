#!/usr/bin/env python3
"""Independent post-merge adversary for audit-v3 finding AV3-006 / issue #410."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
SPEC = importlib.util.spec_from_file_location("mc_sufficiency_reverify", SCRIPT)
assert SPEC and SPEC.loader
mc = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mc)


def make_plan(kind, design, threshold, pairing):
    plan = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": f"av3-006-reverify-{kind}",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": kind,
            "confidenceLevel": 0.95,
            "maxHalfWidth": threshold,
        },
        "design": design,
        "pairing": pairing,
        "rationale": "Independent post-merge AV3-006 adversarial reverification.",
    }
    plan["planIdentity"] = mc.plan_identity(plan)
    return plan


def sample(left_seeds, left_values, right_seeds, right_values):
    return {
        "schemaVersion": 1,
        "groups": [
            {
                "id": "left",
                "replicates": [
                    {"seed": seed, "value": value}
                    for seed, value in zip(left_seeds, left_values, strict=True)
                ],
            },
            {
                "id": "right",
                "replicates": [
                    {"seed": seed, "value": value}
                    for seed, value in zip(right_seeds, right_values, strict=True)
                ],
            },
        ],
    }


def must_reject(plan, fragment):
    try:
        mc.validate_plan(plan)
    except ValueError as error:
        assert fragment in str(error), (fragment, str(error))
        return
    raise AssertionError(f"expected rejection containing {fragment!r}")


def main():
    seeds = list(range(1, 21))
    values = [float(index) - 9.5 for index in range(20)]
    negatives = [-value for value in values]

    # The exact frozen AV3-006 design shape: same seeds in an estimator declaring independence.
    # It must now fail before an independent two-sample interval can be produced.
    old_contradictory = make_plan(
        "difference_in_means",
        {"mode": "fixed", "seedBatches": [seeds]},
        4.5,
        "independent",
    )
    must_reject(old_contradictory, "groupSeedBatches")

    # Genuine independent arms use disjoint, separately predeclared seed identities.
    right_seeds = list(range(101, 121))
    independent = make_plan(
        "difference_in_means",
        {"mode": "fixed", "groupSeedBatches": [[seeds], [right_seeds]]},
        4.5,
        "independent",
    )
    mc.validate_plan(independent)
    independent_result = mc.derive(
        independent,
        sample(seeds, values, right_seeds, negatives),
        None,
    )
    independent_half = independent_result["precision"]["halfWidth"]
    assert abs(independent_half - 3.666756860283) < 1e-12, independent_half
    assert independent_result["precision"]["sufficient"] is True
    assert independent_result["pairingSemantics"] == "independent"
    assert independent_result["groupSeedIdentities"]["left"] == seeds
    assert independent_result["groupSeedIdentities"]["right"] == right_seeds

    # Any overlap invalidates the independent-arm contract and must fail closed.
    overlap = make_plan(
        "difference_in_means",
        {"mode": "fixed", "groupSeedBatches": [[seeds], [list(range(20, 40))]]},
        4.5,
        "independent",
    )
    must_reject(overlap, "disjoint")

    # The same exact 20-seed anti-correlated values are scientifically a paired contrast.
    # Their covariance-aware half-width is the audit reference 5.185577281736 > 4.5.
    paired = make_plan(
        "paired_mean_difference",
        {"mode": "fixed", "seedBatches": [seeds]},
        4.5,
        "paired_by_seed",
    )
    mc.validate_plan(paired)
    paired_result = mc.derive(paired, sample(seeds, values, seeds, negatives), None)
    paired_half = paired_result["precision"]["halfWidth"]
    assert abs(paired_half - 5.185577281736) < 1e-12, paired_half
    assert paired_result["precision"]["sufficient"] is False

    # Positive covariance control: identical same-seed arms have zero difference variance.
    positive = mc.derive(paired, sample(seeds, values, seeds, values), None)
    positive_half = positive["precision"]["halfWidth"]
    assert positive_half == 0.0, positive_half
    assert positive["precision"]["sufficient"] is True

    print(f"independent disjoint half-width: {independent_half:.12f}")
    print(f"same-seed paired half-width:      {paired_half:.12f}")
    print(f"positive-covariance half-width:   {positive_half:.12f}")
    print("AV3-006 post-merge adversarial reverification passed")


if __name__ == "__main__":
    main()
