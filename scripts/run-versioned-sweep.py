#!/usr/bin/env python3
"""Launch an AnthroSim sweep from a versioned JSON definition.

This adapter does not implement simulation or aggregation. It translates a
reviewable definition file into the ordinary `anthrosim sweep` CLI, then checks
that the resulting immutable sweep manifest records the same exact seeds,
settings and dimensions.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

BASE_ARGUMENTS = {
    "years": "--years",
    "worldWidth": "--world-width",
    "worldHeight": "--world-height",
    "population": "--population",
    "householdSize": "--household-size",
    "maxPersonRecords": "--max-person-records",
    "resourceProductivityScalePermille": "--resource-productivity-scale-permille",
    "resourceSeasonalityScalePermille": "--resource-seasonality-scale-permille",
    "annualFoodNeed": "--annual-food-need",
    "disableMigration": "--disable-migration",
    "migrationRadius": "--migration-radius",
}

DIMENSION_ARGUMENTS = {
    "population": "--sweep-population",
    "householdSize": "--sweep-household-size",
    "resourceProductivityScalePermille": "--sweep-resource-productivity-scale-permille",
    "resourceSeasonalityScalePermille": "--sweep-resource-seasonality-scale-permille",
    "annualFoodNeed": "--sweep-annual-food-need",
    "disableMigration": "--sweep-disable-migration",
    "migrationRadius": "--sweep-migration-radius",
}

MANIFEST_BASE_KEYS = {
    "years": "years",
    "worldWidth": "worldWidth",
    "worldHeight": "worldHeight",
    "population": "population",
    "householdSize": "householdSize",
    "maxPersonRecords": "maxPersonRecords",
    "resourceProductivityScalePermille": "resourceProductivityScalePermille",
    "resourceSeasonalityScalePermille": "resourceSeasonalityScalePermille",
    "annualFoodNeed": "annualFoodNeed",
    "disableMigration": "disableMigration",
    "migrationRadius": "migrationRadius",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("definition", type=Path)
    parser.add_argument("--binary", type=Path, default=Path("target/release/anthrosim"))
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--retry", action="store_true")
    return parser.parse_args()


def scalar(value: object) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


def load_definition(path: Path) -> tuple[dict, bytes]:
    raw = path.read_bytes()
    definition = json.loads(raw)
    if definition.get("schemaVersion") != 1:
        raise SystemExit("unsupported versioned sweep definition schema")
    if definition.get("scientificStatus") != "synthetic_validation":
        raise SystemExit("v0.1 versioned sweep definitions must declare synthetic_validation status")
    if not definition.get("definitionId"):
        raise SystemExit("versioned sweep definition requires definitionId")
    seeds = definition.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        raise SystemExit("versioned sweep definition requires a non-empty seeds array")
    if len(set(seeds)) != len(seeds):
        raise SystemExit("versioned sweep definition contains duplicate seeds")
    base = definition.get("base")
    if not isinstance(base, dict):
        raise SystemExit("versioned sweep definition requires base settings")
    dimensions = definition.get("dimensions")
    if not isinstance(dimensions, dict) or not dimensions:
        raise SystemExit("versioned sweep definition requires at least one dimension")
    unknown_base = set(base) - set(BASE_ARGUMENTS)
    unknown_dimensions = set(dimensions) - set(DIMENSION_ARGUMENTS)
    if unknown_base:
        raise SystemExit(f"unsupported base setting(s): {sorted(unknown_base)}")
    if unknown_dimensions:
        raise SystemExit(f"unsupported sweep dimension(s): {sorted(unknown_dimensions)}")
    for name, values in dimensions.items():
        if not isinstance(values, list) or not values:
            raise SystemExit(f"sweep dimension {name} must be a non-empty array")
        if len({scalar(value) for value in values}) != len(values):
            raise SystemExit(f"sweep dimension {name} contains duplicate values")
    return definition, raw


def build_command(definition: dict, binary: Path, run_dir: Path, retry: bool) -> list[str]:
    command = [str(binary.resolve()), "sweep"]
    for key, flag in BASE_ARGUMENTS.items():
        if key not in definition["base"]:
            continue
        value = definition["base"][key]
        if isinstance(value, bool):
            if value:
                command.append(flag)
        else:
            command.extend([flag, scalar(value)])
    command.extend(["--seeds", ",".join(str(seed) for seed in definition["seeds"])])
    for key, flag in DIMENSION_ARGUMENTS.items():
        values = definition["dimensions"].get(key)
        if values is not None:
            command.extend([flag, ",".join(scalar(value) for value in values)])
    command.extend(["--run-dir", str(run_dir.resolve())])
    if retry:
        command.append("--retry")
    return command


def expected_manifest_definition(definition: dict) -> tuple[dict, dict]:
    base = {
        manifest_key: definition["base"][source_key]
        for source_key, manifest_key in MANIFEST_BASE_KEYS.items()
        if source_key in definition["base"]
    }
    defaults = {
        "years": 1000,
        "worldWidth": 128,
        "worldHeight": 128,
        "population": 10000,
        "householdSize": 5,
        "maxPersonRecords": 1000000,
        "resourceProductivityScalePermille": 1000,
        "resourceSeasonalityScalePermille": 1000,
        "annualFoodNeed": 100,
        "disableMigration": False,
        "migrationRadius": 3,
    }
    defaults.update(base)
    dimensions = {key: definition["dimensions"].get(key, []) for key in DIMENSION_ARGUMENTS}
    return defaults, dimensions


def verify_manifest(definition: dict, run_dir: Path) -> dict:
    manifest_path = run_dir / "sweep-manifest.json"
    if not manifest_path.is_file():
        raise SystemExit("sweep completed without immutable sweep-manifest.json")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_definition = manifest.get("definition", {})
    expected_base, expected_dimensions = expected_manifest_definition(definition)
    if manifest_definition.get("seeds") != definition["seeds"]:
        raise SystemExit("immutable sweep manifest seeds do not match source definition")
    if manifest_definition.get("baseSettings") != expected_base:
        raise SystemExit("immutable sweep manifest base settings do not match source definition")
    if manifest_definition.get("dimensions") != expected_dimensions:
        raise SystemExit("immutable sweep manifest dimensions do not match source definition")
    return manifest


def is_dirty_source_identity(revision: str) -> bool:
    return revision.endswith("-dirty") or "-dirty-" in revision


def inspect_binary_provenance(binary: Path) -> dict:
    completed = subprocess.run(
        [str(binary), "provenance"],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "no diagnostic output"
        raise SystemExit(
            f"could not inspect AnthroSim binary provenance before sweep execution: {detail}"
        )
    try:
        provenance = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(
            "AnthroSim binary provenance command did not return valid JSON"
        ) from error
    if not isinstance(provenance, dict):
        raise SystemExit("AnthroSim binary provenance command returned a non-object JSON value")
    if not provenance.get("modelVersion") or not provenance.get("modelSemanticsId"):
        raise SystemExit("AnthroSim binary provenance is missing model identity fields")
    return provenance


def require_reproducible_source_identity(manifest: dict) -> str:
    revision = manifest.get("gitCommit")
    if not revision:
        raise SystemExit(
            "versioned research sweep requires an exact Git source identity; rebuild inside a Git checkout or provide ANTHROSIM_GIT_COMMIT explicitly"
        )
    if is_dirty_source_identity(revision):
        raise SystemExit(
            "versioned research sweep refuses a dirty tracked source tree; commit/stash tracked changes or use a controlled explicit ANTHROSIM_GIT_COMMIT override"
        )
    return revision


def main() -> int:
    args = parse_args()
    definition_path = args.definition.resolve()
    definition, raw = load_definition(definition_path)
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"AnthroSim binary not found: {binary}")

    binary_provenance = inspect_binary_provenance(binary)
    source_revision = require_reproducible_source_identity(binary_provenance)

    command = build_command(definition, binary, args.run_dir, args.retry)
    completed = subprocess.run(command, check=False)
    if completed.returncode != 0:
        return completed.returncode

    manifest = verify_manifest(definition, args.run_dir)
    manifest_revision = require_reproducible_source_identity(manifest)
    if manifest.get("modelVersion") != binary_provenance["modelVersion"]:
        raise SystemExit(
            "completed sweep manifest modelVersion does not match the preflighted AnthroSim binary"
        )
    if manifest_revision != source_revision:
        raise SystemExit(
            "completed sweep manifest gitCommit does not match the preflighted AnthroSim binary"
        )
    source_hash = hashlib.sha256(raw).hexdigest()
    source_copy = args.run_dir / "source-definition.json"
    if args.retry and source_copy.is_file():
        existing_hash = hashlib.sha256(source_copy.read_bytes()).hexdigest()
        if existing_hash != source_hash:
            raise SystemExit("retry source-definition.json does not match requested definition")
    else:
        shutil.copyfile(definition_path, source_copy)

    record = {
        "schemaVersion": 1,
        "definitionId": definition["definitionId"],
        "scientificStatus": definition["scientificStatus"],
        "definitionSha256": source_hash,
        "modelVersion": manifest["modelVersion"],
        "gitCommit": source_revision,
        "sourceDefinition": "source-definition.json",
        "sweepManifest": "sweep-manifest.json",
        "sweepId": manifest["sweepId"],
        "analysisDirectory": "analysis",
        "note": "The source definition launches the ordinary AnthroSim sweep path. The immutable sweep manifest is authoritative for exact expanded point/run provenance.",
    }
    (args.run_dir / "reproduction-record.json").write_text(
        json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps(record, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
