#!/usr/bin/env python3
"""Execute/capture downstream AnthroSim analyses with provenance-bound lineage."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA_VERSION = 1
DEFINITION_TYPE = "anthrosim-analysis-definition"
RECORD_TYPE = "anthrosim-analysis-provenance"
DEFAULT_RECORD = "analysis/analysis-provenance.json"
READ_CHUNK_SIZE = 1024 * 1024
ALLOWED_STATUSES = {"exploratory", "confirmatory"}
ALLOWED_MODES = {"scripted", "external_or_manual"}
ALLOWED_REPRODUCTION_CRITERIA = {"exact_output_bytes"}


class AnalysisProvenanceError(Exception):
    """Raised when downstream analysis lineage cannot be preserved safely."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run or capture a downstream AnthroSim analysis and bind the exact study, "
            "inputs, implementation, environment, configuration, RNG seeds and outputs."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run = subparsers.add_parser(
        "run", help="execute a scripted analysis and write immutable provenance"
    )
    run.add_argument("study_root", type=Path)
    run.add_argument("definition", type=Path)
    run.add_argument("--output", type=Path)

    capture = subparsers.add_parser(
        "capture", help="capture already-produced external/manual analysis outputs"
    )
    capture.add_argument("study_root", type=Path)
    capture.add_argument("definition", type=Path)
    capture.add_argument("--output", type=Path)

    replay = subparsers.add_parser(
        "replay",
        help="re-execute a wrapper-run scripted analysis in an isolated temporary root and require exact output bytes",
    )
    replay.add_argument("study_root", type=Path)
    replay.add_argument("--record", type=Path)

    verify = subparsers.add_parser(
        "verify", help="verify current files against a preserved provenance record"
    )
    verify.add_argument("study_root", type=Path)
    verify.add_argument("--record", type=Path)

    return parser.parse_args()


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for key, value in pairs:
        if key in output:
            raise AnalysisProvenanceError(f"duplicate JSON object key: {key}")
        output[key] = value
    return output


def load_json(path: Path, role: str) -> dict[str, Any]:
    if path.is_symlink():
        raise AnalysisProvenanceError(f"{role} must not be a symbolic link: {path}")
    if not path.is_file():
        raise AnalysisProvenanceError(
            f"{role} is missing or is not a regular file: {path}"
        )
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except AnalysisProvenanceError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AnalysisProvenanceError(f"cannot read {role} {path}: {error}") from error
    if not isinstance(value, dict):
        raise AnalysisProvenanceError(f"{role} root must be a JSON object")
    return value


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def require_exact_keys(
    value: dict[str, Any], required: set[str], optional: set[str], role: str
) -> None:
    keys = set(value)
    missing = required - keys
    unknown = keys - required - optional
    if missing:
        raise AnalysisProvenanceError(
            f"{role} is missing required field(s): {', '.join(sorted(missing))}"
        )
    if unknown:
        raise AnalysisProvenanceError(
            f"{role} contains unknown field(s): {', '.join(sorted(unknown))}"
        )


def require_nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AnalysisProvenanceError(f"{role} must be a non-empty string")
    return value


