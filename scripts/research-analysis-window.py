#!/usr/bin/env python3
"""Bind declared analysis/burn-in windows to an immutable AnthroSim research execution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from pathlib import Path
from typing import Any

DAYS_PER_YEAR = 365
PROTOCOL_SCHEMA_VERSION = 1
OUTPUT_SCHEMA_VERSION = 1
MANIFEST_TYPE = "anthrosim-research-analysis-window"
ALLOWED_SELECTION_RULES = {
    "predeclared_fixed_duration",
    "convergence_diagnostic",
    "externally_meaningful_historical_start",
    "initial_state_in_scope",
    "other_explicit",
}


class AnalysisWindowError(Exception):
    """Raised when analysis-window provenance cannot be prepared safely."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Bind a versioned study analysis window to an existing anthrosim-research "
            "execution and derive per-run source intervals without changing simulation state."
        )
    )
    parser.add_argument("research_root", type=Path, help="anthrosim-research output directory")
    parser.add_argument("protocol", type=Path, help="versioned analysis-window protocol JSON")
    return parser.parse_args()


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise AnalysisWindowError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink():
        raise AnalysisWindowError(f"{role} must not be a symbolic link: {path}")
    if not path.is_file():
        raise AnalysisWindowError(f"{role} is missing or is not a regular file: {path}")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except AnalysisWindowError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AnalysisWindowError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise AnalysisWindowError(f"{role} root must be a JSON object")
    return value


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")


def protocol_identity(protocol: dict[str, Any]) -> tuple[str, str]:
    digest = hashlib.sha256(canonical_bytes(protocol)).hexdigest()
    return f"analysis-window-protocol-v1-sha256-{digest}", digest


def require_exact_keys(
    value: dict[str, Any], required: set[str], optional: set[str], role: str
) -> None:
    keys = set(value)
    missing = required - keys
    unknown = keys - required - optional
    if missing:
        raise AnalysisWindowError(
            f"{role} is missing required field(s): {', '.join(sorted(missing))}"
        )
    if unknown:
        raise AnalysisWindowError(
            f"{role} contains unknown field(s): {', '.join(sorted(unknown))}"
        )


def require_nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AnalysisWindowError(f"{role} must be a non-empty string")
    return value


