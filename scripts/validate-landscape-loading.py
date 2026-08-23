#!/usr/bin/env python3
"""Validate M8.3 uninterrupted/resumed equivalence and emit a canonical golden record."""

from __future__ import annotations

import argparse
import copy
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


def strip_resume_lineage(value: Any) -> Any:
    """Remove only resume provenance before authoritative-state equivalence comparison."""
    normalized = copy.deepcopy(value)
    if isinstance(normalized, dict):
        normalized.pop("resumeLineage", None)
        for key, child in list(normalized.items()):
            normalized[key] = strip_resume_lineage(child)
    elif isinstance(normalized, list):
        normalized = [strip_resume_lineage(child) for child in normalized]
    return normalized


def source_identity(artifact: dict[str, Any]) -> dict[str, Any]:
    return {
        "modelVersion": artifact["modelVersion"],
        "modelSemanticsId": artifact["modelSemanticsId"],
        "gitCommit": artifact.get("gitCommit"),
    }


def validate_resume_lineage(uninterrupted: Path, resumed: Path) -> None:
    uninterrupted_manifest = read_json(uninterrupted / "manifest.json")
    uninterrupted_checkpoint = read_json(uninterrupted / "checkpoint.json")
    resumed_manifest = read_json(resumed / "manifest.json")
    resumed_checkpoint = read_json(resumed / "checkpoint.json")
    resumed_metrics = read_json(resumed / "metrics.json")

    empty = {"schemaVersion": 1, "boundaries": []}
    if uninterrupted_manifest.get("resumeLineage") != empty:
        raise SystemExit("uninterrupted manifest unexpectedly contains resume boundaries")
    if uninterrupted_checkpoint.get("resumeLineage") != empty:
        raise SystemExit("uninterrupted checkpoint unexpectedly contains resume boundaries")

    lineage = resumed_manifest.get("resumeLineage")
    if lineage != resumed_checkpoint.get("resumeLineage"):
        raise SystemExit("resumed manifest/checkpoint lineage mismatch")
    if not isinstance(lineage, dict) or lineage.get("schemaVersion") != 1:
        raise SystemExit("resumed lineage schema mismatch")
    boundaries = lineage.get("boundaries")
    if not isinstance(boundaries, list) or len(boundaries) != 1:
        raise SystemExit(f"expected exactly one resume boundary, found {boundaries!r}")

    boundary = boundaries[0]
    identity = source_identity(resumed_manifest)
    if boundary.get("source") != identity or boundary.get("continuation") != identity:
        raise SystemExit("same-revision landscape resume did not preserve exact source identities")

    boundary_day = boundary.get("boundaryDay")
    boundary_years = boundary.get("boundaryCompletedYears")
    if (
        not isinstance(boundary_day, int)
        or not isinstance(boundary_years, int)
        or boundary_day != boundary_years * 365
        or boundary_day >= resumed_manifest["endTime"]
    ):
        raise SystemExit("resume boundary time is inconsistent with the completed run")

    boundary_snapshot = next(
        (snapshot for snapshot in resumed_metrics["snapshots"] if snapshot["day"] == boundary_day),
        None,
    )
    if boundary_snapshot is None:
        raise SystemExit("resume boundary has no matching metric-state snapshot")
    if boundary.get("sourceStateDigest64") != boundary_snapshot["stateDigest64"]:
        raise SystemExit("resume boundary source-state digest does not match boundary state")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--uninterrupted", type=Path, required=True)
    parser.add_argument("--resumed", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    validate_resume_lineage(args.uninterrupted, args.resumed)

    for filename in CORE_FILES:
        left = args.uninterrupted / filename
        right = args.resumed / filename
        if not left.is_file() or not right.is_file():
            raise SystemExit(f"missing required M8.3 artifact {filename}")
        if strip_resume_lineage(read_json(left)) != strip_resume_lineage(read_json(right)):
            raise SystemExit(f"checkpoint/resume authoritative-state mismatch in {filename}")

    landscape_manifest = read_json(args.uninterrupted / "landscape-manifest.json")
    landscape_checkpoint = read_json(args.uninterrupted / "landscape-checkpoint.json")
    binding = landscape_manifest["landscape"]
    if binding != landscape_checkpoint["landscape"]:
        raise SystemExit("landscape manifest/checkpoint binding mismatch")
    if landscape_manifest["coreManifest"] != read_json(args.uninterrupted / "manifest.json"):
        raise SystemExit("landscape manifest does not wrap manifest.json exactly")
    if landscape_checkpoint["coreCheckpoint"] != read_json(args.uninterrupted / "checkpoint.json"):
        raise SystemExit("landscape checkpoint does not wrap checkpoint.json exactly")

    resumed_landscape_manifest = read_json(args.resumed / "landscape-manifest.json")
    resumed_landscape_checkpoint = read_json(args.resumed / "landscape-checkpoint.json")
    if resumed_landscape_manifest["coreManifest"] != read_json(args.resumed / "manifest.json"):
        raise SystemExit("resumed landscape manifest does not wrap manifest.json exactly")
    if resumed_landscape_checkpoint["coreCheckpoint"] != read_json(args.resumed / "checkpoint.json"):
        raise SystemExit("resumed landscape checkpoint does not wrap checkpoint.json exactly")

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
    print(f"M8.3 landscape checkpoint/resume authoritative equivalence and lineage verified: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())