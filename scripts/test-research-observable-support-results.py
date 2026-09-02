#!/usr/bin/env python3
from __future__ import annotations

import copy
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-observable-support-results.py")
PROVENANCE = Path(__file__).with_name("research-analysis-provenance.py")


def canonical_bytes(value):
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode()


def plan_identity(plan):
    return "observable-support-plan-v1-sha256-" + hashlib.sha256(
        canonical_bytes(plan)
    ).hexdigest()


def assessment_identity(assessment):
    payload = dict(assessment)
    payload.pop("assessmentIdentity", None)
    return "observable-support-assessment-v1-sha256-" + hashlib.sha256(
        canonical_bytes(payload)
    ).hexdigest()


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_bytes(value))


def run_script(script, *args, ok=True):
    result = subprocess.run(
        [sys.executable, str(script), *map(str, args)],
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


def run(*args, ok=True):
    return run_script(SCRIPT, *args, ok=ok)


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
    binding = {
        "schemaVersion": 1,
        "resultIdentity": "study-result-v1-support-test",
        "studyExecutionId": "study-execution-v1-support-test",
        "protocolIdentity": "study-protocol-v1-0123456789abcdef",
        "protocolRevision": 1,
        "studyId": "test-study",
        "scientificStatus": "exploratory",
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": False,
        "definitionIdentity": "research-definition-v1-support-test",
        "researchId": "research-execution-v1-support-test",
        "source": {"repository": "Datengu/anthrosim", "commit": "test"},
        "researchRelativeDir": "research",
        "runCounts": {"completed": 1, "failed": 0},
        "resultArtifacts": [],
        "analysisRequirements": [
            {
                "kind": "observable_support_sensitivity",
                "identity": plan_identity(plan),
            }
        ],
    }
    assessment = {
        "schema": "anthrosim-observable-support-assessment-v1",
        "planIdentity": plan_identity(plan),
        "protocolIdentity": binding["protocolIdentity"],
        "studyId": binding["studyId"],
        "entries": plan["entries"],
        "sourceStudyExecutionId": binding["studyExecutionId"],
        "sourceStudyResultIdentity": binding["resultIdentity"],
        "sourceResearchId": binding["researchId"],
    }
    assessment["assessmentIdentity"] = assessment_identity(assessment)
    return plan, binding, assessment


ANALYSIS_IMPLEMENTATION = r'''#!/usr/bin/env python3
import json
import sys
from pathlib import Path

config = json.loads(Path(sys.argv[1]).read_text())
output = {
    "schema": "anthrosim-observable-support-inference-v1",
    "supportAssessmentIdentity": config["supportAssessmentIdentity"],
    "observableId": config["observableId"],
    "binningId": config["binningId"],
    "inferenceClass": sys.argv[3],
}
Path(sys.argv[2]).parent.mkdir(parents=True, exist_ok=True)
Path(sys.argv[2]).write_text(json.dumps(output, sort_keys=True, separators=(",", ":")) + "\n")
'''


def make_analysis(
    root: Path,
    plan_path: Path,
    assessment_path: Path,
    *,
    label: str,
    observable_id: str,
    binning_id: str,
    inference_class: str,
):
    analysis_dir = root / "analysis" / "observable-support" / label
    analysis_dir.mkdir(parents=True, exist_ok=True)
    config_path = analysis_dir / "binning.json"
    inference_path = analysis_dir / "inference.json"
    definition_path = analysis_dir / "definition.json"
    record_path = analysis_dir / "analysis-provenance.json"
    relative_config = config_path.relative_to(root).as_posix()
    relative_inference = inference_path.relative_to(root).as_posix()
    relative_impl = "analysis/observable-support/implementation.py"
    relative_record = record_path.relative_to(root).as_posix()

    run(
        "binning-definition",
        "--plan",
        plan_path,
        "--assessment",
        assessment_path,
        "--observable-id",
        observable_id,
        "--binning-id",
        binning_id,
        "--output",
        config_path,
    )

    definition = {
        "schemaVersion": 2,
        "definitionType": "anthrosim-analysis-definition",
        "analysisId": f"support-{label}",
        "analysisStatus": "exploratory",
        "executionMode": "scripted",
        "workingDirectory": ".",
        "command": [
            sys.executable,
            relative_impl,
            relative_config,
            relative_inference,
            inference_class,
        ],
        "annotations": {"purpose": "observable support sensitivity test"},
        "runtimeDescription": "Python standard library test analysis",
        "reproductionCriterion": "exact_output_bytes",
        "inputs": [
            {
                "path": relative_config,
                "role": "observable-support-binning-definition",
            }
        ],
        "implementation": [
            {"path": relative_impl, "role": "support-analysis-implementation"}
        ],
        "environment": [],
        "outputs": [
            {"path": relative_inference, "role": "observable-support-inference"}
        ],
        "manualSteps": [],
    }
    write(definition_path, definition)
    result = run_script(
        PROVENANCE,
        "run",
        root,
        definition_path,
        "--output",
        relative_record,
    )
    identity = result.stdout.strip()
    assert identity.startswith("analysis-provenance-v2-sha256-")
    return identity, inference_path, record_path


def validate_paths(root, plan_path, assessment_path, report_path, ok=True):
    return run(
        "validate",
        "--study-root",
        root,
        "--plan",
        plan_path,
        "--assessment",
        assessment_path,
        "--report",
        report_path,
        ok=ok,
    )


def main():
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        plan, binding, assessment = fixtures()
        plan_path = root / "support-plan.json"
        assessment_path = root / "support-assessment.json"
        declaration_path = root / "declaration.json"
        report_path = root / "report.json"
        write(root / "study-result-binding.json", binding)
        write(plan_path, plan)
        write(assessment_path, assessment)
        impl = root / "analysis" / "observable-support" / "implementation.py"
        impl.parent.mkdir(parents=True, exist_ok=True)
        impl.write_text(ANALYSIS_IMPLEMENTATION)

        primary_id, primary_output, primary_record = make_analysis(
            root,
            plan_path,
            assessment_path,
            label="primary",
            observable_id="occupancy",
            binning_id="primary",
            inference_class="supports_h1",
        )
        coarse_id, coarse_output, coarse_record = make_analysis(
            root,
            plan_path,
            assessment_path,
            label="coarse",
            observable_id="occupancy",
            binning_id="coarse-phase",
            inference_class="supports_h1",
        )
        changed_id, _, _ = make_analysis(
            root,
            plan_path,
            assessment_path,
            label="coarse-changed",
            observable_id="occupancy",
            binning_id="coarse-phase",
            inference_class="does_not_support_h1",
        )

        declaration = {
            "schema": "anthrosim-observable-support-sensitivity-report-v1",
            "supportAssessmentIdentity": assessment["assessmentIdentity"],
            "observableResults": [
                {
                    "observableId": "occupancy",
                    "primary": {
                        "binningId": "primary",
                        "analysisIdentity": primary_id,
                        "inferenceClass": "supports_h1",
                    },
                    "alternatives": [
                        {
                            "binningId": "coarse-phase",
                            "analysisIdentity": coarse_id,
                            "inferenceClass": "supports_h1",
                        }
                    ],
                    "materialScaleDependence": False,
                    "dependenceStatement": None,
                }
            ],
        }
        write(declaration_path, declaration)

        run(
            "derive",
            "--study-root",
            root,
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
            "--study-root",
            root,
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
            "--study-root",
            root,
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

        fabricated = copy.deepcopy(declaration)
        fabricated["observableResults"][0]["primary"][
            "analysisIdentity"
        ] = "definitely-not-a-real-analysis-primary"
        write(root / "fabricated.json", fabricated)
        validate_paths(root, plan_path, assessment_path, root / "fabricated.json", ok=False)

        wrong_binning = copy.deepcopy(declaration)
        wrong_binning["observableResults"][0]["alternatives"][0][
            "analysisIdentity"
        ] = primary_id
        write(root / "wrong-binning.json", wrong_binning)
        validate_paths(root, plan_path, assessment_path, root / "wrong-binning.json", ok=False)

        inference_mismatch = copy.deepcopy(declaration)
        inference_mismatch["observableResults"][0]["alternatives"][0][
            "inferenceClass"
        ] = "does_not_support_h1"
        inference_mismatch["observableResults"][0]["materialScaleDependence"] = True
        inference_mismatch["observableResults"][0][
            "dependenceStatement"
        ] = "changed"
        write(root / "inference-mismatch.json", inference_mismatch)
        validate_paths(
            root, plan_path, assessment_path, root / "inference-mismatch.json", ok=False
        )

        missing = copy.deepcopy(declaration)
        missing["observableResults"][0]["alternatives"] = []
        write(root / "missing.json", missing)
        validate_paths(root, plan_path, assessment_path, root / "missing.json", ok=False)

        changed = copy.deepcopy(declaration)
        changed["observableResults"][0]["alternatives"][0][
            "analysisIdentity"
        ] = changed_id
        changed["observableResults"][0]["alternatives"][0][
            "inferenceClass"
        ] = "does_not_support_h1"
        changed["observableResults"][0]["materialScaleDependence"] = True
        changed["observableResults"][0][
            "dependenceStatement"
        ] = "The substantive inference changes under the coarser chronology."
        write(root / "changed.json", changed)
        validate_paths(root, plan_path, assessment_path, root / "changed.json")

        unreported = copy.deepcopy(changed)
        unreported["observableResults"][0]["dependenceStatement"] = ""
        write(root / "unreported.json", unreported)
        validate_paths(root, plan_path, assessment_path, root / "unreported.json", ok=False)

        false_flag = copy.deepcopy(changed)
        false_flag["observableResults"][0]["materialScaleDependence"] = False
        false_flag["observableResults"][0]["dependenceStatement"] = None
        write(root / "false-flag.json", false_flag)
        validate_paths(root, plan_path, assessment_path, root / "false-flag.json", ok=False)

        wrong_assessment = copy.deepcopy(declaration)
        wrong_assessment["supportAssessmentIdentity"] = "other-assessment"
        write(root / "wrong-assessment.json", wrong_assessment)
        validate_paths(
            root, plan_path, assessment_path, root / "wrong-assessment.json", ok=False
        )

        duplicate_dir = root / "analysis" / "observable-support" / "duplicate"
        duplicate_dir.mkdir(parents=True)
        duplicate_record = duplicate_dir / "analysis-provenance.json"
        shutil.copyfile(coarse_record, duplicate_record)
        validate_paths(root, plan_path, assessment_path, declaration_path, ok=False)
        duplicate_record.unlink()

        original_primary = primary_output.read_bytes()
        primary_output.write_text('{"tampered":true}\n')
        validate_paths(root, plan_path, assessment_path, declaration_path, ok=False)
        primary_output.write_bytes(original_primary)
        validate_paths(root, plan_path, assessment_path, declaration_path)

        # Ensure provenance records themselves remain directly verifiable after all controls.
        run_script(PROVENANCE, "verify", root, "--record", primary_record.relative_to(root))
        run_script(PROVENANCE, "verify", root, "--record", coarse_record.relative_to(root))

    print("research observable-support sensitivity regression suite passed")


if __name__ == "__main__":
    main()
