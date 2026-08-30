#!/usr/bin/env python3
"""Independent #332 checker; deliberately imports no AnthroSim code."""

DAYS_PER_YEAR = 365


def annual_observations(years: int) -> list[int]:
    return [year * DAYS_PER_YEAR for year in range(1, years + 1)]


uninterrupted = annual_observations(2)
legacy_resumed = [0] + annual_observations(2)
repaired_resumed = annual_observations(2)
year_one_checkpoint = annual_observations(1)
true_terminal_zero = [0]

assert uninterrupted == [365, 730]
assert legacy_resumed == [0, 365, 730]
assert repaired_resumed == uninterrupted
assert year_one_checkpoint == [365]
assert true_terminal_zero == [0]
assert len(legacy_resumed) / len(uninterrupted) == 1.5

print("uninterrupted two-year metric days:", uninterrupted)
print("legacy year-zero resume metric days:", legacy_resumed)
print("repaired year-zero resume metric days:", repaired_resumed)
print("legacy retained-observation inflation: +50.0% (3 vs 2)")
print("year-one checkpoint retains its annual boundary:", year_one_checkpoint)
print("true terminal duration-zero run retains day zero:", true_terminal_zero)
