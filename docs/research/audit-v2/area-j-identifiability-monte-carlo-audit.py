#!/usr/bin/env python3
"""Independent adversarial checker for audit-v2 Area J / issue #338.

This script does not import AnthroSim's identifiability analyzer. It compares the
legacy point-estimate decision with an interval-aware compatibility decision for
the same fixed calibration estimates at two Monte Carlo precision levels.
"""

TARGET = 0.0
TOLERANCE = 0.05
POINTS = {"theta-0": 0.00, "theta-1": 0.10}


def legacy_accepts(estimate: float) -> bool:
    return abs(estimate - TARGET) <= TOLERANCE


def interval_status(estimate: float, half_width: float) -> str:
    lower, upper = estimate - half_width, estimate + half_width
    band_lower, band_upper = TARGET - TOLERANCE, TARGET + TOLERANCE
    if upper < band_lower or lower > band_upper:
        return "rejected"
    if lower >= band_lower and upper <= band_upper:
        return "acceptable"
    return "unresolved"


def compatible_parameter_values(half_width: float) -> list[str]:
    return [point_id for point_id, estimate in POINTS.items() if interval_status(estimate, half_width) != "rejected"]


def main() -> None:
    legacy = {point_id: legacy_accepts(estimate) for point_id, estimate in POINTS.items()}
    assert legacy == {"theta-0": True, "theta-1": False}

    low_precision = {point_id: interval_status(estimate, 0.20) for point_id, estimate in POINTS.items()}
    high_precision = {point_id: interval_status(estimate, 0.01) for point_id, estimate in POINTS.items()}

    assert low_precision == {"theta-0": "unresolved", "theta-1": "unresolved"}
    assert compatible_parameter_values(0.20) == ["theta-0", "theta-1"]
    assert high_precision == {"theta-0": "acceptable", "theta-1": "rejected"}
    assert compatible_parameter_values(0.01) == ["theta-0"]

    print(f"target={TARGET:.2f} tolerance={TOLERANCE:.2f}")
    print(f"legacy_point_estimate_verdict={legacy}")
    print(f"same_estimates_halfwidth_0.20={low_precision} compatible={compatible_parameter_values(0.20)}")
    print(f"same_estimates_halfwidth_0.01={high_precision} compatible={compatible_parameter_values(0.01)}")
    print("independent result: stochastic precision alone changes the scientifically supportable identifiability verdict")


if __name__ == "__main__":
    main()
