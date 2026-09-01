#!/usr/bin/env python3
"""Audit-v3 Area I adversary for support-sensitivity execution binding."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "research-observable-support-results.py"
SPEC = importlib.util.spec_from_file_location("support_results", SCRIPT)
assert SPEC and SPEC.loader
support = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(support)


def canonical_bytes(value):
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode()


aggregation = {
    "source": "spatial-observability.json:occupancy",
    "operation": "sum",
    "grouping": "declared site polygon",
    "weighting": "person-days",
    "missingDataRule": "reject missing cells",
}
plan = {
    "schema": "anthrosim-observable-support-plan-v1",
    "planId": "audit-v3-support-execution-binding",
    "entries": [
        {
            "id": "occupancy-support",
            "observableId": "occupancy",
            "empirical": True,
            "observedSpatialSupport": {
                "kind": "polygon",
                "unit": "m2",
                "definition": "survey polygon",
                "sourceIdentity": "evidence-spatial-1",
            },
            "observedTemporalSupport": {
                "kind": "phase",
                "unit": "year",
                "definition": "phase 1",
                "sourceIdentity": "chronology-1",
            },
            "simulatedSpatialSupport": {
                "kind": "cell",
                "unit": "cell",
                "definition": "native transformed landscape cell",
            },
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
plan = support.normalize_plan(plan)
plan_identity = "observable-support-plan-v1-sha256-" + hashlib.sha256(
    canonical_bytes(plan)
).hexdigest()
assessment = {
    "schema": "anthrosim-observable-support-assessment-v1",
    "planIdentity": plan_identity,
    "protocolIdentity": "study-protocol-v1-audit-v3",
    "studyId": "audit-v3",
    "entries": plan["entries"],
    "assessmentIdentity": "observable-support-assessment-v1-sha256-audit-v3",
}
assessment_identity = support.validate_assessment(assessment, plan)

# These values deliberately do not identify any analysis artifact. They are merely
# non-empty strings. The Area-I contract says every alternative binning must actually
# be executed; a machine-enforced report must therefore not be able to establish
# scale robustness from fabricated analysis identities alone.
report = {
    "schema": "anthrosim-observable-support-sensitivity-report-v1",
    "supportAssessmentIdentity": assessment_identity,
    "observableResults": [
        {
            "observableId": "occupancy",
            "primary": {
                "binningId": "primary",
                "analysisIdentity": "definitely-not-a-real-analysis-primary",
                "inferenceClass": "supports_h1",
            },
            "alternatives": [
                {
                    "binningId": "coarse-phase",
                    "analysisIdentity": "definitely-not-a-real-analysis-alternative",
                    "inferenceClass": "supports_h1",
                }
            ],
            "materialScaleDependence": False,
            "dependenceStatement": None,
        }
    ],
}

accepted = support.normalize_report(report, plan, assessment_identity)
print(f"accepted primary identity: {accepted['observableResults'][0]['primary']['analysisIdentity']}")
print(
    "accepted alternative identity: "
    + accepted["observableResults"][0]["alternatives"][0]["analysisIdentity"]
)
print(f"materialScaleDependence: {accepted['observableResults'][0]['materialScaleDependence']}")

assert False, (
    "support-sensitivity reporting accepted fabricated analysis identities and therefore "
    "did not demonstrate that the predeclared primary/alternative binnings were actually executed"
)
