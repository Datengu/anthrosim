#!/usr/bin/env python3
"""Validate and bind spatial/temporal support for empirical model-data comparisons.

This is research-governance tooling. It does not alter simulation state or trajectories.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

SCHEMA = "anthrosim-observable-support-plan-v1"
ASSESSMENT_SCHEMA = "anthrosim-observable-support-assessment-v1"
BINDING_PREFIX = "observable-support-plan-v1:"
ANALYSIS_REQUIREMENT_KIND = "observable_support_sensitivity"


class ContractError(ValueError):
    pass


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot read JSON {path}: {exc}") from exc


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def identity(prefix: str, value: Any) -> str:
    return prefix + hashlib.sha256(canonical_bytes(value)).hexdigest()


def nonempty(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ContractError(f"{field} must be a non-empty string")
    return value


def exact_keys(obj: Any, allowed: set[str], required: set[str], field: str) -> dict[str, Any]:
    if not isinstance(obj, dict):
        raise ContractError(f"{field} must be an object")
    extra = set(obj) - allowed
    missing = required - set(obj)
    if extra:
        raise ContractError(f"{field} has unsupported fields: {sorted(extra)}")
    if missing:
        raise ContractError(f"{field} is missing required fields: {sorted(missing)}")
    return obj


def validate_support(support: Any, field: str) -> None:
    support = exact_keys(
        support,
        {"kind", "unit", "definition", "sourceIdentity"},
        {"kind", "unit", "definition"},
        field,
    )
    kind = nonempty(support["kind"], f"{field}.kind")
    if kind not in {"cell", "polygon", "transect", "catchment", "site", "phase", "interval", "instant", "other"}:
        raise ContractError(f"{field}.kind is unsupported: {kind}")
    nonempty(support["unit"], f"{field}.unit")
    nonempty(support["definition"], f"{field}.definition")
    if "sourceIdentity" in support:
        nonempty(support["sourceIdentity"], f"{field}.sourceIdentity")


def validate_aggregation(rule: Any, field: str) -> None:
    rule = exact_keys(
        rule,
        {"source", "operation", "grouping", "weighting", "missingDataRule"},
        {"source", "operation", "grouping", "weighting", "missingDataRule"},
        field,
    )
    for key in ("source", "operation", "grouping", "weighting", "missingDataRule"):
        nonempty(rule[key], f"{field}.{key}")


def normalize_plan(raw: Any) -> dict[str, Any]:
    raw = exact_keys(
        raw,
        {"schema", "planId", "entries"},
        {"schema", "planId", "entries"},
        "plan",
    )
    if raw["schema"] != SCHEMA:
        raise ContractError(f"unsupported plan schema: {raw['schema']!r}")
    nonempty(raw["planId"], "plan.planId")
    if not isinstance(raw["entries"], list) or not raw["entries"]:
        raise ContractError("plan.entries must be a non-empty array")

    seen: set[str] = set()
    normalized: list[dict[str, Any]] = []
    for idx, item in enumerate(raw["entries"]):
        field = f"plan.entries[{idx}]"
        item = exact_keys(
            item,
            {
                "id", "observableId", "empirical", "observedSpatialSupport",
                "observedTemporalSupport", "simulatedSpatialSupport",
                "simulatedTemporalSupport", "simulatedSpatialAggregation",
                "simulatedTemporalAggregation", "resolutionUncertainty",
                "alternativeBinnings", "dependenceReportingRule",
            },
            {
                "id", "observableId", "empirical", "simulatedSpatialSupport",
                "simulatedTemporalSupport", "simulatedSpatialAggregation",
                "simulatedTemporalAggregation", "resolutionUncertainty",
                "alternativeBinnings", "dependenceReportingRule",
            },
            field,
        )
        entry_id = nonempty(item["id"], f"{field}.id")
        if entry_id in seen:
            raise ContractError(f"duplicate support entry id: {entry_id}")
        seen.add(entry_id)
        nonempty(item["observableId"], f"{field}.observableId")
        if not isinstance(item["empirical"], bool):
            raise ContractError(f"{field}.empirical must be boolean")

        validate_support(item["simulatedSpatialSupport"], f"{field}.simulatedSpatialSupport")
        validate_support(item["simulatedTemporalSupport"], f"{field}.simulatedTemporalSupport")
        validate_aggregation(item["simulatedSpatialAggregation"], f"{field}.simulatedSpatialAggregation")
        validate_aggregation(item["simulatedTemporalAggregation"], f"{field}.simulatedTemporalAggregation")

        if item["empirical"]:
            if "observedSpatialSupport" not in item or "observedTemporalSupport" not in item:
                raise ContractError(f"{field}: empirical observable requires observed spatial and temporal support")
            validate_support(item["observedSpatialSupport"], f"{field}.observedSpatialSupport")
            validate_support(item["observedTemporalSupport"], f"{field}.observedTemporalSupport")
        elif "observedSpatialSupport" in item or "observedTemporalSupport" in item:
            raise ContractError(f"{field}: synthetic-only entry must not claim observed support")

        uncertainty = item["resolutionUncertainty"]
        if uncertainty not in {"fixed", "uncertain"}:
            raise ContractError(f"{field}.resolutionUncertainty must be fixed or uncertain")
        alternatives = item["alternativeBinnings"]
        if not isinstance(alternatives, list):
            raise ContractError(f"{field}.alternativeBinnings must be an array")
        if uncertainty == "uncertain" and not alternatives:
            raise ContractError(f"{field}: uncertain support requires at least one alternative binning")
        if uncertainty == "fixed" and alternatives:
            raise ContractError(f"{field}: fixed support must not declare alternative binnings")
        for alt_idx, alt in enumerate(alternatives):
            alt = exact_keys(
                alt,
                {"id", "spatialAggregation", "temporalAggregation", "rationale"},
                {"id", "spatialAggregation", "temporalAggregation", "rationale"},
                f"{field}.alternativeBinnings[{alt_idx}]",
            )
            nonempty(alt["id"], f"{field}.alternativeBinnings[{alt_idx}].id")
            validate_aggregation(alt["spatialAggregation"], f"{field}.alternativeBinnings[{alt_idx}].spatialAggregation")
            validate_aggregation(alt["temporalAggregation"], f"{field}.alternativeBinnings[{alt_idx}].temporalAggregation")
            nonempty(alt["rationale"], f"{field}.alternativeBinnings[{alt_idx}].rationale")
        nonempty(item["dependenceReportingRule"], f"{field}.dependenceReportingRule")
        normalized.append(item)

    return {"schema": SCHEMA, "planId": raw["planId"], "entries": normalized}


def plan_identity(plan: dict[str, Any]) -> str:
    return identity("observable-support-plan-v1-sha256-", plan)


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def protocol_identity(protocol: dict[str, Any]) -> str:
    # Match StudyProtocol::identity(): canonical JSON bytes + FNV-1a 64.
    return f"study-protocol-v1-{fnv1a64(canonical_bytes(protocol).rstrip(b'\n')):016x}"


def find_binding(interpretation: str) -> str | None:
    for token in interpretation.replace(";", " ").split():
        if token.startswith(BINDING_PREFIX):
            return token[len(BINDING_PREFIX):]
    return None


def validate_protocol_binding(protocol: Any, plan: dict[str, Any]) -> str:
    if not isinstance(protocol, dict):
        raise ContractError("protocol must be an object")
    if protocol.get("schemaVersion") != 1:
        raise ContractError("only StudyProtocol schemaVersion 1 is supported")
    observables = protocol.get("observables")
    if not isinstance(observables, list):
        raise ContractError("protocol.observables must be an array")
    expected = plan_identity(plan)
    entries_by_observable = {entry["observableId"]: entry for entry in plan["entries"]}
    protocol_ids: set[str] = set()
    for idx, observable in enumerate(observables):
        if not isinstance(observable, dict):
            raise ContractError(f"protocol.observables[{idx}] must be an object")
        observable_id = nonempty(observable.get("id"), f"protocol.observables[{idx}].id")
        interpretation = nonempty(observable.get("interpretation"), f"protocol.observables[{idx}].interpretation")
        protocol_ids.add(observable_id)
        entry = entries_by_observable.get(observable_id)
        if entry is None:
            # Synthetic mechanism observables may omit a support plan; empirical comparison observables may not.
            continue
        binding = find_binding(interpretation)
        if binding != expected:
            raise ContractError(
                f"observable {observable_id} must bind exact plan identity with "
                f"'{BINDING_PREFIX}{expected}'"
            )
    unknown = set(entries_by_observable) - protocol_ids
    if unknown:
        raise ContractError(f"support plan references unknown protocol observables: {sorted(unknown)}")
    return expected


def verify_empirical_comparison_coverage(protocol: dict[str, Any], plan: dict[str, Any]) -> None:
    entries = {entry["observableId"]: entry for entry in plan["entries"]}
    comparisons = protocol.get("comparisons", [])
    if not isinstance(comparisons, list):
        raise ContractError("protocol.comparisons must be an array")
    for idx, comparison in enumerate(comparisons):
        if not isinstance(comparison, dict):
            raise ContractError(f"protocol.comparisons[{idx}] must be an object")
        ids = comparison.get("observableIds")
        if not isinstance(ids, list):
            raise ContractError(f"protocol.comparisons[{idx}].observableIds must be an array")
        for observable_id in ids:
            entry = entries.get(observable_id)
            if entry is None:
                raise ContractError(
                    f"empirical comparison {comparison.get('id', idx)!r} lacks support declaration for observable {observable_id!r}"
                )
            if not entry["empirical"]:
                raise ContractError(
                    f"empirical comparison {comparison.get('id', idx)!r} marks observable {observable_id!r} as synthetic-only"
                )
            # Full observed/simulated support + aggregation were validated above.


def build_assessment(protocol: dict[str, Any], plan: dict[str, Any], result_binding: dict[str, Any] | None) -> dict[str, Any]:
    pid = validate_protocol_binding(protocol, plan)
    verify_empirical_comparison_coverage(protocol, plan)
    assessment: dict[str, Any] = {
        "schema": ASSESSMENT_SCHEMA,
        "planIdentity": pid,
        "protocolIdentity": protocol_identity(protocol),
        "studyId": nonempty(protocol.get("studyId"), "protocol.studyId"),
        "entries": plan["entries"],
    }
    if result_binding is not None:
        if not isinstance(result_binding, dict):
            raise ContractError("study result binding must be an object")
        for key in ("resultIdentity", "studyExecutionId", "protocolIdentity", "researchId"):
            nonempty(result_binding.get(key), f"studyResultBinding.{key}")
        if result_binding["protocolIdentity"] != assessment["protocolIdentity"]:
            raise ContractError("study result binding protocolIdentity does not match supplied frozen protocol")

        requirements = result_binding.get("analysisRequirements")
        if not isinstance(requirements, list):
            raise ContractError(
                "study result binding must declare analysisRequirements for a support-bound protocol"
            )
        seen_requirements: set[tuple[str, str]] = set()
        expected_requirement = (ANALYSIS_REQUIREMENT_KIND, pid)
        for idx, requirement in enumerate(requirements):
            field = f"studyResultBinding.analysisRequirements[{idx}]"
            requirement = exact_keys(
                requirement, {"kind", "identity"}, {"kind", "identity"}, field
            )
            pair = (
                nonempty(requirement["kind"], f"{field}.kind"),
                nonempty(requirement["identity"], f"{field}.identity"),
            )
            if pair in seen_requirements:
                raise ContractError(f"duplicate study analysis requirement: {pair}")
            seen_requirements.add(pair)
        if expected_requirement not in seen_requirements:
            raise ContractError(
                "study result binding does not require the exact observable-support sensitivity plan"
            )

        assessment["sourceStudyExecutionId"] = result_binding["studyExecutionId"]
        assessment["sourceStudyResultIdentity"] = result_binding["resultIdentity"]
        assessment["sourceResearchId"] = result_binding["researchId"]
    assessment["assessmentIdentity"] = identity("observable-support-assessment-v1-sha256-", assessment)
    return assessment


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
    validate.add_argument("--protocol", required=True, type=Path)

    derive = sub.add_parser("derive")
    derive.add_argument("--plan", required=True, type=Path)
    derive.add_argument("--protocol", required=True, type=Path)
    derive.add_argument("--study-result-binding", type=Path)
    derive.add_argument("--output", required=True, type=Path)

    verify = sub.add_parser("verify")
    verify.add_argument("--plan", required=True, type=Path)
    verify.add_argument("--protocol", required=True, type=Path)
    verify.add_argument("--study-result-binding", type=Path)
    verify.add_argument("--assessment", required=True, type=Path)

    args = parser.parse_args()
    try:
        plan = normalize_plan(load_json(args.plan))
        protocol = load_json(args.protocol)
        binding = load_json(args.study_result_binding) if getattr(args, "study_result_binding", None) else None
        assessment = build_assessment(protocol, plan, binding)
        if args.cmd == "validate":
            print(json.dumps({"planIdentity": assessment["planIdentity"], "protocolIdentity": assessment["protocolIdentity"]}, sort_keys=True))
        elif args.cmd == "derive":
            write_new(args.output, assessment)
            print(assessment["assessmentIdentity"])
        else:
            existing = load_json(args.assessment)
            if existing != assessment:
                raise ContractError("assessment does not match deterministic re-derivation")
            print(assessment["assessmentIdentity"])
        return 0
    except ContractError as exc:
        parser.error(str(exc))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
