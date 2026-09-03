import json
from pathlib import Path

ARTIFACT_ID = 9907214330
ARTIFACT_SHA = "c43a3a920e8c2ac440beb793c448fd39ecd81f1ca23e3ec934e318901f1eea82"
EVIDENCE_HEAD = "0442c7d8a42402713195b24420f722a5d7226392"
CI_RUN = 33789214317
CI_JOB = 100766028843
NEW_SEMANTICS = "anthrosim-model-semantics-v27"

root = Path("/tmp/m7-v27")
points = json.loads((root / "analysis/points.json").read_text())
summary = json.loads((root / "analysis/summary.json").read_text())
manifest = json.loads((root / "sweep-manifest.json").read_text())
record = json.loads((root / "reproduction-record.json").read_text())

assert len(points) == 18
assert summary["completedRuns"] == 144
assert summary["nonCompletedRuns"] == 0
assert summary["scientificallyEligibleRuns"] == 144
assert summary["operationallyCensoredRuns"] == 0
assert all(p["completedRuns"] == 8 for p in points)
assert all(p["failedRuns"] == 0 and p["incompleteRuns"] == 0 for p in points)
assert all(p["scientificallyEligibleRuns"] == 8 and p["operationallyCensoredRuns"] == 0 for p in points)
assert record["definitionSha256"] == "3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e"
assert record["sweepId"] == manifest["sweepId"]

for i in range(0, 18, 2):
    on = points[i]
    off = points[i + 1]
    assert not on["disableMigration"] and off["disableMigration"]
    assert on["resourceProductivityScalePermille"] == off["resourceProductivityScalePermille"]
    assert on["resourceSeasonalityScalePermille"] == off["resourceSeasonalityScalePermille"]
    assert on["meanFinalLivingPopulationScientificallyEligibleOnly"] > off["meanFinalLivingPopulationScientificallyEligibleOnly"]
    assert on["meanConditionMortalityDeathsScientificallyEligibleOnly"] < off["meanConditionMortalityDeathsScientificallyEligibleOnly"]
    assert on["meanResourceUnmetNeedScientificallyEligibleOnly"] < off["meanResourceUnmetNeedScientificallyEligibleOnly"]
    if on["resourceProductivityScalePermille"] == 250:
        assert on["durationReachedRuns"] == 8 and on["populationExtinctRuns"] == 0
        assert off["durationReachedRuns"] == 0 and off["populationExtinctRuns"] == 8

observed = []
for row in points:
    observed.append({
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
    })

ref_path = Path("experiments/v0.1-resource-variability-reference.json")
old = json.loads(ref_path.read_text())
assert old["schemaVersion"] == 3
assert old["modelSemanticsId"] == "anthrosim-model-semantics-v26"
assert old["definitionSha256"] == record["definitionSha256"]
new = dict(old)
new.update({
    "modelSemanticsId": NEW_SEMANTICS,
    "referenceCiRunId": CI_RUN,
    "referenceTestMergeCommit": None,
    "referenceHeadCommit": EVIDENCE_HEAD,
    "referenceSweepId": record["sweepId"],
    "plannedRuns": 144,
    "completedRuns": 144,
    "nonCompletedRuns": 0,
    "pointResults": observed,
    "note": (
        "Derived reference summary deliberately rebaselined for model semantics v27 after Audit-v4 AV4-002 / #488 "
        "replaced arbitrary PersonId-ordered background-mortality draw assignment with persistent scientifically canonical "
        "stochastic coupling ranks. The source definition, 18-point factorial design, paired seeds, completion/censoring rules, "
        "migration configuration, resource-response semantics, and declared endpoints are unchanged. All 144 runs completed "
        "and remained scientifically eligible. The low-productivity no-migration arms still go extinct in all 8/8 seeds, "
        "migration-enabled arms still persist, and at every matched productivity/seasonality point migration-enabled runs "
        "retain higher terminal population with lower condition-mediated mortality and lower unmet resource need than "
        "migration-disabled controls. Quantitative values change as the expected downstream consequence of the corrected "
        "background-mortality coupling. This snapshot freezes synthetic verification outputs only; it is not calibration "
        "evidence or an empirical anthropological claim."
    ),
})
ref_path.write_text(json.dumps(new, separators=(",", ":")) + "\n")

