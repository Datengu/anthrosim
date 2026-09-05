#!/usr/bin/env python3
"""Regression tests for research-evidence-role-audit.py."""

from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-evidence-role-audit.py")
SPEC = importlib.util.spec_from_file_location("research_evidence_role_audit", SCRIPT)
assert SPEC and SPEC.loader
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


def source(source_id: str, persistent_id: str) -> dict:
    return {
        "sourceId": source_id,
        "citation": f"Synthetic citation for {source_id}",
        "persistentId": persistent_id,
        "datasetVersion": "v1",
    }


def definition() -> dict:
    held_out_source = source("dataset-held-out", "doi:10.example/held-out")
    return {
        "schemaVersion": 1,
        "base": {
            "experiment": {
                "evidence": {
                    "schemaVersion": 1,
                    "records": [
                        {
                            "schemaVersion": 1,
                            "evidenceId": "source-design",
                            "source": source("dataset-design", "doi:10.example/design"),
                        },
                        {
                            "schemaVersion": 1,
                            "evidenceId": "source-calibration",
                            "source": source("dataset-calibration", "doi:10.example/calibration"),
                        },
                        {
                            "schemaVersion": 1,
                            "evidenceId": "source-held-out",
                            "source": held_out_source,
                        },
                        {
                            "schemaVersion": 1,
                            "evidenceId": "source-held-out-alias",
                            "source": copy.deepcopy(held_out_source),
                        },
                    ],
                }
            }
        },
    }


def protocol() -> dict:
    return {
        "schemaVersion": 1,
        "protocolRevision": 1,
        "studyId": "evidence-firewall-test",
        "status": "confirmatory",
        "researchQuestion": "Can held-out evidence remain independent of calibration?",
        "applicabilityDomain": "Synthetic research-governance fixture only.",
        "hypotheses": [
            {"id": "null", "kind": "null_model", "statement": "No effect."},
            {"id": "alt", "kind": "alternative", "statement": "An effect exists."},
        ],
        "analysisWindows": [
            {
                "id": "primary",
                "analysisStartDay": 0,
                "selectionRule": "initial_state_in_scope",
                "rationale": "Synthetic fixture uses the complete interval.",
            }
        ],
        "observables": [
            {
                "id": "occupancy_pattern",
                "role": "primary",
                "source": "synthetic.occupancy",
                "analysisWindowId": "primary",
                "interpretation": "Synthetic occupancy score.",
            },
            {
                "id": "resource_pattern",
                "role": "secondary",
                "source": "synthetic.resources",
                "analysisWindowId": "primary",
                "interpretation": "Synthetic resource score.",
            },
        ],
        "comparisons": [
            {
                "id": "primary",
                "hypothesisIds": ["null", "alt"],
                "observableIds": ["occupancy_pattern"],
                "prediction": "Alternative differs from null.",
                "decisionCriterion": "Use the predeclared contrast.",
            }
        ],
        "evidenceRoles": [
            {
                "evidenceId": "source-design",
                "role": "model_construction",
                "target": "mechanism:settlement-choice",
                "notes": "Qualitative contextual evidence used to define the conceptual mechanism.",
            },
            {
                "evidenceId": "source-calibration",
                "role": "calibration",
                "target": "occupancy_pattern",
                "notes": "Used to fit the declared occupancy target; not independent validation.",
            },
            {
                "evidenceId": "source-calibration",
                "role": "model_output_verification",
                "target": "occupancy_pattern",
                "notes": "Explicit in-sample verification reuse; no independence claim.",
            },
            {
                "evidenceId": "source-held-out",
                "role": "independent_corroboration",
                "target": "occupancy_pattern",
                "notes": "Reserved until the frozen confirmatory result is evaluated.",
            },
        ],
        "uncertainty": {"parameterUncertainty": [], "structuralUncertainty": []},
        "ensemblePolicy": {
            "seedPolicy": "Use frozen seeds.",
            "pairingPolicy": "No special pairing claim.",
            "replicationPolicy": "Use the predeclared replicate set.",
        },
        "runHandling": {
            "stoppingRules": ["Use declared scientific stop reasons."],
            "exclusionRules": ["No post-hoc outcome exclusions."],
            "censoringRules": ["Report operational censoring."],
        },
        "sensitivityPlan": [],
        "equifinalityPlan": [],
        "manipulationChecks": [],
        "analysisMethod": "Synthetic comparison.",
        "multiplicityPolicy": "One primary comparison.",
        "heldOutCorroboration": [
            {
                "id": "held-out-occupancy",
                "evidenceId": "source-held-out",
                "observableId": "occupancy_pattern",
                "criterion": "Apply only after the frozen primary result exists.",
            }
        ],
        "permittedInterpretations": ["Research-governance validation."],
        "prohibitedInterpretations": ["Empirical archaeological inference."],
    }


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def expect_rejected(value: dict, research_definition: dict | None, fragment: str) -> None:
    try:
        AUDIT.assess_protocol(value, research_definition)
    except AUDIT.EvidenceRoleAuditError as exc:
        assert fragment in str(exc), (fragment, str(exc))
    else:
        raise AssertionError(f"expected rejection containing {fragment!r}")


