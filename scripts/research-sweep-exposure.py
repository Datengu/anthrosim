#!/usr/bin/env python3
"""Derive exposure-aware sweep outcomes from AnthroSim analysis/runs.json.

This analysis intentionally preserves raw cumulative outcomes while adding rates whose
only denominator is realized simulated time. It does not claim person-time,
household-opportunity, or population-exposure normalization.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
DAYS_PER_MODEL_YEAR = 365
ELIGIBLE = "eligibleScientificOutcome"
OPERATIONALLY_CENSORED = "operationallyCensored"
CUMULATIVE_FIELDS = (
    "birthsSinceStart",
    "deathsSinceStart",
    "conditionMortalityDeaths",
    "resourceUnmetNeed",
    "migrationMovesCompleted",
    "migrationTotalDistanceCells",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=Path, required=True, help="sweep analysis/runs.json")
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_sha256(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(encoded).hexdigest()


def mean(values: list[float | int]) -> float | None:
    if not values:
        return None
    return sum(values) / len(values)


def per_365(value: int | None, days: int | None) -> float | None:
    if value is None or days is None or days <= 0:
        return None
    return value * DAYS_PER_MODEL_YEAR / days


def run_record(row: dict[str, Any]) -> dict[str, Any]:
    status = row.get("scientificAggregationStatus")
    days = row.get("simulatedDays")
    eligible = status == ELIGIBLE
    rates = {
        f"{field}Per365SimulatedDays": per_365(row.get(field), days) if eligible else None
        for field in CUMULATIVE_FIELDS
    }
    return {
        "pointId": row.get("pointId"),
        "runId": row.get("runId"),
        "seed": row.get("seed"),
        "state": row.get("state"),
        "stopReason": row.get("stopReason"),
        "scientificAggregationStatus": status,
        "simulatedDays": days,
        "endDay": row.get("endDay"),
        "cumulative": {field: row.get(field) for field in CUMULATIVE_FIELDS},
        "per365SimulatedDays": rates,
    }


def summarize_point(rows: list[dict[str, Any]]) -> dict[str, Any]:
    eligible = [row for row in rows if row.get("scientificAggregationStatus") == ELIGIBLE]
    censored = [row for row in rows if row.get("scientificAggregationStatus") == OPERATIONALLY_CENSORED]
    duration = [row for row in eligible if row.get("stopReason") == "durationReached"]
    extinct = [row for row in eligible if row.get("stopReason") == "populationExtinct"]
    days = [int(row["simulatedDays"]) for row in eligible if row.get("simulatedDays") is not None]

    raw_means: dict[str, float | None] = {}
    rate_means: dict[str, float | None] = {}
    for field in CUMULATIVE_FIELDS:
        raw_values = [int(row[field]) for row in eligible if row.get(field) is not None]
        rates = [
            rate
            for row in eligible
            if (rate := per_365(row.get(field), row.get("simulatedDays"))) is not None
        ]
        raw_means[f"mean{field[0].upper()}{field[1:]}Cumulative"] = mean(raw_values)
        rate_means[f"mean{field[0].upper()}{field[1:]}Per365SimulatedDays"] = mean(rates)

    return {
        "plannedRuns": len(rows),
        "scientificallyEligibleRuns": len(eligible),
        "durationReachedRuns": len(duration),
        "populationExtinctRuns": len(extinct),
        "operationallyCensoredRuns": len(censored),
        "populationExtinctionFractionScientificallyEligibleOnly": (
            len(extinct) / len(eligible) if eligible else None
        ),
        "realizedExposure": {
            "meanSimulatedDaysScientificallyEligibleOnly": mean(days),
            "minSimulatedDaysScientificallyEligibleOnly": min(days) if days else None,
            "maxSimulatedDaysScientificallyEligibleOnly": max(days) if days else None,
        },
        "rawCumulativeMeansScientificallyEligibleOnly": raw_means,
        "meanPerRunRatesScientificallyEligibleOnly": rate_means,
        "sourceScientificallyEligibleRunIds": [
            f"{row.get('pointId')}/{row.get('runId')}" for row in eligible
        ],
        "sourceOperationallyCensoredRunIds": [
            f"{row.get('pointId')}/{row.get('runId')}" for row in censored
        ],
    }


def derive(rows: list[dict[str, Any]]) -> dict[str, Any]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        point_id = row.get("pointId")
        run_id = row.get("runId")
        if not isinstance(point_id, str) or not isinstance(run_id, str):
            raise ValueError("every sweep run row must contain string pointId and runId")
        grouped[point_id].append(row)

    result: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "provenance": "derived",
        "normalization": {
            "timeDenominator": "realizedSimulatedDays",
            "rateScaleDays": DAYS_PER_MODEL_YEAR,
            "interpretation": (
                "Per-365-simulated-day rates normalize only for realized simulated time. "
                "They are not person-time, household-opportunity, or population-exposure rates."
            ),
            "zeroDayRatePolicy": "undefinedNull",
        },
        "sourceRunsCanonicalSha256": canonical_sha256(rows),
        "runs": [run_record(row) for row in rows],
        "points": {
            point_id: summarize_point(point_rows)
            for point_id, point_rows in sorted(grouped.items())
        },
    }
    result["assessmentCanonicalSha256"] = canonical_sha256(result)
    return result


def main() -> None:
    args = parse_args()
    rows = read_json(args.runs)
    if not isinstance(rows, list):
        raise SystemExit("runs analysis must be a JSON array")
    result = derive(rows)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(result["assessmentCanonicalSha256"])


if __name__ == "__main__":
    main()
