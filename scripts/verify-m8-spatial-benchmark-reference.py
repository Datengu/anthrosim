#!/usr/bin/env python3
"""Verify an M8.6 aggregate against the preserved scientific reference.

The comparison deliberately ignores source/release provenance that is expected to
change between builds. It requires the scientific baseline itself to remain
identical: classification, primary paired-effect summaries, arm mechanism
identity/degeneracy status, and every authoritative terminal state digest.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ARMS = ("flat", "weak", "moderate", "strong")
TREATMENT_ARMS = ("weak", "moderate", "strong")
PRIMARY_METRIC_COMPARISON_KEYS = (
    "medianEffect",
    "medianAbsoluteRelativeEffect",
    "positiveEffects",
    "negativeEffects",
    "zeroEffects",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--actual", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    return parser.parse_args()


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def require_equal(label: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise SystemExit(
            f"M8.6 scientific regression mismatch for {label}: "
            f"expected {expected!r}, found {actual!r}"
        )


def verify(actual: dict[str, Any], reference: dict[str, Any]) -> None:
    require_equal("schemaVersion", actual.get("schemaVersion"), reference.get("schemaVersion"))
    require_equal("benchmarkId", actual.get("benchmarkId"), reference.get("benchmarkId"))
    require_equal("declaredSeeds", actual.get("declaredSeeds"), reference.get("declaredSeeds"))
    require_equal("classification", actual.get("classification"), reference.get("classification"))

    actual_metrics = actual.get("primaryMetrics", {})
    reference_metrics = reference.get("primaryMetrics", {})
    require_equal(
        "primary metric names",
        sorted(actual_metrics),
        sorted(reference_metrics),
    )
    for metric in sorted(reference_metrics):
        actual_metric = actual_metrics[metric]
        reference_metric = reference_metrics[metric]
        require_equal(
            f"{metric}.classification",
            actual_metric.get("classification"),
            reference_metric.get("classification"),
        )
        require_equal(
            f"{metric}.robustCriteria",
            actual_metric.get("robustCriteria"),
            reference_metric.get("robustCriteria"),
        )
        comparisons = actual_metric.get("comparisonsToFlat", {})
        for arm in TREATMENT_ARMS:
            actual_arm = comparisons.get(arm, {})
            reference_arm = reference_metric.get(arm, {})
            for key in PRIMARY_METRIC_COMPARISON_KEYS:
                require_equal(
                    f"{metric}.{arm}.{key}",
                    actual_arm.get(key),
                    reference_arm.get(key),
                )

    actual_arms = actual.get("arms", {})
    reference_arms = reference.get("arms", {})
    for arm in ARMS:
        actual_arm = actual_arms.get(arm, {})
        reference_arm = reference_arms.get(arm, {})
        require_equal(
            f"{arm}.mechanismsCanonicalSha256",
            actual_arm.get("mechanismsCanonicalSha256"),
            reference_arm.get("mechanismsCanonicalSha256"),
        )
        require_equal(
            f"{arm}.terminalDegenerateRuns",
            actual_arm.get("terminalDegenerateRuns"),
            reference_arm.get("terminalDegenerateRuns"),
        )

        actual_runs = actual_arm.get("runs", {})
        expected_state_digests = reference_arm.get("runStateDigest64", {})
        require_equal(
            f"{arm}.run seeds",
            sorted(actual_runs),
            sorted(expected_state_digests),
        )
        for seed, expected_digest in sorted(expected_state_digests.items()):
            run = actual_runs.get(seed, {})
            require_equal(
                f"{arm}.seed-{seed}.stateDigest64",
                run.get("stateDigest64"),
                expected_digest,
            )
            require_equal(
                f"{arm}.seed-{seed}.spatialConfigIdentity",
                run.get("spatialConfigIdentity"),
                reference_arm.get("spatialConfigIdentity"),
            )


def main() -> None:
    args = parse_args()
    actual = read_json(args.actual)
    reference = read_json(args.reference)
    verify(actual, reference)
    print("M8.6 scientific regression baseline matches preserved reference")


if __name__ == "__main__":
    main()
