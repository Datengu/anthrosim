#!/usr/bin/env python3
"""Verify finalized study bindings while preserving the producer's historical research-ID byte order."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_research_study_result_binding_legacy",
    HERE / "research-study-result-binding-legacy.py",
)


def research_execution_identity(definition_id: str, source: dict[str, Any]) -> str:
    """Match anthrosim-research/study schema-v1 ordered serde_json identity exactly."""
    definition_id = _legacy.nonempty_string(definition_id, "definition identity")
    source = _legacy.validate_source(source, "research execution source")
    payload = {
        "schemaVersion": 1,
        "definitionIdentity": definition_id,
        "source": {
            "modelVersion": source["modelVersion"],
            "modelSemanticsId": source["modelSemanticsId"],
            "gitCommit": source["gitCommit"],
        },
    }
    encoded = json.dumps(
        payload, ensure_ascii=False, sort_keys=False, separators=(",", ":")
    ).encode("utf-8")
    return f"{_legacy.RESEARCH_EXECUTION_PREFIX}-{_legacy.fnv1a64(encoded):016x}"


# validate_study_root is defined in the preserved module, so replace the global it
# resolves at call time before re-exporting the public surface.
_legacy.research_execution_identity = research_execution_identity

for _name in dir(_legacy):
    if _name.startswith("__") or _name == "research_execution_identity":
        continue
    globals()[_name] = getattr(_legacy, _name)
