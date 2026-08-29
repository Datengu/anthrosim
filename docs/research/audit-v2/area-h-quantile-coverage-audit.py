#!/usr/bin/env python3
"""Independent exact-coverage checker for scientific audit v2 Area H.

This reproduces the current rank-index arithmetic in
scripts/research-monte-carlo-sufficiency.py and evaluates the exact
finite-sample distribution-free coverage of the resulting order-statistic
interval under a continuous population.
"""

from math import ceil, comb, floor, sqrt
from statistics import NormalDist


def current_indices(n: int, p: float, confidence: float = 0.95) -> tuple[int, int]:
    z = NormalDist().inv_cdf(0.5 + confidence / 2.0)
    center = p * (n - 1)
    half = z * sqrt(max(n * p * (1.0 - p), 1e-12))
    return max(0, floor(center - half)), min(n - 1, ceil(center + half))


def exact_coverage(n: int, p: float, lower: int, upper: int) -> float:
    # K is the number of sample values strictly below the true population
    # p-quantile. [X_(lower+1), X_(upper+1)] covers iff lower+1 <= K <= upper.
    return sum(
        comb(n, k) * p**k * (1.0 - p) ** (n - k)
        for k in range(lower + 1, upper + 1)
    )


def main() -> None:
    cases = [
        (0.50, 2),
        (0.90, 8),
        (0.95, 8),
        (0.95, 20),
        (0.99, 20),
        (0.99, 100),
    ]
    for p, n in cases:
        lower, upper = current_indices(n, p)
        coverage = exact_coverage(n, p, lower, upper)
        print(f"p={p:.2f} n={n:3d} ranks=[{lower},{upper}] exact_coverage={coverage:.9f}")

    lower, upper = current_indices(8, 0.95)
    coverage = exact_coverage(8, 0.95, lower, upper)
    assert (lower, upper) == (5, 7)
    assert abs(coverage - 0.3307913507812502) < 1e-15
    assert coverage < 0.95


if __name__ == "__main__":
    main()
