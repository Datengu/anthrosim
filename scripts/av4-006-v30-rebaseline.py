#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REVIEW = Path("/tmp/av4-006-v30-review")
PRODUCTION_HEAD = "340e57ca7a864f6ae3b689f927ecda5e67db1a98"
MODEL_SEMANTICS = "anthrosim-model-semantics-v30"

ISSUE304_RUN = 33820753145
ISSUE304_JOB = 100862690900
ISSUE304_ARTIFACT = 9918271667
ISSUE304_DIGEST = "sha256:76a943bae2a8a3bef13ee551f17edeb6d485897b03a52d8607ebe3deeb3bb634"

M8_RUN = 33820753353
M8_JOB = 100862718819
M8_ARTIFACT = 9918254916
M8_DIGEST = "sha256:da9a934eb46176d7f8acc4e74f08a628ac72881c0756e7840a365420ea4ed292"

M9_RUN = 33820753353
M9_JOB = 100862718824
M9_ARTIFACT = 9918256387
M9_DIGEST = "sha256:98df4dd2bb411cc7dee7652c87466045e2ea53cb895686863245277c4c0d7cfc"

M7_RUN = 33821069979
M7_JOB = 100863637973
M7_ARTIFACT = 9918382546
M7_DIGEST = "sha256:3f27490d2c30098aab71ccc83d4f0ad2f822af708503694dae5b7f525cdd3920"


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value, *, compact: bool = False) -> None:
    if compact:
        text = json.dumps(value, separators=(",", ":")) + "\n"
    else:
        text = json.dumps(value, indent=2, sort_keys=True) + "\n"
    path.write_text(text, encoding="utf-8")


def append_once(path: Path, marker: str, section: str) -> None:
    text = path.read_text(encoding="utf-8")
    if marker in text:
        return
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text + "\n" + section.rstrip() + "\n", encoding="utf-8")


def pct(value: float) -> str:
    return f"{100.0 * value:.3f}%"


