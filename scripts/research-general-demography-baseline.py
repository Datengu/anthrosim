#!/usr/bin/env python3
"""Summarize the issue #304 general-demography baseline experiment."""

from __future__ import annotations

import argparse
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path
from typing import Any

DAYS_PER_YEAR = 365
Z95 = 1.959963984540054
U64_MAX = 18446744073709551615


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def mean_ci(values: list[float]) -> dict[str, Any]:
    if not values:
        return {"n": 0, "mean": None, "sd": None, "ci95Lower": None, "ci95Upper": None}
    mean = statistics.fmean(values)
    if len(values) < 2:
        return {"n": len(values), "mean": mean, "sd": None, "ci95Lower": None, "ci95Upper": None}
    sd = statistics.stdev(values)
    half = Z95 * sd / math.sqrt(len(values))
    return {"n": len(values), "mean": mean, "sd": sd, "ci95Lower": mean - half, "ci95Upper": mean + half}


def wilson(successes: int, n: int) -> dict[str, Any]:
    if n == 0:
        return {"n": 0, "successes": 0, "estimate": None, "ci95Lower": None, "ci95Upper": None}
    p = successes / n
    den = 1 + Z95 * Z95 / n
    center = (p + Z95 * Z95 / (2 * n)) / den
    half = Z95 * math.sqrt(p * (1 - p) / n + Z95 * Z95 / (4 * n * n)) / den
    return {"n": n, "successes": successes, "estimate": p, "ci95Lower": max(0.0, center - half), "ci95Upper": min(1.0, center + half)}


def linear_log_growth(snapshots: list[dict[str, Any]], start_year: int, end_year: int) -> float | None:
    pairs: list[tuple[float, float]] = []
    for snap in snapshots:
        day = int(snap["day"])
        if day % DAYS_PER_YEAR != 0:
            continue
        year = day / DAYS_PER_YEAR
        if year < start_year or year > end_year:
            continue
        pop = int(snap["population"]["livingPopulation"])
        if pop <= 0:
            return None
        pairs.append((year, math.log(pop)))
    if len(pairs) < 2:
        return None
    mx = statistics.fmean(x for x, _ in pairs)
    my = statistics.fmean(y for _, y in pairs)
    denom = sum((x - mx) ** 2 for x, _ in pairs)
    if denom == 0:
        return None
    slope = sum((x - mx) * (y - my) for x, y in pairs) / denom
    return math.expm1(slope)


def snapshot_at_or_before(snapshots: list[dict[str, Any]], year: int) -> dict[str, Any]:
    day = year * DAYS_PER_YEAR
    candidates = [snapshot for snapshot in snapshots if int(snapshot["day"]) <= day]
    if not candidates:
        raise ValueError(f"no metric snapshot at or before year {year}")
    return max(candidates, key=lambda snapshot: int(snapshot["day"]))


def coord_values(row: dict[str, Any]) -> dict[str, Any]:
    return {coordinate["id"]: coordinate["value"] for coordinate in row["coordinates"]}


def arm_key(coords: dict[str, Any]) -> tuple[str, str, int, int]:
    demography = coords["demography"]["scheduleId"]
    lifecycle_value = coords["household_lifecycle"]
    lifecycle = "fixed_founder_v1" if lifecycle_value is None else lifecycle_value["modelId"]
    founder = int(coords["founder_age_ceiling_years"])
    resource = int(coords["resource_productivity_scale_permille"])
    return demography, lifecycle, founder, resource


def alive_sex_fraction(checkpoint: dict[str, Any]) -> tuple[float | None, int, int]:
    population = checkpoint["population"]
    sexes = population["reproductiveSexes"]
    deaths = population["deathDays"]
    male = 0
    female = 0
    for sex, death in zip(sexes, deaths, strict=True):
        if int(death) != U64_MAX:
            continue
        if sex == "male":
            male += 1
        elif sex == "female":
            female += 1
    total = male + female
    return (male / total if total else None), male, female


