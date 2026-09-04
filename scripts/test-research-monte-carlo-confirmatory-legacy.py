#!/usr/bin/env python3
"""Synthetic seed-binding checks for research-monte-carlo-confirmatory.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ENGINE = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
CONFIRMATORY = ROOT / "scripts" / "research-monte-carlo-confirmatory.py"


def canonical_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def fnv1a64(data):
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def protocol_identity(protocol):
    return f"study-protocol-v1-{fnv1a64(canonical_bytes(protocol)):016x}"


def plan_identity(plan_path: Path) -> str:
    result = subprocess.run(
        [sys.executable, str(ENGINE), "identity", str(plan_path)],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        study = root / "study"
        study.mkdir()
        plan_path = root / "plan.json"
        samples_path = root / "samples.json"
        output_path = root / "diagnostic.json"

        plan = {
            "schemaVersion": 1,
            "planIdentity": "",
            "planId": "confirmatory-seed-binding",
            "uncertaintyCategory": "process_stochastic_monte_carlo",
            "estimand": {"kind": "mean", "confidenceLevel": 0.95, "maxHalfWidth": 20.0},
            "design": {"mode": "fixed", "seedBatches": [[11, 22, 33, 44]]},
            "pairing": "independent",
            "rationale": "Synthetic exact frozen seed binding test.",
        }
        write_json(plan_path, plan)
        plan["planIdentity"] = plan_identity(plan_path)
        write_json(plan_path, plan)

        protocol = {
            "schemaVersion": 1,
            "protocolRevision": 1,
            "studyId": "synthetic-seed-binding",
            "status": "confirmatory",
            "researchQuestion": "Is the planned Monte Carlo sample exactly the executed frozen sample?",
            "applicabilityDomain": "Synthetic verification",
            "hypotheses": [],
            "analysisWindows": [],
            "observables": [],
            "comparisons": [],
            "evidenceRoles": [],
            "uncertainty": {"parameterUncertainty": [], "structuralUncertainty": []},
            "ensemblePolicy": {
                "seedPolicy": "Exact ordered frozen seeds",
                "pairingPolicy": "Independent",
                "replicationPolicy": "monte-carlo-precision-plan-v1:" + plan["planIdentity"],
            },
            "runHandling": {"stoppingRules": [], "exclusionRules": [], "censoringRules": []},
            "sensitivityPlan": [],
            "equifinalityPlan": [],
            "manipulationChecks": [],
            "analysisMethod": "Synthetic",
            "multiplicityPolicy": "One estimand",
            "heldOutCorroboration": [],
            "permittedInterpretations": [],
            "prohibitedInterpretations": [],
        }
        write_json(study / "study-protocol.json", protocol)
        write_json(
            study / "study-result-binding.json",
            {
                "protocolIdentity": protocol_identity(protocol),
                "protocolRevision": 1,
                "studyId": "synthetic-seed-binding",
                "resultIdentity": "synthetic-result",
                "researchId": "synthetic-research",
                "scientificStatus": "confirmatory",
                "boundBeforeExecution": True,
                "confirmatoryPreResultClaimEligible": True,
            },
        )
        write_json(study / "research-definition.json", {"seeds": [11, 22, 33, 44]})
        write_json(
            samples_path,
            {
                "schemaVersion": 1,
                "groups": [
                    {
                        "id": "mean",
                        "replicates": [
                            {"seed": 11, "value": 10.0},
                            {"seed": 22, "value": 11.0},
                            {"seed": 33, "value": 9.0},
                            {"seed": 44, "value": 10.5},
                        ],
                    }
                ],
            },
        )

        accepted = subprocess.run(
            [
                sys.executable,
                str(CONFIRMATORY),
                str(plan_path),
                str(samples_path),
                str(output_path),
                "--study-dir",
                str(study),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert accepted.returncode == 0, accepted.stderr
        diagnostic = json.loads(output_path.read_text(encoding="utf-8"))
        assert diagnostic["seedIdentities"] == [11, 22, 33, 44]

        write_json(study / "research-definition.json", {"seeds": [11, 22, 33, 55]})
        rejected = subprocess.run(
            [
                sys.executable,
                str(CONFIRMATORY),
                str(plan_path),
                str(samples_path),
                str(output_path),
                "--study-dir",
                str(study),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert rejected.returncode == 1
        assert "do not exactly equal" in rejected.stderr

    print("confirmatory Monte Carlo frozen-seed binding suite passed")


if __name__ == "__main__":
    main()
