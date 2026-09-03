#!/usr/bin/env python3
"""Audit-v4 Area J fresh decision matrix for identifiability/equifinality.

This deliberately recomputes expected scientific classifications from simple,
hand-checkable constructions rather than relying on the permanent regression
suite. AV4-011's missing executed-coordinate binding is tested separately.
"""

from __future__ import annotations

import hashlib
import importlib.util
import itertools
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
ANALYZER_PATH = ROOT / "scripts" / "research-identifiability.py"


def load_analyzer():
    spec = importlib.util.spec_from_file_location("research_identifiability", ANALYZER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load research-identifiability.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def deterministic_point(point_id: str, parameters: dict[str, Any], score: float, *, structure: str = "same", heldout: float | None = None) -> dict[str, Any]:
    outputs: dict[str, Any] = {"score": score}
    if heldout is not None:
        outputs["heldout"] = heldout
    return {
        "id": point_id,
        "parameters": parameters,
        "structure": structure,
        "outputs": outputs,
        "outputEvidence": {name: {"kind": "deterministic"} for name in outputs},
    }


def canonical_sha256(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def mc_diagnostic(estimate: float, half_width: float, declared: float, seeds: list[int]) -> tuple[str, dict[str, Any]]:
    diagnostic = {
        "schemaVersion": 2,
        "planIdentity": "audit-v4-area-j-decision-matrix",
        "planId": "audit-v4-area-j-decision-matrix",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {"kind": "mean", "confidenceLevel": 0.95, "maxHalfWidth": declared},
        "designMode": "fixed",
        "batchBoundary": len(seeds),
        "replicateCount": len(seeds),
        "seedIdentities": seeds,
        "groupIds": ["output"],
        "precision": {
            "estimate": estimate,
            "intervalLower": estimate - half_width,
            "intervalUpper": estimate + half_width,
            "halfWidth": half_width,
            "confidenceLevel": 0.95,
            "precisionMethod": "normal_clt_mean_se",
            "declaredMaxHalfWidth": declared,
            "sufficient": True,
        },
        "decision": "sufficient_stop",
        "nextDeclaredBatchSeeds": [],
        "scientificInterpretation": {
            "represents": "Monte Carlo/process stochastic uncertainty",
            "doesNotRepresent": ["empirical uncertainty"],
        },
    }
    return canonical_sha256(diagnostic), diagnostic


def plan(*, target: float = 0.0, tolerance: float = 0.0, parameters: list[str] | None = None, structural: bool = False, heldout: bool = False, heldout_tolerance: float = 0.0, width: float = 0.25) -> dict[str, Any]:
    return {
        "schemaVersion": 2,
        "analysisId": "audit-v4-area-j-decision-matrix",
        "calibrationTargets": [{"observable": "score", "target": target, "tolerance": tolerance}],
        "corroborationObservables": ["heldout"] if heldout else [],
        "corroborationDiscriminationTolerance": heldout_tolerance,
        "claim": {"parameterIds": parameters or [], "structuralHypothesis": structural},
        "maxNormalizedAcceptableWidth": width,
    }


def data(points: list[dict[str, Any]], diagnostics: dict[str, dict[str, Any]] | None = None) -> dict[str, Any]:
    return {"schemaVersion": 2, "monteCarloDiagnostics": diagnostics or {}, "points": points}


def main() -> None:
    analyzer = load_analyzer()
    checks = 0

    # 1) Exact deterministic calibration boundaries remain compatible, while a
    # broad compatible numeric range correctly fails practical identification.
    boundary_points = [
        deterministic_point("t0", {"theta": 0}, -1.0),
        deterministic_point("t1", {"theta": 1}, 0.0),
        deterministic_point("t2", {"theta": 2}, 1.0),
        deterministic_point("t3", {"theta": 3}, 2.0),
    ]
    broad = analyzer.analyse(plan(tolerance=1.0, parameters=["theta"], width=0.34), data(boundary_points))
    statuses = {item["pointId"]: item["status"] for item in broad["pointClassifications"]}
    assert statuses == {"t0": "acceptable", "t1": "acceptable", "t2": "acceptable", "t3": "rejected"}; checks += 1
    theta = broad["parameterDiagnostics"][0]
    assert theta["fullRange"] == [0.0, 3.0] and theta["compatibleRange"] == [0.0, 2.0]; checks += 1
    assert abs(theta["normalizedCompatibleWidth"] - (2.0 / 3.0)) < 1e-12; checks += 1
    assert theta["identified"] is False and broad["researchGate"]["passes"] is False; checks += 1

    # 2) With a genuinely varied design and one exact compatible level, the
    # parameter can pass at zero compatible width (distinct from fixed-by-design).
    narrow = analyzer.analyse(plan(tolerance=0.0, parameters=["theta"], width=0.0), data(boundary_points))
    assert narrow["compatibleRegion"]["pointIds"] == ["t1"]; checks += 1
    assert narrow["parameterDiagnostics"][0]["exploredLevelCount"] == 4; checks += 1
    assert narrow["parameterDiagnostics"][0]["identified"] is True; checks += 1
    assert narrow["researchGate"] == {"requiredFor": "quantitative calibration/parameter inference and competing-hypothesis claims", "passes": True, "reason": "declared_claim_identified", "simulationUncertaintyResolved": True}; checks += 1

    # 3) Monte Carlo intervals: wholly inside -> acceptable, boundary-overlap ->
    # unresolved and still compatible, wholly outside -> rejected.
    d0_id, d0 = mc_diagnostic(0.0, 0.2, 0.2, [101, 102, 103, 104])
    d1_id, d1 = mc_diagnostic(1.2, 0.3, 0.3, [201, 202, 203, 204])
    d2_id, d2 = mc_diagnostic(2.0, 0.2, 0.2, [301, 302, 303, 304])
    mc_points = []
    for point_id, theta_value, estimate, diagnostic_id in [
        ("m0", 0, 0.0, d0_id),
        ("m1", 1, 1.2, d1_id),
        ("m2", 2, 2.0, d2_id),
    ]:
        mc_points.append({
            "id": point_id,
            "parameters": {"theta": theta_value},
            "structure": "same",
            "outputs": {"score": estimate},
            "outputEvidence": {"score": {"kind": "monte_carlo", "diagnosticId": diagnostic_id}},
        })
    mc_result = analyzer.analyse(
        plan(tolerance=1.0, parameters=["theta"], width=1.0),
        data(mc_points, {d0_id: d0, d1_id: d1, d2_id: d2}),
    )
    mc_status = {item["pointId"]: item["status"] for item in mc_result["pointClassifications"]}
    assert mc_status == {"m0": "acceptable", "m1": "unresolved", "m2": "rejected"}; checks += 1
    assert set(mc_result["compatibleRegion"]["pointIds"]) == {"m0", "m1"}; checks += 1
    assert mc_result["unresolvedRegion"]["pointIds"] == ["m1"]; checks += 1
    assert mc_result["researchGate"]["passes"] is False and mc_result["researchGate"]["reason"] == "simulation_monte_carlo_uncertainty_unresolved"; checks += 1

    # 4) Structural equifinality uses the full compatible parameter envelope for
    # held-out discrimination, not structure means.
    overlap_points = [
        deterministic_point("a0", {}, 0.0, structure="A", heldout=0.0),
        deterministic_point("a1", {}, 0.0, structure="A", heldout=10.0),
        deterministic_point("b0", {}, 0.0, structure="B", heldout=9.0),
        deterministic_point("b1", {}, 0.0, structure="B", heldout=11.0),
    ]
    overlap_result = analyzer.analyse(plan(structural=True, heldout=True, heldout_tolerance=1.0), data(overlap_points))
    assert overlap_result["structuralDiagnostic"] == {"compatibleStructures": ["A", "B"], "identified": False, "equifinal": True}; checks += 1
    assert overlap_result["researchGate"]["passes"] is False; checks += 1
    pred = overlap_result["discriminatingPredictions"][0]
    assert pred["leftSimulationIntervalEnvelope"] == [0.0, 10.0] and pred["rightSimulationIntervalEnvelope"] == [9.0, 11.0]; checks += 1
    assert pred["minimumIntervalSeparation"] == 0.0 and pred["discriminating"] is False; checks += 1

    separated_points = [
        deterministic_point("a0", {}, 0.0, structure="A", heldout=0.0),
        deterministic_point("a1", {}, 0.0, structure="A", heldout=10.0),
        deterministic_point("b0", {}, 0.0, structure="B", heldout=20.0),
        deterministic_point("b1", {}, 0.0, structure="B", heldout=21.0),
    ]
    separated = analyzer.analyse(plan(structural=True, heldout=True, heldout_tolerance=1.0), data(separated_points))
    separated_pred = separated["discriminatingPredictions"][0]
    assert separated_pred["minimumIntervalSeparation"] == 10.0 and separated_pred["discriminating"] is True; checks += 1

    # 5) A narrow claimed parameter must not erase compensating nuisance
    # variation/equifinality from the top-level interpretation.
    compensation_points = [
        deterministic_point("x0-z0", {"x": 0, "z": 0}, 0.0),
        deterministic_point("x0-z1", {"x": 0, "z": 1}, 0.0),
        deterministic_point("x1-z0", {"x": 1, "z": 0}, 10.0),
        deterministic_point("x1-z1", {"x": 1, "z": 1}, 10.0),
    ]
    compensation = analyzer.analyse(plan(parameters=["x"], width=0.0), data(compensation_points))
    diag = {item["parameter"]: item for item in compensation["parameterDiagnostics"]}
    assert diag["x"]["identified"] is True and diag["z"]["identified"] is False; checks += 1
    assert compensation["researchGate"]["passes"] is True; checks += 1
    assert compensation["equifinality"]["present"] is True and compensation["equifinality"]["distinctCompatibleParameterCombinationCount"] == 2; checks += 1
    assert compensation["equifinality"]["nuisanceParameterCompensation"] == {"present": True, "parameterIds": ["z"]}; checks += 1

    # 6) Categorical practical identification requires explored alternatives and
    # exactly one compatible category.
    categorical_points = [
        deterministic_point("ca", {"category": "a"}, 1.0),
        deterministic_point("cb", {"category": "b"}, 0.0),
        deterministic_point("cc", {"category": "c"}, 1.0),
    ]
    categorical = analyzer.analyse(plan(parameters=["category"], width=0.0), data(categorical_points))
    cat_diag = categorical["parameterDiagnostics"][0]
    assert cat_diag["kind"] == "categorical" and cat_diag["exploredLevelCount"] == 3; checks += 1
    assert cat_diag["compatibleValues"] == ["b"] and cat_diag["identified"] is True; checks += 1
    assert categorical["researchGate"]["passes"] is True; checks += 1

    # 7) Scientific decisions must be invariant to row ordering. All 24
    # permutations of the compensation design retain the same compatible set,
    # parameter decisions and nuisance-compensation conclusion.
    reference_diag = {item["parameter"]: (item["identified"], item["reason"]) for item in compensation["parameterDiagnostics"]}
    for ordering in itertools.permutations(compensation_points):
        permuted = analyzer.analyse(plan(parameters=["x"], width=0.0), data(list(ordering)))
        assert permuted["researchGate"]["passes"] is True; checks += 1
        assert set(permuted["compatibleRegion"]["pointIds"]) == {"x0-z0", "x0-z1"}; checks += 1
        actual_diag = {item["parameter"]: (item["identified"], item["reason"]) for item in permuted["parameterDiagnostics"]}
        assert actual_diag == reference_diag; checks += 1
        assert permuted["equifinality"]["nuisanceParameterCompensation"] == {"present": True, "parameterIds": ["z"]}; checks += 1

    # 8) Evidence-role firewall remains fail-closed.
    overlapping_roles = plan(heldout=True)
    overlapping_roles["corroborationObservables"] = ["score"]
    try:
        analyzer.analyse(overlapping_roles, data([deterministic_point("only", {}, 0.0)]))
    except analyzer.IdentifiabilityError as error:
        assert "must be disjoint" in str(error); checks += 1
    else:
        raise AssertionError("calibration/held-out observable overlap must fail closed")

    # 9) Duplicate point identities remain fail-closed.
    duplicate_points = [
        deterministic_point("duplicate", {"x": 0}, 0.0),
        deterministic_point("duplicate", {"x": 1}, 1.0),
    ]
    try:
        analyzer.analyse(plan(parameters=["x"]), data(duplicate_points))
    except analyzer.IdentifiabilityError as error:
        assert "point ids must be unique" in str(error); checks += 1
    else:
        raise AssertionError("duplicate point ids must fail closed")

    print(f"audit_v4_area_j_decision_matrix_checks={checks}")
    print("audit_v4_area_j_decision_matrix_status=pass")


if __name__ == "__main__":
    main()
