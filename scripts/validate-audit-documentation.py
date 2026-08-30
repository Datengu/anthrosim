#!/usr/bin/env python3
"""Lightweight fail-closed checks for documentation-only AnthroSim pull requests."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STATUS = ROOT / "docs/research/audit-v2/STATUS.md"
PROVENANCE = ROOT / "crates/anthrosim-core/src/provenance.rs"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def current_model_semantics() -> str:
    source = PROVENANCE.read_text(encoding="utf-8")
    match = re.search(r'pub const MODEL_SEMANTICS_ID: &str = "([^"]+)";', source)
    require(match is not None, "MODEL_SEMANTICS_ID could not be parsed from provenance.rs")
    return match.group(1)


def validate_status() -> None:
    text = STATUS.read_text(encoding="utf-8")
    require(text.startswith("# AnthroSim scientific audit v2 — status ledger\n"), "unexpected audit status title")
    require("## Current baseline and state" in text, "audit status lacks current baseline section")
    require("## Coverage matrix" in text, "audit status lacks coverage matrix")
    require("## Finding register" in text, "audit status lacks finding register")
    require("## Remaining closure work" in text, "audit status lacks closure-work section")

    semantics = current_model_semantics()
    require(
        f"| Current model semantics | `{semantics}` |" in text,
        f"audit status current model semantics does not match executable {semantics}",
    )

    coverage_ids = re.findall(r"^\| ([A-N]) \|", text, flags=re.MULTILINE)
    require(
        coverage_ids == list("ABCDEFGHIJKLMN"),
        f"coverage matrix must contain A-N exactly once in order; found {coverage_ids}",
    )

    finding_ids = re.findall(r"^\| (AV2-\d{3}) —", text, flags=re.MULTILINE)
    require(finding_ids, "audit status contains no AV2 finding rows")
    require(len(finding_ids) == len(set(finding_ids)), "audit status contains duplicate finding identifiers")
    expected = [f"AV2-{index:03d}" for index in range(1, len(finding_ids) + 1)]
    require(
        finding_ids == expected,
        f"audit finding identifiers must remain contiguous and ordered; found {finding_ids}",
    )

    # Protect obvious handoff corruption while keeping the validator independent of GitHub network
    # availability. Commit identifiers written as full hexadecimal SHAs must remain 40 characters.
    for token in re.findall(r"`([0-9a-f]{32,64})`", text):
        require(len(token) == 40, f"malformed full commit SHA in audit status: {token}")


def validate_markdown_bytes() -> None:
    for path in sorted((ROOT / "docs").rglob("*.md")):
        data = path.read_bytes()
        require(b"\x00" not in data, f"NUL byte in Markdown document: {path.relative_to(ROOT)}")
        require(data.endswith(b"\n"), f"Markdown document lacks final newline: {path.relative_to(ROOT)}")


def main() -> None:
    validate_status()
    validate_markdown_bytes()
    print("audit/documentation lightweight validation passed")


if __name__ == "__main__":
    main()
