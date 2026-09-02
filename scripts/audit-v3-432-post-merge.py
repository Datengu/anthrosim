#!/usr/bin/env python3
"""Independent post-merge adversary for Audit-v3 AV3-017.

This deliberately does not import or execute the production documentation guard.
It reconstructs the original executable/current-document identity claim directly.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "crates/anthrosim-core/src/config.rs"
SCIENTIFIC = ROOT / "docs/scientific-model.md"
CURRENT = ROOT / "docs/research/household-lifecycle-structural-sensitivity-v2.md"
HISTORICAL = ROOT / "docs/research/household-lifecycle-structural-sensitivity-v1.md"
ODD = ROOT / "docs/research/odd.md"
ODD_D = ROOT / "docs/research/odd-d.md"
TRACE = ROOT / "docs/research/trace.md"
CONST = "DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID"
STALE = "deterministic_size_fission_v1"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def executable_id(config: str) -> str:
    m = re.search(rf"pub const {CONST}: &str =\s*\n?\s*\"([^\"]+)\";", config)
    assert m, "current executable lifecycle constant not found"
    return m.group(1)


def verify(*, current_id: str, scientific: str, contract: str, historical: str, odd: str, odd_d: str, trace: str) -> None:
    assert current_id and current_id != STALE
    assert current_id in scientific, "normative scientific model does not name executable current lifecycle"
    assert "current structural-sensitivity experiments" in scientific
    assert "dependency-aware stress test" in scientific
    assert "independent-age members can anchor daughter groups" in scientific
    assert "preferentially keeps dependents with living parents" in scientific
    assert "fewer groups or defer fission" in scientific
    assert f"historical `{STALE}` treatment is superseded" in scientific
    assert "household-lifecycle-structural-sensitivity-v2.md" in scientific

    assert f"The active treatment is `{current_id}`" in contract
    assert f"supersedes `{STALE}`" in contract
    assert "dependency" in contract.lower()
    assert "Superseded historical contract" in historical
    assert STALE in historical

    # These are living/current surfaces. They can delegate detailed lifecycle semantics,
    # but they cannot independently revive v1 as the current identity.
    for name, content in (("ODD", odd), ("ODD+D", odd_d), ("TRACE", trace)):
        assert STALE not in content, f"{name} revives superseded v1 identity"


def must_reject(label: str, **kwargs) -> None:
    try:
        verify(**kwargs)
    except AssertionError as exc:
        print(f"{label}: rejected ({exc or 'identity/semantic mismatch'})")
        return
    raise AssertionError(f"{label}: unexpectedly accepted")


def main() -> None:
    config = read(CONFIG)
    scientific = read(SCIENTIFIC)
    contract = read(CURRENT)
    historical = read(HISTORICAL)
    odd = read(ODD)
    odd_d = read(ODD_D)
    trace = read(TRACE)
    current_id = executable_id(config)

    base = dict(
        current_id=current_id,
        scientific=scientific,
        contract=contract,
        historical=historical,
        odd=odd,
        odd_d=odd_d,
        trace=trace,
    )
    verify(**base)
    print(
        "AV3-017 post-merge adversary: ok "
        f"(executable={current_id}, normative-doc={current_id}, v1=historical/superseded)"
    )

    # Recreate the frozen defect: the living normative scientific model names the
    # superseded v1 treatment while the executable/current contract remain on v2.
    stale_scientific = scientific.replace(current_id, STALE)
    must_reject("original executable-v2/current-doc-v1 adversary", **{**base, "scientific": stale_scientific})

    # Future executable identity must not drift ahead of the living normative page.
    must_reject(
        "future executable-ID drift adversary",
        **{**base, "current_id": "deterministic_dependency_fission_v999"},
    )

    # Retaining v1 is valid provenance only when its historical/superseded status remains explicit.
    unmarked = scientific.replace(f"historical `{STALE}` treatment is superseded", f"`{STALE}` treatment")
    must_reject("unmarked historical-v1 adversary", **{**base, "scientific": unmarked})


if __name__ == "__main__":
    main()