def make_finalized_study(root: Path, value: dict, research_definition: dict) -> Path:
    study = root / "study"
    study.mkdir(parents=True)
    identity = AUDIT.protocol_identity(value)
    plan = {
        "schemaVersion": 1,
        "studyExecutionId": "study-execution-v1-test",
        "protocolIdentity": identity,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": value["status"] == "confirmatory",
        "protocol": value,
        "definitionIdentity": "research-definition-v1-test",
        "source": {"kind": "synthetic-test"},
        "definition": research_definition,
        "researchRelativeDir": "research",
    }
    binding = {
        "schemaVersion": 1,
        "resultIdentity": "study-result-v1-test",
        "studyExecutionId": plan["studyExecutionId"],
        "protocolIdentity": identity,
        "protocolRevision": value["protocolRevision"],
        "studyId": value["studyId"],
        "scientificStatus": value["status"],
        "boundBeforeExecution": plan["boundBeforeExecution"],
        "confirmatoryPreResultClaimEligible": plan[
            "confirmatoryPreResultClaimEligible"
        ],
        "definitionIdentity": plan["definitionIdentity"],
        "researchId": "research-v1-test",
        "source": plan["source"],
        "researchRelativeDir": "research",
        "runCounts": {"completed": 2, "failed": 0},
        "resultArtifacts": [],
    }
    write_json(study / "study-plan.json", plan)
    write_json(study / "study-manifest.json", plan)
    write_json(study / "study-protocol.json", value)
    write_json(study / "study-result-binding.json", binding)
    return study


