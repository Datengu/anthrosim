#!/usr/bin/env python3
"""Validate survivor-conditioned condition estimands in frozen StudyProtocols.

This is a downstream research-governance gate. It does not alter simulator state.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

SURVIVOR_SOURCE = "meanLivingConditionPermille"
ESTIMAND_TOKEN = "estimand=survivor_condition_at_boundary"
CONDITIONING_TOKEN = "conditioning=survival"
DEATH_HANDLING_TOKEN = "death_handling=no_post_death_imputation"

SURVIVAL_SOURCE_MARKERS = (
    "finalLivingPopulation",
    "livingPopulation",
    "survival",
    "deathsSinceStart",
    "mortality",
    "populationExtinct",
)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text())


def is_survivor_condition_observable(observable: dict) -> bool:
    return SURVIVOR_SOURCE in str(observable.get("source", ""))


def is_survival_population_observable(observable: dict) -> bool:
    source = str(observable.get("source", ""))
    return any(marker in source for marker in SURVIVAL_SOURCE_MARKERS)


def validate_protocol(protocol: dict) -> dict:
    observables = {item["id"]: item for item in protocol.get("observables", [])}
    failures: list[str] = []
    survivor_ids: list[str] = []

    for observable_id, observable in observables.items():
        if not is_survivor_condition_observable(observable):
            continue
        survivor_ids.append(observable_id)
        interpretation = str(observable.get("interpretation", ""))
        for token in (ESTIMAND_TOKEN, CONDITIONING_TOKEN, DEATH_HANDLING_TOKEN):
            if token not in interpretation:
                failures.append(
                    f"observable {observable_id!r} uses {SURVIVOR_SOURCE} but lacks {token!r} "
                    "in its predeclared interpretation"
                )

    for comparison in protocol.get("comparisons", []):
        ids = comparison.get("observableIds", [])
        compared_survivor_ids = [oid for oid in ids if oid in survivor_ids]
        if not compared_survivor_ids:
            continue
        has_joint_survival = any(
            oid in observables and is_survival_population_observable(observables[oid])
            for oid in ids
        )
        if not has_joint_survival:
            failures.append(
                f"comparison {comparison.get('id')!r} uses survivor-conditioned condition "
                "without a jointly declared survival/population observable"
            )

    return {
        "schemaVersion": 1,
        "provenance": "derived",
        "studyId": protocol.get("studyId"),
        "survivorConditionObservableIds": survivor_ids,
        "valid": not failures,
        "failures": failures,
        "interpretation": (
            "meanLivingConditionPermille is descriptive among survivors only; a higher or lower "
            "value is not an unconditional population-level condition effect when mortality differs."
        ),
        "postDeathImputation": "none_automatic",
    }


def direction(delta: float) -> str:
    if delta > 0:
        return "higher"
    if delta < 0:
        return "lower"
    return "equal"


def assess_pair(control: dict, treatment: dict) -> dict:
    control_mean = control.get("meanLivingConditionPermille")
    treatment_mean = treatment.get("meanLivingConditionPermille")
    control_living = int(control["finalLivingPopulation"])
    treatment_living = int(treatment["finalLivingPopulation"])
    if control_mean is None or treatment_mean is None:
        mean_direction = "undefined"
    else:
        mean_direction = direction(float(treatment_mean) - float(control_mean))
    survival_direction = direction(treatment_living - control_living)
    discordant = (
        mean_direction == "higher" and survival_direction == "lower"
    ) or (
        mean_direction == "lower" and survival_direction == "higher"
    )
    return {
        "survivorMeanConditionDirection": mean_direction,
        "livingPopulationDirection": survival_direction,
        "discordantDirections": discordant,
        "survivorConditionIsPopulationTreatmentEffect": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("protocol", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    assessment = validate_protocol(load_json(args.protocol))
    encoded = json.dumps(assessment, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded)
    else:
        print(encoded, end="")
    if not assessment["valid"]:
        for failure in assessment["failures"]:
            print(f"research-survivor-conditioning: {failure}", file=__import__("sys").stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
