#!/usr/bin/env python3
"""Bind real-study deterministic identifiability outputs to authoritative run artifacts.

This module is intentionally separate from the statistical identifiability implementation.
The executed-design binder proves which treatments and run identities were requested. This
module proves that deterministic values used by a real-study identifiability analysis are
reproducible from the exact completed run artifacts attached to those identities.

Synthetic fixtures do not use this module. Their deterministic values remain a test-only
surface protected by the synthetic design digest in ``research-identifiability.py``.
"""

from __future__ import annotations

from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA_VERSION = 1
BINDING_TYPE = "anthrosim-identifiability-authoritative-output-binding"
SOURCE_KIND = "anthrosim_research_completed_run_artifacts_v1"
BUILTIN_ALL_COMPLETED = "all_executions_completed"
RUN_MANIFEST_SCALAR = "run_manifest_scalar_v1"
REQUIRE_EQUAL = "require_equal_v1"


class OutputBindingError(Exception):
    """Raised when the authoritative research archive cannot be interpreted safely."""


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise OutputBindingError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def _read_json(path: Path, role: str) -> tuple[dict[str, Any], str]:
    if path.is_symlink() or not path.is_file():
        raise OutputBindingError(f"{role} must be a regular non-symlink file: {path}")
    try:
        raw = path.read_bytes()
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=_reject_duplicate_keys)
    except OutputBindingError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise OutputBindingError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise OutputBindingError(f"{role} root must be an object: {path}")
    return value, "sha256:" + hashlib.sha256(raw).hexdigest()


