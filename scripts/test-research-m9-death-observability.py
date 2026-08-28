#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-m9-death-observability.py")
spec = importlib.util.spec_from_file_location("research_m9_death_observability", SCRIPT)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def event(sequence: int, day: int, payload: dict[str, object]) -> dict[str, object]:
    return {
        "sequence": sequence,
        "day": day,
        "provenance": "authoritative",
        "event": payload,
    }


def fixture(root: Path) -> None:
    events = {
        "schemaVersion": 4,
        "events": [
            event(1, 10, {
                "type": "temporaryJourneyDeparted",
                "household": 1,
                "journey": 1,
                "residence": 1,
                "destination": 2,
            }),
            event(2, 11, {
                "type": "death",
                "person": 1,
                "household": 1,
                "cell": 1,
                "cause": "background",
            }),
            event(3, 12, {
                "type": "temporaryJourneyArrived",
                "household": 1,
                "journey": 1,
                "destination": 2,
            }),
            event(4, 13, {
                "type": "death",
                "person": 2,
                "household": 1,
                "cell": 1,
                "cause": "condition_mediated",
            }),
            event(5, 14, {
                "type": "temporaryReturnDeparted",
                "household": 1,
                "journey": 1,
            }),
            event(6, 15, {
                "type": "death",
                "person": 3,
                "household": 1,
                "cell": 1,
                "cause": "background",
            }),
            event(7, 16, {
                "type": "temporaryJourneyCompleted",
                "household": 1,
                "journey": 1,
                "residence": 1,
            }),
            event(8, 17, {
                "type": "death",
                "person": 4,
                "household": 1,
                "cell": 1,
                "cause": "background",
            }),
        ],
    }
    checkpoint = {
        "modelVersion": "0.3.0",
        "modelSemanticsId": "anthrosim-model-semantics-v16",
        "stateDigest64": 123456,
        "experiment": {"seed": 77},
        "events": events,
    }
    spatial = {
        "schemaVersion": 4,
        "source": {
            "runStateDigest64": 123456,
            "modelSemanticsId": "anthrosim-model-semantics-v16",
        },
        "cells": [
            {"cell": 1, "derived": {"deaths": 4}},
            {"cell": 2, "derived": {"deaths": 0}},
        ],
    }
    temporary = {
        "schemaVersion": 2,
        "source": {
            "runStateDigest64": 123456,
            "modelSemanticsId": "anthrosim-model-semantics-v16",
        },
    }
    write_json(root / "events.json", events)
    write_json(root / "checkpoint.json", checkpoint)
    write_json(root / "spatial-observability.json", spatial)
    write_json(root / "temporary-observability.json", temporary)


def expect_error(fn, fragment: str) -> None:
    try:
        fn()
    except module.ContractError as exc:
        assert fragment in str(exc), str(exc)
    else:
        raise AssertionError(f"expected ContractError containing {fragment!r}")


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-m9-death-observability-") as directory:
        root = Path(directory)
        fixture(root)
        report = module.derive_command(root)
        assert module.verify_command(root) == report
        summary = report["summary"]
        assert summary["deaths"] == 4
        assert summary["byPresenceState"] == {
            "at_residence": 1,
            "outbound_transit": 1,
            "return_transit": 1,
            "visiting": 1,
        }
        assert summary["byPersistentResidenceCell"] == {"1": 4}
        assert summary["byPhysicalCell"] == {"1": 1, "2": 1}
        assert summary["transitDeathsWithoutPhysicalCell"] == 2
        assert summary["byResourceProvisioningAttribution"] == {
            "persistent_residence": 3,
            "visitor_destination": 1,
        }
        assert report["crossChecks"]["spatialObservability"]["residenceDeathCountsMatch"] is True
        assert report["crossChecks"]["temporaryObservability"]["runIdentityMatches"] is True

        integrated_path = root / module.INTEGRATED_FILENAME
        tampered = json.loads(integrated_path.read_text(encoding="utf-8"))
        tampered["summary"]["deaths"] += 1
        write_json(integrated_path, tampered)
        expect_error(lambda: module.verify_command(root), "does not match deterministic re-derivation")

    with tempfile.TemporaryDirectory(prefix="anthrosim-m9-death-observability-spatial-") as directory:
        root = Path(directory)
        fixture(root)
        spatial_path = root / "spatial-observability.json"
        spatial = json.loads(spatial_path.read_text(encoding="utf-8"))
        spatial["cells"][0]["derived"]["deaths"] = 3
        write_json(spatial_path, spatial)
        expect_error(lambda: module.derive_run(root), "residence-attributed death counts")

    with tempfile.TemporaryDirectory(prefix="anthrosim-m9-death-observability-events-") as directory:
        root = Path(directory)
        fixture(root)
        checkpoint_path = root / "checkpoint.json"
        checkpoint = json.loads(checkpoint_path.read_text(encoding="utf-8"))
        checkpoint["events"]["events"] = checkpoint["events"]["events"][:-1]
        write_json(checkpoint_path, checkpoint)
        expect_error(lambda: module.derive_run(root), "events.json does not match")

    print("research M9 death-observability regression suite passed")


if __name__ == "__main__":
    main()
