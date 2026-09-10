#!/usr/bin/env python3
"""Fail-closed research-facing identifiability facade.

The frozen statistical and executed-coordinate implementation lives in
``research-identifiability-core.py``. This facade adds AV6-012 authoritative output binding for
real ``anthrosim-research`` roots without changing the historical synthetic-fixture surface.
"""

from __future__ import annotations

import argparse
import copy
import importlib.util
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
CORE_PATH = ROOT / "research-identifiability-core.py"
OUTPUT_BINDER_PATH = ROOT / "research-identifiability-output-binding.py"


def _load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


core = _load_module(CORE_PATH, "anthrosim_research_identifiability_core")
output_binder = _load_module(
    OUTPUT_BINDER_PATH, "anthrosim_research_identifiability_output_binding"
)

# Preserve the complete historical public Python surface for tests and downstream tooling. The
# definitions below deliberately override only the real-study analysis entry points.
for _name, _value in vars(core).items():
    if not _name.startswith("__"):
        globals().setdefault(_name, _value)

IdentifiabilityError = core.IdentifiabilityError


def _output_binding_failure_result(
    plan: dict[str, Any],
    data: dict[str, Any],
    binding: dict[str, Any],
    summary: dict[str, Any],
    errors: list[str],
) -> dict[str, Any]:
    """Return a non-identifying result without calculating from unbound outputs."""

    result = core._binding_failure_result(plan, data, binding, errors)
    result["researchGate"].update(
        {
            "reason": "authoritative_output_binding_invalid",
            "executedDesignBound": True,
            "authoritativeOutputsBound": False,
        }
    )
    result["executedDesignBinding"] = core._binding_summary(binding, valid=True, errors=[])
    result["authoritativeOutputBinding"] = summary
    for diagnostic in result["parameterDiagnostics"]:
        diagnostic["kind"] = "unbound_output"
        diagnostic["reason"] = "claim_outputs_not_bound_to_authoritative_executions"
    result["uncertaintySeparation"] = {
        "simulationMonteCarlo": "not evaluated because claim-driving deterministic outputs are not authoritatively bound",
        "empiricalOrEvidence": "not evaluated because claim-driving deterministic outputs are not authoritatively bound",
    }
    result["equifinality"]["reportingPolicy"] = (
        "no_identifiability_inference_until_authoritative_output_binding_is_valid"
    )
    return result


def _legacy_data_after_authoritative_binding(data: dict[str, Any]) -> dict[str, Any]:
    """Remove only metadata already validated by the AV6-012 authority layer.

    The frozen statistical implementation intentionally accepts deterministic declarations only as
    ``{\"kind\":\"deterministic\"}``. Real-study derivation metadata is therefore stripped from a
    private copy *after* the authoritative resolver has validated it. The original downstream table
    and complete derivation remain untouched and are preserved in ``authoritativeOutputBinding``.
    """

    legacy_data = copy.deepcopy(data)
    points = legacy_data.get("points")
    if not isinstance(points, list):
        return legacy_data
    for point in points:
        if not isinstance(point, dict):
            continue
        evidence = point.get("outputEvidence")
        if not isinstance(evidence, dict):
            continue
        for observable, declaration in list(evidence.items()):
            if (
                isinstance(declaration, dict)
                and declaration.get("kind") == "deterministic"
                and "derivation" in declaration
            ):
                evidence[observable] = {"kind": "deterministic"}
    return legacy_data


def analyse(
    plan: dict[str, Any],
    data: dict[str, Any],
    design_binding: dict[str, Any] | None = None,
    *,
    authoritative_output_binding: dict[str, Any] | None = None,
    authoritative_output_errors: list[str] | None = None,
) -> dict[str, Any]:
    """Analyse only after both coordinate and, for real studies, output authority are valid."""

    if design_binding is not None and design_binding.get("sourceKind") == core.MANIFEST_SOURCE_KIND:
        if authoritative_output_binding is None or authoritative_output_errors is None:
            raise IdentifiabilityError(
                "real-study identifiability requires authoritative output binding derived from --research-root"
            )
        if authoritative_output_errors:
            return _output_binding_failure_result(
                plan,
                data,
                design_binding,
                authoritative_output_binding,
                authoritative_output_errors,
            )

    core_data = (
        _legacy_data_after_authoritative_binding(data)
        if authoritative_output_binding is not None
        else data
    )
    result = core.analyse(plan, core_data, design_binding)
    if authoritative_output_binding is not None:
        result["authoritativeOutputBinding"] = authoritative_output_binding
        result["researchGate"]["authoritativeOutputsBound"] = True
    return result


def analyse_with_research_root(
    plan: dict[str, Any], data: dict[str, Any], research_root: Path
) -> dict[str, Any]:
    try:
        binding = core.binder.derive_from_root(research_root)
    except core.binder.BindingError as error:
        raise IdentifiabilityError(
            f"cannot derive immutable executed-design binding: {error}"
        ) from error

    # Coordinate mismatches are reported using the existing AV4-011 fail-closed result before any
    # output artifact is resolved. This keeps coordinate authority and result authority distinct.
    coordinate_errors = core._validate_manifest_binding(data, binding)
    if coordinate_errors:
        return core._binding_failure_result(plan, data, binding, coordinate_errors)

    try:
        output_summary, output_errors = output_binder.validate_real_study_outputs(
            plan, data, binding, research_root
        )
    except output_binder.OutputBindingError as error:
        raise IdentifiabilityError(
            f"cannot bind identifiability outputs to authoritative executions: {error}"
        ) from error

    return analyse(
        plan,
        data,
        binding,
        authoritative_output_binding=output_summary,
        authoritative_output_errors=output_errors,
    )


def _self_test() -> None:
    # The core self-test is entirely synthetic and intentionally retains the historical fixture
    # contract. Real archive/output binding is covered by the dedicated runner regression.
    core._self_test()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", nargs="?", type=Path)
    parser.add_argument("data", nargs="?", type=Path)
    parser.add_argument("output", nargs="?", type=Path)
    parser.add_argument(
        "--research-root",
        type=Path,
        help="anthrosim-research root whose immutable design and completed run artifacts authorize real-study inference",
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
        plan = core._load(args.plan)
        data = core._load(args.data)
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
