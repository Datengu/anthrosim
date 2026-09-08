#!/usr/bin/env python3
"""Fresh Audit-v5 Area-H adversary for sequential small-n mean precision coverage."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
SPEC = importlib.util.spec_from_file_location("mc_sufficiency", SCRIPT)
assert SPEC and SPEC.loader
mc = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(mc)


def plan() -> dict:
    value = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": "audit-v5-area-h-small-n-mean-coverage",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": "mean",
            "confidenceLevel": 0.95,
            "maxHalfWidth": 0.1,
        },
        "design": {
            "mode": "sequential",
            "seedBatches": [
                [1, 2],
                list(range(3, 103)),
            ],
        },
        "pairing": "independent",
        "rationale": (
            "Audit-v5 adversary: a nominal 95% sequential mean-precision gate must not "
            "certify a two-observation zero-width interval when the declared supported method "
            "has catastrophically insufficient finite-sample coverage."
        ),
    }
    value["planIdentity"] = mc.plan_identity(value)
    return value


def main() -> None:
    precision_plan = plan()
    samples = {
        "schemaVersion": 1,
        "groups": [
            {
                "id": "focal",
                "replicates": [
                    {"seed": 1, "value": 0},
                    {"seed": 2, "value": 0},
                ],
            }
        ],
    }

    result = mc.derive(precision_plan, samples, None)
    precision = result["precision"]

    # Independent exact coverage construction for the stopping procedure:
    # X=0 with probability 0.9 and X=10 with probability 0.1, so E[X]=1.
    # The first two observations are both zero with probability 0.9^2 = 0.81.
    # On that event the current gate reports [0,0] and stops, which cannot cover E[X]=1.
    # Even granting perfect coverage on every other path, overall coverage is <= 0.19.
    zero_probability = 0.9
    first_boundary_false_stop_probability = zero_probability * zero_probability
    maximum_possible_overall_coverage = 1.0 - first_boundary_false_stop_probability
    true_mean = 1.0

    print(
        "decision={}; n={}; estimate={}; interval=[{},{}]; half_width={}; "
        "false_stop_probability={:.6f}; max_overall_coverage={:.6f}; nominal_confidence={:.6f}".format(
            result["decision"],
            result["replicateCount"],
            precision["estimate"],
            precision["intervalLower"],
            precision["intervalUpper"],
            precision["halfWidth"],
            first_boundary_false_stop_probability,
            maximum_possible_overall_coverage,
            precision["confidenceLevel"],
        )
    )

    assert result["replicateCount"] == 2
    assert precision["precisionMethod"] == "normal_clt_mean_se"
    assert precision["estimate"] == 0.0
    assert precision["intervalLower"] == 0.0
    assert precision["intervalUpper"] == 0.0
    assert precision["halfWidth"] == 0.0
    assert true_mean < precision["intervalLower"] or true_mean > precision["intervalUpper"]
    assert maximum_possible_overall_coverage < precision["confidenceLevel"]

    # Scientific oracle: a gate advertising the requested confidence level must not certify
    # stopping at a boundary that makes that coverage mathematically impossible for a simple
    # supported finite-variance process.
    assert result["decision"] != "sufficient_stop", (
        "nominal 95% sequential mean gate stopped on n=2 zero variance even though a simple "
        "90% zero / 10% ten process makes this false stop occur with probability 0.81, bounding "
        "overall coverage at 0.19"
    )


if __name__ == "__main__":
    main()
