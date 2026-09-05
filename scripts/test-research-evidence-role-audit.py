#!/usr/bin/env python3
"""Evidence-role regressions with producer-valid synthetic result identities."""

from __future__ import annotations

import importlib.util
import json
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
    "anthrosim_test_research_evidence_role_audit_legacy",
    HERE / "test-research-evidence-role-audit-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_make = _legacy.make_finalized_study


def make_finalized_study(root: Path, value: dict, research_definition: dict) -> Path:
    study = _original_make(root, value, research_definition)
    plan_path = study / "study-plan.json"
    manifest_path = study / "study-manifest.json"
    binding_path = study / "study-result-binding.json"
    plan = json.loads(plan_path.read_text(encoding="utf-8"))
    result = json.loads(binding_path.read_text(encoding="utf-8"))
    source = {
        "modelVersion": "0.3.0",
        "modelSemanticsId": "anthrosim-model-semantics-v14",
        "gitCommit": "synthetic-fixture",
    }
    plan["source"] = source
    result["source"] = source
    result["resultIdentity"] = "pending"
    result["resultIdentity"] = _binding.result_identity(result)
    _legacy.write_json(plan_path, plan)
    _legacy.write_json(manifest_path, plan)
    _legacy.write_json(binding_path, result)
    return study


_legacy.make_finalized_study = make_finalized_study

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"make_finalized_study", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
