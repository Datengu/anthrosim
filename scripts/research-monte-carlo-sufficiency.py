#!/usr/bin/env python3
"""Predeclared Monte Carlo precision / replicate-sufficiency gate for AnthroSim studies.

This is analysis-layer tooling. It never changes simulation state, RNG streams, or model semantics.
"""

from __future__ import annotations

import argparse
from bisect import bisect_left
import hashlib
import json
import math
import sys
from pathlib import Path
from statistics import NormalDist
from typing import Any

PLAN_SCHEMA = 1
SAMPLE_SCHEMA = 1
DIAGNOSTIC_SCHEMA = 2
PLAN_PREFIX = "monte-carlo-precision-plan-v1:"
UNCERTAINTY_CATEGORY = "process_stochastic_monte_carlo"
SUPPORTED_KINDS = {
    "mean",
    "difference_in_means",
    "probability",
    "quantile",
    "paired_mean_difference",
}


def fail(message: str) -> "NoReturn":
    raise ValueError(message)


def read_json(path: Path) -> Any:
    if not path.is_file():
        fail(f"expected regular JSON file: {path}")
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
    path.write_text(encoded, encoding="utf-8")


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def plan_identity(plan: dict[str, Any]) -> str:
    normalized = dict(plan)
    normalized["planIdentity"] = ""
    return "monte-carlo-precision-plan-v1-" + hashlib.sha256(canonical_bytes(normalized)).hexdigest()


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def study_protocol_identity(protocol: dict[str, Any]) -> str:
    return f"study-protocol-v1-{fnv1a64(canonical_bytes(protocol)):016x}"


def require_keys(obj: dict[str, Any], allowed: set[str], required: set[str], role: str) -> None:
    unknown = set(obj) - allowed
    missing = required - set(obj)
    if unknown:
        fail(f"{role} contains unknown field(s): {', '.join(sorted(unknown))}")
    if missing:
        fail(f"{role} is missing required field(s): {', '.join(sorted(missing))}")


def validate_plan(plan: dict[str, Any]) -> str:
    require_keys(
        plan,
        {"schemaVersion", "planIdentity", "planId", "uncertaintyCategory", "estimand", "design", "pairing", "rationale"},
        {"schemaVersion", "planIdentity", "planId", "uncertaintyCategory", "estimand", "design", "pairing", "rationale"},
        "precision plan",
    )
    if plan["schemaVersion"] != PLAN_SCHEMA:
        fail(f"unsupported precision-plan schema {plan['schemaVersion']}; supported schema is {PLAN_SCHEMA}")
    if plan["uncertaintyCategory"] != UNCERTAINTY_CATEGORY:
        fail("precision plan must explicitly classify uncertainty as process_stochastic_monte_carlo")
    if not isinstance(plan["planId"], str) or not plan["planId"].strip():
        fail("precision plan planId must be non-empty")
    if not isinstance(plan["rationale"], str) or not plan["rationale"].strip():
        fail("precision plan rationale must be non-empty")

    estimand = plan["estimand"]
    if not isinstance(estimand, dict):
        fail("precision plan estimand must be an object")
    require_keys(
        estimand,
        {"kind", "confidenceLevel", "maxHalfWidth", "quantileProbability"},
        {"kind", "confidenceLevel", "maxHalfWidth"},
        "estimand",
    )
    kind = estimand["kind"]
    if kind not in SUPPORTED_KINDS:
        fail(f"unsupported estimand kind: {kind}")
    confidence = float(estimand["confidenceLevel"])
    threshold = float(estimand["maxHalfWidth"])
    if not 0.0 < confidence < 1.0:
        fail("estimand confidenceLevel must be between 0 and 1")
    if not math.isfinite(threshold) or threshold <= 0.0:
        fail("estimand maxHalfWidth must be finite and > 0")
    if kind == "quantile":
        if "quantileProbability" not in estimand:
            fail("quantile estimand requires quantileProbability")
        probability = float(estimand["quantileProbability"])
        if not 0.0 < probability < 1.0:
            fail("quantileProbability must be between 0 and 1")
    elif "quantileProbability" in estimand:
        fail("quantileProbability is only valid for quantile estimands")

    design = plan["design"]
    if not isinstance(design, dict):
        fail("precision plan design must be an object")
    require_keys(design, {"mode", "seedBatches"}, {"mode", "seedBatches"}, "design")
    mode = design["mode"]
    if mode not in {"fixed", "sequential"}:
        fail("design.mode must be fixed or sequential")
    batches = design["seedBatches"]
    if not isinstance(batches, list) or not batches:
        fail("design.seedBatches must contain at least one declared batch")
    if mode == "fixed" and len(batches) != 1:
        fail("fixed design must declare exactly one seed batch")
    seen: set[int] = set()
    for index, batch in enumerate(batches):
        if not isinstance(batch, list) or not batch:
            fail(f"design.seedBatches[{index}] must be non-empty")
        for seed in batch:
            if not isinstance(seed, int) or isinstance(seed, bool) or seed < 0:
                fail("declared seeds must be non-negative integers")
            if seed in seen:
                fail(f"declared seed {seed} appears more than once")
            seen.add(seed)

    pairing = plan["pairing"]
    expected_pairing = "paired_by_seed" if kind == "paired_mean_difference" else "independent"
    if pairing != expected_pairing:
        fail(f"estimand {kind} requires pairing={expected_pairing}")

    expected = plan_identity(plan)
    if plan["planIdentity"] != expected:
        fail(f"precision plan identity mismatch; expected {expected}")
    return expected


