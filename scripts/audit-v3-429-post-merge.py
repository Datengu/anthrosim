#!/usr/bin/env python3
"""Independent post-merge adversary for Audit v3 finding AV3-015 / issue #429.

This deliberately does not import the production general-demography summarizer. It derives
expected paired-cell cardinality from the frozen research definition and checks a generated
summary/result against that contract.
"""

from __future__ import annotations

import argparse
import copy
import json
from itertools import product
from pathlib import Path
from typing import Any

FIXED_LIFECYCLE = "fixed_founder_v1"
LIFECYCLE_DIMENSION = "household_lifecycle"
LIFECYCLE_PATH = "/experiment/householdLifecycle"


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected object in {path}")
    return value


def dimension(definition: dict[str, Any], dimension_id: str) -> dict[str, Any]:
    matches = [d for d in definition.get("dimensions", []) if d.get("id") == dimension_id]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one {dimension_id!r} dimension, found {len(matches)}")
    return matches[0]


def lifecycle_contrast(definition: dict[str, Any]) -> tuple[str, str]:
    lifecycle = dimension(definition, LIFECYCLE_DIMENSION)
    if lifecycle.get("kind") != "structural" or lifecycle.get("path") != LIFECYCLE_PATH:
        raise ValueError("household_lifecycle is not the declared structural lifecycle contrast")
    values = lifecycle.get("values")
    if not isinstance(values, list) or len(values) != 2:
        raise ValueError("household_lifecycle must declare exactly two structural values")

    ids: list[str] = []
    for value in values:
        if value is None:
            ids.append(FIXED_LIFECYCLE)
            continue
        if not isinstance(value, dict) or not isinstance(value.get("modelId"), str) or not value["modelId"]:
            raise ValueError("non-null household lifecycle value requires a modelId")
        ids.append(value["modelId"])

    if len(set(ids)) != 2 or FIXED_LIFECYCLE not in ids:
        raise ValueError(f"malformed fixed-vs-treatment lifecycle contrast: {ids}")
    treatment = [model_id for model_id in ids if model_id != FIXED_LIFECYCLE]
    if len(treatment) != 1:
        raise ValueError(f"expected exactly one treatment lifecycle, got {ids}")
    return FIXED_LIFECYCLE, treatment[0]


def declared_design(definition: dict[str, Any]) -> tuple[set[tuple[str, int, int]], int, int, str, str]:
    fixed, treatment = lifecycle_contrast(definition)

    demographies = []
    for value in dimension(definition, "demography").get("values", []):
        if not isinstance(value, dict) or not isinstance(value.get("scheduleId"), str):
            raise ValueError("demography values require scheduleId")
        demographies.append(value["scheduleId"])
    founders = [int(value) for value in dimension(definition, "founder_age_ceiling_years").get("values", [])]
    resources = [int(value) for value in dimension(definition, "resource_productivity_scale_permille").get("values", [])]
    seeds = [int(value) for value in definition.get("seeds", [])]

    for label, values in (
        ("demographies", demographies),
        ("founder age ceilings", founders),
        ("resource scales", resources),
        ("seeds", seeds),
    ):
        if not values:
            raise ValueError(f"declared design has no {label}")
        if len(values) != len(set(values)):
            raise ValueError(f"declared design has duplicate {label}")

    group_keys = set(product(demographies, founders, resources))
    pairs_per_group = len(seeds)
    expected_runs = len(group_keys) * pairs_per_group * 2
    return group_keys, pairs_per_group, expected_runs, fixed, treatment


def verify(definition: dict[str, Any], summary: dict[str, Any]) -> tuple[int, int]:
    expected_groups, pairs_per_group, expected_runs, fixed, treatment = declared_design(definition)

    if int(summary.get("runCount", -1)) != expected_runs:
        raise ValueError(f"runCount mismatch: expected {expected_runs}, got {summary.get('runCount')}")

    paired = summary.get("pairedHouseholdEffects")
    if not isinstance(paired, list):
        raise ValueError("pairedHouseholdEffects is not a list")
    if len(paired) != len(expected_groups):
        raise ValueError(
            f"paired group count mismatch: expected {len(expected_groups)}, got {len(paired)}"
        )

    observed: set[tuple[str, int, int]] = set()
    represented_pairs = 0
    for effect in paired:
        if not isinstance(effect, dict):
            raise ValueError("paired effect entry is not an object")
        key = (
            str(effect.get("demography")),
            int(effect.get("founderAgeCeilingYears")),
            int(effect.get("resourceProductivityScalePermille")),
        )
        if key in observed:
            raise ValueError(f"duplicate paired-effect group {key}")
        observed.add(key)
        if effect.get("fixedHouseholdLifecycle") != fixed:
            raise ValueError(
                f"fixed lifecycle mismatch for {key}: expected {fixed}, got {effect.get('fixedHouseholdLifecycle')}"
            )
        if effect.get("fissionHouseholdLifecycle") != treatment:
            raise ValueError(
                f"treatment lifecycle mismatch for {key}: expected {treatment}, got {effect.get('fissionHouseholdLifecycle')}"
            )
        replicates = int(effect.get("pairedReplicates", -1))
        if replicates != pairs_per_group:
            raise ValueError(
                f"paired replicate mismatch for {key}: expected {pairs_per_group}, got {replicates}"
            )
        represented_pairs += replicates

    if observed != expected_groups:
        raise ValueError(
            f"paired group identity mismatch: missing={sorted(expected_groups - observed)}, "
            f"unexpected={sorted(observed - expected_groups)}"
        )

    expected_pairs = len(expected_groups) * pairs_per_group
    if represented_pairs != expected_pairs:
        raise ValueError(f"represented pair mismatch: expected {expected_pairs}, got {represented_pairs}")
    return len(expected_groups), represented_pairs


def expect_rejected(label: str, definition: dict[str, Any], summary: dict[str, Any]) -> None:
    try:
        verify(definition, summary)
    except (KeyError, TypeError, ValueError) as exc:
        print(f"{label}: rejected ({exc})")
        return
    raise AssertionError(f"{label}: adversary unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("definition", type=Path)
    parser.add_argument("summary", type=Path)
    args = parser.parse_args()

    definition = load(args.definition)
    summary = load(args.summary)
    groups, represented_pairs = verify(definition, summary)
    pairs_per_group = represented_pairs // groups
    print(
        f"AV3-015 post-merge adversary: ok ({groups} groups, {pairs_per_group} pairs each, "
        f"{represented_pairs}/{represented_pairs} represented)"
    )

    original_defect = copy.deepcopy(summary)
    original_defect["pairedHouseholdEffects"] = []
    expect_rejected("original empty-summary adversary", definition, original_defect)

    missing_pair = copy.deepcopy(summary)
    missing_pair["pairedHouseholdEffects"][0]["pairedReplicates"] -= 1
    expect_rejected("missing same-seed pair adversary", definition, missing_pair)

    future_definition = copy.deepcopy(definition)
    lifecycle_values = dimension(future_definition, LIFECYCLE_DIMENSION)["values"]
    treatment_values = [value for value in lifecycle_values if value is not None]
    assert len(treatment_values) == 1
    treatment_values[0]["modelId"] = "future_declared_household_treatment_v999"
    expect_rejected("declared-treatment drift adversary", future_definition, summary)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
