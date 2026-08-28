#!/usr/bin/env python3

from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "research-archaeological-observation.py"


def base_model() -> dict:
    return {
        "schemaVersion": 1,
        "observationModelId": "synthetic-activity-detection-v1",
        "comparisonId": "fixture-comparison",
        "simulationSource": {
            "id": "fixture-run",
            "kind": "derived_anthrosim_observability",
            "contentSha256": "a" * 64,
            "reference": "fixture://simulation",
        },
        "evidenceSource": {
            "id": "fixture-evidence",
            "kind": "synthetic_verification_fixture",
            "contentSha256": "b" * 64,
            "reference": "fixture://evidence",
        },
        "evidenceRole": "validation",
        "mappings": [
            {
                "mappingId": "activity-to-recovered-count",
                "simulatedVariable": "activityUnits",
                "archaeologicalObservable": "recoveredMaterialCount",
                "relationship": "independent_detection_count",
                "depositionPerMillion": 500000,
                "preservationPerMillion": 800000,
                "samplingPerMillion": 500000,
                "recoveryPerMillion": 500000,
                "assumptions": [
                    "Each simulated activity unit is an exchangeable opportunity to deposit one countable item.",
                    "Detection stages act independently for this verification fixture.",
                ],
                "uncertaintyNote": "Synthetic fixture only; these probabilities are not archaeological estimates.",
            },
            {
                "mappingId": "condition-has-no-direct-proxy",
                "simulatedVariable": "meanCondition",
                "archaeologicalObservable": None,
                "relationship": "no_direct_observable",
                "assumptions": ["No direct archaeological measurement of the model's synthetic condition scalar is asserted."],
                "uncertaintyNote": "A future study would need a separate defensible proxy model before comparison.",
            },
        ],
    }


def base_simulated() -> dict:
    return {
        "schemaVersion": 1,
        "simulationSourceId": "fixture-run",
        "values": {"activityUnits": 100, "meanCondition": 750},
    }


def run_case(model: dict, simulated: dict) -> tuple[subprocess.CompletedProcess[str], dict | None]:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        model_path = root / "model.json"
        simulated_path = root / "simulated.json"
        output_path = root / "result.json"
        model_path.write_text(json.dumps(model), encoding="utf-8")
        simulated_path.write_text(json.dumps(simulated), encoding="utf-8")
        process = subprocess.run(
            [sys.executable, str(SCRIPT), "--model", str(model_path), "--simulated", str(simulated_path), "--output", str(output_path)],
            text=True,
            capture_output=True,
            check=False,
        )
        result = json.loads(output_path.read_text(encoding="utf-8")) if output_path.exists() else None
        return process, result


def main() -> None:
    process, result = run_case(base_model(), base_simulated())
    assert process.returncode == 0, process.stderr
    assert result is not None
    mapped = result["results"][0]
    assert mapped["distribution"] == {
        "family": "binomial",
        "trials": 100,
        "successProbability": {"numerator": 1, "denominator": 10},
    }
    assert mapped["expectedDetectedCount"] == {"numerator": 10, "denominator": 1}
    assert mapped["absenceSemantics"] == "non_detection_possible_after_deposition_preservation_sampling_recovery"
    assert result["results"][1]["status"] == "not_comparable"
    assert result["results"][1]["archaeologicalObservable"] is None
    assert result["observationModelIdentity"].startswith("archaeological-observation-model-v1-")
    assert result["resultIdentity"].startswith("archaeological-observation-result-v1-")

    zero = base_simulated()
    zero["values"]["activityUnits"] = 0
    process, result = run_case(base_model(), zero)
    assert process.returncode == 0, process.stderr
    assert result["results"][0]["absenceSemantics"] == "simulated_absence"

    malformed = base_model()
    del malformed["mappings"][0]["preservationPerMillion"]
    process, _ = run_case(malformed, base_simulated())
    assert process.returncode != 0
    assert "preservationPerMillion" in process.stderr

    unknown = base_model()
    unknown["silentShortcut"] = True
    process, _ = run_case(unknown, base_simulated())
    assert process.returncode != 0
    assert "unknown field" in process.stderr

    mismatch = base_simulated()
    mismatch["simulationSourceId"] = "other-run"
    process, _ = run_case(base_model(), mismatch)
    assert process.returncode != 0
    assert "does not match" in process.stderr

    bad_null = copy.deepcopy(base_model())
    bad_null["mappings"][1]["archaeologicalObservable"] = "inventedProxy"
    process, _ = run_case(bad_null, base_simulated())
    assert process.returncode != 0
    assert "must be null" in process.stderr

    print("archaeological observation-model regression suite passed")


if __name__ == "__main__":
    main()
