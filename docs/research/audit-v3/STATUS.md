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
| Current P1 findings | **2 open — AV3-001 / #387; AV3-006 / #410** |
| Current P2 findings | **4 open — AV3-002 / #392; AV3-003 / #396; AV3-004 / #399; AV3-005 / #402** |
| Current P3 findings | none discovered |
| Coverage state | **Areas A–F complete; Area G in progress by overlapping agent/PR #408; Area H complete; I–N pending** |
| Audit-v3 convergence classification | **non-clean convergence pass: v3 discovered new P1 findings** |
| Closure state | **in progress — discovery-only; findings remain unrepaired until A–N discovery is complete** |

The immutable `v0.3.3` tag is the sole scientific discovery target for audit v3. Audit documentation may advance protected `main`, but executable audit evidence must continue to interrogate the frozen tag/commit rather than a repaired successor.

Audit v2 is historical context only. Its green Areas, repaired findings, protected CI and preserved benchmarks do not automatically satisfy any audit-v3 Area.

## Discovery-only rule

- Demonstrated defects are preserved immediately as GitHub issues with normal P0/P1/P2/P3 severity and exact evidence.
- Findings are recorded in this ledger as they are discovered.
- **No production repair of audit-v3 findings occurs until the A–N discovery pass is complete.**
- Do not advance model semantics, alter executable behaviour, rebaseline scientific references, or close findings as repaired during discovery.
- Intentionally failing adversarial evidence may be preserved in a test-only branch/closed PR, but knowingly red evidence is not merged into protected `main`.
- Later Areas continue against immutable `v0.3.3` even when an earlier finding affects them; record the dependency/limitation rather than switching to repaired semantics.
- After A–N discovery is complete, enter a separate repair/re-verification phase for the complete backlog, freeze a successor baseline, and begin a new audit generation from zero coverage.

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — P1 finding open** | Fresh M3/M9 half-open boundary-collision adversary passed. Source inspection confirmed explicit M9 within-day order and frozen-snapshot/simultaneous M4 application. Cross-host parity adversary demonstrated AV3-001/#387: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 birth in `SpatialLandscapeSimulation`. Supplementary 10-seed × 4-year neutral spatial-host parity PR #391 passed `Quality and tests` before being closed unmerged after concurrent Area-A closure. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — late P2 finding added** | Fresh certain-fertility M9 locality case confirmed documented persistent-residence parentage: visitor co-presence births = 0; persistent co-residence births = 1. That run exposed AV3-002/#392 in M9 integrity replay. A later independent founder-history consistency adversary demonstrated AV3-003/#396: declared `lastBirthDay=-2000` can contradict an explicitly declared child at day -100 and create 1 artificial first-boundary birth where coherent spacing requires 0. AV3-001 remains a spatial-host limitation. |
| C | Households, kinship, social links, lifecycle structure | **complete — P2 finding open** | Fresh dependency-aware fission relabelling adversary PR #397 demonstrated AV3-004/#399. Two isomorphic declared founder kin graphs differing only by a consistent canonical-ID swap between same-age/same-sex adult males produced different scientifically meaningful topology after annual fission: the same abstract dependent retained **2** co-resident living parents in one labelling and **1** in the other. Format, Clippy and all 277 existing core tests passed before the intended assertion failed. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — P2 finding open** | Fresh reachable two-year adversary PR #401 demonstrated AV3-005/#402: year-1 mild under-supply leaves visible condition 1000 plus an **8/1000** causal loss remainder; after zero-cost relocation, year 2 is fully supplied with configured recovery exactly zero, yet runtime clears the remainder to **0**. Format, Clippy and all 277 existing core tests passed before the intended assertion failed `0 vs 8`. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — no new finding** | Fresh quarter-turn metamorphic adversary on closed test-only PR #404 head `dc0e5ded1923ab4ef67e88607b4a2112a08d316b`, CI run `33475847586`, rotated an asymmetric non-flat 5×5 M9 movement-cost landscape, origin and focal destination by 90°. Format, Clippy, all 277 existing core tests and the adversary passed. Independent minimum routed cost was **8000** in both orientations, giving **3 outbound + 3 return days** at capacity 3000. No fresh spatial symmetry/boundary defect was demonstrated. |
| F | Aggregation and interaction mechanisms | **complete — no new finding** | Fresh simultaneous-crowding superposition adversary on closed test-only PR #406 head `5befc83204ae3d534001f9d5e2396fe8e61eec9c`, central CI run `33480471002`, `Quality and tests` job `99768757946`. With mortality/fertility/migration disabled and one-unit daily need, one one-person household visiting the same focal destination for seven days produced **7 visitor-person-days, peak visitors 1, visitor need 7**; two simultaneous one-person households produced exactly **14, 2, 14**. Format, Clippy, all **277** existing core tests and the fresh adversary passed. Source/contract review confirmed duration-weighted M3 destination demand, explicit home provisioning for transit, current-boundary visitor crowding in M4, and separation of temporary physical presence from persistent residence/occupancy. No new Area-F defect was demonstrated. |
| G | Initialization, burn-in, path dependence, continuation state | **in progress by overlapping agent / PR #408; AV3-001/002/003 cross-cutting** | AV3-001 drops declared founder reproductive history in spatial annual M2. AV3-002 prevents M9 history replay from reconstructing declared founders. AV3-003 allows mutually contradictory declared reproductive chronology to enter execution and alter first-year fertility. PR #408 is independently attacking initialization persistence and checkpoint continuation, so this session did not overlap it. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — P1 finding open** | Fresh same-seed covariance adversary on closed red test-only PR #409 head `4b837284825669435f4dcd64eef2f9161d459f9f`, central CI run `33487273744`, `Quality and tests` job `99790237742`, demonstrated AV3-006/#410. `difference_in_means` declares an independent two-sample CLT estimator, but the validator forces both arms to use identical seed identities/order. With 20 accepted anti-correlated seed pairs and variance **35** per arm, the gate reported 95% half-width **3.666756860283** and `sufficient_stop` at threshold **4.5**; the covariance-aware same-seed contrast has variance **140** and half-width **5.185577281736**, so precision is actually insufficient. Format, Clippy, and the existing Monte Carlo regression passed before the intended audit assertion failed. Frozen RNG source confirms equal seeds reproduce equal named stream realizations while conditional draw consumption means same-seed arms cannot be assumed statistically independent. |
| I | Sensitivity, uncertainty, convergence, robustness | **pending; AV3-005 cross-cutting** | AV3-005 can erase repeated sub-unit scarcity exposure under intermittent full-support intervals when the declared recovery coefficient is zero. |
| J | Identifiability, equifinality, calibration, discrimination | **pending; AV3-006 cross-cutting** | AV3-006 can falsely certify replicate sufficiency for between-scenario comparisons when mandatory same-seed covariance is ignored. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending; AV3-002 cross-cutting** | AV3-002 is a fail-closed recorded-run integrity/replay limitation for declared-founder + M9 runs. |
| L | Observability, analysis outputs, statistical summaries | **pending; AV3-002/006 cross-cutting** | M9 event-history replay cannot reconstruct declared founder state. AV3-006 can emit a spuriously narrow comparative Monte Carlo interval and a false machine-readable `sufficient_stop`. |
| M | Documentation, TRACE/ODD/ODD+D, and claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001/002/003/004/005/006 cross-cutting** | AV3-001/002/003 involve explicit initialization crossing execution/replay boundaries. AV3-004 shows arbitrary canonical IDs can change post-fission kin topology that is subsequently causal for M3 resource sharing, M4 kin/migration and M9 household selection. AV3-005 shows M3 latent condition state can be erased by a subsequent full-support interval even when its configured recovery mechanism has zero strength. AV3-006 shows stochastic comparison inference can contradict the seed-coupling structure enforced by its own analysis gate. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #387 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf`: female founder `lastBirthDay=-100`, minimum spacing 1278 days, certain fertility, zero mortality; core births = 0, spatial births = 1. |
| AV3-002 — M9 history validator cannot replay declared-founder runs | **P2** | K/L primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #392 | Frozen v21. Closed red evidence PR #390 head `3161ddd1269ad78bfb519f1d3eda3111c6e833e7`: locality assertions pass, then `RecordedRun::validate_invariants()` fails because temporary-history replay uses synthetic-only population initialization. |
| AV3-003 — declared `lastBirthDay` can predate a later explicitly declared child | **P2** | B/G primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #396 | Red test-only PR #395 head `662bf03c1b6b3908adea02e1f8f118d833404c7b`: mother declares `lastBirthDay=-2000` while explicit child is born at day -100; runtime produces **1 vs expected 0** first-boundary births. |
| AV3-004 — dependency-aware household fission remains PersonId-sensitive through kin-role tie-breaking | **P2** | C primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #399 | Closed red evidence PR #397 head `b04d756a35e83fee3b294df74544b41b1f5bdd76`: isomorphic founder graphs under canonical-ID relabelling leave **2 vs 1** co-resident living parents after fission. |
| AV3-005 — zero-recovery full supply clears latent M3 condition deterioration | **P2** | D primary; I/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #402 | Closed red evidence PR #401 head `d3d6f8392b184ca34c418a24d666dc89ae947375`, CI `33474425681`: reachable run creates **8/1000** latent deterioration, then a fully supplied interval with configured recovery 0 clears it to **0**. |
| AV3-006 — independent difference-in-means gate ignores mandatory same-seed covariance | **P1** | H primary; J/L/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #410 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #409 head `4b837284825669435f4dcd64eef2f9161d459f9f`, CI `33487273744`: accepted 20-seed anti-correlated arms produced reported half-width **3.666756860283** and `sufficient_stop`, while covariance-aware half-width is **5.185577281736** against threshold **4.5**. |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

