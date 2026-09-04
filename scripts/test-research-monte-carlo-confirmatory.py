#!/usr/bin/env python3
"""Monte Carlo confirmatory regressions with producer-valid result bindings."""

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
    "anthrosim_test_research_monte_carlo_confirmatory_legacy",
    HERE / "test-research-monte-carlo-confirmatory-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_write_json = _legacy.write_json


def write_json(path: Path, value) -> None:
    if path.name == "study-result-binding.json" and isinstance(value, dict):
        value = dict(value)
        value.update(
            {
                "schemaVersion": 1,
                "studyExecutionId": value.get(
                    "studyExecutionId", "study-execution-v1-synthetic"
                ),
                "definitionIdentity": value.get(
                    "definitionIdentity", "research-definition-v1-synthetic"
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
                "runCounts": value.get("runCounts", {"completed": 4, "failed": 0}),
                "resultArtifacts": value.get("resultArtifacts", []),
            }
        )
        value["resultIdentity"] = "pending"
        value["resultIdentity"] = _binding.result_identity(value)
    _original_write_json(path, value)


_legacy.write_json = write_json

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"write_json", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    _legacy.main()
