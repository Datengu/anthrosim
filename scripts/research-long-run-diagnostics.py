#!/usr/bin/env python3
"""Diagnose stable, cyclic, drifting and path-dependent AnthroSim trajectories."""

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
STABLE_STATUSES = {"stable", "cyclic_stable"}


class LongRunDiagnosticError(Exception):
    """Raised when a long-run assessment cannot be derived safely."""


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
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except LongRunDiagnosticError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise LongRunDiagnosticError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise LongRunDiagnosticError(f"{role} root must be a JSON object")
    return value


def require_exact_keys(
    value: dict[str, Any], required: set[str], optional: set[str], role: str
) -> None:
    missing = required - set(value)
    unknown = set(value) - required - optional
    if missing:
        raise LongRunDiagnosticError(
            f"{role} missing field(s): {', '.join(sorted(missing))}"
        )
    if unknown:
        raise LongRunDiagnosticError(
            f"{role} contains unknown field(s): {', '.join(sorted(unknown))}"
        )


def require_nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise LongRunDiagnosticError(f"{role} must be a non-empty string")
    return value


def require_uint(value: Any, role: str, *, positive: bool = False) -> int:
    minimum = 1 if positive else 0
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
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
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


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
        raise LongRunDiagnosticError(
            f"{role}.sourcePointer must be an RFC 6901-style absolute pointer"
        )
    mean_shift = require_uint(
        value["maxAdjacentWindowMeanShiftPermille"],
        f"{role}.maxAdjacentWindowMeanShiftPermille",
    )
    drift = require_uint(
        value["maxWithinWindowDriftPermille"],
        f"{role}.maxWithinWindowDriftPermille",
    )
    if mean_shift > 1_000 or drift > 1_000:
        raise LongRunDiagnosticError(f"{role} permille tolerances must be <= 1000")
    bin_width = require_uint(value["regimeBinWidth"], f"{role}.regimeBinWidth", positive=True)
    cycle_period = value.get("cyclePeriodSnapshots")
    if cycle_period is not None:
        cycle_period = require_uint(
            cycle_period, f"{role}.cyclePeriodSnapshots", positive=True
        )
        if cycle_period < 2:
            raise LongRunDiagnosticError(
                f"{role}.cyclePeriodSnapshots must be >= 2 when declared"
            )
    return {
        "id": metric_id,
        "sourcePointer": pointer,
        "maxAdjacentWindowMeanShiftPermille": mean_shift,
        "maxWithinWindowDriftPermille": drift,
        "regimeBinWidth": bin_width,
        **({"cyclePeriodSnapshots": cycle_period} if cycle_period is not None else {}),
    }