- AV3-001 and AV3-006 mean v3 is necessarily a non-clean convergence pass.
- This does not end v3: discovery continues through all Areas A–N on the same frozen target so the complete defect backlog is visible before repair.
- After v3 discovery closes, repair and independently reverify the accumulated findings in a separate phase, freeze a successor release/baseline, and run a fresh audit generation from zero coverage.
- Do not lower severity to improve the convergence classification.

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

### 2026-09-01 — initialization and Areas A–E

- Audit v3 was initialized from immutable `v0.3.3`; discovery-only policy was merged through PR #389.
- Area A demonstrated AV3-001/#387 (P1) with cross-host declared-founder reproductive-history divergence.
- Area B demonstrated AV3-002/#392 and AV3-003/#396 (P2) in declared-founder M9 replay and founder reproductive chronology consistency.
- Area C demonstrated AV3-004/#399 (P2) through PersonId-sensitive dependency-aware fission.
- Area D demonstrated AV3-005/#402 (P2) through zero-recovery clearing of latent M3 deterioration.
- Area E closed with no new finding after the passing 90° M9 route-cost/duration metamorphic adversary on PR #404.
- All findings remain deliberately unrepaired during discovery.

### 2026-09-01 — Area F complete / no new finding

- Re-read audit-v2 aggregation evidence only as historical context, then inspected frozen v21 temporary-mobility lifecycle, M9 duration ledger, M3 resource attribution, M4 visitor-aware resource-support cue, and temporary-mobility observability contracts from first principles.
- Fresh falsification question: does simultaneous aggregation superpose exactly, or can household ordering/crowding cause visitor presence or resource pressure to be lost, duplicated or conflated with persistent residence?
- Test-only PR #406 was based from frozen `358ae93b57a9b8f7053575dc6651aa959de2b4f9`; final evidence head `5befc83204ae3d534001f9d5e2396fe8e61eec9c`.
- Central CI run `33480471002`, `Quality and tests` job `99768757946`: format and Clippy passed; all **277** existing core tests passed; fresh `simultaneous_aggregation_superposes_presence_and_resource_pressure_exactly` passed.
- Quantitative result: one seven-day one-person visit => **7 visitor-person-days / peak 1 / visitor need 7**; two simultaneous one-person visits => **14 / 2 / 14**. This is exact 2× superposition in both derived presence and causal destination resource demand.
- Source/contract review confirmed half-open duration accounting, visitor destination attribution, transit home provisioning, current-boundary visitor crowding for M4, and separation of temporary physical presence from persistent residence and permanent occupancy.
- No new scientific defect was demonstrated. PR #406 was closed unmerged as passing test-only audit evidence. No production code or semantics changed.