def read_runs(root: Path, observability_root: Path) -> list[dict[str, Any]]:
    rows = load(root / "analysis/runs.json")["runs"]
    output = []
    for row in rows:
        if row["state"] != "completed":
            raise ValueError(f"non-completed run: {row['runId']} state={row['state']}")
        coords = coord_values(row)
        demography, lifecycle, founder, resource = arm_key(coords)
        run_dir = root / row["relativeDir"]
        snapshots = load(run_dir / "metrics.json")["snapshots"]
        checkpoint = load(run_dir / "checkpoint.json")
        manifest = load(run_dir / "manifest.json")
        observability = load(observability_root / f"{row['runId']}.json")
        terminal = snapshots[-1]
        late_base = snapshot_at_or_before(snapshots, 120)
        late_births = int(terminal["population"]["birthsSinceStart"]) - int(late_base["population"]["birthsSinceStart"])
        late_deaths = int(terminal["population"]["deathsSinceStart"]) - int(late_base["population"]["deathsSinceStart"])
        summary = observability["summary"]
        spacing_eligible = int(summary["spacingEligible"])
        local_male_eligible = int(summary["localMaleEligible"])
        age_eligible = int(summary["ageScheduleEligible"])
        male_fraction, male_living, female_living = alive_sex_fraction(checkpoint)
        extinction_year = None
        for snapshot in snapshots:
            if int(snapshot["population"]["livingPopulation"]) == 0:
                extinction_year = int(snapshot["day"]) / DAYS_PER_YEAR
                break
        output.append(
            {
                "runId": row["runId"],
                "seed": int(row["seed"]),
                "demography": demography,
                "householdLifecycle": lifecycle,
                "founderAgeCeilingYears": founder,
                "resourceProductivityScalePermille": resource,
                "terminalPopulation": int(terminal["population"]["livingPopulation"]),
                "births": int(terminal["population"]["birthsSinceStart"]),
                "deaths": int(terminal["population"]["deathsSinceStart"]),
                "lateBirths": late_births,
                "lateDeaths": late_deaths,
                "lateBirthDeathRatio": late_births / late_deaths if late_deaths else None,
                "earlyGrowthRatePerYear": linear_log_growth(snapshots, 1, 40),
                "lateGrowthRatePerYear": linear_log_growth(snapshots, 120, 240),
                "extinct": int(terminal["population"]["livingPopulation"]) == 0,
                "extinctionYear": extinction_year,
                "mateLimitationFraction": 1 - local_male_eligible / spacing_eligible if spacing_eligible else None,
                "spacingLimitationFraction": 1 - spacing_eligible / age_eligible if age_eligible else None,
                "successfulBirths": int(summary["successfulBirths"]),
                "maleLiving": male_living,
                "femaleLiving": female_living,
                "maleLivingFraction": male_fraction,
                "stopReason": manifest["stopReason"],
            }
        )
    return output


def summarize_arms(runs: list[dict[str, Any]]) -> list[dict[str, Any]]:
    grouped: dict[tuple[str, str, int, int], list[dict[str, Any]]] = defaultdict(list)
    for run in runs:
        grouped[(run["demography"], run["householdLifecycle"], run["founderAgeCeilingYears"], run["resourceProductivityScalePermille"])].append(run)
    result = []
    for key, rows in sorted(grouped.items()):
        demography, lifecycle, founder, resource = key
        growth = [run["lateGrowthRatePerYear"] for run in rows if run["lateGrowthRatePerYear"] is not None]
        mate = [run["mateLimitationFraction"] for run in rows if run["mateLimitationFraction"] is not None]
        spacing = [run["spacingLimitationFraction"] for run in rows if run["spacingLimitationFraction"] is not None]
        male = [run["maleLivingFraction"] for run in rows if run["maleLivingFraction"] is not None]
        ratios = [run["lateBirthDeathRatio"] for run in rows if run["lateBirthDeathRatio"] is not None]
        terminal_values = [run["terminalPopulation"] for run in rows]
        result.append(
            {
                "demography": demography,
                "householdLifecycle": lifecycle,
                "founderAgeCeilingYears": founder,
                "resourceProductivityScalePermille": resource,
                "replicates": len(rows),
                "terminalPopulation": {**mean_ci(terminal_values), "median": statistics.median(terminal_values), "min": min(terminal_values), "max": max(terminal_values)},
                "earlyGrowthRatePerYear": mean_ci([run["earlyGrowthRatePerYear"] for run in rows if run["earlyGrowthRatePerYear"] is not None]),
                "lateGrowthRatePerYear": mean_ci(growth),
                "extinction": wilson(sum(run["extinct"] for run in rows), len(rows)),
                "births": mean_ci([run["births"] for run in rows]),
                "deaths": mean_ci([run["deaths"] for run in rows]),
                "lateBirthDeathRatio": mean_ci(ratios),
                "mateLimitationFraction": mean_ci(mate),
                "spacingLimitationFraction": mean_ci(spacing),
                "terminalMaleFraction": mean_ci(male),
            }
        )
    return result


