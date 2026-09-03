import json
import shutil
from pathlib import Path

BRANCH_HEAD = "14b5290525f97c6404432e53fd91af5760f400cc"
M8_RUN = 33785449208
M8_ARTIFACT = 9905356901
M8_ARTIFACT_SHA = "f7a8fe383d05826cb1ea52d7c5c0721381c55eddb7eb1853da5535e4448d6d41"
M9_ARTIFACT = 9905367753
M9_ARTIFACT_SHA = "350c6344ec721d201ee4e528de9daec9c5136faa0d6bf90727f5ec24a17c5020"
A304_RUN = 33785448601
A304_JOB = 100748979710
A304_ARTIFACT = 9905391271
A304_ARTIFACT_SHA = "9c8c2cc4f661377e2dca151147bb4c98d146a5668bc8d7c8de48c26ab381be96"
PRIVACY_REF = "pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite"


def read_json(path: str):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def compact_write(path: str, value):
    Path(path).write_text(json.dumps(value, separators=(",", ":")) + "\n", encoding="utf-8")


# Issue #304: use the exact workflow-generated expected result rather than transcribing values.
a304 = read_json("/tmp/a304/expected-result.json")
assert a304["modelSemanticsId"] == "anthrosim-model-semantics-v27"
assert a304["runCount"] == 780
assert a304["recommendation"] == "no_universal_demographic_baseline"
assert all(v["decision"] == "sufficient_stop" for v in a304["monteCarloPrecision"].values())
assert a304["longRun"]["researchGateStatus"] == "failed"
assert a304["longRun"]["multipleStableRegimesDetected"] is True
assert a304["longRun"]["environmentDependenceDetected"] is False
assert a304["longRun"]["initializationDependenceDetected"] is False
assert a304["longRun"]["primaryClassificationCounts"] == {
    "drifting": 747,
    "insufficient_data": 25,
    "stable": 8,
}
paired = {v["demography"]: v for v in a304["pairedHouseholdEffects"]}
assert paired["negative_growth_control_v1"]["fissionMinusFixedTerminalPopulation"]["mean"] < 0
assert paired["positive_growth_control_v1"]["fissionMinusFixedTerminalPopulation"]["mean"] < 0
assert paired["replacement_control_v1"]["fissionMinusFixedTerminalPopulation"]["mean"] < 0
assert all(v["fissionMinusFixedLateGrowthRatePerYear"]["mean"] < 0 for v in paired.values())
assert all(v["fissionMinusFixedMateLimitationFraction"]["mean"] > 0 for v in paired.values())
shutil.copyfile(
    "/tmp/a304/expected-result.json",
    "research/general-demography-baseline-v1/confirmatory-result.json",
)

