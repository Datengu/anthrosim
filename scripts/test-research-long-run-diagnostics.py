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
        "analysisStartDay": MODULE.DAYS_PER_YEAR,
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
        "runLengthSensitivityEndDays": [16 * MODULE.DAYS_PER_YEAR],
        "analysisStartSensitivityDays": [2 * MODULE.DAYS_PER_YEAR],
        "analysisEndSensitivityDays": [16 * MODULE.DAYS_PER_YEAR],
        "initializationCoordinateIds": [],
        "environmentCoordinateIds": [],
    }
    value.update(overrides)
    return value


def write_research(root: Path, runs: list[dict]) -> None:
    research_id = "research-test"
    points = []
    analysis_rows = []
    for index, run in enumerate(runs):
        run_id = f"run-{index}"
        point_id = run.get("point_id", f"point-{index}")
        relative = Path("points") / f"point-{index:06}" / "runs" / run_id
        run_dir = root / relative
        run_dir.mkdir(parents=True)
        seed = run.get("seed", index + 1)
        experiment = {"seed": seed, "marker": run.get("marker", "same")}
        coordinates = run.get("coordinates", [])
        run_config = {"experiment": experiment, "spatial": None}
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

        values = run["values"]
        snapshots = [
            {
                "day": (snapshot_index + 1) * MODULE.DAYS_PER_YEAR,
                "population": {"livingPopulation": value},
            }
            for snapshot_index, value in enumerate(values)
        ]
        terminal_day = run.get("terminal_day", len(values) * MODULE.DAYS_PER_YEAR)
        if terminal_day % MODULE.DAYS_PER_YEAR != 0:
            snapshots.append(
                {
                    "day": terminal_day,
                    "population": {"livingPopulation": values[-1]},
                }
            )
        digest = 10_000 + index
        run_manifest = {
            "modelVersion": SOURCE["modelVersion"],
            "modelSemanticsId": SOURCE["modelSemanticsId"],
            "gitCommit": SOURCE["gitCommit"],
            "experiment": experiment,
            "stateDigest64": digest,
            "endTime": terminal_day,
            "stopReason": run.get("stop_reason", "durationReached"),
        }
        (run_dir / "manifest.json").write_text(
            json.dumps(run_manifest), encoding="utf-8"
        )
        (run_dir / "metrics.json").write_text(
            json.dumps(
                {
                    "schemaVersion": 3,
                    "cadence": "annual_boundary_plus_terminal",
                    "snapshots": snapshots,
                }
            ),
            encoding="utf-8",
        )
        analysis_rows.append(
            {
                "pointId": point_id,
                "runId": run_id,
                "seed": seed,
                "coordinates": coordinates,
                "resultingConfiguration": run_config,
                "relativeDir": str(relative),
                "attempt": 1,
                "state": run.get("state", "completed"),
                "stateDigest64": digest if run.get("state", "completed") == "completed" else None,
                "error": None,
            }
        )

    manifest = {
        "schemaVersion": 1,
        "researchId": research_id,
        "definitionIdentity": "definition-test",
        "source": SOURCE,
        "definition": {},
        "points": points,
    }
    (root / "analysis").mkdir(parents=True)
    (root / "research-manifest.json").write_text(
        json.dumps(manifest), encoding="utf-8"
    )
    (root / "research-plan.json").write_text(json.dumps(manifest), encoding="utf-8")
    (root / "analysis" / "runs.json").write_text(
        json.dumps(
            {"schemaVersion": 1, "researchId": research_id, "runs": analysis_rows}
        ),
        encoding="utf-8",
    )


def assess(runs: list[dict], raw_protocol: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="anthrosim-long-run-") as directory:
        root = Path(directory)
        write_research(root, runs)
        return MODULE.derive_assessment(root, raw_protocol)


def stable_values(value: int = 100, count: int = 20) -> list[int]:
    return [value] * count


def test_stable_replicates_pass_complete_equilibrium_gate() -> None:
    result = assess(
        [
            {"values": stable_values(), "seed": 1},
            {"values": stable_values(), "seed": 2},
        ],
        protocol(),
    )
    assert result["plannedRunCount"] == 2
    assert result["primaryClassificationCounts"] == {"stable": 2}
    assert result["stableRegimeFrequenciesByTreatmentContext"] == {
        "default": {"population=bin:10": 2}
    }
    assert result["multipleStableRegimesDetected"] is False
    assert result["requiredEquilibriumSensitivityCoverageComplete"] is True
    assert result["equilibriumLikeClaimSupported"] is True
    assert result["singleRegimePooledLongRunAverageSupported"] is True
    assert result["researchGateStatus"] == "passed"


