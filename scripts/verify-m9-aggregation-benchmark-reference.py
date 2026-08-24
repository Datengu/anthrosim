#!/usr/bin/env python3
"""Verify an M9.7 aggregate against the preserved first-observation reference."""
import argparse, json
from pathlib import Path


def read(path): return json.loads(Path(path).read_text(encoding="utf-8"))
def require(label, actual, expected):
    if actual != expected:
        raise SystemExit(f"M9.7 scientific regression mismatch for {label}: expected {expected!r}, found {actual!r}")


def project_pair(pair):
    continuous_keys=("residentPersonDays","visitorPersonDays","daysWithAnyVisitors","peakVisitors","permanentMigrations","resourceScarcityDeaths","journeysStarted","journeysCompleted")
    intermittent_keys=continuous_keys+("transitPersonDays","notStartedTotal","notStartedUnreachable","originCatchmentCells","totalTravelDays","totalRoundTripTravelCostUnits","totalRoundTripRouteDistanceEdges")
    return {
        "seed": pair["seed"],
        "continuous": {k:pair["continuous"][k] for k in continuous_keys},
        "intermittent": {k:pair["intermittent"][k] for k in intermittent_keys},
        "totalFocalPersonDayDifferencePermilleExact": pair["totalFocalPersonDayDifferencePermilleExact"],
        "totalFocalPersonDayDifferencePermilleRounded": pair["totalFocalPersonDayDifferencePermilleRounded"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded"],
        "criteria": pair["criteria"],
        "pass": pair["pass"],
    }


def verify(actual, reference):
    for key in ("schemaVersion","benchmarkId","definitionCanonicalSha256","declaredSeeds","classification","aggregate"):
        require(key, actual.get(key), reference.get(key))
    for arm in ("continuous","intermittent"):
        a=actual["arms"][arm]; r=reference["arms"][arm]
        # Experiment IDs intentionally include build/source identity and therefore change across
        # commits. Preserve them in the reference as provenance, but compare only scientific
        # configuration identity and authoritative terminal state digests here.
        for key in ("configCanonicalSha256","stateDigests"):
            require(f"{arm}.{key}", a.get(key), r.get(key))
    require("pairs", [project_pair(p) for p in actual["pairs"]], reference["pairs"])


def main():
    p=argparse.ArgumentParser(); p.add_argument("--actual",required=True,type=Path); p.add_argument("--reference",required=True,type=Path); args=p.parse_args()
    verify(read(args.actual),read(args.reference))
    print("M9.7 scientific regression baseline matches preserved first observation")

if __name__=="__main__": main()
