#!/usr/bin/env python3
"""Focused semantic sample-value binding regressions for confirmatory Monte Carlo."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONFIRMATORY = HERE / "research-monte-carlo-confirmatory.py"


def load_confirmatory():
    spec = importlib.util.spec_from_file_location("anthrosim_monte_carlo_confirmatory", CONFIRMATORY)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {CONFIRMATORY}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sample(group_id: str, seeds: list[int], values: list[float]) -> dict:
    return {
        "schemaVersion": 1,
        "groups": [
            {
                "id": group_id,
                "replicates": [
                    {"seed": seed, "value": value}
                    for seed, value in zip(seeds, values, strict=True)
                ],
            }
        ],
    }


def main() -> None:
    module = load_confirmatory()
    seeds = [101, 102, 103]
    with tempfile.TemporaryDirectory(prefix="anthrosim-mc-sample-binding-") as temporary:
        study = Path(temporary) / "study"
        write_json(study / "research-definition.json", {"seeds": seeds})
        write_json(
            study / "research" / "analysis" / "runs.json",
            {
                "schemaVersion": 1,
                "runs": [
                    {"seed": 101, "state": "completed"},
                    {"seed": 102, "state": "failed"},
                    {"seed": 103, "state": "completed"},
                ],
            },
        )

        authoritative = Path(temporary) / "authoritative.json"
        contradictory = Path(temporary) / "contradictory.json"
        external = Path(temporary) / "external.json"

        write_json(
            study / "study-protocol.json",
            {
                "observables": [
                    {
                        "id": "run_completed_probability",
                        "source": "research.analysis.runs.state",
                    }
                ]
            },
        )
        write_json(authoritative, sample("run_completed_probability", seeds, [1.0, 0.0, 1.0]))
        module.bind_authoritative_samples(study, authoritative, seeds)

        write_json(contradictory, sample("run_completed_probability", seeds, [0.0, 0.0, 1.0]))
        try:
            module.bind_authoritative_samples(study, contradictory, seeds)
        except ValueError as error:
            assert "sample-value binding failed" in str(error)
            assert "seed 101" in str(error)
        else:
            raise AssertionError("contradictory authoritative sample was accepted")

        write_json(
            study / "study-protocol.json",
            {"observables": [{"id": "survey_score", "source": "external.field_observation"}]},
        )
        write_json(external, sample("survey_score", seeds, [3.5, 4.0, 2.5]))
        module.bind_authoritative_samples(study, external, seeds)

        write_json(
            study / "study-protocol.json",
            {"observables": [{"id": "future_metric", "source": "research.unsupported.metric"}]},
        )
        write_json(external, sample("future_metric", seeds, [1.0, 1.0, 1.0]))
        try:
            module.bind_authoritative_samples(study, external, seeds)
        except ValueError as error:
            assert "without a supported authoritative sample-value derivation" in str(error)
        else:
            raise AssertionError("unsupported AnthroSim-derived sample source was accepted")

    print("confirmatory Monte Carlo sample-value binding suite passed")


if __name__ == "__main__":
    main()
