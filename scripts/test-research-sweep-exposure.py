#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("research-sweep-exposure.py")
spec = importlib.util.spec_from_file_location("research_sweep_exposure", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)


def row(run_id, stop_reason, days, unmet_need, status="eligibleScientificOutcome"):
    return {
        "pointId": "point-000000",
        "runId": run_id,
        "seed": int(run_id[-1]),
        "state": "completed",
        "stopReason": stop_reason,
        "scientificAggregationStatus": status,
        "simulatedDays": days,
        "endDay": days,
        "birthsSinceStart": 10,
        "deathsSinceStart": 10,
        "conditionMortalityDeaths": 2,
        "resourceUnmetNeed": unmet_need,
        "migrationMovesCompleted": 4,
        "migrationTotalDistanceCells": 20,
    }


long = row("run-1", "durationReached", 3650, 10_000)
short = row("run-2", "populationExtinct", 365, 2_000)
censored = row("run-3", "personRecordLimitReached", 100, 999_999, "operationallyCensored")
result = module.derive([long, short, censored])
point = result["points"]["point-000000"]

assert point["scientificallyEligibleRuns"] == 2
assert point["populationExtinctRuns"] == 1
assert point["operationallyCensoredRuns"] == 1
assert point["populationExtinctionFractionScientificallyEligibleOnly"] == 0.5
assert point["realizedExposure"]["meanSimulatedDaysScientificallyEligibleOnly"] == 2007.5
assert point["realizedExposure"]["minSimulatedDaysScientificallyEligibleOnly"] == 365
assert point["realizedExposure"]["maxSimulatedDaysScientificallyEligibleOnly"] == 3650

raw = point["rawCumulativeMeansScientificallyEligibleOnly"]["meanResourceUnmetNeedCumulative"]
rate = point["meanPerRunRatesScientificallyEligibleOnly"]["meanResourceUnmetNeedPer365SimulatedDays"]
assert raw == 6000.0
assert rate == 1500.0
assert long["resourceUnmetNeed"] > short["resourceUnmetNeed"]
assert module.per_365(long["resourceUnmetNeed"], long["simulatedDays"]) == 1000.0
assert module.per_365(short["resourceUnmetNeed"], short["simulatedDays"]) == 2000.0

zero_day = row("run-4", "durationReached", 0, 10)
zero_result = module.derive([zero_day])
zero_run = zero_result["runs"][0]
assert zero_run["per365SimulatedDays"]["resourceUnmetNeedPer365SimulatedDays"] is None
assert module.per_365(0, 0) is None

assert result["normalization"]["timeDenominator"] == "realizedSimulatedDays"
assert "not person-time" in result["normalization"]["interpretation"].lower()
print("research sweep exposure tests passed")
