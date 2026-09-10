#!/usr/bin/env python3
"""Regression coverage for finalized study-result binding integrity."""

from __future__ import annotations

import copy
import importlib.util
import json
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
HELPER = HERE / "research-study-result-binding.py"

spec = importlib.util.spec_from_file_location("anthrosim_study_result_binding", HELPER)
assert spec is not None and spec.loader is not None
binding = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binding)

SOURCE = {
    "modelVersion": "0.3.6",
    "modelSemanticsId": "anthrosim-model-semantics-v35",
    "gitCommit": "0123456789abcdef0123456789abcdef01234567",
}
SUPPORT_IDENTITY = "observable-support-plan-v1-sha256-" + "a" * 64
POINT_ID = "research-point-v1-test"
RUN_ID = "research-run-v1-test"
RUN_CONFIG = {"experiment": {"durationYears": 1}}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def protocol() -> dict:
    return {
        "schemaVersion": 1,
        "protocolRevision": 1,
        "studyId": "study-result-binding-regression",
        "status": "exploratory",
        "observables": [
            {
                "id": "synthetic-observable",
                "interpretation": "Synthetic fixture; observable-support-plan-v1:"
                + SUPPORT_IDENTITY,
            }
        ],
    }


def definition() -> dict:
    return {
        "schemaVersion": 1,
        "seeds": [11],
        "base": {},
        "dimensions": [],
    }


def canonical_points(research_id: str) -> dict:
    return {
        "schemaVersion": 1,
        "researchId": research_id,
        "points": [
            {
                "pointId": POINT_ID,
                "index": 0,
                "coordinates": [],
                "resultingConfiguration": copy.deepcopy(RUN_CONFIG),
                "runIds": [RUN_ID],
            }
        ],
    }


def canonical_runs(research_id: str) -> dict:
    return {
        "schemaVersion": 1,
        "researchId": research_id,
        "runs": [
            {
                "pointId": POINT_ID,
                "runId": RUN_ID,
                "seed": 11,
                "coordinates": [],
                "resultingConfiguration": copy.deepcopy(RUN_CONFIG),
                "relativeDir": "runs/point-0000/seed-0000-11",
                "attempt": 1,
                "state": "completed",
                "stateDigest64": 123,
                "error": None,
            }
        ],
    }


def make_root(root: Path) -> tuple[dict, Path]:
    proto = protocol()
    research_definition = definition()
    protocol_id = binding.protocol_identity(proto)
    definition_id = binding.definition_identity(research_definition)
    study_execution_id = binding.study_execution_identity(
        protocol_id, definition_id, SOURCE
    )
    research_id = binding.research_execution_identity(definition_id, SOURCE)

    plan = {
        "schemaVersion": 1,
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_id,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": False,
        "protocol": proto,
        "definitionIdentity": definition_id,
        "source": copy.deepcopy(SOURCE),
        "definition": research_definition,
        "researchRelativeDir": "research",
    }
    write_json(root / "study-plan.json", plan)
    write_json(root / "study-manifest.json", plan)
    write_json(root / "study-protocol.json", proto)
    write_json(root / "research-definition.json", research_definition)

    planned_run = {
        "seed": 11,
        "runId": RUN_ID,
        "relativeDir": "runs/point-0000/seed-0000-11",
        "runConfig": copy.deepcopy(RUN_CONFIG),
    }
    research_manifest = {
        "schemaVersion": 1,
        "researchId": research_id,
        "definitionIdentity": definition_id,
        "source": copy.deepcopy(SOURCE),
        "definition": research_definition,
        "points": [
            {
                "point": {
                    "pointId": POINT_ID,
                    "index": 0,
                    "coordinates": [],
                    "runConfig": copy.deepcopy(RUN_CONFIG),
                },
                "runs": [planned_run],
            }
        ],
    }
    write_json(root / "research/research-manifest.json", research_manifest)
    write_json(root / "research/research-plan.json", research_manifest)
    write_json(
        root / "research/research-state.json",
        {
            "schemaVersion": 1,
            "researchId": research_id,
            "runs": {
                RUN_ID: {
                    "runId": RUN_ID,
                    "pointId": POINT_ID,
                    "seed": 11,
                    "relativeDir": "runs/point-0000/seed-0000-11",
                    "attempt": 1,
                    "state": "completed",
                    "stateDigest64": 123,
                    "error": None,
                }
            },
        },
    )
    points_path = root / "research/analysis/points.json"
    runs_path = root / "research/analysis/runs.json"
    write_json(points_path, canonical_points(research_id))
    write_json(runs_path, canonical_runs(research_id))

    result = {
        "schemaVersion": 1,
        "resultIdentity": "pending",
        "studyExecutionId": study_execution_id,
        "protocolIdentity": protocol_id,
        "protocolRevision": 1,
        "studyId": proto["studyId"],
        "scientificStatus": "exploratory",
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": False,
        "definitionIdentity": definition_id,
        "researchId": research_id,
        "source": copy.deepcopy(SOURCE),
        "researchRelativeDir": "research",
        "runCounts": {"completed": 1, "failed": 0},
        "resultArtifacts": [
            {
                "path": "research/analysis/points.json",
                "digest64": binding.fnv1a64(points_path.read_bytes()),
            },
            {
                "path": "research/analysis/runs.json",
                "digest64": binding.fnv1a64(runs_path.read_bytes()),
            },
        ],
        "analysisRequirements": [
            {
                "kind": "observable_support_sensitivity",
                "identity": SUPPORT_IDENTITY,
            }
        ],
    }
    result["resultIdentity"] = binding.result_identity(result)
    path = root / "study-result-binding.json"
    write_json(path, result)
    return result, path


