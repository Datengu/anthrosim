#!/usr/bin/env python3
"""Keep the living TRACE demographic-baseline narrative synchronized with machine evidence."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFINITION = ROOT / "research" / "general-demography-baseline-v1" / "confirmatory-definition.json"
RESULT = ROOT / "research" / "general-demography-baseline-v1" / "confirmatory-result.json"
CURRENT_DOC = ROOT / "docs" / "research" / "general-scientific-demographic-baseline-v1.md"
HISTORICAL_DOC = ROOT / "docs" / "research" / "general-scientific-demographic-baseline-v1-historical.md"
TRACE = ROOT / "docs" / "research" / "trace.md"

FIXED_LIFECYCLE = "fixed_founder_v1"
STALE_LIFECYCLE = "deterministic_size_fission_v1"
LIFECYCLE_DIMENSION = "household_lifecycle"


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object in {path.relative_to(ROOT)}")
    return value


def dimension(definition: dict[str, Any], dimension_id: str) -> dict[str, Any]:
    matches = [item for item in definition.get("dimensions", []) if item.get("id") == dimension_id]
    if len(matches) != 1:
        raise AssertionError(f"expected exactly one {dimension_id!r} dimension, found {len(matches)}")
    return matches[0]


def lifecycle_treatment(definition: dict[str, Any]) -> str:
    lifecycle = dimension(definition, LIFECYCLE_DIMENSION)
    values = lifecycle.get("values")
    if not isinstance(values, list) or len(values) != 2 or None not in values:
        raise AssertionError("confirmatory household_lifecycle dimension must be a two-arm null-vs-treatment contrast")
    treatments = [value for value in values if value is not None]
    if len(treatments) != 1 or not isinstance(treatments[0], dict):
        raise AssertionError("confirmatory household_lifecycle dimension has no unique treatment object")
    model_id = treatments[0].get("modelId")
    if not isinstance(model_id, str) or not model_id:
        raise AssertionError("confirmatory household lifecycle treatment has no modelId")
    return model_id


def schedule_ids(definition: dict[str, Any]) -> list[str]:
    values = dimension(definition, "demography").get("values", [])
    schedules: list[str] = []
    for value in values:
        if not isinstance(value, dict) or not isinstance(value.get("scheduleId"), str):
            raise AssertionError("confirmatory demography dimension contains a value without scheduleId")
        schedules.append(value["scheduleId"])
    return schedules


def signed_percent(value: float) -> str:
    return f"{value * 100:+.3f}"


def arm_row(arm: dict[str, Any]) -> str:
    growth = arm["lateGrowthRatePerYear"]
    return (
        f"| `{arm['demography']}` | `{arm['householdLifecycle']}` | "
        f"{signed_percent(float(growth['mean']))} "
        f"[{signed_percent(float(growth['ci95Lower']))}, {signed_percent(float(growth['ci95Upper']))}] | "
        f"{float(arm['terminalPopulation']['mean']):.1f} | "
        f"{float(arm['extinction']['estimate']) * 100:.1f}% | "
        f"{float(arm['mateLimitationFraction']['mean']) * 100:.1f}% |"
    )


def paired_line(effect: dict[str, Any]) -> str:
    return (
        f"- `{effect['demography']}`: fission-minus-fixed mean N240 = "
        f"**{float(effect['fissionMinusFixedTerminalPopulation']['mean']):+.1f} people** "
        f"across {int(effect['pairedReplicates'])} same-seed pairs."
    )


def main() -> None:
    definition = load(DEFINITION)
    result = load(RESULT)
    current = CURRENT_DOC.read_text(encoding="utf-8")
    historical = HISTORICAL_DOC.read_text(encoding="utf-8")
    trace = TRACE.read_text(encoding="utf-8")

    treatment = lifecycle_treatment(definition)
    schedules = schedule_ids(definition)
    seeds = [int(seed) for seed in definition.get("seeds", [])]
    founders = dimension(definition, "founder_age_ceiling_years").get("values", [])
    resources = dimension(definition, "resource_productivity_scale_permille").get("values", [])
    lifecycle_values = dimension(definition, LIFECYCLE_DIMENSION).get("values", [])

    if not seeds or len(seeds) != len(set(seeds)):
        raise AssertionError("confirmatory seed list is empty or contains duplicates")
    expected_runs = len(schedules) * len(founders) * len(resources) * len(lifecycle_values) * len(seeds)
    if int(result.get("runCount", -1)) != expected_runs:
        raise AssertionError(
            f"confirmatory result runCount drift: definition implies {expected_runs}, result reports {result.get('runCount')}"
        )

    if "**Status: living/current TRACE-linked result.**" not in current:
        raise AssertionError("living demographic-baseline page is not explicitly marked current")
    if treatment not in current:
        raise AssertionError(f"living demographic-baseline page does not name current treatment {treatment}")
    if f"fresh process seeds per arm: **{len(seeds)}**" not in current:
        raise AssertionError("living demographic-baseline page does not report the definition's current seed count")
    if f"= {expected_runs} completed runs**" not in current:
        raise AssertionError("living demographic-baseline page does not report the definition's current run count")

    research_id = result.get("researchId")
    if not isinstance(research_id, str) or not research_id:
        raise AssertionError("confirmatory result has no researchId")
    if f"`{research_id}`" not in current:
        raise AssertionError("living demographic-baseline page does not report the current researchId")

    arms = result.get("arms")
    if not isinstance(arms, list):
        raise AssertionError("confirmatory result arms is not a list")
    expected_arm_keys = {
        (schedule, lifecycle)
        for schedule in schedules
        for lifecycle in (FIXED_LIFECYCLE, treatment)
    }
    observed_arm_keys: set[tuple[str, str]] = set()
    for arm in arms:
        if not isinstance(arm, dict):
            raise AssertionError("confirmatory arm is not an object")
        key = (str(arm.get("demography")), str(arm.get("householdLifecycle")))
        if key in observed_arm_keys:
            raise AssertionError(f"duplicate confirmatory arm {key}")
        observed_arm_keys.add(key)
        row = arm_row(arm)
        if row not in current:
            raise AssertionError(f"living demographic-baseline table is stale for arm {key}; expected row: {row}")
    if observed_arm_keys != expected_arm_keys:
        raise AssertionError(
            f"confirmatory arm identity mismatch: expected {sorted(expected_arm_keys)}, got {sorted(observed_arm_keys)}"
        )

    paired = result.get("pairedHouseholdEffects")
    if not isinstance(paired, list) or len(paired) != len(schedules):
        raise AssertionError(
            f"current paired summary must contain one group per demography schedule; got {len(paired) if isinstance(paired, list) else 'non-list'}"
        )
    represented_pairs = 0
    for effect in paired:
        if not isinstance(effect, dict):
            raise AssertionError("paired household effect is not an object")
        if effect.get("fixedHouseholdLifecycle") != FIXED_LIFECYCLE:
            raise AssertionError("paired effect no longer binds to fixed_founder_v1 control")
        if effect.get("fissionHouseholdLifecycle") != treatment:
            raise AssertionError("paired effect treatment no longer matches confirmatory definition")
        represented_pairs += int(effect["pairedReplicates"])
        line = paired_line(effect)
        if line not in current:
            raise AssertionError(
                f"living demographic-baseline paired-effect narrative is stale for {effect.get('demography')}; expected line: {line}"
            )
    expected_pairs = len(schedules) * len(seeds) * len(founders) * len(resources)
    if represented_pairs != expected_pairs:
        raise AssertionError(f"paired contrast count drift: expected {expected_pairs}, result reports {represented_pairs}")
    if f"= {represented_pairs}/{represented_pairs} contrasts**" not in current:
        raise AssertionError("living demographic-baseline page does not report the current represented-pair total")

    if "**Status: historical/superseded.**" not in historical:
        raise AssertionError("preserved v1/64-seed narrative is not explicitly marked historical/superseded")
    if STALE_LIFECYCLE not in historical or "64-seed confirmation" not in historical:
        raise AssertionError("historical demographic-baseline page no longer preserves the superseded v1/64-seed provenance")
    if "general-scientific-demographic-baseline-v1-historical.md" not in current:
        raise AssertionError("living demographic-baseline page does not link to the preserved historical narrative")

    trace_link = "[`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md)"
    if trace_link not in trace:
        raise AssertionError("TRACE no longer links to the living current demographic-baseline page")
    if "general-scientific-demographic-baseline-v1-historical.md" in trace:
        raise AssertionError("TRACE points directly at the superseded historical demographic-baseline page")

    print(
        "Current demographic-baseline documentation is synchronized: "
        f"treatment={treatment}, seeds/arm={len(seeds)}, runs={expected_runs}, paired={represented_pairs}/{expected_pairs}."
    )


if __name__ == "__main__":
    main()
