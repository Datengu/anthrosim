#!/usr/bin/env python3
"""Assemble a normalized AnthroSim landscape bundle from pre-aligned GIS exports.

Raw GIS work remains external (for example QGIS/GDAL). This script is the
reproducible boundary that validates aligned integer-grid exports, records the
recipe/tool/source identities, and emits the M8.1 LandscapeBundle JSON shape.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from pathlib import Path
from typing import Any

RECIPE_SCHEMA_VERSION = 2
LANDSCAPE_SCHEMA_VERSION = 2
GRID_CONVENTION = {
    "originAnchor": "upper_left_outer_corner",
    "columnDirection": "increasing_x",
    "rowDirection": "decreasing_y",
    "cellInterpretation": "area",
}
ALLOWED_ROLES = {
    "terrain_traversal",
    "water_accessibility",
    "resource_opportunity",
    "auxiliary",
}


class RecipeError(ValueError):
    pass


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def require_nonempty(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise RecipeError(f"{field} must be a non-empty string")
    return value


def require_positive_int(value: Any, field: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise RecipeError(f"{field} must be a positive integer")
    return value


def require_grid_convention(value: Any, field: str) -> dict[str, str]:
    if not isinstance(value, dict):
        raise RecipeError(f"{field} must be an object")
    if value != GRID_CONVENTION:
        raise RecipeError(
            f"{field} must exactly match the supported normalized grid convention: {GRID_CONVENTION}"
        )
    return dict(GRID_CONVENTION)


def resolve_input(recipe_dir: Path, raw_path: Any, field: str) -> tuple[Path, str]:
    declared = require_nonempty(raw_path, field)
    raw = Path(declared)
    path = raw.resolve() if raw.is_absolute() else (recipe_dir / raw).resolve()
    if not path.is_file():
        raise RecipeError(f"{field} does not exist: {declared}")
    # Preserve only the recipe-declared locator in the reproduction record.
    # Do not leak the machine-specific resolved path of private/external data.
    return path, declared.replace("\\", "/")


def load_grid(path: Path, width: int, height: int, nodata_token: str) -> list[int | None]:
    values: list[int | None] = []
    row_count = 0
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.reader(handle)
        for row_index, row in enumerate(reader):
            if len(row) != width:
                raise RecipeError(
                    f"{path.name} row {row_index + 1} has {len(row)} cells; expected {width}"
                )
            row_count += 1
            for column_index, raw in enumerate(row):
                token = raw.strip()
                if token == nodata_token:
                    values.append(None)
                    continue
                try:
                    values.append(int(token))
                except ValueError as exc:
                    raise RecipeError(
                        f"{path.name} row {row_index + 1} column {column_index + 1} "
                        f"is neither integer nor nodata token {nodata_token!r}: {token!r}"
                    ) from exc
    if row_count != height:
        raise RecipeError(f"{path.name} has {row_count} rows; expected {height}")
    return values


def build(recipe_path: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    recipe_bytes = recipe_path.read_bytes()
    try:
        recipe = json.loads(recipe_bytes)
    except json.JSONDecodeError as exc:
        raise RecipeError(f"invalid recipe JSON: {exc}") from exc

    if recipe.get("schemaVersion") != RECIPE_SCHEMA_VERSION:
        raise RecipeError(
            f"recipe schemaVersion must be {RECIPE_SCHEMA_VERSION}; found {recipe.get('schemaVersion')!r}"
        )

    normalized_grid_convention = require_grid_convention(
        recipe.get("gridConvention"), "gridConvention"
    )
    aligned_input_grid_convention = require_grid_convention(
        recipe.get("alignedInputGridConvention"), "alignedInputGridConvention"
    )
    if aligned_input_grid_convention != normalized_grid_convention:
        raise RecipeError(
            "alignedInputGridConvention must match gridConvention; this builder does not silently flip or transpose aligned CSV inputs"
        )

    recipe_dir = recipe_path.parent.resolve()
    geometry = recipe.get("geometry")
    if not isinstance(geometry, dict):
        raise RecipeError("geometry must be an object")

    width = require_positive_int(geometry.get("width"), "geometry.width")
    height = require_positive_int(geometry.get("height"), "geometry.height")
    cell_size_x = require_positive_int(geometry.get("cellSizeX"), "geometry.cellSizeX")
    cell_size_y = require_positive_int(geometry.get("cellSizeY"), "geometry.cellSizeY")
    origin_x = geometry.get("originX")
    origin_y = geometry.get("originY")
    if not isinstance(origin_x, int) or isinstance(origin_x, bool):
        raise RecipeError("geometry.originX must be an integer")
    if not isinstance(origin_y, int) or isinstance(origin_y, bool):
        raise RecipeError("geometry.originY must be an integer")
    coordinate_unit = require_nonempty(geometry.get("coordinateUnit"), "geometry.coordinateUnit")
    spatial_reference = require_nonempty(
        geometry.get("spatialReference"), "geometry.spatialReference"
    )

    toolchain = recipe.get("toolchain", [])
    if not isinstance(toolchain, list):
        raise RecipeError("toolchain must be an array")
    normalized_tools = []
    for index, tool in enumerate(toolchain):
        if not isinstance(tool, dict):
            raise RecipeError(f"toolchain[{index}] must be an object")
        normalized_tools.append(
            {
                "name": require_nonempty(tool.get("name"), f"toolchain[{index}].name"),
                "version": require_nonempty(tool.get("version"), f"toolchain[{index}].version"),
            }
        )

    steps = recipe.get("steps", [])
    if not isinstance(steps, list):
        raise RecipeError("steps must be an array")
    normalized_steps = []
    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            raise RecipeError(f"steps[{index}] must be an object")
        normalized_steps.append(
            {
                "description": require_nonempty(
                    step.get("description"), f"steps[{index}].description"
                ),
                "command": require_nonempty(step.get("command"), f"steps[{index}].command"),
            }
        )

    sources = recipe.get("sources", [])
    if not isinstance(sources, list):
        raise RecipeError("sources must be an array")
    source_records = []
    source_ids: set[str] = set()
    for index, source in enumerate(sources):
        if not isinstance(source, dict):
            raise RecipeError(f"sources[{index}] must be an object")
        source_id = require_nonempty(source.get("sourceId"), f"sources[{index}].sourceId")
        if source_id in source_ids:
            raise RecipeError(f"duplicate sourceId: {source_id}")
        source_ids.add(source_id)
        source_path, source_locator = resolve_input(
            recipe_dir, source.get("path"), f"sources[{index}].path"
        )
        source_records.append(
            {
                "sourceId": source_id,
                "path": source_locator,
                "sha256": sha256(source_path),
                "citation": require_nonempty(source.get("citation"), f"sources[{index}].citation"),
                "datasetVersion": source.get("datasetVersion"),
                "licence": source.get("licence"),
            }
        )

    layers = recipe.get("layers")
    if not isinstance(layers, list) or not layers:
        raise RecipeError("layers must be a non-empty array")
    layer_ids: set[str] = set()
    normalized_layers = []
    layer_inputs = []
    for index, layer in enumerate(layers):
        if not isinstance(layer, dict):
            raise RecipeError(f"layers[{index}] must be an object")
        layer_id = require_nonempty(layer.get("layerId"), f"layers[{index}].layerId")
        if layer_id in layer_ids:
            raise RecipeError(f"duplicate layerId: {layer_id}")
        layer_ids.add(layer_id)
        role = require_nonempty(layer.get("role"), f"layers[{index}].role")
        if role not in ALLOWED_ROLES:
            raise RecipeError(f"layers[{index}].role is unsupported: {role}")
        unit = require_nonempty(layer.get("unit"), f"layers[{index}].unit")
        grid_path, grid_locator = resolve_input(
            recipe_dir, layer.get("path"), f"layers[{index}].path"
        )
        nodata_token = str(layer.get("nodataToken", "NA"))
        values = load_grid(grid_path, width, height, nodata_token)

        domain = layer.get("valueDomain")
        normalized_domain = None
        if domain is not None:
            if not isinstance(domain, dict):
                raise RecipeError(f"layers[{index}].valueDomain must be an object")
            minimum = domain.get("min")
            maximum = domain.get("max")
            if not isinstance(minimum, int) or isinstance(minimum, bool):
                raise RecipeError(f"layers[{index}].valueDomain.min must be an integer")
            if not isinstance(maximum, int) or isinstance(maximum, bool):
                raise RecipeError(f"layers[{index}].valueDomain.max must be an integer")
            if minimum > maximum:
                raise RecipeError(f"layers[{index}].valueDomain min exceeds max")
            for cell_index, value in enumerate(values):
                if value is not None and not minimum <= value <= maximum:
                    raise RecipeError(
                        f"layer {layer_id} cell {cell_index} value {value} is outside [{minimum}, {maximum}]"
                    )
            normalized_domain = {"min": minimum, "max": maximum}

        normalized = {"layerId": layer_id, "role": role, "unit": unit}
        if normalized_domain is not None:
            normalized["valueDomain"] = normalized_domain
        evidence_input_id = layer.get("evidenceInputId")
        if evidence_input_id is not None:
            normalized["evidenceInputId"] = require_nonempty(
                evidence_input_id, f"layers[{index}].evidenceInputId"
            )
        normalized["values"] = values
        normalized_layers.append(normalized)
        layer_inputs.append(
            {
                "layerId": layer_id,
                "path": grid_locator,
                "sha256": sha256(grid_path),
                "nodataToken": nodata_token,
                "gridConvention": aligned_input_grid_convention,
            }
        )

    landscape = {
        "schemaVersion": LANDSCAPE_SCHEMA_VERSION,
        "width": width,
        "height": height,
        "gridConvention": normalized_grid_convention,
        "geometry": {
            "originX": origin_x,
            "originY": origin_y,
            "cellSizeX": cell_size_x,
            "cellSizeY": cell_size_y,
            "coordinateUnit": coordinate_unit,
            "spatialReference": spatial_reference,
        },
        "layers": normalized_layers,
    }

    output_bytes = canonical_json_bytes(landscape)
    record = {
        "schemaVersion": RECIPE_SCHEMA_VERSION,
        "recipeSha256": hashlib.sha256(recipe_bytes).hexdigest(),
        "landscapeSha256": hashlib.sha256(output_bytes).hexdigest(),
        "normalizedGridConvention": normalized_grid_convention,
        "alignedInputGridConvention": aligned_input_grid_convention,
        "toolchain": normalized_tools,
        "steps": normalized_steps,
        "sources": source_records,
        "layerInputs": layer_inputs,
        "nodataPolicy": require_nonempty(recipe.get("nodataPolicy"), "nodataPolicy"),
        "notes": recipe.get("notes"),
    }
    return landscape, record


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("recipe", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    args = parser.parse_args()

    try:
        landscape, record = build(args.recipe.resolve())
    except (OSError, RecipeError) as exc:
        parser.error(str(exc))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.record.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(canonical_json_bytes(landscape))
    args.record.write_bytes(canonical_json_bytes(record))
    print(f"wrote {args.output}")
    print(f"wrote {args.record}")
    print(f"landscape sha256: {record['landscapeSha256']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())