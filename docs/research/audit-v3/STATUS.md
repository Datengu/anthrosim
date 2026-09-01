# AnthroSim scientific audit v3 — status ledger

Audit target: immutable AnthroSim `v0.3.3`, tag commit `358ae93b57a9b8f7053575dc6651aa959de2b4f9`, model semantics `anthrosim-model-semantics-v21`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v3/README.md`

Purpose: durable repository-authoritative state for the third independent/adversarial comprehensive scientific audit and the first full convergence audit of the frozen `v0.3.3` baseline.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v3 / third independent scientific audit |
| Immutable discovery target | `v0.3.3` |
| Target tag SHA | `358ae93b57a9b8f7053575dc6651aa959de2b4f9` |
| Target software version | `0.3.3` |
| Target model semantics | `anthrosim-model-semantics-v21` |
| Required protected-main contexts at initialization | 24 |
| Open issues at initialization | none |
| Open PRs at initialization | none |
| Current P0 findings | none discovered |
| Current P1 findings | **4 open — AV3-001 / #387; AV3-006 / #410; AV3-008 / #415; AV3-009 / #416** |
| Current P2 findings | **6 open — AV3-002 / #392; AV3-003 / #396; AV3-004 / #399; AV3-005 / #402; AV3-007 / #413; AV3-010 / #418** |
| Current P3 findings | none discovered |
| Coverage state | **Areas A–F complete; Area G in progress by overlapping agent/PR #408; Area H complete; Area I in progress with AV3-007/008; Area J in progress with AV3-009/010; K–N pending** |
| Audit-v3 convergence classification | **non-clean convergence pass: v3 discovered new P1 findings** |
| Closure state | **in progress — discovery-only; findings remain unrepaired until A–N discovery is complete** |

The immutable `v0.3.3` tag is the sole scientific discovery target for audit v3. Audit documentation may advance protected `main`, but executable/scientific evidence must continue to interrogate the frozen tag or source proven causally identical to it rather than a repaired successor.

Audit v2 is historical context only. Its green Areas, repaired findings, protected CI and preserved benchmarks do not automatically satisfy any audit-v3 Area.

## Discovery-only rule

- Demonstrated defects are preserved immediately as GitHub issues with normal P0/P1/P2/P3 severity and exact evidence.
- **No production repair of audit-v3 findings occurs until the A–N discovery pass is complete.**
- Do not advance model semantics, alter executable behaviour, rebaseline scientific references, or close findings as repaired during discovery.
- Intentionally failing adversarial evidence may be preserved in a test-only branch/closed PR, but knowingly red evidence is not merged into protected `main`.
- Later Areas continue against immutable `v0.3.3` even when an earlier finding affects them; record the dependency/limitation rather than switching to repaired semantics.
- After A–N discovery is complete, enter a separate repair/re-verification phase for the complete backlog, freeze a successor baseline, and begin a new audit generation from zero coverage.

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — P1 finding open** | Fresh M3/M9 half-open boundary-collision adversary passed. Cross-host parity demonstrated AV3-001/#387: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 in `SpatialLandscapeSimulation`. Supplementary 10-seed × 4-year neutral host-parity evidence also passed. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — P2 findings open** | Fresh certain-fertility M9 locality case confirmed persistent-residence parentage and exposed AV3-002/#392 in M9 replay. Founder-history consistency adversary demonstrated AV3-003/#396: `lastBirthDay=-2000` plus explicit child at day -100 creates 1 artificial first-boundary birth where coherent spacing requires 0. |
| C | Households, kinship, social links, lifecycle structure | **complete — P2 finding open** | PR #397 demonstrated AV3-004/#399: isomorphic founder kin graphs differing only by canonical-ID relabelling leave **2 vs 1** co-resident living parents after dependency-aware fission. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — P2 finding open** | PR #401 demonstrated AV3-005/#402: mild under-supply leaves visible condition 1000 plus **8/1000** latent loss; a later fully supplied interval with configured recovery exactly zero clears the remainder to **0**. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — no new finding** | Closed evidence PR #404 head `dc0e5ded1923ab4ef67e88607b4a2112a08d316b`, CI `33475847586`: 90° metamorphic rotation of an asymmetric non-flat 5×5 M9 routing case preserved minimum cost **8000** and **3 outbound + 3 return days**. |
| F | Aggregation and interaction mechanisms | **complete — no new finding** | Closed evidence PR #406 head `5befc83204ae3d534001f9d5e2396fe8e61eec9c`, CI `33480471002`: one seven-day one-person visit produced **7 visitor-person-days / peak 1 / visitor need 7**; two simultaneous visits produced exactly **14 / 2 / 14**. |
| G | Initialization, burn-in, path dependence, continuation state | **in progress by overlapping agent / PR #408; AV3-001/002/003 cross-cutting** | PR #408 independently attacks initialization persistence and checkpoint continuation. Do not duplicate while active. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — P1 finding open** | Closed red PR #409 head `4b837284825669435f4dcd64eef2f9161d459f9f`, CI `33487273744`, demonstrated AV3-006/#410. Twenty mandatory same-seed anti-correlated arms gave reported 95% half-width **3.666756860283** and `sufficient_stop` at threshold **4.5**, while covariance-aware half-width is **5.185577281736**. |
| I | Sensitivity, uncertainty, convergence, robustness | **in progress — P1/P2 findings open; AV3-005/007/008 cross-cutting** | AV3-007/#413: support-sensitivity reporting accepts fabricated analysis identities and can certify `materialScaleDependence=false` without proving alternatives ran. Concurrent Area-I work demonstrated AV3-008/#415 (**P1**): ancestor/descendant research-dimension paths can overwrite one another in declaration order, leaving **4 recorded coordinate combinations but only 2 distinct executable treatments**. Area I remains incomplete pending additional horizon/initialization/replicate/numerical-convergence robustness coverage as applicable. |
| J | Identifiability, equifinality, calibration, discrimination | **in progress — P1/P2 findings open; AV3-006/007/008 cross-cutting** | Fresh inspection of frozen `docs/research/identifiability-equifinality-v1.md` and exact analyzer blob `ef90fc82c28ab07b2995512197680f095cc883a7` demonstrated AV3-009/#416 (**P1**): `point.structure` lacks canonical type validation and `str(...)` collapses JSON `1` and `"1"`, allowing `identified=true` with two distinct accepted structure values. A second fresh quantitative attack demonstrated AV3-010/#418 (**P2**): documented conservative held-out structural envelopes are implemented as averages of interval bounds. With A predictions `{0,10}`, B `{9,11}`, tolerance 1, true envelopes `[0,10]` and `[9,11]` overlap (gap 0), but implementation averages to `[5,5]` and `[10,10]` (gap 5) and labels the observable discriminating. Area J remains incomplete pending further parameter-compensation/profile/conditional and leakage/discrimination attacks. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending; AV3-002/007/008 cross-cutting** | AV3-002 blocks declared-founder + M9 replay validation. AV3-007 leaves support-analysis identities unbound. AV3-008 can preserve declared coordinate metadata after a later dimension erases its executable treatment. |
| L | Observability, analysis outputs, statistical summaries | **pending; AV3-002/006/007/009/010 cross-cutting** | AV3-006 can emit spuriously narrow comparative precision; AV3-007 can identity-seal unsupported scale robustness; AV3-009 can collapse structural identities; AV3-010 can turn overlapping compatible held-out ranges into a false discriminating prediction. |
| M | Documentation, TRACE/ODD/ODD+D, and claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001 through AV3-010 cross-cutting** | Existing findings span initialization→demography, replay, household topology→resources/movement, latent condition, stochastic precision, sensitivity execution/provenance, dimension interactions, and identifiability/structural interpretation. Explicit coupled attacks remain required. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #387 | Closed red PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf`: founder `lastBirthDay=-100`, certain fertility; core births 0, spatial births 1. |
| AV3-002 — M9 history validator cannot replay declared-founder runs | **P2** | K/L primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #392 | Closed red PR #390 head `3161ddd1269ad78bfb519f1d3eda3111c6e833e7`: scientific locality assertions pass, then replay reconstructs with synthetic-only initialization and fails. |
| AV3-003 — declared `lastBirthDay` can predate a later explicitly declared child | **P2** | B/G primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #396 | Red PR #395 head `662bf03c1b6b3908adea02e1f8f118d833404c7b`: contradictory chronology produces **1 vs expected 0** first-boundary births. |
| AV3-004 — dependency-aware household fission remains PersonId-sensitive through kin-role tie-breaking | **P2** | C primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #399 | Closed red PR #397 head `b04d756a35e83fee3b294df74544b41b1f5bdd76`: isomorphic relabellings leave **2 vs 1** co-resident parents. |
| AV3-005 — zero-recovery full supply clears latent M3 condition deterioration | **P2** | D primary; I/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #402 | Closed red PR #401 head `d3d6f8392b184ca34c418a24d666dc89ae947375`, CI `33474425681`: **8/1000** latent deterioration resets to **0** under full supply with recovery coefficient 0. |
| AV3-006 — independent difference-in-means gate ignores mandatory same-seed covariance | **P1** | H primary; J/L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #410 | Closed red PR #409 head `4b837284825669435f4dcd64eef2f9161d459f9f`, CI `33487273744`: reported half-width **3.666756860283** versus covariance-aware **5.185577281736** at threshold **4.5**. |
| AV3-007 — support-sensitivity report can claim executed alternatives with fabricated analysis identities | **P2** | I primary; J/K/L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #413 | Closed red PR #412 head `6c145969f8e11a042f623c1582e59141b77b42c9`, CI `33488012229`: nonexistent primary/alternative identities accepted and `materialScaleDependence=false` certified. |
| AV3-008 — overlapping research dimensions can erase declared sensitivity treatments | **P1** | I primary; J/K/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #415 | Frozen `ResearchExperimentDefinition` rejects exact duplicate paths only. Child annual-need values 100/200 followed by parent resource alternatives containing 300/400 yields **4 recorded coordinates but only 2 executable treatments**; declaration order changes realized design. |
| AV3-009 — structural-identifiability gate can collapse distinct typed structure identifiers | **P1** | J primary; L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #416 | Exact frozen/current analyzer blob `ef90fc82c28ab07b2995512197680f095cc883a7`. Compatible structure values JSON `1` and `"1"` both stringify to `"1"`, so the structural diagnostic reports one identified structure and can pass the claim. |
| AV3-010 — held-out discrimination averages compatible intervals instead of conservative envelopes | **P2** | J primary; L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #418 | Same frozen analyzer blob. A=`{0,10}`, B=`{9,11}`, tolerance 1: contract envelopes `[0,10]`/`[9,11]` overlap (gap 0); implementation averages to `[5,5]`/`[10,10]` (gap 5) and returns `discriminating=true`. |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

