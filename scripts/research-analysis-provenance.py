#!/usr/bin/env python3
"""Downstream analysis provenance with producer-valid study-result binding verification."""

from __future__ import annotations

import importlib.util
from pathlib import Path

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


def validate_study_binding(root: Path):
    try:
        resolved_root = _legacy.require_root(root)
        context = _binding.validate_study_root(resolved_root)
    except _binding.StudyBindingError as error:
        raise _legacy.AnalysisProvenanceError(str(error)) from error
    path = resolved_root / "study-result-binding.json"
    digest, size = _legacy.sha256_file(path, "study result binding")
    artifact = {
        "path": "study-result-binding.json",
        "role": "frozen-study-result-binding",
        "sha256": digest,
        "sizeBytes": size,
    }
    return context["binding"], artifact


_legacy.validate_study_binding = validate_study_binding

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"validate_study_binding", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