def test_declared_cycle_is_preserved() -> None:
    cycle_protocol = protocol()
    cycle_protocol["metrics"][0]["cyclePeriodSnapshots"] = 2
    result = assess([{"values": [90, 110] * 10}], cycle_protocol)
    assert result["primaryClassificationCounts"] == {"cyclic_stable": 1}
    run = result["runs"][0]["primary"]
    assert run["status"] == "cyclic_stable"
    assert run["metrics"][0]["terminalCycleAmplitudePermille"] > 0
    assert result["researchGateStatus"] == "passed"


def test_drifting_trajectory_fails_equilibrium_gate() -> None:
    result = assess(
        [{"values": [100 + 10 * index for index in range(20)]}], protocol()
    )
    assert result["primaryClassificationCounts"] == {"drifting": 1}
    assert result["equilibriumLikeClaimSupported"] is False
    assert result["researchGateStatus"] == "failed"


def test_multiple_regimes_are_reported_by_initialization_not_pooled() -> None:
    runs = [
        {
            "values": stable_values(100),
            "seed": 1,
            "coordinates": [{"id": "initialization", "value": "A"}],
        },
        {
            "values": stable_values(100),
            "seed": 2,
            "coordinates": [{"id": "initialization", "value": "A"}],
        },
        {
            "values": stable_values(200),
            "seed": 3,
            "coordinates": [{"id": "initialization", "value": "B"}],
        },
        {
            "values": stable_values(200),
            "seed": 4,
            "coordinates": [{"id": "initialization", "value": "B"}],
        },
    ]
    result = assess(runs, protocol(initializationCoordinateIds=["initialization"]))
    assert result["multipleStableRegimesDetected"] is True
    assert result["initializationDependenceDetected"] is True
    assert result["equilibriumLikeClaimSupported"] is True
    assert result["singleRegimePooledLongRunAverageSupported"] is False
    assert set(result["stableRegimeFrequenciesByTreatmentContext"]["default"]) == {
        "population=bin:10",
        "population=bin:20",
    }


def test_frequency_shift_detected_even_when_regime_support_matches() -> None:
    runs = []
    seed = 1
    for initialization, values in (
        ("A", [100, 100, 100, 200]),
        ("B", [100, 200, 200, 200]),
    ):
        for terminal in values:
            runs.append(
                {
                    "values": stable_values(terminal),
                    "seed": seed,
                    "coordinates": [
                        {"id": "initialization", "value": initialization}
                    ],
                }
            )
            seed += 1
    result = assess(runs, protocol(initializationCoordinateIds=["initialization"]))
    assert result["initializationDependenceDetected"] is True
    groups = result["initializationRegimeFrequenciesByTreatmentContext"]["default"]
    assert set(groups) == {'initialization="A"', 'initialization="B"'}
    assert set(groups['initialization="A"']) == set(groups['initialization="B"'])


def test_seed_environment_context_can_be_multiregime() -> None:
    runs = [
        {
            "values": stable_values(100),
            "seed": 11,
            "coordinates": [{"id": "environment", "value": "same-world"}],
        },
        {
            "values": stable_values(200),
            "seed": 12,
            "coordinates": [{"id": "environment", "value": "same-world"}],
        },
    ]
    result = assess(runs, protocol(environmentCoordinateIds=["environment"]))
    assert result["stochasticMultiRegimeContexts"]
    assert result["multipleStableRegimesDetected"] is True


def test_unrelated_treatments_are_not_mislabelled_as_multiple_attractors() -> None:
    runs = [
        {
            "values": stable_values(100),
            "coordinates": [{"id": "treatment", "value": "low"}],
        },
        {
            "values": stable_values(200),
            "coordinates": [{"id": "treatment", "value": "high"}],
        },
    ]
    result = assess(runs, protocol())
    assert result["multipleStableRegimesDetected"] is False
    assert len(result["stableRegimeFrequenciesByTreatmentContext"]) == 2
    assert result["singleRegimePooledLongRunAverageSupported"] is False