def validate_coordinate_ids(value: Any, role: str) -> list[str]:
    if not isinstance(value, list):
        raise LongRunDiagnosticError(f"protocol.{role} must be an array")
    result = [
        require_nonempty_string(item, f"protocol.{role}[{index}]")
        for index, item in enumerate(value)
    ]
    if len(set(result)) != len(result):
        raise LongRunDiagnosticError(f"protocol.{role} must not contain duplicates")
    return result


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
            "analysisEndSensitivityDays",
            "initializationCoordinateIds",
            "environmentCoordinateIds",
            "rationale",
        },
        "protocol",
    )
    if value["schemaVersion"] != PROTOCOL_SCHEMA_VERSION:
        raise LongRunDiagnosticError(
            f"protocol schemaVersion must be {PROTOCOL_SCHEMA_VERSION}, "
            f"found {value['schemaVersion']}"
        )
    study_id = require_nonempty_string(value["studyId"], "protocol.studyId")
    claim_mode = require_nonempty_string(value["claimMode"], "protocol.claimMode")
    if claim_mode not in CLAIM_MODES:
        raise LongRunDiagnosticError(
            f"protocol.claimMode must be one of {sorted(CLAIM_MODES)}"
        )
    start = require_uint(value["analysisStartDay"], "protocol.analysisStartDay")
    end = value.get("analysisEndDayInclusive")
    if end is not None:
        end = require_uint(end, "protocol.analysisEndDayInclusive")
        if end < start:
            raise LongRunDiagnosticError(
                "analysisEndDayInclusive must be >= analysisStartDay"
            )
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
    metric_ids = [metric["id"] for metric in metrics]
    if len(set(metric_ids)) != len(metric_ids):
        raise LongRunDiagnosticError("protocol.metrics contains duplicate ids")
    for metric in metrics:
        period = metric.get("cyclePeriodSnapshots")
        if period is not None and window % period != 0:
            raise LongRunDiagnosticError(
                "windowSnapshots must be divisible by cyclePeriodSnapshots for metric "
                f"{metric['id']}"
            )

    initialization_ids = validate_coordinate_ids(
        value.get("initializationCoordinateIds", []), "initializationCoordinateIds"
    )
    environment_ids = validate_coordinate_ids(
        value.get("environmentCoordinateIds", []), "environmentCoordinateIds"
    )
    overlap = set(initialization_ids) & set(environment_ids)
    if overlap:
        raise LongRunDiagnosticError(
            "initializationCoordinateIds and environmentCoordinateIds overlap: "
            + ", ".join(sorted(overlap))
        )
    rationale = value.get("rationale")
    if rationale is not None:
        require_nonempty_string(rationale, "protocol.rationale")

    normalized: dict[str, Any] = {
        "schemaVersion": PROTOCOL_SCHEMA_VERSION,
        "studyId": study_id,
        "claimMode": claim_mode,
        "analysisStartDay": start,
        "windowSnapshots": window,
        "requiredConsecutiveStableWindows": stable_windows,
        "metrics": metrics,
        "runLengthSensitivityEndDays": require_uint_list(
            value.get("runLengthSensitivityEndDays", []),
            "protocol.runLengthSensitivityEndDays",
        ),
        "analysisStartSensitivityDays": require_uint_list(
            value.get("analysisStartSensitivityDays", []),
            "protocol.analysisStartSensitivityDays",
        ),
        "analysisEndSensitivityDays": require_uint_list(
            value.get("analysisEndSensitivityDays", []),
            "protocol.analysisEndSensitivityDays",
        ),
        "initializationCoordinateIds": initialization_ids,
        "environmentCoordinateIds": environment_ids,
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
            raise LongRunDiagnosticError(
                f"metric source pointer does not resolve: {pointer}"
            )
    return current


def relative_difference_permille(
    left_num: int, left_den: int, right_num: int, right_den: int
) -> int:
    left_scaled = left_num * right_den
    right_scaled = right_num * left_den
    difference = abs(left_scaled - right_scaled)
    denominator = max(left_scaled, right_scaled)
    if denominator == 0:
        return 0
    return min(1_000, (difference * 1_000 + denominator - 1) // denominator)


def mean_pair(values: list[int]) -> tuple[int, int]:
    return sum(values), len(values)


def phase_profile(values: list[int], period: int) -> list[tuple[int, int]]:
    return [mean_pair(values[phase::period]) for phase in range(period)]


def profile_shift_permille(
    left: list[tuple[int, int]], right: list[tuple[int, int]]
) -> int:
    return max(
        relative_difference_permille(left_num, left_den, right_num, right_den)
        for (left_num, left_den), (right_num, right_den) in zip(
            left, right, strict=True
        )
    )


def within_window_drift_permille(
    values: list[int], cycle_period: int | None
) -> int:
    if len(values) < 2:
        return 0
    if cycle_period is None:
        midpoint = len(values) // 2
        first = values[:midpoint]
        second = values[midpoint:]
        if not first or not second:
            return 0
        return relative_difference_permille(*mean_pair(first), *mean_pair(second))

    cycles = len(values) // cycle_period
    if cycles < 2:
        return 1_000
    midpoint_cycles = cycles // 2
    first = values[: midpoint_cycles * cycle_period]
    second = values[midpoint_cycles * cycle_period :]
    if not first or not second:
        return 1_000
    return profile_shift_permille(
        phase_profile(first, cycle_period), phase_profile(second, cycle_period)
    )


def amplitude_permille(profile: list[tuple[int, int]]) -> int:
    if not profile:
        return 0
    common_denominator = 1
    for _, denominator in profile:
        common_denominator *= denominator
    values = [
        numerator * (common_denominator // denominator)
        for numerator, denominator in profile
    ]
    maximum = max(values)
    if maximum == 0:
        return 0
    return min(
        1_000,
        ((maximum - min(values)) * 1_000 + maximum - 1) // maximum,
    )


def canonical_cycle_bins(values: list[int]) -> list[int]:
    if len(values) < 2:
        return values
    rotations = [values[index:] + values[:index] for index in range(len(values))]
    return min(rotations)


def build_windows(
    observations: list[dict[str, Any]], window_snapshots: int
) -> list[list[dict[str, Any]]]:
    full_count = len(observations) // window_snapshots
    if full_count == 0:
        return []
    used = full_count * window_snapshots
    selected = observations[-used:]
    return [
        selected[index : index + window_snapshots]
        for index in range(0, used, window_snapshots)
    ]


def insufficient_assessment(
    analysis_start_day: int,
    analysis_end_day: int | None,
    snapshot_count: int,
    full_window_count: int,
    required_full_window_count: int,
    reason: str,
) -> dict[str, Any]:
    return {
        "status": "insufficient_data",
        "reason": reason,
        "analysisStartDay": analysis_start_day,
        "analysisEndDayInclusive": analysis_end_day,
        "snapshotCount": snapshot_count,
        "fullWindowCount": full_window_count,
        "requiredFullWindowCount": required_full_window_count,
        "metrics": [],
        "regimeSignature": None,
    }


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
    required_windows = required_comparisons + 1
    if len(windows) < required_windows:
        return insufficient_assessment(
            analysis_start_day,
            analysis_end_day,
            len(filtered),
            len(windows),
            required_windows,
            "too_few_complete_windows",
        )

    metric_results: list[dict[str, Any]] = []
    regime_parts: list[str] = []
    any_cycle = False
    all_stable = True
    for metric in protocol["metrics"]:
        metric_id = metric["id"]
        window_values = [
            [observation["values"][metric_id] for observation in window]
            for window in windows
        ]
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
            current_drift = within_window_drift_permille(
                current_values, cycle_period
            )
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
        stable = len(trailing) == required_comparisons and all(
            comparison["withinTolerance"] for comparison in trailing
        )
        all_stable &= stable
        terminal_values = window_values[-1]
        terminal_mean_num, terminal_mean_den = mean_pair(terminal_values)
        terminal_mean_floor = terminal_mean_num // terminal_mean_den
        bin_width = metric["regimeBinWidth"]
        cycle_amplitude = 0
        if cycle_period is not None:
            profile = phase_profile(terminal_values, cycle_period)
            cycle_amplitude = amplitude_permille(profile)
            profile_bins = canonical_cycle_bins(
                [
                    (numerator // denominator) // bin_width
                    for numerator, denominator in profile
                ]
            )
            regime_parts.append(
                f"{metric_id}=cycle:" + ",".join(map(str, profile_bins))
            )
            any_cycle |= (
                stable
                and cycle_amplitude > metric["maxWithinWindowDriftPermille"]
            )
        else:
            regime_parts.append(
                f"{metric_id}=bin:{terminal_mean_floor // bin_width}"
            )
        metric_results.append(
            {
                "metricId": metric_id,
                "stable": stable,
                "terminalWindowMeanFloor": terminal_mean_floor,
                "terminalCycleAmplitudePermille": cycle_amplitude,
                "comparisons": comparisons,
            }
        )

    status = (
        "cyclic_stable"
        if all_stable and any_cycle
        else "stable"
        if all_stable
        else "drifting"
    )
    return {
        "status": status,
        "analysisStartDay": analysis_start_day,
        "analysisEndDayInclusive": analysis_end_day,
        "snapshotCount": len(filtered),
        "fullWindowCount": len(windows),
        "requiredFullWindowCount": required_windows,
        "metrics": metric_results,
        "regimeSignature": "|".join(regime_parts) if all_stable else None,
    }


def coordinate_map(coordinates: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    output: dict[str, dict[str, Any]] = {}
    for index, coordinate in enumerate(coordinates):
        if not isinstance(coordinate, dict):
            raise LongRunDiagnosticError(
                f"run coordinate[{index}] must be an object"
            )
        coordinate_id = require_nonempty_string(
            coordinate.get("id"), f"run coordinate[{index}].id"
        )
        if coordinate_id in output:
            raise LongRunDiagnosticError(
                f"run contains duplicate coordinate id: {coordinate_id}"
            )
        output[coordinate_id] = coordinate
    return output


def render_coordinate_value(coordinate: dict[str, Any]) -> str:
    return json.dumps(
        coordinate.get("value"), sort_keys=True, separators=(",", ":")
    )


def selected_coordinate_label(
    by_id: dict[str, dict[str, Any]], ids: list[str], role: str
) -> str:
    if not ids:
        return "default"
    missing = [coordinate_id for coordinate_id in ids if coordinate_id not in by_id]
    if missing:
        raise LongRunDiagnosticError(
            f"run is missing {role} coordinate(s) declared by protocol: "
            + ", ".join(missing)
        )
    return "|".join(
        f"{coordinate_id}={render_coordinate_value(by_id[coordinate_id])}"
        for coordinate_id in ids
    )


def treatment_coordinate_label(
    by_id: dict[str, dict[str, Any]], excluded_ids: set[str]
) -> str:
    remaining = sorted(set(by_id) - excluded_ids)
    if not remaining:
        return "default"
    return "|".join(
        f"{coordinate_id}={render_coordinate_value(by_id[coordinate_id])}"
        for coordinate_id in remaining
    )


def planned_runs(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    points = manifest.get("points")
    if not isinstance(points, list):
        raise LongRunDiagnosticError("immutable research manifest points must be an array")
    output: dict[str, dict[str, Any]] = {}
    for point_index, planned_point in enumerate(points):
        if not isinstance(planned_point, dict):
            raise LongRunDiagnosticError(
                f"immutable research point[{point_index}] must be an object"
            )
        point = planned_point.get("point")
        runs = planned_point.get("runs")
        if not isinstance(point, dict) or not isinstance(runs, list):
            raise LongRunDiagnosticError(
                f"immutable research point[{point_index}] is malformed"
            )
        point_id = require_nonempty_string(
            point.get("pointId"), f"immutable research point[{point_index}].pointId"
        )
        coordinates = point.get("coordinates")
        if not isinstance(coordinates, list):
            raise LongRunDiagnosticError(
                f"immutable research point[{point_index}].coordinates must be an array"
            )
        for run_index, planned in enumerate(runs):
            if not isinstance(planned, dict):
                raise LongRunDiagnosticError(
                    f"immutable planned run[{run_index}] must be an object"
                )
            run_id = require_nonempty_string(
                planned.get("runId"), "immutable planned run.runId"
            )
            if run_id in output:
                raise LongRunDiagnosticError(
                    f"immutable research manifest contains duplicate runId: {run_id}"
                )
            output[run_id] = {
                "pointId": point_id,
                "coordinates": coordinates,
                "seed": require_uint(planned.get("seed"), "immutable planned run.seed"),
                "relativeDir": planned.get("relativeDir"),
                "runConfig": planned.get("runConfig"),
            }
    return output


def read_research_runs(
    root: Path, protocol: dict[str, Any]
) -> tuple[dict[str, Any], list[dict[str, Any]], int]:
    manifest = load_json(root / "research-manifest.json", "immutable research manifest")
    plan = load_json(root / "research-plan.json", "immutable research plan")
    if manifest != plan:
        raise LongRunDiagnosticError(
            "research-manifest.json and research-plan.json differ"
        )
    runs_analysis = load_json(root / "analysis/runs.json", "research run analysis")
    if runs_analysis.get("researchId") != manifest.get("researchId"):
        raise LongRunDiagnosticError(
            "analysis/runs.json researchId does not match immutable research metadata"
        )
    rows = runs_analysis.get("runs")
    if not isinstance(rows, list):
        raise LongRunDiagnosticError("analysis/runs.json runs must be an array")

    planned = planned_runs(manifest)
    rows_by_id: dict[str, dict[str, Any]] = {}
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            raise LongRunDiagnosticError(
                f"analysis/runs.json runs[{index}] must be an object"
            )
        run_id = require_nonempty_string(row.get("runId"), f"runs[{index}].runId")
        if run_id in rows_by_id:
            raise LongRunDiagnosticError(
                f"analysis/runs.json contains duplicate runId: {run_id}"
            )
        rows_by_id[run_id] = row
    if set(rows_by_id) != set(planned):
        missing = sorted(set(planned) - set(rows_by_id))
        extra = sorted(set(rows_by_id) - set(planned))
        raise LongRunDiagnosticError(
            "analysis/runs.json run set differs from immutable research plan; "
            f"missing={missing}, extra={extra}"
        )

    source = manifest.get("source")
    if not isinstance(source, dict):
        raise LongRunDiagnosticError(
            "immutable research manifest source must be an object"
        )
    completed: list[dict[str, Any]] = []
    non_completed_count = 0
    excluded_ids = set(protocol["initializationCoordinateIds"]) | set(
        protocol["environmentCoordinateIds"]
    )

    for run_id, immutable in planned.items():
        row = rows_by_id[run_id]
        if (
            row.get("pointId") != immutable["pointId"]
            or row.get("seed") != immutable["seed"]
            or row.get("relativeDir") != immutable["relativeDir"]
            or row.get("resultingConfiguration") != immutable["runConfig"]
            or row.get("coordinates") != immutable["coordinates"]
        ):
            raise LongRunDiagnosticError(
                f"analysis row {run_id} differs from immutable planned run identity/configuration"
            )
        if row.get("state") != "completed":
            non_completed_count += 1
            continue

        relative = immutable["relativeDir"]
        if not isinstance(relative, str) or not relative:
            raise LongRunDiagnosticError(
                f"completed run {run_id} has invalid relativeDir"
            )
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise LongRunDiagnosticError(
                f"completed run uses unsafe relativeDir: {relative}"
            )
        run_dir = root / relative_path
        run_manifest = load_json(run_dir / "manifest.json", "completed run manifest")
        metrics = load_json(run_dir / "metrics.json", "completed run metrics")
        run_config = immutable["runConfig"]
        if not isinstance(run_config, dict) or not isinstance(
            run_config.get("experiment"), dict
        ):
            raise LongRunDiagnosticError(
                f"immutable planned run {run_id} lacks runConfig.experiment"
            )
        if run_manifest.get("experiment") != run_config["experiment"]:
            raise LongRunDiagnosticError(
                f"completed run {run_id} differs from planned experiment"
            )
        if run_manifest.get("stateDigest64") != row.get("stateDigest64"):
            raise LongRunDiagnosticError(
                f"completed run {run_id} state digest differs from research state"
            )
        if (
            run_manifest.get("modelVersion") != source.get("modelVersion")
            or run_manifest.get("modelSemanticsId") != source.get("modelSemanticsId")
            or run_manifest.get("gitCommit") != source.get("gitCommit")
        ):
            raise LongRunDiagnosticError(
                f"completed run {run_id} source revision differs from research plan"
            )
        snapshots = metrics.get("snapshots")
        if not isinstance(snapshots, list):
            raise LongRunDiagnosticError(
                f"completed run {run_id} metrics snapshots must be an array"
            )
        observations: list[dict[str, Any]] = []
        previous_day = -1
        for snapshot_index, snapshot in enumerate(snapshots):
            if not isinstance(snapshot, dict):
                raise LongRunDiagnosticError("metric snapshot must be an object")
            day = require_uint(
                snapshot.get("day"), f"metrics.snapshots[{snapshot_index}].day"
            )
            if day <= previous_day:
                raise LongRunDiagnosticError(
                    "metric snapshot days must be strictly increasing"
                )
            previous_day = day
            if day % DAYS_PER_YEAR != 0:
                # MetricSeries is annual-boundary-plus-terminal. A subannual early-stop terminal
                # point is valid run provenance but not a regular annual stationarity observation.
                continue
            values: dict[str, int] = {}
            for metric in protocol["metrics"]:
                raw = json_pointer(snapshot, metric["sourcePointer"])
                if not isinstance(raw, int) or isinstance(raw, bool) or raw < 0:
                    raise LongRunDiagnosticError(
                        f"metric {metric['id']} source must resolve to a non-negative integer"
                    )
                values[metric["id"]] = raw
            observations.append({"day": day, "values": values})

        coordinates = immutable["coordinates"]
        by_id = coordinate_map(coordinates)
        terminal_day = require_uint(run_manifest.get("endTime"), "manifest.endTime")
        completed.append(
            {
                "runId": run_id,
                "pointId": immutable["pointId"],
                "seed": immutable["seed"],
                "initialization": selected_coordinate_label(
                    by_id,
                    protocol["initializationCoordinateIds"],
                    "initialization",
                ),
                "environment": selected_coordinate_label(
                    by_id,
                    protocol["environmentCoordinateIds"],
                    "environment",
                ),
                "treatmentContext": treatment_coordinate_label(by_id, excluded_ids),
                "observations": observations,
                "terminalDay": terminal_day,
                "stopReason": run_manifest.get("stopReason"),
            }
        )
    return manifest, completed, non_completed_count


def counts(values: list[str]) -> dict[str, int]:
    return dict(sorted(Counter(values).items()))


def run_outcome_label(run: dict[str, Any]) -> str:
    signature = run["primary"]["regimeSignature"]
    return signature if signature is not None else f"status:{run['primary']['status']}"


def normalized_distributions_differ(groups: dict[str, dict[str, int]]) -> bool:
    if len(groups) < 2:
        return False
    labels = sorted(groups)
    first = groups[labels[0]]
    first_total = sum(first.values())
    for label in labels[1:]:
        current = groups[label]
        current_total = sum(current.values())
        for outcome in set(first) | set(current):
            if first.get(outcome, 0) * current_total != current.get(outcome, 0) * first_total:
                return True
    return False


def grouped_frequencies(
    runs: list[dict[str, Any]], grouping_key: str
) -> dict[str, dict[str, dict[str, int]]]:
    grouped: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    for run in runs:
        grouped[run["treatmentContext"]][run[grouping_key]].append(
            run_outcome_label(run)
        )
    return {
        treatment: {
            group: counts(outcomes) for group, outcomes in sorted(groups.items())
        }
        for treatment, groups in sorted(grouped.items())
    }


def dependence_detected(
    grouped: dict[str, dict[str, dict[str, int]]]
) -> bool:
    return any(normalized_distributions_differ(groups) for groups in grouped.values())


def stable_regimes_by_treatment(
    runs: list[dict[str, Any]]
) -> dict[str, dict[str, int]]:
    grouped: dict[str, list[str]] = defaultdict(list)
    for run in runs:
        if run["primary"]["status"] in STABLE_STATUSES:
            signature = run["primary"]["regimeSignature"]
            if signature is not None:
                grouped[run["treatmentContext"]].append(signature)
    return {
        treatment: counts(signatures)
        for treatment, signatures in sorted(grouped.items())
    }


def assess_runs(
    protocol: dict[str, Any], source_runs: list[dict[str, Any]], non_completed_count: int
) -> dict[str, Any]:
    run_results: list[dict[str, Any]] = []
    explicit_primary_end = protocol.get("analysisEndDayInclusive")
    unavailable_sensitivity_count = 0

    for source in source_runs:
        terminal_day = source["terminalDay"]
        if explicit_primary_end is not None and explicit_primary_end > terminal_day:
            primary = insufficient_assessment(
                protocol["analysisStartDay"],
                explicit_primary_end,
                len(source["observations"]),
                0,
                protocol["requiredConsecutiveStableWindows"] + 1,
                "declared_analysis_end_not_observed",
            )
            primary_end = explicit_primary_end
        else:
            primary_end = terminal_day if explicit_primary_end is None else explicit_primary_end
            primary = classify_trajectory(
                source["observations"],
                protocol,
                protocol["analysisStartDay"],
                primary_end,
            )

        run_length = []
        for requested_end in protocol["runLengthSensitivityEndDays"]:
            available = requested_end <= terminal_day
            assessment = (
                classify_trajectory(
                    source["observations"],
                    protocol,
                    protocol["analysisStartDay"],
                    requested_end,
                )
                if available
                else None
            )
            if not available:
                unavailable_sensitivity_count += 1
            run_length.append(
                {
                    "requestedEndDayInclusive": requested_end,
                    "available": available,
                    "assessment": assessment,
                }
            )

        start_sensitivity = [
            {
                "analysisStartDay": sensitivity_start,
                "assessment": classify_trajectory(
                    source["observations"], protocol, sensitivity_start, primary_end
                ),
            }
            for sensitivity_start in protocol["analysisStartSensitivityDays"]
        ]

        end_sensitivity = []
        for sensitivity_end in protocol["analysisEndSensitivityDays"]:
            available = sensitivity_end <= terminal_day
            assessment = (
                classify_trajectory(
                    source["observations"],
                    protocol,
                    protocol["analysisStartDay"],
                    sensitivity_end,
                )
                if available
                else None
            )
            if not available:
                unavailable_sensitivity_count += 1
            end_sensitivity.append(
                {
                    "analysisEndDayInclusive": sensitivity_end,
                    "available": available,
                    "assessment": assessment,
                }
            )

        run_results.append(
            {
                "runId": source["runId"],
                "pointId": source["pointId"],
                "seed": source["seed"],
                "initialization": source["initialization"],
                "environment": source["environment"],
                "treatmentContext": source["treatmentContext"],
                "terminalDay": terminal_day,
                "stopReason": source["stopReason"],
                "primary": primary,
                "runLengthSensitivity": run_length,
                "analysisStartSensitivity": start_sensitivity,
                "analysisEndSensitivity": end_sensitivity,
            }
        )

    primary_status_counts = counts(
        [run["primary"]["status"] for run in run_results]
    )
    stable_runs = [
        run for run in run_results if run["primary"]["status"] in STABLE_STATUSES
    ]
    regimes_by_treatment = stable_regimes_by_treatment(run_results)
    multiple_regime_treatments = {
        treatment: frequencies
        for treatment, frequencies in regimes_by_treatment.items()
        if len(frequencies) > 1
    }
    multiple_regimes = bool(multiple_regime_treatments)
    initialization_frequencies = grouped_frequencies(run_results, "initialization")
    environment_frequencies = grouped_frequencies(run_results, "environment")

    seed_contexts: dict[str, list[str]] = defaultdict(list)
    for run in stable_runs:
        signature = run["primary"]["regimeSignature"]
        if signature is not None:
            context = (
                f"treatment={run['treatmentContext']}||"
                f"initialization={run['initialization']}||"
                f"environment={run['environment']}"
            )
            seed_contexts[context].append(signature)
    stochastic_multiregime_contexts = {
        context: counts(signatures)
        for context, signatures in sorted(seed_contexts.items())
        if len(set(signatures)) > 1
    }

    run_length_changed = False
    analysis_start_changed = False
    analysis_end_changed = False
    for run in run_results:
        primary_key = (
            run["primary"]["status"],
            run["primary"]["regimeSignature"],
        )
        for item in run["runLengthSensitivity"]:
            assessment = item["assessment"]
            if assessment is not None and (
                assessment["status"], assessment["regimeSignature"]
            ) != primary_key:
                run_length_changed = True
        for item in run["analysisStartSensitivity"]:
            assessment = item["assessment"]
            if (
                assessment["status"], assessment["regimeSignature"]
            ) != primary_key:
                analysis_start_changed = True
        for item in run["analysisEndSensitivity"]:
            assessment = item["assessment"]
            if assessment is not None and (
                assessment["status"], assessment["regimeSignature"]
            ) != primary_key:
                analysis_end_changed = True

    early_terminated_count = sum(
        run["stopReason"] != "durationReached" for run in run_results
    )
    all_primary_stable = bool(run_results) and len(stable_runs) == len(run_results)
    sensitivity_coverage = {
        "runLengthSensitivityDeclared": bool(protocol["runLengthSensitivityEndDays"]),
        "analysisStartSensitivityDeclared": bool(protocol["analysisStartSensitivityDays"]),
        "analysisEndSensitivityDeclared": bool(protocol["analysisEndSensitivityDays"]),
        "initializationGroupingDeclared": bool(protocol["initializationCoordinateIds"]),
        "environmentGroupingDeclared": bool(protocol["environmentCoordinateIds"]),
    }
    required_equilibrium_sensitivity_coverage = (
        sensitivity_coverage["runLengthSensitivityDeclared"]
        and sensitivity_coverage["analysisStartSensitivityDeclared"]
        and sensitivity_coverage["analysisEndSensitivityDeclared"]
    )
    sensitivity_robust = (
        not run_length_changed
        and not analysis_start_changed
        and not analysis_end_changed
        and unavailable_sensitivity_count == 0
    )
    equilibrium_supported = (
        protocol["claimMode"] == "equilibrium_like"
        and non_completed_count == 0
        and early_terminated_count == 0
        and all_primary_stable
        and required_equilibrium_sensitivity_coverage
        and sensitivity_robust
    )
    gate_status = (
        "not_required"
        if protocol["claimMode"] == "explicitly_transient"
        else "passed"
        if equilibrium_supported
        else "failed"
    )
    treatment_context_count = len(
        {run["treatmentContext"] for run in run_results}
    )

    return {
        "plannedRunCount": len(run_results) + non_completed_count,
        "completedRunCount": len(run_results),
        "nonCompletedRunCount": non_completed_count,
        "earlyTerminatedRunCount": early_terminated_count,
        "primaryClassificationCounts": primary_status_counts,
        "stableRegimeFrequenciesByTreatmentContext": regimes_by_treatment,
        "multipleStableRegimesDetected": multiple_regimes,
        "multipleStableRegimeTreatmentContexts": multiple_regime_treatments,
        "initializationRegimeFrequenciesByTreatmentContext": initialization_frequencies,
        "environmentRegimeFrequenciesByTreatmentContext": environment_frequencies,
        "initializationDependenceDetected": dependence_detected(initialization_frequencies),
        "environmentDependenceDetected": dependence_detected(environment_frequencies),
        "stochasticMultiRegimeContexts": stochastic_multiregime_contexts,
        "runLengthSensitivityDetected": run_length_changed,
        "analysisStartSensitivityDetected": analysis_start_changed,
        "analysisEndSensitivityDetected": analysis_end_changed,
        "unavailableSensitivityAssessmentCount": unavailable_sensitivity_count,
        "sensitivityCoverage": sensitivity_coverage,
        "requiredEquilibriumSensitivityCoverageComplete": required_equilibrium_sensitivity_coverage,
        "equilibriumLikeClaimSupported": equilibrium_supported,
        "singleRegimePooledLongRunAverageSupported": (
            equilibrium_supported
            and treatment_context_count == 1
            and not multiple_regimes
        ),
        "researchGateStatus": gate_status,
        "runs": run_results,
    }


def derive_assessment(root: Path, raw_protocol: dict[str, Any]) -> dict[str, Any]:
    protocol = validate_protocol(raw_protocol)
    identity = protocol_identity(protocol)
    manifest, source_runs, non_completed_count = read_research_runs(root, protocol)
    result = assess_runs(protocol, source_runs, non_completed_count)
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
    file_descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(
            file_descriptor, "w", encoding="utf-8", newline="\n"
        ) as handle:
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
