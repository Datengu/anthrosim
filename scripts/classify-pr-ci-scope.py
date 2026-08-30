#!/usr/bin/env python3
"""Classify AnthroSim pull requests for conservative path-aware CI."""

from __future__ import annotations

import argparse
import fnmatch
from pathlib import Path

AUDIT_STATUS_PATTERNS = (
    "docs/research/audit-v2/*.md",
    "docs/research/audit-v2/**/*.md",
)

SCIENTIFIC_DOC_PATTERNS = (
    "docs/*.md",
    "docs/**/*.md",
)

VALID_SCOPES = ("audit_status", "scientific_docs", "full")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def normalize(paths: list[str]) -> list[str]:
    require(paths, "changed-file set is empty; refusing ambiguous CI-scope classification")
    normalized: list[str] = []
    for raw in paths:
        value = raw.strip().replace("\\", "/")
        require(value != "", "changed-file set contains an empty path")
        require(not value.startswith("/"), f"changed path must be repository-relative: {value}")
        require("/../" not in f"/{value}/", f"changed path may not traverse parents: {value}")
        normalized.append(value)
    return normalized


def all_match(paths: list[str], patterns: tuple[str, ...]) -> bool:
    return all(any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns) for path in paths)


def classify(paths: list[str]) -> str:
    normalized = normalize(paths)
    if all_match(normalized, AUDIT_STATUS_PATTERNS):
        return "audit_status"
    if all_match(normalized, SCIENTIFIC_DOC_PATTERNS):
        return "scientific_docs"
    return "full"


def write_outputs(path: Path, scope: str, changed: list[str]) -> None:
    require(scope in VALID_SCOPES, f"invalid CI scope {scope}")
    with path.open("a", encoding="utf-8") as handle:
        handle.write(f"scope={scope}\n")
        handle.write(f"full_required={'true' if scope == 'full' else 'false'}\n")
        handle.write(f"documentation_required={'false' if scope == 'full' else 'true'}\n")
        handle.write(f"changed_count={len(changed)}\n")


def self_test() -> None:
    fixtures = (
        (["docs/research/audit-v2/STATUS.md"], "audit_status"),
        (["docs/research/audit-v2/area-i-2026-08-29.md"], "audit_status"),
        (
            [
                "docs/research/audit-v2/STATUS.md",
                "docs/research/audit-v2/area-l-2026-08-29.md",
            ],
            "audit_status",
        ),
        (["docs/scientific-model.md"], "scientific_docs"),
        (["docs/research/odd.md", "docs/research/odd-d.md"], "scientific_docs"),
        (
            ["docs/research/audit-v2/STATUS.md", "docs/scientific-model.md"],
            "scientific_docs",
        ),
        (["crates/anthrosim-core/src/world.rs"], "full"),
        (["scripts/test-current-model-semantics-docs.py"], "full"),
        (["experiments/v0.1-resource-variability.json"], "full"),
        (["research/general-demography-baseline-v1/confirmatory-result.json"], "full"),
        ([".github/workflows/ci.yml"], "full"),
        (["docs/schema.json"], "full"),
        (["README.md"], "full"),
        (["docs/scientific-model.md", "crates/anthrosim-core/src/world.rs"], "full"),
        # Rename-aware callers pass both filename and previous_filename. Moving executable
        # material under docs must therefore remain full even if the new path looks harmless.
        (["docs/research/world.md", "crates/anthrosim-core/src/world.rs"], "full"),
    )
    for paths, expected in fixtures:
        actual = classify(paths)
        require(actual == expected, f"classification mismatch for {paths}: {actual} != {expected}")

    for bad in ([], [""], ["../Cargo.toml"], ["/Cargo.toml"]):
        try:
            classify(bad)
        except ValueError:
            pass
        else:
            raise ValueError(f"ambiguous/invalid changed-file set was accepted: {bad}")

    print("PR CI-scope classifier self-test passed")


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
    scope = classify(changed)
    write_outputs(args.github_output, scope, changed)

    print(f"classified {len(changed)} changed file(s): {scope}")


if __name__ == "__main__":
    main()
