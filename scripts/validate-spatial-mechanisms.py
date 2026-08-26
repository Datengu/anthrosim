#!/usr/bin/env python3
"""Validate M8.4 transformed-landscape equivalence and emit a canonical golden record."""

from __future__ import annotations

import argparse
import copy
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


def strip_resume_metadata(value: Any) -> Any:
    """Remove resume-only provenance/integrity metadata before state equivalence comparison."""
    normalized = copy.deepcopy(value)
    if isinstance(normalized, dict):
        normalized.pop("resumeLineage", None)
        normalized.pop("continuationDigest64", None)
        for key, child in list(normalized.items()):
            normalized[key] = strip_resume_metadata(child)
    elif isinstance(normalized, list):
        normalized = [strip_resume_metadata(child) for child in normalized]
    return normalized


def source_identity(artifact: dict[str, Any]) -> dict[str, Any]:
    return {
        "modelVersion": artifact["modelVersion"],
        "modelSemanticsId": artifact["modelSemanticsId"],
        "gitCommit": artifact.get("gitCommit"),
    }


def validate_u64(value: Any, label: str) -> int:
    if type(value) is not int or value < 0 or value > (1 << 64) - 1:
        raise SystemExit(f"{label} is not a valid u64")
    return value


def validate_resume_lineage(uninterrupted: Path, resumed: Path) -> None:
    uninterrupted_manifest = read_json(uninterrupted / "manifest.json")
    uninterrupted_checkpoint = read_json(uninterrupted / "checkpoint.json")
    resumed_manifest = read_json(resumed / "manifest.json")
    resumed_checkpoint = read_json(resumed / "checkpoint.json")
    resumed_metrics = read_json(resumed / "metrics.json")

    empty = {"schemaVersion": 2, "boundaries": []}
    if uninterrupted_manifest.get("resumeLineage") != empty:
        raise SystemExit("uninterrupted manifest unexpectedly contains resume boundaries")
    if uninterrupted_checkpoint.get("resumeLineage") != empty:
        raise SystemExit("uninterrupted checkpoint unexpectedly contains resume boundaries")

    lineage = resumed_manifest.get("resumeLineage")
    if lineage != resumed_checkpoint.get("resumeLineage"):
        raise SystemExit("resumed manifest/checkpoint lineage mismatch")
    if not isinstance(lineage, dict) or lineage.get("schemaVersion") != 2:
        raise SystemExit("resumed lineage schema mismatch")
    boundaries = lineage.get("boundaries")
    if not isinstance(boundaries, list) or len(boundaries) != 1:
        raise SystemExit(f"expected exactly one resume boundary, found {boundaries!r}")

    boundary = boundaries[0]
    identity = source_identity(resumed_manifest)
    if boundary.get("source") != identity or boundary.get("continuation") != identity:
        raise SystemExit("same-revision spatial resume did not preserve exact source identities")

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
    validate_u64(
        boundary.get("sourceContinuationDigest64"),
        "resume boundary source-continuation digest",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--uninterrupted", type=Path, required=True)
    parser.add_argument("--resumed", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    validate_resume_lineage(args.uninterrupted, args.resumed)

    for filename in ARTIFACTS:
        left = args.uninterrupted / filename
        right = args.resumed / filename
        if not left.is_file() or not right.is_file():
            raise SystemExit(f"missing required M8.4 artifact {filename}")
        if strip_resume_metadata(read_json(left)) != strip_resume_metadata(read_json(right)):
            raise SystemExit(f"checkpoint/resume authoritative-state mismatch in {filename}")

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

    resumed_wrapper_manifest = read_json(args.resumed / "landscape-manifest.json")
    resumed_wrapper_checkpoint = read_json(args.resumed / "landscape-checkpoint.json")
    resumed_core_manifest = read_json(args.resumed / "manifest.json")
    resumed_core_checkpoint = read_json(args.resumed / "checkpoint.json")
    if resumed_wrapper_manifest["coreManifest"] != resumed_core_manifest:
        raise SystemExit("resumed spatial landscape manifest does not wrap manifest.json exactly")
    if resumed_wrapper_checkpoint["coreCheckpoint"] != resumed_core_checkpoint:
        raise SystemExit("resumed spatial landscape checkpoint does not wrap checkpoint.json exactly")

    spatial = wrapper_checkpoint["spatial"]
    if spatial["config"] != mechanisms:
        raise SystemExit("stored spatial mechanism config does not match spatial-mechanisms.json")
    if spatial["transformedWorldDigest64"] != core_checkpoint["worldDigest64"]:
        raise SystemExit("spatial binding transformed-world digest does not match core checkpoint")
    if int(core_manifest["world"]["digest64"], 16) != spatial["transformedWorldDigest64"]:
        raise SystemExit("spatial binding transformed-world digest does not match core manifest")

    resumed_spatial = resumed_wrapper_checkpoint["spatial"]
    if resumed_spatial != spatial:
        raise SystemExit("resumed spatial mechanism binding differs from uninterrupted run")
    if resumed_wrapper_manifest["spatial"] != resumed_spatial:
        raise SystemExit("resumed spatial manifest/checkpoint binding mismatch")

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
    print(f"M8.4 transformed checkpoint/resume authoritative equivalence and lineage verified: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