def _canonical_sha256(value: Any) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def _finite_number(value: Any, role: str) -> int | float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise OutputBindingError(f"{role} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise OutputBindingError(f"{role} must be finite")
    return value


def _numeric_equal(left: Any, right: Any) -> bool:
    left = _finite_number(left, "submitted deterministic output")
    right = _finite_number(right, "authoritative deterministic output")
    if isinstance(left, int) and isinstance(right, int):
        return left == right
    left_fraction = Fraction(left) if isinstance(left, int) else Fraction.from_float(left)
    right_fraction = Fraction(right) if isinstance(right, int) else Fraction.from_float(right)
    return left_fraction == right_fraction


def _pointer_segments(pointer: str) -> list[str]:
    if not isinstance(pointer, str) or not pointer.startswith("/") or pointer == "/":
        raise OutputBindingError("deterministic derivation jsonPointer must be a non-root JSON pointer")
    segments: list[str] = []
    for raw in pointer.split("/")[1:]:
        output = ""
        index = 0
        while index < len(raw):
            if raw[index] != "~":
                output += raw[index]
                index += 1
                continue
            if index + 1 >= len(raw) or raw[index + 1] not in {"0", "1"}:
                raise OutputBindingError(f"invalid JSON-pointer escape in {pointer!r}")
            output += "~" if raw[index + 1] == "0" else "/"
            index += 2
        segments.append(output)
    return segments


def _pointer_value(value: Any, pointer: str) -> Any:
    current = value
    for segment in _pointer_segments(pointer):
        if isinstance(current, dict) and segment in current:
            current = current[segment]
        elif isinstance(current, list) and segment.isdigit() and int(segment) < len(current):
            current = current[int(segment)]
        else:
            raise OutputBindingError(f"authoritative run manifest has no value at {pointer!r}")
    return current


def _safe_run_dir(root: Path, relative: Any) -> Path:
    if not isinstance(relative, str) or not relative or "\\" in relative:
        raise OutputBindingError("planned research relativeDir must be a non-empty POSIX path")
    parsed = PurePosixPath(relative)
    if parsed.is_absolute() or any(part in {"", ".", ".."} for part in parsed.parts):
        raise OutputBindingError(f"unsafe planned research relativeDir: {relative!r}")
    current = root
    for part in parsed.parts:
        current = current / part
        if current.is_symlink():
            raise OutputBindingError(f"research run path component must not be a symlink: {current}")
    try:
        resolved_root = root.resolve(strict=True)
        resolved_run = current.resolve(strict=True)
        resolved_run.relative_to(resolved_root)
    except (FileNotFoundError, ValueError) as error:
        raise OutputBindingError(f"research run path is missing or escapes its root: {relative!r}") from error
    if not resolved_run.is_dir():
        raise OutputBindingError(f"research run path is not a directory: {resolved_run}")
    return resolved_run


def _required_observables(plan: dict[str, Any]) -> list[str]:
    targets = plan.get("calibrationTargets")
    corroboration = plan.get("corroborationObservables", [])
    if not isinstance(targets, list) or not isinstance(corroboration, list):
        raise OutputBindingError("identifiability plan does not expose valid observable declarations")
    names: list[str] = []
    for target in targets:
        if not isinstance(target, dict) or not isinstance(target.get("observable"), str):
            raise OutputBindingError("identifiability calibration target lacks an observable id")
        names.append(target["observable"])
    for observable in corroboration:
        if not isinstance(observable, str):
            raise OutputBindingError("identifiability corroboration observable id is invalid")
        names.append(observable)
    return list(dict.fromkeys(names))


def _planned_points(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    points = manifest.get("points")
    if not isinstance(points, list) or not points:
        raise OutputBindingError("research manifest points must be a non-empty array")
    output: dict[str, dict[str, Any]] = {}
    for planned in points:
        if not isinstance(planned, dict) or not isinstance(planned.get("point"), dict):
            raise OutputBindingError("research manifest contains an invalid planned point")
        point_id = planned["point"].get("pointId")
        if not isinstance(point_id, str) or not point_id or point_id in output:
            raise OutputBindingError("research manifest contains invalid or duplicate point ids")
        output[point_id] = planned
    return output


def _validate_spatial_attachment(
    run_dir: Path,
    expected_spatial: dict[str, Any],
    core_manifest: dict[str, Any],
    core_checkpoint: dict[str, Any],
) -> dict[str, str]:
    landscape, landscape_digest = _read_json(run_dir / "landscape.json", "research landscape")
    mechanisms, mechanisms_digest = _read_json(
        run_dir / "spatial-mechanisms.json", "research spatial mechanisms"
    )
    wrapper_manifest, wrapper_manifest_digest = _read_json(
        run_dir / "landscape-manifest.json", "research spatial run manifest"
    )
    wrapper_checkpoint, wrapper_checkpoint_digest = _read_json(
        run_dir / "landscape-checkpoint.json", "research spatial checkpoint"
    )
    if landscape != expected_spatial.get("landscape"):
        raise OutputBindingError("completed spatial run landscape differs from immutable run configuration")
    if mechanisms != expected_spatial.get("mechanisms"):
        raise OutputBindingError("completed spatial run mechanisms differ from immutable run configuration")
    if wrapper_manifest.get("coreManifest") != core_manifest:
        raise OutputBindingError("spatial wrapper manifest does not contain the validated core manifest")
    if wrapper_checkpoint.get("coreCheckpoint") != core_checkpoint:
        raise OutputBindingError("spatial wrapper checkpoint does not contain the validated core checkpoint")
    if wrapper_manifest.get("landscape") != wrapper_checkpoint.get("landscape"):
        raise OutputBindingError("spatial manifest/checkpoint landscape bindings disagree")
    if wrapper_manifest.get("spatial") != wrapper_checkpoint.get("spatial"):
        raise OutputBindingError("spatial manifest/checkpoint mechanism bindings disagree")
    spatial_binding = wrapper_manifest.get("spatial")
    if not isinstance(spatial_binding, dict):
        raise OutputBindingError("spatial wrapper manifest lacks its mechanism binding")
    if spatial_binding.get("config") != expected_spatial.get("mechanisms"):
        raise OutputBindingError("spatial wrapper mechanism binding differs from immutable configuration")
    if spatial_binding.get("spatialModelSemanticsId") != expected_spatial.get("spatialModelSemanticsId"):
        raise OutputBindingError("spatial wrapper semantics differ from immutable configuration")
    return {
        "landscape": landscape_digest,
        "spatialMechanisms": mechanisms_digest,
        "landscapeManifest": wrapper_manifest_digest,
        "landscapeCheckpoint": wrapper_checkpoint_digest,
    }


def _validate_completed_execution(
    root: Path,
    research_id: str,
    source: dict[str, Any],
    point_id: str,
    planned: dict[str, Any],
    state_runs: dict[str, Any],
) -> dict[str, Any]:
    run_id = planned.get("runId")
    seed = planned.get("seed")
    relative = planned.get("relativeDir")
    run_config = planned.get("runConfig")
    if not isinstance(run_id, str) or not run_id or not isinstance(run_config, dict):
        raise OutputBindingError(f"planned run for {point_id} is malformed")
    state = state_runs.get(run_id)
    if not isinstance(state, dict):
        raise OutputBindingError(f"research state is missing planned execution {run_id}")
    expected_identity = {
        "runId": run_id,
        "pointId": point_id,
        "seed": seed,
        "relativeDir": relative,
    }
    for key, expected in expected_identity.items():
        if state.get(key) != expected:
            raise OutputBindingError(f"research state {run_id} {key} differs from immutable plan")
    if state.get("state") != "completed":
        raise OutputBindingError(f"research execution {run_id} is not completed")
    state_digest = state.get("stateDigest64")
    if not isinstance(state_digest, int) or isinstance(state_digest, bool):
        raise OutputBindingError(f"research execution {run_id} lacks a completed state digest")

    run_dir = _safe_run_dir(root, relative)
    manifest, manifest_digest = _read_json(run_dir / "manifest.json", "completed run manifest")
    checkpoint, checkpoint_digest = _read_json(run_dir / "checkpoint.json", "completed run checkpoint")
    expected_experiment = run_config.get("experiment")
    if not isinstance(expected_experiment, dict):
        raise OutputBindingError(f"research execution {run_id} lacks runConfig.experiment")
    for key in ("modelVersion", "modelSemanticsId", "gitCommit"):
        if manifest.get(key) != source.get(key) or checkpoint.get(key) != source.get(key):
            raise OutputBindingError(f"research execution {run_id} source identity mismatch at {key}")
    if manifest.get("experiment") != expected_experiment or checkpoint.get("experiment") != expected_experiment:
        raise OutputBindingError(f"research execution {run_id} experiment differs from immutable run configuration")
    if manifest.get("stateDigest64") != state_digest or checkpoint.get("stateDigest64") != state_digest:
        raise OutputBindingError(f"research execution {run_id} state digest disagrees across state/manifest/checkpoint")

    artifact_digests = {"manifest": manifest_digest, "checkpoint": checkpoint_digest}
    expected_spatial = run_config.get("spatial")
    if expected_spatial is not None:
        if not isinstance(expected_spatial, dict):
            raise OutputBindingError(f"research execution {run_id} has malformed spatial configuration")
        artifact_digests.update(
            _validate_spatial_attachment(run_dir, expected_spatial, manifest, checkpoint)
        )

    return {
        "runId": run_id,
        "pointId": point_id,
        "seed": seed,
        "relativeDir": relative,
        "stateDigest64": state_digest,
        "manifest": manifest,
        "artifactDigests": artifact_digests,
        "researchId": research_id,
    }


def _resolve_deterministic(
    point_id: str,
    observable: str,
    declaration: dict[str, Any],
    submitted: Any,
    executions: list[dict[str, Any]],
) -> tuple[dict[str, Any], str | None]:
    if declaration == {"kind": "deterministic"} and observable == BUILTIN_ALL_COMPLETED:
        expected: int | float = 1.0
        record = {
            "pointId": point_id,
            "observable": observable,
            "derivation": {"kind": "all_executions_completed_v1"},
            "executionIds": [item["runId"] for item in executions],
            "authoritativeValue": expected,
            "artifacts": [
                {"runId": item["runId"], "digests": item["artifactDigests"]}
                for item in executions
            ],
        }
        error = None if _numeric_equal(submitted, expected) else f"point_{point_id}_{observable}_authoritative_value_mismatch"
        return record, error

    derivation = declaration.get("derivation")
    if set(declaration) != {"kind", "derivation"} or not isinstance(derivation, dict):
        return (
            {
                "pointId": point_id,
                "observable": observable,
                "executionIds": [item["runId"] for item in executions],
                "authoritativeValue": None,
            },
            f"point_{point_id}_{observable}_deterministic_output_missing_authoritative_derivation",
        )
    if set(derivation) != {"kind", "jsonPointer", "reducer"}:
        raise OutputBindingError(
            f"deterministic derivation for {point_id}.{observable} must contain only kind, jsonPointer and reducer"
        )
    if derivation.get("kind") != RUN_MANIFEST_SCALAR or derivation.get("reducer") != REQUIRE_EQUAL:
        raise OutputBindingError(
            f"deterministic derivation for {point_id}.{observable} must use {RUN_MANIFEST_SCALAR}/{REQUIRE_EQUAL}"
        )
    pointer = derivation.get("jsonPointer")
    values = [
        _finite_number(_pointer_value(item["manifest"], pointer), f"{item['runId']} {pointer}")
        for item in executions
    ]
    expected = values[0]
    if any(not _numeric_equal(value, expected) for value in values[1:]):
        error = f"point_{point_id}_{observable}_authoritative_values_not_equal_across_executions"
        expected_output: int | float | None = None
    else:
        expected_output = expected
        error = None if _numeric_equal(submitted, expected) else f"point_{point_id}_{observable}_authoritative_value_mismatch"
    return (
        {
            "pointId": point_id,
            "observable": observable,
            "derivation": {
                "kind": RUN_MANIFEST_SCALAR,
                "jsonPointer": pointer,
                "reducer": REQUIRE_EQUAL,
            },
            "executionIds": [item["runId"] for item in executions],
            "sourceValues": values,
            "authoritativeValue": expected_output,
            "artifacts": [
                {"runId": item["runId"], "digests": item["artifactDigests"]}
                for item in executions
            ],
        },
        error,
    )


def validate_real_study_outputs(
    plan: dict[str, Any],
    data: dict[str, Any],
    binding: dict[str, Any],
    research_root: Path,
) -> tuple[dict[str, Any], list[str]]:
    """Validate all claim-driving deterministic values against the exact research archive."""

    if research_root.is_symlink() or not research_root.is_dir():
        raise OutputBindingError(f"research root must be a regular directory: {research_root}")
    manifest, _ = _read_json(research_root / "research-manifest.json", "immutable research manifest")
    state, state_digest = _read_json(research_root / "research-state.json", "research execution state")
    if manifest.get("researchId") != binding.get("researchId"):
        raise OutputBindingError("authoritative output resolver researchId disagrees with executed-design binding")
    if manifest.get("definitionIdentity") != binding.get("definitionIdentity"):
        raise OutputBindingError("authoritative output resolver definitionIdentity disagrees with executed-design binding")
    source = manifest.get("source")
    if not isinstance(source, dict) or source != binding.get("source"):
        raise OutputBindingError("authoritative output resolver source disagrees with executed-design binding")
    if state.get("schemaVersion") != 1 or state.get("researchId") != binding.get("researchId"):
        raise OutputBindingError("research-state.json does not belong to the bound research execution")
    state_runs = state.get("runs")
    if not isinstance(state_runs, dict):
        raise OutputBindingError("research-state.json runs must be an object")

    planned_by_id = _planned_points(manifest)
    bound_points = binding.get("points")
    raw_points = data.get("points")
    if not isinstance(bound_points, list) or not isinstance(raw_points, list):
        raise OutputBindingError("identifiability points are unavailable for output binding")
    required = _required_observables(plan)
    errors: list[str] = []
    derivations: list[dict[str, Any]] = []
    observed_run_ids: set[str] = set()

    for raw, bound in zip(raw_points, bound_points, strict=True):
        if not isinstance(raw, dict) or not isinstance(bound, dict):
            raise OutputBindingError("identifiability point shape is invalid during output binding")
        point_id = bound.get("id")
        planned_point = planned_by_id.get(point_id)
        if not isinstance(planned_point, dict):
            raise OutputBindingError(f"bound point {point_id} is absent from immutable research manifest")
        planned_runs = planned_point.get("runs")
        if not isinstance(planned_runs, list) or not planned_runs:
            raise OutputBindingError(f"bound point {point_id} has no immutable planned runs")
        execution_ids = bound.get("executionIds")
        if [item.get("runId") for item in planned_runs if isinstance(item, dict)] != execution_ids:
            raise OutputBindingError(f"bound execution IDs for point {point_id} disagree with immutable manifest")
        executions = [
            _validate_completed_execution(
                research_root,
                binding["researchId"],
                source,
                point_id,
                planned,
                state_runs,
            )
            for planned in planned_runs
        ]
        observed_run_ids.update(item["runId"] for item in executions)

        outputs = raw.get("outputs")
        evidence = raw.get("outputEvidence")
        if not isinstance(outputs, dict) or not isinstance(evidence, dict):
            raise OutputBindingError(f"point {point_id} lacks outputs/outputEvidence")
        for observable in required:
            declaration = evidence.get(observable)
            if not isinstance(declaration, dict) or declaration.get("kind") != "deterministic":
                continue
            if observable not in outputs:
                raise OutputBindingError(f"point {point_id} is missing deterministic output {observable}")
            record, mismatch = _resolve_deterministic(
                point_id, observable, declaration, outputs[observable], executions
            )
            derivations.append(record)
            if mismatch is not None:
                errors.append(mismatch)

    if set(state_runs) != observed_run_ids:
        raise OutputBindingError("research-state.json run set differs from the immutable bound execution set")

    summary_core = {
        "schemaVersion": SCHEMA_VERSION,
        "bindingType": BINDING_TYPE,
        "sourceKind": SOURCE_KIND,
        "researchId": binding.get("researchId"),
        "definitionIdentity": binding.get("definitionIdentity"),
        "executedDesignBindingIdentity": _canonical_sha256(binding),
        "researchStateIdentity": state_digest,
        "derivations": derivations,
    }
    summary = {
        **summary_core,
        "bindingIdentity": _canonical_sha256(summary_core),
        "valid": not errors,
        "validationErrors": errors,
    }
    return summary, errors
