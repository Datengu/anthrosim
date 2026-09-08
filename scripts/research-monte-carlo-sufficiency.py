#!/usr/bin/env python3
"""Monte Carlo sufficiency with producer-valid study-result binding verification."""

from __future__ import annotations

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_legacy = _load(
    "anthrosim_research_monte_carlo_sufficiency_legacy",
    HERE / "research-monte-carlo-sufficiency-legacy.py",
)
_binding = _load(
    "anthrosim_research_study_result_binding",
    HERE / "research-study-result-binding.py",
)
_original_validate_study_binding = _legacy.validate_study_binding
_original_diagnostic = _legacy.diagnostic

# The normal-CLT mean-family methods are asymptotic approximations, not finite-sample confidence
# procedures for arbitrary simulation-output distributions.  A hard sample-size floor prevents the
# machine gate from certifying very small samples where the approximation has no defensible basis.
# Thirty is deliberately an operational guard rather than a coverage theorem; the diagnostic makes
# that limitation machine-visible.  Zero observed variance also fails closed because a finite sample
# cannot establish that the underlying stochastic estimand is deterministic.
NORMAL_CLT_MIN_REPLICATES = 30
_NORMAL_CLT_KINDS = {"mean", "difference_in_means", "paired_mean_difference"}


def validate_study_binding(study_dir: Path, plan, identity: str):
    try:
        raw_binding = _binding.load_json(
            study_dir / "study-result-binding.json", "study result binding"
        )
        _binding.validate_result_binding(raw_binding)
    except _binding.StudyBindingError as error:
        _legacy.fail(str(error))
    return _original_validate_study_binding(study_dir, plan, identity)


def _variance_is_positive(values, *, exact: bool) -> bool:
    _estimate, variance = _legacy.mean_and_variance(values, exact=exact)
    return variance > 0


def _normal_clt_validity(groups, kind: str) -> dict:
    exact = any(group.get("requiresExactArithmetic") is True for group in groups)
    counts = [len(group["values"]) for group in groups]
    reasons: list[str] = []

    if any(count < NORMAL_CLT_MIN_REPLICATES for count in counts):
        reasons.append("replicate_count_below_asymptotic_floor")

    if kind == "mean":
        positive_variance = _variance_is_positive(groups[0]["values"], exact=exact)
    elif kind == "difference_in_means":
        positive_variance = all(
            _variance_is_positive(group["values"], exact=exact) for group in groups
        )
    elif kind == "paired_mean_difference":
        if exact:
            differences = [
                _legacy.exact_rational(left) - _legacy.exact_rational(right)
                for left, right in zip(groups[0]["values"], groups[1]["values"])
            ]
        else:
            differences = [
                left - right
                for left, right in zip(groups[0]["values"], groups[1]["values"])
            ]
        positive_variance = _variance_is_positive(differences, exact=exact)
    else:
        raise AssertionError(kind)

    if not positive_variance:
        reasons.append("zero_observed_variance_cannot_establish_determinism")

    return {
        "validForStopping": not reasons,
        "minimumReplicatesPerGroup": NORMAL_CLT_MIN_REPLICATES,
        "observedReplicatesPerGroup": counts,
        "requiresPositiveObservedVariance": True,
        "reasons": reasons,
        "contract": "guarded_asymptotic_normal_approximation_not_finite_sample_distribution_free_coverage",
    }


def diagnostic(groups, plan):
    precision = _original_diagnostic(groups, plan)
    kind = plan["estimand"]["kind"]
    if kind in _NORMAL_CLT_KINDS:
        validity = _normal_clt_validity(groups, kind)
        precision["normalApproximationValidity"] = validity
        precision["confidenceSemantics"] = (
            "nominal_asymptotic_normal_approximation; not a finite-sample distribution-free coverage guarantee"
        )
        if not validity["validForStopping"]:
            precision["sufficient"] = False
    return precision


_legacy.validate_study_binding = validate_study_binding
_legacy.diagnostic = diagnostic

for _name in dir(_legacy):
    if _name.startswith("__") or _name in {"validate_study_binding", "diagnostic", "main"}:
        continue
    globals()[_name] = getattr(_legacy, _name)


if __name__ == "__main__":
    raise SystemExit(_legacy.main())
