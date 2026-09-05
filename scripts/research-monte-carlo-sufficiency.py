#!/usr/bin/env python3
"""Monte Carlo sufficiency with producer-valid study-result binding verification."""

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
    "anthrosim_research_monte_carlo_sufficiency_legacy",
    HERE / "research-monte-carlo-sufficiency-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_validate_study_binding = _legacy.validate_study_binding


def validate_study_binding(study_dir: Path, plan, identity: str):
    try:
        raw_binding = _binding.load_json(
            study_dir / "study-result-binding.json", "study result binding"
        )
        _binding.validate_result_binding(raw_binding)
    except _binding.StudyBindingError as error:
        _legacy.fail(str(error))
    return _original_validate_study_binding(study_dir, plan, identity)


_legacy.validate_study_binding = validate_study_binding

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"validate_study_binding", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
