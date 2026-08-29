#!/usr/bin/env python3
"""Independent exact-arithmetic adversarial checks for scientific audit v2 Area A.

This intentionally re-expresses the documented/source scheduler arithmetic without
calling AnthroSim itself. Source inspection on the audited SHA must separately
confirm that both authoritative hosts implement the same formulas/order.
"""

from collections import Counter
from fractions import Fraction

DAYS_PER_YEAR = 365
PROBABILITY_SCALE = 1_000_000


def partition_bounds(periods: int):
    return [
        (
            index * DAYS_PER_YEAR // periods,
            (index + 1) * DAYS_PER_YEAR // periods,
        )
        for index in range(periods)
    ]


def boundary_days(periods: int):
    return [end for _, end in partition_bounds(periods)]


def merged_dispatch(resource_periods: int, migration_periods: int):
    resource_index = 0
    migration_index = 0
    dispatches = []
    while resource_index < resource_periods or migration_index < migration_periods:
        resource_day = (
            (resource_index + 1) * DAYS_PER_YEAR // resource_periods
            if resource_index < resource_periods
            else None
        )
        migration_day = (
            (migration_index + 1) * DAYS_PER_YEAR // migration_periods
            if migration_index < migration_periods
            else None
        )
        day = min(value for value in (resource_day, migration_day) if value is not None)
        resource_due = resource_day == day
        migration_due = migration_day == day
        dispatches.append((day, resource_due, migration_due))
        resource_index += int(resource_due)
        migration_index += int(migration_due)
    return dispatches


def annual_background_interval_probability(annual_probability: int, start: int, end: int):
    if annual_probability == 0 or start == end:
        return Fraction(0, 1)
    return Fraction(
        annual_probability * (end - start),
        PROBABILITY_SCALE * DAYS_PER_YEAR - annual_probability * start,
    )


def main():
    invalid_partitions = []
    interval_lengths = Counter()
    for periods in range(1, DAYS_PER_YEAR + 1):
        intervals = partition_bounds(periods)
        days = [end for _, end in intervals]
        interval_lengths.update(end - start for start, end in intervals)
        if (
            len(days) != periods
            or len(set(days)) != periods
            or days[-1] != DAYS_PER_YEAR
            or any(start >= end for start, end in intervals)
            or any(day < 1 or day > DAYS_PER_YEAR for day in days)
        ):
            invalid_partitions.append(periods)
    assert not invalid_partitions, invalid_partitions

    merge_pair_count = 0
    merge_failures = []
    collision_counts = Counter()
    for resource_periods in range(1, DAYS_PER_YEAR + 1):
        resource_days = set(boundary_days(resource_periods))
        for migration_periods in range(1, DAYS_PER_YEAR + 1):
            merge_pair_count += 1
            migration_days = set(boundary_days(migration_periods))
            dispatches = merged_dispatch(resource_periods, migration_periods)
            dispatch_days = [day for day, _, _ in dispatches]
            collision_count = sum(
                resource_due and migration_due
                for _, resource_due, migration_due in dispatches
            )
            collision_counts[collision_count] += 1

            expected_union = sorted(resource_days | migration_days)
            if dispatch_days != expected_union:
                merge_failures.append((resource_periods, migration_periods, "union"))
            if sum(resource_due for _, resource_due, _ in dispatches) != resource_periods:
                merge_failures.append((resource_periods, migration_periods, "resource_count"))
            if sum(migration_due for _, _, migration_due in dispatches) != migration_periods:
                merge_failures.append((resource_periods, migration_periods, "migration_count"))
            if collision_count != len(resource_days & migration_days):
                merge_failures.append((resource_periods, migration_periods, "collision_count"))
            if any(
                dispatch_days[index] >= dispatch_days[index + 1]
                for index in range(len(dispatch_days) - 1)
            ):
                merge_failures.append((resource_periods, migration_periods, "ordering"))
    assert not merge_failures, merge_failures[:10]

    probabilities = [0, 1, 50_000, 200_000, 500_000, 999_999, 1_000_000]
    mortality_failures = []
    for annual_probability in probabilities:
        expected_survival = Fraction(
            PROBABILITY_SCALE - annual_probability,
            PROBABILITY_SCALE,
        )
        for periods in range(1, DAYS_PER_YEAR + 1):
            survival = Fraction(1, 1)
            for start, end in partition_bounds(periods):
                interval_probability = annual_background_interval_probability(
                    annual_probability,
                    start,
                    end,
                )
                if not Fraction(0, 1) <= interval_probability <= Fraction(1, 1):
                    mortality_failures.append(
                        (annual_probability, periods, start, end, "range")
                    )
                    break
                survival *= 1 - interval_probability
            if survival != expected_survival:
                mortality_failures.append(
                    (annual_probability, periods, "annual_survival")
                )
    assert not mortality_failures, mortality_failures[:10]

    print(f"resource_partitions_checked={DAYS_PER_YEAR}")
    print(f"resource_migration_clock_pairs_checked={merge_pair_count}")
    print(f"minimum_same_day_collisions={min(collision_counts)}")
    print(f"maximum_same_day_collisions={max(collision_counts)}")
    print(
        "clock_pairs_with_multiple_collisions="
        f"{sum(count for collisions, count in collision_counts.items() if collisions > 1)}"
    )
    print(f"mortality_probabilities_checked={len(probabilities)}")
    print(
        "mortality_partition_compositions_checked="
        f"{len(probabilities) * DAYS_PER_YEAR}"
    )
    print("failures=0")


if __name__ == "__main__":
    main()