def paired_household_effects(runs: list[dict[str, Any]]) -> list[dict[str, Any]]:
    cells: dict[tuple[str, int, int, int], dict[str, dict[str, Any]]] = defaultdict(dict)
    for run in runs:
        cells[(run["demography"], run["founderAgeCeilingYears"], run["resourceProductivityScalePermille"], run["seed"])][run["householdLifecycle"]] = run
    grouped: dict[tuple[str, int, int], list[tuple[dict[str, Any], dict[str, Any]]]] = defaultdict(list)
    for (demography, founder, resource, _seed), by_lifecycle in cells.items():
        if "fixed_founder_v1" in by_lifecycle and "deterministic_size_fission_v1" in by_lifecycle:
            grouped[(demography, founder, resource)].append((by_lifecycle["deterministic_size_fission_v1"], by_lifecycle["fixed_founder_v1"]))
    result = []
    for key, pairs in sorted(grouped.items()):
        def differences(field: str) -> list[float]:
            values = []
            for fission, fixed in pairs:
                left, right = fission[field], fixed[field]
                if left is not None and right is not None:
                    values.append(float(left) - float(right))
            return values
        result.append(
            {
                "demography": key[0],
                "founderAgeCeilingYears": key[1],
                "resourceProductivityScalePermille": key[2],
                "pairedReplicates": len(pairs),
                "fissionMinusFixedTerminalPopulation": mean_ci(differences("terminalPopulation")),
                "fissionMinusFixedLateGrowthRatePerYear": mean_ci(differences("lateGrowthRatePerYear")),
                "fissionMinusFixedMateLimitationFraction": mean_ci(differences("mateLimitationFraction")),
            }
        )
    return result


def summarize_long_run(path: Path | None) -> dict[str, Any] | None:
    if path is None:
        return None
    assessment = load(path)
    by_context: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    sensitivity_bad: dict[str, int] = defaultdict(int)
    for run in assessment["runs"]:
        by_context[run["treatmentContext"]][run["primary"]["status"]] += 1
        primary = (run["primary"]["status"], run["primary"]["regimeSignature"])
        changed = False
        for item in run["runLengthSensitivity"]:
            value = item["assessment"]
            if value is not None and (value["status"], value["regimeSignature"]) != primary:
                changed = True
        for item in run["analysisStartSensitivity"]:
            value = item["assessment"]
            if (value["status"], value["regimeSignature"]) != primary:
                changed = True
        for item in run["analysisEndSensitivity"]:
            value = item["assessment"]
            if value is not None and (value["status"], value["regimeSignature"]) != primary:
                changed = True
        sensitivity_bad[run["treatmentContext"]] += int(changed)
    return {
        "researchGateStatus": assessment["researchGateStatus"],
        "primaryClassificationCounts": assessment["primaryClassificationCounts"],
        "multipleStableRegimesDetected": assessment["multipleStableRegimesDetected"],
        "initializationDependenceDetected": assessment["initializationDependenceDetected"],
        "environmentDependenceDetected": assessment["environmentDependenceDetected"],
        "stochasticMultiRegimeContextCount": len(assessment["stochasticMultiRegimeContexts"]),
        "statusCountsByTreatmentContext": {key: dict(value) for key, value in sorted(by_context.items())},
        "sensitivityChangedRunCountByTreatmentContext": dict(sorted(sensitivity_bad.items())),
    }


def print_compact(summary: dict[str, Any]) -> None:
    print("ISSUE #304 EXPLORATORY NUMERICAL SUMMARY")
    print(f"runs={summary['runCount']}")
    for arm in summary["arms"]:
        growth = arm["lateGrowthRatePerYear"]["mean"]
        if growth is None:
            growth_text = "NA"
        else:
            lower = arm["lateGrowthRatePerYear"]["ci95Lower"]
            upper = arm["lateGrowthRatePerYear"]["ci95Upper"]
            growth_text = f"{100 * growth:+.3f}%/yr [{100 * lower:+.3f}, {100 * upper:+.3f}]"
        terminal = arm["terminalPopulation"]["mean"]
        extinction = arm["extinction"]["estimate"]
        mate = arm["mateLimitationFraction"]["mean"]
        print(f"{arm['demography']} | {arm['householdLifecycle']} | founder<={arm['founderAgeCeilingYears']} | resource={arm['resourceProductivityScalePermille']}: N240={terminal:.1f}, late_growth={growth_text}, extinction={100 * extinction:.1f}%, mate_block={100 * mate:.1f}%")
    if summary.get("longRun"):
        long_run = summary["longRun"]
        keys = ("researchGateStatus", "primaryClassificationCounts", "multipleStableRegimesDetected", "initializationDependenceDetected", "environmentDependenceDetected", "stochasticMultiRegimeContextCount")
        print("LONG-RUN:", json.dumps({key: long_run[key] for key in keys}, sort_keys=True))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("research_root", type=Path)
    parser.add_argument("observability_root", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--long-run", type=Path)
    args = parser.parse_args()
    runs = read_runs(args.research_root, args.observability_root)
    summary = {
        "schemaVersion": 1,
        "purpose": "Issue #304 exploratory general-demography baseline analysis; not itself a final calibrated demographic claim.",
        "researchId": load(args.research_root / "research-manifest.json")["researchId"],
        "runCount": len(runs),
        "runs": runs,
        "arms": summarize_arms(runs),
        "pairedHouseholdEffects": paired_household_effects(runs),
        "longRun": summarize_long_run(args.long_run),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print_compact(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