Path("research/general-demography-baseline-v1/model-semantics-v27-reverification.md").write_text(
    """# Model-semantics v27 reverification

Audit-v4 AV4-002 / issue #488 changes the causal same-seed assignment of background-demographic mortality RNG draws by replacing incidental `PersonId` record ordering with the persisted scientific stochastic-coupling rank introduced by AV4-001. Condition-mediated mortality remains on its pre-existing ordering for its separate AV4-006/#497 finding. Because AV4-002 changes deterministic continuation semantics, the current model advances from `anthrosim-model-semantics-v26` to `anthrosim-model-semantics-v27` and the frozen issue #304 demographic baseline must be scientifically re-executed rather than relabelled.

Issue-304 workflow run `33785448601`, job `100748979710`, re-executed the unchanged confirmatory design under v27. Artifact `9905391271` (`issue-304-demographic-baseline-confirmatory`, SHA-256 `9c8c2cc4f661377e2dca151147bb4c98d146a5668bc8d7c8de48c26ab381be96`) contains the exact workflow-generated `expected-result.json` copied into the canonical `confirmatory-result.json`.

Reverification outcome:

- all **780/780** declared runs completed;
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`;
- the recommendation remains **`no_universal_demographic_baseline`**;
- long-run analysis still rejects a universal stable regime (`researchGateStatus = failed`) and still detects multiple stable regimes;
- environment dependence and initialization dependence remain undetected in this design;
- for every demographic schedule, dependency-aware fission still lowers terminal population and late realized growth and increases mate limitation relative to the fixed-founder control.

Representative movement from the v26 reference to v27 is expected because mortality draws are now coupled to scientifically invariant person identities rather than arbitrary founder labels:

- negative-growth fission-minus-fixed mean terminal population: `-10.1308` -> `-11.0077` people;
- positive-growth fission-minus-fixed mean terminal population: `-89.1077` -> `-91.1231` people;
- replacement fission-minus-fixed mean terminal population: `-40.9` -> `-36.5923` people;
- positive-growth fixed-founder mean late growth rate/year moves from approximately `-0.00025668` to `-0.00038166`;
- primary long-run classifications move from `drifting=743, insufficient_data=31, stable=6` to `drifting=747, insufficient_data=25, stable=8`;
- stochastic multi-regime treatment contexts move from 2 to 1 while `multipleStableRegimesDetected` remains `true`.

This is therefore a **causal v27 scientific-reference rebaseline**, not a silent rewrite of v26 evidence. The immutable Audit-v4 discovery target `v0.3.4` / `anthrosim-model-semantics-v25` and the reviewed v26 AV4-001 reference retain their original provenance. The v27 canonical result is copied byte-for-byte from the exact workflow artifact after validating its semantics ID, run count, recommendation, precision decisions, long-run conclusion and paired-effect directions.
""",
    encoding="utf-8",
)

Path("docs/research/general-scientific-demographic-baseline-v1.md").write_text(
    """# General scientific demographic baseline result — current confirmation

**Status: living/current TRACE-linked result.** This stable path reports the authoritative current #304 confirmation. The superseded `deterministic_size_fission_v1` / 64-seed narrative is preserved separately in [`general-scientific-demographic-baseline-v1-historical.md`](general-scientific-demographic-baseline-v1-historical.md).

**Conclusion: no universal demographic baseline should be designated.** Future scientific studies must explicitly declare demography and household lifecycle. `replacement_control_v1` remains an intrinsic replacement control, not a promise of realized stationarity.

## Authoritative confirmation identity

The machine-readable sources of truth are:

- [`../../research/general-demography-baseline-v1/confirmatory-definition.json`](../../research/general-demography-baseline-v1/confirmatory-definition.json);
- [`../../research/general-demography-baseline-v1/confirmatory-result.json`](../../research/general-demography-baseline-v1/confirmatory-result.json).

Current confirmation:

- fixed household control: `fixed_founder_v1`;
- structural treatment: `deterministic_dependency_fission_v2`;
- fresh process seeds per arm: **130** (`3042001..3042130`);
- design: **3 demography schedules × 2 household-lifecycle arms × 130 seeds = 780 completed runs**;
- founder age ceiling: **60 years**;
- resource productivity: **1000 permille**;
- current model semantics: `anthrosim-model-semantics-v27`;
- current preserved research execution identity: `research-execution-v1-2a74837d17e12d37`.

The v27 causal semantics change in Audit-v4 AV4-002/#488 changes same-seed background-mortality draw assignment while preserving the study's high-level conclusion. The canonical v27 result was reproduced by issue-304 workflow run `33785448601`, job `100748979710` (artifact `9905391271`, SHA-256 `9c8c2cc4f661377e2dca151147bb4c98d146a5668bc8d7c8de48c26ab381be96`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25 and v26 results remain immutable provenance rather than being rewritten as v27 evidence.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.553 [-2.844, -2.262] | 2.4 | 53.8% | 38.1% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.219 [-1.395, -1.042] | 13.5 | 18.5% | 23.1% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.062 [-1.207, -0.917] | 20.8 | 3.8% | 43.3% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.038 [-0.095, +0.019] | 111.9 | 0.8% | 11.8% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.839 [-2.053, -1.625] | 7.4 | 24.6% | 41.0% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.480 [-0.588, -0.373] | 44.0 | 2.3% | 17.9% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.0 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-91.1 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-36.6 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v27 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.038%/year under fixed-founder households to -1.062%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 11.8% to 43.3% and mean N240 falls from 111.9 to 20.8.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v27 long-run analysis still detects multiple stable regimes, with primary classifications `drifting=747`, `insufficient_data=25`, `stable=8`; one treatment context meets the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25 and v26 evidence likewise remains attached to its original semantics rather than being relabelled as v27 output.
""",
    encoding="utf-8",
)

