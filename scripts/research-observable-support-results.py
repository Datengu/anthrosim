#!/usr/bin/env python3
"""Validate execution/reporting of predeclared observable-support sensitivity analyses."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

PLAN_SCHEMA = "anthrosim-observable-support-plan-v1"
ASSESSMENT_SCHEMA = "anthrosim-observable-support-assessment-v1"
REPORT_SCHEMA = "anthrosim-observable-support-sensitivity-report-v1"


class ContractError(ValueError):
    pass


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot read JSON {path}: {exc}") from exc


def canonical_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode()


def identity(prefix: str, value: Any) -> str:
    return prefix + hashlib.sha256(canonical_bytes(value)).hexdigest()


def nonempty(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ContractError(f"{field} must be a non-empty string")
    return value


def exact_keys(
    obj: Any, allowed: set[str], required: set[str], field: str
) -> dict[str, Any]:
    if not isinstance(obj, dict):
        raise ContractError(f"{field} must be an object")
    extra = set(obj) - allowed
    missing = required - set(obj)
    if extra:
        raise ContractError(f"{field} has unsupported fields: {sorted(extra)}")
    if missing:
        raise ContractError(f"{field} is missing required fields: {sorted(missing)}")
    return obj


def plan_identity(plan: dict[str, Any]) -> str:
    return identity("observable-support-plan-v1-sha256-", plan)


def normalize_plan(raw: Any) -> dict[str, Any]:
    raw = exact_keys(
        raw,
        {"schema", "planId", "entries"},
        {"schema", "planId", "entries"},
        "plan",
    )
    if raw["schema"] != PLAN_SCHEMA:
        raise ContractError(f"unsupported plan schema: {raw['schema']!r}")
    nonempty(raw["planId"], "plan.planId")
    entries = raw["entries"]
    if not isinstance(entries, list) or not entries:
        raise ContractError("plan.entries must be a non-empty array")

    seen_entries: set[str] = set()
    for idx, entry in enumerate(entries):
        field = f"plan.entries[{idx}]"
        if not isinstance(entry, dict):
            raise ContractError(f"{field} must be an object")
        observable_id = nonempty(entry.get("observableId"), f"{field}.observableId")
        if observable_id in seen_entries:
            raise ContractError(f"duplicate plan observableId: {observable_id}")
        seen_entries.add(observable_id)
        alternatives = entry.get("alternativeBinnings")
        if not isinstance(alternatives, list):
            raise ContractError(f"{field}.alternativeBinnings must be an array")
        seen_alt: set[str] = set()
        for alt_idx, alt in enumerate(alternatives):
            if not isinstance(alt, dict):
                raise ContractError(f"{field}.alternativeBinnings[{alt_idx}] must be an object")
            alt_id = nonempty(
                alt.get("id"), f"{field}.alternativeBinnings[{alt_idx}].id"
            )
            if alt_id == "primary":
                raise ContractError(f"{field}: alternative binning id 'primary' is reserved")
            if alt_id in seen_alt:
                raise ContractError(
                    f"{field}: duplicate alternative binning id {alt_id!r}"
                )
            seen_alt.add(alt_id)
    return raw


def validate_assessment(raw: Any, plan: dict[str, Any]) -> str:
    if not isinstance(raw, dict):
        raise ContractError("assessment must be an object")
    if raw.get("schema") != ASSESSMENT_SCHEMA:
        raise ContractError("unsupported observable-support assessment schema")
    assessment_identity = nonempty(
        raw.get("assessmentIdentity"), "assessment.assessmentIdentity"
    )
    expected_plan_identity = plan_identity(plan)
    if raw.get("planIdentity") != expected_plan_identity:
        raise ContractError("assessment.planIdentity does not match supplied support plan")
    assessment_entries = raw.get("entries")
    if assessment_entries != plan["entries"]:
        raise ContractError("assessment entries do not exactly match supplied support plan")
    return assessment_identity


def validate_execution(raw: Any, field: str) -> dict[str, str]:
    raw = exact_keys(
        raw,
        {"binningId", "analysisIdentity", "inferenceClass"},
        {"binningId", "analysisIdentity", "inferenceClass"},
        field,
    )
    return {
        "binningId": nonempty(raw["binningId"], f"{field}.binningId"),
        "analysisIdentity": nonempty(
            raw["analysisIdentity"], f"{field}.analysisIdentity"
        ),
        "inferenceClass": nonempty(raw["inferenceClass"], f"{field}.inferenceClass"),
    }


def normalize_report(
    raw: Any, plan: dict[str, Any], assessment_identity: str
) -> dict[str, Any]:
    raw = exact_keys(
        raw,
        {"schema", "supportAssessmentIdentity", "observableResults"},
        {"schema", "supportAssessmentIdentity", "observableResults"},
        "report",
    )
    if raw["schema"] != REPORT_SCHEMA:
        raise ContractError(f"unsupported report schema: {raw['schema']!r}")
    if raw["supportAssessmentIdentity"] != assessment_identity:
        raise ContractError(
            "report.supportAssessmentIdentity does not match supplied assessment"
        )
    results = raw["observableResults"]
    if not isinstance(results, list):
        raise ContractError("report.observableResults must be an array")

    plan_entries = {entry["observableId"]: entry for entry in plan["entries"]}
    seen: set[str] = set()
    normalized_results: list[dict[str, Any]] = []

    for idx, item in enumerate(results):
        field = f"report.observableResults[{idx}]"
        item = exact_keys(
            item,
            {
                "observableId",
                "primary",
                "alternatives",
                "materialScaleDependence",
                "dependenceStatement",
            },
            {
                "observableId",
                "primary",
                "alternatives",
                "materialScaleDependence",
                "dependenceStatement",
            },
            field,
        )
        observable_id = nonempty(item["observableId"], f"{field}.observableId")
        if observable_id in seen:
            raise ContractError(f"duplicate report observableId: {observable_id}")
        seen.add(observable_id)
        plan_entry = plan_entries.get(observable_id)
        if plan_entry is None:
            raise ContractError(
                f"report references unknown support-plan observable {observable_id!r}"
            )

        primary = validate_execution(item["primary"], f"{field}.primary")
        if primary["binningId"] != "primary":
            raise ContractError(f"{field}.primary.binningId must be 'primary'")

        alternatives_raw = item["alternatives"]
        if not isinstance(alternatives_raw, list):
            raise ContractError(f"{field}.alternatives must be an array")
        alternatives = [
            validate_execution(alt, f"{field}.alternatives[{alt_idx}]")
            for alt_idx, alt in enumerate(alternatives_raw)
        ]
        actual_ids = [alt["binningId"] for alt in alternatives]
        if len(actual_ids) != len(set(actual_ids)):
            raise ContractError(f"{field}.alternatives contains duplicate binning ids")
        expected_ids = [alt["id"] for alt in plan_entry["alternativeBinnings"]]
        if set(actual_ids) != set(expected_ids) or len(actual_ids) != len(expected_ids):
            raise ContractError(
                f"{field}.alternatives must execute exactly the declared alternative "
                f"binnings; expected {sorted(expected_ids)}, got {sorted(actual_ids)}"
            )

        material = item["materialScaleDependence"]
        if not isinstance(material, bool):
            raise ContractError(f"{field}.materialScaleDependence must be boolean")
        inferred_material = any(
            alt["inferenceClass"] != primary["inferenceClass"] for alt in alternatives
        )
        if material != inferred_material:
            raise ContractError(
                f"{field}.materialScaleDependence must equal whether substantive "
                "inferenceClass changes across declared binnings"
            )

        statement = item["dependenceStatement"]
        if material:
            nonempty(statement, f"{field}.dependenceStatement")
        elif statement is not None:
            nonempty(statement, f"{field}.dependenceStatement")

        normalized_results.append(
            {
                "observableId": observable_id,
                "primary": primary,
                "alternatives": alternatives,
                "materialScaleDependence": material,
                "dependenceStatement": statement,
            }
        )

    missing = set(plan_entries) - seen
    if missing:
        raise ContractError(
            f"report is missing support-plan observables: {sorted(missing)}"
        )

    normalized = {
        "schema": REPORT_SCHEMA,
        "supportAssessmentIdentity": assessment_identity,
        "observableResults": normalized_results,
    }
    normalized["reportIdentity"] = identity(
        "observable-support-sensitivity-report-v1-sha256-", normalized
    )
    return normalized


def write_new(path: Path, value: Any) -> None:
    if path.exists():
        existing = load_json(path)
        if existing == value:
            return
        raise ContractError(f"refusing to overwrite differing existing output: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_bytes(value))


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)

    validate = sub.add_parser("validate")
    validate.add_argument("--plan", required=True, type=Path)
    validate.add_argument("--assessment", required=True, type=Path)
    validate.add_argument("--report", required=True, type=Path)

    derive = sub.add_parser("derive")
    derive.add_argument("--plan", required=True, type=Path)
    derive.add_argument("--assessment", required=True, type=Path)
    derive.add_argument("--declaration", required=True, type=Path)
    derive.add_argument("--output", required=True, type=Path)

    verify = sub.add_parser("verify")
    verify.add_argument("--plan", required=True, type=Path)
    verify.add_argument("--assessment", required=True, type=Path)
    verify.add_argument("--report", required=True, type=Path)

    args = parser.parse_args()
    try:
        plan = normalize_plan(load_json(args.plan))
        assessment_raw = load_json(args.assessment)
        assessment_identity = validate_assessment(assessment_raw, plan)

        if args.cmd == "derive":
            report = normalize_report(
                load_json(args.declaration), plan, assessment_identity
            )
            write_new(args.output, report)
            print(report["reportIdentity"])
        else:
            raw_report = load_json(args.report)
            expected = normalize_report(
                {
                    "schema": raw_report.get("schema"),
                    "supportAssessmentIdentity": raw_report.get(
                        "supportAssessmentIdentity"
                    ),
                    "observableResults": raw_report.get("observableResults"),
                },
                plan,
                assessment_identity,
            )
            if args.cmd == "verify":
                if raw_report != expected:
                    raise ContractError(
                        "report does not match deterministic re-derivation"
                    )
            print(expected["reportIdentity"])
        return 0
    except ContractError as exc:
        parser.error(str(exc))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
