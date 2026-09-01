#!/usr/bin/env python3
"""Independent post-merge adversary for Audit-v3 AV3-011 / issue #419.

This file is test-only evidence and must not be merged to production.
"""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
spec = importlib.util.spec_from_file_location("av3_011_reverify_analyzer", ANALYZER)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def plan(analysis_id: str) -> dict:
    return {
        "schemaVersion": 2,
        "analysisId": analysis_id,
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }


def point(point_id: str, theta, nuisance, score: float = 0.0) -> dict:
    return {
        "id": point_id,
        "parameters": {"theta": theta, "nuisance": nuisance},
        "outputs": {"score": score},
        "outputEvidence": {"score": {"kind": "deterministic"}},
    }


def analyse(case: str, points: list[dict]) -> dict:
    return module.analyse(
        plan(f"av3-011-post-merge-{case}"),
        {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points},
    )


def theta_diagnostic(report: dict) -> dict:
    return next(item for item in report["parameterDiagnostics"] if item["parameter"] == "theta")


def theta_values_from_pairwise(report: dict) -> set:
    surface = next(
        item
        for item in report["pairwiseSurfaces"]
        if set(item["parameters"]) == {"theta", "nuisance"}
    )
    theta_side = surface["parameters"].index("theta")
    key = "left" if theta_side == 0 else "right"
    return {cell[key] for cell in surface["cells"]}


