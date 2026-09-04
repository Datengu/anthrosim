#!/usr/bin/env python3
"""Regression tests for analysis provenance using a producer-valid synthetic study root."""

from __future__ import annotations

import importlib.util
import json
import tempfile
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
    "anthrosim_test_research_analysis_provenance_legacy",
    HERE / "test-research-analysis-provenance-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_make_study = _legacy.make_study

_SOURCE = {
    "modelVersion": "0.3.0",
    "modelSemanticsId": "anthrosim-model-semantics-v14",
    "gitCommit": "synthetic-fixture",
}


def _protocol(status: str) -> dict:
    return {
        "schemaVersion": 1,
        "protocolRevision": 1,
        "studyId": "synthetic-analysis-provenance",
        "status": status,
        "observables": [],
    }


def _definition() -> dict:
    return {
        "schemaVersion": 1,
        "seeds": [1, 2],
        "base": {},
        "dimensions": [],
    }


def _identity_context(status: str, eligible: bool):
    protocol = _protocol(status)
    definition = _definition()
    protocol_id = _binding.protocol_identity(protocol)
    definition_id = _binding.definition_identity(definition)
    study_execution_id = _binding.study_execution_identity(
        protocol_id, definition_id, _SOURCE
    )
    research_id = _binding.research_execution_identity(definition_id, _SOURCE)
    return protocol, definition, protocol_id, definition_id, study_execution_id, research_id


def binding(*, eligible: bool = True, status: str = "confirmatory") -> dict:
    protocol, definition, protocol_id, definition_id, study_execution_id, research_id = (
        _identity_context(status, eligible)
    )
    value = {
        "schemaVersion": 1,
        "resultIdentity": "pending",
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_id,
        "protocolRevision": 1,
        "studyId": protocol["studyId"],
        "scientificStatus": status,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": eligible,
        "definitionIdentity": definition_id,
        "researchId": research_id,
        "source": dict(_SOURCE),
        "researchRelativeDir": "research",
        "runCounts": {"completed": 2, "failed": 0},
        "resultArtifacts": [
            {"path": "research/analysis/points.json", "digest64": 0},
            {"path": "research/analysis/runs.json", "digest64": 0},
        ],
    }
    value["resultIdentity"] = _binding.result_identity(value)
    return value


def _write(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def make_study(
    root: Path, *, eligible: bool = True, status: str = "confirmatory"
) -> Path:
    definition_path = _original_make_study(root, eligible=eligible, status=status)
    protocol, definition, protocol_id, definition_id, study_execution_id, research_id = (
        _identity_context(status, eligible)
    )

    runs_path = root / "research/analysis/runs.json"
    runs = json.loads(runs_path.read_text(encoding="utf-8"))
    runs["researchId"] = research_id
    _write(runs_path, runs)
    points_path = root / "research/analysis/points.json"
    _write(
        points_path,
        {"schemaVersion": 1, "researchId": research_id, "points": []},
    )

    plan = {
        "schemaVersion": 1,
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_id,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": eligible,
        "protocol": protocol,
        "definitionIdentity": definition_id,
        "source": dict(_SOURCE),
        "definition": definition,
        "researchRelativeDir": "research",
    }
    _write(root / "study-plan.json", plan)
    _write(root / "study-manifest.json", plan)
    _write(root / "study-protocol.json", protocol)
    _write(root / "research-definition.json", definition)

    research_manifest = {
        "schemaVersion": 1,
        "researchId": research_id,
        "definitionIdentity": definition_id,
        "source": dict(_SOURCE),
        "definition": definition,
    }
    _write(root / "research/research-manifest.json", research_manifest)
    _write(root / "research/research-plan.json", research_manifest)
    _write(
        root / "research/research-state.json",
        {
            "schemaVersion": 1,
            "researchId": research_id,
            "runs": {
                "run-a": {"state": "completed"},
                "run-b": {"state": "completed"},
            },
        },
    )

    value = {
        "schemaVersion": 1,
        "resultIdentity": "pending",
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_id,
        "protocolRevision": 1,
        "studyId": protocol["studyId"],
        "scientificStatus": status,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": eligible,
        "definitionIdentity": definition_id,
        "researchId": research_id,
        "source": dict(_SOURCE),
        "researchRelativeDir": "research",
        "runCounts": {"completed": 2, "failed": 0},
        "resultArtifacts": [
            {
                "path": "research/analysis/points.json",
                "digest64": _binding.fnv1a64(points_path.read_bytes()),
            },
            {
                "path": "research/analysis/runs.json",
                "digest64": _binding.fnv1a64(runs_path.read_bytes()),
            },
        ],
    }
    value["resultIdentity"] = _binding.result_identity(value)
    _write(root / "study-result-binding.json", value)
    return definition_path


def test_source_mutation_during_execution_fails_closed() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        mutated = _legacy.definition(mutate_input=True)
        definition_path.write_text(
            json.dumps(mutated, indent=2) + "\n", encoding="utf-8"
        )
        failed = _legacy.run("run", root, definition_path, expect_success=False)
        # Full study-root validation may now catch the mutated canonical research
        # artifact before the older generic before/after snapshot diagnostic.
        assert (
            "changed during analysis execution" in failed.stderr
            or "result artifact digest64 does not match current bytes" in failed.stderr
        ), failed.stderr
        assert not (root / "analysis/analysis-provenance.json").exists()


_legacy.binding = binding
_legacy.make_study = make_study
_legacy.test_source_mutation_during_execution_fails_closed = (
    test_source_mutation_during_execution_fails_closed
)

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {
        "binding",
        "make_study",
        "test_source_mutation_during_execution_fails_closed",
        "main",
    }:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    _legacy.main()
