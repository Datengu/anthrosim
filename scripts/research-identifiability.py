#!/usr/bin/env python3
"""Fail-closed identifiability/equifinality diagnostics for AnthroSim studies.

The statistical implementation is preserved in ``research-identifiability-legacy.py``.
This front-end adds the AV4-011 scientific contract: parameter and structural
coordinates may authorize an identification claim only after they are bound to an
immutable executed research design.  Real-study CLI use derives that binding from
``anthrosim-research``'s redundant immutable manifest/plan pair.  Repository
synthetic fixtures use an explicit digest-bound test-only binding.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
LEGACY_PATH = ROOT / "research-identifiability-legacy.py"
BINDER_PATH = ROOT / "research-identifiability-bind-design.py"
BINDING_SCHEMA_VERSION = 1
BINDING_TYPE = "anthrosim-identifiability-executed-design"
MANIFEST_SOURCE_KIND = "anthrosim_research_manifest_v1"
SYNTHETIC_SOURCE_KIND = "synthetic_fixture"


def _load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


legacy = _load_module(LEGACY_PATH, "anthrosim_research_identifiability_legacy")
binder = _load_module(BINDER_PATH, "anthrosim_research_identifiability_binding")

IdentifiabilityError = legacy.IdentifiabilityError
SCHEMA_VERSION = legacy.SCHEMA_VERSION
MONTE_CARLO_DIAGNOSTIC_SCHEMA = legacy.MONTE_CARLO_DIAGNOSTIC_SCHEMA
RESULT_TYPE = legacy.RESULT_TYPE
MONTE_CARLO_CATEGORY = legacy.MONTE_CARLO_CATEGORY
EPSILON = legacy.EPSILON


def _canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False
    ).encode("utf-8")


def _canonical_sha256(value: Any) -> str:
    return "sha256:" + hashlib.sha256(_canonical_bytes(value)).hexdigest()


def _load(path: Path) -> dict[str, Any]:
    return legacy._load(path)


def _nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise IdentifiabilityError(f"{role} must be a non-empty string")
    return value


def _effective_structure(point: dict[str, Any]) -> Any:
    return point["structure"] if "structure" in point else "default"


def _raw_design_projection(points: Any) -> list[dict[str, Any]]:
    if not isinstance(points, list) or not points:
        raise IdentifiabilityError("points must be a non-empty array")
    projection: list[dict[str, Any]] = []
    seen: set[str] = set()
    for index, point in enumerate(points):
        if not isinstance(point, dict):
            raise IdentifiabilityError(f"points[{index}] must be an object")
        point_id = point.get("id")
        if not isinstance(point_id, str) or not point_id or point_id in seen:
            raise IdentifiabilityError("point ids must be unique non-empty strings")
        seen.add(point_id)
        parameters = point.get("parameters")
        if not isinstance(parameters, dict):
            raise IdentifiabilityError(f"point {point_id} requires a parameters object")
        projection.append(
            {
                "id": point_id,
                "parameters": copy.deepcopy(parameters),
                "structure": copy.deepcopy(_effective_structure(point)),
            }
        )
    return projection


def synthetic_fixture_binding(points: list[dict[str, Any]], fixture_id: str) -> dict[str, Any]:
    """Return an explicit test-only design binding for a synthetic fixture.

    This helper is intentionally unsuitable for real research execution: the source
    kind is permanently marked ``synthetic_fixture`` and contains no runner identity.
    """

    fixture = _nonempty_string(fixture_id, "synthetic fixture id")
    projection = _raw_design_projection(points)
    return {
        "schemaVersion": BINDING_SCHEMA_VERSION,
        "bindingType": BINDING_TYPE,
        "sourceKind": SYNTHETIC_SOURCE_KIND,
        "sourceIdentity": fixture,
        "pointDigest": _canonical_sha256(projection),
    }


def bind_synthetic_fixture(data: dict[str, Any], fixture_id: str) -> dict[str, Any]:
    """Attach a digest-bound synthetic design declaration to a copied data table."""

    bound = copy.deepcopy(data)
    bound["executedDesignBinding"] = synthetic_fixture_binding(bound.get("points"), fixture_id)
    return bound


def _binding_summary(binding: dict[str, Any], *, valid: bool, errors: list[str]) -> dict[str, Any]:
    points = binding.get("points")
    execution_count = 0
    if isinstance(points, list):
        for point in points:
            if isinstance(point, dict) and isinstance(point.get("executionIds"), list):
                execution_count += len(point["executionIds"])
    return {
        "schemaVersion": BINDING_SCHEMA_VERSION,
        "bindingType": BINDING_TYPE,
        "bindingIdentity": _canonical_sha256(binding),
        "sourceKind": binding.get("sourceKind"),
        "sourceIdentity": binding.get("sourceIdentity"),
        "researchId": binding.get("researchId"),
        "definitionIdentity": binding.get("definitionIdentity"),
        "pointCount": len(points) if isinstance(points, list) else None,
        "executionCount": execution_count if isinstance(points, list) else None,
        "valid": valid,
        "validationErrors": errors,
    }


def _validate_binding_shape(binding: Any) -> dict[str, Any]:
    if not isinstance(binding, dict):
        raise IdentifiabilityError("executedDesignBinding must be an object")
    if binding.get("schemaVersion") != BINDING_SCHEMA_VERSION:
        raise IdentifiabilityError(
            f"executedDesignBinding must use schemaVersion {BINDING_SCHEMA_VERSION}"
        )
    if binding.get("bindingType") != BINDING_TYPE:
        raise IdentifiabilityError(f"executedDesignBinding.bindingType must be {BINDING_TYPE}")
    source_kind = binding.get("sourceKind")
    if source_kind not in {MANIFEST_SOURCE_KIND, SYNTHETIC_SOURCE_KIND}:
        raise IdentifiabilityError("executedDesignBinding has an unsupported sourceKind")
    _nonempty_string(binding.get("sourceIdentity"), "executedDesignBinding.sourceIdentity")
    return binding


def _validate_synthetic_binding(data: dict[str, Any], binding: dict[str, Any]) -> list[str]:
    if set(binding) != {
        "schemaVersion",
        "bindingType",
        "sourceKind",
        "sourceIdentity",
        "pointDigest",
    }:
        raise IdentifiabilityError(
            "synthetic executedDesignBinding must contain only schemaVersion, bindingType, sourceKind, sourceIdentity and pointDigest"
        )
    digest = binding.get("pointDigest")
    if not isinstance(digest, str) or not digest.startswith("sha256:"):
        raise IdentifiabilityError("synthetic executedDesignBinding.pointDigest must be a sha256 identity")
    actual = _canonical_sha256(_raw_design_projection(data.get("points")))
    if actual != digest:
        return ["synthetic_fixture_design_digest_mismatch"]
    return []


def _validate_manifest_binding(data: dict[str, Any], binding: dict[str, Any]) -> list[str]:
    required = {
        "schemaVersion",
        "bindingType",
        "sourceKind",
        "sourceIdentity",
        "researchId",
        "definitionIdentity",
        "source",
        "points",
    }
    if set(binding) != required:
        raise IdentifiabilityError(
            "manifest-derived executedDesignBinding does not have the exact schema emitted by research-identifiability-bind-design.py"
        )
    _nonempty_string(binding.get("researchId"), "executedDesignBinding.researchId")
    _nonempty_string(binding.get("definitionIdentity"), "executedDesignBinding.definitionIdentity")
    if not isinstance(binding.get("source"), dict):
        raise IdentifiabilityError("executedDesignBinding.source must be an object")
    bound_points = binding.get("points")
    raw_points = data.get("points")
    if not isinstance(bound_points, list) or not bound_points:
        raise IdentifiabilityError("manifest-derived executedDesignBinding.points must be non-empty")
    if not isinstance(raw_points, list) or not raw_points:
        raise IdentifiabilityError("points must be a non-empty array")

    errors: list[str] = []
    if len(raw_points) != len(bound_points):
        errors.append("point_count_mismatch")
        return errors
    seen_execution_ids: set[str] = set()
    for index, (raw, bound) in enumerate(zip(raw_points, bound_points, strict=True)):
        if not isinstance(raw, dict) or not isinstance(bound, dict):
            errors.append(f"point_{index}_shape_mismatch")
            continue
        bound_id = bound.get("id")
        if raw.get("id") != bound_id:
            errors.append(f"point_{index}_identity_mismatch")
            continue
        if raw.get("parameters") != bound.get("parameters"):
            errors.append(f"point_{bound_id}_parameter_coordinate_mismatch")
        if _effective_structure(raw) != bound.get("structure"):
            errors.append(f"point_{bound_id}_structure_coordinate_mismatch")
        execution_ids = bound.get("executionIds")
        if (
            not isinstance(execution_ids, list)
            or not execution_ids
            or any(not isinstance(item, str) or not item for item in execution_ids)
            or len(set(execution_ids)) != len(execution_ids)
        ):
            raise IdentifiabilityError(
                f"executedDesignBinding point {bound_id} has invalid executionIds"
            )
        if seen_execution_ids.intersection(execution_ids):
            raise IdentifiabilityError("executedDesignBinding contains duplicate executionIds")
        seen_execution_ids.update(execution_ids)
        raw_execution_ids = raw.get("executionIds")
        if raw_execution_ids != execution_ids:
            errors.append(f"point_{bound_id}_execution_identity_mismatch")
        coordinates = bound.get("coordinates")
        if not isinstance(coordinates, list):
            raise IdentifiabilityError(
                f"executedDesignBinding point {bound_id} coordinates must be an array"
            )
    return errors


def _claimed_parameters(plan: dict[str, Any]) -> list[str]:
    claim = plan.get("claim", {})
    if not isinstance(claim, dict):
        raise IdentifiabilityError("claim must be an object")
    claimed = claim.get("parameterIds", [])
    if not isinstance(claimed, list) or any(not isinstance(item, str) or not item for item in claimed):
        raise IdentifiabilityError("claim.parameterIds must be an array of non-empty strings")
    if len(set(claimed)) != len(claimed):
        raise IdentifiabilityError("claim.parameterIds must not contain duplicates")
    return claimed


def _binding_failure_result(
    plan: dict[str, Any], data: dict[str, Any], binding: dict[str, Any], errors: list[str]
) -> dict[str, Any]:
    claimed = _claimed_parameters(plan)
    point_ids = [
        point.get("id")
        for point in data.get("points", [])
        if isinstance(point, dict) and isinstance(point.get("id"), str)
    ]
    diagnostics = [
        {
            "parameter": parameter,
            "kind": "unbound",
            "identified": False,
            "reason": "parameter_not_bound_to_executed_design",
            "fullRange": None,
            "compatibleRange": None,
            "normalizedCompatibleWidth": None,
            "exploredLevelCount": 0,
        }
        for parameter in claimed
    ]
    return {
        "schemaVersion": SCHEMA_VERSION,
        "resultType": RESULT_TYPE,
        "analysisId": plan.get("analysisId"),
        "researchGate": {
            "requiredFor": "quantitative calibration/parameter inference and competing-hypothesis claims",
            "passes": False,
            "reason": "executed_design_binding_invalid",
            "simulationUncertaintyResolved": False,
            "executedDesignBound": False,
        },
        "executedDesignBinding": _binding_summary(binding, valid=False, errors=errors),
        "uncertaintySeparation": {
            "simulationMonteCarlo": "not evaluated because the executed-design coordinate binding failed",
            "empiricalOrEvidence": "not evaluated because the executed-design coordinate binding failed",
        },
        "monteCarloDiagnosticIdsUsed": [],
        "evidenceRoles": {"calibration": [], "heldOutCorroboration": []},
        "acceptableRegion": {"pointCount": 0, "pointIds": [], "fractionOfDesign": 0.0},
        "unresolvedRegion": {
            "pointCount": len(point_ids),
            "pointIds": point_ids,
            "fractionOfDesign": 1.0 if point_ids else 0.0,
        },
        "compatibleRegion": {"pointCount": 0, "pointIds": [], "fractionOfDesign": 0.0},
        "rejectedPointIds": [],
        "pointClassifications": [],
        "parameterDiagnostics": diagnostics,
        "structuralDiagnostic": {
            "compatibleStructures": [],
            "identified": False,
            "equifinal": False,
        },
        "stagedEvidenceDiagnostics": [],
        "profiles": {},
        "pairwiseInteractionSurfaces": [],
        "discriminatingPredictions": [],
        "equifinality": {
            "present": False,
            "parameterCombinationEquifinality": False,
            "distinctCompatibleParameterCombinationCount": 0,
            "structuralEquifinality": False,
            "nuisanceParameterCompensation": {"present": False, "parameterIds": []},
            "reportingPolicy": "no_identifiability_inference_until_executed_design_binding_is_valid",
        },
    }


def analyse(
    plan: dict[str, Any],
    data: dict[str, Any],
    design_binding: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Analyse a design only after validating its executed-coordinate authority."""

    if not isinstance(plan, dict) or not isinstance(data, dict):
        raise IdentifiabilityError("plan and data roots must be objects")
    embedded = data.get("executedDesignBinding")
    if design_binding is None:
        if embedded is None:
            raise IdentifiabilityError(
                "identifiability data requires executedDesignBinding; real studies must pass --research-root and synthetic fixtures must embed an explicit synthetic binding"
            )
        binding = _validate_binding_shape(embedded)
        if binding["sourceKind"] != SYNTHETIC_SOURCE_KIND:
            raise IdentifiabilityError(
                "manifest-derived executedDesignBinding must be regenerated from --research-root rather than trusted from the analysis table"
            )
        errors = _validate_synthetic_binding(data, binding)
    else:
        binding = _validate_binding_shape(design_binding)
        if binding["sourceKind"] != MANIFEST_SOURCE_KIND:
            raise IdentifiabilityError(
                "an external real-study design binding must be derived from an AnthroSim research manifest"
            )
        if embedded is not None and embedded != binding:
            raise IdentifiabilityError(
                "embedded executedDesignBinding disagrees with the binding derived from --research-root"
            )
        errors = _validate_manifest_binding(data, binding)

    if errors:
        return _binding_failure_result(plan, data, binding, errors)

    legacy_data = copy.deepcopy(data)
    legacy_data.pop("executedDesignBinding", None)
    result = legacy.analyse(plan, legacy_data)
    result["executedDesignBinding"] = _binding_summary(binding, valid=True, errors=[])
    result["researchGate"]["executedDesignBound"] = True
    return result


