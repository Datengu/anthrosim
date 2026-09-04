#!/usr/bin/env python3
"""Verify finalized AnthroSim study-result bindings against their producer contract."""

from __future__ import annotations

import json
import os
import stat
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA_VERSION = 1
RESULT_PREFIX = "study-result-v1"
STUDY_EXECUTION_PREFIX = "study-execution-v1"
PROTOCOL_PREFIX = "study-protocol-v1"
DEFINITION_PREFIX = "research-definition-v1"
RESEARCH_EXECUTION_PREFIX = "research-execution-v1"
RESEARCH_DIR = "research"
EXPECTED_RESULT_ARTIFACTS = (
    "research/analysis/points.json",
    "research/analysis/runs.json",
)
OBSERVABLE_SUPPORT_BINDING_PREFIX = "observable-support-plan-v1:"
OBSERVABLE_SUPPORT_REQUIREMENT_KIND = "observable_support_sensitivity"


class StudyBindingError(Exception):
    """Raised when a finalized study binding is not producer-valid."""


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise StudyBindingError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise StudyBindingError(f"{role} must be a regular non-symlink file: {path}")
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except StudyBindingError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise StudyBindingError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise StudyBindingError(f"{role} root must be a JSON object")
    return value


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x00000100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def stable_identity(prefix: str, value: Any) -> str:
    return f"{prefix}-{fnv1a64(canonical_bytes(value)):016x}"


def nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise StudyBindingError(f"{role} must be a non-empty string")
    return value


