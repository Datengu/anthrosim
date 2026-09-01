#!/usr/bin/env python3
"""Classify which path-dependent AnthroSim PR gates must run."""

from __future__ import annotations

import argparse
import fnmatch
from pathlib import Path

M8_PATTERNS = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "build/**",
    "crates/anthrosim-core/**",
    "crates/anthrosim-cli/**",
    "scripts/aggregate-m8-spatial-benchmark.py",
    "scripts/verify-m8-spatial-benchmark-reference.py",
    "examples/m8-first-evidence-grounded-benchmark/**",
    ".github/workflows/m8-spatial-benchmark.yml",
)

M9_PATTERNS = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "build/**",
    "crates/anthrosim-core/**",
    "crates/anthrosim-cli/**",
    "scripts/aggregate-m9-aggregation-benchmark.py",
    "scripts/verify-m9-aggregation-benchmark-reference.py",
    "examples/m9-controlled-aggregation-benchmark/**",
    "docs/research/m9-controlled-aggregation-benchmark-result.md",
    "docs/research/m9-controlled-aggregation-benchmark-v1.md",
    ".github/workflows/m9-aggregation-benchmark.yml",
)

RUSTSEC_PATTERNS = (
    "Cargo.toml",
    "Cargo.lock",
    "crates/**/Cargo.toml",
    ".github/workflows/dependency-audit.yml",
)

AUDIT_STATUS_PATTERNS = (
    "docs/research/audit-v2/**",
)

SCIENTIFIC_DOCUMENTATION_PATTERNS = (
    "docs/scientific-model.md",
    "docs/research-integrity.md",
    "docs/research-principles.md",
    "docs/research/**",
)

# Changes to the classifier/aggregator can otherwise weaken their own gate policy.
# Force every expensive gate while this enforcement layer itself is under review.
SELF_PROTECTING_PATHS = (
    ".github/workflows/applicable-scientific-security-gates.yml",
    "scripts/classify-applicable-pr-gates.py",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def path_matches(path: str, patterns: tuple[str, ...]) -> bool:
    return any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)


def normalize_paths(paths: list[str]) -> list[str]:
    require(paths, "changed-file set is empty; refusing ambiguous PR gate classification")
    normalized: list[str] = []
    for raw in paths:
        value = raw.strip().replace("\\", "/")
        require(value != "", "changed-file set contains an empty path")
        require(not value.startswith("/"), f"changed path must be repository-relative: {value}")
        require("/../" not in f"/{value}/", f"changed path may not traverse parents: {value}")
        normalized.append(value)
    return normalized


def classify_change_class(paths: list[str]) -> str:
    """Return the conservative PR risk class enforced by path-aware CI.

    Required contexts remain present for every class. Documentation-only classes resolve
    irrelevant executable work through explicit successful/N/A dispositions, while mixed or
    unrecognized path sets fail safe to ``full``.
    """

    if all(path_matches(path, AUDIT_STATUS_PATTERNS) for path in paths):
        return "audit_status_only"
    if all(path_matches(path, SCIENTIFIC_DOCUMENTATION_PATTERNS) for path in paths):
        return "scientific_documentation_only"
    return "full"


def classify(paths: list[str]) -> dict[str, bool | str]:
    normalized = normalize_paths(paths)
    force_all = any(path in SELF_PROTECTING_PATHS for path in normalized)
    return {
        "change_class": "full" if force_all else classify_change_class(normalized),
        "m8_required": force_all or any(path_matches(path, M8_PATTERNS) for path in normalized),
        "m9_required": force_all or any(path_matches(path, M9_PATTERNS) for path in normalized),
        "rustsec_required": force_all
        or any(path_matches(path, RUSTSEC_PATTERNS) for path in normalized),
    }


def write_outputs(path: Path, result: dict[str, bool | str], changed: list[str]) -> None:
    with path.open("a", encoding="utf-8") as handle:
        for name, value in result.items():
            if isinstance(value, bool):
                rendered = "true" if value else "false"
            else:
                rendered = value
            handle.write(f"{name}={rendered}\n")
        handle.write(f"changed_count={len(changed)}\n")


def self_test() -> None:
    gate_fixtures = (
        (["docs/vision.md"], (False, False, False)),
        (["crates/anthrosim-core/src/world.rs"], (True, True, False)),
        (["crates/anthrosim-cli/src/main.rs"], (True, True, False)),
        (
            ["docs/world.rs", "crates/anthrosim-core/src/world.rs"],
            (True, True, False),
        ),
        (["Cargo.lock"], (True, True, True)),
        (["crates/anthrosim-core/Cargo.toml"], (True, True, True)),
        (["examples/m8-first-evidence-grounded-benchmark/landscape.json"], (True, False, False)),
        (["docs/research/m9-controlled-aggregation-benchmark-v1.md"], (False, True, False)),
        ([".github/workflows/dependency-audit.yml"], (False, False, True)),
        ([".github/workflows/applicable-scientific-security-gates.yml"], (True, True, True)),
        (["scripts/classify-applicable-pr-gates.py"], (True, True, True)),
    )
    for paths, expected in gate_fixtures:
        result = classify(paths)
        actual = (
            result["m8_required"],
            result["m9_required"],
            result["rustsec_required"],
        )
        require(actual == expected, f"classification mismatch for {paths}: {actual} != {expected}")

    class_fixtures = (
        (["docs/research/audit-v2/STATUS.md"], "audit_status_only"),
        (
            ["docs/research/audit-v2/STATUS.md", "docs/research/audit-v2/area-i.md"],
            "audit_status_only",
        ),
        (["docs/scientific-model.md"], "scientific_documentation_only"),
        (["docs/research/m9-controlled-aggregation-benchmark-result.md"], "scientific_documentation_only"),
        (
            ["docs/scientific-model.md", "docs/research/audit-v2/STATUS.md"],
            "scientific_documentation_only",
        ),
        (["docs/vision.md"], "full"),
        (["docs/research/audit-v2/STATUS.md", "Cargo.toml"], "full"),
        (["scripts/classify-applicable-pr-gates.py"], "full"),
    )
    for paths, expected in class_fixtures:
        actual = classify(paths)["change_class"]
        require(actual == expected, f"change-class mismatch for {paths}: {actual} != {expected}")

    for bad in ([], [""], ["../Cargo.toml"], ["/Cargo.toml"]):
        try:
            classify(bad)
        except ValueError:
            pass
        else:
            raise ValueError(f"ambiguous/invalid changed-file set was accepted: {bad}")

    print("applicable PR gate classifier self-test passed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--changed-files", type=Path)
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    require(args.changed_files is not None, "--changed-files is required")
    require(args.github_output is not None, "--github-output is required")
    changed = args.changed_files.read_text(encoding="utf-8").splitlines()
    result = classify(changed)
    write_outputs(args.github_output, result, changed)

    print(f"classified {len(changed)} changed file(s)")
    for name, value in result.items():
        if isinstance(value, bool):
            rendered = "required" if value else "not applicable"
        else:
            rendered = value
        print(f"  {name}: {rendered}")


if __name__ == "__main__":
    main()