def require_uint(value: Any, role: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise AnalysisProvenanceError(f"{role} must be a non-negative integer")
    return value


def validate_relative_path(value: Any, role: str) -> str:
    raw = require_nonempty_string(value, role)
    if "\\" in raw:
        raise AnalysisProvenanceError(f"{role} must use POSIX separators")
    path = PurePosixPath(raw)
    if (
        path.is_absolute()
        or raw == "."
        or any(part in ("", ".", "..") for part in path.parts)
        or path.as_posix() != raw
    ):
        raise AnalysisProvenanceError(f"{role} is not a safe canonical relative path")
    return raw


def validate_optional_identity(value: Any, role: str) -> str | None:
    if value is None:
        return None
    return require_nonempty_string(value, role)


def validate_artifact_specs(
    raw: Any, role: str, *, require_nonempty: bool
) -> list[dict[str, str]]:
    if not isinstance(raw, list):
        raise AnalysisProvenanceError(f"{role} must be an array")
    if require_nonempty and not raw:
        raise AnalysisProvenanceError(f"{role} must contain at least one artifact")
    result: list[dict[str, str]] = []
    seen: set[str] = set()
    for index, item in enumerate(raw):
        item_role = f"{role}[{index}]"
        if not isinstance(item, dict):
            raise AnalysisProvenanceError(f"{item_role} must be an object")
        require_exact_keys(item, {"path", "role"}, set(), item_role)
        path = validate_relative_path(item["path"], f"{item_role}.path")
        artifact_role = require_nonempty_string(item["role"], f"{item_role}.role")
        if path in seen:
            raise AnalysisProvenanceError(f"{role} contains duplicate path: {path}")
        seen.add(path)
        result.append({"path": path, "role": artifact_role})
    return result


def validate_definition(raw: dict[str, Any]) -> dict[str, Any]:
    require_exact_keys(
        raw,
        {
            "schemaVersion",
            "definitionType",
            "analysisId",
            "analysisStatus",
            "executionMode",
            "workingDirectory",
            "command",
            "arguments",
            "analysisRngSeeds",
            "runtimeDescription",
            "reproductionCriterion",
            "inputs",
            "implementation",
            "environment",
            "outputs",
            "manualSteps",
        },
        {"observationModelIdentity"},
        "analysis definition",
    )
    if raw["schemaVersion"] != SCHEMA_VERSION:
        raise AnalysisProvenanceError(
            f"unsupported analysis definition schema {raw['schemaVersion']!r}; "
            f"supported schema is {SCHEMA_VERSION}"
        )
    if raw["definitionType"] != DEFINITION_TYPE:
        raise AnalysisProvenanceError("unsupported analysis definition type")

    analysis_id = require_nonempty_string(raw["analysisId"], "analysisId")
    status = require_nonempty_string(raw["analysisStatus"], "analysisStatus")
    if status not in ALLOWED_STATUSES:
        raise AnalysisProvenanceError(
            f"analysisStatus must be one of: {', '.join(sorted(ALLOWED_STATUSES))}"
        )
    mode = require_nonempty_string(raw["executionMode"], "executionMode")
    if mode not in ALLOWED_MODES:
        raise AnalysisProvenanceError(
            f"executionMode must be one of: {', '.join(sorted(ALLOWED_MODES))}"
        )

    working_directory = raw["workingDirectory"]
    if working_directory != ".":
        working_directory = validate_relative_path(
            working_directory, "workingDirectory"
        )

    command = raw["command"]
    if not isinstance(command, list) or any(
        not isinstance(item, str) or not item for item in command
    ):
        raise AnalysisProvenanceError("command must be an array of non-empty strings")
    if mode == "scripted" and not command:
        raise AnalysisProvenanceError("scripted analysis requires a non-empty command")
    if mode == "external_or_manual" and command:
        raise AnalysisProvenanceError(
            "external_or_manual analysis must use an empty command; preserve the external/manual process in manualSteps"
        )

    arguments = raw["arguments"]
    if not isinstance(arguments, dict):
        raise AnalysisProvenanceError("arguments must be a JSON object")

    seeds_raw = raw["analysisRngSeeds"]
    if not isinstance(seeds_raw, list):
        raise AnalysisProvenanceError("analysisRngSeeds must be an array")
    seeds = [require_uint(seed, f"analysisRngSeeds[{i}]") for i, seed in enumerate(seeds_raw)]
    if len(set(seeds)) != len(seeds):
        raise AnalysisProvenanceError("analysisRngSeeds must not contain duplicates")

    runtime_description = require_nonempty_string(
        raw["runtimeDescription"], "runtimeDescription"
    )
    reproduction_criterion = require_nonempty_string(
        raw["reproductionCriterion"], "reproductionCriterion"
    )
    if reproduction_criterion not in ALLOWED_REPRODUCTION_CRITERIA:
        raise AnalysisProvenanceError(
            "unsupported reproductionCriterion; schema v1 supports exact_output_bytes"
        )
    inputs = validate_artifact_specs(raw["inputs"], "inputs", require_nonempty=True)
    implementation = validate_artifact_specs(
        raw["implementation"], "implementation", require_nonempty=True
    )
    environment = validate_artifact_specs(
        raw["environment"], "environment", require_nonempty=False
    )
    if status == "confirmatory" and not environment:
        raise AnalysisProvenanceError(
            "confirmatory analysis requires at least one environment artifact"
        )
    outputs = validate_artifact_specs(raw["outputs"], "outputs", require_nonempty=True)

    manual_steps = raw["manualSteps"]
    if not isinstance(manual_steps, list) or any(
        not isinstance(item, str) or not item.strip() for item in manual_steps
    ):
        raise AnalysisProvenanceError("manualSteps must be an array of non-empty strings")
    if mode == "external_or_manual" and not manual_steps:
        raise AnalysisProvenanceError(
            "external_or_manual analysis requires at least one declared manualStep"
        )

    all_paths: dict[str, str] = {}
    for category, entries in (
        ("inputs", inputs),
        ("implementation", implementation),
        ("environment", environment),
        ("outputs", outputs),
    ):
        for entry in entries:
            path = entry["path"]
            if path == "study-result-binding.json":
                raise AnalysisProvenanceError(
                    "study-result-binding.json is automatically bound and may not be redeclared as an analysis artifact"
                )
            if path in all_paths:
                raise AnalysisProvenanceError(
                    f"artifact path {path!r} appears in both {all_paths[path]} and {category}"
                )
            all_paths[path] = category

    return {
        "schemaVersion": SCHEMA_VERSION,
        "definitionType": DEFINITION_TYPE,
        "analysisId": analysis_id,
        "analysisStatus": status,
        "executionMode": mode,
        "workingDirectory": working_directory,
        "command": command,
        "arguments": arguments,
        "analysisRngSeeds": seeds,
        "runtimeDescription": runtime_description,
        "reproductionCriterion": reproduction_criterion,
        "inputs": inputs,
        "implementation": implementation,
        "environment": environment,
        "outputs": outputs,
        "manualSteps": manual_steps,
        "observationModelIdentity": validate_optional_identity(
            raw.get("observationModelIdentity"), "observationModelIdentity"
        ),
    }


def definition_identity(definition: dict[str, Any]) -> str:
    digest = hashlib.sha256(canonical_bytes(definition)).hexdigest()
    return f"analysis-definition-v1-sha256-{digest}"


def require_root(root: Path) -> Path:
    if root.is_symlink():
        raise AnalysisProvenanceError(f"study root must not be a symbolic link: {root}")
    try:
        resolved = root.resolve(strict=True)
    except FileNotFoundError as error:
        raise AnalysisProvenanceError(f"study root does not exist: {root}") from error
    if not resolved.is_dir():
        raise AnalysisProvenanceError(f"study root is not a directory: {root}")
    return resolved


def resolve_inside(root: Path, relative: str, role: str, *, must_exist: bool) -> Path:
    path = root / Path(PurePosixPath(relative))
    if path.is_symlink():
        raise AnalysisProvenanceError(f"{role} must not be a symbolic link: {path}")
    resolved = path.resolve(strict=must_exist)
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise AnalysisProvenanceError(f"{role} escapes the study root: {relative}") from error
    return resolved


def sha256_file(path: Path, role: str) -> tuple[str, int]:
    if path.is_symlink():
        raise AnalysisProvenanceError(f"{role} must not be a symbolic link: {path}")
    try:
        mode = os.lstat(path).st_mode
    except FileNotFoundError as error:
        raise AnalysisProvenanceError(f"{role} is missing: {path}") from error
    if not stat.S_ISREG(mode):
        raise AnalysisProvenanceError(f"{role} must be a regular file: {path}")

    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        before = os.fstat(handle.fileno())
        while True:
            chunk = handle.read(READ_CHUNK_SIZE)
            if not chunk:
                break
            digest.update(chunk)
            size += len(chunk)
        after = os.fstat(handle.fileno())
    if (
        before.st_size != after.st_size
        or before.st_mtime_ns != after.st_mtime_ns
        or size != after.st_size
    ):
        raise AnalysisProvenanceError(f"{role} changed while hashing: {path}")
    return digest.hexdigest(), size


def fingerprint_artifacts(
    root: Path, specs: list[dict[str, str]], category: str
) -> list[dict[str, Any]]:
    entries = []
    for spec in specs:
        path = resolve_inside(root, spec["path"], f"{category} artifact", must_exist=True)
        digest, size = sha256_file(path, f"{category} artifact")
        entries.append(
            {
                "path": spec["path"],
                "role": spec["role"],
                "sha256": digest,
                "sizeBytes": size,
            }
        )
    return entries


def validate_study_binding(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    relative = "study-result-binding.json"
    path = resolve_inside(root, relative, "study result binding", must_exist=True)
    binding = load_json(path, "study result binding")
    required = {
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
    }
    missing = required - set(binding)
    if missing:
        raise AnalysisProvenanceError(
            "study result binding is missing required field(s): "
            + ", ".join(sorted(missing))
        )
    if binding["schemaVersion"] != 1:
        raise AnalysisProvenanceError("unsupported study result binding schema")
    for key in (
        "resultIdentity",
        "studyExecutionId",
        "protocolIdentity",
        "studyId",
        "definitionIdentity",
        "researchId",
    ):
        require_nonempty_string(binding[key], f"study result binding.{key}")
    require_uint(binding["protocolRevision"], "study result binding.protocolRevision")
    if binding["scientificStatus"] not in ALLOWED_STATUSES:
        raise AnalysisProvenanceError("study result binding has invalid scientificStatus")
    if not isinstance(binding["boundBeforeExecution"], bool) or not isinstance(
        binding["confirmatoryPreResultClaimEligible"], bool
    ):
        raise AnalysisProvenanceError("study result binding eligibility flags must be booleans")
    if not isinstance(binding["source"], dict):
        raise AnalysisProvenanceError("study result binding source must be an object")

    digest, size = sha256_file(path, "study result binding")
    artifact = {
        "path": relative,
        "role": "frozen-study-result-binding",
        "sha256": digest,
        "sizeBytes": size,
    }
    return binding, artifact


def require_status_compatibility(
    definition: dict[str, Any], binding: dict[str, Any]
) -> None:
    if definition["analysisStatus"] != "confirmatory":
        return
    if binding["scientificStatus"] != "confirmatory":
        raise AnalysisProvenanceError(
            "confirmatory analysis cannot be bound to an exploratory study protocol"
        )
    if not binding["boundBeforeExecution"]:
        raise AnalysisProvenanceError(
            "confirmatory analysis requires a study protocol bound before execution"
        )
    if not binding["confirmatoryPreResultClaimEligible"]:
        raise AnalysisProvenanceError(
            "confirmatory analysis requires a study result eligible for a pre-result confirmatory claim"
        )


def file_set_snapshot(
    root: Path, definition: dict[str, Any], study_artifact: dict[str, Any]
) -> dict[str, list[dict[str, Any]]]:
    return {
        "study": [study_artifact],
        "inputs": fingerprint_artifacts(root, definition["inputs"], "input"),
        "implementation": fingerprint_artifacts(
            root, definition["implementation"], "implementation"
        ),
        "environment": fingerprint_artifacts(
            root, definition["environment"], "environment"
        ),
    }


def output_snapshot(root: Path, definition: dict[str, Any]) -> list[dict[str, Any]]:
    return fingerprint_artifacts(root, definition["outputs"], "output")


def ensure_sources_unchanged(
    before: dict[str, list[dict[str, Any]]],
    after: dict[str, list[dict[str, Any]]],
) -> None:
    if before != after:
        raise AnalysisProvenanceError(
            "study/input/implementation/environment artifacts changed during analysis execution"
        )


def validate_record_path(root: Path, raw_path: Path | None) -> tuple[Path, str]:
    if raw_path is None:
        relative = DEFAULT_RECORD
    else:
        if raw_path.is_absolute():
            resolved = raw_path.resolve(strict=False)
            try:
                relative = resolved.relative_to(root).as_posix()
            except ValueError as error:
                raise AnalysisProvenanceError(
                    "provenance record must be written inside the study root"
                ) from error
        else:
            relative = validate_relative_path(raw_path.as_posix(), "record path")
    path = resolve_inside(root, relative, "provenance record", must_exist=False)
    return path, relative


def write_json_atomic(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, indent=2, sort_keys=True) + "\n"
    with tempfile.NamedTemporaryFile(
        mode="w",
        encoding="utf-8",
        newline="\n",
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
        delete=False,
    ) as handle:
        temporary = Path(handle.name)
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    try:
        os.replace(temporary, path)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def build_record(
    definition: dict[str, Any],
    binding: dict[str, Any],
    sources: dict[str, list[dict[str, Any]]],
    outputs: list[dict[str, Any]],
    execution_status: str,
) -> dict[str, Any]:
    record = {
        "schemaVersion": SCHEMA_VERSION,
        "recordType": RECORD_TYPE,
        "provenanceIdentity": "",
        "definitionIdentity": definition_identity(definition),
        "definition": definition,
        "analysisId": definition["analysisId"],
        "analysisStatus": definition["analysisStatus"],
        "executionStatus": execution_status,
        "study": {
            "resultIdentity": binding["resultIdentity"],
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
            "artifact": sources["study"][0],
        },
        "artifacts": {
            "inputs": sources["inputs"],
            "implementation": sources["implementation"],
            "environment": sources["environment"],
            "outputs": outputs,
        },
    }
    identity_payload = dict(record)
    identity_payload["provenanceIdentity"] = ""
    digest = hashlib.sha256(canonical_bytes(identity_payload)).hexdigest()
    record["provenanceIdentity"] = f"analysis-provenance-v1-sha256-{digest}"
    return record


def prepare_record(
    study_root: Path,
    definition_path: Path,
    record_path: Path | None,
    *,
    execute: bool,
) -> dict[str, Any]:
    root = require_root(study_root)
    definition_raw = load_json(definition_path, "analysis definition")
    definition = validate_definition(definition_raw)
    if execute and definition["executionMode"] != "scripted":
        raise AnalysisProvenanceError("run requires executionMode=scripted")
    if not execute and definition["executionMode"] != "external_or_manual":
        raise AnalysisProvenanceError(
            "capture requires executionMode=external_or_manual"
        )

    target, target_relative = validate_record_path(root, record_path)
    for output in definition["outputs"]:
        if output["path"] == target_relative:
            raise AnalysisProvenanceError(
                "provenance record path may not also be a declared analysis output"
            )
    if target.exists() or target.is_symlink():
        raise AnalysisProvenanceError(
            f"provenance record already exists; verify or create a new analysis identity instead: {target}"
        )

    binding, study_artifact = validate_study_binding(root)
    require_status_compatibility(definition, binding)
    before = file_set_snapshot(root, definition, study_artifact)

    if execute:
        cwd = root if definition["workingDirectory"] == "." else resolve_inside(
            root,
            definition["workingDirectory"],
            "analysis working directory",
            must_exist=True,
        )
        if not cwd.is_dir():
            raise AnalysisProvenanceError(
                f"analysis workingDirectory is not a directory: {cwd}"
            )
        for output in definition["outputs"]:
            output_path = resolve_inside(
                root, output["path"], "declared output", must_exist=False
            )
            if output_path.exists() or output_path.is_symlink():
                raise AnalysisProvenanceError(
                    f"scripted analysis output already exists; use a fresh canonical output path or replay an existing result: {output_path}"
                )
        result = subprocess.run(definition["command"], cwd=cwd, check=False)
        if result.returncode != 0:
            raise AnalysisProvenanceError(
                f"analysis command failed with exit code {result.returncode}; provenance was not published"
            )
        binding_after, study_artifact_after = validate_study_binding(root)
        if binding_after != binding:
            raise AnalysisProvenanceError(
                "study result binding changed during analysis execution"
            )
        after = file_set_snapshot(root, definition, study_artifact_after)
        ensure_sources_unchanged(before, after)
        execution_status = "executed_by_wrapper"
    else:
        execution_status = "captured_external_or_manual"

    outputs = output_snapshot(root, definition)
    record = build_record(definition, binding, before, outputs, execution_status)
    write_json_atomic(target, record)
    return record


def verify_artifact_entry(root: Path, entry: Any, role: str) -> None:
    if not isinstance(entry, dict):
        raise AnalysisProvenanceError(f"{role} entry must be an object")
    require_exact_keys(entry, {"path", "role", "sha256", "sizeBytes"}, set(), role)
    relative = validate_relative_path(entry["path"], f"{role}.path")
    require_nonempty_string(entry["role"], f"{role}.role")
    digest = entry["sha256"]
    size = entry["sizeBytes"]
    if (
        not isinstance(digest, str)
        or len(digest) != 64
        or any(character not in "0123456789abcdef" for character in digest)
    ):
        raise AnalysisProvenanceError(f"{role}.sha256 is invalid")
    require_uint(size, f"{role}.sizeBytes")
    path = resolve_inside(root, relative, role, must_exist=True)
    actual_digest, actual_size = sha256_file(path, role)
    if actual_size != size or actual_digest != digest:
        raise AnalysisProvenanceError(f"{role} digest/size mismatch: {relative}")


def verify_record(study_root: Path, record_path: Path | None) -> dict[str, Any]:
    root = require_root(study_root)
    target, _ = validate_record_path(root, record_path)
    record = load_json(target, "analysis provenance record")
    require_exact_keys(
        record,
        {
            "schemaVersion",
            "recordType",
            "provenanceIdentity",
            "definitionIdentity",
            "definition",
            "analysisId",
            "analysisStatus",
            "executionStatus",
            "study",
            "artifacts",
        },
        set(),
        "analysis provenance record",
    )
    if record["schemaVersion"] != SCHEMA_VERSION or record["recordType"] != RECORD_TYPE:
        raise AnalysisProvenanceError("unsupported analysis provenance record schema/type")
    definition_raw = record["definition"]
    if not isinstance(definition_raw, dict):
        raise AnalysisProvenanceError("record definition must be an object")
    definition = validate_definition(definition_raw)
    expected_definition_identity = definition_identity(definition)
    if record["definitionIdentity"] != expected_definition_identity:
        raise AnalysisProvenanceError("analysis definition identity mismatch")
    if record["analysisId"] != definition["analysisId"] or record["analysisStatus"] != definition["analysisStatus"]:
        raise AnalysisProvenanceError("record analysis identity/status differs from embedded definition")
    expected_execution_status = (
        "executed_by_wrapper"
        if definition["executionMode"] == "scripted"
        else "captured_external_or_manual"
    )
    if record["executionStatus"] != expected_execution_status:
        raise AnalysisProvenanceError("record executionStatus differs from executionMode")

    identity_payload = dict(record)
    identity_payload["provenanceIdentity"] = ""
    digest = hashlib.sha256(canonical_bytes(identity_payload)).hexdigest()
    expected_identity = f"analysis-provenance-v1-sha256-{digest}"
    if record["provenanceIdentity"] != expected_identity:
        raise AnalysisProvenanceError("analysis provenance identity mismatch")

    binding, study_artifact = validate_study_binding(root)
    require_status_compatibility(definition, binding)
    study = record["study"]
    if not isinstance(study, dict):
        raise AnalysisProvenanceError("record study must be an object")
    required_study = {
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
        "artifact",
    }
    if set(study) != required_study:
        raise AnalysisProvenanceError("record study has missing/unknown fields")
    for key in required_study - {"artifact"}:
        binding_key = key
        if study[key] != binding[binding_key]:
            raise AnalysisProvenanceError(
                f"record study.{key} differs from current frozen study result binding"
            )
    if study["artifact"] != study_artifact:
        raise AnalysisProvenanceError(
            "recorded study-result-binding digest differs from current binding"
        )
    verify_artifact_entry(root, study["artifact"], "study artifact")

    artifacts = record["artifacts"]
    if not isinstance(artifacts, dict) or set(artifacts) != {
        "inputs",
        "implementation",
        "environment",
        "outputs",
    }:
        raise AnalysisProvenanceError("record artifacts has missing/unknown categories")
    for category in ("inputs", "implementation", "environment", "outputs"):
        entries = artifacts[category]
        if not isinstance(entries, list):
            raise AnalysisProvenanceError(f"record artifacts.{category} must be an array")
        for index, entry in enumerate(entries):
            verify_artifact_entry(root, entry, f"{category}[{index}]")

    expected_sources = file_set_snapshot(root, definition, study_artifact)
    if artifacts["inputs"] != expected_sources["inputs"]:
        raise AnalysisProvenanceError("recorded input artifacts differ from definition/current files")
    if artifacts["implementation"] != expected_sources["implementation"]:
        raise AnalysisProvenanceError("recorded implementation artifacts differ from definition/current files")
    if artifacts["environment"] != expected_sources["environment"]:
        raise AnalysisProvenanceError("recorded environment artifacts differ from definition/current files")
    expected_outputs = output_snapshot(root, definition)
    if artifacts["outputs"] != expected_outputs:
        raise AnalysisProvenanceError("recorded output artifacts differ from definition/current files")
    return record


def copy_recorded_artifact(source_root: Path, target_root: Path, entry: dict[str, Any]) -> None:
    relative = validate_relative_path(entry["path"], "recorded artifact path")
    source = resolve_inside(source_root, relative, "recorded replay source", must_exist=True)
    target = target_root / Path(PurePosixPath(relative))
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)


def replay_record(study_root: Path, record_path: Path | None) -> dict[str, Any]:
    root = require_root(study_root)
    record = verify_record(root, record_path)
    definition = record["definition"]
    if definition["executionMode"] != "scripted" or record["executionStatus"] != "executed_by_wrapper":
        raise AnalysisProvenanceError(
            "replay is available only for analyses originally executed by the wrapper"
        )
    if definition["reproductionCriterion"] != "exact_output_bytes":
        raise AnalysisProvenanceError("unsupported replay reproduction criterion")

    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-replay-") as directory:
        replay_root = Path(directory).resolve()
        copied: set[str] = set()
        study_artifact = record["study"]["artifact"]
        for entry in [
            study_artifact,
            *record["artifacts"]["inputs"],
            *record["artifacts"]["implementation"],
            *record["artifacts"]["environment"],
        ]:
            relative = entry["path"]
            if relative in copied:
                continue
            copied.add(relative)
            copy_recorded_artifact(root, replay_root, entry)

        for output in definition["outputs"]:
            (replay_root / Path(PurePosixPath(output["path"]))).parent.mkdir(
                parents=True, exist_ok=True
            )
        if definition["workingDirectory"] == ".":
            cwd = replay_root
        else:
            cwd = replay_root / Path(PurePosixPath(definition["workingDirectory"]))
            cwd.mkdir(parents=True, exist_ok=True)

        binding, replay_study_artifact = validate_study_binding(replay_root)
        require_status_compatibility(definition, binding)
        before = file_set_snapshot(replay_root, definition, replay_study_artifact)
        result = subprocess.run(definition["command"], cwd=cwd, check=False)
        if result.returncode != 0:
            raise AnalysisProvenanceError(
                f"replay analysis command failed with exit code {result.returncode}"
            )
        binding_after, replay_study_artifact_after = validate_study_binding(replay_root)
        if binding_after != binding:
            raise AnalysisProvenanceError(
                "study result binding changed during replay execution"
            )
        after = file_set_snapshot(replay_root, definition, replay_study_artifact_after)
        ensure_sources_unchanged(before, after)
        replay_outputs = output_snapshot(replay_root, definition)
        if replay_outputs != record["artifacts"]["outputs"]:
            raise AnalysisProvenanceError(
                "replayed outputs do not exactly reproduce the canonical output bytes"
            )
    return record


def main() -> int:
    args = parse_args()
    try:
        if args.command == "run":
            record = prepare_record(
                args.study_root, args.definition, args.output, execute=True
            )
            print(record["provenanceIdentity"])
        elif args.command == "capture":
            record = prepare_record(
                args.study_root, args.definition, args.output, execute=False
            )
            print(record["provenanceIdentity"])
        elif args.command == "replay":
            record = replay_record(args.study_root, args.record)
            print(record["provenanceIdentity"])
        else:
            record = verify_record(args.study_root, args.record)
            print(record["provenanceIdentity"])
        return 0
    except (AnalysisProvenanceError, OSError, subprocess.SubprocessError) as error:
        print(f"research-analysis-provenance: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
