#!/usr/bin/env python3
"""Fresh Audit-v5 Area-I control for equilibrium-window sensitivity gating."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "research-long-run-diagnostics.py"
SPEC = importlib.util.spec_from_file_location("long_run", SCRIPT)
assert SPEC and SPEC.loader
long_run = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(long_run)


def protocol() -> dict:
    return long_run.validate_protocol(
        {
            "schemaVersion": 1,
            "studyId": "audit-v5-area-i-window-control",
            "claimMode": "equilibrium_like",
            "analysisStartDay": 365,
            "windowSnapshots": 2,
            "requiredConsecutiveStableWindows": 1,
            "metrics": [
                {
                    "id": "population",
                    "sourcePointer": "/population/livingPopulation",
                    "maxAdjacentWindowMeanShiftPermille": 10,
                    "maxWithinWindowDriftPermille": 10,
                    "regimeBinWidth": 10,
                }
            ],
            # Available and stable under the late plateau.
            "runLengthSensitivityEndDays": [2555],
            "analysisStartSensitivityDays": [730],
            # Earlier endpoint is still strongly drifting and must fail robustness.
            "analysisEndSensitivityDays": [1460],
            "initializationCoordinateIds": [],
            "environmentCoordinateIds": [],
            "rationale": (
                "Audit-v5 control: a stable terminal plateau must not support an equilibrium-like "
                "claim when a predeclared earlier analysis end remains strongly drifting."
            ),
        }
    )


def source_run() -> dict:
    values = [100, 200, 300, 400, 400, 400, 400, 400]
    observations = [
        {"day": 365 * (index + 1), "values": {"population": value}}
        for index, value in enumerate(values)
    ]
    return {
        "runId": "run-1",
        "pointId": "point-1",
        "seed": 1,
        "initialization": "default",
        "environment": "default",
        "treatmentContext": "default",
        "observations": observations,
        "terminalDay": 365 * len(values),
        "stopReason": "durationReached",
    }


def main() -> None:
    result = long_run.assess_runs(protocol(), [source_run()], 0)
    run = result["runs"][0]
    earlier = run["analysisEndSensitivity"][0]["assessment"]

    print(
        "primary_status={}; primary_regime={}; earlier_end_status={}; earlier_end_regime={}; "
        "analysis_end_sensitivity={}; equilibrium_supported={}; gate={}".format(
            run["primary"]["status"],
            run["primary"]["regimeSignature"],
            earlier["status"],
            earlier["regimeSignature"],
            result["analysisEndSensitivityDetected"],
            result["equilibriumLikeClaimSupported"],
            result["researchGateStatus"],
        )
    )

    assert run["primary"]["status"] == "stable"
    assert earlier["status"] == "drifting"
    assert result["analysisEndSensitivityDetected"] is True
    assert result["equilibriumLikeClaimSupported"] is False
    assert result["researchGateStatus"] == "failed"


if __name__ == "__main__":
    main()
