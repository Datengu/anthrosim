#!/usr/bin/env python3
"""Validate execution/reporting of predeclared observable-support sensitivity analyses."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
from typing import Any

PLAN_SCHEMA = "anthrosim-observable-support-plan-v1"
ASSESSMENT_SCHEMA = "anthrosim-observable-support-assessment-v1"
REPORT_SCHEMA = "anthrosim-observable-support-sensitivity-report-v1"
BINNING_SCHEMA = "anthrosim-observable-support-binning-v1"
INFERENCE_SCHEMA = "anthrosim-observable-support-inference-v1"
PROVENANCE_PREFIX = "analysis-provenance-v2-sha256-"
SUPPORT_ANALYSIS_ROOT = PurePosixPath("analysis/observable-support")
SUPPORT_BINNING_ROLE = "observable-support-binning-definition"
SUPPORT_INFERENCE_ROLE = "observable-support-inference"


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


def validate_assessment(raw: Any, plan: dict[str, Any]) -> dict[str, Any]:
    raw = exact_keys(
        raw,
        {
            "schema",
            "planIdentity",
            "protocolIdentity",
            "studyId",
            "entries",
            "sourceStudyExecutionId",
            "sourceStudyResultIdentity",
            "sourceResearchId",
            "assessmentIdentity",
        },
        {
            "schema",
            "planIdentity",
            "protocolIdentity",
            "studyId",
            "entries",
            "sourceStudyExecutionId",
            "sourceStudyResultIdentity",
            "sourceResearchId",
            "assessmentIdentity",
        },
        "assessment",
    )
    if raw["schema"] != ASSESSMENT_SCHEMA:
        raise ContractError("unsupported observable-support assessment schema")
    expected_plan_identity = plan_identity(plan)
    if raw["planIdentity"] != expected_plan_identity:
        raise ContractError("assessment.planIdentity does not match supplied support plan")
    if raw["entries"] != plan["entries"]:
        raise ContractError("assessment entries do not exactly match supplied support plan")
    for key in (
        "protocolIdentity",
        "studyId",
        "sourceStudyExecutionId",
        "sourceStudyResultIdentity",
        "sourceResearchId",
    ):
        nonempty(raw[key], f"assessment.{key}")
    claimed_identity = nonempty(raw["assessmentIdentity"], "assessment.assessmentIdentity")
    payload = dict(raw)
    del payload["assessmentIdentity"]
    expected_identity = identity("observable-support-assessment-v1-sha256-", payload)
    if claimed_identity != expected_identity:
        raise ContractError("assessment.assessmentIdentity does not match assessment contents")
    return raw


def expected_binning_definition(
    plan_entry: dict[str, Any], binning_id: str, assessment_identity: str
) -> dict[str, Any]:
    if binning_id == "primary":
        spatial = plan_entry["simulatedSpatialAggregation"]
        temporal = plan_entry["simulatedTemporalAggregation"]
    else:
        matches = [
            alt for alt in plan_entry["alternativeBinnings"] if alt["id"] == binning_id
        ]
        if len(matches) != 1:
            raise ContractError(
                f"support plan does not define exactly one binning {binning_id!r} "
                f"for observable {plan_entry['observableId']!r}"
            )
        spatial = matches[0]["spatialAggregation"]
        temporal = matches[0]["temporalAggregation"]

    definition: dict[str, Any] = {
        "schema": BINNING_SCHEMA,
        "supportAssessmentIdentity": assessment_identity,
        "observableId": plan_entry["observableId"],
        "binningId": binning_id,
        "empirical": plan_entry.get("empirical", False),
        "simulatedSpatialSupport": plan_entry["simulatedSpatialSupport"],
        "simulatedTemporalSupport": plan_entry["simulatedTemporalSupport"],
        "spatialAggregation": spatial,
        "temporalAggregation": temporal,
    }
    if plan_entry.get("empirical", False):
        definition["observedSpatialSupport"] = plan_entry["observedSpatialSupport"]
        definition["observedTemporalSupport"] = plan_entry["observedTemporalSupport"]
    return definition


def safe_relative_path(root: Path, raw: Any, field: str) -> Path:
    text = nonempty(raw, field)
    if "\\" in text:
        raise ContractError(f"{field} must use POSIX separators")
    pure = PurePosixPath(text)
    if (
        pure.is_absolute()
        or text == "."
        or any(part in ("", ".", "..") for part in pure.parts)
        or pure.as_posix() != text
    ):
        raise ContractError(f"{field} must be a safe canonical relative path")
    candidate = (root / Path(pure)).resolve(strict=True)
    try:
        candidate.relative_to(root)
    except ValueError as exc:
        raise ContractError(f"{field} escapes the study root") from exc
    if candidate.is_symlink() or not candidate.is_file():
        raise ContractError(f"{field} must resolve to a regular file")
    return candidate


def provenance_module():
    script = Path(__file__).with_name("research-analysis-provenance.py")
    spec = importlib.util.spec_from_file_location("anthrosim_analysis_provenance", script)
    if spec is None or spec.loader is None:
        raise ContractError("cannot load research-analysis-provenance.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_analysis_registry(study_root: Path) -> dict[str, dict[str, Any]]:
    if study_root.is_symlink() or not study_root.is_dir():
        raise ContractError("study root must be an existing non-symlink directory")
    root = study_root.resolve(strict=True)
    analysis_root = root / Path(SUPPORT_ANALYSIS_ROOT)
    if not analysis_root.is_dir():
        raise ContractError(
            f"support analysis provenance root is missing: {SUPPORT_ANALYSIS_ROOT.as_posix()}"
        )
    module = provenance_module()
    registry: dict[str, dict[str, Any]] = {}
    record_paths = sorted(analysis_root.rglob("analysis-provenance.json"))
    if not record_paths:
        raise ContractError("no support analysis provenance records were found")
    for path in record_paths:
        if path.is_symlink() or not path.is_file():
            raise ContractError(f"support analysis provenance record is not a regular file: {path}")
        relative = path.relative_to(root)
        try:
            record = module.verify_record(root, relative)
        except Exception as exc:  # exact provenance verifier owns detailed contract errors
            raise ContractError(
                f"support analysis provenance verification failed for {relative.as_posix()}: {exc}"
            ) from exc
        provenance_identity = nonempty(
            record.get("provenanceIdentity"),
            f"analysis provenance {relative.as_posix()}.provenanceIdentity",
        )
        if not provenance_identity.startswith(PROVENANCE_PREFIX):
            raise ContractError(
                f"support analysis uses unsupported provenance identity {provenance_identity!r}"
            )
        if record.get("executionStatus") != "executed_by_wrapper":
            raise ContractError(
                f"support analysis {provenance_identity!r} was not executed by the provenance wrapper"
            )
        if provenance_identity in registry:
            raise ContractError(
                f"duplicate support analysis provenance identity: {provenance_identity}"
            )
        registry[provenance_identity] = record
    return registry


def artifact_by_role(
    definition: dict[str, Any], category: str, role: str, field: str
) -> dict[str, str]:
    items = [item for item in definition[category] if item.get("role") == role]
    if len(items) != 1:
        raise ContractError(
            f"{field} must declare exactly one {category} artifact with role {role!r}"
        )
    return items[0]


def resolve_execution(
    raw: Any,
    field: str,
    *,
    observable_id: str,
    plan_entry: dict[str, Any],
    assessment: dict[str, Any],
    registry: dict[str, dict[str, Any]],
    study_root: Path,
    used_analysis_ids: set[str],
) -> dict[str, str]:
    raw = exact_keys(
        raw,
        {"binningId", "analysisIdentity", "inferenceClass"},
        {"binningId", "analysisIdentity", "inferenceClass"},
        field,
    )
    binning_id = nonempty(raw["binningId"], f"{field}.binningId")
    analysis_identity = nonempty(raw["analysisIdentity"], f"{field}.analysisIdentity")
    inference_class = nonempty(raw["inferenceClass"], f"{field}.inferenceClass")
    if analysis_identity in used_analysis_ids:
        raise ContractError(
            f"analysisIdentity {analysis_identity!r} is reused by multiple support executions"
        )
    used_analysis_ids.add(analysis_identity)

    record = registry.get(analysis_identity)
    if record is None:
        raise ContractError(
            f"{field}.analysisIdentity does not resolve to an integrity-checked support analysis: "
            f"{analysis_identity!r}"
        )
    study = record.get("study")
    if not isinstance(study, dict):
        raise ContractError(f"{field} resolved provenance record has no study binding")
    expected_study_fields = {
        "studyExecutionId": assessment["sourceStudyExecutionId"],
        "resultIdentity": assessment["sourceStudyResultIdentity"],
        "researchId": assessment["sourceResearchId"],
        "protocolIdentity": assessment["protocolIdentity"],
    }
    for key, expected in expected_study_fields.items():
        if study.get(key) != expected:
            raise ContractError(
                f"{field} resolved analysis study.{key} does not match the finalized support assessment"
            )

    definition = record.get("definition")
    if not isinstance(definition, dict):
        raise ContractError(f"{field} resolved provenance record has no analysis definition")
    config_spec = artifact_by_role(
        definition, "inputs", SUPPORT_BINNING_ROLE, field
    )
    config_path_text = config_spec["path"]
    if config_path_text not in definition.get("command", []):
        raise ContractError(
            f"{field} support binning definition is not an executed command argument"
        )
    root = study_root.resolve(strict=True)
    config_path = safe_relative_path(root, config_path_text, f"{field}.binningDefinition")
    config = load_json(config_path)
    expected_config = expected_binning_definition(
        plan_entry, binning_id, assessment["assessmentIdentity"]
    )
    if config != expected_config:
        raise ContractError(
            f"{field} resolved analysis does not execute the exact declared observable/binning semantics"
        )

    inference_spec = artifact_by_role(
        definition, "outputs", SUPPORT_INFERENCE_ROLE, field
    )
    inference_path = safe_relative_path(
        root, inference_spec["path"], f"{field}.inferenceArtifact"
    )
    inference = exact_keys(
        load_json(inference_path),
        {
            "schema",
            "supportAssessmentIdentity",
            "observableId",
            "binningId",
            "inferenceClass",
        },
        {
            "schema",
            "supportAssessmentIdentity",
            "observableId",
            "binningId",
            "inferenceClass",
        },
        f"{field}.inferenceArtifact",
    )
    if inference["schema"] != INFERENCE_SCHEMA:
        raise ContractError(f"{field} resolved inference artifact has unsupported schema")
    expected_inference = {
        "supportAssessmentIdentity": assessment["assessmentIdentity"],
        "observableId": observable_id,
        "binningId": binning_id,
        "inferenceClass": inference_class,
    }
    for key, expected in expected_inference.items():
        if inference.get(key) != expected:
            raise ContractError(
                f"{field} reported {key} does not match the fingerprinted analysis output"
            )

    return {
        "binningId": binning_id,
        "analysisIdentity": analysis_identity,
        "inferenceClass": inference_class,
    }


def normalize_report(
    raw: Any,
    plan: dict[str, Any],
    assessment: dict[str, Any],
    registry: dict[str, dict[str, Any]],
    study_root: Path,
) -> dict[str, Any]:
    raw = exact_keys(
        raw,
        {"schema", "supportAssessmentIdentity", "observableResults"},
        {"schema", "supportAssessmentIdentity", "observableResults"},
        "report",
    )
    if raw["schema"] != REPORT_SCHEMA:
        raise ContractError(f"unsupported report schema: {raw['schema']!r}")
    if raw["supportAssessmentIdentity"] != assessment["assessmentIdentity"]:
        raise ContractError(
            "report.supportAssessmentIdentity does not match supplied assessment"
        )
    results = raw["observableResults"]
    if not isinstance(results, list):
        raise ContractError("report.observableResults must be an array")

    plan_entries = {entry["observableId"]: entry for entry in plan["entries"]}
    seen: set[str] = set()
    used_analysis_ids: set[str] = set()
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

        primary = resolve_execution(
            item["primary"],
            f"{field}.primary",
            observable_id=observable_id,
            plan_entry=plan_entry,
            assessment=assessment,
            registry=registry,
            study_root=study_root,
            used_analysis_ids=used_analysis_ids,
        )
        if primary["binningId"] != "primary":
            raise ContractError(f"{field}.primary.binningId must be 'primary'")

        alternatives_raw = item["alternatives"]
        if not isinstance(alternatives_raw, list):
            raise ContractError(f"{field}.alternatives must be an array")
        alternatives = [
            resolve_execution(
                alt,
                f"{field}.alternatives[{alt_idx}]",
                observable_id=observable_id,
                plan_entry=plan_entry,
                assessment=assessment,
                registry=registry,
                study_root=study_root,
                used_analysis_ids=used_analysis_ids,
            )
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
        "supportAssessmentIdentity": assessment["assessmentIdentity"],
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

    binning = sub.add_parser("binning-definition")
    binning.add_argument("--plan", required=True, type=Path)
    binning.add_argument("--assessment", required=True, type=Path)
    binning.add_argument("--observable-id", required=True)
    binning.add_argument("--binning-id", required=True)
    binning.add_argument("--output", required=True, type=Path)

    for name in ("validate", "derive", "verify"):
        command = sub.add_parser(name)
        command.add_argument("--study-root", required=True, type=Path)
        command.add_argument("--plan", required=True, type=Path)
        command.add_argument("--assessment", required=True, type=Path)
        if name == "derive":
            command.add_argument("--declaration", required=True, type=Path)
            command.add_argument("--output", required=True, type=Path)
        else:
            command.add_argument("--report", required=True, type=Path)

    args = parser.parse_args()
    try:
        plan = normalize_plan(load_json(args.plan))
        assessment = validate_assessment(load_json(args.assessment), plan)

        if args.cmd == "binning-definition":
            entries = [
                entry for entry in plan["entries"] if entry["observableId"] == args.observable_id
            ]
            if len(entries) != 1:
                raise ContractError(
                    f"observable {args.observable_id!r} does not resolve uniquely in support plan"
                )
            definition = expected_binning_definition(
                entries[0], args.binning_id, assessment["assessmentIdentity"]
            )
            write_new(args.output, definition)
            print(identity("observable-support-binning-v1-sha256-", definition))
            return 0

        registry = load_analysis_registry(args.study_root)
        if args.cmd == "derive":
            report = normalize_report(
                load_json(args.declaration), plan, assessment, registry, args.study_root
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
                assessment,
                registry,
                args.study_root,
            )
            if args.cmd == "verify" and raw_report != expected:
                raise ContractError("report does not match deterministic re-derivation")
            print(expected["reportIdentity"])
        return 0
    except ContractError as exc:
        parser.error(str(exc))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
