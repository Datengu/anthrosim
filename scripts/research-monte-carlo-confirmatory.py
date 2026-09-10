#!/usr/bin/env python3
"""Confirmatory Monte Carlo entry point with frozen-seed and sample-value binding.

The generic precision engine validates estimands, stopping boundaries, the frozen StudyProtocol
identity and study-result binding. This confirmatory wrapper additionally requires the complete
predeclared seed schedule to equal the exact ordered seed list in the frozen
ResearchExperimentDefinition and, for supported AnthroSim-derived observables, validates every
submitted per-seed value against the authoritative finalized study output before inference.
"""

from __future__ import annotations

import argparse
import json
import math
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
        for seed in batch:
            if not isinstance(seed, int):
                fail(f"precision plan design.seedBatches[{index}] contains non-integer seed")
            seeds.append(seed)
    if len(set(seeds)) != len(seeds):
        fail("precision plan contains duplicate seeds")
    return seeds


def frozen_research_seeds(study_dir: Path) -> list[int]:
    definition = read_json(study_dir / "research-definition.json")
    if not isinstance(definition, dict):
        fail("frozen research definition must be a JSON object")
    seeds = definition.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        fail("frozen research definition has no non-empty ordered seeds list")
    if any(not isinstance(seed, int) for seed in seeds):
        fail("frozen research definition seeds must all be integers")
    if len(set(seeds)) != len(seeds):
        fail("frozen research definition contains duplicate seeds")
    return seeds


def protocol_observables(study_dir: Path) -> dict[str, dict[str, Any]]:
    protocol = read_json(study_dir / "study-protocol.json")
    if not isinstance(protocol, dict):
        fail("frozen study protocol must be a JSON object")
    observables = protocol.get("observables", [])
    if not isinstance(observables, list):
        fail("frozen study protocol observables must be a list")
    by_id: dict[str, dict[str, Any]] = {}
    for index, observable in enumerate(observables):
        if not isinstance(observable, dict):
            fail(f"study protocol observables[{index}] must be an object")
        observable_id = observable.get("id")
        if not isinstance(observable_id, str) or not observable_id:
            fail(f"study protocol observables[{index}] has no non-empty id")
        if observable_id in by_id:
            fail(f"study protocol has duplicate observable id {observable_id!r}")
        by_id[observable_id] = observable
    return by_id


def authoritative_run_rows(study_dir: Path, seeds: list[int]) -> dict[int, dict[str, Any]]:
    table = read_json(study_dir / "research" / "analysis" / "runs.json")
    if not isinstance(table, dict) or not isinstance(table.get("runs"), list):
        fail("authoritative research/analysis/runs.json has no runs list")
    by_seed: dict[int, dict[str, Any]] = {}
    for index, row in enumerate(table["runs"]):
        if not isinstance(row, dict):
            fail(f"authoritative run row {index} must be an object")
        seed = row.get("seed")
        if not isinstance(seed, int):
            fail(f"authoritative run row {index} has no integer seed")
        if seed in by_seed:
            fail(f"authoritative run table has duplicate seed {seed}")
        by_seed[seed] = row
    if set(by_seed) != set(seeds):
        fail("authoritative run table seeds do not exactly match the frozen confirmatory seeds")
    return by_seed


def sample_groups(samples: dict[str, Any]) -> list[dict[str, Any]]:
    groups = samples.get("groups")
    if not isinstance(groups, list) or not groups:
        fail("sample document groups must be a non-empty list")
    result: list[dict[str, Any]] = []
    seen: set[str] = set()
    for index, group in enumerate(groups):
        if not isinstance(group, dict):
            fail(f"sample groups[{index}] must be an object")
        group_id = group.get("id")
        if not isinstance(group_id, str) or not group_id:
            fail(f"sample groups[{index}] has no non-empty id")
        if group_id in seen:
            fail(f"sample document has duplicate group id {group_id!r}")
        seen.add(group_id)
        result.append(group)
    return result


def validate_replicates(
    group_id: str, group: dict[str, Any], seeds: list[int]
) -> dict[int, float]:
    replicates = group.get("replicates")
    if not isinstance(replicates, list):
        fail(f"sample group {group_id!r} has no replicates list")
    values: dict[int, float] = {}
    order: list[int] = []
    for index, replicate in enumerate(replicates):
        if not isinstance(replicate, dict):
            fail(f"sample group {group_id!r} replicate {index} must be an object")
        seed = replicate.get("seed")
        value = replicate.get("value")
        if not isinstance(seed, int):
            fail(f"sample group {group_id!r} replicate {index} has no integer seed")
        if seed in values:
            fail(f"sample group {group_id!r} contains duplicate seed {seed}")
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            fail(f"sample group {group_id!r} seed {seed} has non-numeric value")
        numeric = float(value)
        if not math.isfinite(numeric):
            fail(f"sample group {group_id!r} seed {seed} has non-finite value")
        order.append(seed)
        values[seed] = numeric
    if order != seeds:
        fail(
            f"sample group {group_id!r} replicate seeds do not exactly equal the ordered frozen seeds"
        )
    return values


def bind_authoritative_samples(
    study_dir: Path, samples_path: Path, seeds: list[int]
) -> None:
    """Fail closed when an AnthroSim-derived sample contradicts its authoritative source.

    `research.analysis.runs.state` is the first explicit supported deterministic derivation:
    a per-seed value is 1 exactly for a canonical completed run and 0 otherwise. Protocol
    sources prefixed `external.` or `user.` are intentionally treated as user-authored
    observations and are not rewritten or validated against AnthroSim execution state.

    Other `research.` sources are rejected until an explicit authoritative derivation is
    implemented rather than silently accepting free-form values under confirmatory status.
    Legacy/synthetic protocols with no matching observable retain their existing behaviour.
    """
    samples = read_json(samples_path)
    if not isinstance(samples, dict):
        fail("sample document must be a JSON object")
    observables = protocol_observables(study_dir)
    groups = sample_groups(samples)
    run_rows: dict[int, dict[str, Any]] | None = None

    for group in groups:
        group_id = group["id"]
        submitted = validate_replicates(group_id, group, seeds)
        observable = observables.get(group_id)
        if observable is None:
            continue
        source = observable.get("source")
        if not isinstance(source, str) or not source:
            fail(f"study observable {group_id!r} has no non-empty source")
        if source.startswith("external.") or source.startswith("user."):
            continue
        if source == "research.analysis.runs.state":
            if run_rows is None:
                run_rows = authoritative_run_rows(study_dir, seeds)
            for seed in seeds:
                expected = 1.0 if run_rows[seed].get("state") == "completed" else 0.0
                if submitted[seed] != expected:
                    fail(
                        "confirmatory sample-value binding failed for observable "
                        f"{group_id!r}, seed {seed}: submitted {submitted[seed]!r}, "
                        f"authoritative {expected!r} from {source}"
                    )
            continue
        if source.startswith("research."):
            fail(
                f"confirmatory observable {group_id!r} uses AnthroSim-derived source {source!r} "
                "without a supported authoritative sample-value derivation"
            )


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

        bind_authoritative_samples(args.study_dir, args.samples, frozen)

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