- AV3-001, AV3-006, AV3-008 and AV3-009 mean v3 is necessarily a **non-clean convergence pass**.
- Discovery continues through all Areas A–N on the same frozen target so the complete defect backlog is visible before repair.
- After v3 discovery closes, repair and independently reverify the accumulated findings in a separate phase, freeze a successor release/baseline, and run a fresh audit generation from zero coverage.
- Do not lower severity to improve convergence classification.

## Freshness requirements for Area completion

Every Area A–N must record authoritative implementation/documentation inspected; exact target SHA/semantics; at least one fresh falsification-oriented question/construction; quantitative or executable evidence where feasible; neighbouring interactions; relevant audit-v2 comparison without treating v2 as sole evidence; findings/dispositions; and limitations introduced by unrepaired v3 findings.

## Initial repository verification — 2026-09-01

- `refs/tags/v0.3.3` resolves exactly to `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.
- Protected `main` was the same SHA at audit initialization.
- Frozen target declares `MODEL_SEMANTICS_ID = "anthrosim-model-semantics-v21"`.
- Exact-SHA release verification passed protected CI, RustSec, M8.6, M9.7 and the fail-closed release-tag workflow before audit-v3 initialization.
- Open issues/PRs at initialization: none.

These establish target provenance, not scientific completion evidence.

## Session / handoff log

### 2026-09-01 — initialization through Area H

- Audit v3 initialized from immutable `v0.3.3`; discovery-only policy merged through PR #389.
- Area A demonstrated AV3-001/#387 (P1).
- Area B demonstrated AV3-002/#392 and AV3-003/#396 (P2).
- Area C demonstrated AV3-004/#399 (P2).
- Area D demonstrated AV3-005/#402 (P2).
- Area E completed with no new finding after passing 90° M9 route-cost/duration metamorphic evidence PR #404.
- Area F completed with no new finding after exact 2× simultaneous aggregation superposition PR #406.
- Area H demonstrated AV3-006/#410 (P1) with the same-seed covariance false-precision adversary.
- All findings remain deliberately unrepaired during discovery.

### 2026-09-01 — Area I progress

- AV3-007/#413 (P2) showed the support-scale sensitivity gate can accept fabricated `analysisIdentity` values while claiming required alternative analyses were executed.
- Concurrent work demonstrated AV3-008/#415 (P1): overlapping ancestor/descendant dimension paths can erase a declared treatment while preserving its recorded coordinate and nominal factorial identity.
- Area I remains **in progress**; AV3-007/008 do not complete required sensitivity/convergence coverage.

### 2026-09-01 — Area J progress / AV3-009 and AV3-010

- Live `main` was `7ca6c243d4cbad2355c0a78a579500f4ac4e71d4` when this Area-J pass began. Open overlapping audit PR #408 owns Area G; Area I had just received #415, so this session avoided duplicating either surface and moved to J.
- Inspected frozen `docs/research/identifiability-equifinality-v1.md` and `scripts/research-identifiability.py` from first principles. Analyzer blob `ef90fc82c28ab07b2995512197680f095cc883a7` is identical on frozen v0.3.3 and the then-current main.
- Fresh falsification 1: two calibration-compatible points declare distinct structure JSON values `1` and `"1"`. Because structure identity is unvalidated and stringified, both collapse to `"1"`; the structural diagnostic reports one compatible structure, `identified=true`, `equifinal=false`. This demonstrates AV3-009/#416 (**P1**).
- Fresh falsification 2: the contract says held-out predictions use conservative envelopes across compatible points, but the implementation averages interval bounds. With deterministic held-out predictions A=`{0,10}`, B=`{9,11}` and tolerance 1, true envelopes overlap with gap **0**, whereas averaged intervals become `[5,5]` and `[10,10]`, gap **5**, producing `discriminating=true`. This demonstrates AV3-010/#418 (**P2**).
- No repair was made. Area J remains **in progress** pending additional fresh parameter-compensation/profile/conditional, calibration/held-out leakage and discrimination attacks.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. Do not duplicate Area G while PR #408 remains active. Reconcile newer Area-I work before touching I. **Continue Area J from first principles if no other agent has taken it; AV3-009/010 do not by themselves complete Area J.** Treat AV3-001 through AV3-010 as known cross-cutting limitations, but repair none of them. Create issues for fresh demonstrated defects and update this ledger before handoff. Do not begin site empirical work, release work, or unrelated development.