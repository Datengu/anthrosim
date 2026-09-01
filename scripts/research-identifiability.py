#!/usr/bin/env python3
"""Fail-closed identifiability/equifinality diagnostics for AnthroSim studies.

The tool analyses an already-executed uncertainty/sensitivity design. It does not
optimise parameters and deliberately reports compatible regions instead of a
single best-fit point when the declared calibration evidence is non-identifying.
Simulation Monte Carlo uncertainty is kept separate from observational/evidence
uncertainty and must be resolved before stochastic calibration can identify a
claim.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import tempfile
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 2
MONTE_CARLO_DIAGNOSTIC_SCHEMA = 2
RESULT_TYPE = "anthrosim-identifiability-diagnostic"
MONTE_CARLO_CATEGORY = "process_stochastic_monte_carlo"
EPSILON = 1e-12


class IdentifiabilityError(Exception):
    pass


def _number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise IdentifiabilityError(f"{label} must be a finite number")
    return float(value)


def _structure_id(point: dict[str, Any], *, required: bool) -> str:
    if "structure" not in point:
        if required:
            raise IdentifiabilityError(
                f"point {point.get('id', '<unknown>')} requires a non-empty string structure identifier for a structural claim"
            )
        return "default"
    structure = point["structure"]
    if not isinstance(structure, str) or not structure.strip():
        raise IdentifiabilityError(
            f"point {point.get('id', '<unknown>')} structure must be a non-empty string"
        )
    return structure


def _load(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise IdentifiabilityError(f"cannot read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise IdentifiabilityError(f"JSON root must be an object: {path}")
    return value


def _canonical_sha256(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def _validate_mc_diagnostic(diagnostic_id: str, diagnostic: Any) -> None:
    if not isinstance(diagnostic, dict):
        raise IdentifiabilityError(f"Monte Carlo diagnostic {diagnostic_id} must be an object")
    if _canonical_sha256(diagnostic) != diagnostic_id:
        raise IdentifiabilityError(f"Monte Carlo diagnostic content digest mismatch: {diagnostic_id}")
    if diagnostic.get("schemaVersion") != MONTE_CARLO_DIAGNOSTIC_SCHEMA:
        raise IdentifiabilityError(
            f"Monte Carlo diagnostic {diagnostic_id} must use schemaVersion {MONTE_CARLO_DIAGNOSTIC_SCHEMA}"
        )
    if diagnostic.get("uncertaintyCategory") != MONTE_CARLO_CATEGORY:
        raise IdentifiabilityError(f"Monte Carlo diagnostic {diagnostic_id} has the wrong uncertainty category")
    replicate_count = diagnostic.get("replicateCount")
    seeds = diagnostic.get("seedIdentities")
    if not isinstance(replicate_count, int) or isinstance(replicate_count, bool) or replicate_count < 2:
        raise IdentifiabilityError(f"Monte Carlo diagnostic {diagnostic_id} has invalid replicateCount")
    if (
        not isinstance(seeds, list)
        or len(seeds) != replicate_count
        or any(not isinstance(seed, int) or isinstance(seed, bool) or seed < 0 for seed in seeds)
        or len(set(seeds)) != len(seeds)
    ):
        raise IdentifiabilityError(f"Monte Carlo diagnostic {diagnostic_id} has invalid seed identities")
    if not isinstance(diagnostic.get("precision"), dict):
        raise IdentifiabilityError(f"Monte Carlo diagnostic {diagnostic_id} has no precision object")


def _validate(plan: dict[str, Any], data: dict[str, Any]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, dict[str, Any]]]:
    if plan.get("schemaVersion") != SCHEMA_VERSION or data.get("schemaVersion") != SCHEMA_VERSION:
        raise IdentifiabilityError(f"unsupported schemaVersion; plan and data must both use {SCHEMA_VERSION}")
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

    claim = plan.get("claim", {})
    if not isinstance(claim, dict):
        raise IdentifiabilityError("claim must be an object")
    structural_claim = claim.get("structuralHypothesis", False)
    if not isinstance(structural_claim, bool):
        raise IdentifiabilityError("claim.structuralHypothesis must be a boolean")

    diagnostics = data.get("monteCarloDiagnostics", {})
    if not isinstance(diagnostics, dict):
        raise IdentifiabilityError("monteCarloDiagnostics must be an object")
    for diagnostic_id, diagnostic in diagnostics.items():
        if not isinstance(diagnostic_id, str) or not diagnostic_id.startswith("sha256:"):
            raise IdentifiabilityError("Monte Carlo diagnostic ids must be sha256 content identities")
        _validate_mc_diagnostic(diagnostic_id, diagnostic)

    points = data.get("points")
    if not isinstance(points, list) or not points:
        raise IdentifiabilityError("points must be a non-empty array")
    point_ids: set[str] = set()
    required_observables = seen.union(corroboration)
    for index, point in enumerate(points):
        if not isinstance(point, dict):
            raise IdentifiabilityError(f"points[{index}] must be an object")
        point_id = point.get("id")
        if not isinstance(point_id, str) or not point_id or point_id in point_ids:
            raise IdentifiabilityError("point ids must be unique non-empty strings")
        point_ids.add(point_id)
        _structure_id(point, required=structural_claim)
        if not isinstance(point.get("parameters"), dict) or not isinstance(point.get("outputs"), dict):
            raise IdentifiabilityError(f"point {point_id} requires parameters and outputs objects")
        evidence = point.get("outputEvidence")
        if not isinstance(evidence, dict):
            raise IdentifiabilityError(f"point {point_id} requires outputEvidence")
        for observable in required_observables:
            if observable not in point["outputs"]:
                raise IdentifiabilityError(f"point {point_id} is missing output {observable}")
            observed = _number(point["outputs"][observable], f"{point_id}.{observable}")
            declaration = evidence.get(observable)
            if not isinstance(declaration, dict):
                raise IdentifiabilityError(f"point {point_id} is missing outputEvidence for {observable}")
            kind = declaration.get("kind")
            if kind == "deterministic":
                if set(declaration) != {"kind"}:
                    raise IdentifiabilityError(
                        f"deterministic outputEvidence for {point_id}.{observable} may contain only kind"
                    )
            elif kind == "monte_carlo":
                if set(declaration) != {"kind", "diagnosticId"}:
                    raise IdentifiabilityError(
                        f"Monte Carlo outputEvidence for {point_id}.{observable} requires only kind and diagnosticId"
                    )
                diagnostic_id = declaration.get("diagnosticId")
                if not isinstance(diagnostic_id, str) or diagnostic_id not in diagnostics:
                    raise IdentifiabilityError(f"point {point_id}.{observable} references an unknown Monte Carlo diagnostic")
                precision = diagnostics[diagnostic_id]["precision"]
                estimate = _number(precision.get("estimate"), f"Monte Carlo estimate for {point_id}.{observable}")
                if abs(estimate - observed) > EPSILON:
                    raise IdentifiabilityError(
                        f"point {point_id}.{observable} does not equal the bound Monte Carlo diagnostic estimate"
                    )
            else:
                raise IdentifiabilityError(
                    f"point {point_id}.{observable} outputEvidence.kind must be deterministic or monte_carlo"
                )
    return targets, points, diagnostics


def _mc_evidence(
    point: dict[str, Any],
    observable: str,
    diagnostics: dict[str, dict[str, Any]],
    task_tolerance: float,
) -> dict[str, Any]:
    observed = _number(point["outputs"][observable], f"{point['id']}.{observable}")
    declaration = point["outputEvidence"][observable]
    if declaration["kind"] == "deterministic":
        return {
            "kind": "deterministic",
            "adequate": True,
            "inadequacyReasons": [],
            "intervalLower": observed,
            "intervalUpper": observed,
            "diagnosticId": None,
            "replicateCount": None,
            "seedIdentities": [],
        }

    diagnostic_id = declaration["diagnosticId"]
    diagnostic = diagnostics[diagnostic_id]
    precision = diagnostic["precision"]
    reasons: list[str] = []
    if precision.get("sufficient") is not True:
        reasons.append("upstream_precision_not_sufficient")
    if diagnostic.get("decision") != "sufficient_stop":
        reasons.append("upstream_decision_not_sufficient_stop")

    lower_raw = precision.get("intervalLower")
    upper_raw = precision.get("intervalUpper")
    half_raw = precision.get("halfWidth")
    declared_raw = precision.get("declaredMaxHalfWidth")
    lower = upper = half = declared = None
    if lower_raw is None or upper_raw is None or half_raw is None:
        reasons.append("upstream_interval_unavailable")
    else:
        lower = _number(lower_raw, f"intervalLower for {point['id']}.{observable}")
        upper = _number(upper_raw, f"intervalUpper for {point['id']}.{observable}")
        half = _number(half_raw, f"halfWidth for {point['id']}.{observable}")
        if lower > upper + EPSILON or half < 0:
            raise IdentifiabilityError(f"invalid Monte Carlo interval for {point['id']}.{observable}")

    if declared_raw is None:
        reasons.append("upstream_declared_precision_threshold_missing")
    else:
        declared = _number(declared_raw, f"declaredMaxHalfWidth for {point['id']}.{observable}")
        if declared <= 0:
            raise IdentifiabilityError(f"declaredMaxHalfWidth for {point['id']}.{observable} must be > 0")
        if declared > task_tolerance + EPSILON:
            reasons.append("declared_monte_carlo_precision_too_wide_for_identifiability_task")
    if half is not None and declared is not None and half > declared + EPSILON:
        reasons.append("observed_half_width_exceeds_declared_precision_threshold")

    confidence = _number(precision.get("confidenceLevel"), f"confidenceLevel for {point['id']}.{observable}")
    if not 0 < confidence < 1:
        raise IdentifiabilityError(f"confidenceLevel for {point['id']}.{observable} must be between 0 and 1")
    if diagnostic.get("estimand", {}).get("kind") == "quantile":
        if precision.get("coverageFeasible") is not True:
            reasons.append("quantile_rank_coverage_infeasible")
        achieved = precision.get("achievedCoverage")
        if achieved is None or _number(achieved, "achievedCoverage") + EPSILON < confidence:
            reasons.append("quantile_coverage_below_declared_confidence")

    return {
        "kind": "monte_carlo",
        "adequate": not reasons,
        "inadequacyReasons": reasons,
        "intervalLower": lower,
        "intervalUpper": upper,
        "diagnosticId": diagnostic_id,
        "replicateCount": diagnostic["replicateCount"],
        "seedIdentities": list(diagnostic["seedIdentities"]),
        "confidenceLevel": confidence,
        "precisionMethod": precision.get("precisionMethod"),
        "declaredMaxHalfWidth": declared,
    }


def _classify_target(
    point: dict[str, Any],
    target: dict[str, Any],
    diagnostics: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    observable = target["observable"]
    expected = _number(target["target"], "target")
    tolerance = _number(target["tolerance"], "tolerance")
    evidence = _mc_evidence(point, observable, diagnostics, tolerance)
    band_lower, band_upper = expected - tolerance, expected + tolerance
    lower, upper = evidence["intervalLower"], evidence["intervalUpper"]
    if not evidence["adequate"] or lower is None or upper is None:
        status = "unresolved"
        reason = "simulation_monte_carlo_precision_inadequate"
    elif upper < band_lower - EPSILON or lower > band_upper + EPSILON:
        status = "rejected"
        reason = "uncertainty_interval_outside_calibration_band"
    elif lower >= band_lower - EPSILON and upper <= band_upper + EPSILON:
        status = "acceptable"
        reason = "uncertainty_interval_inside_calibration_band"
    else:
        status = "unresolved"
        reason = "uncertainty_interval_overlaps_calibration_boundary"
    return {
        "observable": observable,
        "status": status,
        "reason": reason,
        "target": expected,
        "tolerance": tolerance,
        "calibrationBand": [band_lower, band_upper],
        "simulationInterval": None if lower is None or upper is None else [lower, upper],
        "simulationUncertainty": evidence,
    }


def _classify_point(
    point: dict[str, Any],
    targets: list[dict[str, Any]],
    diagnostics: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    target_results = [_classify_target(point, target, diagnostics) for target in targets]
    if any(result["status"] == "rejected" for result in target_results):
        status = "rejected"
    elif any(result["status"] == "unresolved" for result in target_results):
        status = "unresolved"
    else:
        status = "acceptable"
    return {"pointId": point["id"], "status": status, "targets": target_results}


def _region(
    points: list[dict[str, Any]],
    targets: list[dict[str, Any]],
    diagnostics: dict[str, dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    classifications = [_classify_point(point, targets, diagnostics) for point in points]
    by_id = {point["id"]: point for point in points}
    acceptable = [by_id[item["pointId"]] for item in classifications if item["status"] == "acceptable"]
    unresolved = [by_id[item["pointId"]] for item in classifications if item["status"] == "unresolved"]
    rejected = [by_id[item["pointId"]] for item in classifications if item["status"] == "rejected"]
    compatible = [point for point in points if point["id"] in {p["id"] for p in acceptable + unresolved}]
    return acceptable, unresolved, rejected, compatible, classifications


def _parameter_ids(points: list[dict[str, Any]]) -> list[str]:
    common = set(points[0]["parameters"])
    for point in points[1:]:
        common.intersection_update(point["parameters"])
    return sorted(common)


def _numeric_parameter_diagnostic(
    parameter: str,
    points: list[dict[str, Any]],
    compatible: list[dict[str, Any]],
    max_normalized_width: float,
) -> dict[str, Any]:
    all_values = [_number(p["parameters"][parameter], f"parameter {parameter}") for p in points]
    compatible_values = [_number(p["parameters"][parameter], f"parameter {parameter}") for p in compatible]
    full_min, full_max = min(all_values), max(all_values)
    explored_level_count = len(set(all_values))
    if not compatible_values:
        return {
            "parameter": parameter,
            "kind": "numeric",
            "identified": False,
            "reason": "no_compatible_points",
            "fullRange": [full_min, full_max],
            "compatibleRange": None,
            "normalizedCompatibleWidth": None,
            "exploredLevelCount": explored_level_count,
        }
    compatible_min, compatible_max = min(compatible_values), max(compatible_values)
    if explored_level_count < 2:
        return {
            "parameter": parameter,
            "kind": "numeric",
            "identified": False,
            "reason": "insufficient_explored_variation",
            "fullRange": [full_min, full_max],
            "compatibleRange": [compatible_min, compatible_max],
            "normalizedCompatibleWidth": None,
            "exploredLevelCount": explored_level_count,
        }
    denominator = full_max - full_min
    width = (compatible_max - compatible_min) / denominator
    return {
        "parameter": parameter,
        "kind": "numeric",
        "identified": width <= max_normalized_width,
        "reason": "compatible_region_within_threshold" if width <= max_normalized_width else "compatible_region_too_wide",
        "fullRange": [full_min, full_max],
        "compatibleRange": [compatible_min, compatible_max],
        "normalizedCompatibleWidth": width,
        "exploredLevelCount": explored_level_count,
    }


def _categorical_parameter_diagnostic(
    parameter: str, points: list[dict[str, Any]], compatible: list[dict[str, Any]]
) -> dict[str, Any]:
    all_values = sorted({str(p["parameters"][parameter]) for p in points})
    compatible_values = sorted({str(p["parameters"][parameter]) for p in compatible})
    explored_level_count = len(all_values)
    if not compatible_values:
        identified = False
        reason = "no_compatible_points"
    elif explored_level_count < 2:
        identified = False
        reason = "insufficient_explored_variation"
    elif len(compatible_values) == 1:
        identified = True
        reason = "single_compatible_value"
    else:
        identified = False
        reason = "multiple_compatible_values"
    return {
        "parameter": parameter,
        "kind": "categorical",
        "identified": identified,
        "reason": reason,
        "fullValues": all_values,
        "compatibleValues": compatible_values,
        "exploredLevelCount": explored_level_count,
    }


def _parameter_diagnostics(
    points: list[dict[str, Any]], compatible: list[dict[str, Any]], max_normalized_width: float
) -> list[dict[str, Any]]:
    diagnostics: list[dict[str, Any]] = []
    for parameter in _parameter_ids(points):
        values = [p["parameters"][parameter] for p in points]
        numeric = all(isinstance(v, (int, float)) and not isinstance(v, bool) for v in values)
        if numeric:
            diagnostics.append(_numeric_parameter_diagnostic(parameter, points, compatible, max_normalized_width))
        else:
            diagnostics.append(_categorical_parameter_diagnostic(parameter, points, compatible))
    return diagnostics


def _profiles(points: list[dict[str, Any]], compatible: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    compatible_ids = {p["id"] for p in compatible}
    result: dict[str, list[dict[str, Any]]] = {}
    for parameter in _parameter_ids(points):
        buckets: dict[str, list[int]] = {}
        for point in points:
            key = json.dumps(point["parameters"][parameter], sort_keys=True, separators=(",", ":"))
            counts = buckets.setdefault(key, [0, 0])
            counts[1] += 1
            if point["id"] in compatible_ids:
                counts[0] += 1
        result[parameter] = [
            {"value": json.loads(key), "compatible": counts[0], "evaluated": counts[1]}
            for key, counts in sorted(buckets.items())
        ]
    return result


def _pairwise_surfaces(points: list[dict[str, Any]], compatible: list[dict[str, Any]]) -> list[dict[str, Any]]:
    parameters = _parameter_ids(points)
    compatible_ids = {p["id"] for p in compatible}
    surfaces: list[dict[str, Any]] = []
    for left_index, left in enumerate(parameters):
        for right in parameters[left_index + 1 :]:
            cells: dict[tuple[str, str], list[int]] = {}
            for point in points:
                lv = json.dumps(point["parameters"][left], sort_keys=True, separators=(",", ":"))
                rv = json.dumps(point["parameters"][right], sort_keys=True, separators=(",", ":"))
                counts = cells.setdefault((lv, rv), [0, 0])
                counts[1] += 1
                if point["id"] in compatible_ids:
                    counts[0] += 1
            surfaces.append(
                {
                    "parameters": [left, right],
                    "cells": [
                        {
                            "left": json.loads(key[0]),
                            "right": json.loads(key[1]),
                            "compatible": counts[0],
                            "evaluated": counts[1],
                        }
                        for key, counts in sorted(cells.items())
                    ],
                }
            )
    return surfaces


def _structural_diagnostic(compatible: list[dict[str, Any]]) -> dict[str, Any]:
    structures = sorted({_structure_id(p, required=False) for p in compatible})
    return {
        "compatibleStructures": structures,
        "identified": len(structures) == 1,
        "equifinal": len(structures) > 1,
    }


def _discriminating_predictions(
    plan: dict[str, Any],
    compatible: list[dict[str, Any]],
    diagnostics: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    observables = plan.get("corroborationObservables", [])
    if not observables:
        return []
    tolerance = _number(plan.get("corroborationDiscriminationTolerance", 0.0), "corroborationDiscriminationTolerance")
    by_structure: dict[str, list[dict[str, Any]]] = {}
    for point in compatible:
        by_structure.setdefault(_structure_id(point, required=False), []).append(point)
    structures = sorted(by_structure)
    result: list[dict[str, Any]] = []
    for left_index, left in enumerate(structures):
        for right in structures[left_index + 1 :]:
            for observable in observables:
                left_evidence = [_mc_evidence(point, observable, diagnostics, tolerance) for point in by_structure[left]]
                right_evidence = [_mc_evidence(point, observable, diagnostics, tolerance) for point in by_structure[right]]
                adequate = all(item["adequate"] for item in left_evidence + right_evidence)
                intervals_available = all(
                    item["intervalLower"] is not None and item["intervalUpper"] is not None
                    for item in left_evidence + right_evidence
                )
                left_mean = sum(_number(p["outputs"][observable], observable) for p in by_structure[left]) / len(by_structure[left])
                right_mean = sum(_number(p["outputs"][observable], observable) for p in by_structure[right]) / len(by_structure[right])
                if intervals_available:
                    left_interval = [
                        sum(float(item["intervalLower"]) for item in left_evidence) / len(left_evidence),
                        sum(float(item["intervalUpper"]) for item in left_evidence) / len(left_evidence),
                    ]
                    right_interval = [
                        sum(float(item["intervalLower"]) for item in right_evidence) / len(right_evidence),
                        sum(float(item["intervalUpper"]) for item in right_evidence) / len(right_evidence),
                    ]
                    if right_interval[0] > left_interval[1]:
                        minimum_gap = right_interval[0] - left_interval[1]
                    elif left_interval[0] > right_interval[1]:
                        minimum_gap = left_interval[0] - right_interval[1]
                    else:
                        minimum_gap = 0.0
                else:
                    left_interval = right_interval = None
                    minimum_gap = 0.0
                discriminating = adequate and intervals_available and minimum_gap > tolerance + EPSILON
                result.append(
                    {
                        "structures": [left, right],
                        "observable": observable,
                        "leftMean": left_mean,
                        "rightMean": right_mean,
                        "leftSimulationIntervalEnvelope": left_interval,
                        "rightSimulationIntervalEnvelope": right_interval,
                        "minimumIntervalSeparation": minimum_gap,
                        "absolutePointEstimateDifference": abs(right_mean - left_mean),
                        "simulationPrecisionAdequate": adequate,
                        "discriminating": discriminating,
                        "role": "held_out_corroboration_prediction",
                    }
                )
    return result


def analyse(plan: dict[str, Any], data: dict[str, Any]) -> dict[str, Any]:
    targets, points, diagnostics = _validate(plan, data)
    threshold = _number(plan.get("maxNormalizedAcceptableWidth", 0.25), "maxNormalizedAcceptableWidth")
    if threshold < 0 or threshold > 1:
        raise IdentifiabilityError("maxNormalizedAcceptableWidth must be in [0,1]")

    stages: list[dict[str, Any]] = []
    for end in range(1, len(targets) + 1):
        active = targets[:end]
        acceptable, unresolved, rejected, compatible, classifications = _region(points, active, diagnostics)
        parameter_diagnostics = _parameter_diagnostics(points, compatible, threshold)
        stages.append(
            {
                "calibrationObservables": [t["observable"] for t in active],
                "acceptablePointCount": len(acceptable),
                "acceptablePointIds": [p["id"] for p in acceptable],
                "unresolvedPointCount": len(unresolved),
                "unresolvedPointIds": [p["id"] for p in unresolved],
                "compatiblePointCount": len(compatible),
                "compatiblePointIds": [p["id"] for p in compatible],
                "rejectedPointCount": len(rejected),
                "simulationUncertaintyResolved": len(unresolved) == 0,
                "pointClassifications": classifications,
                "parameterDiagnostics": parameter_diagnostics,
                "structuralDiagnostic": _structural_diagnostic(compatible),
            }
        )

    acceptable, unresolved, rejected, compatible, classifications = _region(points, targets, diagnostics)
    final_parameters = _parameter_diagnostics(points, compatible, threshold)
    diagnostic_by_id = {item["parameter"]: item for item in final_parameters}
    claim = plan.get("claim", {})
    claimed_parameters = claim.get("parameterIds", [])
    if not isinstance(claimed_parameters, list) or any(not isinstance(x, str) for x in claimed_parameters):
        raise IdentifiabilityError("claim.parameterIds must be an array of strings")
    unknown = [x for x in claimed_parameters if x not in diagnostic_by_id]
    if unknown:
        raise IdentifiabilityError("claimed parameters are not present in every design point: " + ", ".join(unknown))
    structural = _structural_diagnostic(compatible)
    require_structure = claim.get("structuralHypothesis", False)
    uncertainty_gate = len(unresolved) == 0
    parameter_gate = bool(compatible) and all(diagnostic_by_id[x]["identified"] for x in claimed_parameters)
    structure_gate = (not require_structure) or structural["identified"]
    gate_passes = uncertainty_gate and parameter_gate and structure_gate
    if not uncertainty_gate:
        gate_reason = "simulation_monte_carlo_uncertainty_unresolved"
    elif not parameter_gate or not structure_gate:
        gate_reason = "declared_claim_not_identified"
    else:
        gate_reason = "declared_claim_identified"

    used_diagnostics = sorted(
        {
            target_result["simulationUncertainty"]["diagnosticId"]
            for classification in classifications
            for target_result in classification["targets"]
            if target_result["simulationUncertainty"]["diagnosticId"] is not None
        }
    )

    return {
        "schemaVersion": SCHEMA_VERSION,
        "resultType": RESULT_TYPE,
        "analysisId": plan.get("analysisId"),
        "researchGate": {
            "requiredFor": "quantitative calibration/parameter inference and competing-hypothesis claims",
            "passes": gate_passes,
            "reason": gate_reason,
            "simulationUncertaintyResolved": uncertainty_gate,
        },
        "uncertaintySeparation": {
            "simulationMonteCarlo": "bound per output through immutable Monte Carlo diagnostics and used in compatibility decisions",
            "empiricalOrEvidence": "represented separately by the declared calibration target/tolerance and is not estimated by the Monte Carlo diagnostic",
        },
        "monteCarloDiagnosticIdsUsed": used_diagnostics,
        "evidenceRoles": {
            "calibration": [t["observable"] for t in targets],
            "heldOutCorroboration": list(plan.get("corroborationObservables", [])),
        },
        "acceptableRegion": {
            "pointCount": len(acceptable),
            "pointIds": [p["id"] for p in acceptable],
            "fractionOfDesign": len(acceptable) / len(points),
        },
        "unresolvedRegion": {
            "pointCount": len(unresolved),
            "pointIds": [p["id"] for p in unresolved],
            "fractionOfDesign": len(unresolved) / len(points),
        },
        "compatibleRegion": {
            "pointCount": len(compatible),
            "pointIds": [p["id"] for p in compatible],
            "fractionOfDesign": len(compatible) / len(points),
        },
        "rejectedPointIds": [p["id"] for p in rejected],
        "pointClassifications": classifications,
        "parameterDiagnostics": final_parameters,
        "structuralDiagnostic": structural,
        "stagedEvidenceDiagnostics": stages,
        "profiles": _profiles(points, compatible),
        "pairwiseInteractionSurfaces": _pairwise_surfaces(points, compatible),
        "discriminatingPredictions": _discriminating_predictions(plan, compatible, diagnostics),
        "equifinality": {
            "present": len(compatible) > 1 and (not parameter_gate or (require_structure and not structure_gate)),
            "reportingPolicy": "report_compatible_region_not_unique_optimum",
        },
    }


def _deterministic_evidence(outputs: dict[str, Any]) -> dict[str, dict[str, str]]:
    return {observable: {"kind": "deterministic"} for observable in outputs}


def _make_mc_diagnostic(estimate: float, half_width: float, declared: float, seeds: list[int]) -> tuple[str, dict[str, Any]]:
    diagnostic = {
        "schemaVersion": 2,
        "planIdentity": "monte-carlo-precision-plan-v1-self-test",
        "planId": "identifiability-self-test",
        "uncertaintyCategory": MONTE_CARLO_CATEGORY,
        "estimand": {"kind": "mean", "confidenceLevel": 0.95, "maxHalfWidth": declared},
        "designMode": "fixed",
        "batchBoundary": 1,
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
            "sufficient": half_width <= declared,
        },
        "decision": "sufficient_stop" if half_width <= declared else "insufficient_no_predeclared_additional_batch",
        "nextDeclaredBatchSeeds": [],
        "scientificInterpretation": {
            "represents": "Monte Carlo/process stochastic uncertainty",
            "doesNotRepresent": ["empirical uncertainty"],
        },
    }
    return _canonical_sha256(diagnostic), diagnostic


def _self_test() -> None:
    plan = {
        "schemaVersion": 2,
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
            outputs = {"ratio": a / b, "sum": a + b, "heldout": a - b}
            points.append(
                {
                    "id": f"a{a}-b{b}",
                    "parameters": {"a": a, "b": b},
                    "structure": "same",
                    "outputs": outputs,
                    "outputEvidence": _deterministic_evidence(outputs),
                }
            )
    data = {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points}
    result = analyse(plan, data)
    first = result["stagedEvidenceDiagnostics"][0]
    second = result["stagedEvidenceDiagnostics"][1]
    assert first["acceptablePointCount"] == 4
    assert first["unresolvedPointCount"] == 0
    assert not all(item["identified"] for item in first["parameterDiagnostics"])
    assert second["acceptablePointCount"] == 1
    assert result["researchGate"]["passes"]
    assert result["compatibleRegion"]["pointIds"] == ["a2-b2"]

    bad = dict(plan)
    bad["calibrationTargets"] = [{"observable": "ratio", "target": 1.0, "tolerance": 0.0}]
    result = analyse(bad, data)
    assert not result["researchGate"]["passes"]
    assert result["equifinality"]["present"]

    structural_plan = {
        "schemaVersion": 2,
        "analysisId": "structure-id-self-test",
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": [], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }
    typed_structure_data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": [
            {
                "id": "numeric-structure",
                "parameters": {},
                "structure": 1,
                "outputs": {"score": 0.0},
                "outputEvidence": {"score": {"kind": "deterministic"}},
            },
            {
                "id": "string-structure",
                "parameters": {},
                "structure": "1",
                "outputs": {"score": 0.0},
                "outputEvidence": {"score": {"kind": "deterministic"}},
            },
        ],
    }
    try:
        analyse(structural_plan, typed_structure_data)
    except IdentifiabilityError as error:
        assert "structure must be a non-empty string" in str(error)
    else:
        raise AssertionError("typed structure identifiers must not collapse through string coercion")

    missing_structure_data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": [
            {
                "id": "missing-structure",
                "parameters": {},
                "outputs": {"score": 0.0},
                "outputEvidence": {"score": {"kind": "deterministic"}},
            }
        ],
    }
    try:
        analyse(structural_plan, missing_structure_data)
    except IdentifiabilityError as error:
        assert "requires a non-empty string structure identifier" in str(error)
    else:
        raise AssertionError("a structural claim must bind an explicit structure identifier")

    two_structure_data = {
        "schemaVersion": 2,
        "monteCarloDiagnostics": {},
        "points": [
            {
                "id": structure,
                "parameters": {},
                "structure": structure,
                "outputs": {"score": 0.0},
                "outputEvidence": {"score": {"kind": "deterministic"}},
            }
            for structure in ["structure-a", "structure-b"]
        ],
    }
    two_structure_result = analyse(structural_plan, two_structure_data)
    assert two_structure_result["structuralDiagnostic"]["compatibleStructures"] == [
        "structure-a",
        "structure-b",
    ]
    assert two_structure_result["structuralDiagnostic"]["identified"] is False
    assert two_structure_result["structuralDiagnostic"]["equifinal"] is True
    assert two_structure_result["researchGate"]["passes"] is False

    stochastic_plan = {
        "schemaVersion": 2,
        "analysisId": "stochastic-precision-self-test",
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.05}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.0,
    }

    def stochastic_data(half_width: float, declared: float) -> dict[str, Any]:
        diagnostic_a_id, diagnostic_a = _make_mc_diagnostic(0.0, half_width, declared, [101, 102, 103, 104])
        diagnostic_b_id, diagnostic_b = _make_mc_diagnostic(0.10, half_width, declared, [201, 202, 203, 204])
        return {
            "schemaVersion": 2,
            "monteCarloDiagnostics": {diagnostic_a_id: diagnostic_a, diagnostic_b_id: diagnostic_b},
            "points": [
                {
                    "id": "theta-0",
                    "parameters": {"theta": 0},
                    "structure": "same",
                    "outputs": {"score": 0.0},
                    "outputEvidence": {"score": {"kind": "monte_carlo", "diagnosticId": diagnostic_a_id}},
                },
                {
                    "id": "theta-1",
                    "parameters": {"theta": 1},
                    "structure": "same",
                    "outputs": {"score": 0.10},
                    "outputEvidence": {"score": {"kind": "monte_carlo", "diagnosticId": diagnostic_b_id}},
                },
            ],
        }

    low_precision = analyse(stochastic_plan, stochastic_data(0.20, 0.20))
    assert low_precision["researchGate"]["passes"] is False
    assert low_precision["researchGate"]["reason"] == "simulation_monte_carlo_uncertainty_unresolved"
    assert low_precision["unresolvedRegion"]["pointCount"] == 2
    assert low_precision["compatibleRegion"]["pointIds"] == ["theta-0", "theta-1"]

    high_precision = analyse(stochastic_plan, stochastic_data(0.01, 0.02))
    assert high_precision["researchGate"]["passes"] is True
    assert high_precision["unresolvedRegion"]["pointCount"] == 0
    assert high_precision["acceptableRegion"]["pointIds"] == ["theta-0"]
    assert high_precision["compatibleRegion"]["pointIds"] == ["theta-0"]
    assert high_precision["parameterDiagnostics"][0]["identified"] is True

    tampered = stochastic_data(0.01, 0.02)
    diagnostic_id = next(iter(tampered["monteCarloDiagnostics"]))
    tampered["monteCarloDiagnostics"][diagnostic_id]["replicateCount"] = 5
    try:
        analyse(stochastic_plan, tampered)
    except IdentifiabilityError:
        pass
    else:
        raise AssertionError("tampered Monte Carlo provenance must fail closed")

    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "result.json"
        path.write_text(json.dumps(high_precision, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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
