#!/usr/bin/env python3
"""Audit-v5 Area-H matched small-n control for Bernoulli/Wilson precision."""

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
        "planId": "audit-v5-area-h-wilson-small-n-control",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": "probability",
            "confidenceLevel": 0.95,
            "maxHalfWidth": 0.1,
        },
        "design": {
            "mode": "sequential",
            "seedBatches": [[1, 2], list(range(3, 103))],
        },
        "pairing": "independent",
        "rationale": (
            "Matched Audit-v5 control for AV5-005: the probability/Wilson method should not "
            "collapse to a zero-width sufficient stop after only two identical Bernoulli outcomes."
        ),
    }
    value["planIdentity"] = mc.plan_identity(value)
    return value


def diagnose(values: list[bool]) -> dict:
    samples = {
        "schemaVersion": 1,
        "groups": [
            {
                "id": "focal",
                "replicates": [
                    {"seed": 1, "value": values[0]},
                    {"seed": 2, "value": values[1]},
                ],
            }
        ],
    }
    return mc.derive(plan(), samples, None)


def main() -> None:
    zero = diagnose([False, False])
    one = diagnose([True, True])

    for label, result in [("zero", zero), ("one", one)]:
        precision = result["precision"]
        print(
            f"{label}: decision={result['decision']}; n={result['replicateCount']}; "
            f"estimate={precision['estimate']}; interval=[{precision['intervalLower']},"
            f"{precision['intervalUpper']}]; half_width={precision['halfWidth']}; "
            f"method={precision['precisionMethod']}"
        )
        assert result["replicateCount"] == 2
        assert precision["precisionMethod"] == "wilson_score_probability"
        assert precision["halfWidth"] is not None
        assert precision["halfWidth"] > 0.1
        assert precision["sufficient"] is False
        assert result["decision"] == "insufficient_continue_with_declared_next_batch"
        assert result["nextDeclaredBatchSeeds"] == list(range(3, 103))


if __name__ == "__main__":
    main()