def declared_prefixes(plan: dict[str, Any]) -> list[list[int]]:
    result: list[list[int]] = []
    current: list[int] = []
    for batch in plan["design"]["seedBatches"]:
        current = current + list(batch)
        result.append(list(current))
    return result


def validate_study_binding(study_dir: Path, plan: dict[str, Any], identity: str) -> dict[str, Any]:
    protocol = read_json(study_dir / "study-protocol.json")
    binding = read_json(study_dir / "study-result-binding.json")
    if not isinstance(protocol, dict) or not isinstance(binding, dict):
        fail("study protocol/result binding must be JSON objects")
    protocol_identity = study_protocol_identity(protocol)
    if binding.get("protocolIdentity") != protocol_identity:
        fail("study-result binding does not match the exact frozen study protocol")
    ensemble = protocol.get("ensemblePolicy")
    if not isinstance(ensemble, dict):
        fail("frozen study protocol has no ensemblePolicy object")
    expected_policy = PLAN_PREFIX + identity
    if ensemble.get("replicationPolicy") != expected_policy:
        fail(
            "frozen protocol does not bind this Monte Carlo precision plan; "
            f"ensemblePolicy.replicationPolicy must equal {expected_policy}"
        )
    if protocol.get("status") == "confirmatory":
        if binding.get("scientificStatus") != "confirmatory":
            fail("confirmatory protocol is not bound as a confirmatory study result")
        if binding.get("boundBeforeExecution") is not True:
            fail("confirmatory Monte Carlo gate requires protocol binding before execution")
        if binding.get("confirmatoryPreResultClaimEligible") is not True:
            fail("confirmatory Monte Carlo gate requires a pre-result eligible frozen protocol")
    return {
        "protocolIdentity": protocol_identity,
        "protocolRevision": binding.get("protocolRevision"),
        "studyId": binding.get("studyId"),
        "studyResultIdentity": binding.get("resultIdentity"),
        "researchId": binding.get("researchId"),
        "scientificStatus": binding.get("scientificStatus"),
        "boundBeforeExecution": binding.get("boundBeforeExecution"),
        "confirmatoryPreResultClaimEligible": binding.get("confirmatoryPreResultClaimEligible"),
    }


