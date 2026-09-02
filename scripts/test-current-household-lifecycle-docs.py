#!/usr/bin/env python3
"""Fail closed when current-facing household-lifecycle documentation drifts from executable identity."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "crates/anthrosim-core/src/config.rs"
SCIENTIFIC = ROOT / "docs/scientific-model.md"
ODD = ROOT / "docs/research/odd.md"
ODD_D = ROOT / "docs/research/odd-d.md"
TRACE = ROOT / "docs/research/trace.md"
CURRENT_CONTRACT = ROOT / "docs/research/household-lifecycle-structural-sensitivity-v2.md"
HISTORICAL_CONTRACT = ROOT / "docs/research/household-lifecycle-structural-sensitivity-v1.md"

STALE = "deterministic_size_fission_v1"
CONST = "DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID"


def text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def executable_id(config: str) -> str:
    match = re.search(
        rf"pub const {CONST}: &str =\s*\n?\s*\"([^\"]+)\";",
        config,
    )
    if not match:
        raise AssertionError("could not derive current household-lifecycle ID from executable config")
    return match.group(1)


def main() -> None:
    current_id = executable_id(text(CONFIG))
    scientific = text(SCIENTIFIC)
    odd = text(ODD)
    odd_d = text(ODD_D)
    trace = text(TRACE)
    contract = text(CURRENT_CONTRACT)
    historical = text(HISTORICAL_CONTRACT)

    if current_id not in scientific:
        raise AssertionError(f"scientific-model does not name executable lifecycle {current_id}")
    required_semantics = (
        "independent-age members can anchor daughter groups",
        "preferentially keeps dependents with living parents",
        "defer fission rather than manufacture child-only autonomous units",
    )
    for phrase in required_semantics:
        if phrase not in scientific:
            raise AssertionError(f"scientific-model omits current dependency-aware semantic: {phrase}")
    if f"historical `{STALE}` treatment is superseded" not in scientific:
        raise AssertionError("scientific-model does not mark v1 lifecycle as historical/superseded")
    if "household-lifecycle-structural-sensitivity-v2.md" not in scientific:
        raise AssertionError("scientific-model does not link the current lifecycle contract")

    if current_id not in contract or "defines the current synthetic household-lifecycle" not in contract:
        raise AssertionError("current lifecycle contract does not identify executable current treatment")
    if f"supersedes `{STALE}`" not in contract:
        raise AssertionError("current lifecycle contract does not mark v1 superseded")
    if "Superseded historical contract" not in historical or STALE not in historical:
        raise AssertionError("historical v1 contract is not explicitly retained as superseded provenance")

    # ODD, ODD+D and TRACE are living/current surfaces. They may delegate lifecycle detail to
    # the normative scientific model rather than repeat the model ID, but they must never revive
    # the superseded v1 identity as a current mechanism.
    for name, content in (("ODD", odd), ("ODD+D", odd_d), ("TRACE", trace)):
        if STALE in content:
            raise AssertionError(f"{name} current-facing surface contains superseded lifecycle ID {STALE}")

    print(
        "Current household-lifecycle documentation is synchronized: "
        f"executable={current_id}, scientific-model=current dependency-aware v2, "
        "v1=historical/superseded; ODD/ODD+D/TRACE contain no stale v1 identity."
    )


if __name__ == "__main__":
    main()
