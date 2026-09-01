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
| Current P1 findings | **6 open — AV3-001/#387; AV3-006/#410; AV3-008/#415; AV3-009/#416; AV3-011/#419; AV3-013/#423** |
| Current P2 findings | **7 open — AV3-002/#392; AV3-003/#396; AV3-004/#399; AV3-005/#402; AV3-007/#413; AV3-010/#418; AV3-012/#421** |
| Current P3 findings | none discovered |
| Coverage state | **Areas A–H complete; Area I in progress with AV3-007/008; Area J complete with AV3-009/010/011/012/013; K–N pending** |
| Audit-v3 convergence classification | **non-clean convergence pass: v3 discovered new P1 findings** |
| Closure state | **in progress — discovery-only; findings remain unrepaired until A–N discovery is complete** |

The immutable `v0.3.3` tag is the sole scientific discovery target for audit v3. Audit documentation may advance protected `main`, but executable/scientific evidence must continue to interrogate the frozen tag or source proven causally identical to it rather than a repaired successor. Audit v2 is historical context only and is not v3 completion evidence.

## Discovery-only rule

- Demonstrated defects are preserved immediately as GitHub issues with normal P0/P1/P2/P3 severity and exact evidence.
- **No production repair of audit-v3 findings occurs until the A–N discovery pass is complete.**
- Do not advance model semantics, alter executable behaviour, rebaseline scientific references, or close findings as repaired during discovery.
- Intentionally failing adversarial evidence may be preserved in a test-only branch/closed PR, but knowingly red evidence is not merged into protected `main`.
- Later Areas continue against immutable `v0.3.3`; record cross-cutting limitations rather than switching to repaired semantics.

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — P1 finding open** | Fresh M3/M9 half-open boundary-collision adversary passed. AV3-001/#387: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 in `SpatialLandscapeSimulation`; supplementary 10-seed × 4-year neutral host-parity evidence passed. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — P2 findings open** | Certain-fertility M9 locality case exposed AV3-002/#392 in replay. AV3-003/#396: `lastBirthDay=-2000` plus explicit child at day -100 creates 1 artificial first-boundary birth where coherent spacing requires 0. |
| C | Households, kinship, social links, lifecycle structure | **complete — P2 finding open** | PR #397 demonstrated AV3-004/#399: isomorphic founder kin graphs differing only by canonical-ID relabelling leave **2 vs 1** co-resident living parents after dependency-aware fission. PR #394 was later closed unmerged as an invalid stale adversary because its new fixture failed founder validation before reaching its intended mortality/fission assertion; no finding was inferred. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — P2 finding open** | PR #401 demonstrated AV3-005/#402: mild under-supply leaves visible condition 1000 plus **8/1000** latent loss; later full supply with configured recovery zero clears the remainder to **0**. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — no new finding** | Closed evidence PR #404 head `dc0e5ded1923ab4ef67e88607b4a2112a08d316b`, CI `33475847586`: 90° rotation of asymmetric non-flat 5×5 M9 routing preserved minimum cost **8000** and **3 outbound + 3 return days**. |
| F | Aggregation and interaction mechanisms | **complete — no new finding** | Closed evidence PR #406 head `5befc83204ae3d534001f9d5e2396fe8e61eec9c`, CI `33480471002`: one seven-day one-person visit produced **7 visitor-person-days / peak 1 / visitor need 7**; two simultaneous visits produced **14 / 2 / 14**. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — no new finding; AV3-001/002/003 cross-cutting** | Evidence PR #408 head `6ee0ce4444cf9814a37ffcff46daa8d980d067f6`, CI `33486199606`: with mortality, fertility, migration, resource need, condition loss and recovery all disabled, founder-condition arms **400** and **900** permille remained exactly **400** and **900** after five years. Independent year-2 checkpoint/resume in both arms produced the same complete `RecordedRun` as uninterrupted execution after excluding operational resume-lineage metadata. Existing 277 core tests plus the fresh adversary passed. No new Area G defect demonstrated; AV3-001/002/003 remain open cross-cutting limitations. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — P1 finding open** | Closed red PR #409 head `4b837284825669435f4dcd64eef2f9161d459f9f`, CI `33487273744`: AV3-006/#410. Same-seed anti-correlated arms reported half-width **3.666756860283** vs covariance-aware **5.185577281736** at threshold **4.5**. |
| I | Sensitivity, uncertainty, convergence, robustness | **in progress — P1/P2 findings open** | AV3-007/#413 accepts fabricated support-analysis identities. AV3-008/#415: overlapping ancestor/descendant dimension paths leave **4 recorded coordinates but only 2 executable treatments**. AV3-011/#419 is cross-cutting. Remaining horizon/initialization/replicate/numerical-convergence coverage required as applicable. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — P1/P2 findings open** | Frozen identifiability analyzer produced AV3-009/#416 typed-structure collapse, AV3-010/#418 non-conservative held-out averaging, AV3-011/#419 false identification of an unvaried parameter, and AV3-012/#421 hidden nuisance equifinality. AV3-013/#423 (**P1**) then showed protocol-local `evidenceId` strings are not bound to authoritative EvidenceCatalog/source identities, so one underlying source can use separate calibration and held-out aliases for the same observable and evade exact-pair circularity detection. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending; AV3-002/007/008/013 cross-cutting** | Replay, unbound analysis identities, overwritten sensitivity treatments and unbound evidence-source aliases are known hypotheses/limitations. |
| L | Observability, analysis outputs, statistical summaries | **pending; AV3-002/006/007/009/010/012/013 cross-cutting** | Existing findings can misstate replay validity, precision, scale robustness, structural identity, held-out discrimination, equifinality and evidence independence. |
| M | Documentation, TRACE/ODD/ODD+D, and claim consistency | **pending; AV3-013 cross-cutting** | Evidence-role documentation promises a machine-auditable target-level firewall but the source identity behind protocol-local evidence IDs is not bound. |
| N | Cross-system integration | **pending; AV3-001 through AV3-013 cross-cutting** | Explicit coupled attacks remain required. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #387 | Closed red PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf`: founder `lastBirthDay=-100`, certain fertility; core births 0, spatial births 1. |
| AV3-002 — M9 history validator cannot replay declared-founder runs | **P2** | K/L primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #392 | Closed red PR #390 head `3161ddd1269ad78bfb519f1d3eda3111c6e833e7`: replay reconstructs with synthetic-only initialization and fails. |
| AV3-003 — declared `lastBirthDay` can predate a later explicitly declared child | **P2** | B/G primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #396 | Red PR #395 head `662bf03c1b6b3908adea02e1f8f118d833404c7b`: contradictory chronology produces **1 vs expected 0** first-boundary births. |
| AV3-004 — dependency-aware household fission remains PersonId-sensitive through kin-role tie-breaking | **P2** | C primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #399 | Closed red PR #397 head `b04d756a35e83fee3b294df74544b41b1f5bdd76`: isomorphic relabellings leave **2 vs 1** co-resident parents. |
| AV3-005 — zero-recovery full supply clears latent M3 condition deterioration | **P2** | D primary; I/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #402 | Closed red PR #401 head `d3d6f8392b184ca34c418a24d666dc89ae947375`, CI `33474425681`: **8/1000** latent deterioration resets to **0** under full supply with recovery coefficient 0. |
| AV3-006 — independent difference-in-means gate ignores mandatory same-seed covariance | **P1** | H primary; J/L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #410 | Closed red PR #409: reported half-width **3.666756860283** versus covariance-aware **5.185577281736** at threshold **4.5**. |
| AV3-007 — support-sensitivity report can claim executed alternatives with fabricated analysis identities | **P2** | I primary; J/K/L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #413 | Closed red PR #412: nonexistent primary/alternative identities accepted and `materialScaleDependence=false` certified. |
| AV3-008 — overlapping research dimensions can erase declared sensitivity treatments | **P1** | I primary; J/K/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #415 | **4 recorded coordinates but only 2 executable treatments**; declaration order changes realized design. |
| AV3-009 — structural-identifiability gate can collapse distinct typed structure identifiers | **P1** | J primary; L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #416 | JSON `1` and `"1"` both stringify to `"1"`, allowing a false single identified structure. |
| AV3-010 — held-out discrimination averages compatible intervals instead of conservative envelopes | **P2** | J primary; L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #418 | A=`{0,10}`, B=`{9,11}`, tolerance 1: true envelope gap 0; implementation gap 5 and `discriminating=true`. |
| AV3-011 — constant unvaried numeric parameter can be certified as identified | **P1** | J primary; I/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #419 | Claimed `theta=7` at all points yields normalized width 0 and `identified=true` despite zero treatment variation. |
| AV3-012 — equifinality summary can hide nuisance-parameter compensation | **P2** | J primary; L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #421 | Closed red PR #420: theta width **0.1**, nuisance width **1.0**, yet `equifinality.present=false`. |
| AV3-013 — evidence-role firewall can certify held-out independence through unbound evidence-ID aliases | **P1** | J primary; K/M/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #423 | Frozen `scripts/research-evidence-role-audit.py` validates `evidenceId` only as a non-empty string and checks circularity by exact `(evidenceId, observableId)` pair. Separate aliases for the same underlying source can therefore represent calibration and independent corroboration of the same observable without conflict. Later repair must bind role declarations to authoritative stable evidence/source identities and fail closed on aliases/unknown IDs. |

