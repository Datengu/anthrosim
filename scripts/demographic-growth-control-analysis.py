#!/usr/bin/env python3
"""Reproduce the intrinsic demographic growth targets used by issue #239 controls.

This is a model-analysis tool, not authoritative simulation code. It evaluates a female-only
age x birth-spacing state transition under the committed M2 mortality/fertility schedule. It
intentionally excludes spatial local-male availability, resources, migration, and founder-age
transients so those effects remain separately diagnosable in replicated simulation.
"""

from __future__ import annotations

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTROL_DIR = ROOT / "research" / "demography-controls-v1"
MAX_AGE = 100
FEMALE_BIRTH_SHARE = 1.0 - 0.512
SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH = 3


def probability_at(age: int, bands: list[dict]) -> float:
    for band in bands:
        if band["startAgeYears"] <= age < band["endAgeYearsExclusive"]:
            return band["annualProbabilityPerMillion"] / 1_000_000.0
    return 0.0


def transition(schedule: dict, vector: list[float]) -> list[float]:
    width = SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH + 1
    out = [0.0] * len(vector)

    def index(age: int, cooldown: int) -> int:
        return age * width + cooldown

    for age in range(MAX_AGE):
        mortality = probability_at(age, schedule["mortalityBands"])
        fertility = probability_at(age, schedule["fertilityBands"])
        survival = 1.0 - mortality
        for cooldown in range(width):
            mass = vector[index(age, cooldown)]
            if mass == 0.0:
                continue
            if cooldown == 0 and fertility > 0.0:
                births = mass * survival * fertility
                out[index(0, 0)] += births * FEMALE_BIRTH_SHARE
                if age + 1 < MAX_AGE:
                    out[index(age + 1, 0)] += mass * survival * (1.0 - fertility)
                    out[index(age + 1, SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH)] += births
            elif age + 1 < MAX_AGE:
                out[index(age + 1, max(0, cooldown - 1))] += mass * survival
    return out


def dominant_growth_factor(schedule: dict) -> float:
    width = SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH + 1
    vector = [0.0] * (MAX_AGE * width)
    vector[0] = 1.0
    factor = 1.0
    for _ in range(10_000):
        out = transition(schedule, vector)
        norm = sum(out)
        if norm == 0.0:
            return 0.0
        out = [value / norm for value in out]
        if abs(norm - factor) < 1e-13:
            return norm
        vector = out
        factor = norm
    raise RuntimeError("growth-factor iteration did not converge")


def net_reproduction_daughters(schedule: dict) -> float:
    """Expected lifetime daughters per newborn female under the intrinsic schedule.

    This follows one female cohort through mortality and the same post-birth spacing state used by
    the dominant-growth calculation. It is a generation-replacement diagnostic (R0-like quantity),
    not an annual growth rate and not a realized spatial-simulation statistic.
    """

    width = SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH + 1
    cohort = [0.0] * width
    cohort[0] = 1.0
    expected_daughters = 0.0

    for age in range(MAX_AGE):
        mortality = probability_at(age, schedule["mortalityBands"])
        fertility = probability_at(age, schedule["fertilityBands"])
        survival = 1.0 - mortality
        next_cohort = [0.0] * width

        for cooldown, mass in enumerate(cohort):
            surviving_mass = mass * survival
            if cooldown == 0 and fertility > 0.0:
                births = surviving_mass * fertility
                expected_daughters += births * FEMALE_BIRTH_SHARE
                next_cohort[0] += surviving_mass * (1.0 - fertility)
                next_cohort[SKIPPED_ANNUAL_BOUNDARIES_AFTER_BIRTH] += births
            else:
                next_cohort[max(0, cooldown - 1)] += surviving_mass

        cohort = next_cohort

    return expected_daughters


def main() -> None:
    for filename in (
        "negative-growth-control.json",
        "replacement-control.json",
        "positive-growth-control.json",
    ):
        schedule = json.loads((CONTROL_DIR / filename).read_text(encoding="utf-8"))
        factor = dominant_growth_factor(schedule)
        daughters = net_reproduction_daughters(schedule)
        print(
            f"{schedule['scheduleId']}: lambda={factor:.12f}, "
            f"r=ln(lambda)={math.log(factor):.12f}/year, "
            f"netDaughtersPerNewbornFemale={daughters:.12f}"
        )


if __name__ == "__main__":
    main()