def analyse_with_research_root(
    plan: dict[str, Any], data: dict[str, Any], research_root: Path
) -> dict[str, Any]:
    try:
        binding = binder.derive_from_root(research_root)
    except binder.BindingError as error:
        raise IdentifiabilityError(f"cannot derive immutable executed-design binding: {error}") from error
    return analyse(plan, data, binding)


def _self_test() -> None:
    # Preserve the complete pre-AV4-011 statistical regression surface first.
    legacy._self_test()

    plan = {
        "schemaVersion": 2,
        "analysisId": "executed-design-binding-self-test",
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": [],
        "claim": {"parameterIds": ["theta"], "structuralHypothesis": False},
        "maxNormalizedAcceptableWidth": 0.0,
    }
    points = [
        {
            "id": "theta-0",
            "parameters": {"theta": 0},
            "structure": "same",
            "outputs": {"score": 0.0},
            "outputEvidence": {"score": {"kind": "deterministic"}},
        },
        {
            "id": "theta-1",
            "parameters": {"theta": 1},
            "structure": "same",
            "outputs": {"score": 1.0},
            "outputEvidence": {"score": {"kind": "deterministic"}},
        },
    ]
    data = bind_synthetic_fixture(
        {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points},
        "identifiability-binding-self-test-v1",
    )
    result = analyse(plan, data)
    assert result["researchGate"]["passes"] is True
    assert result["researchGate"]["executedDesignBound"] is True

    fabricated = copy.deepcopy(data)
    for point in fabricated["points"]:
        point["parameters"]["fabricated_theta"] = 0 if point["id"] == "theta-0" else 1
    fabricated_plan = copy.deepcopy(plan)
    fabricated_plan["claim"] = {
        "parameterIds": ["fabricated_theta"],
        "structuralHypothesis": False,
    }
    rejected = analyse(fabricated_plan, fabricated)
    assert rejected["researchGate"]["passes"] is False
    assert rejected["researchGate"]["reason"] == "executed_design_binding_invalid"
    assert rejected["parameterDiagnostics"][0]["parameter"] == "fabricated_theta"
    assert rejected["parameterDiagnostics"][0]["identified"] is False
    assert rejected["profiles"] == {}
    assert rejected["pairwiseInteractionSurfaces"] == []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", nargs="?", type=Path)
    parser.add_argument("data", nargs="?", type=Path)
    parser.add_argument("output", nargs="?", type=Path)
    parser.add_argument(
        "--research-root",
        type=Path,
        help="anthrosim-research root whose immutable manifest/plan define the executed design; required for real-study use",
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            _self_test()
            print("research-identifiability self-test: ok")
            return 0
        if args.plan is None or args.data is None or args.output is None:
            parser.error("PLAN DATA OUTPUT are required unless --self-test is used")
        plan = _load(args.plan)
        data = _load(args.data)
        if args.research_root is None:
            result = analyse(plan, data)
        else:
            result = analyse_with_research_root(plan, data, args.research_root)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(result["researchGate"], sort_keys=True))
        return 0 if result["researchGate"]["passes"] else 2
    except IdentifiabilityError as error:
        print(f"identifiability error: {error}", file=__import__("sys").stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
