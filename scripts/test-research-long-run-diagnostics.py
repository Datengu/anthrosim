#!/usr/bin/env python3
"""Regression tests for research-long-run-diagnostics.py."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-long-run-diagnostics.py")
SPEC = importlib.util.spec_from_file_location("anthrosim_long_run", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

SOURCE = {
    "modelVersion": "0.3.0",
    "modelSemanticsId": "test-semantics",
    "gitCommit": "test-commit",
}


def protocol(**overrides):
    value = {
        "schemaVersion": 1,
        "studyId": "long-run-test",
        "claimMode": "equilibrium_like",
        "analysisStartDay": 0,
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
        "runLengthSensitivityEndDays": [],
        "analysisStartSensitivityDays": [],
        "initializationCoordinateIds": [],
        "environmentCoordinateIds": [],
    }
    value.update(overrides)
    return value


def write_research(root: Path, runs: list[dict]) -> None:
    research_id = "research-test"
    manifest = {
        "schemaVersion": 1,
        "researchId": research_id,
        "definitionIdentity": "definition-test",
        "source": SOURCE,
    }
    (root / "analysis").mkdir(parents=True)
    (root / "research-manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (root / "research-plan.json").write_text(json.dumps(manifest), encoding="utf-8")
    analysis_rows = []
    for index, run in enumerate(runs):
        run_id = f"run-{index}"
        relative = Path("points") / "point-000000" / "runs" / run_id
        run_dir = root / relative
        run_dir.mkdir(parents=True)
        seed = run.get("seed", index + 1)
        experiment = {"seed": seed, "marker": run.get("marker", "same")}
        coordinates = run.get("coordinates", [])
        values = run["values"]
        snapshots = [
            {
                "day": (snapshot_index + 1) * MODULE.DAYS_PER_YEAR,
                "population": {"livingPopulation": value},
            }
            for snapshot_index, value in enumerate(values)
        ]
        digest = 10_000 + index
        run_manifest = {
            "modelVersion": SOURCE["modelVersion"],
            "modelSemanticsId": SOURCE["modelSemanticsId"],
            "gitCommit": SOURCE["gitCommit"],
            "experiment": experiment,
            "stateDigest64": digest,
            "endTime": len(values) * MODULE.DAYS_PER_YEAR,
            "stopReason": "durationReached",
        }
        (run_dir / "manifest.json").write_text(json.dumps(run_manifest), encoding="utf-8")
        (run_dir / "metrics.json").write_text(
            json.dumps({"schemaVersion": 3, "cadence": "annual_boundary_plus_terminal", "snapshots": snapshots}),
            encoding="utf-8",
        )
        analysis_rows.append(
            {
                "pointId": "point-test",
                "runId": run_id,
                "seed": seed,
                "coordinates": coordinates,
                "resultingConfiguration": {"experiment": experiment},
                "relativeDir": str(relative),
                "attempt": 1,
                "state": "completed",
                "stateDigest64": digest,
                "error": None,
            }
        )
    (root / "analysis" / "runs.json").write_text(
        json.dumps({"schemaVersion": 1, "researchId": research_id, "runs": analysis_rows}),
        encoding="utf-8",
    )


def assess(runs: list[dict], raw_protocol: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="anthrosim-long-run-") as directory:
        root = Path(directory)
        write_research(root, runs)
        return MODULE.derive_assessment(root, raw_protocol)


def test_stable_replicates() -> None:
    result = assess(
        [{"values": [100] * 12, "seed": 1}, {"values": [100] * 12, "seed": 2}],
        protocol(),
    )
    assert result["primaryClassificationCounts"] == {"stable": 2}
    assert result["stableRegimeFrequencies"] == {"population=bin:10": 2}
    assert result["multipleStableRegimesDetected"] is False
    assert result["equilibriumLikeClaimSupported"] is True
    assert result["singleRegimePooledLongRunAverageSupported"] is True
    assert result["researchGateStatus"] == "passed"


def test_declared_cycle_is_preserved() -> None:
    cycle_protocol = protocol()
    cycle_protocol["metrics"][0]["cyclePeriodSnapshots"] = 2
    result = assess([{"values": [90, 110] * 6}], cycle_protocol)
    assert result["primaryClassificationCounts"] == {"cyclic_stable": 1}
    run = result["runs"][0]["primary"]
    assert run["status"] == "cyclic_stable"
    assert run["metrics"][0]["terminalCycleAmplitudePermille"] > 0
    assert result["researchGateStatus"] == "passed"


def test_drifting_trajectory_fails_equilibrium_gate() -> None:
    result = assess([{"values": [100 + 10 * index for index in range(12)]}], protocol())
    assert result["primaryClassificationCounts"] == {"drifting": 1}
    assert result["equilibriumLikeClaimSupported"] is False
    assert result["researchGateStatus"] == "failed"


def test_multiple_regimes_are_reported_by_initialization() -> None:
    runs = [
        {
            "values": [100] * 12,
            "seed": 1,
            "coordinates": [{"id": "initialization", "value": "A"}],
        },
        {
            "values": [100] * 12,
            "seed": 2,
            "coordinates": [{"id": "initialization", "value": "A"}],
        },
        {
            "values": [200] * 12,
            "seed": 3,
            "coordinates": [{"id": "initialization", "value": "B"}],
        },
        {
            "values": [200] * 12,
            "seed": 4,
            "coordinates": [{"id": "initialization", "value": "B"}],
        },
    ]
    result = assess(runs, protocol(initializationCoordinateIds=["initialization"]))
    assert result["multipleStableRegimesDetected"] is True
    assert result["initializationDependenceDetected"] is True
    assert result["equilibriumLikeClaimSupported"] is True
    assert result["singleRegimePooledLongRunAverageSupported"] is False
    assert set(result["stableRegimeFrequencies"]) == {"population=bin:10", "population=bin:20"}


def test_seed_environment_context_can_be_multiregime() -> None:
    runs = [
        {
            "values": [100] * 12,
            "seed": 11,
            "coordinates": [{"id": "environment", "value": "same-world"}],
        },
        {
            "values": [200] * 12,
            "seed": 12,
            "coordinates": [{"id": "environment", "value": "same-world"}],
        },
    ]
    result = assess(runs, protocol(environmentCoordinateIds=["environment"]))
    assert result["stochasticMultiRegimeContexts"]
    assert result["multipleStableRegimesDetected"] is True


def test_run_length_and_window_sensitivity_are_explicit() -> None:
    values = [220, 200, 180, 160, 140, 120, 110, 105] + [100] * 12
    result = assess(
        [{"values": values}],
        protocol(
            analysisStartDay=9 * MODULE.DAYS_PER_YEAR,
            runLengthSensitivityEndDays=[12 * MODULE.DAYS_PER_YEAR, 20 * MODULE.DAYS_PER_YEAR],
            analysisStartSensitivityDays=[15 * MODULE.DAYS_PER_YEAR],
        ),
    )
    assert result["runs"][0]["primary"]["status"] == "stable"
    assert result["runLengthSensitivityDetected"] is True
    assert result["analysisWindowSensitivityDetected"] is True
    assert result["equilibriumLikeClaimSupported"] is False


def test_explicitly_transient_question_does_not_require_stationarity() -> None:
    result = assess(
        [{"values": [100 + 10 * index for index in range(12)]}],
        protocol(claimMode="explicitly_transient"),
    )
    assert result["primaryClassificationCounts"] == {"drifting": 1}
    assert result["researchGateStatus"] == "not_required"
    assert result["equilibriumLikeClaimSupported"] is False


def test_immutable_research_binding_fails_closed() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-long-run-binding-") as directory:
        root = Path(directory)
        write_research(root, [{"values": [100] * 12}])
        plan = json.loads((root / "research-plan.json").read_text(encoding="utf-8"))
        plan["researchId"] = "different"
        (root / "research-plan.json").write_text(json.dumps(plan), encoding="utf-8")
        try:
            MODULE.derive_assessment(root, protocol())
        except MODULE.LongRunDiagnosticError as error:
            assert "differ" in str(error)
        else:
            raise AssertionError("mismatched immutable research metadata was accepted")


def main() -> None:
    test_stable_replicates()
    test_declared_cycle_is_preserved()
    test_drifting_trajectory_fails_equilibrium_gate()
    test_multiple_regimes_are_reported_by_initialization()
    test_seed_environment_context_can_be_multiregime()
    test_run_length_and_window_sensitivity_are_explicit()
    test_explicitly_transient_question_does_not_require_stationarity()
    test_immutable_research_binding_fails_closed()
    print("research long-run diagnostic regression suite passed")


if __name__ == "__main__":
    main()
