#!/usr/bin/env python3
"""Evidence-role audit with producer-valid finalized study binding verification."""

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
    "anthrosim_research_evidence_role_audit_legacy",
    HERE / "research-evidence-role-audit-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_load_finalized_study = _legacy._load_finalized_study


def _load_finalized_study(study_dir: Path):
    try:
        raw_binding = _binding.load_json(
            study_dir / "study-result-binding.json", "study result binding"
        )
        _binding.validate_result_binding(raw_binding)
    except _binding.StudyBindingError as error:
        raise _legacy.EvidenceRoleAuditError(str(error)) from error
    return _original_load_finalized_study(study_dir)


_legacy._load_finalized_study = _load_finalized_study

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"_load_finalized_study", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
