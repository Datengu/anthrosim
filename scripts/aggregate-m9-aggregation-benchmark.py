#!/usr/bin/env python3
"""Aggregate the predeclared M9.7 continuous-vs-intermittent benchmark."""
from __future__ import annotations
import argparse, copy, hashlib, json
from fractions import Fraction
from pathlib import Path
from statistics import median

DURATION = "durationReached"
ARMS = ("continuous", "intermittent")


def read(path): return json.loads(Path(path).read_text(encoding="utf-8"))
def canon(value): return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()).hexdigest()
def fsha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def run_id(seed): return f"seed-{seed:020d}"
def frac_text(v): return f"{v.numerator}/{v.denominator}"
def rounded(v): return (v.numerator*2+v.denominator)//(2*v.denominator) if v>=0 else -rounded(-v)


def replay(pop, events, region, end):
    homes=[int(x) for x in pop["householdLocations"]]; living=[0]*len(homes)
    for h in pop["households"]: living[int(h)-1]+=1
    visiting=[False]*len(homes)
    residents=sum(n for n,home in zip(living,homes) if home in region); visitors=0; last=0
    resident_days=visitor_days=visitor_calendar_days=peak=permanent=condition_mortality=0
    def accrue(day):
        nonlocal last,resident_days,visitor_days,visitor_calendar_days
        if day<last: raise SystemExit("event replay moved backwards")
        delta=day-last; resident_days+=residents*delta; visitor_days+=visitors*delta
        if visitors>0: visitor_calendar_days+=delta
        last=day
    for rec in events.get("events",[]):
        day=int(rec["day"]); accrue(day); e=rec["event"]; kind=e["type"]
        if kind=="birth":
            h=int(e["household"])-1; living[h]+=1
            if homes[h] in region: residents+=1
            elif visiting[h]: visitors+=1
        elif kind=="death":
            h=int(e["household"])-1
            if homes[h] in region: residents-=1
            elif visiting[h]: visitors-=1
            living[h]-=1
            if e.get("cause")=="condition_mediated": condition_mortality+=1
        elif kind=="householdMigration":
            permanent+=1; h=int(e["household"])-1; was=homes[h] in region; homes[h]=int(e["destination"]); now=homes[h] in region
            if was!=now: residents += living[h] if now else -living[h]
        elif kind=="temporaryJourneyArrived":
            h=int(e["household"])-1
            if homes[h] in region or visiting[h]: raise SystemExit("invalid temporary arrival during replay")
            visiting[h]=True; visitors+=living[h]
        elif kind=="temporaryReturnDeparted":
            h=int(e["household"])-1
            if not visiting[h]: raise SystemExit("return departure without visitor presence")
            visitors-=living[h]; visiting[h]=False
        peak=max(peak,visitors)
    accrue(end)
    return {"residentPersonDays":resident_days,"visitorPersonDays":visitor_days,"totalFocalPersonDays":resident_days+visitor_days,"daysWithAnyVisitors":visitor_calendar_days,"peakVisitors":peak,"permanentMigrations":permanent,"conditionMortalityDeaths":condition_mortality}


