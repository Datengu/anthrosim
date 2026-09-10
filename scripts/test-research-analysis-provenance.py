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
_RUN_CONFIG = {"experiment": {"durationYears": 1}}
_POINT_ID = "research-point-v1-synthetic"
_RUN_A = "run-a"
_RUN_B = "run-b"
_REL_A = "runs/point-0000/seed-0000-2"
_REL_B = "runs/point-0000/seed-0001-3"

# The preserved provenance tests only need a deterministic numeric input. Use the
# producer-defined canonical run-table seed field rather than a synthetic non-schema `value`.
_ANALYSIS_PROGRAM = r'''import argparse, json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--input", required=True)
parser.add_argument("--output", required=True)
scale_source = parser.add_mutually_exclusive_group(required=True)
scale_source.add_argument("--scale", type=int)
scale_source.add_argument("--config")
parser.add_argument("--mutate-input", action="store_true")
args = parser.parse_args()

source = Path(args.input)
rows = json.loads(source.read_text(encoding="utf-8"))["runs"]
scale = args.scale
if args.config is not None:
    scale = int(json.loads(Path(args.config).read_text(encoding="utf-8"))["scale"])
value = sum(row["seed"] for row in rows) * scale
Path(args.output).write_text(
    json.dumps({"schemaVersion": 1, "scaledTotal": value}, sort_keys=True) + "\n",
    encoding="utf-8",
)
if args.mutate_input:
    source.write_text(source.read_text(encoding="utf-8") + "\n", encoding="utf-8")
'''
_legacy.ANALYSIS_PROGRAM = _ANALYSIS_PROGRAM


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
        "seeds": [2, 3],
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

    coordinates: list[dict] = []
    research_manifest = {
        "schemaVersion": 1,
        "researchId": research_id,
        "definitionIdentity": definition_id,
        "source": dict(_SOURCE),
        "definition": definition,
        "points": [
            {
                "point": {
                    "pointId": _POINT_ID,
                    "index": 0,
                    "coordinates": coordinates,
                    "runConfig": _RUN_CONFIG,
                },
                "runs": [
                    {
                        "seed": 2,
                        "runId": _RUN_A,
                        "relativeDir": _REL_A,
                        "runConfig": _RUN_CONFIG,
                    },
                    {
                        "seed": 3,
                        "runId": _RUN_B,
                        "relativeDir": _REL_B,
                        "runConfig": _RUN_CONFIG,
                    },
                ],
            }
        ],
    }
    _write(root / "research/research-manifest.json", research_manifest)
    _write(root / "research/research-plan.json", research_manifest)

    state_runs = {
        _RUN_A: {
            "runId": _RUN_A,
            "pointId": _POINT_ID,
            "seed": 2,
            "relativeDir": _REL_A,
            "attempt": 1,
            "state": "completed",
            "stateDigest64": 101,
            "error": None,
        },
        _RUN_B: {
            "runId": _RUN_B,
            "pointId": _POINT_ID,
            "seed": 3,
            "relativeDir": _REL_B,
            "attempt": 1,
            "state": "completed",
            "stateDigest64": 102,
            "error": None,
        },
    }
    _write(
        root / "research/research-state.json",
        {"schemaVersion": 1, "researchId": research_id, "runs": state_runs},
    )

    points_path = root / "research/analysis/points.json"
    runs_path = root / "research/analysis/runs.json"
    _write(
        points_path,
        {
            "schemaVersion": 1,
            "researchId": research_id,
            "points": [
                {
                    "pointId": _POINT_ID,
                    "index": 0,
                    "coordinates": coordinates,
                    "resultingConfiguration": _RUN_CONFIG,
                    "runIds": [_RUN_A, _RUN_B],
                }
            ],
        },
    )
    _write(
        runs_path,
        {
            "schemaVersion": 1,
            "researchId": research_id,
            "runs": [
                {
                    "pointId": _POINT_ID,
                    "runId": _RUN_A,
                    "seed": 2,
                    "coordinates": coordinates,
                    "resultingConfiguration": _RUN_CONFIG,
                    "relativeDir": _REL_A,
                    "attempt": 1,
                    "state": "completed",
                    "stateDigest64": 101,
                    "error": None,
                },
                {
                    "pointId": _POINT_ID,
                    "runId": _RUN_B,
                    "seed": 3,
                    "coordinates": coordinates,
                    "resultingConfiguration": _RUN_CONFIG,
                    "relativeDir": _REL_B,
                    "attempt": 1,
                    "state": "completed",
                    "stateDigest64": 102,
                    "error": None,
                },
            ],
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
