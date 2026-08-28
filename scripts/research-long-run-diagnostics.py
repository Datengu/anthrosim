#!/usr/bin/env python3
"""Diagnose stable, cyclic, drifting and multi-regime AnthroSim trajectories."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

DAYS_PER_YEAR = 365
PROTOCOL_SCHEMA_VERSION = 1
OUTPUT_SCHEMA_VERSION = 1
MANIFEST_TYPE = "anthrosim-long-run-diagnostics"
CLAIM_MODES = {"equilibrium_like", "explicitly_transient"}


class LongRunDiagnosticError(Exception):
    """Raised when long-run analysis cannot be derived safely."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Derive versioned long-run stationarity/path-dependence diagnostics from an "
            "immutable anthrosim-research execution."
        )
    )
    parser.add_argument("research_root", type=Path)
    parser.add_argument("protocol", type=Path)
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise LongRunDiagnosticError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink():
        raise LongRunDiagnosticError(f"{role} must not be a symbolic link: {path}")
    if not path.is_file():
        raise LongRunDiagnosticError(f"{role} is missing or not a regular file: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys)
    except LongRunDiagnosticError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise LongRunDiagnosticError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise LongRunDiagnosticError(f"{role} root must be a JSON object")
    return value


def require_exact_keys(value: dict[str, Any], required: set[str], optional: set[str], role: str) -> None:
    missing = required - set(value)
    unknown = set(value) - required - optional
    if missing:
        raise LongRunDiagnosticError(f"{role} missing field(s): {', '.join(sorted(missing))}")
    if unknown:
        raise LongRunDiagnosticError(f"{role} contains unknown field(s): {', '.join(sorted(unknown))}")


def require_nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise LongRunDiagnosticError(f"{role} must be a non-empty string")
    return value


def require_uint(value: Any, role: str, *, positive: bool = False) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < (1 if positive else 0):
        qualifier = "positive" if positive else "non-negative"
        raise LongRunDiagnosticError(f"{role} must be a {qualifier} integer")
    return value


def require_uint_list(value: Any, role: str) -> list[int]:
    if not isinstance(value, list):
        raise LongRunDiagnosticError(f"{role} must be an array")
    result = [require_uint(item, f"{role}[{index}]") for index, item in enumerate(value)]
    if len(set(result)) != len(result):
        raise LongRunDiagnosticError(f"{role} must not contain duplicate values")
    return sorted(result)


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def protocol_identity(protocol: dict[str, Any]) -> str:
    digest = hashlib.sha256(canonical_bytes(protocol)).hexdigest()
    return f"long-run-protocol-v1-sha256-{digest}"


def validate_metric(value: Any, index: int) -> dict[str, Any]:
    role = f"metrics[{index}]"
    if not isinstance(value, dict):
        raise LongRunDiagnosticError(f"{role} must be an object")
    require_exact_keys(
        value,
        {
            "id",
            "sourcePointer",
            "maxAdjacentWindowMeanShiftPermille",
            "maxWithinWindowDriftPermille",
            "regimeBinWidth",
        },
        {"cyclePeriodSnapshots"},
        role,
    )
    metric_id = require_nonempty_string(value["id"], f"{role}.id")
    pointer = require_nonempty_string(value["sourcePointer"], f"{role}.sourcePointer")
    if not pointer.startswith("/"):
        raise LongRunDiagnosticError(f"{role}.sourcePointer must be an RFC 6901-style absolute pointer")
    mean_shift = require_uint(value["maxAdjacentWindowMeanShiftPermille"], f"{role}.maxAdjacentWindowMeanShiftPermille")
    drift = require_uint(value["maxWithinWindowDriftPermille"], f"{role}.maxWithinWindowDriftPermille")
    if mean_shift > 1000 or drift > 1000:
        raise LongRunDiagnosticError(f"{role} permille tolerances must be <= 1000")
    bin_width = require_uint(value["regimeBinWidth"], f"{role}.regimeBinWidth", positive=True)
    cycle_period = value.get("cyclePeriodSnapshots")
    if cycle_period is not None:
        cycle_period = require_uint(cycle_period, f"{role}.cyclePeriodSnapshots", positive=True)
        if cycle_period < 2:
            raise LongRunDiagnosticError(f"{role}.cyclePeriodSnapshots must be >= 2 when declared")
    return {
        "id": metric_id,
        "sourcePointer": pointer,
        "maxAdjacentWindowMeanShiftPermille": mean_shift,
        "maxWithinWindowDriftPermille": drift,
        "regimeBinWidth": bin_width,
        **({"cyclePeriodSnapshots": cycle_period} if cycle_period is not None else {}),
    }


def validate_protocol(value: dict[str, Any]) -> dict[str, Any]:
    require_exact_keys(
        value,
        {
            "schemaVersion",
            "studyId",
            "claimMode",
            "analysisStartDay",
            "windowSnapshots",
            "requiredConsecutiveStableWindows",
            "metrics",
        },
        {
            "analysisEndDayInclusive",
            "runLengthSensitivityEndDays",
            "analysisStartSensitivityDays",
            "initializationCoordinateIds",
            "environmentCoordinateIds",
            "rationale",
        },
        "protocol",
    )
    if value["schemaVersion"] != PROTOCOL_SCHEMA_VERSION:
        raise LongRunDiagnosticError(
            f"protocol schemaVersion must be {PROTOCOL_SCHEMA_VERSION}, found {value['schemaVersion']}"
        )
    study_id = require_nonempty_string(value["studyId"], "protocol.studyId")
    claim_mode = require_nonempty_string(value["claimMode"], "protocol.claimMode")
    if claim_mode not in CLAIM_MODES:
        raise LongRunDiagnosticError(f"protocol.claimMode must be one of {sorted(CLAIM_MODES)}")
    start = require_uint(value["analysisStartDay"], "protocol.analysisStartDay")
    end = value.get("analysisEndDayInclusive")
    if end is not None:
        end = require_uint(end, "protocol.analysisEndDayInclusive")
        if end < start:
            raise LongRunDiagnosticError("analysisEndDayInclusive must be >= analysisStartDay")
    window = require_uint(value["windowSnapshots"], "protocol.windowSnapshots", positive=True)
    stable_windows = require_uint(
        value["requiredConsecutiveStableWindows"],
        "protocol.requiredConsecutiveStableWindows",
        positive=True,
    )
    metrics_value = value["metrics"]
    if not isinstance(metrics_value, list) or not metrics_value:
        raise LongRunDiagnosticError("protocol.metrics must be a non-empty array")
    metrics = [validate_metric(metric, index) for index, metric in enumerate(metrics_value)]
    ids = [metric["id"] for metric in metrics]
    if len(set(ids)) != len(ids):
        raise LongRunDiagnosticError("protocol.metrics contains duplicate ids")
    for metric in metrics:
        period = metric.get("cyclePeriodSnapshots")
        if period is not None and window % period != 0:
            raise LongRunDiagnosticError(
                f"windowSnapshots must be divisible by cyclePeriodSnapshots for metric {metric['id']}"
            )
    init_ids = value.get("initializationCoordinateIds", [])
    env_ids = value.get("environmentCoordinateIds", [])
    for role, coordinate_ids in (
        ("initializationCoordinateIds", init_ids),
        ("environmentCoordinateIds", env_ids),
    ):
        if not isinstance(coordinate_ids, list):
            raise LongRunDiagnosticError(f"protocol.{role} must be an array")
        for index, coordinate_id in enumerate(coordinate_ids):
            require_nonempty_string(coordinate_id, f"protocol.{role}[{index}]")
        if len(set(coordinate_ids)) != len(coordinate_ids):
            raise LongRunDiagnosticError(f"protocol.{role} must not contain duplicates")
    overlap = set(init_ids) & set(env_ids)
    if overlap:
        raise LongRunDiagnosticError(
            "initializationCoordinateIds and environmentCoordinateIds overlap: " + ", ".join(sorted(overlap))
        )
    rationale = value.get("rationale")
    if rationale is not None:
        require_nonempty_string(rationale, "protocol.rationale")
    normalized = {
        "schemaVersion": PROTOCOL_SCHEMA_VERSION,
        "studyId": study_id,
        "claimMode": claim_mode,
        "analysisStartDay": start,
        "windowSnapshots": window,
        "requiredConsecutiveStableWindows": stable_windows,
        "metrics": metrics,
        "runLengthSensitivityEndDays": require_uint_list(
            value.get("runLengthSensitivityEndDays", []), "protocol.runLengthSensitivityEndDays"
        ),
        "analysisStartSensitivityDays": require_uint_list(
            value.get("analysisStartSensitivityDays", []), "protocol.analysisStartSensitivityDays"
        ),
        "initializationCoordinateIds": list(init_ids),
        "environmentCoordinateIds": list(env_ids),
    }
    if end is not None:
        normalized["analysisEndDayInclusive"] = end
    if rationale is not None:
        normalized["rationale"] = rationale
    return normalized


def json_pointer(value: Any, pointer: str) -> Any:
    current = value
    for raw_segment in pointer.split("/")[1:]:
        segment = raw_segment.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and segment in current:
            current = current[segment]
        else:
            raise LongRunDiagnosticError(f"metric source pointer does not resolve: {pointer}")
    return current


def relative_difference_permille(left_num: int, left_den: int, right_num: int, right_den: int) -> int:
    left_scaled = left_num * right_den
    right_scaled = right_num * left_den
    difference = abs(left_scaled - right_scaled)
    denominator = max(left_scaled, right_scaled)
    if denominator == 0:
        return 0
    return min(1000, (difference * 1000 + denominator - 1) // denominator)


def mean_pair(values: list[int]) -> tuple[int, int]:
    return sum(values), len(values)


def within_window_drift_permille(values: list[int], cycle_period: int | None) -> int:
    if len(values) < 2:
        return 0
    if cycle_period is not None:
        cycles = len(values) // cycle_period
        if cycles < 2:
            return 1000
        midpoint_cycles = cycles // 2
        first = values[: midpoint_cycles * cycle_period]
        second = values[midpoint_cycles * cycle_period :]
    else:
        midpoint = len(values) // 2
        first = values[:midpoint]
        second = values[midpoint:]
    if not first or not second:
        return 0
    return relative_difference_permille(*mean_pair(first), *mean_pair(second))


def phase_profile(values: list[int], period: int) -> list[tuple[int, int]]:
    return [mean_pair(values[phase::period]) for phase in range(period)]


def profile_shift_permille(left: list[tuple[int, int]], right: list[tuple[int, int]]) -> int:
    return max(
        relative_difference_permille(left_num, left_den, right_num, right_den)
        for (left_num, left_den), (right_num, right_den) in zip(left, right, strict=True)
    )


def amplitude_permille(profile: list[tuple[int, int]]) -> int:
    if not profile:
        return 0
    common_den = 1
    for _, denominator in profile:
        common_den *= denominator
    values = [numerator * (common_den // denominator) for numerator, denominator in profile]
    maximum = max(values)
    if maximum == 0:
        return 0
    return min(1000, ((maximum - min(values)) * 1000 + maximum - 1) // maximum)


def build_windows(observations: list[dict[str, Any]], window_snapshots: int) -> list[list[dict[str, Any]]]:
    full_count = len(observations) // window_snapshots
    if full_count == 0:
        return []
    used = full_count * window_snapshots
    selected = observations[-used:]
    return [
        selected[index : index + window_snapshots]
        for index in range(0, used, window_snapshots)
    ]


def classify_trajectory(
    observations: list[dict[str, Any]],
    protocol: dict[str, Any],
    analysis_start_day: int,
    analysis_end_day: int | None,
) -> dict[str, Any]:
    filtered = [
        observation
        for observation in observations
        if observation["day"] >= analysis_start_day
        and (analysis_end_day is None or observation["day"] <= analysis_end_day)
    ]
    windows = build_windows(filtered, protocol["windowSnapshots"])
    required_comparisons = protocol["requiredConsecutiveStableWindows"]
    if len(windows) < required_comparisons + 1:
        return {
            "status": "insufficient_data",
            "analysisStartDay": analysis_start_day,
            "analysisEndDayInclusive": analysis_end_day,
            "snapshotCount": len(filtered),
            "fullWindowCount": len(windows),
            "requiredFullWindowCount": required_comparisons + 1,
            "metrics": [],
            "regimeSignature": None,
        }

    metric_results: list[dict[str, Any]] = []
    regime_parts: list[str] = []
    any_cycle = False
    all_stable = True
    for metric in protocol["metrics"]:
        metric_id = metric["id"]
        window_values = [[observation["values"][metric_id] for observation in window] for window in windows]
        cycle_period = metric.get("cyclePeriodSnapshots")
        comparisons: list[dict[str, Any]] = []
        for previous_index in range(len(windows) - 1):
            previous_values = window_values[previous_index]
            current_values = window_values[previous_index + 1]
            if cycle_period is None:
                shift = relative_difference_permille(
                    *mean_pair(previous_values), *mean_pair(current_values)
                )
            else:
                shift = profile_shift_permille(
                    phase_profile(previous_values, cycle_period),
                    phase_profile(current_values, cycle_period),
                )
            current_drift = within_window_drift_permille(current_values, cycle_period)
            within = (
                shift <= metric["maxAdjacentWindowMeanShiftPermille"]
                and current_drift <= metric["maxWithinWindowDriftPermille"]
            )
            comparisons.append(
                {
                    "previousWindowIndex": previous_index,
                    "currentWindowIndex": previous_index + 1,
                    "adjacentWindowShiftPermille": shift,
                    "currentWindowDriftPermille": current_drift,
                    "withinTolerance": within,
                }
            )
        trailing = comparisons[-required_comparisons:]
        stable = len(trailing) == required_comparisons and all(item["withinTolerance"] for item in trailing)
        all_stable &= stable
        terminal_values = window_values[-1]
        terminal_mean_num, terminal_mean_den = mean_pair(terminal_values)
        terminal_mean_floor = terminal_mean_num // terminal_mean_den
        bin_width = metric["regimeBinWidth"]
        cycle_amplitude = 0
        if cycle_period is not None:
            profile = phase_profile(terminal_values, cycle_period)
            cycle_amplitude = amplitude_permille(profile)
            profile_bins = [
                (numerator // denominator) // bin_width for numerator, denominator in profile
            ]
            regime_parts.append(f"{metric_id}=cycle:" + ",".join(map(str, profile_bins)))
            any_cycle |= stable and cycle_amplitude > metric["maxWithinWindowDriftPermille"]
        else:
            regime_parts.append(f"{metric_id}=bin:{terminal_mean_floor // bin_width}")
        metric_results.append(
            {
                "metricId": metric_id,
                "stable": stable,
                "terminalWindowMeanFloor": terminal_mean_floor,
                "terminalCycleAmplitudePermille": cycle_amplitude,
                "comparisons": comparisons,
            }
        )

    status = "cyclic_stable" if all_stable and any_cycle else "stable" if all_stable else "drifting"
    return {
        "status": status,
        "analysisStartDay": analysis_start_day,
        "analysisEndDayInclusive": analysis_end_day,
        "snapshotCount": len(filtered),
        "fullWindowCount": len(windows),
        "requiredFullWindowCount": required_comparisons + 1,
        "metrics": metric_results,
        "regimeSignature": "|".join(regime_parts) if all_stable else None,
    }


def coordinate_label(coordinates: list[dict[str, Any]], ids: list[str], role: str) -> str:
    if not ids:
        return "default"
    by_id = {coordinate.get("id"): coordinate for coordinate in coordinates}
    missing = [coordinate_id for coordinate_id in ids if coordinate_id not in by_id]
    if missing:
        raise LongRunDiagnosticError(
            f"run is missing {role} coordinate(s) declared by protocol: {', '.join(missing)}"
        )
    return "|".join(
        f"{coordinate_id}={json.dumps(by_id[coordinate_id].get('value'), sort_keys=True, separators=(',', ':'))}"
        for coordinate_id in ids
    )


def read_research_runs(root: Path, protocol: dict[str, Any]) -> tuple[dict[str, Any], list[dict[str, Any]], int]:
    manifest = load_json(root / "research-manifest.json", "immutable research manifest")
    plan = load_json(root / "research-plan.json", "immutable research plan")
    if manifest != plan:
        raise LongRunDiagnosticError("research-manifest.json and research-plan.json differ")
    runs_analysis = load_json(root / "analysis/runs.json", "research run analysis")
    if runs_analysis.get("researchId") != manifest.get("researchId"):
        raise LongRunDiagnosticError("analysis/runs.json researchId does not match immutable research metadata")
    runs = runs_analysis.get("runs")
    if not isinstance(runs, list):
        raise LongRunDiagnosticError("analysis/runs.json runs must be an array")

    source = manifest.get("source")
    if not isinstance(source, dict):
        raise LongRunDiagnosticError("immutable research manifest source must be an object")
    completed: list[dict[str, Any]] = []
    failed_count = 0
    for index, row in enumerate(runs):
        if not isinstance(row, dict):
            raise LongRunDiagnosticError(f"analysis/runs.json runs[{index}] must be an object")
        state = row.get("state")
        if state != "completed":
            failed_count += 1
            continue
        relative = row.get("relativeDir")
        if not isinstance(relative, str) or not relative:
            raise LongRunDiagnosticError(f"completed run {index} has invalid relativeDir")
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise LongRunDiagnosticError(f"completed run uses unsafe relativeDir: {relative}")
        run_dir = root / relative_path
        run_manifest = load_json(run_dir / "manifest.json", "completed run manifest")
        metrics = load_json(run_dir / "metrics.json", "completed run metrics")
        configuration = row.get("resultingConfiguration")
        if not isinstance(configuration, dict) or not isinstance(configuration.get("experiment"), dict):
            raise LongRunDiagnosticError("completed analysis row lacks resultingConfiguration.experiment")
        if run_manifest.get("experiment") != configuration["experiment"]:
            raise LongRunDiagnosticError(f"completed run {row.get('runId')} differs from planned experiment")
        if run_manifest.get("stateDigest64") != row.get("stateDigest64"):
            raise LongRunDiagnosticError(f"completed run {row.get('runId')} state digest differs from research state")
        if run_manifest.get("modelVersion") != source.get("modelVersion") or run_manifest.get("modelSemanticsId") != source.get("modelSemanticsId") or run_manifest.get("gitCommit") != source.get("gitCommit"):
            raise LongRunDiagnosticError(f"completed run {row.get('runId')} source revision differs from research plan")
        snapshots = metrics.get("snapshots")
        if not isinstance(snapshots, list):
            raise LongRunDiagnosticError(f"completed run {row.get('runId')} metrics snapshots must be an array")
        observations: list[dict[str, Any]] = []
        previous_day = -1
        for snapshot_index, snapshot in enumerate(snapshots):
            if not isinstance(snapshot, dict):
                raise LongRunDiagnosticError("metric snapshot must be an object")
            day = require_uint(snapshot.get("day"), f"metrics.snapshots[{snapshot_index}].day")
            if day <= previous_day:
                raise LongRunDiagnosticError("metric snapshot days must be strictly increasing")
            previous_day = day
            values: dict[str, int] = {}
            for metric in protocol["metrics"]:
                raw = json_pointer(snapshot, metric["sourcePointer"])
                if not isinstance(raw, int) or isinstance(raw, bool) or raw < 0:
                    raise LongRunDiagnosticError(
                        f"metric {metric['id']} source must resolve to a non-negative integer"
                    )
                values[metric["id"]] = raw
            observations.append({"day": day, "values": values})
        coordinates = row.get("coordinates")
        if not isinstance(coordinates, list):
            raise LongRunDiagnosticError("completed analysis row coordinates must be an array")
        completed.append(
            {
                "runId": require_nonempty_string(row.get("runId"), "run.runId"),
                "pointId": require_nonempty_string(row.get("pointId"), "run.pointId"),
                "seed": require_uint(row.get("seed"), "run.seed"),
                "initialization": coordinate_label(
                    coordinates, protocol["initializationCoordinateIds"], "initialization"
                ),
                "environment": coordinate_label(
                    coordinates, protocol["environmentCoordinateIds"], "environment"
                ),
                "observations": observations,
                "terminalDay": require_uint(run_manifest.get("endTime"), "manifest.endTime"),
                "stopReason": run_manifest.get("stopReason"),
            }
        )
    return manifest, completed, failed_count


def counts(values: list[str]) -> dict[str, int]:
    return dict(sorted(Counter(values).items()))


def frequencies_by(runs: list[dict[str, Any]], key: str) -> dict[str, dict[str, int]]:
    grouped: dict[str, list[str]] = defaultdict(list)
    for run in runs:
        signature = run["primary"]["regimeSignature"]
        grouped[run[key]].append(signature if signature is not None else f"status:{run['primary']['status']}")
    return {label: counts(signatures) for label, signatures in sorted(grouped.items())}


def supports_differ(frequencies: dict[str, dict[str, int]]) -> bool:
    supports = {tuple(sorted(values)) for values in frequencies.values()}
    return len(frequencies) > 1 and len(supports) > 1


def assess_runs(protocol: dict[str, Any], source_runs: list[dict[str, Any]], failed_count: int) -> dict[str, Any]:
    run_results: list[dict[str, Any]] = []
    primary_end = protocol.get("analysisEndDayInclusive")
    for source in source_runs:
        end = source["terminalDay"] if primary_end is None else min(primary_end, source["terminalDay"])
        primary = classify_trajectory(source["observations"], protocol, protocol["analysisStartDay"], end)
        run_length = []
        for requested_end in protocol["runLengthSensitivityEndDays"]:
            if requested_end > source["terminalDay"]:
                run_length.append(
                    {
                        "requestedEndDayInclusive": requested_end,
                        "available": False,
                        "assessment": None,
                    }
                )
            else:
                run_length.append(
                    {
                        "requestedEndDayInclusive": requested_end,
                        "available": True,
                        "assessment": classify_trajectory(
                            source["observations"],
                            protocol,
                            protocol["analysisStartDay"],
                            requested_end,
                        ),
                    }
                )
        start_sensitivity = []
        for sensitivity_start in protocol["analysisStartSensitivityDays"]:
            start_sensitivity.append(
                {
                    "analysisStartDay": sensitivity_start,
                    "assessment": classify_trajectory(
                        source["observations"], protocol, sensitivity_start, end
                    ),
                }
            )
        run_results.append(
            {
                "runId": source["runId"],
                "pointId": source["pointId"],
                "seed": source["seed"],
                "initialization": source["initialization"],
                "environment": source["environment"],
                "terminalDay": source["terminalDay"],
                "stopReason": source["stopReason"],
                "primary": primary,
                "runLengthSensitivity": run_length,
                "analysisStartSensitivity": start_sensitivity,
            }
        )

    primary_status_counts = counts([run["primary"]["status"] for run in run_results])
    stable_runs = [
        run for run in run_results if run["primary"]["status"] in {"stable", "cyclic_stable"}
    ]
    regime_counts = counts(
        [run["primary"]["regimeSignature"] for run in stable_runs if run["primary"]["regimeSignature"]]
    )
    multiple_regimes = len(regime_counts) > 1
    initialization_frequencies = frequencies_by(run_results, "initialization")
    environment_frequencies = frequencies_by(run_results, "environment")

    context_groups: dict[str, list[str]] = defaultdict(list)
    for run in run_results:
        signature = run["primary"]["regimeSignature"]
        if signature is not None:
            context_groups[f"{run['initialization']}||{run['environment']}"] .append(signature)
    stochastic_multiregime_contexts = {
        context: counts(signatures)
        for context, signatures in sorted(context_groups.items())
        if len(set(signatures)) > 1
    }

    run_length_changed = False
    analysis_start_changed = False
    for run in run_results:
        primary_key = (run["primary"]["status"], run["primary"]["regimeSignature"])
        for item in run["runLengthSensitivity"]:
            assessment = item["assessment"]
            if assessment is not None and (assessment["status"], assessment["regimeSignature"]) != primary_key:
                run_length_changed = True
        for item in run["analysisStartSensitivity"]:
            assessment = item["assessment"]
            if (assessment["status"], assessment["regimeSignature"]) != primary_key:
                analysis_start_changed = True

    all_primary_stable = bool(run_results) and len(stable_runs) == len(run_results)
    sensitivity_robust = not run_length_changed and not analysis_start_changed
    equilibrium_supported = (
        protocol["claimMode"] == "equilibrium_like"
        and failed_count == 0
        and all_primary_stable
        and sensitivity_robust
    )
    if protocol["claimMode"] == "explicitly_transient":
        gate_status = "not_required"
    else:
        gate_status = "passed" if equilibrium_supported else "failed"

    return {
        "completedRunCount": len(run_results),
        "nonCompletedRunCount": failed_count,
        "primaryClassificationCounts": primary_status_counts,
        "stableRegimeFrequencies": regime_counts,
        "multipleStableRegimesDetected": multiple_regimes,
        "initializationRegimeFrequencies": initialization_frequencies,
        "environmentRegimeFrequencies": environment_frequencies,
        "initializationDependenceDetected": supports_differ(initialization_frequencies),
        "environmentDependenceDetected": supports_differ(environment_frequencies),
        "stochasticMultiRegimeContexts": stochastic_multiregime_contexts,
        "runLengthSensitivityDetected": run_length_changed,
        "analysisWindowSensitivityDetected": analysis_start_changed,
        "equilibriumLikeClaimSupported": equilibrium_supported,
        "singleRegimePooledLongRunAverageSupported": equilibrium_supported and not multiple_regimes,
        "researchGateStatus": gate_status,
        "runs": run_results,
    }


def derive_assessment(root: Path, raw_protocol: dict[str, Any]) -> dict[str, Any]:
    protocol = validate_protocol(raw_protocol)
    identity = protocol_identity(protocol)
    manifest, source_runs, failed_count = read_research_runs(root, protocol)
    result = assess_runs(protocol, source_runs, failed_count)
    return {
        "schemaVersion": OUTPUT_SCHEMA_VERSION,
        "manifestType": MANIFEST_TYPE,
        "protocolIdentity": identity,
        "studyId": protocol["studyId"],
        "claimMode": protocol["claimMode"],
        "researchId": manifest.get("researchId"),
        "definitionIdentity": manifest.get("definitionIdentity"),
        "source": manifest.get("source"),
        "protocol": protocol,
        **result,
    }


def atomic_write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink():
        raise LongRunDiagnosticError(f"output must not be a symbolic link: {path}")
    data = json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def main() -> int:
    args = parse_args()
    try:
        raw_protocol = load_json(args.protocol, "long-run diagnostic protocol")
        assessment = derive_assessment(args.research_root, raw_protocol)
        output = args.output
        if output is None:
            output = (
                args.research_root
                / "analysis"
                / "studies"
                / assessment["protocolIdentity"]
                / "long-run-diagnostics.json"
            )
        atomic_write_json(output, assessment)
        print(f"wrote {output}")
        return 0
    except LongRunDiagnosticError as error:
        print(f"research-long-run-diagnostics: {error}", file=os.sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
