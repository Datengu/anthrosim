#!/usr/bin/env python3
"""Audit-v6 Area-L control for cumulative-vs-realized-time exposure ranking.

Two scientific points deliberately reverse ordering between raw cumulative unmet need and
per-365-simulated-day intensity. The analysis must preserve both estimands and extinction status.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "research-sweep-exposure.py"
SPEC = importlib.util.spec_from_file_location("anthrosim_sweep_exposure_av6_l", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

ELIGIBLE = "eligibleScientificOutcome"


def row(point_id: str, run_id: str, seed: int, stop_reason: str, days: int, unmet: int) -> dict:
    return {
        "pointId": point_id,
        "runId": run_id,
        "seed": seed,
        "state": "completed",
        "stopReason": stop_reason,
        "scientificAggregationStatus": ELIGIBLE,
        "simulatedDays": days,
        "endDay": days,
        "birthsSinceStart": 0,
        "deathsSinceStart": 0,
        "conditionMortalityDeaths": 0,
        "resourceUnmetNeed": unmet,
        "migrationMovesCompleted": 0,
        "migrationTotalDistanceCells": 0,
    }


def main() -> int:
    # Point A survives ten years and accumulates the larger raw total, but lower annualized
    # intensity. Point B becomes extinct after one year with a smaller total but higher intensity.
    rows = [
        row("long-survivor", "run-a", 1, "durationReached", 3650, 10_000),
        row("early-extinction", "run-b", 2, "populationExtinct", 365, 2_000),
    ]
    result = MODULE.derive(rows)
    a = result["points"]["long-survivor"]
    b = result["points"]["early-extinction"]

    a_raw = a["rawCumulativeMeansScientificallyEligibleOnly"]["meanResourceUnmetNeedCumulative"]
    b_raw = b["rawCumulativeMeansScientificallyEligibleOnly"]["meanResourceUnmetNeedCumulative"]
    a_rate = a["meanPerRunRatesScientificallyEligibleOnly"]["meanResourceUnmetNeedPer365SimulatedDays"]
    b_rate = b["meanPerRunRatesScientificallyEligibleOnly"]["meanResourceUnmetNeedPer365SimulatedDays"]

    assert a_raw == 10_000.0
    assert b_raw == 2_000.0
    assert a_rate == 1_000.0
    assert b_rate == 2_000.0
    assert a_raw > b_raw
    assert a_rate < b_rate

    assert a["populationExtinctRuns"] == 0
    assert a["populationExtinctionFractionScientificallyEligibleOnly"] == 0.0
    assert b["populationExtinctRuns"] == 1
    assert b["populationExtinctionFractionScientificallyEligibleOnly"] == 1.0
    assert a["sourceScientificallyEligibleRunIds"] == ["long-survivor/run-a"]
    assert b["sourceScientificallyEligibleRunIds"] == ["early-extinction/run-b"]

    normalization = result["normalization"]
    assert normalization["timeDenominator"] == "realizedSimulatedDays"
    assert "not person-time" in normalization["interpretation"].lower()

    print(f"raw_cumulative_ranking=long-survivor({a_raw})>early-extinction({b_raw})")
    print(f"realized_time_rate_ranking=early-extinction({b_rate})>long-survivor({a_rate})")
    print(f"early_extinction_fraction={b['populationExtinctionFractionScientificallyEligibleOnly']}")
    print(f"time_denominator={normalization['timeDenominator']}")
    print("exposure_ranking_control=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
