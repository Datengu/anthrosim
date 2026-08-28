#!/usr/bin/env python3
import copy, hashlib, json, subprocess, sys, tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-observable-support.py")


def canon(v):
    return (json.dumps(v, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def pid(plan):
    return "observable-support-plan-v1-sha256-" + hashlib.sha256(canon(plan)).hexdigest()


def fnv1a64(data):
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def protocol_identity(protocol):
    raw = json.dumps(protocol, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return f"study-protocol-v1-{fnv1a64(raw):016x}"


def base_plan():
    agg = {"source":"spatial-observability.json","operation":"mean","grouping":"declared site polygon","weighting":"equal represented area","missingDataRule":"fail_closed"}
    timeagg = {"source":"metric snapshots","operation":"mean","grouping":"declared phase interval","weighting":"elapsed model days","missingDataRule":"fail_closed"}
    return {
        "schema":"anthrosim-observable-support-plan-v1","planId":"synthetic-support-check",
        "entries":[{
            "id":"settlement-density-support","observableId":"settlementDensity","empirical":True,
            "observedSpatialSupport":{"kind":"site","unit":"site polygon","definition":"survey polygon A","sourceIdentity":"evidence-v1"},
            "observedTemporalSupport":{"kind":"phase","unit":"model-year equivalent","definition":"phase I: 100-year bin","sourceIdentity":"chronology-v1"},
            "simulatedSpatialSupport":{"kind":"cell","unit":"100 m cell","definition":"cells intersecting survey polygon A"},
            "simulatedTemporalSupport":{"kind":"interval","unit":"model day","definition":"all snapshots inside predeclared phase-I window"},
            "simulatedSpatialAggregation":agg,
            "simulatedTemporalAggregation":timeagg,
            "resolutionUncertainty":"uncertain",
            "alternativeBinnings":[{
                "id":"phase-wide",
                "spatialAggregation":agg,
                "temporalAggregation":dict(timeagg, grouping="declared 150-year alternative phase"),
                "rationale":"chronological resolution is uncertain"
            }],
            "dependenceReportingRule":"If substantive inference changes across the declared alternative, report aggregation-scale dependence."
        }]
    }


def base_protocol(plan):
    binding = "observable-support-plan-v1:" + pid(plan)
    return {
      "schemaVersion":1,"protocolRevision":1,"studyId":"support-test","status":"exploratory",
      "researchQuestion":"Does the synthetic pattern resemble the observed pattern at compatible support?",
      "applicabilityDomain":"synthetic regression only","hypotheses":[],
      "analysisWindows":[{"id":"phase1","analysisStartDay":0,"analysisEndDayInclusive":36500,"rationale":"declared synthetic phase"}],
      "observables":[{"id":"settlementDensity","role":"primary","source":"spatial observability","analysisWindowId":"phase1","interpretation":"empirical comparison; " + binding}],
      "comparisons":[],"evidenceRoles":[],
      "uncertainty":{"parameterUncertainty":[],"structuralUncertainty":[]},
      "ensemblePolicy":{"seedPolicy":"fixed","pairingPolicy":"none","replicationPolicy":"fixed"},
      "runHandling":{"stoppingRules":[],"exclusionRules":[],"censoringRules":[]},
      "sensitivityPlan":["aggregation support alternatives are predeclared in the bound support plan"],
      "equifinalityPlan":[],"manipulationChecks":[],"analysisMethod":"support-aware comparison",
      "multiplicityPolicy":"not applicable","heldOutCorroboration":[],
      "permittedInterpretations":["support-compatible comparison"],"prohibitedInterpretations":["raw-cell archaeological fit"]
    }


def run(args, ok=True):
    process = subprocess.run([sys.executable, str(SCRIPT), *args], text=True, capture_output=True)
    if ok and process.returncode != 0:
        raise AssertionError(process.stderr)
    if not ok and process.returncode == 0:
        raise AssertionError("expected failure")
    return process


def main():
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        plan = base_plan()
        protocol = base_protocol(plan)
        plan_path = root / "plan.json"
        protocol_path = root / "protocol.json"
        output = root / "assessment.json"
        plan_path.write_bytes(canon(plan))
        protocol_path.write_bytes(canon(protocol))

        run(["validate", "--plan", str(plan_path), "--protocol", str(protocol_path)])
        run(["derive", "--plan", str(plan_path), "--protocol", str(protocol_path), "--output", str(output)])
        run(["verify", "--plan", str(plan_path), "--protocol", str(protocol_path), "--assessment", str(output)])
        assessment = json.loads(output.read_text())
        assert assessment["entries"][0]["observedSpatialSupport"]["definition"] == "survey polygon A"

        bad = copy.deepcopy(plan)
        del bad["entries"][0]["observedTemporalSupport"]
        (root / "bad.json").write_bytes(canon(bad))
        run(["validate", "--plan", str(root / "bad.json"), "--protocol", str(protocol_path)], ok=False)

        bad = copy.deepcopy(plan)
        bad["entries"][0]["alternativeBinnings"] = []
        (root / "bad.json").write_bytes(canon(bad))
        run(["validate", "--plan", str(root / "bad.json"), "--protocol", str(protocol_path)], ok=False)

        changed = copy.deepcopy(plan)
        changed["entries"][0]["simulatedSpatialAggregation"]["operation"] = "sum"
        (root / "changed.json").write_bytes(canon(changed))
        run(["validate", "--plan", str(root / "changed.json"), "--protocol", str(protocol_path)], ok=False)
        assert pid(changed) != pid(plan)

        result = {
            "studyExecutionId":"study-execution-v1-test",
            "protocolIdentity":protocol_identity(protocol),
            "researchId":"anthrosim-research-v1-test"
        }
        (root / "binding.json").write_bytes(canon(result))
        bound_output = root / "assessment-bound.json"
        run(["derive", "--plan", str(plan_path), "--protocol", str(protocol_path), "--study-result-binding", str(root / "binding.json"), "--output", str(bound_output)])
        bound = json.loads(bound_output.read_text())
        assert bound["sourceResearchId"] == "anthrosim-research-v1-test"

        result["protocolIdentity"] = "study-protocol-v1-wrong"
        (root / "binding-bad.json").write_bytes(canon(result))
        run(["derive", "--plan", str(plan_path), "--protocol", str(protocol_path), "--study-result-binding", str(root / "binding-bad.json"), "--output", str(root / "x.json")], ok=False)

    print("observable support regression suite passed")


if __name__ == "__main__":
    main()