def main() -> int:
    base = protocol()
    research_definition = definition()
    assessment = AUDIT.assess_protocol(base, research_definition)
    assert assessment["assessmentStatus"] == "confirmatory_independence_firewall_satisfied"
    assert assessment["firewallEnforced"] is True
    assert assessment["summary"]["roleCounts"]["calibration"] == 1
    assert assessment["summary"]["roleCounts"]["model_output_verification"] == 1
    assert assessment["summary"]["roleCounts"]["independent_corroboration"] == 1
    assert assessment["summary"]["evidenceRecordCount"] == 3
    assert assessment["summary"]["evidenceSourceCount"] == 3
    held_out_binding = next(
        item for item in assessment["evidenceBindings"] if item["evidenceId"] == "source-held-out"
    )
    assert held_out_binding["sourceIdentity"].startswith("evidence-source-v1-sha256-")
    assert next(
        item for item in assessment["heldOutCorroboration"] if item["id"] == "held-out-occupancy"
    )["sourceIdentity"] == held_out_binding["sourceIdentity"]

    # A confirmatory claim cannot be assessed from protocol-local strings alone.
    expect_rejected(base, None, "require the bound research definition")

    # Unknown/fabricated IDs fail closed.
    unknown = copy.deepcopy(base)
    unknown["evidenceRoles"][-1]["evidenceId"] = "fabricated-held-out"
    unknown["heldOutCorroboration"][0]["evidenceId"] = "fabricated-held-out"
    expect_rejected(unknown, research_definition, "unknown EvidenceCatalog evidence ID")

    # Exact same-ID circularity remains rejected.
    circular = copy.deepcopy(base)
    circular["evidenceRoles"].append(
        {
            "evidenceId": "source-held-out",
            "role": "calibration",
            "target": "occupancy_pattern",
            "notes": "Illegally tunes the same target later called independent.",
        }
    )
    expect_rejected(circular, research_definition, "circular independence claim")

    # AV3-013 adversary: a second EvidenceCatalog record aliases the exact same immutable
    # source. The literal evidence IDs differ, but source identity must collapse them.
    alias_attack = copy.deepcopy(base)
    alias_attack["evidenceRoles"].append(
        {
            "evidenceId": "source-held-out-alias",
            "role": "calibration",
            "target": "occupancy_pattern",
            "notes": "Same underlying source hidden behind a second catalog-local alias.",
        }
    )
    expect_rejected(alias_attack, research_definition, "aliases of one immutable source")

    independent_without_holdout = copy.deepcopy(base)
    independent_without_holdout["heldOutCorroboration"] = []
    expect_rejected(
        independent_without_holdout,
        research_definition,
        "is not declared in heldOutCorroboration",
    )

    holdout_without_independent = copy.deepcopy(base)
    holdout_without_independent["evidenceRoles"] = [
        assignment
        for assignment in holdout_without_independent["evidenceRoles"]
        if assignment["role"] != "independent_corroboration"
    ]
    expect_rejected(
        holdout_without_independent,
        research_definition,
        "has no matching independent_corroboration",
    )

    unknown_calibration_target = copy.deepcopy(base)
    unknown_calibration_target["evidenceRoles"][1]["target"] = "undeclared_pattern"
    expect_rejected(
        unknown_calibration_target,
        research_definition,
        "must target a declared observable id",
    )

    # Legitimate disclosed same-source reuse across a genuinely different target remains legal,
    # including when it uses a second local alias of the same source.
    reused_across_targets = copy.deepcopy(base)
    reused_across_targets["evidenceRoles"].append(
        {
            "evidenceId": "source-held-out-alias",
            "role": "parameterisation",
            "target": "parameter:resource-productivity",
            "notes": "The same source informs a different target; reuse is explicit.",
        }
    )
    reused_assessment = AUDIT.assess_protocol(reused_across_targets, research_definition)
    held_source = next(
        item
        for item in reused_assessment["reusedSources"]
        if set(item["evidenceIds"]) == {"source-held-out", "source-held-out-alias"}
    )
    assert held_source["roles"] == ["independent_corroboration", "parameterisation"]
    assert held_source["targets"] == ["occupancy_pattern", "parameter:resource-productivity"]

    # Exploratory work remains permissive, while binding is retained when supplied.
    exploratory = copy.deepcopy(alias_attack)
    exploratory["status"] = "exploratory"
    exploratory_assessment = AUDIT.assess_protocol(exploratory, research_definition)
    assert exploratory_assessment["assessmentStatus"] == "exploratory_permissive"
    assert exploratory_assessment["firewallEnforced"] is False

    duplicate = copy.deepcopy(base)
    duplicate["evidenceRoles"].append(copy.deepcopy(duplicate["evidenceRoles"][0]))
    expect_rejected(duplicate, research_definition, "duplicate evidence-role assignment")

    with tempfile.TemporaryDirectory(prefix="anthrosim-evidence-role-audit-") as temp:
        root = Path(temp)
        source_protocol = root / "protocol.json"
        source_definition = root / "definition.json"
        write_json(source_protocol, base)
        write_json(source_definition, research_definition)
        validate = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "validate",
                str(source_protocol),
                "--definition",
                str(source_definition),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        assert validate.returncode == 0, validate.stderr
        assert validate.stdout.strip() == AUDIT.protocol_identity(base)

        missing_definition = subprocess.run(
            [sys.executable, str(SCRIPT), "validate", str(source_protocol)],
            check=False,
            capture_output=True,
            text=True,
        )
        assert missing_definition.returncode != 0
        assert "require the bound research definition" in missing_definition.stderr

        alias_protocol = root / "alias.json"
        write_json(alias_protocol, alias_attack)
        invalid = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "validate",
                str(alias_protocol),
                "--definition",
                str(source_definition),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        assert invalid.returncode != 0
        assert "aliases of one immutable source" in invalid.stderr

        study = make_finalized_study(root, base, research_definition)
        identity = AUDIT.derive(study)
        assert identity.startswith("evidence-role-assessment-v1-sha256-")
        assert AUDIT.verify(study) == identity

        report_path = study / AUDIT.REPORT_RELATIVE_PATH
        report = json.loads(report_path.read_text(encoding="utf-8"))
        assert report["protocolIdentity"] == AUDIT.protocol_identity(base)
        assert report["studyBinding"]["researchId"] == "research-v1-test"
        assert report["firewall"]["independenceClaimEligible"] is True
        assert report["evidenceBindings"] == assessment["evidenceBindings"]

        # Mutating the frozen EvidenceCatalog source while leaving the preserved assessment
        # untouched must invalidate verification, even if the protocol-local evidenceId is unchanged.
        plan_path = study / "study-plan.json"
        manifest_path = study / "study-manifest.json"
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        plan["definition"]["base"]["experiment"]["evidence"]["records"][2]["source"][
            "datasetVersion"
        ] = "tampered-v2"
        write_json(plan_path, plan)
        write_json(manifest_path, plan)
        try:
            AUDIT.verify(study)
        except AUDIT.EvidenceRoleAuditError as exc:
            assert "does not match" in str(exc)
        else:
            raise AssertionError("mutated frozen evidence source should fail assessment verification")

    print("research evidence-role audit regression tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
