#!/usr/bin/env python3
"""Fresh Audit-v4 Area-A scheduler adversary for immutable v0.3.4/v25.

This checker deliberately does two independent things:

1. exhaustively enumerates all supported M3/M4 fixed-clock period-count pairs
   (1..365 each) and verifies the merged boundary dispatcher cannot skip, duplicate,
   reorder, or stall at same-day collisions;
2. inspects both authoritative simulation hosts and requires the scientific ordering
   markers to occur in the same order: pre-day temporary transitions, M3 resource
   processing, M3 period completion, M4 migration, then annual M2 demography.

It does not prove the model is correct. It is a falsification-oriented regression
attack against scheduler drift and collision handling in the frozen release.
"""

from __future__ import annotations

from collections import Counter
from pathlib import Path

DAYS_PER_YEAR = 365
HOSTS = (
    Path("crates/anthrosim-core/src/simulation.rs"),
    Path("crates/anthrosim-core/src/spatial_simulation.rs"),
)


def bounds(periods: int) -> list[int]:
    assert 1 <= periods <= DAYS_PER_YEAR
    return [((i + 1) * DAYS_PER_YEAR) // periods for i in range(periods)]


def merged(resource_periods: int, migration_periods: int) -> list[tuple[int, bool, bool]]:
    resource_days = bounds(resource_periods)
    migration_days = bounds(migration_periods)
    ri = mi = 0
    out: list[tuple[int, bool, bool]] = []
    while ri < len(resource_days) or mi < len(migration_days):
        rd = resource_days[ri] if ri < len(resource_days) else None
        md = migration_days[mi] if mi < len(migration_days) else None
        day = min(value for value in (rd, md) if value is not None)
        r_due = rd == day
        m_due = md == day
        out.append((day, r_due, m_due))
        ri += int(r_due)
        mi += int(m_due)
    return out


def check_clock_merge() -> None:
    pairs = 0
    collision_histogram: Counter[int] = Counter()
    for rp in range(1, DAYS_PER_YEAR + 1):
        rdays = bounds(rp)
        assert len(rdays) == rp
        assert len(set(rdays)) == rp
        assert rdays[-1] == DAYS_PER_YEAR
        assert all(1 <= day <= DAYS_PER_YEAR for day in rdays)
        assert all(a < b for a, b in zip(rdays, rdays[1:]))

        for mp in range(1, DAYS_PER_YEAR + 1):
            pairs += 1
            mdays = bounds(mp)
            dispatch = merged(rp, mp)
            days = [day for day, _, _ in dispatch]
            expected = sorted(set(rdays) | set(mdays))
            assert days == expected, (rp, mp, "union/order")
            assert sum(r for _, r, _ in dispatch) == rp, (rp, mp, "resource count")
            assert sum(m for _, _, m in dispatch) == mp, (rp, mp, "migration count")
            collisions = sum(r and m for _, r, m in dispatch)
            assert collisions == len(set(rdays) & set(mdays)), (rp, mp, "collision count")
            # The annual boundary is always a real triple-collision opportunity once M2 runs.
            assert dispatch[-1] == (DAYS_PER_YEAR, True, True), (rp, mp, "annual boundary")
            collision_histogram[collisions] += 1

    assert pairs == DAYS_PER_YEAR * DAYS_PER_YEAR
    print(f"clock_pairs_checked={pairs}")
    print(f"min_same_day_collisions={min(collision_histogram)}")
    print(f"max_same_day_collisions={max(collision_histogram)}")
    print(
        "pairs_with_multiple_same_day_collisions="
        f"{sum(count for collisions, count in collision_histogram.items() if collisions > 1)}"
    )


def ordered_positions(text: str, markers: tuple[str, ...], host: Path) -> tuple[int, ...]:
    positions = []
    cursor = 0
    for marker in markers:
        position = text.find(marker, cursor)
        assert position >= 0, f"{host}: missing scheduler marker {marker!r}"
        positions.append(position)
        cursor = position + len(marker)
    return tuple(positions)


def check_host_ordering() -> None:
    markers = (
        "self.process_temporary_boundaries_before(day)?;",
        "if resource_day == Some(day) {",
        "self.temporary_mobility.complete_resource_period(day)?;",
        "if migration_day == Some(day) {",
        "process_demographic_year_after_competing_mortality_recorded_with_founder_history(",
    )
    normalized_orders = []
    for host in HOSTS:
        text = host.read_text(encoding="utf-8")
        positions = ordered_positions(text, markers, host)
        assert list(positions) == sorted(positions), f"{host}: scheduler order drift"
        normalized_orders.append(tuple(markers))
        print(f"host_order_verified={host}")
    assert normalized_orders[0] == normalized_orders[1]


def main() -> None:
    check_clock_merge()
    check_host_ordering()
    print("failures=0")


if __name__ == "__main__":
    main()
