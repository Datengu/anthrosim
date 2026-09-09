#!/usr/bin/env python3
"""Audit-v6 Area-L adversary for initialization dependence in drifting trajectories.

The long-run contract says initialization/environment dependence compares full normalized outcome
distributions within treatment context. This adversary checks whether large persistent differences
between two initialization arms remain visible when both trajectories are non-stationary/drifting.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "research-long-run-diagnostics.py"
SPEC = importlib.util.spec_from_file_location("anthrosim_long_run_av6_l", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

DAYS = MODULE.DAYS_PER_YEAR


def protocol() -> dict:
    return MODULE.validate_protocol(
        {
            "schemaVersion": 1,
            "studyId": "audit-v6-area-l-drifting-initialization-dependence",
            "claimMode": "explicitly_transient",
            "analysisStartDay": DAYS,
            "windowSnapshots": 4,
            "requiredConsecutiveStableWindows": 2,
            "metrics": [
                {
                    "id": "population",
                    "sourcePointer": "/population/livingPopulation",
                    "maxAdjacentWindowMeanShiftPermille": 5,
                    "maxWithinWindowDriftPermille": 5,
                    "regimeBinWidth": 10,
                }
            ],
            "runLengthSensitivityEndDays": [],
            "analysisStartSensitivityDays": [],
            "analysisEndSensitivityDays": [],
            "initializationCoordinateIds": ["founder_state"],
            "environmentCoordinateIds": [],
            "rationale": "Audit-only transient path-dependence/observability adversary.",
        }
    )


def source_run(run_id: str, seed: int, initialization: str, values: list[int]) -> dict:
    observations = [
        {"day": (index + 1) * DAYS, "values": {"population": value}}
        for index, value in enumerate(values)
    ]
    return {
        "runId": run_id,
        "pointId": run_id,
        "seed": seed,
        "initialization": f'founder_state="{initialization}"',
        "environment": "default",
        "treatmentContext": "default",
        "observations": observations,
        "terminalDay": len(values) * DAYS,
        "stopReason": "durationReached",
    }


def assess(runs: list[dict]) -> dict:
    return MODULE.assess_runs(protocol(), runs, 0)


def main() -> int:
    # Positive control: two stable initialization arms in materially different regimes must be
    # detected as initialization-dependent by the existing regime-signature machinery.
    stable = assess(
        [
            source_run("stable-a-1", 1, "A", [100] * 12),
            source_run("stable-a-2", 2, "A", [100] * 12),
            source_run("stable-b-1", 3, "B", [1000] * 12),
            source_run("stable-b-2", 4, "B", [1000] * 12),
        ]
    )
    assert stable["primaryClassificationCounts"] == {"stable": 4}
    assert stable["initializationDependenceDetected"] is True

    # Falsification arm: preserve the same ten-fold initialization separation while both arms
    # drift with the same relative slope. The trajectory level remains radically different at
    # every observation, but drifting runs have no regimeSignature.
    low = [100 + 10 * index for index in range(12)]
    high = [1000 + 100 * index for index in range(12)]
    drifting = assess(
        [
            source_run("drift-a-1", 11, "A", low),
            source_run("drift-a-2", 12, "A", low),
            source_run("drift-b-1", 13, "B", high),
            source_run("drift-b-2", 14, "B", high),
        ]
    )

    assert drifting["primaryClassificationCounts"] == {"drifting": 4}
    assert all(run["primary"]["regimeSignature"] is None for run in drifting["runs"])

    groups = drifting["initializationRegimeFrequenciesByTreatmentContext"]["default"]
    print(f"stable_control_initialization_dependence={str(stable['initializationDependenceDetected']).lower()}")
    print(f"drifting_classification_counts={drifting['primaryClassificationCounts']}")
    print(f"drifting_initialization_frequencies={groups}")
    print(f"drifting_initialization_dependence={str(drifting['initializationDependenceDetected']).lower()}")
    print(f"initialization_a_terminal_population={low[-1]}")
    print(f"initialization_b_terminal_population={high[-1]}")
    print(f"terminal_population_ratio={high[-1] / low[-1]:.1f}")

    if drifting["initializationDependenceDetected"] is not True:
        raise AssertionError(
            "predeclared Area-L oracle failed: initialization dependence is reported false for "
            "two same-treatment drifting trajectory families that remain separated by exactly "
            "10x at every observation; the aggregate outcome label collapses both families to "
            "status:drifting and discards the claim-relevant trajectory level"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
