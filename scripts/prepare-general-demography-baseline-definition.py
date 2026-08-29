#!/usr/bin/env python3
"""Materialize the issue #304 exploratory research definition.

The checked-in design intentionally declares semantic uniform layers. This helper
replaces their literal raster payloads with exactly width*height neutral values
before the immutable research definition is consumed. It is deterministic and
must run before simulation; it does not inspect any simulation result.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED = {
    "uniform-traversal": 0,
    "uniform-water": 1000,
    "uniform-productivity": 1000,
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    definition = json.loads(args.source.read_text(encoding="utf-8"))
    landscape = definition["base"]["spatial"]["landscape"]
    width = int(landscape["width"])
    height = int(landscape["height"])
    cells = width * height
    seen: set[str] = set()
    for layer in landscape["layers"]:
        layer_id = layer["layerId"]
        if layer_id not in EXPECTED:
            raise ValueError(f"unexpected issue #304 layer {layer_id!r}")
        layer["values"] = [EXPECTED[layer_id]] * cells
        seen.add(layer_id)
    if seen != set(EXPECTED):
        raise ValueError(f"missing issue #304 layers: {sorted(set(EXPECTED) - seen)}")
    assert all(len(layer["values"]) == cells for layer in landscape["layers"])

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(definition, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"materialized {width}x{height} uniform landscape ({cells} cells per layer)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
