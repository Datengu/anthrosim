#!/usr/bin/env python3
"""Independent arithmetic checker for audit-v2 issue #324.

This checker deliberately does not import AnthroSim. It reproduces the legacy
contiguous-ID failure and verifies the dependency-anchor contract separately.
"""

from collections import Counter


def balanced_targets(n: int, groups: int) -> list[int]:
    base, extra = divmod(n, groups)
    return [base + int(i < extra) for i in range(groups)]


def legacy_partition(records: list[tuple[str, int]], ceiling: int):
    groups = (len(records) + ceiling - 1) // ceiling
    sizes = balanced_targets(len(records), groups)
    out, cursor = [], 0
    for size in sizes:
        out.append(records[cursor : cursor + size])
        cursor += size
    return out


def dependency_partition(records: list[tuple[str, int]], ceiling: int, independent_age: int):
    required = (len(records) + ceiling - 1) // ceiling
    anchors = sorted((r for r in records if r[1] >= independent_age), key=lambda r: (-r[1], r[0]))
    group_count = min(required, len(anchors))
    if group_count < 2:
        return [records[:]]

    targets = balanced_targets(len(records), group_count)
    groups = [[] for _ in range(group_count)]
    for i, anchor in enumerate(anchors):
        groups[i % group_count].append(anchor)

    dependents = sorted((r for r in records if r[1] < independent_age), key=lambda r: (-r[1], r[0]))
    for dependent in dependents:
        available = [i for i in range(group_count) if len(groups[i]) < targets[i]]
        if not available:
            available = list(range(group_count))
        chosen = max(available, key=lambda i: (targets[i] - len(groups[i]), -i))
        groups[chosen].append(dependent)
    return groups


def cohort_counts(groups, independent_age: int):
    return [
        Counter("independent" if age >= independent_age else "dependent" for _, age in group)
        for group in groups
    ]


def age_signature(groups):
    return sorted(tuple(sorted(age for _, age in group)) for group in groups)


def main() -> None:
    # Original failure: five adults occupy the low IDs and four newborns the appended tail.
    legacy_records = [(f"p{i+1}", age) for i, age in enumerate([40, 38, 35, 32, 30, 0, 0, 0, 0])]
    legacy = legacy_partition(legacy_records, 5)
    assert cohort_counts(legacy, 18) == [
        Counter({"independent": 5}),
        Counter({"dependent": 4}),
    ]

    repaired = dependency_partition(legacy_records, 5, 18)
    repaired_counts = cohort_counts(repaired, 18)
    assert len(repaired) == 2
    assert all(group["independent"] >= 1 for group in repaired_counts)
    assert sum(group["dependent"] for group in repaired_counts) == 4

    # One anchor cannot support two autonomous groups; target ceiling is deferred instead.
    low_anchor = [("adult", 30)] + [(f"child-{i}", 10 - i) for i in range(8)]
    deferred = dependency_partition(low_anchor, 5, 18)
    assert len(deferred) == 1
    assert len(deferred[0]) == 9

    # With unique scientific ages, PersonId relabelling cannot change unlabelled age composition.
    ages = [60, 50, 40, 30, 20, 15, 10, 5]
    first = [(f"a-{i}", age) for i, age in enumerate(ages)]
    second = [(f"z-{len(ages)-1-i}", age) for i, age in enumerate(ages)]
    assert age_signature(dependency_partition(first, 4, 18)) == age_signature(
        dependency_partition(second, 4, 18)
    )

    print("legacy:", cohort_counts(legacy, 18))
    print("dependency-aware:", repaired_counts)
    print("one-anchor target deferred: 9 members remain together")
    print("unique-age relabelling composition invariant: yes")


if __name__ == "__main__":
    main()