def load_arm(root, arm, definition, config):
    base=Path(root)/arm; manifest_path=base/"experiment-manifest.json"; em=read(manifest_path)
    seeds=[int(x) for x in definition["seeds"]]; specs={int(x["experiment"]["seed"]):x for x in em["runs"]}
    if set(specs)!=set(seeds): raise SystemExit(f"{arm}: immutable seed set differs")
    region={int(x) for x in definition["focalRegion"]["memberCells"]}; runs={}
    for seed in seeds:
        rid=run_id(seed); status=read(base/"status"/f"{rid}.json"); spec=specs[seed]; d=base/spec["relativeRunDir"]
        if status.get("state")!="completed": raise SystemExit(f"{arm}/{rid}: not completed")
        for name in ("manifest.json","checkpoint.json","world.json","initial-population.json","events.json","temporary-observability.json"):
            if not (d/name).is_file(): raise SystemExit(f"{arm}/{rid}: missing {name}")
        man,cp,world,pop,events,report=[read(d/n) for n in ("manifest.json","checkpoint.json","world.json","initial-population.json","events.json","temporary-observability.json")]
        if man["experiment"]!=spec["experiment"] or man["experiment"].get("temporaryMobility")!=config: raise SystemExit(f"{arm}/{rid}: immutable experiment mismatch")
        if man.get("stopReason")!=DURATION or cp["events"]!=events: raise SystemExit(f"{arm}/{rid}: terminal/event integrity failure")
        src=report["source"]; s=report["summary"]; end=int(s["observationDurationDays"])
        if int(src["seed"])!=seed or int(src["runStateDigest64"])!=int(man["stateDigest64"]): raise SystemExit(f"{arm}/{rid}: observability provenance mismatch")
        if src["regionId"]!=definition["focalRegion"]["regionId"] or end!=int(definition["sharedSettings"]["years"])*365: raise SystemExit(f"{arm}/{rid}: focal region/duration mismatch")
        planned_days=int(s["plannedRoundTripTravelDays"]); observed_days=int(s["observedTransitDays"]); unrealized_days=int(s["unrealizedPlannedTransitDays"])
        planned_cost=int(s["plannedRoundTripTravelCostUnits"]); realized_cost=int(s["realizedTravelCostUnits"]); unrealized_cost=int(s["unrealizedPlannedTravelCostUnits"])
        planned_distance=int(s["plannedRoundTripRouteDistanceEdges"]); realized_distance=int(s["realizedRouteDistanceEdges"]); unrealized_distance=int(s["unrealizedPlannedRouteDistanceEdges"])
        if observed_days+unrealized_days!=planned_days: raise SystemExit(f"{arm}/{rid}: planned travel days do not reconcile")
        if realized_cost+unrealized_cost!=planned_cost: raise SystemExit(f"{arm}/{rid}: planned travel cost does not reconcile")
        if realized_distance+unrealized_distance!=planned_distance: raise SystemExit(f"{arm}/{rid}: planned route distance does not reconcile")
        r=replay(pop,events,region,end)
        if r["residentPersonDays"]!=int(s["focalRegionResidentPersonDays"]) or r["visitorPersonDays"]!=int(s["visitorPersonDays"]) or r["peakVisitors"]!=int(s["peakVisitors"]): raise SystemExit(f"{arm}/{rid}: authoritative replay disagrees with M9 report")
        origins=sum(int(x.get("journeysStarted",0))>0 for x in report.get("originCatchment",[]))
        runs[str(seed)]={"worldSha256":canon(world),"populationSha256":canon(pop),"regionIdentity":src["regionIdentity"],"travelModelIdentity":src.get("travelModelIdentity"),"stateDigest64":man["stateDigest64"],"summary":{**r,"journeysStarted":int(s["journeysStarted"]),"journeysCompleted":int(s["journeysCompleted"]),"arrivals":int(s["arrivals"]),"returnDepartures":int(s["returnDepartures"]),"transitPersonDays":int(s["transitPersonDays"]),"notStartedTotal":int(s["notStartedTotal"]),"notStartedUnreachable":int(s["notStartedUnreachable"]),"originCatchmentCells":origins,"plannedRoundTripTravelDays":planned_days,"observedTransitDays":observed_days,"unrealizedPlannedTransitDays":unrealized_days,"plannedRoundTripTravelCostUnits":planned_cost,"realizedTravelCostUnits":realized_cost,"unrealizedPlannedTravelCostUnits":unrealized_cost,"plannedRoundTripRouteDistanceEdges":planned_distance,"realizedRouteDistanceEdges":realized_distance,"unrealizedPlannedRouteDistanceEdges":unrealized_distance} }
    return {"experimentId":em["experimentId"],"experimentManifestSha256":fsha(manifest_path),"configSha256":canon(config),"runs":runs,"manifest":em}


def strip_schedule(exp):
    value=copy.deepcopy(exp); value["temporaryMobility"]["schedule"]="<arm-specific>"; return value


