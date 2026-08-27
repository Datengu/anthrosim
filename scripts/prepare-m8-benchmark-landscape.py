#!/usr/bin/env python3
"""Prepare the fixed public M8.6 terrain benchmark from a Mapzen Skadi HGT tile.

This is preprocessing, not simulation runtime. It uses only the Python standard library so a
third party can regenerate the normalized LandscapeBundle without GDAL once the source HGT tile is
available. The source byte digest is always recorded and may be required with --expected-sha256.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import math
import struct
import urllib.request
from pathlib import Path

SOURCE_URL = "https://elevation-tiles-prod.s3.amazonaws.com/skadi/N46/N46E007.hgt.gz"
SOURCE_TILE = "N46E007.hgt.gz"
WIDTH = 16
HEIGHT = 16
ROW_START = 1300
COL_START = 1300
STRIDE_ARCSECONDS = 20
SOURCE_VOID = -32768
TERRAIN_DOMAIN_MAX = 2500
INPUT_ID = "mapzen_skadi_n46e007"
EVIDENCE_ID = "mapzen_skadi_n46e007_elevation"
GRID_CONVENTION = {
    "originAnchor": "upper_left_outer_corner",
    "columnDirection": "increasing_x",
    "rowDirection": "decreasing_y",
    "cellInterpretation": "area",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--source-cache", type=Path)
    parser.add_argument("--source-url", default=SOURCE_URL)
    parser.add_argument("--expected-sha256")
    return parser.parse_args()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def source_bytes(args: argparse.Namespace) -> bytes:
    if args.source_cache and args.source_cache.is_file():
        return args.source_cache.read_bytes()
    request = urllib.request.Request(args.source_url, headers={"User-Agent": "AnthroSim-M8.6/1"})
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read()
    if args.source_cache:
        args.source_cache.parent.mkdir(parents=True, exist_ok=True)
        args.source_cache.write_bytes(data)
    return data


def decode_hgt(compressed: bytes) -> tuple[bytes, int]:
    raw = gzip.decompress(compressed)
    if len(raw) % 2:
        raise ValueError("HGT payload has an odd byte length")
    sample_count = len(raw) // 2
    side = math.isqrt(sample_count)
    if side * side != sample_count or side < 2:
        raise ValueError(f"HGT payload is not a square sample grid: {sample_count} samples")
    return raw, side


def sample(raw: bytes, side: int, row: int, col: int) -> int:
    if not (0 <= row < side and 0 <= col < side):
        raise ValueError(f"sample outside HGT grid: row={row} col={col} side={side}")
    value = struct.unpack_from(">h", raw, 2 * (row * side + col))[0]
    if value == SOURCE_VOID:
        raise ValueError(f"source HGT void at row={row} col={col}; benchmark forbids implicit fill")
    return value


def patch_elevations(raw: bytes, side: int) -> list[list[int]]:
    final_row = ROW_START + (HEIGHT - 1) * STRIDE_ARCSECONDS
    final_col = COL_START + (WIDTH - 1) * STRIDE_ARCSECONDS
    if final_row >= side or final_col >= side:
        raise ValueError("declared patch exceeds source tile")
    return [
        [
            sample(
                raw,
                side,
                ROW_START + y * STRIDE_ARCSECONDS,
                COL_START + x * STRIDE_ARCSECONDS,
            )
            for x in range(WIDTH)
        ]
        for y in range(HEIGHT)
    ]


def terrain_contrast(elevation: list[list[int]]) -> list[list[int]]:
    result: list[list[int]] = []
    for y in range(HEIGHT):
        row: list[int] = []
        for x in range(WIDTH):
            current = elevation[y][x]
            neighbours = []
            for dx, dy in ((0, -1), (1, 0), (0, 1), (-1, 0)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < WIDTH and 0 <= ny < HEIGHT:
                    neighbours.append(abs(current - elevation[ny][nx]))
            contrast = max(neighbours, default=0)
            if contrast > TERRAIN_DOMAIN_MAX:
                raise ValueError(
                    f"terrain contrast {contrast} exceeds declared domain {TERRAIN_DOMAIN_MAX}; "
                    "do not clip an evidence-grounded benchmark silently"
                )
            row.append(contrast)
        result.append(row)
    return result


def flatten(rows: list[list[int]]) -> list[int]:
    return [value for row in rows for value in row]


def mechanism(model_id: str, target_max: int) -> dict[str, object]:
    return {
        "schemaVersion": 1,
        "modelId": model_id,
        "transforms": [
            {
                "target": "movement_cost",
                "sourceLayerId": "terrain_contrast",
                "expectedUnit": "metre_elevation_difference_per_20_arcsecond_step",
                "sourceDomain": {"min": 0, "max": TERRAIN_DOMAIN_MAX},
                "targetMin": 1000,
                "targetMax": target_max,
                "direction": "direct",
                "nodata": {"kind": "reject"},
            }
        ],
    }


def main() -> None:
    args = parse_args()
    compressed = source_bytes(args)
    sha256 = hashlib.sha256(compressed).hexdigest()
    if args.expected_sha256 and sha256.lower() != args.expected_sha256.lower():
        raise SystemExit(
            f"source SHA-256 mismatch: expected {args.expected_sha256.lower()}, got {sha256}"
        )

    raw, side = decode_hgt(compressed)
    elevation = patch_elevations(raw, side)
    contrast = terrain_contrast(elevation)
    output = args.output_dir
    output.mkdir(parents=True, exist_ok=True)

    # HGT rows run north-to-south. The v2 bundle records that orientation explicitly as the
    # normalized upper-left/increasing-X/decreasing-Y pixel-as-area convention. Coordinates remain
    # integer arcseconds so no floating-point conversion enters the authoritative geometry.
    origin_x = 7 * 3600 + COL_START
    origin_y = 47 * 3600 - ROW_START
    geometry = {
        "originX": origin_x,
        "originY": origin_y,
        "cellSizeX": STRIDE_ARCSECONDS,
        "cellSizeY": STRIDE_ARCSECONDS,
        "coordinateUnit": "arcsecond",
        "spatialReference": "EPSG:4326; integer arcseconds; rows ordered north-to-south",
    }
    landscape = {
        "schemaVersion": 2,
        "width": WIDTH,
        "height": HEIGHT,
        "gridConvention": GRID_CONVENTION,
        "geometry": geometry,
        "layers": [
            {
                "layerId": "terrain_contrast",
                "role": "terrain_traversal",
                "unit": "metre_elevation_difference_per_20_arcsecond_step",
                "valueDomain": {"min": 0, "max": TERRAIN_DOMAIN_MAX},
                "evidenceInputId": INPUT_ID,
                "values": flatten(contrast),
            },
            {
                "layerId": "elevation_m",
                "role": "auxiliary",
                "unit": "metre",
                "evidenceInputId": INPUT_ID,
                "values": flatten(elevation),
            },
        ],
    }
    evidence = {
        "schemaVersion": 1,
        "records": [
            {
                "schemaVersion": 1,
                "evidenceId": EVIDENCE_ID,
                "provenance": "empirical_derived",
                "source": {
                    "sourceId": "mapzen_terrain_tiles_skadi",
                    "citation": (
                        "Mapzen Terrain Tiles, Skadi N46E007, accessed from the AWS Open Data "
                        "Registry; global SRTM terrain data attribution courtesy of the U.S. "
                        "Geological Survey."
                    ),
                    "persistentId": "https://registry.opendata.aws/terrain-tiles/",
                    "datasetVersion": f"content-addressed-sha256:{sha256}",
                    "licence": "Tilezen/joerd source attribution requirements apply",
                    "spatialCoverage": "fixed 16x16 sample patch within N46E007",
                    "temporalCoverage": "elevation source; no historical reconstruction implied",
                },
                "originalVariable": "bare-earth elevation",
                "originalUnits": "metre",
                "transformation": {
                    "method": (
                        "sample fixed HGT cells every 20 arcseconds; derive terrain_contrast as "
                        "maximum absolute elevation difference to sampled N/E/S/W neighbours"
                    ),
                    "sourceUnits": "metre",
                    "simulationUnits": "metre_elevation_difference_per_20_arcsecond_step",
                    "notes": (
                        "No void filling, smoothing, interpolation or outcome-based selection. "
                        "The contrast layer is a terrain proxy, not a calibrated travel-cost surface."
                    ),
                },
                "simulationUnits": "metre_elevation_difference_per_20_arcsecond_step",
                "uncertainty": {
                    "representation": "qualitative",
                    "value": (
                        "Terrain Tiles combine open DEM sources; source resolution/vertical error "
                        "and the terrain-to-movement translation are not calibrated here."
                    ),
                },
                "applicability": (
                    "M8.6 public terrain-only null-model benchmark. Constrains traversal only; "
                    "does not establish historical landscape conditions or archaeological validity."
                ),
                "competingEstimates": [],
            }
        ],
        "externalInputs": [
            {
                "inputId": INPUT_ID,
                "evidenceId": EVIDENCE_ID,
                "format": "Mapzen Skadi gzip-compressed HGT; derived AnthroSim LandscapeBundle v2",
                "spatialReference": "EPSG:4326",
                "contentDigest": f"sha256:{sha256}",
            }
        ],
    }
    provenance = {
        "schemaVersion": 1,
        "sourceUrl": args.source_url,
        "sourceTile": SOURCE_TILE,
        "sourceSha256": sha256,
        "sourceBytes": len(compressed),
        "hgtSideSamples": side,
        "gridConvention": GRID_CONVENTION,
        "patch": {
            "width": WIDTH,
            "height": HEIGHT,
            "rowStart": ROW_START,
            "colStart": COL_START,
            "strideArcseconds": STRIDE_ARCSECONDS,
            "rowOrder": "north_to_south",
        },
        "derived": {
            "terrainContrastMethod": "max_abs_difference_to_sampled_north_east_south_west_neighbour",
            "terrainContrastDomain": [0, TERRAIN_DOMAIN_MAX],
        },
    }

    write_json(output / "landscape.json", landscape)
    write_json(output / "evidence.json", evidence)
    write_json(output / "source-provenance.json", provenance)
    mechanisms = {
        "flat": 1000,
        "weak": 1500,
        "moderate": 2500,
        "strong": 4000,
    }
    for label, target_max in mechanisms.items():
        write_json(
            output / f"spatial-mechanisms-{label}.json",
            mechanism(f"m8_6_terrain_{label}_v1", target_max),
        )

    print(json.dumps(provenance, indent=2))


if __name__ == "__main__":
    main()
