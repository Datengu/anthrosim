#!/usr/bin/env python3
"""Validate M8.4 transformed-landscape equivalence and emit a canonical golden record."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

ARTIFACTS = (
    "world.json",
    "initial-population.json",
    "events.json",
    "metrics.json",
    "manifest.json",
    "checkpoint.json",
    "landscape.json",
    "spatial-mechanisms.json",
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

    for filename in ARTIFACTS:
        left = args.uninterrupted / filename
        right = args.resumed / filename
        if not left.is_file() or not right.is_file():
            raise SystemExit(f"missing required M8.4 artifact {filename}")
        if read_json(left) != read_json(right):
            raise SystemExit(f"checkpoint/resume mismatch in {filename}")

    wrapper_manifest = read_json(args.uninterrupted / "landscape-manifest.json")
    wrapper_checkpoint = read_json(args.uninterrupted / "landscape-checkpoint.json")
    mechanisms = read_json(args.uninterrupted / "spatial-mechanisms.json")
    core_manifest = read_json(args.uninterrupted / "manifest.json")
    core_checkpoint = read_json(args.uninterrupted / "checkpoint.json")

    if wrapper_manifest["landscape"] != wrapper_checkpoint["landscape"]:
        raise SystemExit("landscape manifest/checkpoint binding mismatch")
    if wrapper_manifest["spatial"] != wrapper_checkpoint["spatial"]:
        raise SystemExit("spatial mechanism manifest/checkpoint binding mismatch")
    if wrapper_manifest["coreManifest"] != core_manifest:
        raise SystemExit("spatial landscape manifest does not wrap manifest.json exactly")
    if wrapper_checkpoint["coreCheckpoint"] != core_checkpoint:
        raise SystemExit("spatial landscape checkpoint does not wrap checkpoint.json exactly")

    spatial = wrapper_checkpoint["spatial"]
    if spatial["config"] != mechanisms:
        raise SystemExit("stored spatial mechanism config does not match spatial-mechanisms.json")
    if spatial["transformedWorldDigest64"] != core_checkpoint["worldDigest64"]:
        raise SystemExit("spatial binding transformed-world digest does not match core checkpoint")
    if int(core_manifest["world"]["digest64"], 16) != spatial["transformedWorldDigest64"]:
        raise SystemExit("spatial binding transformed-world digest does not match core manifest")

    record = {
        "schemaVersion": 1,
        "landscape": wrapper_checkpoint["landscape"],
        "spatial": spatial,
        "coreManifest": core_manifest,
        "coreCheckpoint": core_checkpoint,
        "artifactSha256": {
            name: sha256(args.uninterrupted / name)
            for name in ARTIFACTS
        },
    }
    args.output.write_bytes(canonical_bytes(record))
    print(f"M8.4 transformed checkpoint/resume equivalence verified: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