def pair(seed,c,t,definition):
    if c["worldSha256"]!=t["worldSha256"] or c["populationSha256"]!=t["populationSha256"] or c["regionIdentity"]!=t["regionIdentity"] or c["travelModelIdentity"]!=t["travelModelIdentity"]: raise SystemExit(f"seed {seed}: paired identity mismatch")
    cs,ts=c["summary"],t["summary"]; a=definition["predeclaredAcceptance"]; end=int(definition["sharedSettings"]["years"])*365
    resident_equal=cs["residentPersonDays"]==ts["residentPersonDays"]
    diff=abs(ts["totalFocalPersonDays"]-cs["totalFocalPersonDays"]); diff_pm=Fraction(diff*1000,cs["totalFocalPersonDays"])
    mean_res=Fraction(cs["residentPersonDays"],end)
    if mean_res == 0:
        raise SystemExit(f"seed {seed}: peak visitor share is undefined because continuous mean residents is zero")
    peak_pm=Fraction(ts["peakVisitors"]*1000,1)/mean_res
    checks={"pairedResidentPersonDaysEqual":resident_equal,"continuousJourneysStartedZero":cs["journeysStarted"]==0,"continuousVisitorPersonDaysZero":cs["visitorPersonDays"]==0,"continuousPeakVisitorsZero":cs["peakVisitors"]==0,"intermittentVisitorPersonDaysPositive":ts["visitorPersonDays"]>0,"intermittentJourneysCompletedPositive":ts["journeysCompleted"]>0,"intermittentDaysWithAnyVisitorsExact":ts["daysWithAnyVisitors"]==int(a["requireIntermittentDaysWithAnyVisitorsExact"]),"pairedFocalPersonDayDifferenceWithinBound":diff_pm<=int(a["maxPairedTotalFocalPersonDayDifferencePermille"]),"intermittentPeakVisitorShareAboveMinimum":peak_pm>=int(a["minIntermittentPeakVisitorShareOfContinuousMeanResidentsPermille"]),"noPermanentMigration":cs["permanentMigrations"]==0 and ts["permanentMigrations"]==0,"noConditionMortalityDeaths":cs["conditionMortalityDeaths"]==0 and ts["conditionMortalityDeaths"]==0,"intermittentOriginCatchmentNonempty":ts["originCatchmentCells"]>0,"intermittentTravelBurdenPositive":ts["plannedRoundTripTravelDays"]>0 and ts["observedTransitDays"]>0 and ts["plannedRoundTripTravelCostUnits"]>0 and ts["realizedTravelCostUnits"]>0 and ts["plannedRoundTripRouteDistanceEdges"]>0 and ts["realizedRouteDistanceEdges"]>0}
    return {"seed":seed,"continuous":cs,"intermittent":ts,"totalFocalPersonDayDifferencePermilleExact":frac_text(diff_pm),"totalFocalPersonDayDifferencePermilleRounded":rounded(diff_pm),"intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact":frac_text(peak_pm),"intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded":rounded(peak_pm),"criteria":checks,"pass":all(checks.values())}


