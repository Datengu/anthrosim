#!/usr/bin/env python3
"""Verify an M9.7 aggregate against preserved scientific references."""
import argparse, json
from pathlib import Path


TRAVEL_KEYS = (
    "plannedRoundTripTravelDays",
    "observedTransitDays",
    "unrealizedPlannedTransitDays",
    "plannedRoundTripTravelCostUnits",
    "realizedTravelCostUnits",
    "unrealizedPlannedTravelCostUnits",
    "plannedRoundTripRouteDistanceEdges",
    "realizedRouteDistanceEdges",
    "unrealizedPlannedRouteDistanceEdges",
)


def read(path): return json.loads(Path(path).read_text(encoding="utf-8"))
def require(label, actual, expected):
    if actual != expected:
        raise SystemExit(f"M9.7 scientific regression mismatch for {label}: expected {expected!r}, found {actual!r}")


def project_legacy_pair(pair):
    continuous_keys=("residentPersonDays","visitorPersonDays","daysWithAnyVisitors","peakVisitors","permanentMigrations","conditionMortalityDeaths","journeysStarted","journeysCompleted")
    intermittent_keys=continuous_keys+("transitPersonDays","notStartedTotal","notStartedUnreachable","originCatchmentCells")
    intermittent={k:pair["intermittent"][k] for k in intermittent_keys}
    intermittent.update({
        "totalTravelDays": pair["intermittent"]["plannedRoundTripTravelDays"],
        "totalRoundTripTravelCostUnits": pair["intermittent"]["plannedRoundTripTravelCostUnits"],
        "totalRoundTripRouteDistanceEdges": pair["intermittent"]["plannedRoundTripRouteDistanceEdges"],
    })
    return {
        "seed": pair["seed"],
        "continuous": {k:pair["continuous"][k] for k in continuous_keys},
        "intermittent": intermittent,
        "totalFocalPersonDayDifferencePermilleExact": pair["totalFocalPersonDayDifferencePermilleExact"],
        "totalFocalPersonDayDifferencePermilleRounded": pair["totalFocalPersonDayDifferencePermilleRounded"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded"],
        "criteria": pair["criteria"],
        "pass": pair["pass"],
    }


def project_travel(summary):
    return {key: summary[key] for key in TRAVEL_KEYS}


def verify_travel_reconciliation(label, summary):
    require(
        f"{label}.planned transit reconciliation",
        summary["observedTransitDays"] + summary["unrealizedPlannedTransitDays"],
        summary["plannedRoundTripTravelDays"],
    )
    require(
        f"{label}.planned travel-cost reconciliation",
        summary["realizedTravelCostUnits"] + summary["unrealizedPlannedTravelCostUnits"],
        summary["plannedRoundTripTravelCostUnits"],
    )
    require(
        f"{label}.planned route-distance reconciliation",
        summary["realizedRouteDistanceEdges"] + summary["unrealizedPlannedRouteDistanceEdges"],
        summary["plannedRoundTripRouteDistanceEdges"],
    )


def verify(actual, reference, travel_reference):
    require("actual schemaVersion", actual.get("schemaVersion"), 3)
    require("legacy reference schemaVersion", reference.get("schemaVersion"), 2)
    for key in ("benchmarkId","definitionCanonicalSha256","declaredSeeds","classification","aggregate"):
        require(key, actual.get(key), reference.get(key))
    for arm in ("continuous","intermittent"):
        a=actual["arms"][arm]; r=reference["arms"][arm]
        # Experiment IDs intentionally include build/source identity and therefore change across
        # commits. Preserve them in the reference as provenance, but compare only scientific
        # configuration identity and authoritative terminal state digests here.
        for key in ("configCanonicalSha256","stateDigests"):
            require(f"{arm}.{key}", a.get(key), r.get(key))
    require("legacy pairs", [project_legacy_pair(p) for p in actual["pairs"]], reference["pairs"])

    require("travel reference schemaVersion", travel_reference.get("schemaVersion"), 1)
    require(
        "travel reference temporaryObservabilitySchemaVersion",
        travel_reference.get("temporaryObservabilitySchemaVersion"),
        2,
    )
    seeds=[str(seed) for seed in actual["declaredSeeds"]]
    require("travel reference seeds", list(travel_reference["intermittentBySeed"].keys()), seeds)
    by_seed={str(pair["seed"]): pair for pair in actual["pairs"]}
    for seed in seeds:
        pair=by_seed[seed]
        verify_travel_reconciliation(f"continuous seed {seed}", pair["continuous"])
        verify_travel_reconciliation(f"intermittent seed {seed}", pair["intermittent"])
        require(
            f"continuous seed {seed} travel burden",
            project_travel(pair["continuous"]),
            {key: 0 for key in TRAVEL_KEYS},
        )
        require(
            f"intermittent seed {seed} travel burden",
            project_travel(pair["intermittent"]),
            travel_reference["intermittentBySeed"][seed],
        )


def main():
    p=argparse.ArgumentParser(); p.add_argument("--actual",required=True,type=Path); p.add_argument("--reference",required=True,type=Path); args=p.parse_args()
    travel_reference=args.reference.with_name("travel-burden-reference.json")
    verify(read(args.actual),read(args.reference),read(travel_reference))
    print("M9.7 scientific regression baselines match preserved population and travel-burden observations")

if __name__=="__main__": main()
