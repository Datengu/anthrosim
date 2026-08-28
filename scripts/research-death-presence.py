#!/usr/bin/env python3
"""Derive fail-closed death-time physical-presence context from authoritative event history."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

REPORT_SCHEMA = "anthrosim-death-presence-report-v1"


class ContractError(ValueError):
    pass


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def identity(value: Any) -> str:
    return "death-presence-report-v1-sha256-" + hashlib.sha256(canonical_bytes(value)).hexdigest()


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot read JSON {path}: {exc}") from exc


def nonempty(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ContractError(f"{field} must be a non-empty string")
    return value


def integer(value: Any, field: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ContractError(f"{field} must be a non-negative integer")
    return value


def active_presence(state: dict[str, Any] | None) -> dict[str, Any]:
    if state is None:
        return {
            "presenceState": "at_residence",
            "journey": None,
            "destinationCell": None,
            "physicalCell": None,
            "resourceProvisioningAttribution": "persistent_residence",
        }
    phase = state["phase"]
    visiting = phase == "visiting"
    return {
        "presenceState": phase,
        "journey": state["journey"],
        "destinationCell": state["destination"],
        "physicalCell": state["destination"] if visiting else None,
        "resourceProvisioningAttribution": (
            "visitor_destination" if visiting else "persistent_residence"
        ),
    }


def derive(event_log: Any) -> dict[str, Any]:
    if not isinstance(event_log, dict):
        raise ContractError("event log must be an object")
    schema = integer(event_log.get("schemaVersion"), "eventLog.schemaVersion")
    events = event_log.get("events")
    if not isinstance(events, list):
        raise ContractError("eventLog.events must be an array")

    presence: dict[int, dict[str, Any]] = {}
    residence: dict[int, int] = {}
    deaths: list[dict[str, Any]] = []
    previous_day = 0

    for index, record in enumerate(events):
        field = f"eventLog.events[{index}]"
        if not isinstance(record, dict):
            raise ContractError(f"{field} must be an object")
        if integer(record.get("sequence"), f"{field}.sequence") != index + 1:
            raise ContractError(f"{field}.sequence must be canonical and contiguous")
        day = integer(record.get("day"), f"{field}.day")
        if index and day < previous_day:
            raise ContractError("event days must be non-decreasing")
        previous_day = day
        if record.get("provenance") != "authoritative":
            raise ContractError(f"{field}.provenance must be authoritative")
        event = record.get("event")
        if not isinstance(event, dict):
            raise ContractError(f"{field}.event must be an object")
        kind = nonempty(event.get("type"), f"{field}.event.type")

        if kind == "householdMigration":
            household = integer(event.get("household"), f"{field}.event.household")
            if household in presence:
                raise ContractError("permanent migration cannot occur during an active M9 journey")
            residence[household] = integer(event.get("destination"), f"{field}.event.destination")
            continue

        if kind == "temporaryJourneyDeparted":
            household = integer(event.get("household"), f"{field}.event.household")
            if household in presence:
                raise ContractError(f"household {household} departed with an active journey")
            event_residence = integer(event.get("residence"), f"{field}.event.residence")
            known_residence = residence.get(household)
            if known_residence is not None and known_residence != event_residence:
                raise ContractError(f"household {household} departure residence conflicts with history")
            residence[household] = event_residence
            presence[household] = {
                "phase": "outbound_transit",
                "journey": integer(event.get("journey"), f"{field}.event.journey"),
                "destination": integer(event.get("destination"), f"{field}.event.destination"),
            }
            continue

        if kind in {"temporaryJourneyArrived", "temporaryReturnDeparted", "temporaryJourneyCompleted"}:
            household = integer(event.get("household"), f"{field}.event.household")
            state = presence.get(household)
            if state is None:
                raise ContractError(f"household {household} has an M9 transition without an active journey")
            journey = integer(event.get("journey"), f"{field}.event.journey")
            if journey != state["journey"]:
                raise ContractError(f"household {household} M9 journey identity changed mid-journey")
            if kind == "temporaryJourneyArrived":
                if state["phase"] != "outbound_transit":
                    raise ContractError("arrival requires outbound transit")
                destination = integer(event.get("destination"), f"{field}.event.destination")
                if destination != state["destination"]:
                    raise ContractError("arrival destination conflicts with departure")
                state["phase"] = "visiting"
            elif kind == "temporaryReturnDeparted":
                if state["phase"] != "visiting":
                    raise ContractError("return departure requires visiting state")
                state["phase"] = "return_transit"
            else:
                if state["phase"] != "return_transit":
                    raise ContractError("journey completion requires return transit")
                event_residence = integer(event.get("residence"), f"{field}.event.residence")
                known_residence = residence.get(household)
                if known_residence is not None and event_residence != known_residence:
                    raise ContractError("journey completion residence conflicts with persistent residence")
                residence[household] = event_residence
                del presence[household]
            continue

        if kind == "death":
            household = integer(event.get("household"), f"{field}.event.household")
            event_residence = integer(event.get("cell"), f"{field}.event.cell")
            known_residence = residence.get(household)
            if known_residence is not None and known_residence != event_residence:
                raise ContractError(f"death residence conflicts with household {household} history")
            residence[household] = event_residence
            context = active_presence(presence.get(household))
            if context["presenceState"] == "at_residence":
                context["physicalCell"] = event_residence
            deaths.append(
                {
                    "sequence": index + 1,
                    "day": day,
                    "person": integer(event.get("person"), f"{field}.event.person"),
                    "household": household,
                    "persistentResidenceCell": event_residence,
                    "cause": nonempty(event.get("cause"), f"{field}.event.cause"),
                    **context,
                }
            )

    body = {
        "schema": REPORT_SCHEMA,
        "sourceEventLogSchemaVersion": schema,
        "interpretation": {
            "persistentResidenceIsPhysicalLocationOnlyWhenPresenceState": ["at_residence"],
            "visitingPhysicalCell": "destinationCell",
            "transitPhysicalCell": None,
            "resourceProvisioningAttributionIsPresenceContextNotMortalityCause": True,
        },
        "deaths": deaths,
    }
    return {**body, "reportIdentity": identity(body)}


def write_new(path: Path, value: Any) -> None:
    if path.exists():
        if load_json(path) == value:
            return
        raise ContractError(f"refusing to overwrite differing existing output: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_bytes(value))


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    derive_cmd = sub.add_parser("derive")
    derive_cmd.add_argument("--event-log", required=True, type=Path)
    derive_cmd.add_argument("--output", required=True, type=Path)
    verify_cmd = sub.add_parser("verify")
    verify_cmd.add_argument("--event-log", required=True, type=Path)
    verify_cmd.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    try:
        expected = derive(load_json(args.event_log))
        if args.cmd == "derive":
            write_new(args.output, expected)
        elif load_json(args.report) != expected:
            raise ContractError("report does not match deterministic re-derivation")
        print(expected["reportIdentity"])
        return 0
    except ContractError as exc:
        parser.error(str(exc))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