### 2026-09-01 — Area H complete / AV3-006 P1

- Live-state reconstruction found Area G already under active, non-overlapping audit in PR #408, so this session obeyed the parallel-agent rule and moved to Area H rather than duplicating G.
- Inspected frozen v21 `RngFactory`/stream-position semantics, the normative Monte Carlo sufficiency contract, the precision-plan/sample validator, and the existing Monte Carlo regression suite from first principles. Audit-v2 Area H was treated only as historical context.
- Fresh falsification question: can a `difference_in_means` plan claim independent-group uncertainty while the validator structurally forces same-seed arms, allowing non-zero cross-arm covariance to invalidate the reported precision decision?
- Test-only PR #409 was based from frozen `358ae93b57a9b8f7053575dc6651aa959de2b4f9`; final evidence head `4b837284825669435f4dcd64eef2f9161d459f9f`.
- Central CI run `33487273744`, `Quality and tests` job `99790237742`: format and Clippy passed; the existing Monte Carlo sufficiency regression passed; the fresh scientific assertion then failed intentionally.
- Quantitative result: 20 mandatory same-seed arms with sample variance **35** each and exact negative covariance produced an independent-formula 95% half-width **3.666756860283**, below the predeclared **4.5** threshold, so the gate returned `sufficient_stop`. The accepted same-seed replicate differences have variance **140**, giving covariance-aware half-width **5.185577281736**, above threshold.
- This demonstrates AV3-006/#410 (**P1**): the confirmatory precision gate can certify insufficient comparative replication because its accepted seed-coupling contract contradicts its independent-sample variance formula.
- PR #409 was closed unmerged as intentionally red audit evidence. Issue #410 remains open and deliberately unrepaired. No production behavior, model semantics, release identity or scientific reference was changed.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. Area G is currently owned by another audit agent in PR #408; do not duplicate it while that work remains active. **Continue the next non-overlapping incomplete Area, normally Area I, from first principles against frozen v0.3.3.** If PR #408 has completed and its Area-G ledger update has landed, reconcile that state before proceeding. Treat AV3-001/#387 through AV3-006/#410 as known cross-cutting limitations where relevant, but do not repair any of them. Create issues for any new demonstrated findings and update this ledger. Do **not** repair any audit-v3 finding until the complete frozen-baseline A–N discovery pass is finished.
