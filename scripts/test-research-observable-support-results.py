#!/usr/bin/env python3
from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-observable-support-results.py")


def canonical_bytes(value):
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode()


def plan_identity(plan):
    return "observable-support-plan-v1-sha256-" + hashlib.sha256(
        canonical_bytes(plan)
    ).hexdigest()


def write(path, value):
    path.write_bytes(canonical_bytes(value))


def run(*args, ok=True):
    result = subprocess.run(
        [sys.executable, str(SCRIPT), *map(str, args)],
        capture_output=True,
        text=True,
    )
    if ok and result.returncode != 0:
        raise AssertionError(
            f"expected success\nstdout={result.stdout}\nstderr={result.stderr}"
        )
    if not ok and result.returncode == 0:
        raise AssertionError(f"expected failure\nstdout={result.stdout}")
    return result


def fixtures():
    aggregation = {
        "source": "spatial-observability.json:occupancy",
        "operation": "sum",
        "grouping": "declared site polygon",
        "weighting": "person-days",
        "missingDataRule": "reject missing cells",
    }
    support = {
        "kind": "cell",
        "unit": "cell",
        "definition": "native transformed landscape cell",
    }
    plan = {
        "schema": "anthrosim-observable-support-plan-v1",
        "planId": "site-occupancy-support",
        "entries": [
            {
                "id": "occupancy-support",
                "observableId": "occupancy",
                "empirical": True,
                "observedSpatialSupport": {
                    "kind": "polygon",
                    "unit": "m2",
                    "definition": "survey polygon A",
                    "sourceIdentity": "evidence-spatial-1",
                },
                "observedTemporalSupport": {
                    "kind": "phase",
                    "unit": "year",
                    "definition": "phase 1",
                    "sourceIdentity": "chronology-1",
                },
                "simulatedSpatialSupport": support,
                "simulatedTemporalSupport": {
                    "kind": "instant",
                    "unit": "day",
                    "definition": "authoritative simulation day",
                },
                "simulatedSpatialAggregation": aggregation,
                "simulatedTemporalAggregation": {
                    "source": "spatial-observability.json:occupancy",
                    "operation": "sum",
                    "grouping": "phase interval",
                    "weighting": "elapsed days",
                    "missingDataRule": "reject missing days",
                },
                "resolutionUncertainty": "uncertain",
                "alternativeBinnings": [
                    {
                        "id": "coarse-phase",
                        "spatialAggregation": aggregation,
                        "temporalAggregation": {
                            "source": "spatial-observability.json:occupancy",
                            "operation": "sum",
                            "grouping": "coarse phase interval",
                            "weighting": "elapsed days",
                            "missingDataRule": "reject missing days",
                        },
                        "rationale": "chronology permits a coarser phase",
                    }
                ],
                "dependenceReportingRule": "report if inference class changes",
            }
        ],
    }
    assessment = {
        "schema": "anthrosim-observable-support-assessment-v1",
        "planIdentity": plan_identity(plan),
        "protocolIdentity": "study-protocol-v1-0123456789abcdef",
        "studyId": "test-study",
        "entries": plan["entries"],
        "assessmentIdentity": "observable-support-assessment-v1-sha256-test",
    }
    declaration = {
        "schema": "anthrosim-observable-support-sensitivity-report-v1",
        "supportAssessmentIdentity": assessment["assessmentIdentity"],
        "observableResults": [
            {
                "observableId": "occupancy",
                "primary": {
                    "binningId": "primary",
                    "analysisIdentity": "analysis-primary",
                    "inferenceClass": "supports_h1",
                },
                "alternatives": [
                    {
                        "binningId": "coarse-phase",
                        "analysisIdentity": "analysis-coarse",
                        "inferenceClass": "supports_h1",
                    }
                ],
                "materialScaleDependence": False,
                "dependenceStatement": None,
            }
        ],
    }
    return plan, assessment, declaration


def main():
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        plan, assessment, declaration = fixtures()
        plan_path = td / "plan.json"
        assessment_path = td / "assessment.json"
        declaration_path = td / "declaration.json"
        report_path = td / "report.json"
        write(plan_path, plan)
        write(assessment_path, assessment)
        write(declaration_path, declaration)

        run(
            "derive",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--declaration",
            declaration_path,
            "--output",
            report_path,
        )
        run(
            "verify",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            report_path,
        )
        first = json.loads(report_path.read_text())
        report_path.unlink()
        run(
            "derive",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--declaration",
            declaration_path,
            "--output",
            report_path,
        )
        assert json.loads(report_path.read_text()) == first

        missing = copy.deepcopy(declaration)
        missing["observableResults"][0]["alternatives"] = []
        write(td / "missing.json", missing)
        run(
            "validate",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            td / "missing.json",
            ok=False,
        )

        changed = copy.deepcopy(declaration)
        changed["observableResults"][0]["alternatives"][0][
            "inferenceClass"
        ] = "does_not_support_h1"
        changed["observableResults"][0]["materialScaleDependence"] = True
        changed["observableResults"][0][
            "dependenceStatement"
        ] = "The substantive inference changes under the coarser chronology."
        write(td / "changed.json", changed)
        run(
            "validate",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            td / "changed.json",
        )

        unreported = copy.deepcopy(changed)
        unreported["observableResults"][0]["dependenceStatement"] = ""
        write(td / "unreported.json", unreported)
        run(
            "validate",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            td / "unreported.json",
            ok=False,
        )

        false_flag = copy.deepcopy(changed)
        false_flag["observableResults"][0]["materialScaleDependence"] = False
        false_flag["observableResults"][0]["dependenceStatement"] = None
        write(td / "false-flag.json", false_flag)
        run(
            "validate",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            td / "false-flag.json",
            ok=False,
        )

        wrong_assessment = copy.deepcopy(declaration)
        wrong_assessment["supportAssessmentIdentity"] = "other-assessment"
        write(td / "wrong-assessment.json", wrong_assessment)
        run(
            "validate",
            "--plan",
            plan_path,
            "--assessment",
            assessment_path,
            "--report",
            td / "wrong-assessment.json",
            ok=False,
        )

    print("research observable-support sensitivity regression suite passed")


if __name__ == "__main__":
    main()
