#!/usr/bin/env python3
"""Derive a content-bound identifiability design from an AnthroSim research execution."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any

BINDING_SCHEMA_VERSION = 1
BINDING_TYPE = "anthrosim-identifiability-executed-design"
SOURCE_KIND = "anthrosim_research_manifest_v1"


class BindingError(Exception):
    pass


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def canonical_sha256(value: Any) -> str:
    return "sha256:" + hashlib.sha256(canonical_bytes(value)).hexdigest()


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise BindingError(f"{role} must be a regular non-symlink file: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
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
            raise BindingError(f"research coordinate path {path!r} is absent from resulting configuration")
    return current


def structure_identity(coordinates: list[dict[str, Any]]) -> str:
    if not coordinates:
        return "default"
    projection = [{"id": item["id"], "value": item["value"]} for item in coordinates]
    return "research-structure-v1-" + hashlib.sha256(canonical_bytes(projection)).hexdigest()


def _validate_manifest(manifest: dict[str, Any]) -> None:
    if manifest.get("schemaVersion") != 1:
        raise BindingError("unsupported research execution manifest schema")
    nonempty_string(manifest.get("researchId"), "researchId")
    nonempty_string(manifest.get("definitionIdentity"), "definitionIdentity")
    if not isinstance(manifest.get("source"), dict):
        raise BindingError("research manifest source must be an object")
    definition = manifest.get("definition")
    if not isinstance(definition, dict) or definition.get("schemaVersion") != 1:
        raise BindingError("research manifest definition must use schemaVersion 1")
    dimensions = definition.get("dimensions")
    if not isinstance(dimensions, list):
        raise BindingError("research definition dimensions must be an array")
    seeds = definition.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        raise BindingError("research definition seeds must be a non-empty array")
    for index, seed in enumerate(seeds):
        uint(seed, f"research definition seed {index}")
    if len(set(seeds)) != len(seeds):
        raise BindingError("research definition seeds must be unique")
    if not isinstance(manifest.get("points"), list) or not manifest["points"]:
        raise BindingError("research manifest points must be a non-empty array")


def build_binding(manifest: dict[str, Any]) -> dict[str, Any]:
    _validate_manifest(manifest)
    dimensions = manifest["definition"]["dimensions"]
    dimension_by_id: dict[str, dict[str, Any]] = {}
    for index, dimension in enumerate(dimensions):
        if not isinstance(dimension, dict):
            raise BindingError(f"research dimension {index} must be an object")
        dimension_id = nonempty_string(dimension.get("id"), f"research dimension {index}.id")
        if dimension_id in dimension_by_id:
            raise BindingError(f"duplicate research dimension id: {dimension_id}")
        if dimension.get("kind") not in {"numeric", "structural"}:
            raise BindingError(f"research dimension {dimension_id} has unsupported kind")
        nonempty_string(dimension.get("path"), f"research dimension {dimension_id}.path")
        dimension_by_id[dimension_id] = dimension

    bound_points: list[dict[str, Any]] = []
    seen_points: set[str] = set()
    seen_runs: set[str] = set()
    for point_index, planned in enumerate(manifest["points"]):
        if not isinstance(planned, dict):
            raise BindingError(f"research manifest point {point_index} must be an object")
        point = planned.get("point")
        runs = planned.get("runs")
        if not isinstance(point, dict) or not isinstance(runs, list) or not runs:
            raise BindingError(f"research manifest point {point_index} must contain point and non-empty runs")
        point_id = nonempty_string(point.get("pointId"), f"research manifest point {point_index}.pointId")
        if point_id in seen_points:
            raise BindingError(f"duplicate research point id: {point_id}")
        seen_points.add(point_id)
        uint(point.get("index"), f"research point {point_id}.index")
        run_config = point.get("runConfig")
        coordinates = point.get("coordinates")
        if not isinstance(run_config, dict) or not isinstance(coordinates, list):
            raise BindingError(f"research point {point_id} lacks runConfig/coordinates")
        coordinate_ids: list[str] = []
        numeric_parameters: dict[str, Any] = {}
        structural_coordinates: list[dict[str, Any]] = []
        for coordinate_index, coordinate in enumerate(coordinates):
            if not isinstance(coordinate, dict):
                raise BindingError(f"research point {point_id} coordinate {coordinate_index} must be an object")
            coordinate_id = nonempty_string(coordinate.get("id"), f"research point {point_id} coordinate id")
            if coordinate_id in coordinate_ids:
                raise BindingError(f"research point {point_id} contains duplicate coordinate {coordinate_id}")
            coordinate_ids.append(coordinate_id)
            dimension = dimension_by_id.get(coordinate_id)
            if dimension is None:
                raise BindingError(f"research point {point_id} contains unknown coordinate {coordinate_id}")
            if coordinate.get("kind") != dimension["kind"] or coordinate.get("path") != dimension["path"]:
                raise BindingError(f"research point {point_id} coordinate {coordinate_id} disagrees with definition")
            if pointer_value(run_config, dimension["path"]) != coordinate.get("value"):
                raise BindingError(f"research point {point_id} coordinate {coordinate_id} disagrees with runConfig")
            if dimension["kind"] == "numeric":
                value = coordinate.get("value")
                if isinstance(value, bool) or not isinstance(value, (int, float)):
                    raise BindingError(f"numeric research coordinate {coordinate_id} must be numeric")
                numeric_parameters[coordinate_id] = copy.deepcopy(value)
            else:
                structural_coordinates.append(copy.deepcopy(coordinate))
        if coordinate_ids != [dimension["id"] for dimension in dimensions]:
            raise BindingError(f"research point {point_id} does not contain the exact declared coordinate set/order")

        execution_ids: list[str] = []
        point_seed = None
        if isinstance(run_config.get("experiment"), dict):
            point_seed = run_config["experiment"].get("seed")
        for run_index, run in enumerate(runs):
            if not isinstance(run, dict):
                raise BindingError(f"research point {point_id} run {run_index} must be an object")
            run_id = nonempty_string(run.get("runId"), f"research point {point_id} run {run_index}.runId")
            if run_id in seen_runs:
                raise BindingError(f"duplicate research run id: {run_id}")
            seen_runs.add(run_id)
            seed = uint(run.get("seed"), f"research run {run_id}.seed")
            run_run_config = run.get("runConfig")
            if not isinstance(run_run_config, dict) or not isinstance(run_run_config.get("experiment"), dict):
                raise BindingError(f"research run {run_id} lacks runConfig.experiment")
            if run_run_config["experiment"].get("seed") != seed:
                raise BindingError(f"research run {run_id} seed disagrees with runConfig")
            normalized = copy.deepcopy(run_run_config)
            normalized["experiment"]["seed"] = point_seed
            if normalized != run_config:
                raise BindingError(f"research run {run_id} configuration differs from its immutable point except for seed")
            execution_ids.append(run_id)

        bound_points.append(
            {
                "id": point_id,
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