def validate_samples(samples: dict[str, Any], plan: dict[str, Any]) -> tuple[list[dict[str, Any]], int, int]:
    require_keys(samples, {"schemaVersion", "groups"}, {"schemaVersion", "groups"}, "sample")
    if samples["schemaVersion"] != SAMPLE_SCHEMA:
        fail(f"unsupported sample schema {samples['schemaVersion']}; supported schema is {SAMPLE_SCHEMA}")
    groups = samples["groups"]
    if not isinstance(groups, list) or not groups:
        fail("sample groups must be a non-empty array")
    kind = plan["estimand"]["kind"]
    expected_groups = 2 if kind in {"difference_in_means", "paired_mean_difference"} else 1
    if len(groups) != expected_groups:
        fail(f"estimand {kind} requires exactly {expected_groups} sample group(s)")

    parsed: list[dict[str, Any]] = []
    for group_index, group in enumerate(groups):
        if not isinstance(group, dict):
            fail("sample group must be an object")
        require_keys(group, {"id", "replicates"}, {"id", "replicates"}, f"sample group {group_index}")
        if not isinstance(group["id"], str) or not group["id"].strip():
            fail("sample group id must be non-empty")
        reps = group["replicates"]
        if not isinstance(reps, list) or len(reps) < 2:
            fail("each sample group requires at least two replicates")
        seeds: list[int] = []
        values: list[float] = []
        for rep in reps:
            if not isinstance(rep, dict):
                fail("replicate must be an object")
            require_keys(rep, {"seed", "value"}, {"seed", "value"}, "replicate")
            seed = rep["seed"]
            if not isinstance(seed, int) or isinstance(seed, bool) or seed < 0:
                fail("replicate seed must be a non-negative integer")
            raw_value = rep["value"]
            if kind == "probability":
                if raw_value in (True, 1):
                    value = 1.0
                elif raw_value in (False, 0):
                    value = 0.0
                else:
                    fail("probability replicates must be boolean or 0/1")
            else:
                if isinstance(raw_value, bool) or not isinstance(raw_value, (int, float)):
                    fail("continuous replicate values must be numeric")
                value = float(raw_value)
                if not math.isfinite(value):
                    fail("replicate values must be finite")
            seeds.append(seed)
            values.append(value)
        if len(set(seeds)) != len(seeds):
            fail(f"sample group {group['id']} contains duplicate seeds")
        parsed.append({"id": group["id"], "seeds": seeds, "values": values})

    prefixes = declared_prefixes(plan)
    first_seeds = parsed[0]["seeds"]
    if first_seeds not in prefixes:
        fail("sample seeds are not exactly one predeclared cumulative batch boundary")
    boundary_index = prefixes.index(first_seeds)
    for group in parsed[1:]:
        if group["seeds"] != first_seeds:
            fail("multi-group estimands must use the same exact declared seed identities and order")
    if plan["design"]["mode"] == "fixed" and boundary_index != len(prefixes) - 1:
        fail("fixed design may only be diagnosed at its single final declared sample")
    return parsed, len(first_seeds), boundary_index


def mean_and_variance(values: list[float]) -> tuple[float, float]:
    n = len(values)
    mean = sum(values) / n
    variance = sum((value - mean) ** 2 for value in values) / (n - 1)
    return mean, variance


def normal_interval(mean: float, variance: float, n: int, confidence: float) -> tuple[float, float, float]:
    z = NormalDist().inv_cdf(0.5 + confidence / 2.0)
    half = z * math.sqrt(variance / n)
    return mean - half, mean + half, half


def quantile(values: list[float], probability: float) -> float:
    ordered = sorted(values)
    position = probability * (len(ordered) - 1)
    lower = int(math.floor(position))
    upper = int(math.ceil(position))
    if lower == upper:
        return ordered[lower]
    fraction = position - lower
    return ordered[lower] * (1.0 - fraction) + ordered[upper] * fraction


