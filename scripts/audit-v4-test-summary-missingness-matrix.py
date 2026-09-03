#!/usr/bin/env python3
"""Fresh Audit-v4 Area L paired-summary missingness/order adversary."""

from __future__ import annotations

import importlib.util
import math
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "research-general-demography-baseline.py"
spec = importlib.util.spec_from_file_location("general_demography", MODULE_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError("cannot load general-demography summarizer")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def definition() -> dict:
    return {
        "seeds": [1, 2, 3, 4, 5, 6],
        "dimensions": [
            {
                "id": "demography",
                "kind": "structural",
                "path": "/experiment/demography",
                "values": [{"scheduleId": "audit-v4-l-demography"}],
            },
            {
                "id": "household_lifecycle",
                "kind": "structural",
                "path": "/experiment/householdLifecycle",
                "values": [None, {"modelId": "audit-v4-l-fission"}],
            },
            {
                "id": "founder_age_ceiling_years",
                "kind": "numeric",
                "path": "/experiment/population/founderAgeCeilingYears",
                "values": [60],
            },
            {
                "id": "resource_productivity_scale_permille",
                "kind": "numeric",
                "path": "/experiment/resources/productivityScalePermille",
                "values": [1000],
            },
        ],
    }


def runs() -> list[dict]:
    fixed_late = [0.0, 0.1, None, 0.3, None, 0.5]
    fission_late = [0.2, None, 0.4, 0.6, None, 0.9]
    fixed_mate = [0.1, None, 0.3, 0.4, 0.5, 0.6]
    fission_mate = [0.2, 0.25, None, 0.7, 0.9, 1.0]
    rows: list[dict] = []
    for index, seed in enumerate(range(1, 7)):
        rows.append(
            {
                "runId": f"fixed-{seed}",
                "seed": seed,
                "demography": "audit-v4-l-demography",
                "householdLifecycle": "fixed_founder_v1",
                "founderAgeCeilingYears": 60,
                "resourceProductivityScalePermille": 1000,
                "terminalPopulation": 10.0,
                "lateGrowthRatePerYear": fixed_late[index],
                "mateLimitationFraction": fixed_mate[index],
            }
        )
        rows.append(
            {
                "runId": f"fission-{seed}",
                "seed": seed,
                "demography": "audit-v4-l-demography",
                "householdLifecycle": "audit-v4-l-fission",
                "founderAgeCeilingYears": 60,
                "resourceProductivityScalePermille": 1000,
                "terminalPopulation": 11.0 + index,
                "lateGrowthRatePerYear": fission_late[index],
                "mateLimitationFraction": fission_mate[index],
            }
        )
    return rows


def assert_close(actual: float | None, expected: float) -> None:
    assert actual is not None
    assert math.isclose(actual, expected, rel_tol=0.0, abs_tol=1e-12), (actual, expected)


def check_summary(rows: list[dict]) -> None:
    result = module.paired_household_effects(rows, definition())
    assert len(result) == 1
    item = result[0]
    assert item["pairedReplicates"] == 6

    terminal = item["fissionMinusFixedTerminalPopulation"]
    assert terminal["n"] == 6
    assert_close(terminal["mean"], 3.5)

    late = item["fissionMinusFixedLateGrowthRatePerYear"]
    assert late["n"] == 3
    assert_close(late["mean"], 0.3)

    mate = item["fissionMinusFixedMateLimitationFraction"]
    assert mate["n"] == 4
    assert_close(mate["mean"], 0.3)


def require_failure(rows: list[dict], text: str) -> None:
    try:
        module.paired_household_effects(rows, definition())
    except ValueError as error:
        if text not in str(error):
            raise AssertionError(f"wrong fail-closed reason: {error}") from error
    else:
        raise AssertionError(f"expected fail-closed error containing {text!r}")


def main() -> None:
    baseline = runs()
    check_summary(baseline)

    permutations = 0
    for offset in range(len(baseline)):
        rotated = baseline[offset:] + baseline[:offset]
        check_summary(rotated)
        check_summary(list(reversed(rotated)))
        permutations += 2
    rng = random.Random(304_120_026)
    for _ in range(40):
        shuffled = list(baseline)
        rng.shuffle(shuffled)
        check_summary(shuffled)
        permutations += 1

    missing = [row for row in baseline if row["runId"] != "fission-3"]
    require_failure(missing, "missing declared household lifecycle")

    duplicate = list(baseline) + [dict(baseline[0], runId="fixed-1-duplicate")]
    require_failure(duplicate, "duplicate household lifecycle run")

    unexpected = list(baseline) + [dict(baseline[0], runId="fixed-7", seed=7)]
    require_failure(unexpected, "unexpected household pairing cell")

    print(f"audit_v4_area_l_summary_orderings={permutations}")
    print("audit_v4_area_l_declared_pairs=6")
    print("audit_v4_area_l_terminal_contributing_pairs=6")
    print("audit_v4_area_l_late_growth_contributing_pairs=3")
    print("audit_v4_area_l_mate_limitation_contributing_pairs=4")
    print("audit_v4_area_l_fail_closed_attacks=3")
    print("audit_v4_area_l_summary_missingness_status=pass")


if __name__ == "__main__":
    main()
