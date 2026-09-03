#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

BRANCH_HEAD = "f8c806a811d104c17f32864dd4abee2d123cfa2d"
RUN_ID = 33816687527
JOB_ID = 100852554519
ARTIFACT_ID = 9917067903
ARTIFACT_SHA = "202a464f058a80e7c1b788a5de9bd2f7ab30434a747524f0d7df66fcf7174f48"
DEFINITION_SHA = "3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e"

root = Path("/tmp/m7-v29")
points = json.loads((root / "analysis/points.json").read_text(encoding="utf-8"))
summary = json.loads((root / "analysis/summary.json").read_text(encoding="utf-8"))
record = json.loads((root / "reproduction-record.json").read_text(encoding="utf-8"))
ref_path = Path("experiments/v0.1-resource-variability-reference.json")
old = json.loads(ref_path.read_text(encoding="utf-8"))

assert old["schemaVersion"] == 3
assert old["modelSemanticsId"] == "anthrosim-model-semantics-v28"
assert old["definitionSha256"] == DEFINITION_SHA
assert len(points) == 18
assert summary["completedRuns"] == 144
assert summary["nonCompletedRuns"] == 0
assert summary["scientificallyEligibleRuns"] == 144
assert summary["operationallyCensoredRuns"] == 0
assert record["definitionSha256"] == DEFINITION_SHA


def canonical_point(row: dict) -> dict:
    return {
        "pointId": row["pointId"],
        "resourceProductivityScalePermille": row["resourceProductivityScalePermille"],
        "resourceSeasonalityScalePermille": row["resourceSeasonalityScalePermille"],
        "migrationEnabled": not row["disableMigration"],
        "durationReachedRuns": row["durationReachedRuns"],
        "populationExtinctRuns": row["populationExtinctRuns"],
        "meanFinalLivingPopulationCompletedOnly": row["meanFinalLivingPopulationScientificallyEligibleOnly"],
        "meanFinalLivingOccupiedCellCountCompletedOnly": row["meanFinalLivingOccupiedCellCountScientificallyEligibleOnly"],
        "meanLivingConditionPermilleCompletedOnly": row["meanLivingConditionPermilleScientificallyEligibleOnly"],
        "meanConditionMortalityDeathsCompletedOnly": row["meanConditionMortalityDeathsScientificallyEligibleOnly"],
        "meanResourceUnmetNeedCompletedOnly": row["meanResourceUnmetNeedScientificallyEligibleOnly"],
        "meanMigrationMovesCompletedOnly": row["meanMigrationMovesScientificallyEligibleOnly"],
        "meanMigrationTotalDistanceCellsCompletedOnly": row["meanMigrationTotalDistanceCellsScientificallyEligibleOnly"],
        "pooledMeanMigrationDistanceCellsPerMoveCompletedOnly": row["pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly"],
    }


point_results = [canonical_point(row) for row in points]
old_by_id = {row["pointId"]: row for row in old["pointResults"]}
disabled = [row for row in point_results if not row["migrationEnabled"]]
enabled = [row for row in point_results if row["migrationEnabled"]]
assert len(disabled) == len(enabled) == 9
assert all(row == old_by_id[row["pointId"]] for row in disabled)
assert all(row != old_by_id[row["pointId"]] for row in enabled)

# Require the predeclared qualitative contrasts to survive the semantics change.
by_key = {
    (
        row["resourceProductivityScalePermille"],
        row["resourceSeasonalityScalePermille"],
        row["migrationEnabled"],
    ): row
    for row in point_results
}
for productivity in (250, 500, 1000):
    for seasonality in (0, 500, 1000):
        on = by_key[(productivity, seasonality, True)]
        off = by_key[(productivity, seasonality, False)]
        assert on["meanFinalLivingPopulationCompletedOnly"] > off["meanFinalLivingPopulationCompletedOnly"]
        assert on["meanConditionMortalityDeathsCompletedOnly"] < off["meanConditionMortalityDeathsCompletedOnly"]
        assert on["meanResourceUnmetNeedCompletedOnly"] < off["meanResourceUnmetNeedCompletedOnly"]
        if productivity == 250:
            assert on["durationReachedRuns"] == 8 and on["populationExtinctRuns"] == 0
            assert off["durationReachedRuns"] == 0 and off["populationExtinctRuns"] == 8

