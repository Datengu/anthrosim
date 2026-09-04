#!/usr/bin/env python3
"""Downstream analysis provenance with producer-valid study-result binding verification."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import tempfile
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_research_analysis_provenance_legacy",
    HERE / "research-analysis-provenance-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)


def _study_artifact(resolved_root: Path) -> dict:
    path = resolved_root / "study-result-binding.json"
    digest, size = _legacy.sha256_file(path, "study result binding")
    return {
        "path": "study-result-binding.json",
        "role": "frozen-study-result-binding",
        "sha256": digest,
        "sizeBytes": size,
    }


def validate_study_binding(root: Path):
    """Validate binding plus all frozen producer authority available at the canonical root."""
    try:
        resolved_root = _legacy.require_root(root)
        context = _binding.validate_study_root(resolved_root)
    except _binding.StudyBindingError as error:
        raise _legacy.AnalysisProvenanceError(str(error)) from error
    return context["binding"], _study_artifact(resolved_root)


def validate_replay_binding(root: Path):
    """Validate the exact binding bytes copied into an already-verified replay sandbox."""
    try:
        resolved_root = _legacy.require_root(root)
        raw = _binding.load_json(
            resolved_root / "study-result-binding.json", "study result binding"
        )
        normalized = _binding.validate_result_binding(raw)
    except _binding.StudyBindingError as error:
        raise _legacy.AnalysisProvenanceError(str(error)) from error
    return normalized, _study_artifact(resolved_root)


def replay_record(study_root: Path, record_path: Path | None):
    """Replay after full canonical verification; sandbox needs only self-valid copied binding."""
    root = _legacy.require_root(study_root)
    # This is the authoritative publication/root check. It remains full-strength
    # before any isolated replay directory is constructed.
    record = _legacy.verify_record(root, record_path)
    definition = record["definition"]
    if (
        definition["executionMode"] != "scripted"
        or record["executionStatus"] != "executed_by_wrapper"
    ):
        raise _legacy.AnalysisProvenanceError(
            "replay is available only for analyses originally executed by the wrapper"
        )
    if definition["reproductionCriterion"] != "exact_output_bytes":
        raise _legacy.AnalysisProvenanceError(
            "unsupported replay reproduction criterion"
        )

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
            _legacy.copy_recorded_artifact(root, replay_root, entry)

        for output in definition["outputs"]:
            (replay_root / Path(PurePosixPath(output["path"]))).parent.mkdir(
                parents=True, exist_ok=True
            )
        if definition["workingDirectory"] == ".":
            cwd = replay_root
        else:
            cwd = replay_root / Path(PurePosixPath(definition["workingDirectory"]))
            cwd.mkdir(parents=True, exist_ok=True)

        # The canonical source root above has already been fully resolved and
        # fingerprinted. The replay sandbox intentionally contains only the
        # exact recorded artifacts, so require producer self-identity here
        # rather than inventing missing study-plan/research files.
        binding, replay_study_artifact = validate_replay_binding(replay_root)
        _legacy.require_status_compatibility(definition, binding)
        before = _legacy.file_set_snapshot(
            replay_root, definition, replay_study_artifact
        )
        result = subprocess.run(definition["command"], cwd=cwd, check=False)
        if result.returncode != 0:
            raise _legacy.AnalysisProvenanceError(
                f"replay analysis command failed with exit code {result.returncode}"
            )
        binding_after, replay_study_artifact_after = validate_replay_binding(
            replay_root
        )
        if binding_after != binding:
            raise _legacy.AnalysisProvenanceError(
                "study result binding changed during replay execution"
            )
        after = _legacy.file_set_snapshot(
            replay_root, definition, replay_study_artifact_after
        )
        _legacy.ensure_sources_unchanged(before, after)
        replay_outputs = _legacy.output_snapshot(replay_root, definition)
        if replay_outputs != record["artifacts"]["outputs"]:
            raise _legacy.AnalysisProvenanceError(
                "replayed outputs do not exactly reproduce the canonical output bytes"
            )
    return record


_legacy.validate_study_binding = validate_study_binding
_legacy.replay_record = replay_record

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {
        "validate_study_binding",
        "replay_record",
        "main",
    }:
        continue
    globals()[_name] = getattr(_legacy, _name)

# Export the hardened entry points, not the legacy originals copied above.
globals()["validate_study_binding"] = validate_study_binding
globals()["replay_record"] = replay_record


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