# M8.6: project the exact schema-2 scientific reference from the generated schema-2 aggregate.
m8 = read_json("/tmp/m8/m8-benchmark-output/benchmark-summary.json")
assert m8["schemaVersion"] == 2
assert m8["classification"] == {
    "benchmarkClass": "fragile_spatial_structure",
    "degenerateArms": [],
    "robustMetrics": [],
    "fragileMetrics": ["terminalPopulationHerfindahlPerMillion", "terminalLargestCellSharePermille"],
}
assert m8["aggregateCanonicalSha256"] == "cdb68674436e5eb571445b3e86b18737f960c959c1041bff2ba99657f2ce1dfe"
for arm in m8["arms"].values():
    assert arm["terminalDegenerateRuns"] == 0
flat = m8["arms"]["flat"]
flat_first = flat["runs"][str(m8["declaredSeeds"][0])]

def m8_metric_projection(value):
    out = {
        "classification": value["classification"],
        "robustCriteria": value["robustCriteria"],
    }
    for label in ("weak", "moderate", "strong"):
        src = value["comparisonsToFlat"][label]
        out[label] = {
            "medianEffect": src["medianEffect"],
            "medianAbsoluteRelativeEffect": src["medianAbsoluteRelativeEffect"],
            "positiveEffects": src["positiveEffects"],
            "negativeEffects": src["negativeEffects"],
            "zeroEffects": src["zeroEffects"],
        }
    return out

m8_ref = {
    "schemaVersion": 2,
    "benchmarkId": m8["benchmarkId"],
    "referenceExecution": {
        "workflowRunId": M8_RUN,
        "artifactId": M8_ARTIFACT,
        "artifactSha256": M8_ARTIFACT_SHA,
        "branchHeadSha": BRANCH_HEAD,
        "pullRequestMergeRefBuildSha": None,
        "fullAggregateCanonicalSha256": m8["aggregateCanonicalSha256"],
        "pullRequestMergeRefBuildStatus": PRIVACY_REF,
    },
    "source": {
        "contentDigest": flat["sourceContentDigests"][0],
        "landscapeIdentity": flat_first["landscapeIdentity"],
        "landscapeDigest64": flat_first["landscapeDigest64"],
        "evidenceCatalogCanonicalSha256": flat["evidenceCanonicalSha256"],
        "modelSemanticsId": flat_first["modelSemanticsId"],
        "spatialModelSemanticsId": flat_first["spatialModelSemanticsId"],
    },
    "declaredSeeds": m8["declaredSeeds"],
    "classification": m8["classification"],
    "primaryMetrics": {k: m8_metric_projection(v) for k, v in m8["primaryMetrics"].items()},
    "arms": {},
}
for name, arm in m8["arms"].items():
    first = arm["runs"][str(m8["declaredSeeds"][0])]
    m8_ref["arms"][name] = {
        "experimentId": arm["experimentId"],
        "mechanismsCanonicalSha256": arm["mechanismsCanonicalSha256"],
        "spatialConfigIdentity": first["spatialConfigIdentity"],
        "terminalDegenerateRuns": arm["terminalDegenerateRuns"],
        "runStateDigest64": {seed: run["stateDigest64"] for seed, run in arm["runs"].items()},
    }