## Convergence accounting

The project objective is a fresh full-scale audit with no newly discovered P0/P1 finding. AV3-001, AV3-006, AV3-008, AV3-009, AV3-011 and AV3-013 mean v3 is necessarily a **non-clean convergence pass**. Discovery continues through all Areas A–N on the same frozen target. After discovery closes, repair and independently reverify the complete backlog, freeze a successor baseline, and start a new audit generation from zero coverage. Do not lower severity to improve convergence classification.

## Initial repository verification — 2026-09-01

- `refs/tags/v0.3.3` resolves exactly to `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.
- Protected `main` was the same SHA at audit initialization.
- Frozen target declares `MODEL_SEMANTICS_ID = "anthrosim-model-semantics-v21"`.
- Exact-SHA release verification passed protected CI, RustSec, M8.6, M9.7 and the fail-closed release-tag workflow before audit-v3 initialization.
- Open issues/PRs at initialization: none.

These establish target provenance, not scientific completion evidence.

## Session / handoff log

### 2026-09-01 — Areas A–H and I/J discovery

- Areas A–F completed; AV3-001 through AV3-005 were demonstrated in A–D, while E/F produced fresh passing adversarial evidence.
- Area H completed with AV3-006/#410.
- Area I remains in progress with AV3-007/#413 and AV3-008/#415 plus cross-cutting AV3-005/011.
- Area J demonstrated AV3-009/#416, AV3-010/#418, AV3-011/#419 and AV3-012/#421 through structural typing, held-out-envelope, unvaried-parameter and nuisance-compensation attacks.
- All findings remain deliberately unrepaired.

### 2026-09-01 — Area J completion / AV3-013

- Live protected `main` at session start: `5a01ca56c23d985bb6acc85832bafd7912446a7f`.
- Re-read the scientific audit protocol, audit-v3 charter and authoritative ledger. Open PR #408 owned Area G; stale Area-C evidence PR #394 also remained open, so neither surface was duplicated.
- Frozen evidence-role firewall inspection demonstrated AV3-013/#423 (**P1**): `evidenceId` is accepted as a free non-empty string and confirmatory circularity is checked only on exact `(evidenceId, observableId)` equality, permitting same-source/same-observable calibration and held-out aliases.
- No repair was made. Together with AV3-009/010/011/012, this supplied fresh structural, parameter, nuisance-compensation, held-out discrimination and calibration/held-out leakage coverage. Area J was closed through ledger PR #424, merged as `d9b84d310c9994073959672b92bcd529237a860b` after all required workflows passed.

### 2026-09-01 — Area G completion and audit-PR cleanup

- Live protected `main` at reconciliation: `d9b84d310c9994073959672b92bcd529237a860b`.
- PR #408 exact evidence head `6ee0ce4444cf9814a37ffcff46daa8d980d067f6` passed all workflow families; central CI run `33486199606` passed all **277 existing core tests** plus the fresh Area-G adversary.
- Fresh initialization/path-dependence attack fixed all erasure mechanisms at zero and compared founder-condition arms **400** and **900** permille over five years. End states remained exactly **400** and **900**, so elapsed model time did not itself erase the causal initial-state contrast in this limiting case.
- Both arms were independently checkpointed at year 2 and resumed. Resumed complete `RecordedRun` output matched uninterrupted execution exactly after excluding operational resume-lineage metadata. No new Area G scientific defect was demonstrated on this surface.
- AV3-001/#387, AV3-002/#392 and AV3-003/#396 remain open cross-cutting initialization/replay limitations and were not repaired.
- PR #394 was reviewed as stale Area-C evidence. Its central CI failure occurred before its intended mortality/fission assertion because the new founder fixture violated the configured female reproductive-age contract (`ParentOutsideConfiguredReproductiveAge`, parent PersonId(2), age 7300 days). The 277 pre-existing core tests passed. It was closed unmerged with no scientific finding inferred.
- PR #408 is test-only evidence and is to be closed unmerged after this ledger update lands; its passing adversary is recorded here rather than merged into production.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. Areas **A–H and J are complete**. Area **I remains incomplete** and must still receive fresh horizon/initialization/replicate/numerical-convergence robustness coverage as applicable. If I is owned by another active agent, continue the next non-overlapping pending surface, **Area K — experiment orchestration, configuration, provenance and reproducibility**. Treat AV3-001 through AV3-013 as known cross-cutting limitations, but repair none of them during discovery. Update this ledger before handoff. Do not begin Bulstrode empirical work, release work, or unrelated development.