def require_uint(value: Any, role: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise AnalysisWindowError(f"{role} must be a non-negative integer")
    return value


def validate_primary_window(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise AnalysisWindowError("analysisWindow must be an object")
    require_exact_keys(
        value,
        {"analysisStartDay", "selectionRule", "rationale"},
        {"analysisEndDayInclusive"},
        "analysisWindow",
    )
    start = require_uint(value["analysisStartDay"], "analysisWindow.analysisStartDay")
    end = value.get("analysisEndDayInclusive")
    if end is not None:
        end = require_uint(end, "analysisWindow.analysisEndDayInclusive")
        if end < start:
            raise AnalysisWindowError(
                "analysisWindow.analysisEndDayInclusive must be >= analysisStartDay"
            )
    rule = require_nonempty_string(value["selectionRule"], "analysisWindow.selectionRule")
    if rule not in ALLOWED_SELECTION_RULES:
        allowed = ", ".join(sorted(ALLOWED_SELECTION_RULES))
        raise AnalysisWindowError(
            f"unsupported analysisWindow.selectionRule {rule!r}; allowed: {allowed}"
        )
    rationale = require_nonempty_string(value["rationale"], "analysisWindow.rationale")
    result = {
        "analysisStartDay": start,
        "selectionRule": rule,
        "rationale": rationale,
    }
    if end is not None:
        result["analysisEndDayInclusive"] = end
    return result


def validate_sensitivity_window(value: Any, index: int) -> dict[str, Any]:
    role = f"sensitivityWindows[{index}]"
    if not isinstance(value, dict):
        raise AnalysisWindowError(f"{role} must be an object")
    require_exact_keys(
        value,
        {"id", "analysisStartDay", "rationale"},
        {"analysisEndDayInclusive"},
        role,
    )
    window_id = require_nonempty_string(value["id"], f"{role}.id")
    start = require_uint(value["analysisStartDay"], f"{role}.analysisStartDay")
    end = value.get("analysisEndDayInclusive")
    if end is not None:
        end = require_uint(end, f"{role}.analysisEndDayInclusive")
        if end < start:
            raise AnalysisWindowError(
                f"{role}.analysisEndDayInclusive must be >= analysisStartDay"
            )
    rationale = require_nonempty_string(value["rationale"], f"{role}.rationale")
    result = {"id": window_id, "analysisStartDay": start, "rationale": rationale}
    if end is not None:
        result["analysisEndDayInclusive"] = end
    return result


def validate_protocol(raw: dict[str, Any]) -> dict[str, Any]:
    require_exact_keys(
        raw,
        {"schemaVersion", "studyId", "analysisWindow"},
        {"sensitivityWindows"},
        "analysis-window protocol",
    )
    if raw["schemaVersion"] != PROTOCOL_SCHEMA_VERSION:
        raise AnalysisWindowError(
            f"unsupported analysis-window protocol schema {raw['schemaVersion']!r}; "
            f"supported schema is {PROTOCOL_SCHEMA_VERSION}"
        )
    study_id = require_nonempty_string(raw["studyId"], "studyId")
    primary = validate_primary_window(raw["analysisWindow"])
    sensitivity_raw = raw.get("sensitivityWindows", [])
    if not isinstance(sensitivity_raw, list):
        raise AnalysisWindowError("sensitivityWindows must be an array")
    sensitivity = [
        validate_sensitivity_window(value, index)
        for index, value in enumerate(sensitivity_raw)
    ]
    seen_ids: set[str] = set()
    seen_intervals: set[tuple[int, int | None]] = set()
    for window in sensitivity:
        if window["id"] in seen_ids:
            raise AnalysisWindowError(
                f"duplicate sensitivity window id: {window['id']}"
            )
        seen_ids.add(window["id"])
        interval = (
            window["analysisStartDay"],
            window.get("analysisEndDayInclusive"),
        )
        if interval in seen_intervals:
            raise AnalysisWindowError(
                "sensitivityWindows contains a duplicate analysis interval"
            )
        seen_intervals.add(interval)
    return {
        "schemaVersion": PROTOCOL_SCHEMA_VERSION,
        "studyId": study_id,
        "analysisWindow": primary,
        "sensitivityWindows": sensitivity,
    }


def require_research_root(root: Path) -> Path:
    try:
        resolved = root.resolve(strict=True)
    except FileNotFoundError as error:
        raise AnalysisWindowError(f"research root does not exist: {root}") from error
    if not resolved.is_dir():
        raise AnalysisWindowError(f"research root is not a directory: {root}")
    if root.is_symlink():
        raise AnalysisWindowError(f"research root must not be a symbolic link: {root}")
    return resolved


def validate_research_metadata(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    manifest = load_json(root / "research-manifest.json", "immutable research manifest")
    plan = load_json(root / "research-plan.json", "immutable research plan")
    if manifest != plan:
        raise AnalysisWindowError(
            "research-manifest.json and research-plan.json do not contain the same immutable plan"
        )
    if manifest.get("schemaVersion") != 1:
        raise AnalysisWindowError("unsupported research execution manifest schema")
    research_id = require_nonempty_string(manifest.get("researchId"), "researchId")
    require_nonempty_string(manifest.get("definitionIdentity"), "definitionIdentity")
    if not isinstance(manifest.get("source"), dict):
        raise AnalysisWindowError("research manifest source identity must be an object")
    if not isinstance(manifest.get("points"), list):
        raise AnalysisWindowError("research manifest points must be an array")

    state = load_json(root / "research-state.json", "research execution state")
    if state.get("schemaVersion") != 1:
        raise AnalysisWindowError("unsupported research execution state schema")
    if state.get("researchId") != research_id:
        raise AnalysisWindowError(
            "research-state.json belongs to a different immutable research identity"
        )
    if not isinstance(state.get("runs"), dict):
        raise AnalysisWindowError("research-state.json runs must be an object")
    return manifest, state


def planned_runs(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    runs: list[dict[str, Any]] = []
    seen: set[str] = set()
    for point_index, planned_point in enumerate(manifest["points"]):
        if not isinstance(planned_point, dict):
            raise AnalysisWindowError(f"research manifest point {point_index} must be an object")
        point = planned_point.get("point")
        point_runs = planned_point.get("runs")
        if not isinstance(point, dict) or not isinstance(point_runs, list):
            raise AnalysisWindowError(
                f"research manifest point {point_index} has invalid point/run structure"
            )
        point_id = require_nonempty_string(
            point.get("pointId"), f"research manifest point {point_index}.pointId"
        )
        coordinates = point.get("coordinates", [])
        if not isinstance(coordinates, list):
            raise AnalysisWindowError(
                f"research manifest point {point_index}.coordinates must be an array"
            )
        for run_index, run in enumerate(point_runs):
            if not isinstance(run, dict):
                raise AnalysisWindowError(
                    f"research manifest point {point_index} run {run_index} must be an object"
                )
            run_id = require_nonempty_string(
                run.get("runId"),
                f"research manifest point {point_index} run {run_index}.runId",
            )
            if run_id in seen:
                raise AnalysisWindowError(f"research manifest contains duplicate run id: {run_id}")
            seen.add(run_id)
            seed = require_uint(
                run.get("seed"),
                f"research manifest point {point_index} run {run_index}.seed",
            )
            relative_dir = require_nonempty_string(
                run.get("relativeDir"),
                f"research manifest point {point_index} run {run_index}.relativeDir",
            )
            config = run.get("runConfig")
            if not isinstance(config, dict) or not isinstance(config.get("experiment"), dict):
                raise AnalysisWindowError(
                    f"research manifest run {run_id} lacks a complete runConfig.experiment"
                )
            duration_years = require_uint(
                config["experiment"].get("durationYears"),
                f"research manifest run {run_id}.durationYears",
            )
            runs.append(
                {
                    "pointId": point_id,
                    "runId": run_id,
                    "seed": seed,
                    "relativeDir": relative_dir,
                    "coordinates": coordinates,
                    "durationYears": duration_years,
                }
            )
    return runs


def resolve_window(
    window: dict[str, Any], terminal_day: int, role: str
) -> dict[str, Any]:
    start = window["analysisStartDay"]
    end = window.get("analysisEndDayInclusive", terminal_day)
    if start > terminal_day:
        raise AnalysisWindowError(
            f"{role} starts on day {start}, beyond run terminal day {terminal_day}"
        )
    if end > terminal_day:
        raise AnalysisWindowError(
            f"{role} ends on day {end}, beyond run terminal day {terminal_day}"
        )
    if end < start:
        raise AnalysisWindowError(f"{role} ends before it starts")
    return {
        "executionInterval": {"startDay": 0, "endDayInclusive": terminal_day},
        "burnInInterval": {"startDay": 0, "endDayExclusive": start},
        "analysisInterval": {"startDay": start, "endDayInclusive": end},
    }


def metric_snapshot_selection(
    root: Path, relative_dir: str, start: int, end: int, run_id: str
) -> dict[str, Any]:
    run_path = root / Path(relative_dir)
    metrics = load_json(run_path / "metrics.json", f"metrics for completed run {run_id}")
    snapshots = metrics.get("snapshots")
    cadence = metrics.get("cadence")
    if not isinstance(cadence, str) or not isinstance(snapshots, list):
        raise AnalysisWindowError(f"metrics for completed run {run_id} are malformed")
    days: list[int] = []
    for index, snapshot in enumerate(snapshots):
        if not isinstance(snapshot, dict):
            raise AnalysisWindowError(
                f"metrics for completed run {run_id} snapshot {index} is not an object"
            )
        days.append(
            require_uint(snapshot.get("day"), f"metrics {run_id} snapshot {index}.day")
        )
    if days != sorted(days):
        raise AnalysisWindowError(f"metrics for completed run {run_id} are not day-ordered")
    included = [day for day in days if start <= day <= end]
    preceding = max((day for day in days if day < start), default=None)
    return {
        "source": f"{relative_dir}/metrics.json",
        "cadence": cadence,
        "analysisStartBoundarySnapshotAvailable": start in days,
        "precedingSnapshotDay": preceding,
        "includedSnapshotDays": included,
        "includedSnapshotCount": len(included),
        "cumulativeCounterRule": (
            "metrics.json cumulative counters retain since-start semantics; interval totals must "
            "be derived against an exact boundary snapshot or from raw events, never reported "
            "directly as analysis-window totals"
        ),
    }


def state_for_run(state: dict[str, Any], planned: dict[str, Any]) -> dict[str, Any]:
    raw = state["runs"].get(planned["runId"])
    if not isinstance(raw, dict):
        raise AnalysisWindowError(
            f"research state is missing planned run {planned['runId']}"
        )
    if raw.get("runId") != planned["runId"] or raw.get("pointId") != planned["pointId"]:
        raise AnalysisWindowError(
            f"research state immutable identity fields differ for run {planned['runId']}"
        )
    if raw.get("seed") != planned["seed"] or raw.get("relativeDir") != planned["relativeDir"]:
        raise AnalysisWindowError(
            f"research state immutable run configuration differs for run {planned['runId']}"
        )
    status = raw.get("state")
    if status not in {"planned", "running", "completed", "failed"}:
        raise AnalysisWindowError(f"research state has invalid status for run {planned['runId']}")
    return raw


def build_output(
    root: Path,
    manifest: dict[str, Any],
    state: dict[str, Any],
    protocol: dict[str, Any],
    identity: str,
    digest: str,
) -> dict[str, Any]:
    output_runs: list[dict[str, Any]] = []
    completed = 0
    for planned in planned_runs(manifest):
        run_state = state_for_run(state, planned)
        terminal_day = planned["durationYears"] * DAYS_PER_YEAR
        primary = resolve_window(
            protocol["analysisWindow"], terminal_day, f"primary window for {planned['runId']}"
        )
        sensitivity = []
        for variant in protocol["sensitivityWindows"]:
            resolved = resolve_window(
                variant,
                terminal_day,
                f"sensitivity window {variant['id']} for {planned['runId']}",
            )
            sensitivity.append(
                {
                    "id": variant["id"],
                    "rationale": variant["rationale"],
                    **resolved,
                }
            )
        row: dict[str, Any] = {
            "pointId": planned["pointId"],
            "runId": planned["runId"],
            "seed": planned["seed"],
            "relativeDir": planned["relativeDir"],
            "coordinates": planned["coordinates"],
            "executionStatus": run_state["state"],
            "primaryWindow": {
                "selectionRule": protocol["analysisWindow"]["selectionRule"],
                "rationale": protocol["analysisWindow"]["rationale"],
                **primary,
            },
            "sensitivityWindows": sensitivity,
        }
        if run_state["state"] == "completed":
            completed += 1
            interval = primary["analysisInterval"]
            row["metricSnapshotSelection"] = metric_snapshot_selection(
                root,
                planned["relativeDir"],
                interval["startDay"],
                interval["endDayInclusive"],
                planned["runId"],
            )
        output_runs.append(row)

    return {
        "schemaVersion": OUTPUT_SCHEMA_VERSION,
        "manifestType": MANIFEST_TYPE,
        "studyId": protocol["studyId"],
        "protocolIdentity": identity,
        "protocolSha256": digest,
        "researchId": manifest["researchId"],
        "definitionIdentity": manifest["definitionIdentity"],
        "source": manifest["source"],
        "declaredPrimaryWindow": protocol["analysisWindow"],
        "declaredSensitivityWindows": protocol["sensitivityWindows"],
        "runCount": len(output_runs),
        "completedRunCount": completed,
        "runs": output_runs,
        "interpretation": {
            "windowSemantics": (
                "execution is [0, terminalDay], burn-in is [0, analysisStartDay), and the "
                "analysis interval is [analysisStartDay, analysisEndDayInclusive]"
            ),
            "initializationSensitivityRule": (
                "predeclared sensitivity windows are preserved as alternative analysis windows; "
                "material conclusion changes must be reported as initialization/path dependence "
                "rather than selecting the preferred window retrospectively"
            ),
            "causalSemantics": (
                "analysis windows select observations only; they do not change authoritative "
                "simulation state, transition rules, RNG streams, or model semantics"
            ),
        },
    }


def atomic_write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    with tempfile.NamedTemporaryFile(
        mode="w",
        encoding="utf-8",
        newline="\n",
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
        delete=False,
    ) as handle:
        temporary = Path(handle.name)
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    try:
        os.replace(temporary, path)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def prepare(root: Path, protocol_path: Path) -> Path:
    root = require_research_root(root)
    raw_protocol = load_json(protocol_path, "analysis-window protocol")
    protocol = validate_protocol(raw_protocol)
    manifest, state = validate_research_metadata(root)
    identity, digest = protocol_identity(protocol)
    output = build_output(root, manifest, state, protocol, identity, digest)
    output_dir = root / "analysis" / "studies" / identity
    atomic_write_json(output_dir / "protocol.json", protocol)
    atomic_write_json(output_dir / "analysis-window-manifest.json", output)
    return output_dir


def main() -> int:
    args = parse_args()
    try:
        output_dir = prepare(args.research_root, args.protocol)
    except AnalysisWindowError as error:
        print(f"research-analysis-window: {error}", file=os.sys.stderr)
        return 1
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