def exact_quantile_rank_interval(n: int, probability: float, confidence: float) -> dict[str, Any]:
    """Return deterministic finite-sample order-statistic bounds.

    For a continuous distribution and true p-quantile q, K = # {X_i < q}
    follows Binomial(n, p). A 0-based interval [l, u] covers q exactly when
    l + 1 <= K <= u. Rank selection depends only on n, p, and confidence,
    never on the observed values.
    """
    log_n_factorial = math.lgamma(n + 1)
    log_p = math.log(probability)
    log_one_minus_p = math.log1p(-probability)
    probabilities = [
        math.exp(
            log_n_factorial
            - math.lgamma(k + 1)
            - math.lgamma(n - k + 1)
            + k * log_p
            + (n - k) * log_one_minus_p
        )
        for k in range(n + 1)
    ]
    cumulative: list[float] = []
    running = 0.0
    for mass in probabilities:
        running += mass
        cumulative.append(running)

    maximum_coverage = math.fsum(probabilities[1:n])
    tolerance = 1e-12
    if maximum_coverage + tolerance < confidence:
        return {
            "feasible": False,
            "lowerIndex": None,
            "upperIndex": None,
            "achievedCoverage": None,
            "maximumAchievableCoverage": maximum_coverage,
        }

    estimate_position = probability * (n - 1)
    estimate_lower = int(math.floor(estimate_position))
    estimate_upper = int(math.ceil(estimate_position))
    best: tuple[tuple[float, ...], int, int, float] | None = None
    for lower_index in range(n - 1):
        if lower_index > estimate_lower:
            break
        first_upper = max(lower_index + 1, estimate_upper)
        target_cdf = cumulative[lower_index] + confidence - tolerance
        upper_index = bisect_left(cumulative, target_cdf, lo=first_upper)
        if upper_index >= n:
            continue
        coverage = cumulative[upper_index] - cumulative[lower_index]
        if coverage + tolerance < confidence:
            continue
        lower_tail = cumulative[lower_index]
        upper_tail = max(0.0, 1.0 - cumulative[upper_index])
        key = (
            float(upper_index - lower_index),
            abs(lower_tail - upper_tail),
            float(lower_index),
        )
        if best is None or key < best[0]:
            best = (key, lower_index, upper_index, coverage)

    if best is None:
        raise AssertionError("feasible exact quantile interval was not found")
    _, lower_index, upper_index, coverage = best
    return {
        "feasible": True,
        "lowerIndex": lower_index,
        "upperIndex": upper_index,
        "achievedCoverage": coverage,
        "maximumAchievableCoverage": maximum_coverage,
    }


def diagnostic(groups: list[dict[str, Any]], plan: dict[str, Any]) -> dict[str, Any]:
    estimand = plan["estimand"]
    kind = estimand["kind"]
    confidence = float(estimand["confidenceLevel"])
    threshold = float(estimand["maxHalfWidth"])
    z = NormalDist().inv_cdf(0.5 + confidence / 2.0)

    if kind == "mean":
        values = groups[0]["values"]
        estimate, variance = mean_and_variance(values)
        lower, upper, half = normal_interval(estimate, variance, len(values), confidence)
        method = "normal_clt_mean_se"
    elif kind == "difference_in_means":
        left, right = groups[0]["values"], groups[1]["values"]
        left_mean, left_var = mean_and_variance(left)
        right_mean, right_var = mean_and_variance(right)
        estimate = left_mean - right_mean
        half = z * math.sqrt(left_var / len(left) + right_var / len(right))
        lower, upper = estimate - half, estimate + half
        method = "normal_clt_independent_difference_in_means"
    elif kind == "paired_mean_difference":
        differences = [left - right for left, right in zip(groups[0]["values"], groups[1]["values"])]
        estimate, variance = mean_and_variance(differences)
        lower, upper, half = normal_interval(estimate, variance, len(differences), confidence)
        method = "normal_clt_paired_seed_difference"
    elif kind == "probability":
        values = groups[0]["values"]
        n = len(values)
        successes = sum(values)
        estimate = successes / n
        denominator = 1.0 + z * z / n
        center = (estimate + z * z / (2.0 * n)) / denominator
        half = z * math.sqrt(estimate * (1.0 - estimate) / n + z * z / (4.0 * n * n)) / denominator
        lower, upper = max(0.0, center - half), min(1.0, center + half)
        half = max(estimate - lower, upper - estimate)
        method = "wilson_score_probability"
    elif kind == "quantile":
        values = groups[0]["values"]
        n = len(values)
        probability = float(estimand["quantileProbability"])
        ordered = sorted(values)
        estimate = quantile(values, probability)
        rank_support = exact_quantile_rank_interval(n, probability, confidence)
        method = "distribution_free_exact_binomial_order_statistic_interval"
        if rank_support["feasible"]:
            lower_index = rank_support["lowerIndex"]
            upper_index = rank_support["upperIndex"]
            assert isinstance(lower_index, int) and isinstance(upper_index, int)
            lower, upper = ordered[lower_index], ordered[upper_index]
            half = max(estimate - lower, upper - estimate)
        else:
            lower = upper = half = None
    else:
        raise AssertionError(kind)

    precision = {
        "estimate": estimate,
        "intervalLower": lower,
        "intervalUpper": upper,
        "halfWidth": half,
        "confidenceLevel": confidence,
        "precisionMethod": method,
        "declaredMaxHalfWidth": threshold,
        "sufficient": half is not None and half <= threshold,
    }
    if kind == "quantile":
        precision.update(
            {
                "coverageFeasible": rank_support["feasible"],
                "achievedCoverage": rank_support["achievedCoverage"],
                "maximumAchievableCoverage": rank_support["maximumAchievableCoverage"],
                "lowerOrderStatisticRank": None
                if rank_support["lowerIndex"] is None
                else rank_support["lowerIndex"] + 1,
                "upperOrderStatisticRank": None
                if rank_support["upperIndex"] is None
                else rank_support["upperIndex"] + 1,
                "coverageAssumption": "continuous_distribution_true_quantile",
            }
        )
    return precision


