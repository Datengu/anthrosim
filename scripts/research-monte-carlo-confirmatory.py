#!/usr/bin/env python3
"""Confirmatory entry point binding #231 precision plans to a frozen study seed design.

The generic precision engine validates estimands, stopping boundaries, the frozen StudyProtocol
identity and study-result binding. This confirmatory wrapper additionally requires the complete
predeclared seed schedule to equal the exact ordered seed list in the frozen
ResearchExperimentDefinition before delegating to that engine.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise ValueError(message)


def read_json(path: Path) -> Any:
    if not path.is_file():
        fail(f"expected regular JSON file: {path}")
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def declared_seeds(plan: dict[str, Any]) -> list[int]:
    design = plan.get("design")
    if not isinstance(design, dict):
        fail("precision plan has no design object")
    batches = design.get("seedBatches")
    if not isinstance(batches, list) or not batches:
        fail("precision plan design.seedBatches must be non-empty")
    seeds: list[int] = []
    for index, batch in enumerate(batches):
        if not isinstance(batch, list) or not batch:
            fail(f"precision plan design.seedBatches[{index}] must be non-empty")
        seeds.extend(batch)
    return seeds


def frozen_research_seeds(study_dir: Path) -> list[int]:
    definition = read_json(study_dir / "research-definition.json")
    if not isinstance(definition, dict):
        fail("frozen research definition must be a JSON object")
    seeds = definition.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        fail("frozen research definition has no non-empty ordered seeds list")
    return seeds


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", type=Path)
    parser.add_argument("samples", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--study-dir", type=Path, required=True)
    args = parser.parse_args()

    try:
        plan = read_json(args.plan)
        if not isinstance(plan, dict):
            fail("precision plan must be a JSON object")
        planned = declared_seeds(plan)
        frozen = frozen_research_seeds(args.study_dir)
        if planned != frozen:
            fail(
                "precision-plan seed batches do not exactly equal the ordered seeds in the frozen "
                "ResearchExperimentDefinition; confirmatory replicate provenance is inconsistent"
            )

        engine = Path(__file__).with_name("research-monte-carlo-sufficiency.py")
        command = [
            sys.executable,
            str(engine),
            "diagnose",
            str(args.plan),
            str(args.samples),
            str(args.output),
            "--study-dir",
            str(args.study_dir),
        ]
        return subprocess.run(command, check=False).returncode
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"research-monte-carlo-confirmatory: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