def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--root",required=True,type=Path); ap.add_argument("--definition",required=True,type=Path); ap.add_argument("--output",required=True,type=Path); ap.add_argument("--markdown",type=Path); args=ap.parse_args()
    definition=read(args.definition); configs={"continuous":read(definition["arms"]["continuousResidence"]["temporaryMobilityFile"]),"intermittent":read(definition["arms"]["intermittentAggregation"]["temporaryMobilityFile"])}
    if configs["continuous"]["region"]!=configs["intermittent"]["region"] or configs["continuous"]["travelModel"]!=configs["intermittent"]["travelModel"]: raise SystemExit("arms must share focal region and travel model")
    arms={arm:load_arm(args.root,arm,definition,configs[arm]) for arm in ARMS}; seeds=[int(x) for x in definition["seeds"]]
    for seed in seeds:
        cm=arms["continuous"]["manifest"]; im=arms["intermittent"]["manifest"]
        ce=next(x["experiment"] for x in cm["runs"] if int(x["experiment"]["seed"])==seed); ie=next(x["experiment"] for x in im["runs"] if int(x["experiment"]["seed"])==seed)
        if strip_schedule(ce)!=strip_schedule(ie): raise SystemExit(f"seed {seed}: paired experiments differ outside schedule")
    pairs=[pair(seed,arms["continuous"]["runs"][str(seed)],arms["intermittent"]["runs"][str(seed)],definition) for seed in seeds]
    structure_missing=any(not p["criteria"]["intermittentVisitorPersonDaysPositive"] or not p["criteria"]["intermittentPeakVisitorShareAboveMinimum"] for p in pairs); match_failed=any(not p["criteria"]["pairedFocalPersonDayDifferenceWithinBound"] for p in pairs); all_pass=all(p["pass"] for p in pairs)
    klass="not_distinguished" if structure_missing else "near_match_failed" if match_failed else "capability_distinguished" if all_pass else "degenerate"
    diffs=[Fraction(p["totalFocalPersonDayDifferencePermilleExact"]) for p in pairs]; peaks=[Fraction(p["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact"]) for p in pairs]
    med_diff=Fraction(median(diffs)); med_peak=Fraction(median(peaks))
    result={"schemaVersion":3,"benchmarkId":definition["benchmarkId"],"scientificStatus":definition["scientificStatus"],"interpretationBoundary":definition["interpretationBoundary"],"definitionCanonicalSha256":canon(definition),"declaredSeeds":seeds,"sharedSettings":definition["sharedSettings"],"arms":{arm:{"experimentId":arms[arm]["experimentId"],"experimentManifestSha256":arms[arm]["experimentManifestSha256"],"configCanonicalSha256":arms[arm]["configSha256"],"stateDigests":{str(seed):arms[arm]["runs"][str(seed)]["stateDigest64"] for seed in seeds}} for arm in ARMS},"pairs":pairs,"aggregate":{"pairedSeedsPassing":sum(p["pass"] for p in pairs),"pairedSeedsTotal":len(pairs),"medianTotalFocalPersonDayDifferencePermilleExact":frac_text(med_diff),"medianTotalFocalPersonDayDifferencePermilleRounded":rounded(med_diff),"maximumTotalFocalPersonDayDifferencePermilleRounded":max(rounded(x) for x in diffs),"medianIntermittentPeakVisitorSharePermilleExact":frac_text(med_peak),"medianIntermittentPeakVisitorSharePermilleRounded":rounded(med_peak),"minimumIntermittentPeakVisitorSharePermilleRounded":min(rounded(x) for x in peaks)},"classification":{"benchmarkClass":klass,"allPredeclaredPairedCriteriaPassed":all_pass,"replayAndResumeChecks":"workflow-gated-separately"}}
    result["aggregateCanonicalSha256"]=canon(result); args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text(json.dumps(result,indent=2)+"\n")
    if args.markdown:
        lines=["# M9.7 controlled aggregation benchmark result","",f"Benchmark class: **{klass}**.","",definition["interpretationBoundary"],"","| Seed | Focal person-day difference | Peak visitor share | Visitor days | Completed journeys | Pass |","| ---: | ---: | ---: | ---: | ---: | :---: |"]
        for p in pairs: lines.append(f"| {p['seed']} | {p['totalFocalPersonDayDifferencePermilleRounded']/10:.1f}% | {p['intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded']/10:.1f}% | {p['intermittent']['daysWithAnyVisitors']} | {p['intermittent']['journeysCompleted']} | {'yes' if p['pass'] else 'no'} |")
        lines += ["",f"Paired seeds passing: {sum(p['pass'] for p in pairs)}/{len(pairs)}.",f"Maximum focal person-day difference: {result['aggregate']['maximumTotalFocalPersonDayDifferencePermilleRounded']/10:.1f}%.",f"Minimum peak visitor share: {result['aggregate']['minimumIntermittentPeakVisitorSharePermilleRounded']/10:.1f}%.","","Replay and active-checkpoint/resume equivalence are gated separately by the workflow.",""]
        args.markdown.parent.mkdir(parents=True,exist_ok=True); args.markdown.write_text("\n".join(lines),encoding="utf-8")

if __name__=="__main__": main()