reference = {
    "schemaVersion": 3,
    "provenance": "derived_reference_snapshot",
    "scientificStatus": "synthetic_validation",
    "definitionId": "v0.1_resource_variability_synthetic_validation",
    "definitionSha256": DEFINITION_SHA,
    "modelVersion": "0.3.4",
    "modelSemanticsId": "anthrosim-model-semantics-v29",
    "referenceCiRunId": RUN_ID,
    "referenceTestMergeCommit": None,
    "referenceHeadCommit": BRANCH_HEAD,
    "referenceSweepId": record["sweepId"],
    "plannedRuns": 144,
    "completedRuns": 144,
    "nonCompletedRuns": 0,
    "pointResults": point_results,
    "note": (
        "Derived reference summary deliberately rebaselined for model semantics v29 after Audit-v4 "
        "AV4-005 / #495 replaced arbitrary canonical male PersonId/record ordering in M2 parentage "
        "RNG assignment with persistent person stochastic-coupling-rank ordering. The source definition, "
        "18-point factorial design, paired seeds, completion/censoring rules, migration configuration, "
        "resource-response semantics, and declared endpoints are unchanged. All 144 runs completed and "
        "remained scientifically eligible. All 9/9 migration-disabled point summaries remain numerically "
        "identical to v28, while all 9/9 migration-enabled point summaries change, a diagnostic pattern "
        "consistent with corrected genealogy coupling propagating through residence/migration histories. "
        "The low-productivity no-migration arms still go extinct in all 8/8 seeds, migration-enabled arms "
        "still persist, and at every matched productivity/seasonality point migration-enabled runs retain "
        "higher terminal population with lower condition-mediated mortality and lower unmet resource need "
        "than migration-disabled controls. This snapshot freezes synthetic verification outputs only; it is "
        "not calibration evidence or an empirical anthropological claim."
    ),
    "referenceTestMergeCommitStatus": "pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite",
}
ref_path.write_text(json.dumps(reference, separators=(",", ":")) + "\n", encoding="utf-8")

ci_path = Path(".github/workflows/ci.yml")
ci = ci_path.read_text(encoding="utf-8")
needle = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v28'"
assert ci.count(needle) == 1
ci_path.write_text(
    ci.replace(
        needle,
        "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v29'",
        1,
    ),
    encoding="utf-8",
)


def num(value) -> str:
    if value is None:
        return "—"
    if isinstance(value, float) and value.is_integer():
        return str(int(value))
    return str(value)


rows = [
    "| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |",
    "| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |",
]
for row in point_results:
    distance = row["pooledMeanMigrationDistanceCellsPerMoveCompletedOnly"]
    rows.append(
        f'| {row["resourceProductivityScalePermille"]} | {row["resourceSeasonalityScalePermille"]} | '
        f'{"on" if row["migrationEnabled"] else "off"} | '
        f'{row["durationReachedRuns"]} duration / {row["populationExtinctRuns"]} extinct | '
        f'{num(row["meanFinalLivingPopulationCompletedOnly"])} | '
        f'{num(row["meanFinalLivingOccupiedCellCountCompletedOnly"])} | '
        f'{num(row["meanConditionMortalityDeathsCompletedOnly"])} | '
        f'{num(row["meanResourceUnmetNeedCompletedOnly"])} | '
        f'{num(row["meanMigrationMovesCompletedOnly"])} | '
        f'{"—" if distance is None else f"{distance:.3f}"} |'
    )
