#!/usr/bin/env python3
"""Aggregate the predeclared M8.6 terrain benchmark without manual result selection."""

from __future__ import annotations

import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path
from statistics import median
from typing import Any

ARMS = ("flat", "weak", "moderate", "strong")
SEEDS = tuple(range(8601, 8609))
PRIMARY_METRICS = {
    "migrationTotalDistanceCells": "migrationTotalDistanceCells",
    "cellTimeOccupiedPermille": "cellTimeOccupiedPermille",
    "terminalPopulationHerfindahlPerMillion": "terminalPopulationHerfindahlPerMillion",
    "terminalLargestCellSharePermille": "terminalLargestCellSharePermille",
}
# RunManifest serializes StopReason with serde rename_all = "camelCase".
DURATION_STOP_REASON = "durationReached"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--markdown", type=Path)
    return parser.parse_args()


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def canonical_sha256(value: Any) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(data).hexdigest()


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def seed_run_id(seed: int) -> str:
    return f"seed-{seed:020d}"


def fraction_text(value: Fraction | None) -> str | None:
    if value is None:
        return None
    return f"{value.numerator}/{value.denominator}"


def median_fraction(values: list[Fraction]) -> Fraction | None:
    if not values:
        return None
    return Fraction(median(values))


def sign(value: Fraction | int | None) -> int:
    if value is None or value == 0:
        return 0
    return 1 if value > 0 else -1


