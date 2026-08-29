#!/usr/bin/env python3
"""Independent exact-arithmetic checker for audit-v2 Area D / AV2-005.

This script does not import or call AnthroSim. It preserves the legacy
per-boundary-ceiling reproduction and independently verifies the repaired v20
fixed-point carry rule using integer arithmetic only.
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


def legacy_year_loss(periods: int, deficit_permille: int, reference_max_loss: int = 100) -> tuple[int, int]:
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


def repaired_year_loss(periods: int, deficit_permille: int, reference_max_loss: int = 100) -> tuple[int, int, int]:
    max_budget = 0
    whole_loss = 0
    remainder = 0
    for index in range(periods):
        start, end = bounds(index, periods)
        interval_max = reference_quarter_quantity_for_interval(reference_max_loss, start, end)
        max_budget += interval_max
        numerator = deficit_permille * interval_max + remainder
        whole_loss += numerator // PERMILLE
        remainder = numerator % PERMILLE
    return max_budget, whole_loss, remainder


def main() -> None:
    expected_annual_max = 400
    for periods in range(1, 366):
        legacy_budget, _ = legacy_year_loss(periods, 1)
        repaired_budget, _, _ = repaired_year_loss(periods, 1)
        assert legacy_budget == repaired_budget == expected_annual_max, (periods, legacy_budget, repaired_budget)

    legacy_one_permille = [legacy_year_loss(periods, 1)[1] for periods in range(1, 366)]
    assert legacy_one_permille == list(range(1, 366))

    # v20 must be exactly subdivision-invariant for representative complete-year exposures.
    deficits = [1, 10, 100, 500, 1000]
    for deficit in deficits:
        expected_numerator = deficit * expected_annual_max
        expected = (expected_numerator // PERMILLE, expected_numerator % PERMILLE)
        for periods in range(1, 366):
            budget, whole, remainder = repaired_year_loss(periods, deficit)
            assert budget == expected_annual_max
            assert (whole, remainder) == expected, (deficit, periods, whole, remainder, expected)

    checkpoints = [1, 4, 12, 52, 365]
    expected_rows = {
        1: (0, 400),
        10: (4, 0),
        100: (40, 0),
        500: (200, 0),
        1000: (400, 0),
    }

    print("referenceQuarterMaxLoss=100; annualMaxLossBudget=400")
    print("deficit_permille," + ",".join(f"P{p}" for p in checkpoints))
    for deficit in deficits:
        values = [repaired_year_loss(periods, deficit)[1:] for periods in checkpoints]
        assert all(value == expected_rows[deficit] for value in values)
        rendered = [f"{whole}+{remainder}/1000" for whole, remainder in values]
        print(f"{deficit}," + ",".join(rendered))

    print(
        "legacy_one_permille_all_partitions: "
        f"min={min(legacy_one_permille)}@P1 max={max(legacy_one_permille)}@P365 "
        f"unique={len(set(legacy_one_permille))}"
    )
    print("repaired_representative_deficits_all_partitions: exact subdivision invariance verified for deficits 1,10,100,500,1000 and P=1..365")


if __name__ == "__main__":
    main()
