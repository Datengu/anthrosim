#!/usr/bin/env python3
"""Regression coverage for AV3-015 household paired-effect derivation."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from typing import Any

SCRIPT = Path(__file__).with_name("research-general-demography-baseline.py")
spec = importlib.util.spec_from_file_location("research_general_demography_baseline", SCRIPT)
if spec is None or spec.loader is None:
    raise RuntimeError(f"could not load {SCRIPT}")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

FIXED = "fixed_founder_v1"
CURRENT = "deterministic_dependency_fission_v2"
DEMOGRAPHIES = (
    "negative_growth_control_v1",
    "replacement_control_v1",
    "positive_growth_control_v1",
)
SEEDS = list(range(3042001, 3042131))


def definition(treatment: str = CURRENT) -> dict[str, Any]:
    return {
        "schemaVersion": 1,
        "seeds": SEEDS,
        "dimensions": [
            {
                "id": "demography",
                "kind": "structural",
                "path": "/experiment/demography",
                "values": [{"scheduleId": schedule} for schedule in DEMOGRAPHIES],
            },
            {
                "id": "household_lifecycle",
                "kind": "structural",
                "path": "/experiment/householdLifecycle",
                "values": [
                    None,
                    {
                        "schemaVersion": 2,
                        "modelId": treatment,
                        "provenance": "synthetic_validation",
                        "maxLivingMembers": 8,
                        "minimumIndependentAgeYears": 18,
                    },
                ],
            },
            {
                "id": "founder_age_ceiling_years",
                "kind": "numeric",
                "path": "/experiment/population/syntheticMaxAgeYears",
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


def run(demography: str, seed: int, lifecycle: str, offset: float) -> dict[str, Any]:
    return {
        "runId": f"{demography}-{seed}-{lifecycle}",
        "seed": seed,
        "demography": demography,
        "householdLifecycle": lifecycle,
        "founderAgeCeilingYears": 60,
        "resourceProductivityScalePermille": 1000,
        "terminalPopulation": 100.0 + offset + (seed % 7),
        "lateGrowthRatePerYear": -0.01 + offset / 10000.0,
        "mateLimitationFraction": 0.2 + offset / 1000.0,
    }


def full_design(treatment: str = CURRENT) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for demography in DEMOGRAPHIES:
        for seed in SEEDS:
            rows.append(run(demography, seed, FIXED, 0.0))
            rows.append(run(demography, seed, treatment, -10.0))
    return rows


def require_value_error(callable_obj: Any, expected_fragment: str) -> None:
    try:
        callable_obj()
    except ValueError as exc:
        assert expected_fragment in str(exc), (expected_fragment, str(exc))
    else:
        raise AssertionError(f"expected ValueError containing {expected_fragment!r}")


def main() -> int:
    rows = full_design()
    effects = module.paired_household_effects(rows, definition())
    assert len(rows) == 780
    assert len(effects) == 3
    assert [effect["demography"] for effect in effects] == sorted(DEMOGRAPHIES)
    assert all(effect["pairedReplicates"] == 130 for effect in effects)
    assert sum(effect["pairedReplicates"] for effect in effects) == 390
    assert all(effect["fixedHouseholdLifecycle"] == FIXED for effect in effects)
    assert all(effect["fissionHouseholdLifecycle"] == CURRENT for effect in effects)
    assert all(effect["fissionMinusFixedTerminalPopulation"]["mean"] == -10.0 for effect in effects)

    future = "deterministic_dependency_fission_v3_test"
    future_effects = module.paired_household_effects(full_design(future), definition(future))
    assert len(future_effects) == 3
    assert all(effect["fissionHouseholdLifecycle"] == future for effect in future_effects)

    missing_arm = rows[:-1]
    require_value_error(
        lambda: module.paired_household_effects(missing_arm, definition()),
        "missing declared household lifecycle",
    )

    missing_whole_pair = rows[:-2]
    require_value_error(
        lambda: module.paired_household_effects(missing_whole_pair, definition()),
        "missing declared pairing cell",
    )

    unexpected = list(rows)
    unexpected[-1] = dict(unexpected[-1], householdLifecycle="unknown_lifecycle_v99")
    require_value_error(
        lambda: module.paired_household_effects(unexpected, definition()),
        "unexpected household lifecycle",
    )

    duplicate = list(rows)
    duplicate.append(dict(rows[0], runId="duplicate-fixed-run"))
    require_value_error(
        lambda: module.paired_household_effects(duplicate, definition()),
        "duplicate household lifecycle run",
    )

    stale_definition = definition("deterministic_dependency_fission_v3_test")
    require_value_error(
        lambda: module.paired_household_effects(rows, stale_definition),
        "unexpected household lifecycle",
    )

    malformed = definition()
    malformed["dimensions"][1]["values"] = [None]
    require_value_error(
        lambda: module.paired_household_effects(rows, malformed),
        "exactly two values",
    )

    print("AV3-015 paired household-effect regression: ok (3 groups, 130 pairs each, 390 total)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