def main() -> None:
    provenance = (ROOT / "crates/anthrosim-core/src/provenance.rs").read_text(encoding="utf-8")
    match = re.search(r'pub const MODEL_SEMANTICS_ID: &str = "([^"]+)"', provenance)
    assert match and match.group(1) == MODEL_SEMANTICS

    # #304: use the exact machine-derived expected result from the completed 780-run review.
    expected304 = read_json(REVIEW / "issue304" / "expected-result.json")
    assert expected304["modelSemanticsId"] == MODEL_SEMANTICS
    assert expected304["runCount"] == 780
    assert expected304["recommendation"] == "no_universal_demographic_baseline"
    assert len(expected304["pairedHouseholdEffects"]) == 3
    decisions = [row["decision"] for row in expected304["monteCarloPrecision"].values()]
    assert decisions == ["sufficient_stop"] * 3
    shutil.copyfile(
        REVIEW / "issue304" / "expected-result.json",
        ROOT / "research/general-demography-baseline-v1/confirmatory-result.json",
    )

    # M7.6: derive the reference snapshot directly from the exact 144-run review artifact.
    m7_points = read_json(REVIEW / "m7" / "analysis/points.json")
    m7_summary = read_json(REVIEW / "m7" / "analysis/summary.json")
    m7_repro = read_json(REVIEW / "m7" / "reproduction-record.json")
    assert len(m7_points) == 18
    assert m7_summary["completedRuns"] == 144
    assert m7_summary["nonCompletedRuns"] == 0
    assert m7_repro["gitCommit"] == PRODUCTION_HEAD

    point_results = []
    by_pair = {}
    for point in sorted(m7_points, key=lambda row: row["pointId"]):
        result = {
            "pointId": point["pointId"],
            "resourceProductivityScalePermille": point["resourceProductivityScalePermille"],
            "resourceSeasonalityScalePermille": point["resourceSeasonalityScalePermille"],
            "migrationEnabled": not point["disableMigration"],
            "durationReachedRuns": point["durationReachedRuns"],
            "populationExtinctRuns": point["populationExtinctRuns"],
            "meanFinalLivingPopulationCompletedOnly": point["meanFinalLivingPopulationScientificallyEligibleOnly"],
            "meanFinalLivingOccupiedCellCountCompletedOnly": point["meanFinalLivingOccupiedCellCountScientificallyEligibleOnly"],
            "meanLivingConditionPermilleCompletedOnly": point["meanLivingConditionPermilleScientificallyEligibleOnly"],
            "meanConditionMortalityDeathsCompletedOnly": point["meanConditionMortalityDeathsScientificallyEligibleOnly"],
            "meanResourceUnmetNeedCompletedOnly": point["meanResourceUnmetNeedScientificallyEligibleOnly"],
            "meanMigrationMovesCompletedOnly": point["meanMigrationMovesScientificallyEligibleOnly"],
            "meanMigrationTotalDistanceCellsCompletedOnly": point["meanMigrationTotalDistanceCellsScientificallyEligibleOnly"],
            "pooledMeanMigrationDistanceCellsPerMoveCompletedOnly": point["pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly"],
        }
        point_results.append(result)
        key = (result["resourceProductivityScalePermille"], result["resourceSeasonalityScalePermille"])
        by_pair[(key, result["migrationEnabled"])] = result

    for productivity in (250, 500, 1000):
        for seasonality in (0, 500, 1000):
            mig = by_pair[((productivity, seasonality), True)]
            no_mig = by_pair[((productivity, seasonality), False)]
            assert mig["meanFinalLivingPopulationCompletedOnly"] > no_mig["meanFinalLivingPopulationCompletedOnly"]
            assert mig["meanConditionMortalityDeathsCompletedOnly"] < no_mig["meanConditionMortalityDeathsCompletedOnly"]
            assert mig["meanResourceUnmetNeedCompletedOnly"] < no_mig["meanResourceUnmetNeedCompletedOnly"]
    for seasonality in (0, 500, 1000):
        no_mig = by_pair[((250, seasonality), False)]
        mig = by_pair[((250, seasonality), True)]
        assert no_mig["populationExtinctRuns"] == 8
        assert mig["populationExtinctRuns"] == 0

    m7_path = ROOT / "experiments/v0.1-resource-variability-reference.json"
    m7_ref = read_json(m7_path)
    assert m7_ref["definitionId"] == m7_repro["definitionId"]
    assert m7_ref["definitionSha256"] == m7_repro["definitionSha256"]
    m7_ref.update(
        {
            "modelVersion": m7_repro["modelVersion"],
            "modelSemanticsId": MODEL_SEMANTICS,
            "referenceCiRunId": M7_RUN,
            "referenceTestMergeCommit": None,
            "referenceHeadCommit": PRODUCTION_HEAD,
            "referenceSweepId": m7_summary["sweepId"],
            "plannedRuns": 144,
            "completedRuns": 144,
            "nonCompletedRuns": 0,
            "pointResults": point_results,
            "note": (
                "Derived reference summary deliberately rebaselined for model semantics v30 after Audit-v4 "
                "AV4-006 / #497 replaced arbitrary canonical PersonId/record ordering in M3 condition-mediated "
                "mortality RNG assignment and simultaneous competing-risk cause attribution with persistent person "
                "stochastic-coupling-rank ordering. The source definition, 18-point factorial design, paired seeds, "
                "completion/censoring rules, migration configuration, resource-response semantics, and declared "
                "endpoints are unchanged. All 144 runs completed and remained scientifically eligible. Exact point "
                "values change under the causal mortality reassignment, but the declared synthetic validation pattern "
                "survives: all three low-productivity no-migration points go extinct in 8/8 seeds while their "
                "migration-enabled counterparts persist in 8/8, and at all 9 matched productivity/seasonality points "
                "migration-enabled runs retain higher terminal population with lower condition-mediated mortality and "
                "lower unmet resource need than migration-disabled controls. This snapshot freezes synthetic "
                "verification outputs only; it is not calibration evidence or an empirical anthropological claim."
            ),
        }
    )
    write_json(m7_path, m7_ref, compact=True)

    # M8.6: preserve the reference schema but populate it only from the complete v30 aggregate.
    m8_actual = read_json(REVIEW / "m8" / "m8-benchmark-output/benchmark-summary.json")
    assert m8_actual["classification"] == {
        "benchmarkClass": "fragile_spatial_structure",
        "degenerateArms": [],
        "robustMetrics": [],
        "fragileMetrics": ["terminalPopulationHerfindahlPerMillion", "terminalLargestCellSharePermille"],
    }
    m8_path = ROOT / "examples/m8-first-evidence-grounded-benchmark/reference-result.json"
    m8_ref = read_json(m8_path)
    flat_first = next(iter(m8_actual["arms"]["flat"]["runs"].values()))
    merge_ref_sha = flat_first["gitCommit"]
    assert flat_first["modelSemanticsId"] == MODEL_SEMANTICS
    m8_ref["referenceExecution"] = {
        "workflowRunId": M8_RUN,
        "artifactId": M8_ARTIFACT,
        "artifactSha256": M8_DIGEST.removeprefix("sha256:"),
        "branchHeadSha": PRODUCTION_HEAD,
        "pullRequestMergeRefBuildSha": merge_ref_sha,
        "fullAggregateCanonicalSha256": m8_actual["aggregateCanonicalSha256"],
        "pullRequestMergeRefBuildStatus": "pre-merge-v30-reverification-artifact",
    }
    source = dict(m8_ref["source"])
    source.update(
        {
            "landscapeIdentity": flat_first["landscapeIdentity"],
            "landscapeDigest64": flat_first["landscapeDigest64"],
            "evidenceCatalogCanonicalSha256": flat_first["evidenceCatalogSha256"],
            "modelSemanticsId": MODEL_SEMANTICS,
            "spatialModelSemanticsId": flat_first["spatialModelSemanticsId"],
        }
    )
    m8_ref["source"] = source
    m8_ref["declaredSeeds"] = m8_actual["declaredSeeds"]
    m8_ref["classification"] = m8_actual["classification"]
    metric_keys = (
        "medianEffect",
        "medianAbsoluteRelativeEffect",
        "positiveEffects",
        "negativeEffects",
        "zeroEffects",
    )
    m8_ref["primaryMetrics"] = {}
    for metric_name, metric in m8_actual["primaryMetrics"].items():
        out = {
            "classification": metric["classification"],
            "robustCriteria": metric["robustCriteria"],
        }
        for arm in ("weak", "moderate", "strong"):
            comparison = metric["comparisonsToFlat"][arm]
            out[arm] = {key: comparison[key] for key in metric_keys}
        m8_ref["primaryMetrics"][metric_name] = out
    m8_ref["arms"] = {}
    for arm in ("flat", "weak", "moderate", "strong"):
        actual_arm = m8_actual["arms"][arm]
        runs = actual_arm["runs"]
        first = next(iter(runs.values()))
        m8_ref["arms"][arm] = {
            "experimentId": actual_arm["experimentId"],
            "mechanismsCanonicalSha256": actual_arm["mechanismsCanonicalSha256"],
            "spatialConfigIdentity": first["spatialConfigIdentity"],
            "terminalDegenerateRuns": actual_arm["terminalDegenerateRuns"],
            "runStateDigest64": {seed: row["stateDigest64"] for seed, row in sorted(runs.items())},
        }
    write_json(m8_path, m8_ref, compact=True)

    # One auditable evidence note owns the v30 scientific interpretation and exact run provenance.
    arm_rows = []
    for arm in expected304["arms"]:
        arm_rows.append(
            "| {demography} | {life} | {n:.2f} | {growth} | {ext} | {mate} |".format(
                demography=arm["demography"],
                life=arm["householdLifecycle"],
                n=arm["terminalPopulation"]["mean"],
                growth=pct(arm["lateGrowthRatePerYear"]["mean"]),
                ext=pct(arm["extinction"]["estimate"]),
                mate=pct(arm["mateLimitationFraction"]["mean"]),
            )
        )
    paired_rows = []
    for effect in expected304["pairedHouseholdEffects"]:
        paired_rows.append(
            "| {demography} | {pop:.2f} | {growth} pp/yr | {mate} pp |".format(
                demography=effect["demography"],
                pop=effect["fissionMinusFixedTerminalPopulation"]["mean"],
                growth=f"{100.0 * effect['fissionMinusFixedLateGrowthRatePerYear']['mean']:.3f}",
                mate=f"{100.0 * effect['fissionMinusFixedMateLimitationFraction']['mean']:.3f}",
            )
        )

    m8_strong = {
        name: metric["comparisonsToFlat"]["strong"]
        for name, metric in m8_actual["primaryMetrics"].items()
    }
    note = f"""# Model-semantics v30 re-verification — AV4-006 condition-mortality coupling

This note records the deliberate scientific-reference review for Audit-v4 AV4-006 / #497. The production candidate reviewed here is `{PRODUCTION_HEAD}` with `{MODEL_SEMANTICS}` and checkpoint schema 18. The repair changes same-seed assignment of condition-mediated mortality draws and simultaneous condition/background cause attribution, so downstream frozen results were rerun rather than relabelled.

## #304 confirmatory demographic baseline

Run `{ISSUE304_RUN}`, job `{ISSUE304_JOB}`, artifact `{ISSUE304_ARTIFACT}` (`{ISSUE304_DIGEST}`) completed all **780/780** confirmatory arm-runs. All three predeclared Monte Carlo gates returned `sufficient_stop`; the recommendation remains **`no_universal_demographic_baseline`**.

| demography | household lifecycle | mean N240 | late growth | extinction | mate limitation |
| --- | --- | ---: | ---: | ---: | ---: |
{chr(10).join(arm_rows)}

Paired fission-minus-fixed effects:

| demography | terminal N | late growth | mate limitation |
| --- | ---: | ---: | ---: |
{chr(10).join(paired_rows)}

Long-run classification counts remain `{expected304['longRun']['primaryClassificationCounts']}`; stochastic multi-regime contexts = `{expected304['longRun']['stochasticMultiRegimeContextCount']}`; environment dependence = `{expected304['longRun']['environmentDependenceDetected']}`; initialization dependence = `{expected304['longRun']['initializationDependenceDetected']}`.

## M7.6 resource-variability reference

Review run `{M7_RUN}`, job `{M7_JOB}`, artifact `{M7_ARTIFACT}` (`{M7_DIGEST}`) checked out the exact production candidate and completed **144/144** canonical runs. Exact point values change under corrected condition-mortality coupling, but the declared synthetic validation conclusion survives unchanged: each of the three 250-permille no-migration points goes extinct in 8/8 seeds while the matched migration-enabled point persists in 8/8, and migration-enabled runs have higher terminal population, lower condition-mediated mortality, and lower unmet resource need at **all 9/9** matched productivity/seasonality points.

## M8.6 terrain null-model benchmark

Run `{M8_RUN}`, job `{M8_JOB}`, artifact `{M8_ARTIFACT}` (`{M8_DIGEST}`) completed all four declared arms and all 32 runs. The exact trajectories and paired effects change, but the benchmark classification remains **`fragile_spatial_structure`** with no robust metrics. `terminalPopulationHerfindahlPerMillion` and `terminalLargestCellSharePermille` remain fragile; migration distance and occupied-cell time remain not distinctive.

Strong-arm median absolute relative effects are:

- migration total distance: `{m8_strong['migrationTotalDistanceCells']['medianAbsoluteRelativeEffectDisplay']}`;
- occupied-cell time: `{m8_strong['cellTimeOccupiedPermille']['medianAbsoluteRelativeEffectDisplay']}`;
- terminal Herfindahl: `{m8_strong['terminalPopulationHerfindahlPerMillion']['medianAbsoluteRelativeEffectDisplay']}`;
- largest-cell share: `{m8_strong['terminalLargestCellSharePermille']['medianAbsoluteRelativeEffectDisplay']}`.

## M9.7 controlled aggregation benchmark

The same applicable-gate run `{M9_RUN}`, job `{M9_JOB}`, artifact `{M9_ARTIFACT}` (`{M9_DIGEST}`) passed **without any reference change**. Both ensembles, M9.6 observability, identical replay, active annual checkpoint/resume, the preserved M9.7 scientific reference, and tamper rejection all passed. The M9 machine reference is therefore intentionally left untouched.

## Rebaseline decision

Only #304, M7.6, and M8.6 are rebaselined, from their complete archived v30 outputs. M9.7 is not rebaselined because its existing scientific reference already matches v30 exactly. No threshold, gate, seed set, design, or scientific acceptance rule is weakened. The exact original Audit-v4 #496 adversary remains mandatory after production merge before #497 can close.
"""
    (ROOT / "docs/research/condition-mortality-coupling-v30-reverification.md").write_text(note, encoding="utf-8")

    append_once(
        ROOT / "docs/research/general-scientific-demographic-baseline-v1.md",
        "## v30 condition-mortality-coupling re-verification",
        """## v30 condition-mortality-coupling re-verification

Audit-v4 AV4-006 / #497 deliberately reran the complete 780-run confirmatory design under `anthrosim-model-semantics-v30`. All three Monte Carlo precision gates remained sufficient and the high-level recommendation remains `no_universal_demographic_baseline`. Exact v30 results and provenance are recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md).""",
    )
    append_once(
        ROOT / "docs/research/resource-variability-v0.1.md",
        "## v30 condition-mortality-coupling re-verification",
        """## v30 condition-mortality-coupling re-verification

Audit-v4 AV4-006 / #497 reran the canonical 144-run M7.6 design on the exact production candidate. Exact point summaries change under corrected condition-mortality coupling, while all pre-existing qualitative synthetic-validation contrasts remain intact. Exact v30 results and provenance are recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md).""",
    )
    append_once(
        ROOT / "docs/research/m8-first-evidence-grounded-benchmark-result.md",
        "## v30 condition-mortality-coupling re-verification",
        """## v30 condition-mortality-coupling re-verification

Audit-v4 AV4-006 / #497 reran the full M8.6 benchmark under `anthrosim-model-semantics-v30`. Exact trajectories and paired effects changed, but the declared classification remains `fragile_spatial_structure`, with the same two fragile metrics and no robust metrics. Exact v30 results and provenance are recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md).""",
    )
    append_once(
        ROOT / "docs/research/m9-controlled-aggregation-benchmark-result.md",
        "## v30 condition-mortality-coupling re-verification",
        """## v30 condition-mortality-coupling re-verification

Audit-v4 AV4-006 / #497 reran the complete M9.7 applicable gate under `anthrosim-model-semantics-v30`; the preserved scientific reference matched exactly, including replay and active checkpoint/resume checks. No M9.7 machine rebaseline was required. Exact v30 provenance is recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md).""",
    )

    # Verify the materialized machine references against the archived review outputs.
    assert read_json(ROOT / "research/general-demography-baseline-v1/confirmatory-result.json") == expected304
    assert read_json(m7_path)["pointResults"] == point_results
    assert read_json(m8_path)["classification"] == m8_actual["classification"]


if __name__ == "__main__":
    main()
