#!/usr/bin/env python3
"""Regression tests for AV6-009 sequential-confidence stopping semantics."""

from __future__ import annotations

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location(
    "mc_test_helpers", HERE / "test-research-monte-carlo-sufficiency-legacy.py"
)
assert SPEC and SPEC.loader
helpers = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(helpers)
mc = helpers.mc


def probability_repeated_look_regression():
    first_batch = list(range(1, 31))
    second_batch = list(range(31, 101))
    terminal_batch = list(range(101, 301))
    plan = helpers.make_plan(
        "probability",
        [first_batch, second_batch, terminal_batch],
        0.1685,
    )

    # This extreme first-boundary sample has a narrow ordinary Wilson interval and was exactly the
    # kind of outcome-dependent early stop that caused AV6-009 undercoverage. The interval remains
    # visible descriptively, but it cannot authorize inferential stopping.
    first_rows = [(seed, 1 if seed <= 4 else 0) for seed in first_batch]
    first = mc.derive(plan, helpers.sample([("probability", first_rows)]), None)
    assert first["precision"]["halfWidth"] <= 0.1685
    assert first["precision"]["sufficient"] is False
    assert first["decision"] == "insufficient_continue_with_declared_next_batch"
    validity = first["precision"]["sequentialStoppingValidity"]
    assert validity["validForInferentialStopping"] is False
    assert validity["predeclaredTerminalReplicatesPerPrimaryGroup"] == 300
    assert "not valid for inferential early stopping" in first["precision"]["confidenceSemantics"]

    # A non-terminal n=100 boundary is likewise monitoring-only even when numerically precise.
    middle_rows = first_rows + [(seed, 1 if seed % 4 == 0 else 0) for seed in second_batch]
    middle = mc.derive(plan, helpers.sample([("probability", middle_rows)]), None)
    assert middle["precision"]["halfWidth"] <= 0.1685
    assert middle["precision"]["sufficient"] is False
    assert middle["decision"] == "insufficient_continue_with_declared_next_batch"
    assert middle["precision"]["sequentialStoppingValidity"]["validForInferentialStopping"] is False

    # Because no path may stop from an intermediate width, every path reaches the predeclared final
    # n=300 boundary. Ordinary fixed-sample interval semantics may therefore govern the terminal
    # precision decision.
    terminal_rows = middle_rows + [
        (seed, 1 if seed % 3 == 0 else 0) for seed in terminal_batch
    ]
    terminal = mc.derive(plan, helpers.sample([("probability", terminal_rows)]), None)
    assert terminal["precision"]["sequentialStoppingValidity"]["validForInferentialStopping"] is True
    assert terminal["precision"]["sufficient"] is True
    assert terminal["decision"] == "sufficient_stop"


def other_estimator_families_fail_closed_at_intermediate_looks():
    # AV6-009 was demonstrated with Wilson, but the same repeated-look contract must not silently
    # assume fixed-sample intervals for the other supported families are always-valid.
    first = list(range(1, 31))
    second = list(range(31, 61))

    mean_plan = helpers.make_plan("mean", [first, second], 100.0)
    mean_rows = [(seed, float(seed % 5)) for seed in first]
    mean = mc.derive(mean_plan, helpers.sample([("mean", mean_rows)]), None)
    assert mean["precision"]["normalApproximationValidity"]["validForStopping"] is True
    assert mean["precision"]["sufficient"] is False
    assert mean["precision"]["sequentialStoppingValidity"]["validForInferentialStopping"] is False

    quantile_plan = helpers.make_plan("quantile", [first, second], 100.0, quantile=0.5)
    quantile_rows = [(seed, float(seed)) for seed in first]
    quantile = mc.derive(
        quantile_plan, helpers.sample([("quantile", quantile_rows)]), None
    )
    assert quantile["precision"]["coverageFeasible"] is True
    assert quantile["precision"]["sufficient"] is False
    assert quantile["precision"]["sequentialStoppingValidity"]["validForInferentialStopping"] is False


if __name__ == "__main__":
    probability_repeated_look_regression()
    other_estimator_families_fail_closed_at_intermediate_looks()
    print("AV6-009 terminal-only sequential inference regressions passed")
