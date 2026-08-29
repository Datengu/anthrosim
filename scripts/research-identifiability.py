#!/usr/bin/env python3
"""Fail-closed identifiability/equifinality diagnostics for AnthroSim studies.

The tool analyses an already-executed uncertainty/sensitivity design. It does not
optimise parameters and deliberately reports acceptable regions instead of a
single best-fit point when the declared calibration evidence is non-identifying.
Held-out corroboration observables are reported only as discriminating
predictions; they are never silently promoted into calibration targets.
"""

from __future__ import annotations

import argparse
import json
import math
import tempfile
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
RESULT_TYPE = "anthrosim-identifiability-diagnostic"


class IdentifiabilityError(Exception):
    pass


def _number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise IdentifiabilityError(f"{label} must be a finite number")
    return float(value)


def _load(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise IdentifiabilityError(f"cannot read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise IdentifiabilityError(f"JSON root must be an object: {path}")
    return value


def _validate(plan: dict[str, Any], data: dict[str, Any]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if plan.get("schemaVersion") != SCHEMA_VERSION or data.get("schemaVersion") != SCHEMA_VERSION:
        raise IdentifiabilityError("unsupported schemaVersion")
    targets = plan.get("calibrationTargets")
    if not isinstance(targets, list) or not targets:
        raise IdentifiabilityError("calibrationTargets must be a non-empty array")
    seen: set[str] = set()
    for index, target in enumerate(targets):
        if not isinstance(target, dict):
            raise IdentifiabilityError(f"calibrationTargets[{index}] must be an object")
        observable = target.get("observable")
        if not isinstance(observable, str) or not observable or observable in seen:
            raise IdentifiabilityError("calibration target observable ids must be unique non-empty strings")
        seen.add(observable)
        _number(target.get("target"), f"target for {observable}")
        tolerance = _number(target.get("tolerance"), f"tolerance for {observable}")
        if tolerance < 0:
            raise IdentifiabilityError(f"tolerance for {observable} must be >= 0")

    corroboration = plan.get("corroborationObservables", [])
    if not isinstance(corroboration, list) or any(not isinstance(x, str) or not x for x in corroboration):
        raise IdentifiabilityError("corroborationObservables must be an array of non-empty strings")
    overlap = seen.intersection(corroboration)
    if overlap:
        raise IdentifiabilityError(
            "calibration targets and held-out corroboration must be disjoint: " + ", ".join(sorted(overlap))
        )

    points = data.get("points")
    if not isinstance(points, list) or not points:
        raise IdentifiabilityError("points must be a non-empty array")
    point_ids: set[str] = set()
    for index, point in enumerate(points):
        if not isinstance(point, dict):
            raise IdentifiabilityError(f"points[{index}] must be an object")
        point_id = point.get("id")
        if not isinstance(point_id, str) or not point_id or point_id in point_ids:
            raise IdentifiabilityError("point ids must be unique non-empty strings")
        point_ids.add(point_id)
        if not isinstance(point.get("parameters"), dict) or not isinstance(point.get("outputs"), dict):
            raise IdentifiabilityError(f"point {point_id} requires parameters and outputs objects")
        for observable in seen.union(corroboration):
            if observable not in point["outputs"]:
                raise IdentifiabilityError(f"point {point_id} is missing output {observable}")
            _number(point["outputs"][observable], f"{point_id}.{observable}")
    return targets, points


def _accepts(point: dict[str, Any], targets: list[dict[str, Any]]) -> bool:
    for target in targets:
        observed = _number(point["outputs"][target["observable"]], target["observable"])
        expected = _number(target["target"], "target")
        tolerance = _number(target["tolerance"], "tolerance")
        if abs(observed - expected) > tolerance + 1e-12:
            return False
    return True


def _parameter_ids(points: list[dict[str, Any]]) -> list[str]:
    common = set(points[0]["parameters"])
    for point in points[1:]:
        common.intersection_update(point["parameters"])
    return sorted(common)


def _numeric_parameter_diagnostic(
    parameter: str,
    points: list[dict[str, Any]],
    accepted: list[dict[str, Any]],
    max_normalized_width: float,
) -> dict[str, Any]:
    all_values = [_number(p["parameters"][parameter], f"parameter {parameter}") for p in points]
    accepted_values = [_number(p["parameters"][parameter], f"parameter {parameter}") for p in accepted]
    full_min, full_max = min(all_values), max(all_values)
    if not accepted_values:
        return {
            "parameter": parameter,
            "kind": "numeric",
            "identified": False,
            "reason": "no_acceptable_points",
            "fullRange": [full_min, full_max],
            "acceptableRange": None,
            "normalizedAcceptableWidth": None,
        }
    accepted_min, accepted_max = min(accepted_values), max(accepted_values)
    denominator = full_max - full_min
    width = 0.0 if denominator == 0 else (accepted_max - accepted_min) / denominator
    return {
        "parameter": parameter,
        "kind": "numeric",
        "identified": width <= max_normalized_width,
        "reason": "acceptable_region_within_threshold" if width <= max_normalized_width else "acceptable_region_too_wide",
        "fullRange": [full_min, full_max],
        "acceptableRange": [accepted_min, accepted_max],
        "normalizedAcceptableWidth": width,
    }


def _categorical_parameter_diagnostic(
    parameter: str, points: list[dict[str, Any]], accepted: list[dict[str, Any]]
) -> dict[str, Any]:
    all_values = sorted({str(p["parameters"][parameter]) for p in points})
    accepted_values = sorted({str(p["parameters"][parameter]) for p in accepted})
    return {
        "parameter": parameter,
        "kind": "categorical",
        "identified": len(accepted_values) == 1,
        "reason": "single_acceptable_value" if len(accepted_values) == 1 else "multiple_acceptable_values",
        "fullValues": all_values,
        "acceptableValues": accepted_values,
    }


def _parameter_diagnostics(
    points: list[dict[str, Any]], accepted: list[dict[str, Any]], max_normalized_width: float
) -> list[dict[str, Any]]:
    diagnostics: list[dict[str, Any]] = []
    for parameter in _parameter_ids(points):
        values = [p["parameters"][parameter] for p in points]
        numeric = all(isinstance(v, (int, float)) and not isinstance(v, bool) for v in values)
        if numeric:
            diagnostics.append(_numeric_parameter_diagnostic(parameter, points, accepted, max_normalized_width))
        else:
            diagnostics.append(_categorical_parameter_diagnostic(parameter, points, accepted))
    return diagnostics


def _profiles(points: list[dict[str, Any]], accepted: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    accepted_ids = {p["id"] for p in accepted}
    result: dict[str, list[dict[str, Any]]] = {}
    for parameter in _parameter_ids(points):
        buckets: dict[str, list[int]] = {}
        for point in points:
            key = json.dumps(point["parameters"][parameter], sort_keys=True, separators=(",", ":"))
            counts = buckets.setdefault(key, [0, 0])
            counts[1] += 1
            if point["id"] in accepted_ids:
                counts[0] += 1
        result[parameter] = [
            {"value": json.loads(key), "acceptable": counts[0], "evaluated": counts[1]}
            for key, counts in sorted(buckets.items())
        ]
    return result


def _pairwise_surfaces(points: list[dict[str, Any]], accepted: list[dict[str, Any]]) -> list[dict[str, Any]]:
    parameters = _parameter_ids(points)
    accepted_ids = {p["id"] for p in accepted}
    surfaces: list[dict[str, Any]] = []
    for left_index, left in enumerate(parameters):
        for right in parameters[left_index + 1 :]:
            cells: dict[tuple[str, str], list[int]] = {}
            for point in points:
                lv = json.dumps(point["parameters"][left], sort_keys=True, separators=(",", ":"))
                rv = json.dumps(point["parameters"][right], sort_keys=True, separators=(",", ":"))
                counts = cells.setdefault((lv, rv), [0, 0])
                counts[1] += 1
                if point["id"] in accepted_ids:
                    counts[0] += 1
            surfaces.append(
                {
                    "parameters": [left, right],
                    "cells": [
                        {
                            "left": json.loads(key[0]),
                            "right": json.loads(key[1]),
                            "acceptable": counts[0],
                            "evaluated": counts[1],
                        }
                        for key, counts in sorted(cells.items())
                    ],
                }
            )
    return surfaces


def _structural_diagnostic(accepted: list[dict[str, Any]]) -> dict[str, Any]:
    structures = sorted({str(p.get("structure", "default")) for p in accepted})
    return {
        "acceptableStructures": structures,
        "identified": len(structures) == 1,
        "equifinal": len(structures) > 1,
    }


def _discriminating_predictions(
    plan: dict[str, Any], accepted: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    observables = plan.get("corroborationObservables", [])
    if not observables:
        return []
    tolerance = _number(plan.get("corroborationDiscriminationTolerance", 0.0), "corroborationDiscriminationTolerance")
    by_structure: dict[str, list[dict[str, Any]]] = {}
    for point in accepted:
        by_structure.setdefault(str(point.get("structure", "default")), []).append(point)
    structures = sorted(by_structure)
    result: list[dict[str, Any]] = []
    for left_index, left in enumerate(structures):
        for right in structures[left_index + 1 :]:
            for observable in observables:
                left_mean = sum(_number(p["outputs"][observable], observable) for p in by_structure[left]) / len(by_structure[left])
                right_mean = sum(_number(p["outputs"][observable], observable) for p in by_structure[right]) / len(by_structure[right])
                delta = right_mean - left_mean
                result.append(
                    {
                        "structures": [left, right],
                        "observable": observable,
                        "leftMean": left_mean,
                        "rightMean": right_mean,
                        "absoluteDifference": abs(delta),
                        "discriminating": abs(delta) > tolerance,
                        "role": "held_out_corroboration_prediction",
                    }
                )
    return result


def analyse(plan: dict[str, Any], data: dict[str, Any]) -> dict[str, Any]:
    targets, points = _validate(plan, data)
    threshold = _number(plan.get("maxNormalizedAcceptableWidth", 0.25), "maxNormalizedAcceptableWidth")
    if threshold < 0 or threshold > 1:
        raise IdentifiabilityError("maxNormalizedAcceptableWidth must be in [0,1]")

    stages: list[dict[str, Any]] = []
    for end in range(1, len(targets) + 1):
        active = targets[:end]
        accepted = [p for p in points if _accepts(p, active)]
        diagnostics = _parameter_diagnostics(points, accepted, threshold)
        stages.append(
            {
                "calibrationObservables": [t["observable"] for t in active],
                "acceptablePointCount": len(accepted),
                "acceptablePointIds": [p["id"] for p in accepted],
                "parameterDiagnostics": diagnostics,
                "structuralDiagnostic": _structural_diagnostic(accepted),
            }
        )

    accepted = [p for p in points if _accepts(p, targets)]
    final_parameters = _parameter_diagnostics(points, accepted, threshold)
    diagnostic_by_id = {d["parameter"]: d for d in final_parameters}
    claim = plan.get("claim", {})
    if not isinstance(claim, dict):
        raise IdentifiabilityError("claim must be an object")
    claimed_parameters = claim.get("parameterIds", [])
    if not isinstance(claimed_parameters, list) or any(not isinstance(x, str) for x in claimed_parameters):
        raise IdentifiabilityError("claim.parameterIds must be an array of strings")
    unknown = [x for x in claimed_parameters if x not in diagnostic_by_id]
    if unknown:
        raise IdentifiabilityError("claimed parameters are not present in every design point: " + ", ".join(unknown))
    structural = _structural_diagnostic(accepted)
    require_structure = bool(claim.get("structuralHypothesis", False))
    parameter_gate = bool(accepted) and all(diagnostic_by_id[x]["identified"] for x in claimed_parameters)
    structure_gate = (not require_structure) or structural["identified"]
    gate_passes = parameter_gate and structure_gate

    return {
        "schemaVersion": SCHEMA_VERSION,
        "resultType": RESULT_TYPE,
        "analysisId": plan.get("analysisId"),
        "researchGate": {
            "requiredFor": "quantitative calibration/parameter inference and competing-hypothesis claims",
            "passes": gate_passes,
            "reason": "declared_claim_identified" if gate_passes else "declared_claim_not_identified",
        },
        "evidenceRoles": {
            "calibration": [t["observable"] for t in targets],
            "heldOutCorroboration": list(plan.get("corroborationObservables", [])),
        },
        "acceptableRegion": {
            "pointCount": len(accepted),
            "pointIds": [p["id"] for p in accepted],
            "fractionOfDesign": len(accepted) / len(points),
        },
        "parameterDiagnostics": final_parameters,
        "structuralDiagnostic": structural,
        "stagedEvidenceDiagnostics": stages,
        "profiles": _profiles(points, accepted),
        "pairwiseInteractionSurfaces": _pairwise_surfaces(points, accepted),
        "discriminatingPredictions": _discriminating_predictions(plan, accepted),
        "equifinality": {
            "present": len(accepted) > 1 and (not parameter_gate or (require_structure and not structure_gate)),
            "reportingPolicy": "report_acceptable_region_not_unique_optimum",
        },
    }


def _self_test() -> None:
    plan = {
        "schemaVersion": 1,
        "analysisId": "self-test",
        "calibrationTargets": [
            {"observable": "ratio", "target": 1.0, "tolerance": 0.0},
            {"observable": "sum", "target": 4.0, "tolerance": 0.0},
        ],
        "corroborationObservables": ["heldout"],
        "claim": {"parameterIds": ["a", "b"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    points = []
    for a in range(1, 5):
        for b in range(1, 5):
            points.append(
                {
                    "id": f"a{a}-b{b}",
                    "parameters": {"a": a, "b": b},
                    "structure": "same",
                    "outputs": {"ratio": a / b, "sum": a + b, "heldout": a - b},
                }
            )
    result = analyse(plan, {"schemaVersion": 1, "points": points})
    first = result["stagedEvidenceDiagnostics"][0]
    second = result["stagedEvidenceDiagnostics"][1]
    assert first["acceptablePointCount"] == 4
    assert not all(x["identified"] for x in first["parameterDiagnostics"])
    assert second["acceptablePointCount"] == 1
    assert result["researchGate"]["passes"]
    assert result["acceptableRegion"]["pointIds"] == ["a2-b2"]

    bad = dict(plan)
    bad["calibrationTargets"] = [{"observable": "ratio", "target": 1.0, "tolerance": 0.0}]
    result = analyse(bad, {"schemaVersion": 1, "points": points})
    assert not result["researchGate"]["passes"]
    assert result["equifinality"]["present"]

    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "result.json"
        path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        assert json.loads(path.read_text(encoding="utf-8"))["resultType"] == RESULT_TYPE


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", nargs="?", type=Path)
    parser.add_argument("data", nargs="?", type=Path)
    parser.add_argument("output", nargs="?", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            _self_test()
            print("research-identifiability self-test: ok")
            return 0
        if args.plan is None or args.data is None or args.output is None:
            parser.error("PLAN DATA OUTPUT are required unless --self-test is used")
        result = analyse(_load(args.plan), _load(args.data))
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(result["researchGate"], sort_keys=True))
        return 0 if result["researchGate"]["passes"] else 2
    except IdentifiabilityError as error:
        print(f"identifiability error: {error}", file=__import__("sys").stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
