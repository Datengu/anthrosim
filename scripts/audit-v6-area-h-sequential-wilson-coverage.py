#!/usr/bin/env python3
"""Audit-v6 Area-H adversary for sequential Wilson stopping coverage.

Evidence only. This script must not be merged into production.
"""

from __future__ import annotations

import importlib.util
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GATE_PATH = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"

TRUE_P = 0.305
CONFIDENCE = 0.95
MAX_HALF_WIDTH = 0.1685
BOUNDARIES = [30, 100, 300]


def load_gate():
    spec = importlib.util.spec_from_file_location("anthrosim_mc_gate", GATE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {GATE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def make_plan(gate):
    plan = {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": "audit-v6-area-h-sequential-wilson-coverage",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": "probability",
            "confidenceLevel": CONFIDENCE,
            "maxHalfWidth": MAX_HALF_WIDTH,
        },
        "design": {
            "mode": "sequential",
            "seedBatches": [
                list(range(0, 30)),
                list(range(30, 100)),
                list(range(100, 300)),
            ],
        },
        "pairing": "independent",
        "rationale": (
            "Audit-v6 controlled Bernoulli coverage adversary: every fixed boundary has at least "
            "the declared 95% Wilson coverage at p=0.305, while the predeclared sequential "
            "precision-stopping rule is tested for overall stopped-procedure coverage."
        ),
    }
    plan["planIdentity"] = gate.plan_identity(plan)
    gate.validate_plan(plan)
    return plan


def binomial_mass(n: int, successes: int, probability: float) -> float:
    return (
        math.comb(n, successes)
        * probability**successes
        * (1.0 - probability) ** (n - successes)
    )


def main() -> int:
    gate = load_gate()
    plan = make_plan(gate)
    cache: dict[tuple[int, int], dict] = {}

    def result(n: int, successes: int) -> dict:
        key = (n, successes)
        if key not in cache:
            samples = {
                "schemaVersion": 1,
                "groups": [
                    {
                        "id": "bernoulli",
                        "replicates": [
                            {
                                "seed": seed,
                                "value": 1 if seed < successes else 0,
                            }
                            for seed in range(n)
                        ],
                    }
                ],
            }
            cache[key] = gate.derive(plan, samples, None)
        return cache[key]

    # Control 1: isolate sequential selection from ordinary fixed-boundary Wilson coverage.
    fixed_coverages: dict[int, float] = {}
    for n in BOUNDARIES:
        coverage_terms = []
        for successes in range(n + 1):
            precision = result(n, successes)["precision"]
            assert precision["precisionMethod"] == "wilson_score_probability"
            covers = (
                precision["intervalLower"] <= TRUE_P <= precision["intervalUpper"]
            )
            if covers:
                coverage_terms.append(binomial_mass(n, successes, TRUE_P))
        fixed_coverages[n] = math.fsum(coverage_terms)

    for n, coverage in fixed_coverages.items():
        assert coverage >= CONFIDENCE, (
            f"control failed: fixed n={n} Wilson coverage {coverage:.12f} is below "
            f"declared confidence {CONFIDENCE:.12f}"
        )

    # Control 2 / scientific calculation: exact finite Bernoulli path enumeration under the
    # gate's own predeclared sequential decisions. State is the success count among paths that
    # have not yet stopped. Batch increments are independent Bernoulli draws.
    live = {0: 1.0}
    previous_n = 0
    stopped_covered_terms: list[float] = []
    stop_probabilities: dict[int, float] = {}
    reached_probabilities: dict[int, float] = {}

    for boundary_index, n in enumerate(BOUNDARIES):
        increment = n - previous_n
        expanded: dict[int, float] = {}
        for previous_successes, path_probability in live.items():
            for added_successes in range(increment + 1):
                total_successes = previous_successes + added_successes
                probability = path_probability * binomial_mass(
                    increment, added_successes, TRUE_P
                )
                expanded[total_successes] = (
                    expanded.get(total_successes, 0.0) + probability
                )

        reached_probabilities[n] = math.fsum(expanded.values())
        next_live: dict[int, float] = {}
        stopped_here: list[float] = []
        is_final_boundary = boundary_index == len(BOUNDARIES) - 1

        for successes, path_probability in expanded.items():
            diagnostic = result(n, successes)
            precision = diagnostic["precision"]
            should_stop = diagnostic["decision"] == "sufficient_stop"
            if is_final_boundary:
                # A final insufficient result is still the terminal reported interval for the
                # predeclared design; include it in stopped-procedure coverage accounting.
                should_stop = True

            if should_stop:
                stopped_here.append(path_probability)
                if precision["intervalLower"] <= TRUE_P <= precision["intervalUpper"]:
                    stopped_covered_terms.append(path_probability)
            else:
                assert diagnostic["decision"] == "insufficient_continue_with_declared_next_batch"
                next_live[successes] = path_probability

        stop_probabilities[n] = math.fsum(stopped_here)
        live = next_live
        previous_n = n

    sequential_coverage = math.fsum(stopped_covered_terms)
    total_stop_probability = math.fsum(stop_probabilities.values())

    # The chosen threshold is intentionally such that some extreme n=30 outcomes stop early,
    # while every path still alive at n=100 is certified sufficient even though n=300 was
    # predeclared. The third batch therefore remains available but is never reached.
    assert stop_probabilities[30] > 0.0
    assert stop_probabilities[100] > 0.0
    assert reached_probabilities[300] < 1e-12
    assert abs(total_stop_probability - 1.0) < 1e-12

    print(
        "fixed_coverages="
        + ",".join(f"n{n}:{fixed_coverages[n]:.12f}" for n in BOUNDARIES)
    )
    print(
        "stop_probabilities="
        + ",".join(f"n{n}:{stop_probabilities[n]:.12f}" for n in BOUNDARIES)
    )
    print(f"sequential_stopped_coverage={sequential_coverage:.12f}")
    print(f"declared_confidence={CONFIDENCE:.12f}")
    print(f"coverage_shortfall={CONFIDENCE - sequential_coverage:.12f}")

    # Predeclared scientific oracle. Because each fixed boundary independently meets the nominal
    # coverage control, failure here demonstrates the sequential precision-stopping procedure's
    # selection effect rather than merely one fixed-n Wilson interval being under-covered.
    assert sequential_coverage >= CONFIDENCE, (
        "predeclared 95% sequential Bernoulli precision gate under-covers after its own "
        f"width-based stopping rule: exact stopped-procedure coverage={sequential_coverage:.12f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
