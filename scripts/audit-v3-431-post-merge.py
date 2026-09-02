#!/usr/bin/env python3
"""Independent post-merge adversary for Audit-v3 AV3-016.

This checker deliberately does not import the production documentation guard.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEF = ROOT / "research/general-demography-baseline-v1/confirmatory-definition.json"
RES = ROOT / "research/general-demography-baseline-v1/confirmatory-result.json"
DOC = ROOT / "docs/research/general-scientific-demographic-baseline-v1.md"
HIST = ROOT / "docs/research/general-scientific-demographic-baseline-v1-historical.md"
TRACE = ROOT / "docs/research/trace.md"
FIXED = "fixed_founder_v1"
STALE = "deterministic_size_fission_v1"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def dim(defn, name):
    found = [d for d in defn["dimensions"] if d.get("id") == name]
    assert len(found) == 1, f"dimension {name}: expected 1, got {len(found)}"
    return found[0]


def declared_treatment(defn):
    vals = dim(defn, "household_lifecycle")["values"]
    objs = [v for v in vals if isinstance(v, dict)]
    assert len(vals) == 2 and len(objs) == 1 and None in vals
    model = objs[0].get("modelId")
    assert isinstance(model, str) and model
    return model


def schedule_ids(defn):
    return [v["scheduleId"] for v in dim(defn, "demography")["values"]]


def verify(current: str, historical: str, trace: str, defn, result):
    treatment = declared_treatment(defn)
    seeds = defn["seeds"]
    schedules = schedule_ids(defn)
    founders = dim(defn, "founder_age_ceiling_years")["values"]
    resources = dim(defn, "resource_productivity_scale_permille")["values"]
    lifecycle_arms = dim(defn, "household_lifecycle")["values"]
    expected_runs = len(seeds) * len(schedules) * len(founders) * len(resources) * len(lifecycle_arms)
    assert result["runCount"] == expected_runs

    assert "living/current TRACE-linked result" in current
    assert treatment in current
    assert f"fresh process seeds per arm: **{len(seeds)}**" in current
    assert f"= {expected_runs} completed runs**" in current
    assert STALE not in current.replace("superseded `deterministic_size_fission_v1`", "")

    arms = result["arms"]
    expected_keys = {(d, life) for d in schedules for life in (FIXED, treatment)}
    observed_keys = {(a["demography"], a["householdLifecycle"]) for a in arms}
    assert observed_keys == expected_keys, (expected_keys, observed_keys)
    for arm in arms:
        d = arm["demography"]
        life = arm["householdLifecycle"]
        assert f"`{d}` | `{life}`" in current
        n240 = f"{float(arm['terminalPopulation']['mean']):.1f}"
        mate = f"{float(arm['mateLimitationFraction']['mean']) * 100:.1f}%"
        growth = f"{float(arm['lateGrowthRatePerYear']['mean']) * 100:+.3f}"
        assert n240 in current, (d, life, "N240", n240)
        assert mate in current, (d, life, "mate", mate)
        assert growth in current, (d, life, "growth", growth)

    effects = result["pairedHouseholdEffects"]
    assert len(effects) == len(schedules)
    represented = 0
    for effect in effects:
        assert effect["fixedHouseholdLifecycle"] == FIXED
        assert effect["fissionHouseholdLifecycle"] == treatment
        assert effect["pairedReplicates"] == len(seeds)
        represented += effect["pairedReplicates"]
        d = effect["demography"]
        mean = f"{float(effect['fissionMinusFixedTerminalPopulation']['mean']):+.1f} people"
        assert d in current and mean in current
    expected_pairs = len(schedules) * len(seeds) * len(founders) * len(resources)
    assert represented == expected_pairs
    assert f"{represented}/{expected_pairs} contrasts" in current

    assert "historical/superseded" in historical
    assert STALE in historical and "64-seed confirmation" in historical
    assert "general-scientific-demographic-baseline-v1-historical.md" in current
    assert "[`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md)" in trace
    assert "general-scientific-demographic-baseline-v1-historical.md" not in trace
    return treatment, len(seeds), expected_runs, represented, expected_pairs


def must_reject(label, current, historical, trace, defn, result):
    try:
        verify(current, historical, trace, defn, result)
    except (AssertionError, KeyError, TypeError, ValueError) as exc:
        print(f"{label}: rejected ({exc or 'contract mismatch'})")
        return
    raise AssertionError(f"{label}: unexpectedly accepted")


def main():
    defn = load(DEF)
    result = load(RES)
    current = DOC.read_text(encoding="utf-8")
    historical = HIST.read_text(encoding="utf-8")
    trace = TRACE.read_text(encoding="utf-8")

    treatment, seeds, runs, represented, expected = verify(current, historical, trace, defn, result)
    print(
        "AV3-016 post-merge adversary: ok "
        f"(treatment={treatment}, seeds/arm={seeds}, runs={runs}, paired={represented}/{expected})"
    )

    # Recreate the core frozen failure: the current TRACE-linked page is the old v1/64 narrative.
    stale_current = historical.replace(
        "# General scientific demographic baseline result v1 — historical",
        "# General scientific demographic baseline result v1",
    ).replace(
        "**Status: historical/superseded.** This page preserves the original `deterministic_size_fission_v1` / 64-seed #304 narrative as provenance. It is **not** the current TRACE-linked demographic-baseline evidence. The living result is [`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md), synchronized to the authoritative current confirmatory definition/result.\n\n",
        "",
    )
    must_reject("original stale-v1/64 TRACE-page adversary", stale_current, historical, trace, defn, result)

    # Also prove future declaration drift is not silently tolerated by the living narrative.
    drift_def = copy.deepcopy(defn)
    for value in dim(drift_def, "household_lifecycle")["values"]:
        if isinstance(value, dict):
            value["modelId"] = "future_declared_household_treatment_v999"
    must_reject("declared-treatment drift adversary", current, historical, trace, drift_def, result)

    # TRACE must not be redirected straight to the historical page.
    stale_trace = trace.replace(
        "[`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md)",
        "[`general-scientific-demographic-baseline-v1-historical.md`](general-scientific-demographic-baseline-v1-historical.md)",
    )
    must_reject("historical TRACE-target adversary", current, historical, stale_trace, defn, result)


if __name__ == "__main__":
    main()
