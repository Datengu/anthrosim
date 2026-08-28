#!/usr/bin/env python3
"""Integrate M9 death-presence reconstruction with preserved run observability."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
from typing import Any

SCHEMA = "anthrosim-m9-death-observability-v1"
DEATH_PRESENCE_FILENAME = "death-presence.json"
INTEGRATED_FILENAME = "m9-death-observability.json"


class ContractError(ValueError):
    pass


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def identity(value: Any) -> str:
    return "m9-death-observability-v1-sha256-" + hashlib.sha256(canonical_bytes(value)).hexdigest()


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot read JSON {path}: {exc}") from exc


def load_death_presence_module() -> Any:
    script = Path(__file__).with_name("research-death-presence.py")
    spec = importlib.util.spec_from_file_location("anthrosim_research_death_presence", script)
    if spec is None or spec.loader is None:
        raise ContractError(f"cannot load death-presence derivation module: {script}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require_object(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ContractError(f"{field} must be an object")
    return value


def require_integer(value: Any, field: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ContractError(f"{field} must be a non-negative integer")
    return value


def require_string(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise ContractError(f"{field} must be a non-empty string")
    return value


def count_by(rows: list[dict[str, Any]], field: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        value = row.get(field)
        if value is None:
            continue
        key = str(value)
        counts[key] = counts.get(key, 0) + 1
    return dict(sorted(counts.items(), key=lambda item: item[0]))


def derive_spatial_cross_check(
    run_dir: Path,
    deaths: list[dict[str, Any]],
    checkpoint: dict[str, Any],
) -> dict[str, Any]:
    path = run_dir / "spatial-observability.json"
    if not path.exists():
        return {"present": False, "residenceDeathCountsMatch": None}

    report = require_object(load_json(path), "spatial-observability.json")
    source = require_object(report.get("source"), "spatial-observability.source")
    if source.get("runStateDigest64") != checkpoint.get("stateDigest64"):
        raise ContractError("spatial-observability.json runStateDigest64 does not match checkpoint.json")
    if source.get("modelSemanticsId") != checkpoint.get("modelSemanticsId"):
        raise ContractError("spatial-observability.json modelSemanticsId does not match checkpoint.json")

    cells = report.get("cells")
    if not isinstance(cells, list):
        raise ContractError("spatial-observability.cells must be an array")
    observed: dict[str, int] = {}
    for index, row in enumerate(cells):
        row = require_object(row, f"spatial-observability.cells[{index}]")
        cell = require_integer(row.get("cell"), f"spatial-observability.cells[{index}].cell")
        derived = require_object(row.get("derived"), f"spatial-observability.cells[{index}].derived")
        count = require_integer(derived.get("deaths"), f"spatial-observability.cells[{index}].derived.deaths")
        if count:
            observed[str(cell)] = count

    expected = count_by(deaths, "persistentResidenceCell")
    if observed != expected:
        raise ContractError(
            "spatial-observability residence-attributed death counts do not match death-presence reconstruction"
        )
    return {
        "present": True,
        "schemaVersion": report.get("schemaVersion"),
        "residenceDeathCountsMatch": True,
    }


def derive_temporary_cross_check(run_dir: Path, checkpoint: dict[str, Any]) -> dict[str, Any]:
    path = run_dir / "temporary-observability.json"
    if not path.exists():
        return {"present": False, "runIdentityMatches": None}

    report = require_object(load_json(path), "temporary-observability.json")
    source = require_object(report.get("source"), "temporary-observability.source")
    if source.get("runStateDigest64") != checkpoint.get("stateDigest64"):
        raise ContractError("temporary-observability.json runStateDigest64 does not match checkpoint.json")
    if source.get("modelSemanticsId") != checkpoint.get("modelSemanticsId"):
        raise ContractError("temporary-observability.json modelSemanticsId does not match checkpoint.json")
    return {
        "present": True,
        "schemaVersion": report.get("schemaVersion"),
        "runIdentityMatches": True,
    }


def derive_run(run_dir: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    events_path = run_dir / "events.json"
    checkpoint_path = run_dir / "checkpoint.json"
    event_log = require_object(load_json(events_path), "events.json")
    checkpoint = require_object(load_json(checkpoint_path), "checkpoint.json")
    if checkpoint.get("events") != event_log:
        raise ContractError("events.json does not match the authoritative event log embedded in checkpoint.json")

    state_digest = require_integer(checkpoint.get("stateDigest64"), "checkpoint.stateDigest64")
    model_version = require_string(checkpoint.get("modelVersion"), "checkpoint.modelVersion")
    model_semantics = require_string(checkpoint.get("modelSemanticsId"), "checkpoint.modelSemanticsId")
    experiment = require_object(checkpoint.get("experiment"), "checkpoint.experiment")
    seed = require_integer(experiment.get("seed"), "checkpoint.experiment.seed")

    module = load_death_presence_module()
    try:
        death_presence = module.derive(event_log)
    except Exception as exc:  # preserve the fail-closed upstream derivation boundary
        raise ContractError(str(exc)) from exc

    deaths_value = death_presence.get("deaths")
    if not isinstance(deaths_value, list):
        raise ContractError("death-presence report deaths must be an array")
    deaths = [require_object(row, f"death-presence.deaths[{index}]") for index, row in enumerate(deaths_value)]

    presence_counts = count_by(deaths, "presenceState")
    residence_counts = count_by(deaths, "persistentResidenceCell")
    physical_counts = count_by(deaths, "physicalCell")
    provisioning_counts = count_by(deaths, "resourceProvisioningAttribution")
    transit_unknown = sum(
        1
        for row in deaths
        if row.get("presenceState") in {"outbound_transit", "return_transit"}
        and row.get("physicalCell") is None
    )

    spatial_check = derive_spatial_cross_check(run_dir, deaths, checkpoint)
    temporary_check = derive_temporary_cross_check(run_dir, checkpoint)

    body = {
        "schema": SCHEMA,
        "source": {
            "modelVersion": model_version,
            "modelSemanticsId": model_semantics,
            "seed": seed,
            "runStateDigest64": state_digest,
            "eventLogSchemaVersion": death_presence.get("sourceEventLogSchemaVersion"),
            "deathPresenceReport": DEATH_PRESENCE_FILENAME,
            "deathPresenceReportIdentity": death_presence.get("reportIdentity"),
        },
        "semantics": {
            "persistentResidenceCountsArePhysicalDeathLocations": False,
            "physicalCellAvailableFor": ["at_residence", "visiting"],
            "transitPhysicalCellIsUnknown": True,
            "resourceProvisioningAttributionIsPresenceContextNotMortalityCause": True,
        },
        "summary": {
            "deaths": len(deaths),
            "byPresenceState": presence_counts,
            "byPersistentResidenceCell": residence_counts,
            "byPhysicalCell": physical_counts,
            "transitDeathsWithoutPhysicalCell": transit_unknown,
            "byResourceProvisioningAttribution": provisioning_counts,
        },
        "crossChecks": {
            "spatialObservability": spatial_check,
            "temporaryObservability": temporary_check,
        },
    }
    integrated = {**body, "reportIdentity": identity(body)}
    return death_presence, integrated


def write_new_or_equal(path: Path, value: Any) -> None:
    if path.exists():
        if load_json(path) == value:
            return
        raise ContractError(f"refusing to overwrite differing existing output: {path}")
    path.write_bytes(canonical_bytes(value))


def derive_command(run_dir: Path) -> dict[str, Any]:
    death_presence, integrated = derive_run(run_dir)
    write_new_or_equal(run_dir / DEATH_PRESENCE_FILENAME, death_presence)
    write_new_or_equal(run_dir / INTEGRATED_FILENAME, integrated)
    return integrated


def verify_command(run_dir: Path) -> dict[str, Any]:
    expected_death_presence, expected_integrated = derive_run(run_dir)
    death_path = run_dir / DEATH_PRESENCE_FILENAME
    integrated_path = run_dir / INTEGRATED_FILENAME
    if load_json(death_path) != expected_death_presence:
        raise ContractError("death-presence.json does not match deterministic re-derivation")
    if load_json(integrated_path) != expected_integrated:
        raise ContractError("m9-death-observability.json does not match deterministic re-derivation")
    return expected_integrated


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    for name in ("derive", "verify"):
        command = sub.add_parser(name)
        command.add_argument("--run-dir", required=True, type=Path)
    args = parser.parse_args()
    try:
        report = derive_command(args.run_dir) if args.cmd == "derive" else verify_command(args.run_dir)
        print(report["reportIdentity"])
        return 0
    except ContractError as exc:
        parser.error(str(exc))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