def main() -> None:
    # Frozen AV3-011 adversary: theta is present in multiple compatible design
    # points but never actually varied. That is fixed-by-design, not identified.
    fixed = analyse(
        "fixed-numeric",
        [point("fixed-a", 7, "left"), point("fixed-b", 7, "right")],
    )
    diag = theta_diagnostic(fixed)
    assert diag["fullRange"] == [7.0, 7.0]
    assert diag["compatibleRange"] == [7.0, 7.0]
    assert diag["exploredLevelCount"] == 1
    assert diag["normalizedCompatibleWidth"] is None
    assert diag["reason"] == "insufficient_explored_variation"
    assert diag["identified"] is False
    assert fixed["researchGate"]["passes"] is False
    assert fixed["researchGate"]["reason"] == "declared_claim_not_identified"
    assert fixed["profiles"]["theta"] == [{"value": 7, "compatible": 2, "evaluated": 2}]
    assert theta_values_from_pairwise(fixed) == {7}
    print(
        "fixed_numeric="
        + json.dumps(
            {
                "fullRange": diag["fullRange"],
                "compatibleRange": diag["compatibleRange"],
                "exploredLevelCount": diag["exploredLevelCount"],
                "normalizedCompatibleWidth": diag["normalizedCompatibleWidth"],
                "reason": diag["reason"],
                "identified": diag["identified"],
                "researchGatePasses": fixed["researchGate"]["passes"],
                "profile": fixed["profiles"]["theta"],
                "pairwiseThetaValues": sorted(theta_values_from_pairwise(fixed)),
            },
            sort_keys=True,
        )
    )

    # The same minimum-variation rule must apply to categorical claims.
    fixed_category = analyse(
        "fixed-categorical",
        [point("cat-a", "only-level", 0), point("cat-b", "only-level", 1)],
    )
    cat_diag = theta_diagnostic(fixed_category)
    assert cat_diag["exploredLevelCount"] == 1
    assert cat_diag["fullValues"] == ["only-level"]
    assert cat_diag["compatibleValues"] == ["only-level"]
    assert cat_diag["reason"] == "insufficient_explored_variation"
    assert cat_diag["identified"] is False
    assert fixed_category["researchGate"]["passes"] is False
    print(
        "fixed_categorical="
        + json.dumps(
            {
                "exploredLevelCount": cat_diag["exploredLevelCount"],
                "reason": cat_diag["reason"],
                "identified": cat_diag["identified"],
                "researchGatePasses": fixed_category["researchGate"]["passes"],
            },
            sort_keys=True,
        )
    )

    # A non-finite coordinate cannot be counted as a second numeric level.
    try:
        analyse(
            "non-finite",
            [point("finite", 7.0, 0), point("non-finite", float("nan"), 1)],
        )
    except module.IdentifiabilityError as exc:
        assert "finite number" in str(exc)
        print("non_finite_numeric=fail_closed:" + str(exc))
    else:
        raise AssertionError("non-finite numeric parameter unexpectedly reached identification")

    # Genuine variation can still support identification when the evidence
    # removes all but a sufficiently narrow compatible part of the design.
    varied_narrow = analyse(
        "varied-narrow",
        [point("selected", 0, 0, score=0.0), point("rejected", 10, 0, score=1.0)],
    )
    narrow_diag = theta_diagnostic(varied_narrow)
    assert narrow_diag["fullRange"] == [0.0, 10.0]
    assert narrow_diag["compatibleRange"] == [0.0, 0.0]
    assert narrow_diag["exploredLevelCount"] == 2
    assert narrow_diag["normalizedCompatibleWidth"] == 0.0
    assert narrow_diag["reason"] == "compatible_region_within_threshold"
    assert narrow_diag["identified"] is True
    assert varied_narrow["researchGate"]["passes"] is True
    assert {entry["value"] for entry in varied_narrow["profiles"]["theta"]} == {0, 10}
    assert theta_values_from_pairwise(varied_narrow) == {0, 10}
    print(
        "varied_numeric_identified="
        + json.dumps(
            {
                "fullRange": narrow_diag["fullRange"],
                "compatibleRange": narrow_diag["compatibleRange"],
                "exploredLevelCount": narrow_diag["exploredLevelCount"],
                "normalizedCompatibleWidth": narrow_diag["normalizedCompatibleWidth"],
                "identified": narrow_diag["identified"],
                "researchGatePasses": varied_narrow["researchGate"]["passes"],
            },
            sort_keys=True,
        )
    )

    # Two evaluated levels are necessary, not sufficient: if both remain
    # compatible over the whole explored range, the claim still fails.
    varied_wide = analyse(
        "varied-wide",
        [point("wide-a", 0, 0), point("wide-b", 10, 0)],
    )
    wide_diag = theta_diagnostic(varied_wide)
    assert wide_diag["exploredLevelCount"] == 2
    assert wide_diag["normalizedCompatibleWidth"] == 1.0
    assert wide_diag["reason"] == "compatible_region_too_wide"
    assert wide_diag["identified"] is False
    assert varied_wide["researchGate"]["passes"] is False
    print(
        "varied_numeric_nonidentifying="
        + json.dumps(
            {
                "exploredLevelCount": wide_diag["exploredLevelCount"],
                "normalizedCompatibleWidth": wide_diag["normalizedCompatibleWidth"],
                "reason": wide_diag["reason"],
                "identified": wide_diag["identified"],
                "researchGatePasses": varied_wide["researchGate"]["passes"],
            },
            sort_keys=True,
        )
    )

    varied_category = analyse(
        "varied-categorical",
        [point("cat-selected", "selected", 0), point("cat-rejected", "rejected", 0, score=1.0)],
    )
    varied_cat_diag = theta_diagnostic(varied_category)
    assert varied_cat_diag["exploredLevelCount"] == 2
    assert varied_cat_diag["compatibleValues"] == ["selected"]
    assert varied_cat_diag["reason"] == "single_compatible_value"
    assert varied_cat_diag["identified"] is True
    assert varied_category["researchGate"]["passes"] is True
    print(
        "varied_categorical_identified="
        + json.dumps(
            {
                "exploredLevelCount": varied_cat_diag["exploredLevelCount"],
                "compatibleValues": varied_cat_diag["compatibleValues"],
                "identified": varied_cat_diag["identified"],
                "researchGatePasses": varied_category["researchGate"]["passes"],
            },
            sort_keys=True,
        )
    )

    print("AV3-011 post-merge adversary: PASS")


if __name__ == "__main__":
    main()
