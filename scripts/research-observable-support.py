#!/usr/bin/env python3
"""Observable-support analysis with producer-valid study-result binding verification."""

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
    "anthrosim_research_observable_support_legacy",
    HERE / "research-observable-support-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_build_assessment = _legacy.build_assessment


def build_assessment(protocol, plan, result_binding):
    if result_binding is not None:
        try:
            _binding.validate_result_binding(result_binding)
        except _binding.StudyBindingError as error:
            raise _legacy.ContractError(str(error)) from error
    return _original_build_assessment(protocol, plan, result_binding)


_legacy.build_assessment = build_assessment

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"build_assessment", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
