#!/usr/bin/env python3
from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-death-presence.py")


def write(path: Path, value) -> None:
    path.write_text(json.dumps(value), encoding="utf-8")


def run(*args, ok=True):
    result = subprocess.run([sys.executable, str(SCRIPT), *map(str, args)], capture_output=True, text=True)
    if ok and result.returncode != 0:
        raise AssertionError(f"expected success\nstdout={result.stdout}\nstderr={result.stderr}")
    if not ok and result.returncode == 0:
        raise AssertionError(f"expected failure\nstdout={result.stdout}")
    return result


def record(sequence, day, event):
    return {"sequence": sequence, "day": day, "provenance": "authoritative", "event": event}


def fixture():
    return {
        "schemaVersion": 3,
        "events": [
            record(1, 10, {"type": "temporaryJourneyDeparted", "household": 7, "journey": 1, "residence": 2, "destination": 9}),
            record(2, 11, {"type": "death", "person": 100, "household": 7, "cell": 2, "cause": "demographic_mortality"}),
            record(3, 12, {"type": "temporaryJourneyArrived", "household": 7, "journey": 1, "destination": 9}),
            record(4, 13, {"type": "death", "person": 101, "household": 7, "cell": 2, "cause": "condition_mediated"}),
            record(5, 14, {"type": "temporaryReturnDeparted", "household": 7, "journey": 1, "destination": 9, "residence": 2}),
            record(6, 15, {"type": "death", "person": 102, "household": 7, "cell": 2, "cause": "condition_mediated"}),
            record(7, 16, {"type": "temporaryJourneyCompleted", "household": 7, "journey": 1, "residence": 2}),
            record(8, 17, {"type": "death", "person": 103, "household": 7, "cell": 2, "cause": "demographic_mortality"}),
            record(9, 20, {"type": "temporaryJourneyDeparted", "household": 8, "journey": 2, "residence": 4, "destination": 6}),
            record(10, 20, {"type": "death", "person": 200, "household": 8, "cell": 4, "cause": "demographic_mortality"}),
            record(11, 20, {"type": "temporaryJourneyArrived", "household": 8, "journey": 2, "destination": 6}),
            record(12, 20, {"type": "death", "person": 201, "household": 8, "cell": 4, "cause": "demographic_mortality"}),
        ],
    }


def main():
    with tempfile.TemporaryDirectory() as td_raw:
        td = Path(td_raw)
        events = td / "events.json"
        report = td / "report.json"
        write(events, fixture())
        run("derive", "--event-log", events, "--output", report)
        run("verify", "--event-log", events, "--report", report)
        derived = json.loads(report.read_text(encoding="utf-8"))
        deaths = {row["person"]: row for row in derived["deaths"]}
        assert deaths[100]["presenceState"] == "outbound_transit"
        assert deaths[100]["physicalCell"] is None
        assert deaths[100]["resourceProvisioningAttribution"] == "persistent_residence"
        assert deaths[101]["presenceState"] == "visiting"
        assert deaths[101]["physicalCell"] == 9
        assert deaths[101]["resourceProvisioningAttribution"] == "visitor_destination"
        assert deaths[102]["presenceState"] == "return_transit"
        assert deaths[102]["physicalCell"] is None
        assert deaths[103]["presenceState"] == "at_residence"
        assert deaths[103]["physicalCell"] == 2
        # Same-day ordering is sequence-sensitive: death before arrival is transit; after arrival is visiting.
        assert deaths[200]["presenceState"] == "outbound_transit"
        assert deaths[201]["presenceState"] == "visiting"

        broken = fixture()
        broken["events"][2]["event"]["journey"] = 99
        broken_path = td / "broken.json"
        write(broken_path, broken)
        run("derive", "--event-log", broken_path, "--output", td / "broken-report.json", ok=False)

        tampered = copy.deepcopy(derived)
        tampered["deaths"][1]["physicalCell"] = 2
        tampered_path = td / "tampered.json"
        write(tampered_path, tampered)
        run("verify", "--event-log", events, "--report", tampered_path, ok=False)

    print("research death-presence regression suite passed")


if __name__ == "__main__":
    main()
