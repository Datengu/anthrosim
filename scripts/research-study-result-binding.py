#!/usr/bin/env python3
"""Verify finalized study bindings against frozen producer authority."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_research_study_result_binding_legacy",
    HERE / "research-study-result-binding-legacy.py",
)


def research_execution_identity(definition_id: str, source: dict[str, Any]) -> str:
    """Match anthrosim-research/study schema-v1 ordered serde_json identity exactly."""
    definition_id = _legacy.nonempty_string(definition_id, "definition identity")
    source = _legacy.validate_source(source, "research execution source")
    payload = {
        "schemaVersion": 1,
        "definitionIdentity": definition_id,
        "source": {
            "modelVersion": source["modelVersion"],
            "modelSemanticsId": source["modelSemanticsId"],
            "gitCommit": source["gitCommit"],
        },
    }
    encoded = json.dumps(
        payload, ensure_ascii=False, sort_keys=False, separators=(",", ":")
    ).encode("utf-8")
    return f"{_legacy.RESEARCH_EXECUTION_PREFIX}-{_legacy.fnv1a64(encoded):016x}"


# The preserved legacy verifier resolves this global at call time. Keep the historical
# producer byte order while strengthening the wrapper's semantic root verification.
_legacy.research_execution_identity = research_execution_identity
_legacy_validate_study_root = _legacy.validate_study_root


def _state_value(value: Any, role: str, *, allow_none: bool = False) -> Any:
    if value is None and allow_none:
        return None
    if role.endswith("stateDigest64"):
        digest = _legacy.uint(value, role)
        if digest > 0xFFFFFFFFFFFFFFFF:
            raise _legacy.StudyBindingError(f"{role} exceeds u64")
        return digest
    if role.endswith("error"):
        return _legacy.nonempty_string(value, role)
    return value


def _expected_analysis_artifacts(
    research_manifest: dict[str, Any], state: dict[str, Any], research_id: str
) -> dict[str, dict[str, Any]]:
    points = research_manifest.get("points")
    if not isinstance(points, list):
        raise _legacy.StudyBindingError(
            "immutable research manifest points must be an array"
        )
    state_runs = state.get("runs")
    if not isinstance(state_runs, dict):
        raise _legacy.StudyBindingError("research-state.json runs must be an object")

    analysis_points: list[dict[str, Any]] = []
    analysis_runs: list[dict[str, Any]] = []
    planned_run_ids: set[str] = set()

    for point_position, planned_point in enumerate(points):
        if not isinstance(planned_point, dict):
            raise _legacy.StudyBindingError(
                f"immutable research point[{point_position}] must be an object"
            )
        point = planned_point.get("point")
        runs = planned_point.get("runs")
        if not isinstance(point, dict) or not isinstance(runs, list):
            raise _legacy.StudyBindingError(
                f"immutable research point[{point_position}] is malformed"
            )

        point_id = _legacy.nonempty_string(
            point.get("pointId"),
            f"immutable research point[{point_position}].pointId",
        )
        point_index = _legacy.uint(
            point.get("index"), f"immutable research point[{point_position}].index"
        )
        coordinates = point.get("coordinates")
        if not isinstance(coordinates, list):
            raise _legacy.StudyBindingError(
                f"immutable research point[{point_position}].coordinates must be an array"
            )
        point_run_config = point.get("runConfig")
        if not isinstance(point_run_config, dict):
            raise _legacy.StudyBindingError(
                f"immutable research point[{point_position}].runConfig must be an object"
            )

        point_run_ids: list[str] = []
        for run_position, planned_run in enumerate(runs):
            role = f"immutable research point[{point_position}] run[{run_position}]"
            if not isinstance(planned_run, dict):
                raise _legacy.StudyBindingError(f"{role} must be an object")
            run_id = _legacy.nonempty_string(planned_run.get("runId"), f"{role}.runId")
            if run_id in planned_run_ids:
                raise _legacy.StudyBindingError(
                    f"immutable research manifest contains duplicate runId: {run_id}"
                )
            planned_run_ids.add(run_id)
            point_run_ids.append(run_id)

            seed = _legacy.uint(planned_run.get("seed"), f"{role}.seed")
            relative_dir = _legacy.canonical_relative_path(
                planned_run.get("relativeDir"), f"{role}.relativeDir"
            )
            run_config = planned_run.get("runConfig")
            if not isinstance(run_config, dict):
                raise _legacy.StudyBindingError(f"{role}.runConfig must be an object")

            run_state = state_runs.get(run_id)
            if not isinstance(run_state, dict):
                raise _legacy.StudyBindingError(
                    f"research-state.json is missing immutable planned run {run_id}"
                )
            expected_state_identity = {
                "runId": run_id,
                "pointId": point_id,
                "seed": seed,
                "relativeDir": relative_dir,
            }
            for field, expected in expected_state_identity.items():
                if run_state.get(field) != expected:
                    raise _legacy.StudyBindingError(
                        f"research-state run {run_id} {field} differs from immutable research plan"
                    )

            attempt = _legacy.uint(run_state.get("attempt"), f"research-state run {run_id}.attempt")
            status = _legacy.nonempty_string(
                run_state.get("state"), f"research-state run {run_id}.state"
            )
            if status not in {"completed", "failed"}:
                raise _legacy.StudyBindingError(
                    f"research execution is not finalized; run {run_id} remains {status}"
                )
            state_digest64 = _state_value(
                run_state.get("stateDigest64"),
                f"research-state run {run_id}.stateDigest64",
                allow_none=True,
            )
            error = _state_value(
                run_state.get("error"),
                f"research-state run {run_id}.error",
                allow_none=True,
            )

            analysis_runs.append(
                {
                    "pointId": point_id,
                    "runId": run_id,
                    "seed": seed,
                    "coordinates": coordinates,
                    "resultingConfiguration": run_config,
                    "relativeDir": relative_dir,
                    "attempt": attempt,
                    "state": status,
                    "stateDigest64": state_digest64,
                    "error": error,
                }
            )

        analysis_points.append(
            {
                "pointId": point_id,
                "index": point_index,
                "coordinates": coordinates,
                "resultingConfiguration": point_run_config,
                "runIds": point_run_ids,
            }
        )

    state_run_ids = set(state_runs)
    if state_run_ids != planned_run_ids:
        missing = sorted(planned_run_ids - state_run_ids)
        extra = sorted(state_run_ids - planned_run_ids)
        raise _legacy.StudyBindingError(
            "research-state.json run set differs from immutable research plan; "
            f"missing={missing}, extra={extra}"
        )

    return {
        "research/analysis/points.json": {
            "schemaVersion": 1,
            "researchId": research_id,
            "points": analysis_points,
        },
        "research/analysis/runs.json": {
            "schemaVersion": 1,
            "researchId": research_id,
            "runs": analysis_runs,
        },
    }


def validate_study_root(study_root: Path) -> dict[str, Any]:
    """Validate binding identity, frozen lineage, and producer-equivalent canonical rows."""
    context = _legacy_validate_study_root(study_root)
    root = study_root.resolve(strict=True)
    binding = context["binding"]
    expected = _expected_analysis_artifacts(
        context["researchManifest"], context["researchState"], binding["researchId"]
    )
    for relative_path, expected_json in expected.items():
        actual_json = _legacy.load_json(
            _legacy.resolve_inside(root, relative_path, "research analysis artifact"),
            "research analysis artifact",
        )
        if actual_json != expected_json:
            raise _legacy.StudyBindingError(
                "research analysis artifact differs from immutable research plan/state: "
                f"{root / relative_path}"
            )
    return context


for _name in dir(_legacy):
    if _name.startswith("__") or _name in {
        "research_execution_identity",
        "validate_study_root",
    }:
        continue
    globals()[_name] = getattr(_legacy, _name)
