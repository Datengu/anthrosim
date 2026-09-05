#!/usr/bin/env python3
"""Audit TRACE evidence-use roles in frozen AnthroSim study protocols.

This is a research-governance layer. It does not alter simulation state or model semantics.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
REPORT_TYPE = "anthrosim-evidence-role-assessment"
REPORT_RELATIVE_PATH = Path("analysis/evidence-role-assessment.json")

ROLES = {
    "model_construction",
    "parameterisation",
    "calibration",
    "model_output_verification",
    "independent_corroboration",
}
OBSERVABLE_TARGET_ROLES = {
    "calibration",
    "model_output_verification",
    "independent_corroboration",
}
INDEPENDENT_ROLE = "independent_corroboration"


class EvidenceRoleAuditError(ValueError):
    pass


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceRoleAuditError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def _read_json(path: Path, role: str) -> Any:
    if path.is_symlink():
        raise EvidenceRoleAuditError(f"{role} must not be a symlink: {path}")
    if not path.is_file():
        raise EvidenceRoleAuditError(f"{role} is missing or not a regular file: {path}")
    try:
        return json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=_reject_duplicate_keys
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise EvidenceRoleAuditError(f"cannot read {role} {path}: {exc}") from exc


def _nonempty_string(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise EvidenceRoleAuditError(f"{field} must be a non-empty string")
    return value.strip()


def _canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


def _fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def protocol_identity(protocol: dict[str, Any]) -> str:
    return f"study-protocol-v1-{_fnv1a64(_canonical_bytes(protocol)):016x}"


def _assessment_identity(report: dict[str, Any]) -> str:
    identity_source = dict(report)
    identity_source["assessmentIdentity"] = ""
    digest = hashlib.sha256(_canonical_bytes(identity_source)).hexdigest()
    return f"evidence-role-assessment-v1-sha256-{digest}"


def _source_identity(source: dict[str, Any]) -> str:
    digest = hashlib.sha256(_canonical_bytes(source)).hexdigest()
    return f"evidence-source-v1-sha256-{digest}"


def _validate_protocol_shape(
    protocol: Any,
) -> tuple[str, list[dict[str, Any]], list[dict[str, Any]], set[str]]:
    if not isinstance(protocol, dict):
        raise EvidenceRoleAuditError("study protocol must be a JSON object")
    if protocol.get("schemaVersion") != 1:
        raise EvidenceRoleAuditError(
            f"unsupported study protocol schema {protocol.get('schemaVersion')!r}; expected 1"
        )
    status = protocol.get("status")
    if status not in {"exploratory", "confirmatory"}:
        raise EvidenceRoleAuditError(
            f"study protocol status must be exploratory or confirmatory, found {status!r}"
        )
    _nonempty_string(protocol.get("studyId"), "studyId")

    observables_raw = protocol.get("observables")
    if not isinstance(observables_raw, list):
        raise EvidenceRoleAuditError("observables must be an array")
    observable_ids: set[str] = set()
    for index, observable in enumerate(observables_raw):
        if not isinstance(observable, dict):
            raise EvidenceRoleAuditError(f"observables[{index}] must be an object")
        observable_id = _nonempty_string(observable.get("id"), f"observables[{index}].id")
        if observable_id in observable_ids:
            raise EvidenceRoleAuditError(f"duplicate observable id: {observable_id}")
        observable_ids.add(observable_id)

    assignments = protocol.get("evidenceRoles")
    if not isinstance(assignments, list):
        raise EvidenceRoleAuditError("evidenceRoles must be an array")
    normalized_assignments: list[dict[str, Any]] = []
    exact_assignments: set[tuple[str, str, str]] = set()
    for index, assignment in enumerate(assignments):
        if not isinstance(assignment, dict):
            raise EvidenceRoleAuditError(f"evidenceRoles[{index}] must be an object")
        unknown = set(assignment) - {"evidenceId", "role", "target", "notes"}
        missing = {"evidenceId", "role", "target", "notes"} - set(assignment)
        if unknown or missing:
            raise EvidenceRoleAuditError(
                f"evidenceRoles[{index}] has invalid keys; missing={sorted(missing)}, unknown={sorted(unknown)}"
            )
        evidence_id = _nonempty_string(
            assignment.get("evidenceId"), f"evidenceRoles[{index}].evidenceId"
        )
        role = _nonempty_string(assignment.get("role"), f"evidenceRoles[{index}].role")
        if role not in ROLES:
            raise EvidenceRoleAuditError(
                f"evidenceRoles[{index}].role {role!r} is not a TRACE evidence role"
            )
        target = _nonempty_string(assignment.get("target"), f"evidenceRoles[{index}].target")
        notes = _nonempty_string(assignment.get("notes"), f"evidenceRoles[{index}].notes")
        key = (evidence_id, role, target)
        if key in exact_assignments:
            raise EvidenceRoleAuditError(
                f"duplicate evidence-role assignment for evidence {evidence_id!r}, role {role!r}, target {target!r}"
            )
        exact_assignments.add(key)
        normalized_assignments.append(
            {"evidenceId": evidence_id, "role": role, "target": target, "notes": notes}
        )

    held_out = protocol.get("heldOutCorroboration")
    if not isinstance(held_out, list):
        raise EvidenceRoleAuditError("heldOutCorroboration must be an array")
    normalized_held_out: list[dict[str, Any]] = []
    held_out_ids: set[str] = set()
    held_out_pairs: set[tuple[str, str]] = set()
    for index, target in enumerate(held_out):
        if not isinstance(target, dict):
            raise EvidenceRoleAuditError(f"heldOutCorroboration[{index}] must be an object")
        unknown = set(target) - {"id", "evidenceId", "observableId", "criterion"}
        missing = {"id", "evidenceId", "observableId", "criterion"} - set(target)
        if unknown or missing:
            raise EvidenceRoleAuditError(
                f"heldOutCorroboration[{index}] has invalid keys; missing={sorted(missing)}, unknown={sorted(unknown)}"
            )
        target_id = _nonempty_string(target.get("id"), f"heldOutCorroboration[{index}].id")
        evidence_id = _nonempty_string(
            target.get("evidenceId"), f"heldOutCorroboration[{index}].evidenceId"
        )
        observable_id = _nonempty_string(
            target.get("observableId"), f"heldOutCorroboration[{index}].observableId"
        )
        criterion = _nonempty_string(
            target.get("criterion"), f"heldOutCorroboration[{index}].criterion"
        )
        if target_id in held_out_ids:
            raise EvidenceRoleAuditError(f"duplicate held-out corroboration id: {target_id}")
        held_out_ids.add(target_id)
        if observable_id not in observable_ids:
            raise EvidenceRoleAuditError(
                f"held-out corroboration {target_id!r} references unknown observable {observable_id!r}"
            )
        pair = (evidence_id, observable_id)
        if pair in held_out_pairs:
            raise EvidenceRoleAuditError(
                f"duplicate held-out evidence/observable pair: evidence={evidence_id!r}, observable={observable_id!r}"
            )
        held_out_pairs.add(pair)
        normalized_held_out.append(
            {
                "id": target_id,
                "evidenceId": evidence_id,
                "observableId": observable_id,
                "criterion": criterion,
            }
        )

    return status, normalized_assignments, normalized_held_out, observable_ids


def _evidence_catalog_from_definition(definition: Any) -> dict[str, Any] | None:
    if not isinstance(definition, dict):
        raise EvidenceRoleAuditError("research definition must be a JSON object")
    if definition.get("schemaVersion") != 1:
        raise EvidenceRoleAuditError(
            f"unsupported research definition schema {definition.get('schemaVersion')!r}; expected 1"
        )
    base = definition.get("base")
    if not isinstance(base, dict):
        raise EvidenceRoleAuditError("research definition base must be an object")
    experiment = base.get("experiment")
    if not isinstance(experiment, dict):
        raise EvidenceRoleAuditError("research definition base.experiment must be an object")
    catalog = experiment.get("evidence")
    if catalog is None:
        return None
    if not isinstance(catalog, dict):
        raise EvidenceRoleAuditError("research definition evidence catalog must be an object")
    return catalog


def _catalog_bindings(definition: Any) -> dict[str, dict[str, Any]]:
    catalog = _evidence_catalog_from_definition(definition)
    if catalog is None:
        return {}
    if catalog.get("schemaVersion") != 1:
        raise EvidenceRoleAuditError(
            f"unsupported evidence catalog schema {catalog.get('schemaVersion')!r}; expected 1"
        )
    records = catalog.get("records")
    if not isinstance(records, list):
        raise EvidenceRoleAuditError("evidence catalog records must be an array")

    bindings: dict[str, dict[str, Any]] = {}
    for index, record in enumerate(records):
        if not isinstance(record, dict):
            raise EvidenceRoleAuditError(f"evidence catalog records[{index}] must be an object")
        evidence_id = _nonempty_string(
            record.get("evidenceId"), f"evidence catalog records[{index}].evidenceId"
        )
        if evidence_id in bindings:
            raise EvidenceRoleAuditError(f"duplicate evidence identifier in catalog: {evidence_id}")
        source = record.get("source")
        if not isinstance(source, dict):
            raise EvidenceRoleAuditError(
                f"evidence catalog record {evidence_id!r} must contain a source object"
            )
        _nonempty_string(source.get("sourceId"), f"evidence catalog record {evidence_id!r} source.sourceId")
        identity = _source_identity(source)
        bindings[evidence_id] = {
            "evidenceId": evidence_id,
            "sourceIdentity": identity,
            "source": source,
        }
    return bindings


def assess_protocol(
    protocol: dict[str, Any], definition: dict[str, Any] | None = None
) -> dict[str, Any]:
    status, assignments, held_out, observable_ids = _validate_protocol_shape(protocol)
    firewall_enforced = status == "confirmatory"
    referenced_ids = {
        item["evidenceId"] for item in assignments
    } | {item["evidenceId"] for item in held_out}

    bindings: dict[str, dict[str, Any]] = {}
    if firewall_enforced and referenced_ids:
        if definition is None:
            raise EvidenceRoleAuditError(
                "confirmatory evidence-role claims require the bound research definition so evidence IDs can be resolved through its EvidenceCatalog"
            )
        bindings = _catalog_bindings(definition)
        missing = sorted(referenced_ids - set(bindings))
        if missing:
            raise EvidenceRoleAuditError(
                f"confirmatory evidence-role claim references unknown EvidenceCatalog evidence ID(s): {missing}"
            )
    elif definition is not None:
        bindings = _catalog_bindings(definition)

    by_evidence_target: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)
    by_evidence: dict[str, list[dict[str, Any]]] = defaultdict(list)
    by_source_target: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)
    by_source: dict[str, list[dict[str, Any]]] = defaultdict(list)
    role_counts: Counter[str] = Counter()

    resolved_assignments: list[dict[str, Any]] = []
    for assignment in assignments:
        evidence_id = assignment["evidenceId"]
        resolved = dict(assignment)
        if evidence_id in bindings:
            resolved["sourceIdentity"] = bindings[evidence_id]["sourceIdentity"]
        resolved_assignments.append(resolved)
        by_evidence_target[(evidence_id, assignment["target"])].append(resolved)
        by_evidence[evidence_id].append(resolved)
        if "sourceIdentity" in resolved:
            by_source_target[(resolved["sourceIdentity"], assignment["target"])].append(resolved)
            by_source[resolved["sourceIdentity"]].append(resolved)
        role_counts[assignment["role"]] += 1

    resolved_held_out: list[dict[str, Any]] = []
    for item in held_out:
        resolved = dict(item)
        if item["evidenceId"] in bindings:
            resolved["sourceIdentity"] = bindings[item["evidenceId"]]["sourceIdentity"]
        resolved_held_out.append(resolved)

    held_out_pairs = {(item["evidenceId"], item["observableId"]) for item in held_out}

    if firewall_enforced:
        for assignment in resolved_assignments:
            if assignment["role"] in OBSERVABLE_TARGET_ROLES and assignment["target"] not in observable_ids:
                raise EvidenceRoleAuditError(
                    f"confirmatory {assignment['role']} assignment for evidence {assignment['evidenceId']!r} "
                    f"must target a declared observable id; found {assignment['target']!r}"
                )

        for item in resolved_held_out:
            pair = (item["evidenceId"], item["observableId"])
            assignments_for_pair = by_evidence_target.get(pair, [])
            independent = [a for a in assignments_for_pair if a["role"] == INDEPENDENT_ROLE]
            if not independent:
                raise EvidenceRoleAuditError(
                    f"held-out corroboration {item['id']!r} has no matching independent_corroboration "
                    f"evidence-role assignment for evidence {item['evidenceId']!r} and observable {item['observableId']!r}"
                )

            source_pair = (item["sourceIdentity"], item["observableId"])
            assignments_for_source = by_source_target.get(source_pair, [])
            conflicting = [
                a for a in assignments_for_source if a["role"] != INDEPENDENT_ROLE
            ]
            if conflicting:
                aliases = sorted({a["evidenceId"] for a in assignments_for_source})
                conflicting_roles = sorted({a["role"] for a in conflicting})
                raise EvidenceRoleAuditError(
                    f"circular independence claim for source identity {item['sourceIdentity']} and observable "
                    f"{item['observableId']!r}: EvidenceCatalog alias(es) {aliases} combine "
                    f"independent_corroboration with prior role(s) {conflicting_roles}; aliases of one immutable source are not independent evidence"
                )

        for assignment in resolved_assignments:
            if assignment["role"] == INDEPENDENT_ROLE:
                pair = (assignment["evidenceId"], assignment["target"])
                if pair not in held_out_pairs:
                    raise EvidenceRoleAuditError(
                        f"independent_corroboration assignment for evidence {assignment['evidenceId']!r} "
                        f"and observable {assignment['target']!r} is not declared in heldOutCorroboration"
                    )

    reused_evidence = []
    for evidence_id in sorted(by_evidence):
        entries = by_evidence[evidence_id]
        distinct_roles = sorted({entry["role"] for entry in entries})
        distinct_targets = sorted({entry["target"] for entry in entries})
        if len(distinct_roles) > 1 or len(distinct_targets) > 1:
            reused_evidence.append(
                {
                    "evidenceId": evidence_id,
                    "sourceIdentity": entries[0].get("sourceIdentity"),
                    "roles": distinct_roles,
                    "targets": distinct_targets,
                }
            )

    reused_sources = []
    for source_identity in sorted(by_source):
        entries = by_source[source_identity]
        evidence_ids = sorted({entry["evidenceId"] for entry in entries})
        distinct_roles = sorted({entry["role"] for entry in entries})
        distinct_targets = sorted({entry["target"] for entry in entries})
        if len(evidence_ids) > 1 or len(distinct_roles) > 1 or len(distinct_targets) > 1:
            reused_sources.append(
                {
                    "sourceIdentity": source_identity,
                    "evidenceIds": evidence_ids,
                    "roles": distinct_roles,
                    "targets": distinct_targets,
                }
            )

    if status == "exploratory":
        assessment_status = "exploratory_permissive"
    elif held_out:
        assessment_status = "confirmatory_independence_firewall_satisfied"
    else:
        assessment_status = "confirmatory_no_independent_corroboration_claim"

    bound_ids = sorted(referenced_ids & set(bindings))
    evidence_bindings = [bindings[evidence_id] for evidence_id in bound_ids]
    source_count = len({binding["sourceIdentity"] for binding in evidence_bindings})

    return {
        "protocolIdentity": protocol_identity(protocol),
        "studyId": protocol["studyId"],
        "scientificStatus": status,
        "assessmentStatus": assessment_status,
        "firewallEnforced": firewall_enforced,
        "assignments": resolved_assignments,
        "heldOutCorroboration": resolved_held_out,
        "evidenceBindings": evidence_bindings,
        "reusedEvidence": reused_evidence,
        "reusedSources": reused_sources,
        "summary": {
            "assignmentCount": len(assignments),
            "evidenceRecordCount": len(bound_ids) if bindings else len(by_evidence),
            "evidenceSourceCount": source_count if bindings else len(by_evidence),
            "heldOutTargetCount": len(held_out),
            "roleCounts": {role: role_counts.get(role, 0) for role in sorted(ROLES)},
        },
    }


def _load_finalized_study(
    study_dir: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    if study_dir.is_symlink() or not study_dir.is_dir():
        raise EvidenceRoleAuditError(f"study root is missing, not a directory, or a symlink: {study_dir}")

    plan = _read_json(study_dir / "study-plan.json", "study plan")
    manifest = _read_json(study_dir / "study-manifest.json", "study manifest")
    if plan != manifest:
        raise EvidenceRoleAuditError("study-plan.json and study-manifest.json differ")
    if not isinstance(plan, dict) or plan.get("schemaVersion") != 1:
        raise EvidenceRoleAuditError("unsupported or malformed study plan")

    protocol = _read_json(study_dir / "study-protocol.json", "frozen study protocol")
    if plan.get("protocol") != protocol:
        raise EvidenceRoleAuditError("frozen study-protocol.json differs from the immutable study plan")
    computed_protocol_identity = protocol_identity(protocol)
    if plan.get("protocolIdentity") != computed_protocol_identity:
        raise EvidenceRoleAuditError(
            "study plan protocolIdentity differs from the frozen protocol content identity"
        )

    binding = _read_json(study_dir / "study-result-binding.json", "study result binding")
    if not isinstance(binding, dict) or binding.get("schemaVersion") != 1:
        raise EvidenceRoleAuditError("unsupported or malformed study result binding")

    expected_pairs = {
        "studyExecutionId": plan.get("studyExecutionId"),
        "protocolIdentity": plan.get("protocolIdentity"),
        "protocolRevision": protocol.get("protocolRevision"),
        "studyId": protocol.get("studyId"),
        "scientificStatus": protocol.get("status"),
        "boundBeforeExecution": plan.get("boundBeforeExecution"),
        "confirmatoryPreResultClaimEligible": plan.get("confirmatoryPreResultClaimEligible"),
        "definitionIdentity": plan.get("definitionIdentity"),
    }
    for field, expected in expected_pairs.items():
        if binding.get(field) != expected:
            raise EvidenceRoleAuditError(
                f"study result binding field {field} does not match the frozen study plan/protocol"
            )
    _nonempty_string(binding.get("resultIdentity"), "study-result-binding.resultIdentity")
    _nonempty_string(binding.get("researchId"), "study-result-binding.researchId")
    if not isinstance(plan.get("definition"), dict):
        raise EvidenceRoleAuditError("study plan does not preserve a valid bound research definition")

    return protocol, plan, binding


def build_study_report(study_dir: Path) -> dict[str, Any]:
    protocol, plan, binding = _load_finalized_study(study_dir)
    assessment = assess_protocol(protocol, plan["definition"])

    has_independent_claim = bool(assessment["heldOutCorroboration"]) or any(
        assignment["role"] == INDEPENDENT_ROLE for assignment in assessment["assignments"]
    )
    if (
        assessment["scientificStatus"] == "confirmatory"
        and has_independent_claim
        and (
            binding.get("boundBeforeExecution") is not True
            or binding.get("confirmatoryPreResultClaimEligible") is not True
        )
    ):
        raise EvidenceRoleAuditError(
            "confirmatory independent corroboration requires a protocol frozen before execution and eligible for a pre-result confirmatory claim"
        )

    report: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "reportType": REPORT_TYPE,
        "assessmentIdentity": "",
        "protocolIdentity": assessment["protocolIdentity"],
        "studyBinding": {
            "studyExecutionId": binding["studyExecutionId"],
            "resultIdentity": binding["resultIdentity"],
            "studyId": binding["studyId"],
            "protocolRevision": binding["protocolRevision"],
            "scientificStatus": binding["scientificStatus"],
            "boundBeforeExecution": binding["boundBeforeExecution"],
            "confirmatoryPreResultClaimEligible": binding[
                "confirmatoryPreResultClaimEligible"
            ],
            "definitionIdentity": binding["definitionIdentity"],
            "researchId": binding["researchId"],
        },
        "firewall": {
            "assessmentStatus": assessment["assessmentStatus"],
            "firewallEnforced": assessment["firewallEnforced"],
            "independenceClaimEligible": (
                assessment["scientificStatus"] != "confirmatory"
                or not has_independent_claim
                or (
                    binding["boundBeforeExecution"] is True
                    and binding["confirmatoryPreResultClaimEligible"] is True
                )
            ),
        },
        "assignments": assessment["assignments"],
        "heldOutCorroboration": assessment["heldOutCorroboration"],
        "evidenceBindings": assessment["evidenceBindings"],
        "reusedEvidence": assessment["reusedEvidence"],
        "reusedSources": assessment["reusedSources"],
        "summary": assessment["summary"],
    }
    report["assessmentIdentity"] = _assessment_identity(report)
    return report


def _write_json_atomic(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    if path.exists():
        existing = _read_json(path, "existing evidence-role assessment")
        if existing != value:
            raise EvidenceRoleAuditError(
                f"existing {path} differs from the current frozen study; create a new study revision rather than overwriting provenance"
            )
        return
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(encoded)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    except Exception:
        try:
            os.unlink(temporary)
        except OSError:
            pass
        raise


def derive(study_dir: Path) -> str:
    report = build_study_report(study_dir)
    _write_json_atomic(study_dir / REPORT_RELATIVE_PATH, report)
    return report["assessmentIdentity"]


def verify(study_dir: Path) -> str:
    expected = build_study_report(study_dir)
    actual = _read_json(study_dir / REPORT_RELATIVE_PATH, "evidence-role assessment")
    if actual != expected:
        raise EvidenceRoleAuditError(
            "evidence-role assessment does not match the current frozen study/protocol/evidence binding"
        )
    if actual.get("assessmentIdentity") != _assessment_identity(actual):
        raise EvidenceRoleAuditError("evidence-role assessment identity does not verify")
    return actual["assessmentIdentity"]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit calibration/verification/held-out evidence roles for AnthroSim studies"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser(
        "validate", help="validate evidence-role semantics before freezing a study"
    )
    validate_parser.add_argument("protocol", type=Path)
    validate_parser.add_argument(
        "--definition",
        type=Path,
        help="research definition whose base ExperimentConfig contains the authoritative EvidenceCatalog",
    )

    derive_parser = subparsers.add_parser(
        "derive", help="derive an immutable evidence-role assessment for a finalized study"
    )
    derive_parser.add_argument("study_dir", type=Path)

    verify_parser = subparsers.add_parser(
        "verify", help="verify a preserved evidence-role assessment against its frozen study"
    )
    verify_parser.add_argument("study_dir", type=Path)

    args = parser.parse_args()
    try:
        if args.command == "validate":
            protocol = _read_json(args.protocol, "study protocol")
            definition = (
                _read_json(args.definition, "research definition")
                if args.definition is not None
                else None
            )
            assessment = assess_protocol(protocol, definition)
            print(assessment["protocolIdentity"])
        elif args.command == "derive":
            print(derive(args.study_dir))
        else:
            print(verify(args.study_dir))
        return 0
    except EvidenceRoleAuditError as exc:
        print(f"research-evidence-role-audit: {exc}", file=os.sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