ci_path = Path(".github/workflows/ci.yml")
ci = ci_path.read_text()
old_assert = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v26'"
new_assert = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v27'"
assert ci.count(old_assert) == 1
ci_path.write_text(ci.replace(old_assert, new_assert))

def show(v):
    if v is None:
        return "—"
    if isinstance(v, float) and v.is_integer():
        return str(int(v))
    return str(v)

rows = []
for p in points:
    move_dist = p["pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly"]
    rows.append(
        f"| {p['resourceProductivityScalePermille']} | {p['resourceSeasonalityScalePermille']} | "
        f"{'off' if p['disableMigration'] else 'on'} | {p['durationReachedRuns']} duration / {p['populationExtinctRuns']} extinct | "
        f"{show(p['meanFinalLivingPopulationScientificallyEligibleOnly'])} | "
        f"{show(p['meanFinalLivingOccupiedCellCountScientificallyEligibleOnly'])} | "
        f"{show(p['meanConditionMortalityDeathsScientificallyEligibleOnly'])} | "
        f"{show(p['meanResourceUnmetNeedScientificallyEligibleOnly'])} | "
        f"{show(p['meanMigrationMovesScientificallyEligibleOnly'])} | "
        f"{'—' if move_dist is None else f'{move_dist:.3f}'} |"
    )

current = f'''## Current v27 reference and provenance

Audit-v4 AV4-002 / #488 changes the causal same-seed background-mortality coupling: background mortality draws are no longer assigned by arbitrary canonical `PersonId` record order. Because that correction can propagate through deaths, household composition, resources, fertility and migration, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v26 values.

Reviewed v27 execution:

- CI run: `{CI_RUN}`;
- M7.6 job: `{CI_JOB}`;
- exact production head used by the archived run: `{EVIDENCE_HEAD}`;
- artifact: `{ARTIFACT_ID}` (`m7-6-resource-variability-derived`, `sha256:{ARTIFACT_SHA}`);
- definition SHA-256: `{record['definitionSha256']}`;
- model version: `{record['modelVersion']}`;
- model semantics: `{NEW_SEMANTICS}`;
- sweep ID: `{record['sweepId']}`.

All **144/144** planned runs completed and were scientifically eligible, with no operational censoring. The workflow job failed only after execution when the then-current CI assertion compared the generated v27 point summaries with the frozen v26 machine reference; that stale-reference assertion is not scientific evidence against the v27 execution.

The source definition is unchanged: 18 factorial points × 8 paired seeds. The reviewed v27 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls. Quantitative values move because corrected background-mortality coupling changes downstream trajectories.

## Current v27 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(rows)}

Full unrounded point values are preserved in the machine-readable reference.

## Current v27 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while the exact trajectory-level values are now those generated under the corrected v27 background-mortality coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison still does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v27 rebaseline preserves the experiment design and scientific question while recording the expected causal consequences of correcting arbitrary background-mortality draw assignment. It does not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

'''

doc_path = Path("docs/research/resource-variability-v0.1.md")
doc = doc_path.read_text()
marker = "## Current v26 reference and provenance\n"
assert doc.count(marker) == 1
doc = doc.replace(marker, "## Historical v26 reference and provenance\n", 1)
doc = doc.replace("## Current v26 point results\n", "## Historical v26 point results\n", 1)
doc = doc.replace("## Current v26 interpretation\n", "## Historical v26 interpretation\n", 1)
insert_at = doc.index("## Historical v26 reference and provenance\n")
doc_path.write_text(doc[:insert_at] + current + doc[insert_at:])

check = json.loads(ref_path.read_text())
assert check["modelSemanticsId"] == NEW_SEMANTICS
assert check["referenceCiRunId"] == CI_RUN
assert check["referenceHeadCommit"] == EVIDENCE_HEAD
assert check["referenceSweepId"] == "anthrosim-sweep-v2-71270161787bc7ca"
assert len(check["pointResults"]) == 18
assert new_assert in ci_path.read_text()
assert "## Current v27 reference and provenance" in doc_path.read_text()

Path(".github/workflows/tmp-av4-002-m7-rebaseline.yml").unlink()
Path("scripts/tmp_rebaseline_m7_v27.py").unlink()