def uint(value: Any, role: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise StudyBindingError(f"{role} must be a non-negative integer")
    return value


def exact_keys(
    value: Any, required: set[str], optional: set[str], role: str
) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise StudyBindingError(f"{role} must be an object")
    keys = set(value)
    missing = required - keys
    unknown = keys - required - optional
    if missing:
        raise StudyBindingError(
            f"{role} is missing required field(s): {', '.join(sorted(missing))}"
        )
    if unknown:
        raise StudyBindingError(
            f"{role} contains unknown field(s): {', '.join(sorted(unknown))}"
        )
    return value


def canonical_relative_path(value: Any, role: str) -> str:
    raw = nonempty_string(value, role)
    if "\\" in raw:
        raise StudyBindingError(f"{role} must use POSIX separators")
    path = PurePosixPath(raw)
    if (
        path.is_absolute()
        or raw == "."
        or any(part in {"", ".", ".."} for part in path.parts)
        or path.as_posix() != raw
    ):
        raise StudyBindingError(f"{role} must be a canonical relative path")
    return raw


def validate_source(source: Any, role: str = "study result binding.source") -> dict[str, Any]:
    source = exact_keys(
        source,
        {"modelVersion", "modelSemanticsId", "gitCommit"},
        set(),
        role,
    )
    nonempty_string(source["modelVersion"], f"{role}.modelVersion")
    nonempty_string(source["modelSemanticsId"], f"{role}.modelSemanticsId")
    nonempty_string(source["gitCommit"], f"{role}.gitCommit")
    return source


def normalize_analysis_requirements(raw: Any) -> list[dict[str, str]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise StudyBindingError("study result binding.analysisRequirements must be an array")
    output: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for index, item in enumerate(raw):
        item = exact_keys(
            item,
            {"kind", "identity"},
            set(),
            f"study result binding.analysisRequirements[{index}]",
        )
        pair = (
            nonempty_string(item["kind"], f"analysisRequirements[{index}].kind"),
            nonempty_string(item["identity"], f"analysisRequirements[{index}].identity"),
        )
        if pair in seen:
            raise StudyBindingError(f"duplicate study analysis requirement: {pair}")
        seen.add(pair)
        output.append({"kind": pair[0], "identity": pair[1]})
    if output != sorted(output, key=lambda item: (item["kind"], item["identity"])):
        raise StudyBindingError("study result binding.analysisRequirements must be sorted canonically")
    return output


def normalize_result_artifacts(raw: Any) -> list[dict[str, Any]]:
    if not isinstance(raw, list):
        raise StudyBindingError("study result binding.resultArtifacts must be an array")
    output: list[dict[str, Any]] = []
    seen: set[str] = set()
    for index, item in enumerate(raw):
        item = exact_keys(
            item,
            {"path", "digest64"},
            set(),
            f"study result binding.resultArtifacts[{index}]",
        )
        path = canonical_relative_path(item["path"], f"resultArtifacts[{index}].path")
        digest = uint(item["digest64"], f"resultArtifacts[{index}].digest64")
        if digest > 0xFFFFFFFFFFFFFFFF:
            raise StudyBindingError(f"resultArtifacts[{index}].digest64 exceeds u64")
        if path in seen:
            raise StudyBindingError(f"duplicate study result artifact path: {path}")
        seen.add(path)
        output.append({"path": path, "digest64": digest})
    return output


def normalize_run_counts(raw: Any) -> dict[str, int]:
    raw = exact_keys(raw, {"completed", "failed"}, set(), "study result binding.runCounts")
    return {
        "completed": uint(raw["completed"], "runCounts.completed"),
        "failed": uint(raw["failed"], "runCounts.failed"),
    }


def normalize_binding(binding: Any) -> dict[str, Any]:
    binding = exact_keys(
        binding,
        {
            "schemaVersion",
            "resultIdentity",
            "studyExecutionId",
            "protocolIdentity",
            "protocolRevision",
            "studyId",
            "scientificStatus",
            "boundBeforeExecution",
            "confirmatoryPreResultClaimEligible",
            "definitionIdentity",
            "researchId",
            "source",
            "researchRelativeDir",
            "runCounts",
            "resultArtifacts",
        },
        {"analysisRequirements"},
        "study result binding",
    )
    if binding["schemaVersion"] != SCHEMA_VERSION:
        raise StudyBindingError(
            f"unsupported study result binding schema {binding['schemaVersion']!r}; expected {SCHEMA_VERSION}"
        )
    status = nonempty_string(binding["scientificStatus"], "scientificStatus")
    if status not in {"exploratory", "confirmatory"}:
        raise StudyBindingError("study result binding has invalid scientificStatus")
    for field in (
        "resultIdentity",
        "studyExecutionId",
        "protocolIdentity",
        "studyId",
        "definitionIdentity",
        "researchId",
    ):
        nonempty_string(binding[field], f"study result binding.{field}")
    uint(binding["protocolRevision"], "study result binding.protocolRevision")
    if not isinstance(binding["boundBeforeExecution"], bool) or not isinstance(
        binding["confirmatoryPreResultClaimEligible"], bool
    ):
        raise StudyBindingError("study result binding eligibility flags must be booleans")
    source = validate_source(binding["source"])
    research_relative_dir = canonical_relative_path(
        binding["researchRelativeDir"], "study result binding.researchRelativeDir"
    )
    if research_relative_dir != RESEARCH_DIR:
        raise StudyBindingError(
            "study result binding.researchRelativeDir must be the schema-v1 fixed research root"
        )
    normalized = {
        "schemaVersion": SCHEMA_VERSION,
        "resultIdentity": binding["resultIdentity"],
        "studyExecutionId": binding["studyExecutionId"],
        "protocolIdentity": binding["protocolIdentity"],
        "protocolRevision": binding["protocolRevision"],
        "studyId": binding["studyId"],
        "scientificStatus": status,
        "boundBeforeExecution": binding["boundBeforeExecution"],
        "confirmatoryPreResultClaimEligible": binding[
            "confirmatoryPreResultClaimEligible"
        ],
        "definitionIdentity": binding["definitionIdentity"],
        "researchId": binding["researchId"],
        "source": source,
        "researchRelativeDir": research_relative_dir,
        "runCounts": normalize_run_counts(binding["runCounts"]),
        "resultArtifacts": normalize_result_artifacts(binding["resultArtifacts"]),
        "analysisRequirements": normalize_analysis_requirements(
            binding.get("analysisRequirements")
        ),
    }
    return normalized


def result_identity(binding: Any) -> str:
    binding = normalize_binding(binding)
    payload = {
        "schemaVersion": binding["schemaVersion"],
        "studyExecutionId": binding["studyExecutionId"],
        "protocolIdentity": binding["protocolIdentity"],
        "protocolRevision": binding["protocolRevision"],
        "studyId": binding["studyId"],
        "scientificStatus": binding["scientificStatus"],
        "boundBeforeExecution": binding["boundBeforeExecution"],
        "confirmatoryPreResultClaimEligible": binding[
            "confirmatoryPreResultClaimEligible"
        ],
        "definitionIdentity": binding["definitionIdentity"],
        "researchId": binding["researchId"],
        "source": binding["source"],
        "researchRelativeDir": binding["researchRelativeDir"],
        "runCounts": binding["runCounts"],
        "resultArtifacts": binding["resultArtifacts"],
    }
    if binding["analysisRequirements"]:
        payload["analysisRequirements"] = binding["analysisRequirements"]
    return stable_identity(RESULT_PREFIX, payload)


def validate_result_binding(binding: Any) -> dict[str, Any]:
    normalized = normalize_binding(binding)
    expected = result_identity(normalized)
    if normalized["resultIdentity"] != expected:
        raise StudyBindingError(
            "study result binding resultIdentity does not match producer-defined binding contents; "
            f"expected {expected}"
        )
    return normalized


def protocol_identity(protocol: Any) -> str:
    if not isinstance(protocol, dict):
        raise StudyBindingError("frozen study protocol must be an object")
    if protocol.get("schemaVersion") != 1:
        raise StudyBindingError("unsupported frozen study protocol schema")
    return stable_identity(PROTOCOL_PREFIX, protocol)


def definition_identity(definition: Any) -> str:
    if not isinstance(definition, dict):
        raise StudyBindingError("frozen research definition must be an object")
    if definition.get("schemaVersion") != 1:
        raise StudyBindingError("unsupported frozen research definition schema")
    return stable_identity(DEFINITION_PREFIX, definition)


def study_execution_identity(
    protocol_id: str, definition_id: str, source: dict[str, Any]
) -> str:
    return stable_identity(
        STUDY_EXECUTION_PREFIX,
        {
            "schemaVersion": 1,
            "protocolIdentity": protocol_id,
            "definitionIdentity": definition_id,
            "source": source,
        },
    )


def research_execution_identity(definition_id: str, source: dict[str, Any]) -> str:
    return stable_identity(
        RESEARCH_EXECUTION_PREFIX,
        {
            "schemaVersion": 1,
            "definitionIdentity": definition_id,
            "source": source,
        },
    )


def analysis_requirements_from_protocol(protocol: dict[str, Any]) -> list[dict[str, str]]:
    observables = protocol.get("observables")
    if not isinstance(observables, list):
        raise StudyBindingError("frozen study protocol observables must be an array")
    requirements: set[tuple[str, str]] = set()
    for index, observable in enumerate(observables):
        if not isinstance(observable, dict):
            raise StudyBindingError(f"study protocol observable {index} must be an object")
        observable_id = nonempty_string(
            observable.get("id"), f"study protocol observable {index}.id"
        )
        interpretation = nonempty_string(
            observable.get("interpretation"),
            f"study protocol observable {observable_id}.interpretation",
        )
        normalized = interpretation.replace(";", " ")
        for token in normalized.split():
            if not token.startswith(OBSERVABLE_SUPPORT_BINDING_PREFIX):
                continue
            identity = token[len(OBSERVABLE_SUPPORT_BINDING_PREFIX) :]
            if not identity.startswith("observable-support-plan-v1-sha256-") or len(
                identity
            ) <= len("observable-support-plan-v1-sha256-"):
                raise StudyBindingError(
                    f"observable {observable_id} has malformed observable-support plan binding"
                )
            requirements.add((OBSERVABLE_SUPPORT_REQUIREMENT_KIND, identity))
    return [
        {"kind": kind, "identity": identity}
        for kind, identity in sorted(requirements)
    ]


def regular_file_bytes(path: Path, role: str) -> bytes:
    if path.is_symlink():
        raise StudyBindingError(f"{role} must not be a symbolic link: {path}")
    try:
        mode = os.lstat(path).st_mode
    except FileNotFoundError as error:
        raise StudyBindingError(f"{role} is missing: {path}") from error
    if not stat.S_ISREG(mode):
        raise StudyBindingError(f"{role} must be a regular file: {path}")
    return path.read_bytes()


def resolve_inside(root: Path, relative: str, role: str) -> Path:
    path = root / Path(PurePosixPath(relative))
    if path.is_symlink():
        raise StudyBindingError(f"{role} must not be a symbolic link: {path}")
    try:
        resolved = path.resolve(strict=True)
    except FileNotFoundError as error:
        raise StudyBindingError(f"{role} is missing: {path}") from error
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise StudyBindingError(f"{role} escapes the study root: {relative}") from error
    return resolved


def completed_run_counts(state: dict[str, Any], research_id: str) -> dict[str, int]:
    if state.get("schemaVersion") != 1 or state.get("researchId") != research_id:
        raise StudyBindingError(
            "research-state.json does not match the finalized research execution"
        )
    runs = state.get("runs")
    if not isinstance(runs, dict):
        raise StudyBindingError("research-state.json runs must be an object")
    completed = 0
    failed = 0
    for run_id, run in runs.items():
        nonempty_string(run_id, "research-state run id")
        if not isinstance(run, dict):
            raise StudyBindingError(f"research-state run {run_id} must be an object")
        status = run.get("state")
        if status == "completed":
            completed += 1
        elif status == "failed":
            failed += 1
        elif status in {"planned", "running"}:
            raise StudyBindingError(
                f"research execution is not finalized; run {run_id} remains {status}"
            )
        else:
            raise StudyBindingError(
                f"research-state run {run_id} has unknown state {status!r}"
            )
    return {"completed": completed, "failed": failed}


def validate_study_root(study_root: Path) -> dict[str, Any]:
    if study_root.is_symlink() or not study_root.is_dir():
        raise StudyBindingError(
            f"study root must be an existing non-symlink directory: {study_root}"
        )
    root = study_root.resolve(strict=True)
    plan = load_json(root / "study-plan.json", "immutable study plan")
    manifest = load_json(root / "study-manifest.json", "immutable study manifest")
    if plan != manifest:
        raise StudyBindingError(
            "study-plan.json and study-manifest.json do not contain the same immutable plan"
        )
    if plan.get("schemaVersion") != 1:
        raise StudyBindingError("unsupported immutable study plan schema")

    protocol = load_json(root / "study-protocol.json", "frozen study protocol")
    definition = load_json(root / "research-definition.json", "frozen research definition")
    if plan.get("protocol") != protocol or plan.get("definition") != definition:
        raise StudyBindingError(
            "frozen study protocol/definition copies differ from the immutable study plan"
        )
    expected_protocol_id = protocol_identity(protocol)
    expected_definition_id = definition_identity(definition)
    if plan.get("protocolIdentity") != expected_protocol_id:
        raise StudyBindingError(
            "study plan protocolIdentity does not match frozen protocol content"
        )
    if plan.get("definitionIdentity") != expected_definition_id:
        raise StudyBindingError(
            "study plan definitionIdentity does not match frozen definition content"
        )
    source = validate_source(plan.get("source"), "study plan.source")
    if plan.get("researchRelativeDir") != RESEARCH_DIR:
        raise StudyBindingError("study plan researchRelativeDir is not the fixed research root")
    expected_study_execution = study_execution_identity(
        expected_protocol_id, expected_definition_id, source
    )
    if plan.get("studyExecutionId") != expected_study_execution:
        raise StudyBindingError(
            "study plan studyExecutionId does not match frozen protocol/definition/source"
        )
    if plan.get("boundBeforeExecution") is not True:
        raise StudyBindingError("study plan must record binding before execution")

    binding = validate_result_binding(
        load_json(root / "study-result-binding.json", "study result binding")
    )
    expected_pairs = {
        "studyExecutionId": plan.get("studyExecutionId"),
        "protocolIdentity": expected_protocol_id,
        "protocolRevision": protocol.get("protocolRevision"),
        "studyId": protocol.get("studyId"),
        "scientificStatus": protocol.get("status"),
        "boundBeforeExecution": plan.get("boundBeforeExecution"),
        "confirmatoryPreResultClaimEligible": plan.get(
            "confirmatoryPreResultClaimEligible"
        ),
        "definitionIdentity": expected_definition_id,
        "source": source,
        "researchRelativeDir": RESEARCH_DIR,
    }
    for field, expected in expected_pairs.items():
        if binding.get(field) != expected:
            raise StudyBindingError(
                f"study result binding field {field} does not match the frozen study plan/protocol"
            )
    expected_requirements = analysis_requirements_from_protocol(protocol)
    if binding["analysisRequirements"] != expected_requirements:
        raise StudyBindingError(
            "study result binding analysisRequirements do not match the frozen study protocol"
        )

    research_root = root / RESEARCH_DIR
    if research_root.is_symlink() or not research_root.is_dir():
        raise StudyBindingError("bound research root is missing or is a symbolic link")
    research_manifest = load_json(
        research_root / "research-manifest.json", "research manifest"
    )
    research_plan = load_json(research_root / "research-plan.json", "research plan")
    if research_manifest != research_plan:
        raise StudyBindingError(
            "research-manifest.json and research-plan.json do not contain the same immutable research plan"
        )
    if research_manifest.get("schemaVersion") != 1:
        raise StudyBindingError("unsupported research manifest schema")
    expected_research_id = research_execution_identity(expected_definition_id, source)
    for field, expected in (
        ("researchId", expected_research_id),
        ("definitionIdentity", expected_definition_id),
        ("source", source),
        ("definition", definition),
    ):
        if research_manifest.get(field) != expected:
            raise StudyBindingError(
                f"research manifest {field} does not match the frozen study execution"
            )
    if binding["researchId"] != expected_research_id:
        raise StudyBindingError(
            "study result binding researchId does not match the frozen research execution"
        )

    state = load_json(research_root / "research-state.json", "research state")
    actual_counts = completed_run_counts(state, expected_research_id)
    if binding["runCounts"] != actual_counts:
        raise StudyBindingError(
            "study result binding runCounts do not match research-state.json"
        )

    expected_artifacts = list(EXPECTED_RESULT_ARTIFACTS)
    if [item["path"] for item in binding["resultArtifacts"]] != expected_artifacts:
        raise StudyBindingError(
            "study result binding resultArtifacts do not match the producer-defined artifact set/order"
        )
    for artifact in binding["resultArtifacts"]:
        path = resolve_inside(root, artifact["path"], "study result artifact")
        raw = regular_file_bytes(path, "study result artifact")
        if fnv1a64(raw) != artifact["digest64"]:
            raise StudyBindingError(
                f"study result artifact digest64 does not match current bytes: {artifact['path']}"
            )
        artifact_json = load_json(path, "research analysis artifact")
        if artifact_json.get("schemaVersion") != 1 or artifact_json.get(
            "researchId"
        ) != expected_research_id:
            raise StudyBindingError(
                f"research analysis artifact does not belong to bound research execution: {artifact['path']}"
            )

    return {
        "plan": plan,
        "protocol": protocol,
        "definition": definition,
        "binding": binding,
        "researchManifest": research_manifest,
        "researchState": state,
    }
