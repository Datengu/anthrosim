#!/usr/bin/env python3
"""Validate M8.3 uninterrupted/resumed equivalence and emit a canonical golden record."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

CORE_FILES = (
    "world.json",
    "initial-population.json",
    "events.json",
    "metrics.json",
    "manifest.json",
    "checkpoint.json",
    "landscape.json",
    "landscape-manifest.json",
    "landscape-checkpoint.json",
)


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--uninterrupted", type=Path, required=True)
    parser.add_argument("--resumed", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    for filename in CORE_FILES:
        left = args.uninterrupted / filename
        right = args.resumed / filename
        if not left.is_file() or not right.is_file():
            raise SystemExit(f"missing required M8.3 artifact {filename}")
        if read_json(left) != read_json(right):
            raise SystemExit(f"checkpoint/resume mismatch in {filename}")

    landscape_manifest = read_json(args.uninterrupted / "landscape-manifest.json")
    landscape_checkpoint = read_json(args.uninterrupted / "landscape-checkpoint.json")
    binding = landscape_manifest["landscape"]
    if binding != landscape_checkpoint["landscape"]:
        raise SystemExit("landscape manifest/checkpoint binding mismatch")
    if landscape_manifest["coreManifest"] != read_json(args.uninterrupted / "manifest.json"):
        raise SystemExit("landscape manifest does not wrap manifest.json exactly")
    if landscape_checkpoint["coreCheckpoint"] != read_json(args.uninterrupted / "checkpoint.json"):
        raise SystemExit("landscape checkpoint does not wrap checkpoint.json exactly")

    record = {
        "schemaVersion": 1,
        "landscape": binding,
        "coreManifest": landscape_manifest["coreManifest"],
        "coreCheckpoint": landscape_checkpoint["coreCheckpoint"],
        "artifactSha256": {
            name: sha256(args.uninterrupted / name)
            for name in CORE_FILES
        },
    }
    args.output.write_bytes(canonical_bytes(record))
    print(f"M8.3 landscape checkpoint/resume equivalence verified: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