def test_run_length_and_window_sensitivity_are_explicit() -> None:
    values = [220, 200, 180, 160, 140, 120, 110, 105] + [100] * 16
    result = assess(
        [{"values": values}],
        protocol(
            analysisStartDay=9 * MODULE.DAYS_PER_YEAR,
            runLengthSensitivityEndDays=[12 * MODULE.DAYS_PER_YEAR],
            analysisStartSensitivityDays=[15 * MODULE.DAYS_PER_YEAR],
            analysisEndSensitivityDays=[12 * MODULE.DAYS_PER_YEAR],
        ),
    )
    assert result["runs"][0]["primary"]["status"] == "stable"
    assert result["runLengthSensitivityDetected"] is True
    assert result["analysisStartSensitivityDetected"] is True
    assert result["analysisEndSensitivityDetected"] is True
    assert result["equilibriumLikeClaimSupported"] is False


def test_missing_sensitivity_coverage_fails_equilibrium_gate() -> None:
    result = assess(
        [{"values": stable_values()}],
        protocol(
            runLengthSensitivityEndDays=[],
            analysisStartSensitivityDays=[],
            analysisEndSensitivityDays=[],
        ),
    )
    assert result["runs"][0]["primary"]["status"] == "stable"
    assert result["requiredEquilibriumSensitivityCoverageComplete"] is False
    assert result["researchGateStatus"] == "failed"


def test_early_termination_cannot_masquerade_as_equilibrium() -> None:
    result = assess(
        [
            {
                "values": stable_values(),
                "stop_reason": "populationExtinct",
                "terminal_day": 20 * MODULE.DAYS_PER_YEAR + 10,
            }
        ],
        protocol(),
    )
    assert result["earlyTerminatedRunCount"] == 1
    assert result["equilibriumLikeClaimSupported"] is False
    assert result["researchGateStatus"] == "failed"


def test_unobserved_declared_analysis_end_is_insufficient_not_shortened() -> None:
    result = assess(
        [{"values": stable_values()}],
        protocol(analysisEndDayInclusive=25 * MODULE.DAYS_PER_YEAR),
    )
    primary = result["runs"][0]["primary"]
    assert primary["status"] == "insufficient_data"
    assert primary["reason"] == "declared_analysis_end_not_observed"
    assert result["researchGateStatus"] == "failed"


def test_explicitly_transient_question_does_not_require_stationarity() -> None:
    result = assess(
        [{"values": [100 + 10 * index for index in range(20)]}],
        protocol(
            claimMode="explicitly_transient",
            runLengthSensitivityEndDays=[],
            analysisStartSensitivityDays=[],
            analysisEndSensitivityDays=[],
        ),
    )
    assert result["primaryClassificationCounts"] == {"drifting": 1}
    assert result["researchGateStatus"] == "not_required"
    assert result["equilibriumLikeClaimSupported"] is False


def test_missing_or_tampered_planned_run_is_rejected() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-long-run-plan-") as directory:
        root = Path(directory)
        write_research(root, [{"values": stable_values()}, {"values": stable_values()}])
        analysis = json.loads((root / "analysis" / "runs.json").read_text(encoding="utf-8"))
        analysis["runs"].pop()
        (root / "analysis" / "runs.json").write_text(json.dumps(analysis), encoding="utf-8")
        try:
            MODULE.derive_assessment(root, protocol())
        except MODULE.LongRunDiagnosticError as error:
            assert "run set differs" in str(error)
        else:
            raise AssertionError("missing immutable planned run was accepted")


def test_immutable_research_binding_fails_closed() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-long-run-binding-") as directory:
        root = Path(directory)
        write_research(root, [{"values": stable_values()}])
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
    test_stable_replicates_pass_complete_equilibrium_gate()
    test_declared_cycle_is_preserved()
    test_drifting_trajectory_fails_equilibrium_gate()
    test_multiple_regimes_are_reported_by_initialization_not_pooled()
    test_frequency_shift_detected_even_when_regime_support_matches()
    test_seed_environment_context_can_be_multiregime()
    test_unrelated_treatments_are_not_mislabelled_as_multiple_attractors()
    test_run_length_and_window_sensitivity_are_explicit()
    test_missing_sensitivity_coverage_fails_equilibrium_gate()
    test_early_termination_cannot_masquerade_as_equilibrium()
    test_unobserved_declared_analysis_end_is_insufficient_not_shortened()
    test_explicitly_transient_question_does_not_require_stationarity()
    test_missing_or_tampered_planned_run_is_rejected()
    test_immutable_research_binding_fails_closed()
    print("research long-run diagnostic regression suite passed")


if __name__ == "__main__":
    main()