def derive(plan: dict[str, Any], samples: dict[str, Any], study_dir: Path | None) -> dict[str, Any]:
    identity = validate_plan(plan)
    groups, replicate_count, boundary_index = validate_samples(samples, plan)
    lineage = validate_study_binding(study_dir, plan, identity) if study_dir is not None else None
    precision = diagnostic(groups, plan)
    prefixes = declared_prefixes(plan)
    has_next = boundary_index + 1 < len(prefixes)
    next_batch = plan["design"]["seedBatches"][boundary_index + 1] if has_next else []
    mode = plan["design"]["mode"]
    if precision["sufficient"]:
        decision = "sufficient_stop"
    elif plan["estimand"]["kind"] == "quantile" and not precision["coverageFeasible"]:
        if mode == "sequential" and has_next:
            decision = "insufficient_quantile_coverage_continue_with_declared_next_batch"
        else:
            decision = "insufficient_quantile_coverage_no_predeclared_additional_batch"
    elif mode == "sequential" and has_next:
        decision = "insufficient_continue_with_declared_next_batch"
    else:
        decision = "insufficient_no_predeclared_additional_batch"

    result = {
        "schemaVersion": DIAGNOSTIC_SCHEMA,
        "planIdentity": identity,
        "planId": plan["planId"],
        "uncertaintyCategory": UNCERTAINTY_CATEGORY,
        "estimand": plan["estimand"],
        "designMode": mode,
        "batchBoundary": boundary_index + 1,
        "replicateCount": replicate_count,
        "seedIdentities": groups[0]["seeds"],
        "groupIds": [group["id"] for group in groups],
        "precision": precision,
        "decision": decision,
        "nextDeclaredBatchSeeds": next_batch,
        "scientificInterpretation": {
            "represents": "Monte Carlo/process stochastic uncertainty conditional on this declared model, parameterization, evidence treatment, estimand, and seed design.",
            "doesNotRepresent": [
                "parameter uncertainty",
                "archaeological or evidence uncertainty",
                "structural or model-form uncertainty",
            ],
        },
    }
    if lineage is not None:
        result["studyLineage"] = lineage
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    identity_parser = sub.add_parser("identity", help="print the canonical precision-plan identity")
    identity_parser.add_argument("plan", type=Path)

    validate_parser = sub.add_parser("validate-plan", help="validate a precision plan and print its identity")
    validate_parser.add_argument("plan", type=Path)

    diagnose_parser = sub.add_parser("diagnose", help="derive a deterministic replicate-sufficiency diagnostic")
    diagnose_parser.add_argument("plan", type=Path)
    diagnose_parser.add_argument("samples", type=Path)
    diagnose_parser.add_argument("output", type=Path)
    diagnose_parser.add_argument("--study-dir", type=Path)

    args = parser.parse_args()
    try:
        plan = read_json(args.plan)
        if not isinstance(plan, dict):
            fail("precision plan must be a JSON object")
        if args.command == "identity":
            print(plan_identity(plan))
            return 0
        identity = validate_plan(plan)
        if args.command == "validate-plan":
            print(identity)
            return 0
        samples = read_json(args.samples)
        if not isinstance(samples, dict):
            fail("sample input must be a JSON object")
        result = derive(plan, samples, args.study_dir)
        write_json(args.output, result)
        print(result["decision"])
        return 0 if result["precision"]["sufficient"] else 2
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"research-monte-carlo-sufficiency: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
