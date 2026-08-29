#!/usr/bin/env python3
"""Independent exact-coverage checker for scientific audit v2 Area H.

This script does not import AnthroSim's Monte Carlo gate. It preserves the
legacy normal-rank reproduction that demonstrated #334, then independently
checks the repaired finite-sample binomial/order-statistic contract.
"""

from math import ceil, comb, floor, fsum, sqrt
from statistics import NormalDist


CONFIDENCE = 0.95


def legacy_indices(n: int, p: float, confidence: float = CONFIDENCE) -> tuple[int, int]:
    z = NormalDist().inv_cdf(0.5 + confidence / 2.0)
    center = p * (n - 1)
    half = z * sqrt(max(n * p * (1.0 - p), 1e-12))
    return max(0, floor(center - half)), min(n - 1, ceil(center + half))


def binomial_masses(n: int, p: float) -> list[float]:
    return [comb(n, k) * p**k * (1.0 - p) ** (n - k) for k in range(n + 1)]


def exact_coverage(n: int, p: float, lower: int, upper: int) -> float:
    # K is the number of sample values strictly below the true population
    # p-quantile. [X_(lower+1), X_(upper+1)] covers iff lower+1 <= K <= upper.
    masses = binomial_masses(n, p)
    return fsum(masses[lower + 1 : upper + 1])


def maximum_sample_range_coverage(n: int, p: float) -> float:
    masses = binomial_masses(n, p)
    return fsum(masses[1:n])


def independent_exact_interval(n: int, p: float, confidence: float = CONFIDENCE):
    masses = binomial_masses(n, p)
    maximum = fsum(masses[1:n])
    tolerance = 1e-12
    if maximum + tolerance < confidence:
        return None, maximum

    cumulative = []
    running = 0.0
    for mass in masses:
        running += mass
        cumulative.append(running)

    estimate_position = p * (n - 1)
    estimate_lower = floor(estimate_position)
    estimate_upper = ceil(estimate_position)
    best = None
    for lower in range(n - 1):
        if lower > estimate_lower:
            break
        for upper in range(max(lower + 1, estimate_upper), n):
            coverage = cumulative[upper] - cumulative[lower]
            if coverage + tolerance < confidence:
                continue
            lower_tail = cumulative[lower]
            upper_tail = max(0.0, 1.0 - cumulative[upper])
            key = (upper - lower, abs(lower_tail - upper_tail), lower)
            if best is None or key < best[0]:
                best = (key, lower, upper, coverage)
    assert best is not None
    _, lower, upper, coverage = best
    return (lower, upper, coverage), maximum


def minimum_feasible_n(p: float, confidence: float = CONFIDENCE) -> int:
    n = 2
    while maximum_sample_range_coverage(n, p) + 1e-12 < confidence:
        n += 1
    return n


def main() -> None:
    legacy_cases = [
        (0.50, 2, 0.500000000),
        (0.90, 8, 0.564530790),
        (0.95, 8, 0.330791351),
        (0.95, 20, 0.625583078),
        (0.99, 20, 0.181093062),
        (0.99, 100, 0.615629207),
    ]
    print("legacy nominal-95% rank intervals")
    for p, n, expected in legacy_cases:
        lower, upper = legacy_indices(n, p)
        coverage = exact_coverage(n, p, lower, upper)
        print(f"p={p:.2f} n={n:3d} ranks=[{lower},{upper}] exact_coverage={coverage:.9f}")
        assert abs(coverage - expected) < 5e-9
        assert coverage < CONFIDENCE

    expected_minimum = {0.50: 6, 0.90: 29, 0.95: 59, 0.99: 299}
    print("\nrepaired exact finite-sample support")
    for p, expected_n in expected_minimum.items():
        actual_n = minimum_feasible_n(p)
        assert actual_n == expected_n
        before = maximum_sample_range_coverage(actual_n - 1, p)
        at = maximum_sample_range_coverage(actual_n, p)
        assert before < CONFIDENCE <= at + 1e-12
        interval, maximum = independent_exact_interval(actual_n, p)
        assert interval is not None
        lower, upper, coverage = interval
        assert coverage + 1e-12 >= CONFIDENCE
        assert abs(coverage - exact_coverage(actual_n, p, lower, upper)) < 1e-12
        print(
            f"p={p:.2f} min_n={actual_n:3d} max_coverage={maximum:.9f} "
            f"selected_ranks=[{lower},{upper}] achieved={coverage:.9f}"
        )

    # The exact audit examples must now fail closed because even [min,max]
    # cannot attain 95% coverage at their sample sizes.
    for p, n in [(0.50, 2), (0.90, 8), (0.95, 8), (0.95, 20), (0.99, 20), (0.99, 100)]:
        interval, maximum = independent_exact_interval(n, p)
        assert interval is None
        assert maximum < CONFIDENCE

    print("\nrepaired contract: all accepted intervals independently meet nominal finite-sample coverage; under-supported cases are infeasible")


if __name__ == "__main__":
    main()
