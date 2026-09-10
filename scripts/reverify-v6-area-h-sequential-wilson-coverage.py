#!/usr/bin/env python3
"""Post-merge AV6-009 re-verification under the audit version-drift protocol.

The original discovery adversary remains unchanged beside this wrapper. Its terminal assertions
intentionally require the historical defect (early sufficient_stop decisions) to remain present,
so a correct repair makes that historical oracle fail before it can report repaired coverage.
This wrapper first requires that exact historical failure, then repeats the same exact finite
Bernoulli enumeration with the current-state acceptance oracle only.
"""

from __future__ import annotations

import importlib.util
import math
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL_PATH = ROOT / "scripts" / "audit-v6-area-h-sequential-wilson-coverage.py"


def _load_original():
    spec = importlib.util.spec_from_file_location("av6_009_original", ORIGINAL_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {ORIGINAL_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    # Preserve and execute the historical discovery oracle itself. A correct repair must make its
    # first explicit defect-presence assertion fail because no intermediate path may sufficient_stop.
    historical = subprocess.run(
        [sys.executable, str(ORIGINAL_PATH)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert historical.returncode != 0, "historical defect-presence oracle unexpectedly still passes"
    assert "assert stop_probabilities[30] > 0.0" in historical.stderr, (
        "historical oracle did not fail at the expected repaired early-stop assertion:\n"
        + historical.stderr
    )

    original = _load_original()
    gate = original.load_gate()
    plan = original.make_plan(gate)
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
                            {"seed": seed, "value": 1 if seed < successes else 0}
                            for seed in range(n)
                        ],
                    }
                ],
            }
            cache[key] = gate.derive(plan, samples, None)
        return cache[key]

    # Keep the discovery control exactly in substance: each ordinary fixed-n Wilson procedure must
    # itself cover at least the declared 95%, so the repair is not hiding a fixed-boundary problem.
    fixed_coverages: dict[int, float] = {}
    for n in original.BOUNDARIES:
        covered = []
        for successes in range(n + 1):
            diagnostic = result(n, successes)
            precision = diagnostic["precision"]
            assert precision["precisionMethod"] == "wilson_score_probability"
            if precision["intervalLower"] <= original.TRUE_P <= precision["intervalUpper"]:
                covered.append(original.binomial_mass(n, successes, original.TRUE_P))
        fixed_coverages[n] = math.fsum(covered)
        assert fixed_coverages[n] >= original.CONFIDENCE

    live = {0: 1.0}
    previous_n = 0
    stopped_covered_terms: list[float] = []
    stop_probabilities: dict[int, float] = {}
    reached_probabilities: dict[int, float] = {}

    for boundary_index, n in enumerate(original.BOUNDARIES):
        increment = n - previous_n
        expanded: dict[int, float] = {}
        for previous_successes, path_probability in live.items():
            for added_successes in range(increment + 1):
                successes = previous_successes + added_successes
                probability = path_probability * original.binomial_mass(
                    increment, added_successes, original.TRUE_P
                )
                expanded[successes] = expanded.get(successes, 0.0) + probability

        reached_probabilities[n] = math.fsum(expanded.values())
        next_live: dict[int, float] = {}
        stopped_here: list[float] = []
        final_boundary = boundary_index == len(original.BOUNDARIES) - 1

        for successes, path_probability in expanded.items():
            diagnostic = result(n, successes)
            precision = diagnostic["precision"]
            validity = precision["sequentialStoppingValidity"]
            should_stop = diagnostic["decision"] == "sufficient_stop"

            if final_boundary:
                assert validity["validForInferentialStopping"] is True
                # The predeclared design ends here even when a particular terminal interval remains
                # wider than the requested precision threshold, exactly as in the discovery accounting.
                should_stop = True
            else:
                assert validity["validForInferentialStopping"] is False
                assert diagnostic["decision"] == "insufficient_continue_with_declared_next_batch"
                assert precision["sufficient"] is False

            if should_stop:
                stopped_here.append(path_probability)
                if precision["intervalLower"] <= original.TRUE_P <= precision["intervalUpper"]:
                    stopped_covered_terms.append(path_probability)
            else:
                next_live[successes] = path_probability

        stop_probabilities[n] = math.fsum(stopped_here)
        live = next_live
        previous_n = n

    sequential_coverage = math.fsum(stopped_covered_terms)
    total_stop_probability = math.fsum(stop_probabilities.values())

    # Current-state acceptance oracle: ordinary fixed-sample intervals are monitoring-only until
    # the predeclared terminal boundary, eliminating the outcome-dependent stopping selection that
    # produced AV6-009's 91.8977% stopped-procedure coverage.
    assert abs(stop_probabilities[30]) < 1e-15
    assert abs(stop_probabilities[100]) < 1e-15
    assert abs(reached_probabilities[300] - 1.0) < 1e-12
    assert abs(stop_probabilities[300] - 1.0) < 1e-12
    assert abs(total_stop_probability - 1.0) < 1e-12
    assert abs(sequential_coverage - fixed_coverages[300]) < 1e-12
    assert sequential_coverage >= original.CONFIDENCE

    print("historical_defect_presence_oracle=failed_at_n30_early_stop_assertion")
    print(
        "fixed_coverages="
        + ",".join(
            f"n{n}:{fixed_coverages[n]:.12f}" for n in original.BOUNDARIES
        )
    )
    print(
        "stop_probabilities="
        + ",".join(
            f"n{n}:{stop_probabilities[n]:.12f}" for n in original.BOUNDARIES
        )
    )
    print(
        "reached_probabilities="
        + ",".join(
            f"n{n}:{reached_probabilities[n]:.12f}" for n in original.BOUNDARIES
        )
    )
    print(f"repaired_stopped_procedure_coverage={sequential_coverage:.12f}")
    print(f"declared_confidence={original.CONFIDENCE:.12f}")
    print("av6_009_reverification=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
