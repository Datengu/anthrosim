#!/usr/bin/env python3
"""Independent post-merge Audit-v3 AV3-007 adversary.

Replays the scientific construction from test-only PR #412 against merged main,
while supplying the now-required finalized-assessment identity fields so the
attack reaches the repaired execution-provenance resolver.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "research-observable-support-results.py"
SPEC = importlib.util.spec_from_file_location("support_results", SCRIPT)
assert SPEC and SPEC.loader
support = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(support)


def canonical_bytes(value):
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
        + "\n"
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
assessment_payload = {
    "schema": "anthrosim-observable-support-assessment-v1",
    "planIdentity": plan_identity,
    "protocolIdentity": "study-protocol-v1-audit-v3-post-merge",
    "studyId": "audit-v3",
    "entries": plan["entries"],
    "sourceStudyExecutionId": "study-execution-v1-audit-v3-post-merge",
    "sourceStudyResultIdentity": "study-result-v1-audit-v3-post-merge",
    "sourceResearchId": "research-execution-v1-audit-v3-post-merge",
}
assessment = dict(assessment_payload)
assessment["assessmentIdentity"] = support.identity(
    "observable-support-assessment-v1-sha256-", assessment_payload
)
assessment = support.validate_assessment(assessment, plan)
plan_entry = plan["entries"][0]

fake_primary = "definitely-not-a-real-analysis-primary"
fake_alternative = "definitely-not-a-real-analysis-alternative"

with tempfile.TemporaryDirectory() as td:
    study_root = Path(td)

    # Recheck the exact fabricated identities independently at the repaired
    # execution resolver. Neither may be treated as evidence of execution.
    for field, binning_id, fake_identity in (
        ("primary", "primary", fake_primary),
        ("alternative", "coarse-phase", fake_alternative),
    ):
        try:
            support.resolve_execution(
                {
                    "binningId": binning_id,
                    "analysisIdentity": fake_identity,
                    "inferenceClass": "supports_h1",
                },
                f"audit.{field}",
                observable_id="occupancy",
                plan_entry=plan_entry,
                assessment=assessment,
                registry={},
                study_root=study_root,
                used_analysis_ids=set(),
            )
        except support.ContractError as exc:
            message = str(exc)
            assert fake_identity in message, message
            assert "does not resolve to an integrity-checked support analysis" in message, message
            print(f"rejected fabricated {field} identity: {fake_identity}")
        else:
            raise AssertionError(
                f"fabricated {field} identity unexpectedly satisfied execution binding"
            )

    # Replay the original report-level attack: same fabricated identities,
    # concordant inference classes, and materialScaleDependence=false, with no
    # corresponding analysis artifacts supplied or resolved.
    report = {
        "schema": "anthrosim-observable-support-sensitivity-report-v1",
        "supportAssessmentIdentity": assessment["assessmentIdentity"],
        "observableResults": [
            {
                "observableId": "occupancy",
                "primary": {
                    "binningId": "primary",
                    "analysisIdentity": fake_primary,
                    "inferenceClass": "supports_h1",
                },
                "alternatives": [
                    {
                        "binningId": "coarse-phase",
                        "analysisIdentity": fake_alternative,
                        "inferenceClass": "supports_h1",
                    }
                ],
                "materialScaleDependence": False,
                "dependenceStatement": None,
            }
        ],
    }
    try:
        support.normalize_report(report, plan, assessment, {}, study_root)
    except support.ContractError as exc:
        message = str(exc)
        assert fake_primary in message, message
        assert "does not resolve to an integrity-checked support analysis" in message, message
        print("original AV3-007 fabricated-analysis report rejected fail-closed")
    else:
        raise AssertionError(
            "AV3-007 reproduced: fabricated analysis identities still certify support execution"
        )

print("AV3-007 post-merge adversary passed: original defect no longer demonstrates")
