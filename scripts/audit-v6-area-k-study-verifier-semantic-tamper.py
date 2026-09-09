#!/usr/bin/env python3
"""Audit-v6 Area K helper for a producer-vs-verifier semantic-binding adversary.

The valid study is created by the real Rust study/research CLIs. This helper then
coordinates a post-finalization edit of the two canonical research analysis tables,
recomputes the producer-defined result-artifact digests and result identity, and asks
the root-aware study-result verifier whether the resulting root is still valid.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINDING_PATH = ROOT / "scripts" / "research-study-result-binding.py"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


binding = load_module(BINDING_PATH, "audit_v6_area_k_study_binding")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def treatment_values(value: dict, collection: str) -> list[int]:
    rows = value[collection]
    return [int(row["coordinates"][0]["value"]) for row in rows]


def tamper_and_rebind(study_root: Path) -> None:
    points_path = study_root / "research" / "analysis" / "points.json"
    runs_path = study_root / "research" / "analysis" / "runs.json"
    binding_path = study_root / "study-result-binding.json"

    points = load(points_path)
    runs = load(runs_path)
    before_points = treatment_values(points, "points")
    before_runs = treatment_values(runs, "runs")
    if before_points != [4, 12] or before_runs != [4, 12]:
        raise AssertionError(
            f"unexpected untouched treatment rows: points={before_points} runs={before_runs}"
        )

    points["points"][0]["coordinates"][0]["value"] = 999
    points["points"][0]["resultingConfiguration"]["experiment"]["resources"][
        "periodsPerYear"
    ] = 999
    runs["runs"][0]["coordinates"][0]["value"] = 999
    runs["runs"][0]["resultingConfiguration"]["experiment"]["resources"][
        "periodsPerYear"
    ] = 999
    write(points_path, points)
    write(runs_path, runs)

    forged = load(binding_path)
    for artifact in forged["resultArtifacts"]:
        artifact_path = study_root / artifact["path"]
        artifact["digest64"] = binding.fnv1a64(artifact_path.read_bytes())
    forged["resultIdentity"] = binding.result_identity(forged)
    write(binding_path, forged)

    print(f"immutable_expected_treatments=[4, 12]")
    print(f"forged_points_treatments={treatment_values(points, 'points')}")
    print(f"forged_runs_treatments={treatment_values(runs, 'runs')}")
    print(f"forged_result_identity={forged['resultIdentity']}")
    print(
        "forged_result_artifact_digests="
        + ",".join(f"{item['path']}:{item['digest64']}" for item in forged["resultArtifacts"])
    )


def verify(study_root: Path) -> None:
    try:
        context = binding.validate_study_root(study_root.resolve(strict=True))
    except binding.StudyBindingError as error:
        print(f"root_verifier_accepted=false")
        print(f"root_verifier_error={error}")
        raise SystemExit(3) from error
    print("root_verifier_accepted=true")
    print(f"verified_result_identity={context['binding']['resultIdentity']}")
    print(f"verified_research_id={context['binding']['researchId']}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["tamper-and-rebind", "verify"])
    parser.add_argument("study_root", type=Path)
    args = parser.parse_args()
    if args.mode == "tamper-and-rebind":
        tamper_and_rebind(args.study_root.resolve(strict=True))
    else:
        verify(args.study_root)


if __name__ == "__main__":
    main()
