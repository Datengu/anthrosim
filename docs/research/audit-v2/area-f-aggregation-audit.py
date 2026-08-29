#!/usr/bin/env python3
"""Independent quantitative checks for scientific-audit-v2 Area F.

This checker deliberately does not call AnthroSim implementation code. It consumes the
preserved M9.7 reference result and independently recomputes the two principal paired
aggregation contrasts, then demonstrates why aggregate focal person-days alone cannot
identify temporal aggregation structure.
"""

import argparse
import json
import statistics
from pathlib import Path


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--reference",
        type=Path,
        default=Path("examples/m9-controlled-aggregation-benchmark/reference-result.json"),
    )
    args = parser.parse_args()
    reference = load(args.reference)

    assert reference["benchmarkId"] == "m9_7_controlled_continuous_vs_intermittent_v1"
    assert reference["declaredSeeds"] == list(range(9701, 9709))
    assert len(reference["pairs"]) == 8

    focal_difference_permille = []
    peak_share_permille = []
    for pair in reference["pairs"]:
        control = pair["continuous"]
        treatment = pair["intermittent"]

        assert control["residentPersonDays"] == treatment["residentPersonDays"]
        assert control["visitorPersonDays"] == 0
        assert treatment["visitorPersonDays"] > 0
        assert treatment["daysWithAnyVisitors"] == 270
        assert treatment["journeysCompleted"] > 0
        assert control["permanentMigrations"] == treatment["permanentMigrations"] == 0
        assert control["conditionMortalityDeaths"] == treatment["conditionMortalityDeaths"] == 0

        resident_person_days = control["residentPersonDays"]
        visitor_person_days = treatment["visitorPersonDays"]
        peak_visitors = treatment["peakVisitors"]
        mean_continuous_residents = resident_person_days / 3650.0

        focal_difference_permille.append(visitor_person_days * 1000.0 / resident_person_days)
        peak_share_permille.append(peak_visitors * 1000.0 / mean_continuous_residents)

    median_difference = statistics.median(focal_difference_permille)
    maximum_difference = max(focal_difference_permille)
    median_peak_share = statistics.median(peak_share_permille)
    minimum_peak_share = min(peak_share_permille)

    # Frozen benchmark thresholds.
    assert maximum_difference <= 50.0
    assert minimum_peak_share >= 250.0

    # Adversarial same-total-use construction. Both schedules create exactly 600
    # visitor-person-days, but their temporal concentration is structurally different.
    concentrated_visitors = 20
    concentrated_days = 30
    diffuse_visitors = 10
    diffuse_days = 60
    assert concentrated_visitors * concentrated_days == diffuse_visitors * diffuse_days == 600
    assert concentrated_visitors != diffuse_visitors
    assert concentrated_days != diffuse_days

    print(f"paired seeds: {len(reference['pairs'])}")
    print(f"median focal-person-day difference: {median_difference:.6f} permille")
    print(f"maximum focal-person-day difference: {maximum_difference:.6f} permille")
    print(f"median intermittent peak share: {median_peak_share:.6f} permille")
    print(f"minimum intermittent peak share: {minimum_peak_share:.6f} permille")
    print("same-total-use adversarial example: 20 visitors x 30 days == 10 visitors x 60 days == 600 visitor-person-days")
    print("Area F independent aggregation checks passed")


if __name__ == "__main__":
    main()
