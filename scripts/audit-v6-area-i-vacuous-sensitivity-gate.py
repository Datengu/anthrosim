#!/usr/bin/env python3
"""Audit-v6 Area-I adversary for vacuous long-run sensitivity coverage."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIAGNOSTIC = ROOT / "scripts" / "research-long-run-diagnostics.py"
DAYS = 365
SOURCE = {
    "modelVersion": "0.3.6",
    "modelSemanticsId": "anthrosim-model-semantics-v35",
    "gitCommit": "7d5e47309556e458477cd7283230871363b2c89a",
}


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_research(root: Path) -> None:
    values = [220, 200, 180, 160, 140, 120, 110, 105] + [100] * 16
    points = []
    rows = []
    for index, seed in enumerate((91_001, 91_002)):
        run_id = f"run-{index}"
        point_id = f"point-{index}"
        relative = Path("points") / f"point-{index:06}" / "runs" / run_id
        run_dir = root / relative
        run_dir.mkdir(parents=True)
        experiment = {"seed": seed, "marker": "vacuous-sensitivity-audit"}
        run_config = {"experiment": experiment, "spatial": None}
        coordinates: list[object] = []
        points.append(
            {
                "point": {
                    "schemaVersion": 1,
                    "index": index,
                    "pointId": point_id,
                    "coordinates": coordinates,
                    "runConfig": run_config,
                },
                "runs": [
                    {
                        "seed": seed,
                        "runId": run_id,
                        "relativeDir": str(relative),
                        "runConfig": run_config,
                    }
                ],
            }
        )
        digest = 88_000 + index
        snapshots = [
            {
                "day": (snapshot_index + 1) * DAYS,
                "population": {"livingPopulation": value},
            }
            for snapshot_index, value in enumerate(values)
        ]
        write_json(
            run_dir / "manifest.json",
            {
                **SOURCE,
                "experiment": experiment,
                "stateDigest64": digest,
                "endTime": len(values) * DAYS,
                "stopReason": "durationReached",
            },
        )
        write_json(
            run_dir / "metrics.json",
            {
                "schemaVersion": 3,
                "cadence": "annual_boundary_plus_terminal",
                "snapshots": snapshots,
            },
        )
        rows.append(
            {
                "pointId": point_id,
                "runId": run_id,
                "seed": seed,
                "coordinates": coordinates,
                "resultingConfiguration": run_config,
                "relativeDir": str(relative),
                "attempt": 1,
                "state": "completed",
                "stateDigest64": digest,
                "error": None,
            }
        )

    manifest = {
        "schemaVersion": 1,
        "researchId": "audit-v6-area-i-vacuous-sensitivity",
        "definitionIdentity": "audit-v6-area-i-vacuous-sensitivity-definition",
        "source": SOURCE,
        "definition": {},
        "points": points,
    }
    write_json(root / "research-manifest.json", manifest)
    write_json(root / "research-plan.json", manifest)
    write_json(
        root / "analysis" / "runs.json",
        {
            "schemaVersion": 1,
            "researchId": manifest["researchId"],
            "runs": rows,
        },
    )


def base_protocol() -> dict:
    return {
        "schemaVersion": 1,
        "studyId": "audit-v6-area-i-vacuous-sensitivity",
        "claimMode": "equilibrium_like",
        "analysisStartDay": 9 * DAYS,
        "windowSnapshots": 4,
        "requiredConsecutiveStableWindows": 2,
        "metrics": [
            {
                "id": "population",
                "sourcePointer": "/population/livingPopulation",
                "maxAdjacentWindowMeanShiftPermille": 20,
                "maxWithinWindowDriftPermille": 20,
                "regimeBinWidth": 10,
            }
        ],
        "initializationCoordinateIds": [],
        "environmentCoordinateIds": [],
    }


def run_diagnostic(research: Path, protocol: dict, name: str) -> dict:
    protocol_path = research.parent / f"{name}-protocol.json"
    output_path = research.parent / f"{name}-output.json"
    write_json(protocol_path, protocol)
    completed = subprocess.run(
        [
            sys.executable,
            str(DIAGNOSTIC),
            str(research),
            str(protocol_path),
            "--output",
            str(output_path),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"{name} diagnostic failed with {completed.returncode}\n"
            f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return json.loads(output_path.read_text(encoding="utf-8"))


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="anthrosim-av6-area-i-vacuous-") as directory:
        work = Path(directory)
        research = work / "research"
        write_research(research)

        primary_end = 24 * DAYS
        primary_start = 9 * DAYS

        vacuous_protocol = base_protocol()
        vacuous_protocol.update(
            {
                # Every required sensitivity family is formally non-empty, but each
                # requested assessment is exactly the primary analysis again.
                "runLengthSensitivityEndDays": [primary_end],
                "analysisStartSensitivityDays": [primary_start],
                "analysisEndSensitivityDays": [primary_end],
            }
        )
        vacuous = run_diagnostic(research, vacuous_protocol, "vacuous")

        assert vacuous["requiredEquilibriumSensitivityCoverageComplete"] is True
        assert vacuous["runLengthSensitivityDetected"] is False
        assert vacuous["analysisStartSensitivityDetected"] is False
        assert vacuous["analysisEndSensitivityDetected"] is False
        assert vacuous["equilibriumLikeClaimSupported"] is True
        assert vacuous["researchGateStatus"] == "passed"
        assert all(run["primary"]["status"] == "stable" for run in vacuous["runs"])

        control_protocol = base_protocol()
        control_protocol.update(
            {
                # Same primary trajectory, but these are genuine alternative analysis
                # choices already known to expose insufficient/non-robust early behavior.
                "runLengthSensitivityEndDays": [12 * DAYS],
                "analysisStartSensitivityDays": [15 * DAYS],
                "analysisEndSensitivityDays": [12 * DAYS],
            }
        )
        control = run_diagnostic(research, control_protocol, "nonvacuous-control")
        assert control["requiredEquilibriumSensitivityCoverageComplete"] is True
        assert control["equilibriumLikeClaimSupported"] is False
        assert control["researchGateStatus"] == "failed"
        assert (
            control["runLengthSensitivityDetected"]
            or control["analysisStartSensitivityDetected"]
            or control["analysisEndSensitivityDetected"]
        )

        print(f"primary_start_day={primary_start}")
        print(f"primary_end_day={primary_end}")
        print(
            "vacuous_declared_sensitivity="
            f"run_length:{primary_end},analysis_start:{primary_start},analysis_end:{primary_end}"
        )
        print(
            "vacuous_coverage_complete="
            f"{str(vacuous['requiredEquilibriumSensitivityCoverageComplete']).lower()}"
        )
        print(f"vacuous_gate_status={vacuous['researchGateStatus']}")
        print(
            "vacuous_equilibrium_supported="
            f"{str(vacuous['equilibriumLikeClaimSupported']).lower()}"
        )
        print(
            "nonvacuous_control_detected="
            f"run_length:{str(control['runLengthSensitivityDetected']).lower()},"
            f"analysis_start:{str(control['analysisStartSensitivityDetected']).lower()},"
            f"analysis_end:{str(control['analysisEndSensitivityDetected']).lower()}"
        )
        print(f"nonvacuous_control_gate_status={control['researchGateStatus']}")

        raise AssertionError(
            "predeclared Area-I oracle failed: the equilibrium-like research gate accepted "
            "formally complete run-length/start/end sensitivity coverage even though every "
            "declared sensitivity assessment exactly duplicated the primary analysis; the same "
            "trajectory fails when genuine alternative analysis choices are supplied"
        )


if __name__ == "__main__":
    raise SystemExit(main())