table = "\n".join(rows)

doc_path = Path("docs/research/resource-variability-v0.1.md")
doc = doc_path.read_text(encoding="utf-8")
start = doc.index("## Current v28 reference and provenance")
historical = doc.index("## Historical v27 reference and provenance")
v28 = doc[start:historical]
v28 = v28.replace(
    "## Current v28 reference and provenance",
    "## Historical v28 reference and provenance",
    1,
)
v28 = v28.replace("## Current v28 point results", "## Historical v28 point results", 1)
v28 = v28.replace("## Current v28 interpretation", "## Historical v28 interpretation", 1)
current = f"""## Current v29 reference and provenance

Audit-v4 AV4-005 / #495 changes causal same-seed M2 parentage assignment: eligible residence-local males are no longer coupled to `demography/parentage` draws through arbitrary canonical `PersonId`/record order. Because genealogy can propagate into kin eligibility, household composition and migration histories, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v28 values.

Reviewed v29 execution:

- CI run: `{RUN_ID}`;
- M7.6 job: `{JOB_ID}`;
- exact production head: `{BRANCH_HEAD}`;
- artifact: `{ARTIFACT_ID}` (`m7-6-resource-variability-derived`, `sha256:{ARTIFACT_SHA}`);
- definition SHA-256: `{DEFINITION_SHA}`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v29`;
- sweep ID: `{record['sweepId']}`.

All **144/144** planned runs completed and were scientifically eligible, with no failed, incomplete, record-limit or otherwise operationally censored runs. The exact-head workflow failed only at its final equality assertion against the then-current frozen v28 point-results reference; the complete derived artifact was archived successfully.

The on/off control is again diagnostic: all **9/9 migration-disabled point summaries are numerically identical to v28**, while all **9/9 migration-enabled point summaries change**. The source definition, 18-point factorial design, paired seeds, M3 resource settings, completion/censoring rules and declared endpoints are unchanged. This pattern is consistent with corrected parentage/genealogy coupling becoming consequential when residence and household histories can diverge through M4 migration, rather than unexplained resource-process drift.

The reviewed v29 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls.

## Current v29 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

{table}

Full unrounded point values are preserved in the machine-readable reference.

## Current v29 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while exact migration-enabled trajectories now reflect both HouseholdId-invariant M4 scheduling and PersonId-invariant M2 parentage stochastic coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison still does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v29 rebaseline preserves the experiment design and scientific question while recording the expected downstream consequences of corrected parentage stochastic coupling. It does not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

"""
doc_path.write_text(doc[:start] + current + v28 + doc[historical:], encoding="utf-8")

rever_path = Path("research/general-demography-baseline-v1/model-semantics-v29-reverification.md")
rever = rever_path.read_text(encoding="utf-8")
marker = "These are reviewed upstream-semantics rebaselines, not new empirical validation."
section = f"""### M7.6 resource-variability reference

Reviewed exact-head central CI run `{RUN_ID}`, job `{JOB_ID}`, artifact `{ARTIFACT_ID}`, SHA-256 `{ARTIFACT_SHA}`. All 144/144 runs completed and were scientifically eligible. All 9/9 migration-disabled point summaries are exactly unchanged from v28, while all 9/9 migration-enabled summaries change; the predeclared persistence/resource contrasts remain intact. Sweep ID `{record['sweepId']}`. The checked reference therefore advances deliberately to v29 rather than treating the expected genealogy/migration propagation as a regression.

"""
if "### M7.6 resource-variability reference" not in rever:
    assert marker in rever
    rever = rever.replace(marker, section + marker, 1)
rever_path.write_text(rever, encoding="utf-8")

checked = json.loads(ref_path.read_text(encoding="utf-8"))
assert checked["modelSemanticsId"] == "anthrosim-model-semantics-v29"
assert checked["pointResults"] == point_results
assert checked["referenceSweepId"] == record["sweepId"]