compact_write("examples/m8-first-evidence-grounded-benchmark/reference-result.json", m8_ref)

m8_doc = Path("docs/research/m8-first-evidence-grounded-benchmark-result.md")
text = m8_doc.read_text(encoding="utf-8")
marker = "## Current regression reference — model semantics v26\n\n"
assert marker in text
v27_section = """## Current regression reference — model semantics v27

Audit-v4 AV4-002 / #488 changes the same-seed stochastic coupling used to assign background-demographic mortality draws across scientifically distinguishable person roles. Because that correction changes causal population trajectories, the frozen M8.6 experiment was rerun unchanged and its generated artifact was reviewed before replacing the scientific reference.

Reviewed v27 execution:

- workflow run: `33785449208`;
- branch head: `14b5290525f97c6404432e53fd91af5760f400cc`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9905356901`;
- artifact SHA-256: `f7a8fe383d05826cb1ea52d7c5c0721381c55eddb7eb1853da5535e4448d6d41`;
- aggregate canonical SHA-256: `cdb68674436e5eb571445b3e86b18737f960c959c1041bff2ba99657f2ce1dfe`;
- model semantics: `anthrosim-model-semantics-v27`.

All **32/32** declared runs completed the configured duration and all four arms remained non-degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

The v27 primary-metric results are:

| Primary metric | v27 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | not distinctive | 8.08% | 6 / 2 / 0 |
| cell-time occupied | not distinctive | 1.17% | 2 / 6 / 0 |
| terminal population Herfindahl | **fragile** | **10.29%** | 5 / 3 / 0 |
| terminal largest-cell share | **fragile** | **14.02%** | 5 / 3 / 0 |

No primary metric is robust under v27. Relative to v26, terminal population Herfindahl crosses the predeclared 10% median-absolute-effect threshold in the opposite direction—from **9.50% / not distinctive** to **10.29% / fragile**—while its paired sign split remains insufficiently stable for robustness. Terminal largest-cell share remains fragile; migration distance and cell-time occupancy remain not distinctive.

### Causal review of the v27 rebaseline

This reference was not refreshed merely because a regression check failed. AV4-002 changes which scientifically represented person receives each same-seed background-mortality draw. The benchmark's terrain inputs and spatial transformation semantics are unchanged, but corrected mortality coupling legitimately changes downstream population and migration histories.

The reviewed v27 run preserves the benchmark design and interpretation boundaries:

- declared seeds remain 8601–8608;
- the same evidence content digest, evidence catalogue and `landscape-v2-6827044513b6c9fb` are used;
- `anthrosim-spatial-transform-semantics-v3` is unchanged;
- all mechanism-file canonical identities are unchanged;
- all four arms remain non-degenerate;
- the overall benchmark class remains `fragile_spatial_structure`;
- terminal population Herfindahl changes from not distinctive under v26 to fragile under v27;
- terminal largest-cell share remains fragile;
- no metric becomes robust.

The Herfindahl threshold crossing is scientifically meaningful and is preserved rather than tuned away. It is a downstream sensitivity to a causal demographic repair, not evidence that the terrain mapping itself changed. This remains a result about the declared terrain-only null model under the complete v27 upstream model definition, not archaeological validation.

"""
text = text.replace(marker, v27_section + "## Historical reviewed reference — model semantics v26\n\n", 1)
text = text.replace(
    "The current machine-readable regression reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. It records the reviewed Audit-v4 AV4-001 / #486 execution under `anthrosim-model-semantics-v26`. The benchmark design itself was not changed for this rebaseline: corrected fertility draw coupling changes downstream population histories, so the frozen M8.6 design was rerun and its numerical result inspected before replacement.",
    "The machine-readable reference immediately preceding AV4-002 recorded the reviewed Audit-v4 AV4-001 / #486 execution under `anthrosim-model-semantics-v26`. Its exact values remain historical provenance for the v27 comparison.",
    1,
)
m8_doc.write_text(text, encoding="utf-8")