def percent_text(value: Fraction | None) -> str:
    if value is None:
        return "n/a"
    hundredths = (abs(value.numerator) * 10_000 + abs(value.denominator) // 2) // abs(value.denominator)
    return f"{hundredths / 100:.2f}%"


def load_arm(root: Path, arm: str) -> dict[str, Any]:
    arm_root = root / arm
    manifest_path = arm_root / "experiment-manifest.json"
    landscape_path = arm_root / "landscape.json"
    evidence_path = arm_root / "evidence.json"
    mechanisms_path = arm_root / "spatial-mechanisms.json"
    for path in (manifest_path, landscape_path, evidence_path, mechanisms_path):
        if not path.is_file():
            raise SystemExit(f"{arm}: missing required experiment artifact {path}")

    experiment_manifest = read_json(manifest_path)
    landscape = read_json(landscape_path)
    evidence = read_json(evidence_path)
    mechanisms = read_json(mechanisms_path)
    specs = {int(spec["experiment"]["seed"]): spec for spec in experiment_manifest["runs"]}
    if set(specs) != set(SEEDS):
        raise SystemExit(f"{arm}: immutable experiment seeds differ from predeclared {list(SEEDS)}")

    external_inputs = evidence.get("externalInputs", [])
    source_digests = sorted(
        value.get("contentDigest")
        for value in external_inputs
        if value.get("contentDigest") is not None
    )
    runs: dict[str, Any] = {}
    degenerate_count = 0

    for seed in SEEDS:
        spec = specs[seed]
        run_id = seed_run_id(seed)
        status_path = arm_root / "status" / f"{run_id}.json"
        status = read_json(status_path) if status_path.is_file() else {
            "state": "missing",
            "attempt": 0,
            "message": "status record missing",
        }
        run_dir = arm_root / spec["relativeRunDir"]
        experiment = spec["experiment"]
        evidence_catalog = experiment.get("evidence")
        record: dict[str, Any] = {
            "seed": seed,
            "runId": run_id,
            "status": status.get("state"),
            "attempt": status.get("attempt"),
            "statusMessage": status.get("message"),
            "experimentId": experiment_manifest["experimentId"],
            "experimentConfigSha256": canonical_sha256(experiment),
            "evidenceCatalogSha256": canonical_sha256(evidence_catalog),
            "modelVersion": experiment_manifest["modelVersion"],
            "gitCommit": experiment_manifest.get("gitCommit"),
            "terminalDegenerate": True,
            "stopReason": None,
            "stateDigest64": None,
            "landscapeIdentity": None,
            "landscapeDigest64": None,
            "transformedWorldDigest64": None,
            "spatialModelSemanticsId": None,
            "spatialConfigIdentity": None,
            "modelSemanticsId": None,
            "metrics": None,
        }

        if status.get("state") == "completed":
            run_manifest_path = run_dir / "manifest.json"
            observability_path = run_dir / "spatial-observability.json"
            if not run_manifest_path.is_file() or not observability_path.is_file():
                raise SystemExit(
                    f"{arm}/{run_id}: completed run lacks manifest.json or spatial-observability.json"
                )
            run_manifest = read_json(run_manifest_path)
            observability = read_json(observability_path)
            if run_manifest["experiment"] != experiment:
                raise SystemExit(f"{arm}/{run_id}: run experiment differs from immutable manifest")
            source = observability["source"]
            if int(source["seed"]) != seed:
                raise SystemExit(f"{arm}/{run_id}: observability seed mismatch")
            if int(source["runStateDigest64"]) != int(run_manifest["stateDigest64"]):
                raise SystemExit(f"{arm}/{run_id}: observability state digest mismatch")
            summary = observability["summary"]
            metrics = {name: summary.get(field) for name, field in PRIMARY_METRICS.items()}
            stop_reason = run_manifest["stopReason"]
            terminal_degenerate = stop_reason != DURATION_STOP_REASON
            if terminal_degenerate:
                degenerate_count += 1
            record.update(
                {
                    "terminalDegenerate": terminal_degenerate,
                    "stopReason": stop_reason,
                    "stateDigest64": run_manifest["stateDigest64"],
                    "landscapeIdentity": source["landscapeIdentity"],
                    "landscapeDigest64": source["landscapeDigest64"],
                    "transformedWorldDigest64": source["transformedWorldDigest64"],
                    "spatialModelSemanticsId": source.get("spatialModelSemanticsId"),
                    "spatialConfigIdentity": source.get("spatialConfigIdentity"),
                    "modelSemanticsId": source["modelSemanticsId"],
                    "metrics": metrics,
                    "secondary": {
                        "terminalLivingPopulation": summary["terminalLivingPopulation"],
                        "terminalOccupiedCells": summary["terminalOccupiedCells"],
                        "births": summary["births"],
                        "deaths": summary["deaths"],
                        "conditionMortalityDeaths": summary["conditionMortalityDeaths"],
                        "migrationMoves": summary["migrationMoves"],
                        "migrationPeopleMoved": summary["migrationPeopleMoved"],
                    },
                }
            )
        else:
            degenerate_count += 1

        runs[str(seed)] = record

    return {
        "arm": arm,
        "experimentId": experiment_manifest["experimentId"],
        "experimentManifestSha256": file_sha256(manifest_path),
        "landscapeFileSha256": file_sha256(landscape_path),
        "landscapeCanonicalSha256": canonical_sha256(landscape),
        "evidenceFileSha256": file_sha256(evidence_path),
        "evidenceCanonicalSha256": canonical_sha256(evidence),
        "mechanismsFileSha256": file_sha256(mechanisms_path),
        "mechanismsCanonicalSha256": canonical_sha256(mechanisms),
        "sourceContentDigests": source_digests,
        "terminalDegenerateRuns": degenerate_count,
        "runs": runs,
    }


def pair_stats(flat: dict[str, Any], arm: dict[str, Any], metric: str) -> dict[str, Any]:
    pairs: list[dict[str, Any]] = []
    effects: list[Fraction] = []
    relative: list[Fraction] = []
    positive = negative = zero = unavailable = zero_baselines = 0

    for seed in SEEDS:
        flat_record = flat["runs"][str(seed)]
        arm_record = arm["runs"][str(seed)]
        flat_metrics = flat_record.get("metrics")
        arm_metrics = arm_record.get("metrics")
        flat_value = None if flat_metrics is None else flat_metrics.get(metric)
        arm_value = None if arm_metrics is None else arm_metrics.get(metric)
        pair: dict[str, Any] = {
            "seed": seed,
            "flat": flat_value,
            "arm": arm_value,
            "effect": None,
            "relativeEffect": None,
            "available": False,
        }
        if flat_record.get("terminalDegenerate") or arm_record.get("terminalDegenerate"):
            unavailable += 1
            pair["reason"] = (
                "fixed-horizon primary metric unavailable because one or both paired runs "
                "did not reach the declared duration"
            )
            pairs.append(pair)
            continue
        if flat_value is None or arm_value is None:
            unavailable += 1
            pair["reason"] = "primary metric unavailable because one or both paired runs were not analyzable"
            pairs.append(pair)
            continue

        effect = Fraction(int(arm_value) - int(flat_value), 1)
        effects.append(effect)
        effect_sign = sign(effect)
        if effect_sign > 0:
            positive += 1
        elif effect_sign < 0:
            negative += 1
        else:
            zero += 1
        rel = None
        if int(flat_value) == 0:
            zero_baselines += 1
        else:
            rel = effect / abs(int(flat_value))
            relative.append(abs(rel))
        pair.update(
            {
                "effect": fraction_text(effect),
                "relativeEffect": fraction_text(rel),
                "available": True,
            }
        )
        pairs.append(pair)

    median_effect = median_fraction(effects)
    median_abs_relative = median_fraction(relative)
    return {
        "pairs": pairs,
        "availablePairs": len(effects),
        "unavailablePairs": unavailable,
        "positiveEffects": positive,
        "negativeEffects": negative,
        "zeroEffects": zero,
        "zeroFlatBaselines": zero_baselines,
        "medianEffect": fraction_text(median_effect),
        "medianAbsoluteRelativeEffect": fraction_text(median_abs_relative),
        "medianAbsoluteRelativeEffectDisplay": percent_text(median_abs_relative),
        "_medianEffect": median_effect,
        "_medianAbsRelative": median_abs_relative,
    }


def classify_metric(flat: dict[str, Any], arms: dict[str, dict[str, Any]], metric: str) -> dict[str, Any]:
    comparisons = {
        arm: pair_stats(flat, arms[arm], metric)
        for arm in ("weak", "moderate", "strong")
    }
    strong = comparisons["strong"]
    moderate = comparisons["moderate"]
    strong_same_sign = max(strong["positiveEffects"], strong["negativeEffects"])
    strong_rel = strong["_medianAbsRelative"]
    strong_median = strong["_medianEffect"]
    moderate_median = moderate["_medianEffect"]
    same_median_sign = (
        sign(strong_median) != 0
        and sign(strong_median) == sign(moderate_median)
    )
    robust = (
        strong_same_sign >= 6
        and strong_rel is not None
        and strong_rel >= Fraction(1, 10)
        and same_median_sign
    )
    fragile = (
        not robust
        and strong_rel is not None
        and strong_rel >= Fraction(1, 10)
    )
    classification = "robust" if robust else "fragile" if fragile else "not_distinctive"

    for stats in comparisons.values():
        stats.pop("_medianEffect", None)
        stats.pop("_medianAbsRelative", None)
    return {
        "classification": classification,
        "robustCriteria": {
            "strongSameSignNonZeroAtLeast6Of8": strong_same_sign >= 6,
            "strongMedianAbsoluteRelativeAtLeast10Percent": (
                strong_rel is not None and strong_rel >= Fraction(1, 10)
            ),
            "moderateMedianSameSignAsStrong": same_median_sign,
        },
        "comparisonsToFlat": comparisons,
    }


def aggregate(root: Path) -> dict[str, Any]:
    arms = {arm: load_arm(root, arm) for arm in ARMS}
    metric_results = {
        metric: classify_metric(arms["flat"], arms, metric)
        for metric in PRIMARY_METRICS
    }

    degenerate_arms = [
        arm for arm in ARMS if arms[arm]["terminalDegenerateRuns"] >= 4
    ]
    robust_metrics = [
        metric for metric, result in metric_results.items() if result["classification"] == "robust"
    ]
    fragile_metrics = [
        metric for metric, result in metric_results.items() if result["classification"] == "fragile"
    ]
    if degenerate_arms:
        benchmark_class = "degenerate"
    elif len(robust_metrics) >= 2:
        benchmark_class = "robust_spatial_structure"
    elif robust_metrics or fragile_metrics:
        benchmark_class = "fragile_spatial_structure"
    else:
        benchmark_class = "no_distinctive_spatial_structure"

    result: dict[str, Any] = {
        "schemaVersion": 2,
        "benchmarkId": "m8_6_first_evidence_grounded_terrain_null_model_v1",
        "interpretationBoundary": (
            "Behavior of the declared AnthroSim terrain-only null model under the tested constraints; "
            "not a historical reconstruction or archaeological validation."
        ),
        "declaredSeeds": list(SEEDS),
        "arms": {arm: arms[arm] for arm in ARMS},
        "primaryMetrics": metric_results,
        "classification": {
            "benchmarkClass": benchmark_class,
            "degenerateArms": degenerate_arms,
            "robustMetrics": robust_metrics,
            "fragileMetrics": fragile_metrics,
        },
    }
    result["aggregateCanonicalSha256"] = canonical_sha256(result)
    return result


def render_markdown(result: dict[str, Any]) -> str:
    classification = result["classification"]
    lines = [
        "# M8.6 first evidence-grounded spatial null-model result",
        "",
        f"Benchmark class: **{classification['benchmarkClass']}**.",
        "",
        "This is a result about the declared terrain-only null model, not a historical reconstruction or archaeological validation.",
        "",
        "## Primary metrics",
        "",
        "| Metric | Classification | Strong median absolute relative effect | Strong signs (+ / - / 0) |",
        "| --- | --- | ---: | ---: |",
    ]
    for metric, value in result["primaryMetrics"].items():
        strong = value["comparisonsToFlat"]["strong"]
        lines.append(
            f"| `{metric}` | {value['classification']} | "
            f"{strong['medianAbsoluteRelativeEffectDisplay']} | "
            f"{strong['positiveEffects']} / {strong['negativeEffects']} / {strong['zeroEffects']} |"
        )
    lines.extend([
        "",
        "## Degenerate runs",
        "",
    ])
    for arm in ARMS:
        lines.append(f"- `{arm}`: {result['arms'][arm]['terminalDegenerateRuns']} of 8")
    lines.extend([
        "",
        f"Aggregate canonical SHA-256: `{result['aggregateCanonicalSha256']}`",
        "",
    ])
    return "\n".join(lines)


def main() -> None:
    args = parse_args()
    result = aggregate(args.root)
    write_json(args.output, result)
    if args.markdown:
        args.markdown.parent.mkdir(parents=True, exist_ok=True)
        args.markdown.write_text(render_markdown(result), encoding="utf-8")
    print(json.dumps(result["classification"], indent=2))


if __name__ == "__main__":
    main()