def stale_mutations(valid: dict) -> list[tuple[str, dict]]:
    cases: list[tuple[str, dict]] = []

    def add(name: str, mutate) -> None:
        value = copy.deepcopy(valid)
        mutate(value)
        cases.append((name, value))

    add(
        "studyExecutionId",
        lambda value: value.__setitem__(
            "studyExecutionId", "study-execution-v1-tampered"
        ),
    )
    add(
        "protocolIdentity",
        lambda value: value.__setitem__(
            "protocolIdentity", "study-protocol-v1-tampered"
        ),
    )
    add("protocolRevision", lambda value: value.__setitem__("protocolRevision", 2))
    add(
        "definitionIdentity",
        lambda value: value.__setitem__(
            "definitionIdentity", "research-definition-v1-tampered"
        ),
    )
    add(
        "researchId",
        lambda value: value.__setitem__("researchId", "research-execution-v1-tampered"),
    )
    add("source", lambda value: value["source"].__setitem__("gitCommit", "f" * 40))
    add(
        "runCounts",
        lambda value: value["runCounts"].__setitem__("completed", 2),
    )
    add(
        "resultArtifacts",
        lambda value: value["resultArtifacts"][0].__setitem__(
            "digest64", value["resultArtifacts"][0]["digest64"] ^ 1
        ),
    )
    add(
        "analysisRequirements",
        lambda value: value["analysisRequirements"][0].__setitem__(
            "identity", "observable-support-plan-v1-sha256-" + "b" * 64
        ),
    )
    return cases


def assert_stale_identity_rejected(valid: dict) -> None:
    for name, tampered in stale_mutations(valid):
        assert tampered["resultIdentity"] == valid["resultIdentity"]
        try:
            binding.validate_result_binding(tampered)
        except binding.StudyBindingError as error:
            assert "resultIdentity" in str(error), (name, str(error))
        else:
            raise AssertionError(f"stale identity-covered mutation was accepted: {name}")


def assert_self_consistent_root_tamper_rejected(
    root: Path, valid: dict, path: Path
) -> None:
    for name, tampered in stale_mutations(valid):
        tampered["resultIdentity"] = "pending"
        tampered["resultIdentity"] = binding.result_identity(tampered)
        binding.validate_result_binding(tampered)
        write_json(path, tampered)
        try:
            binding.validate_study_root(root)
        except binding.StudyBindingError:
            pass
        else:
            raise AssertionError(
                f"self-consistent binding mutation contradicted frozen root but was accepted: {name}"
            )
        write_json(path, valid)
        binding.validate_study_root(root)


def assert_semantically_forged_canonical_rows_rejected(
    root: Path, valid: dict, binding_path: Path
) -> None:
    points_path = root / "research/analysis/points.json"
    runs_path = root / "research/analysis/runs.json"
    original_points = json.loads(points_path.read_text(encoding="utf-8"))
    original_runs = json.loads(runs_path.read_text(encoding="utf-8"))

    forged_points = copy.deepcopy(original_points)
    forged_runs = copy.deepcopy(original_runs)
    forged_points["points"][0]["resultingConfiguration"]["experiment"][
        "durationYears"
    ] = 999
    forged_runs["runs"][0]["resultingConfiguration"]["experiment"][
        "durationYears"
    ] = 999
    write_json(points_path, forged_points)
    write_json(runs_path, forged_runs)

    forged_binding = copy.deepcopy(valid)
    forged_binding["resultArtifacts"][0]["digest64"] = binding.fnv1a64(
        points_path.read_bytes()
    )
    forged_binding["resultArtifacts"][1]["digest64"] = binding.fnv1a64(
        runs_path.read_bytes()
    )
    forged_binding["resultIdentity"] = "pending"
    forged_binding["resultIdentity"] = binding.result_identity(forged_binding)
    binding.validate_result_binding(forged_binding)
    write_json(binding_path, forged_binding)

    try:
        binding.validate_study_root(root)
    except binding.StudyBindingError as error:
        assert "differs from immutable research plan/state" in str(error), str(error)
    else:
        raise AssertionError(
            "self-consistent finalized binding over semantically forged canonical rows was accepted"
        )

    write_json(points_path, original_points)
    write_json(runs_path, original_runs)
    write_json(binding_path, valid)
    binding.validate_study_root(root)


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-study-binding-") as directory:
        root = Path(directory)
        valid, path = make_root(root)
        normalized = binding.validate_result_binding(valid)
        assert normalized["resultIdentity"] == valid["resultIdentity"]
        context = binding.validate_study_root(root)
        assert context["binding"]["researchId"] == valid["researchId"]

        assert_stale_identity_rejected(valid)
        assert_self_consistent_root_tamper_rejected(root, valid, path)
        assert_semantically_forged_canonical_rows_rejected(root, valid, path)

    print("finalized study binding identity/root/semantic regressions: ok")


if __name__ == "__main__":
    main()
