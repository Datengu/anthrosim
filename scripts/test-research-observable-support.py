#!/usr/bin/env python3
"""Observable-support regressions with producer-valid result bindings."""

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
    "anthrosim_test_research_observable_support_legacy",
    HERE / "test-research-observable-support-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_run = _legacy.run


def _normalize_binding_file(path: Path) -> None:
    if not path.is_file():
        return
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        return
    value.update(
        {
            "schemaVersion": 1,
            "protocolRevision": value.get("protocolRevision", 1),
            "studyId": value.get("studyId", "support-test"),
            "scientificStatus": value.get("scientificStatus", "exploratory"),
            "boundBeforeExecution": value.get("boundBeforeExecution", True),
            "confirmatoryPreResultClaimEligible": value.get(
                "confirmatoryPreResultClaimEligible", False
            ),
            "definitionIdentity": value.get(
                "definitionIdentity", "research-definition-v1-support-test"
            ),
            "source": value.get(
                "source",
                {
                    "modelVersion": "0.3.0",
                    "modelSemanticsId": "anthrosim-model-semantics-v14",
                    "gitCommit": "synthetic-fixture",
                },
            ),
            "researchRelativeDir": value.get("researchRelativeDir", "research"),
            "runCounts": value.get("runCounts", {"completed": 1, "failed": 0}),
            "resultArtifacts": value.get("resultArtifacts", []),
        }
    )
    value["resultIdentity"] = "pending"
    value["resultIdentity"] = _binding.result_identity(value)
    path.write_bytes(_legacy.canon(value))


def run(args, ok=True):
    if "--study-result-binding" in args:
        index = args.index("--study-result-binding") + 1
        _normalize_binding_file(Path(args[index]))
    return _original_run(args, ok=ok)


_legacy.run = run

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"run", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    _legacy.main()
