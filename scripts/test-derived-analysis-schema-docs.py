#!/usr/bin/env python3
"""Keep current sweep-analysis schema documentation synchronized with source constants."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SWEEP = ROOT / "crates" / "anthrosim-cli" / "src" / "sweep.rs"
DOC = ROOT / "docs" / "experiments-v0.1.md"


def constant(source: str, name: str) -> int:
    match = re.search(rf"const\s+{re.escape(name)}:\s*u32\s*=\s*(\d+)\s*;", source)
    if match is None:
        raise AssertionError(f"could not find {name} in {SWEEP}")
    return int(match.group(1))


def main() -> None:
    source = SWEEP.read_text(encoding="utf-8")
    doc = DOC.read_text(encoding="utf-8")

    run_summary = constant(source, "DERIVED_ANALYSIS_SCHEMA_VERSION")
    point = constant(source, "DERIVED_POINT_ANALYSIS_SCHEMA_VERSION")

    expected = (
        f"The current run-row and top-level analysis-summary wire contracts use "
        f"derived-analysis schema v{run_summary}. The current point-table contract uses schema v{point}."
    )
    if expected not in doc:
        raise AssertionError(
            "docs/experiments-v0.1.md does not state the current derived-analysis schema "
            f"versions from sweep.rs (run/summary v{run_summary}, point v{point})"
        )

    stale = "The run-row and top-level analysis-summary wire contracts remain derived-analysis schema v4. The point-table contract is schema v5"
    if stale in doc:
        raise AssertionError("stale derived-analysis v4/v5 contract remains in current documentation")


if __name__ == "__main__":
    main()
