#!/usr/bin/env python3
"""Independent exact-arithmetic checker for audit-v2 Area D / AV2-005.

This mirrors only the integer equations used by the current M3 condition-loss path:
1. divide each model year into exact half-open M3 intervals;
2. allocate a reference-quarter maximum-loss quantity cumulatively across those intervals;
3. apply the current per-boundary ceiling to a fixed supply deficit.

It is intentionally independent of the Rust implementation and uses integer arithmetic only.
"""

DAYS_PER_YEAR = 365
REFERENCE_PERIODS = 4
PERMILLE = 1000


def bounds(index: int, periods: int) -> tuple[int, int]:
    return index * DAYS_PER_YEAR // periods, (index + 1) * DAYS_PER_YEAR // periods


def reference_quarter_quantity_for_interval(reference: int, start: int, end: int) -> int:
    total = 0
    for quarter in range(REFERENCE_PERIODS):
        quarter_start, quarter_end = bounds(quarter, REFERENCE_PERIODS)
        overlap_start = max(start, quarter_start)
        overlap_end = min(end, quarter_end)
        if overlap_start >= overlap_end:
            continue
        length = quarter_end - quarter_start
        local_start = overlap_start - quarter_start
        local_end = overlap_end - quarter_start
        before = reference * local_start // length
        after = reference * local_end // length
        total += after - before
    return total


def current_year_loss(periods: int, deficit_permille: int, reference_max_loss: int = 100) -> tuple[int, int]:
    max_budget = 0
    realized = 0
    for index in range(periods):
        start, end = bounds(index, periods)
        interval_max = reference_quarter_quantity_for_interval(reference_max_loss, start, end)
        max_budget += interval_max
        numerator = deficit_permille * interval_max
        if numerator:
            realized += (numerator + PERMILLE - 1) // PERMILLE
    return max_budget, realized


def main() -> None:
    expected_annual_max = 4 * 100
    for periods in range(1, 366):
        budget, _ = current_year_loss(periods, 1)
        assert budget == expected_annual_max, (periods, budget)

    one_permille = [current_year_loss(periods, 1)[1] for periods in range(1, 366)]
    assert one_permille == list(range(1, 366))

    checkpoints = [1, 4, 12, 52, 365]
    deficits = [1, 10, 50, 100, 500, 999, 1000]
    print("referenceQuarterMaxLoss=100; annualMaxLossBudget=400")
    print("deficit_permille," + ",".join(f"P{p}" for p in checkpoints))
    for deficit in deficits:
        values = [current_year_loss(periods, deficit)[1] for periods in checkpoints]
        print(f"{deficit}," + ",".join(map(str, values)))

    print(
        "one_permille_all_partitions: "
        f"min={min(one_permille)}@P1 max={max(one_permille)}@P365 unique={len(set(one_permille))}"
    )


if __name__ == "__main__":
    main()