# M9.7 schema-2 projection from the exact schema-3 aggregate artifact.
m9 = read_json("/tmp/m9/m9-benchmark-output/benchmark-summary.json")
assert m9["schemaVersion"] == 3
assert m9["classification"] == {
    "benchmarkClass": "capability_distinguished",
    "allPredeclaredPairedCriteriaPassed": True,
    "replayAndResumeChecks": "workflow-gated-separately",
}
assert m9["aggregate"]["pairedSeedsPassing"] == 8
assert m9["aggregate"]["pairedSeedsTotal"] == 8
assert all(pair["pass"] for pair in m9["pairs"])
assert m9["aggregateCanonicalSha256"] == "4c17ac0e9d1ee601f46baff8392203cc99ce267ed9e3028596a2f2871aaa65a9"

def m9_continuous(src):
    return {k: src[k] for k in (
        "residentPersonDays", "visitorPersonDays", "daysWithAnyVisitors", "peakVisitors",
        "permanentMigrations", "conditionMortalityDeaths", "journeysStarted", "journeysCompleted"
    )}


def m9_intermittent(src):
    out = {k: src[k] for k in (
        "residentPersonDays", "visitorPersonDays", "daysWithAnyVisitors", "peakVisitors",
        "permanentMigrations", "conditionMortalityDeaths", "journeysStarted", "journeysCompleted",
        "transitPersonDays", "notStartedTotal", "notStartedUnreachable", "originCatchmentCells"
    )}
    out["totalTravelDays"] = src["plannedRoundTripTravelDays"]
    out["totalRoundTripTravelCostUnits"] = src["plannedRoundTripTravelCostUnits"]
    out["totalRoundTripRouteDistanceEdges"] = src["plannedRoundTripRouteDistanceEdges"]
    return out

