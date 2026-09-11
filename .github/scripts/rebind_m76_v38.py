import json
from pathlib import Path

reference_path = Path("experiments/v0.1-resource-variability-reference.json")
root = Path("/tmp/m76")
reference = json.loads(reference_path.read_text())
points = json.loads((root / "analysis/points.json").read_text())
runs = json.loads((root / "analysis/runs.json").read_text())
summary = json.loads((root / "analysis/summary.json").read_text())
manifest = json.loads((root / "sweep-manifest.json").read_text())
record = json.loads((root / "reproduction-record.json").read_text())

assert reference["schemaVersion"] == 3
assert reference["modelSemanticsId"] == "anthrosim-model-semantics-v37"
assert reference["definitionSha256"] == "3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e"
assert len(manifest["points"]) == 18
assert len(runs) == 144
assert len(points) == 18
assert all(row["state"] == "completed" for row in runs)
assert all(row["completedRuns"] == 8 for row in points)
assert all(row["failedRuns"] == 0 for row in points)
assert all(row["incompleteRuns"] == 0 for row in points)
assert all(row["personRecordLimitReachedRuns"] == 0 for row in points)
assert all(row["scientificallyEligibleRuns"] == 8 for row in points)
assert all(row["operationallyCensoredRuns"] == 0 for row in points)
assert all(row["scientificAggregationStatus"] == "eligibleScientificOutcome" for row in runs)
assert summary["schemaVersion"] == 6
assert summary["runRows"] == 144
assert summary["pointRows"] == 18
assert summary["completedRuns"] == 144
assert summary["nonCompletedRuns"] == 0
assert summary["scientificallyEligibleRuns"] == 144
assert summary["operationallyCensoredRuns"] == 0
assert record["sweepId"] == manifest["sweepId"]
assert record["definitionSha256"] == reference["definitionSha256"]
assert record["modelVersion"] == "0.3.6"

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

# Fail closed: AV6-006 is M9-only, so the pre-M9 M7.6 factorial must reproduce
# every frozen scientific point exactly. No numerical refresh is permitted here.
assert observed == reference["pointResults"], (
    "AV6-006 v38 changed M7.6 numerical point results; provenance-only rebind refused"
)

reference["modelSemanticsId"] = "anthrosim-model-semantics-v38"
reference["note"] += (
    " Audit-v6 AV6-006/#711 changes only M9 equal-cost temporary-destination spatial coupling, "
    "while this M7.6 resource/migration factorial contains no temporary-mobility mechanism. "
    "Exact-head CI run 34557084821 (production candidate 138d489bcb63d8080858ac203e0d8897fb874f0b; "
    "PR merge-ref f775c66651d3152210fc09582fae5752eb0767ea; derived artifact 10183267124, "
    "SHA-256 2d9ae421993cf116fe6141e503f8a3a285b3d54f9d574ef683e43e3524a10a2e) reran all "
    "18 parameter points and 144/144 declared simulations under living v38. Every run completed, "
    "remained scientifically eligible and reproduced every frozen v37 pointResults value exactly. "
    "Accordingly this is a provenance-only v38 rebind: no M7.6 numerical endpoint, experimental "
    "coordinate, completion/extinction classification, eligibility rule or accounting guard is rebaselined."
)
reference_path.write_text(json.dumps(reference, separators=(",", ":")) + "\n")
print("M7.6 v38 provenance-only rebind validated: 18/18 point results exactly unchanged")
