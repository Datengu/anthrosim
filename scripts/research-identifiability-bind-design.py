#!/usr/bin/env python3
"""Derive a content-bound identifiability design from an AnthroSim research execution.

The emitted binding is intentionally derived from the runner's redundant immutable
research-manifest.json / research-plan.json pair.  It independently recomputes the
research definition, point, run and execution identities used by the Rust runner,
then projects the genuinely executed design coordinates into the identifiability
schema.  Downstream analysis therefore does not need to trust a free-form
``point.parameters`` table as the scientific design authority.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import itertools
import json
import math
from pathlib import Path, PurePosixPath
from typing import Any

BINDING_SCHEMA_VERSION = 1
BINDING_TYPE = "anthrosim-identifiability-executed-design"
SOURCE_KIND = "anthrosim_research_manifest_v1"


class BindingError(Exception):
    pass


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise BindingError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def canonicalize(value: Any) -> Any:
    if isinstance(value, dict):
        return {key: canonicalize(value[key]) for key in sorted(value)}
    if isinstance(value, list):
        return [canonicalize(item) for item in value]
    return value


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        canonicalize(value), ensure_ascii=False, separators=(",", ":"), allow_nan=False
    ).encode("utf-8")


def canonical_sha256(value: Any) -> str:
    return "sha256:" + hashlib.sha256(canonical_bytes(value)).hexdigest()


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x00000100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def stable_identity(prefix: str, value: Any) -> str:
    return f"{prefix}-{fnv1a64(canonical_bytes(value)):016x}"


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise BindingError(f"{role} must be a regular non-symlink file: {path}")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except BindingError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise BindingError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise BindingError(f"{role} root must be an object")
    return value


def nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise BindingError(f"{role} must be a non-empty string")
    return value


def uint(value: Any, role: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise BindingError(f"{role} must be a non-negative integer")
    return value


def finite_number(value: Any, role: str) -> int | float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise BindingError(f"{role} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise BindingError(f"{role} must be finite")
    return value


def pointer_segments(path: str) -> list[str]:
    if not path.startswith("/") or path == "/":
        raise BindingError(f"invalid research coordinate path: {path!r}")
    result: list[str] = []
    for raw in path.split("/")[1:]:
        output = ""
        index = 0
        while index < len(raw):
            if raw[index] != "~":
                output += raw[index]
                index += 1
                continue
            if index + 1 >= len(raw) or raw[index + 1] not in {"0", "1"}:
                raise BindingError(f"invalid JSON-pointer escape in {path!r}")
            output += "~" if raw[index + 1] == "0" else "/"
            index += 2
        result.append(output)
    return result


def pointer_value(value: Any, path: str) -> Any:
    current = value
    for segment in pointer_segments(path):
        if isinstance(current, dict) and segment in current:
            current = current[segment]
        elif isinstance(current, list) and segment.isdigit() and int(segment) < len(current):
            current = current[int(segment)]
        else:
            raise BindingError(
                f"research coordinate path {path!r} is absent from resulting configuration"
            )
    return current


def pointer_set(value: Any, path: str, replacement: Any) -> None:
    segments = pointer_segments(path)
    current = value
    for segment in segments[:-1]:
        if isinstance(current, dict) and segment in current:
            current = current[segment]
        elif isinstance(current, list) and segment.isdigit() and int(segment) < len(current):
            current = current[int(segment)]
        else:
            raise BindingError(f"research coordinate path {path!r} is absent from base configuration")
    final = segments[-1]
    if isinstance(current, dict) and final in current:
        current[final] = copy.deepcopy(replacement)
    elif isinstance(current, list) and final.isdigit() and int(final) < len(current):
        current[int(final)] = copy.deepcopy(replacement)
    else:
        raise BindingError(f"research coordinate path {path!r} is absent from base configuration")


def structure_identity(coordinates: list[dict[str, Any]]) -> str:
    if not coordinates:
        return "default"
    projection = [
        {"id": item["id"], "path": item["path"], "value": item["value"]}
        for item in coordinates
    ]
    return "research-structure-v1-" + hashlib.sha256(canonical_bytes(projection)).hexdigest()


def _source_identity(source: Any) -> dict[str, Any]:
    if not isinstance(source, dict):
        raise BindingError("research manifest source must be an object")
    expected = {"modelVersion", "modelSemanticsId", "gitCommit"}
    if set(source) != expected:
        raise BindingError("research manifest source must contain exactly modelVersion, modelSemanticsId and gitCommit")
    model_version = nonempty_string(source.get("modelVersion"), "source.modelVersion")
    semantics = nonempty_string(source.get("modelSemanticsId"), "source.modelSemanticsId")
    git_commit = nonempty_string(source.get("gitCommit"), "source.gitCommit")
    if git_commit.endswith("-dirty") or "-dirty-" in git_commit:
        raise BindingError("research source identity must be an exact non-dirty git commit")
    return {
        "modelVersion": model_version,
        "modelSemanticsId": semantics,
        "gitCommit": git_commit,
    }


def _execution_identity(definition_identity: str, source: dict[str, Any]) -> str:
    # Mirrors anthrosim-research::execution_identity.  Rust serializes this
    # small struct in declaration order before applying FNV-1a64.
    payload = {
        "schemaVersion": 1,
        "definitionIdentity": definition_identity,
        "source": {
            "modelVersion": source["modelVersion"],
            "modelSemanticsId": source["modelSemanticsId"],
            "gitCommit": source["gitCommit"],
        },
    }
    encoded = json.dumps(payload, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode("utf-8")
    return f"research-execution-v1-{fnv1a64(encoded):016x}"


def _point_identity(index: int, coordinates: list[dict[str, Any]], run_config: dict[str, Any]) -> str:
    return stable_identity(
        "research-point-v1",
        {
            "schemaVersion": 1,
            "index": index,
            "coordinates": coordinates,
            "runConfig": run_config,
        },
    )


def _run_identity(
    point_id: str, run_config: dict[str, Any], source: dict[str, Any]
) -> str:
    return stable_identity(
        "research-run-v1",
        {
            "schemaVersion": 1,
            "pointId": point_id,
            "runConfig": run_config,
            "source": source,
        },
    )


def _validate_manifest(manifest: dict[str, Any]) -> tuple[dict[str, Any], list[dict[str, Any]], list[int], dict[str, Any]]:
    if manifest.get("schemaVersion") != 1:
        raise BindingError("unsupported research execution manifest schema")
    research_id = nonempty_string(manifest.get("researchId"), "researchId")
    definition_identity = nonempty_string(manifest.get("definitionIdentity"), "definitionIdentity")
    source = _source_identity(manifest.get("source"))
    definition = manifest.get("definition")
    if not isinstance(definition, dict) or definition.get("schemaVersion") != 1:
        raise BindingError("research manifest definition must use schemaVersion 1")
    if stable_identity("research-definition-v1", definition) != definition_identity:
        raise BindingError("research definitionIdentity does not match the immutable definition content")
    if _execution_identity(definition_identity, source) != research_id:
        raise BindingError("researchId does not match the immutable definition/source execution identity")

    dimensions = definition.get("dimensions")
    if not isinstance(dimensions, list):
        raise BindingError("research definition dimensions must be an array")
    seeds = definition.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        raise BindingError("research definition seeds must be a non-empty array")
    normalized_seeds = [uint(seed, f"research definition seed {index}") for index, seed in enumerate(seeds)]
    if len(set(normalized_seeds)) != len(normalized_seeds):
        raise BindingError("research definition seeds must be unique")
    base = definition.get("base")
    if not isinstance(base, dict) or not isinstance(base.get("experiment"), dict):
        raise BindingError("research definition base must contain experiment")
    if base["experiment"].get("seed") != normalized_seeds[0]:
        raise BindingError("research definition base seed must equal its first declared seed")
    if not isinstance(manifest.get("points"), list) or not manifest["points"]:
        raise BindingError("research manifest points must be a non-empty array")
    return definition, dimensions, normalized_seeds, source


def _validated_dimensions(dimensions: list[dict[str, Any]], base: dict[str, Any]) -> list[dict[str, Any]]:
    output: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    for index, raw in enumerate(dimensions):
        if not isinstance(raw, dict):
            raise BindingError(f"research dimension {index} must be an object")
        dimension_id = nonempty_string(raw.get("id"), f"research dimension {index}.id")
        if dimension_id in seen_ids:
            raise BindingError(f"duplicate research dimension id: {dimension_id}")
        seen_ids.add(dimension_id)
        kind = raw.get("kind")
        if kind not in {"numeric", "structural"}:
            raise BindingError(f"research dimension {dimension_id} has unsupported kind")
        path = nonempty_string(raw.get("path"), f"research dimension {dimension_id}.path")
        if path in seen_paths:
            raise BindingError(f"duplicate research dimension path: {path}")
        seen_paths.add(path)
        pointer_value(base, path)
        values = raw.get("values")
        if not isinstance(values, list) or not values:
            raise BindingError(f"research dimension {dimension_id} values must be a non-empty array")
        canonical_values = [canonical_bytes(value) for value in values]
        if len(set(canonical_values)) != len(canonical_values):
            raise BindingError(f"research dimension {dimension_id} contains duplicate values")
        if kind == "numeric":
            for value_index, value in enumerate(values):
                finite_number(value, f"research dimension {dimension_id} value {value_index}")
        output.append({"id": dimension_id, "kind": kind, "path": path, "values": copy.deepcopy(values)})
    return output


def build_binding(manifest: dict[str, Any]) -> dict[str, Any]:
    definition, raw_dimensions, seeds, source = _validate_manifest(manifest)
    base = definition["base"]
    dimensions = _validated_dimensions(raw_dimensions, base)
    expected_value_combinations = list(itertools.product(*(dimension["values"] for dimension in dimensions)))
    if not dimensions:
        expected_value_combinations = [()]
    if len(manifest["points"]) != len(expected_value_combinations):
        raise BindingError("research manifest point count does not match the exact Cartesian design")

    bound_points: list[dict[str, Any]] = []
    seen_points: set[str] = set()
    seen_runs: set[str] = set()
    for point_index, (planned, expected_values) in enumerate(
        zip(manifest["points"], expected_value_combinations, strict=True)
    ):
        if not isinstance(planned, dict):
            raise BindingError(f"research manifest point {point_index} must be an object")
        point = planned.get("point")
        runs = planned.get("runs")
        if not isinstance(point, dict) or not isinstance(runs, list):
            raise BindingError(f"research manifest point {point_index} must contain point and runs")
        if point.get("schemaVersion") != 1:
            raise BindingError(f"research point {point_index} must use schemaVersion 1")
        if point.get("index") != point_index:
            raise BindingError(f"research point {point_index} index disagrees with immutable Cartesian order")
        point_id = nonempty_string(point.get("pointId"), f"research manifest point {point_index}.pointId")
        if point_id in seen_points:
            raise BindingError(f"duplicate research point id: {point_id}")
        seen_points.add(point_id)
        run_config = point.get("runConfig")
        coordinates = point.get("coordinates")
        if not isinstance(run_config, dict) or not isinstance(coordinates, list):
            raise BindingError(f"research point {point_id} lacks runConfig/coordinates")
        if len(coordinates) != len(dimensions):
            raise BindingError(f"research point {point_id} does not contain the exact declared coordinate count")

        expected_run_config = copy.deepcopy(base)
        numeric_parameters: dict[str, Any] = {}
        structural_coordinates: list[dict[str, Any]] = []
        normalized_coordinates: list[dict[str, Any]] = []
        for coordinate_index, (coordinate, dimension, expected_value) in enumerate(
            zip(coordinates, dimensions, expected_values, strict=True)
        ):
            if not isinstance(coordinate, dict):
                raise BindingError(f"research point {point_id} coordinate {coordinate_index} must be an object")
            expected_coordinate = {
                "id": dimension["id"],
                "kind": dimension["kind"],
                "path": dimension["path"],
                "value": expected_value,
            }
            if coordinate != expected_coordinate:
                raise BindingError(
                    f"research point {point_id} coordinate {dimension['id']} disagrees with the exact expanded design"
                )
            pointer_set(expected_run_config, dimension["path"], expected_value)
            if pointer_value(run_config, dimension["path"]) != expected_value:
                raise BindingError(
                    f"research point {point_id} coordinate {dimension['id']} disagrees with runConfig"
                )
            normalized_coordinates.append(copy.deepcopy(expected_coordinate))
            if dimension["kind"] == "numeric":
                numeric_parameters[dimension["id"]] = copy.deepcopy(expected_value)
            else:
                structural_coordinates.append(copy.deepcopy(expected_coordinate))
        if run_config != expected_run_config:
            raise BindingError(
                f"research point {point_id} resulting runConfig contains changes outside its declared coordinates"
            )
        expected_point_id = _point_identity(point_index, normalized_coordinates, run_config)
        if point_id != expected_point_id:
            raise BindingError(f"research point {point_id} content does not reproduce its immutable pointId")

        if len(runs) != len(seeds):
            raise BindingError(f"research point {point_id} does not contain exactly one planned run per seed")
        execution_ids: list[str] = []
        point_seed = run_config["experiment"].get("seed")
        for run_index, (run, seed) in enumerate(zip(runs, seeds, strict=True)):
            if not isinstance(run, dict):
                raise BindingError(f"research point {point_id} run {run_index} must be an object")
            if run.get("seed") != seed:
                raise BindingError(f"research point {point_id} run {run_index} is rebound to the wrong seed")
            run_id = nonempty_string(run.get("runId"), f"research point {point_id} run {run_index}.runId")
            if run_id in seen_runs:
                raise BindingError(f"duplicate research run id: {run_id}")
            seen_runs.add(run_id)
            run_run_config = run.get("runConfig")
            if not isinstance(run_run_config, dict) or not isinstance(run_run_config.get("experiment"), dict):
                raise BindingError(f"research run {run_id} lacks runConfig.experiment")
            if run_run_config["experiment"].get("seed") != seed:
                raise BindingError(f"research run {run_id} seed disagrees with runConfig")
            normalized = copy.deepcopy(run_run_config)
            normalized["experiment"]["seed"] = point_seed
            if normalized != run_config:
                raise BindingError(
                    f"research run {run_id} configuration differs from its immutable point except for seed"
                )
            expected_run_id = _run_identity(point_id, run_run_config, source)
            if run_id != expected_run_id:
                raise BindingError(f"research run {run_id} content does not reproduce its immutable runId")
            expected_relative = str(
                PurePosixPath("points")
                / f"point-{point_index:06d}"
                / "runs"
                / f"seed-{run_index:06d}-{seed:020d}"
            )
            if run.get("relativeDir") != expected_relative:
                raise BindingError(f"research run {run_id} relativeDir disagrees with immutable runner layout")
            execution_ids.append(run_id)

        bound_points.append(
            {
                "id": point_id,
                "coordinates": normalized_coordinates,
                "parameters": numeric_parameters,
                "structure": structure_identity(structural_coordinates),
                "executionIds": execution_ids,
            }
        )

    return {
        "schemaVersion": BINDING_SCHEMA_VERSION,
        "bindingType": BINDING_TYPE,
        "sourceKind": SOURCE_KIND,
        "sourceIdentity": canonical_sha256(manifest),
        "researchId": manifest["researchId"],
        "definitionIdentity": manifest["definitionIdentity"],
        "source": source,
        "points": bound_points,
    }


def derive_from_root(root: Path) -> dict[str, Any]:
    if root.is_symlink() or not root.is_dir():
        raise BindingError(f"research root must be a regular directory: {root}")
    manifest = load_json(root / "research-manifest.json", "immutable research manifest")
    plan = load_json(root / "research-plan.json", "immutable research plan")
    if manifest != plan:
        raise BindingError("research-manifest.json and research-plan.json must be exact redundant copies")
    return build_binding(manifest)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("research_root", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        binding = derive_from_root(args.research_root)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(binding, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"designBindingIdentity={canonical_sha256(binding)}")
        print(f"researchId={binding['researchId']}")
        print(f"pointCount={len(binding['points'])}")
        return 0
    except BindingError as error:
        print(f"identifiability binding error: {error}", file=__import__("sys").stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