m9_ref = {
    "schemaVersion": 2,
    "benchmarkId": m9["benchmarkId"],
    "referenceExecution": {
        "workflowRunId": M8_RUN,
        "artifactId": M9_ARTIFACT,
        "artifactSha256": M9_ARTIFACT_SHA,
        "branchHeadSha": BRANCH_HEAD,
        "pullRequestMergeRefBuildSha": None,
        "fullAggregateCanonicalSha256": m9["aggregateCanonicalSha256"],
        "modelSemanticsId": "anthrosim-model-semantics-v27",
        "pullRequestMergeRefBuildStatus": PRIVACY_REF,
    },
    "definitionCanonicalSha256": m9["definitionCanonicalSha256"],
    "declaredSeeds": m9["declaredSeeds"],
    "classification": m9["classification"],
    "aggregate": m9["aggregate"],
    "arms": {
        name: {
            "experimentId": arm["experimentId"],
            "configCanonicalSha256": arm["configCanonicalSha256"],
            "stateDigests": arm["stateDigests"],
        }
        for name, arm in m9["arms"].items()
    },
    "pairs": [],
}
for pair in m9["pairs"]:
    m9_ref["pairs"].append({
        "seed": pair["seed"],
        "continuous": m9_continuous(pair["continuous"]),
        "intermittent": m9_intermittent(pair["intermittent"]),
        "totalFocalPersonDayDifferencePermilleExact": pair["totalFocalPersonDayDifferencePermilleExact"],
        "totalFocalPersonDayDifferencePermilleRounded": pair["totalFocalPersonDayDifferencePermilleRounded"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleExact"],
        "intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded": pair["intermittentPeakVisitorShareOfContinuousMeanResidentsPermilleRounded"],
        "criteria": pair["criteria"],
        "pass": pair["pass"],
    })
compact_write("examples/m9-controlled-aggregation-benchmark/reference-result.json", m9_ref)

m9_doc = Path("docs/research/m9-controlled-aggregation-benchmark-result.md")
text = m9_doc.read_text(encoding="utf-8")
marker = "## Current regression reference — model semantics v26\n\n"
assert marker in text
v27_section = """## Current regression reference — model semantics v27

The current machine-readable regression reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`. Audit-v4 AV4-002 / #488 changes background-mortality stochastic coupling, so the frozen M9.7 design was rerun unchanged and its numerical result reviewed before replacement.

Reviewed v27 execution:

- workflow run: `33785449208`;
- branch head: `14b5290525f97c6404432e53fd91af5760f400cc`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9905367753`;
- artifact SHA-256: `350c6344ec721d201ee4e528de9daec9c5136faa0d6bf90727f5ec24a17c5020`;
- aggregate canonical SHA-256: `4c17ac0e9d1ee601f46baff8392203cc99ce267ed9e3028596a2f2871aaa65a9`;
- reference model semantics: `anthrosim-model-semantics-v27`.

### v27 reference result

The predeclared capability remains distinguished:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment has exactly **270 days** with visitor presence;
- intermittent treatments complete **990–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or condition-mediated death.

The aggregate v27 values are:

- median total focal-person-day difference: **31 permille** (v26: 32);
- maximum paired total focal-person-day difference: **36 permille** (v26: 37);
- median intermittent peak-visitor share: **432 permille** (v26: 441);
- minimum intermittent peak-visitor share: **396 permille** (v26: 398).

All predeclared paired criteria therefore continue to support `capability_distinguished`. The same exact-head workflow separately passed identical intermittent replay and active annual checkpoint/resume before reaching the frozen-reference comparison.

The independently preserved M9.6 travel-burden reference is **not rebaselined** for v27. For every seed, planned and observed transit days, planned and realized travel cost, and planned and realized route distance remain exactly equal to the existing travel-burden reference. The changes are population-dependent occupancy outcomes downstream of corrected background mortality, not changed travel-accounting semantics.

"""
text = text.replace(marker, v27_section + "## Historical reviewed reference — model semantics v26\n\n", 1)
text = text.replace(
    "The current machine-readable regression reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`. It records the reviewed Audit-v4 AV4-001 / #486 execution under `anthrosim-model-semantics-v26`. The benchmark design itself was not changed for this rebaseline: corrected fertility draw coupling changes downstream population histories, so the frozen M9.7 design was rerun and its numerical result inspected before replacement.",
    "The machine-readable reference immediately preceding AV4-002 recorded the reviewed Audit-v4 AV4-001 / #486 execution under `anthrosim-model-semantics-v26`. Its exact values remain historical provenance for the v27 comparison.",
    1,
)
text = text.replace(
    "The checked-in v26 reference verifies that this capability distinction survives the corrected stochastic-coupling semantics represented by the reviewed AV4-001 execution.",
    "The checked-in v27 reference verifies that this capability distinction survives the corrected background-mortality stochastic coupling represented by the reviewed AV4-002 execution.",
    1,
)
m9_doc.write_text(text, encoding="utf-8")

# Living roadmap: update only the current M8.6 result; historical trace documents stay historical.
roadmap = Path("docs/roadmap.md")
text = roadmap.read_text(encoding="utf-8")
old = "All 32 runs reached the configured 100-year duration. The predeclared aggregate classification is **fragile spatial structure**: total migration distance and terminal largest-cell share showed material paired effects under the strong terrain mapping, but effect direction was not stable across seeds; cell-time occupancy and terminal Herfindahl concentration were not distinctive under the predeclared threshold."
new = "All 32 runs reached the configured 100-year duration. Under the current v27 reference, the predeclared aggregate classification remains **fragile spatial structure** with no robust primary metric: terminal Herfindahl concentration and terminal largest-cell share are fragile, while total migration distance and cell-time occupancy are not distinctive. The metric-level classifications are sensitive to upstream causal-demographic semantics and are therefore preserved as reviewed regression evidence rather than calibration targets."
assert old in text
roadmap.write_text(text.replace(old, new, 1), encoding="utf-8")
