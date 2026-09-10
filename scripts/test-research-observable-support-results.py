#!/usr/bin/env python3
"""Observable-support-results regressions with a fully finalized synthetic study root."""

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
    "anthrosim_test_research_observable_support_results_legacy",
    HERE / "test-research-observable-support-results-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_fixtures = _legacy.fixtures
_original_make_analysis = _legacy.make_analysis
_context: dict[str, object] = {}

_POINT_ID = "research-point-v1-observable-support"
_RUN_ID = "run-1"
_RELATIVE_DIR = "runs/point-0000/seed-0000-1"
_RUN_CONFIG = {"experiment": {"durationYears": 1}}


def fixtures():
    plan, _old_binding, _old_assessment = _original_fixtures()
    support_identity = _legacy.plan_identity(plan)
    source = {
        "modelVersion": "0.3.4",
        "modelSemanticsId": "anthrosim-model-semantics-v33",
        "gitCommit": "synthetic-fixture",
    }
    protocol = {
        "schemaVersion": 1,
        "protocolRevision": 1,
        "studyId": "test-study",
        "status": "exploratory",
        "observables": [
            {
                "id": "occupancy",
                "interpretation": (
                    "Synthetic support regression; "
                    f"observable-support-plan-v1:{support_identity}"
                ),
            }
        ],
    }
    definition = {
        "schemaVersion": 1,
        "seeds": [1],
        "base": {},
        "dimensions": [],
    }
    protocol_identity = _binding.protocol_identity(protocol)
    definition_identity = _binding.definition_identity(definition)
    study_execution_id = _binding.study_execution_identity(
        protocol_identity, definition_identity, source
    )
    research_id = _binding.research_execution_identity(definition_identity, source)
    coordinates: list[dict] = []
    points = {
        "schemaVersion": 1,
        "researchId": research_id,
        "points": [
            {
                "pointId": _POINT_ID,
                "index": 0,
                "coordinates": coordinates,
                "resultingConfiguration": _RUN_CONFIG,
                "runIds": [_RUN_ID],
            }
        ],
    }
    runs = {
        "schemaVersion": 1,
        "researchId": research_id,
        "runs": [
            {
                "pointId": _POINT_ID,
                "runId": _RUN_ID,
                "seed": 1,
                "coordinates": coordinates,
                "resultingConfiguration": _RUN_CONFIG,
                "relativeDir": _RELATIVE_DIR,
                "attempt": 1,
                "state": "completed",
                "stateDigest64": 101,
                "error": None,
            }
        ],
    }
    points_bytes = _legacy.canonical_bytes(points)
    runs_bytes = _legacy.canonical_bytes(runs)
    binding = {
        "schemaVersion": 1,
        "resultIdentity": "pending",
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_identity,
        "protocolRevision": 1,
        "studyId": "test-study",
        "scientificStatus": "exploratory",
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": False,
        "definitionIdentity": definition_identity,
        "researchId": research_id,
        "source": source,
        "researchRelativeDir": "research",
        "runCounts": {"completed": 1, "failed": 0},
        "resultArtifacts": [
            {
                "path": "research/analysis/points.json",
                "digest64": _binding.fnv1a64(points_bytes),
            },
            {
                "path": "research/analysis/runs.json",
                "digest64": _binding.fnv1a64(runs_bytes),
            },
        ],
        "analysisRequirements": [
            {
                "kind": "observable_support_sensitivity",
                "identity": support_identity,
            }
        ],
    }
    binding["resultIdentity"] = _binding.result_identity(binding)
    assessment = {
        "schema": "anthrosim-observable-support-assessment-v1",
        "planIdentity": support_identity,
        "protocolIdentity": protocol_identity,
        "studyId": binding["studyId"],
        "entries": plan["entries"],
        "sourceStudyExecutionId": study_execution_id,
        "sourceStudyResultIdentity": binding["resultIdentity"],
        "sourceResearchId": research_id,
    }
    assessment["assessmentIdentity"] = _legacy.assessment_identity(assessment)
    _context.clear()
    _context.update(
        {
            "protocol": protocol,
            "definition": definition,
            "source": source,
            "protocolIdentity": protocol_identity,
            "definitionIdentity": definition_identity,
            "studyExecutionId": study_execution_id,
            "researchId": research_id,
            "coordinates": coordinates,
            "points": points,
            "runs": runs,
            "binding": binding,
        }
    )
    return plan, binding, assessment


def _ensure_finalized_root(root: Path) -> None:
    if not _context:
        raise AssertionError("fixtures() must initialize finalized study context")
    protocol = _context["protocol"]
    definition = _context["definition"]
    source = _context["source"]
    coordinates = _context["coordinates"]
    plan = {
        "schemaVersion": 1,
        "studyExecutionId": _context["studyExecutionId"],
        "protocolIdentity": _context["protocolIdentity"],
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": False,
        "protocol": protocol,
        "definitionIdentity": _context["definitionIdentity"],
        "source": source,
        "definition": definition,
        "researchRelativeDir": "research",
    }
    _legacy.write(root / "study-plan.json", plan)
    _legacy.write(root / "study-manifest.json", plan)
    _legacy.write(root / "study-protocol.json", protocol)
    _legacy.write(root / "research-definition.json", definition)
    research_plan = {
        "schemaVersion": 1,
        "researchId": _context["researchId"],
        "definitionIdentity": _context["definitionIdentity"],
        "source": source,
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
                        "seed": 1,
                        "runId": _RUN_ID,
                        "relativeDir": _RELATIVE_DIR,
                        "runConfig": _RUN_CONFIG,
                    }
                ],
            }
        ],
    }
    _legacy.write(root / "research/research-manifest.json", research_plan)
    _legacy.write(root / "research/research-plan.json", research_plan)
    _legacy.write(
        root / "research/research-state.json",
        {
            "schemaVersion": 1,
            "researchId": _context["researchId"],
            "runs": {
                _RUN_ID: {
                    "runId": _RUN_ID,
                    "pointId": _POINT_ID,
                    "seed": 1,
                    "relativeDir": _RELATIVE_DIR,
                    "attempt": 1,
                    "state": "completed",
                    "stateDigest64": 101,
                    "error": None,
                }
            },
        },
    )
    _legacy.write(root / "research/analysis/points.json", _context["points"])
    _legacy.write(root / "research/analysis/runs.json", _context["runs"])


def make_analysis(root: Path, *args, **kwargs):
    _ensure_finalized_root(root)
    return _original_make_analysis(root, *args, **kwargs)


_legacy.fixtures = fixtures
_legacy.make_analysis = make_analysis

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"fixtures", "make_analysis", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    _legacy.main()
