#!/usr/bin/env python3
"""Independent arithmetic/metamorphic checker for audit-v2 Area E.

This mirrors only two small current contracts without importing AnthroSim:
- M4 Manhattan-radius candidate enumeration; and
- M9 equal-minimum keyed destination selection.

It checks candidate-set reflection symmetry, quantifies edge clipping and stable-order
non-equivariance, and measures the current keyed two-way M9 tie distribution.
"""

FNV_OFFSET_BASIS = 0xCBF29CE484222325
FNV_PRIME = 0x00000100000001B3
MASK64 = (1 << 64) - 1
TIE_POLICY = b"m9/equal-cost-destination-keyed-v1"


def candidates(radius: int) -> list[tuple[int, int]]:
    out: list[tuple[int, int]] = []
    for dy in range(-radius, radius + 1):
        remaining = radius - abs(dy)
        for dx in range(-remaining, remaining + 1):
            if dx == 0 and dy == 0:
                continue
            out.append((dx, dy))
    return out


def corner_candidate_count(radius: int) -> int:
    return sum(1 for dx, dy in candidates(radius) if dx >= 0 and dy >= 0)


def digest_u64(value: int, hash_value: int) -> int:
    for byte in value.to_bytes(8, "little"):
        hash_value ^= byte
        hash_value = (hash_value * FNV_PRIME) & MASK64
    return hash_value


def digest_bytes(value: bytes, hash_value: int) -> int:
    hash_value = digest_u64(len(value), hash_value)
    for byte in value:
        hash_value ^= byte
        hash_value = (hash_value * FNV_PRIME) & MASK64
    return hash_value


def avalanche64(value: int) -> int:
    value ^= value >> 30
    value = (value * 0xBF58476D1CE4E5B9) & MASK64
    value ^= value >> 27
    value = (value * 0x94D049BB133111EB) & MASK64
    return (value ^ (value >> 31)) & MASK64


def keyed_tie_index(seed: int, origin: int, household: int, trigger: int, count: int) -> int:
    h = FNV_OFFSET_BASIS
    h = digest_bytes(TIE_POLICY, h)
    h = digest_u64(seed, h)
    h = digest_u64(origin, h)
    h = digest_u64(household, h)
    h = digest_u64(trigger, h)
    return avalanche64(h) % count


def main() -> None:
    reflection_failures = 0
    for radius in range(1, 33):
        ordered = candidates(radius)
        expected = 2 * radius * (radius + 1)
        assert len(ordered) == expected
        reflected_set = {(-dx, dy) for dx, dy in ordered}
        if reflected_set != set(ordered):
            reflection_failures += 1
    assert reflection_failures == 0
    print(f"m4_radii_1_32: reflection_set_failures={reflection_failures}")

    for radius in [1, 2, 3, 4, 8, 16, 32]:
        ordered = candidates(radius)
        mirrored_same_index = sum(
            1 for index, (dx, dy) in enumerate(ordered)
            if ordered[index] == (-dx, dy)
        )
        # More useful metamorphic diagnostic: compare the original order with the reflected
        # original order. Stable row-major candidate enumeration is intentionally not a
        # coordinate-free common-random-number coupling.
        reflected_order = [(-dx, dy) for dx, dy in ordered]
        positional_matches = sum(a == b for a, b in zip(ordered, reflected_order))
        mismatch = len(ordered) - positional_matches
        assert mirrored_same_index == positional_matches
        print(
            f"m4_radius={radius}: interior={len(ordered)} corner={corner_candidate_count(radius)} "
            f"reflection_order_mismatch={mismatch}/{len(ordered)}"
        )

    counts = [0, 0]
    for seed in range(1, 100_001):
        counts[keyed_tie_index(seed, origin=2, household=1, trigger=0, count=2)] += 1
    assert counts == [49_886, 50_114], counts
    print(
        "m9_two_way_equal_minimum_seeds_1_100000: "
        f"candidate0={counts[0]} candidate1={counts[1]}"
    )


if __name__ == "__main__":
    main()
