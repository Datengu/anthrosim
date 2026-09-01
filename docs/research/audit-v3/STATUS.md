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
| Current P1 findings | **1 open — AV3-001 / #387** |
| Current P2 findings | **3 open — AV3-002 / #392; AV3-003 / #396; AV3-004 / #399** |
| Current P3 findings | none discovered |
| Coverage state | **Areas A–C complete; Area D next / in progress; E–N pending** |
| Audit-v3 convergence classification | **non-clean convergence pass: v3 discovered a new P1** |
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
| B | Demography, fertility, mortality, ageing, population structure | **complete — late P2 finding added** | Fresh certain-fertility M9 locality case confirmed documented persistent-residence parentage: visitor co-presence births = 0; persistent co-residence births = 1. That run exposed AV3-002/#392 in M9 integrity replay. A later independent founder-history consistency adversary then demonstrated AV3-003/#396: a declared `lastBirthDay=-2000` can contradict an explicitly declared child at day -100 and create 1 artificial first-boundary birth where coherent spacing requires 0. AV3-001 remains a spatial-host limitation. |
| C | Households, kinship, social links, lifecycle structure | **complete — P2 finding open** | Fresh dependency-aware fission relabelling adversary PR #397 demonstrated AV3-004/#399. Two isomorphic declared founder kin graphs differing only by a consistent canonical-ID swap between same-age/same-sex adult males produced different scientifically meaningful topology after annual fission: the same abstract dependent retained **2** co-resident living parents in one labelling and **1** in the other. Format, Clippy and all 277 existing core tests passed before the intended assertion failed. Earlier triage also confirmed `maxLivingMembers` is explicitly a target subordinate to dependency safety and at-residence-only annual fission is explicitly declared. |
| D | Resources, condition, subsistence, depletion/recovery | **next / in progress** | Initial v20/v21 review confirmed fractional condition-loss remainder persistence through recovery and M4 travel loss is explicit contract, not hidden drift. Fresh independent Area D evidence is now being developed against frozen v0.3.3. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **pending** | Initial review confirmed M4's Manhattan distance + destination movement-cost excess is explicitly documented; M9 is the routed path-cost mechanism. Fresh independent Area E evidence still required. |
| F | Aggregation and interaction mechanisms | **pending** | — |
| G | Initialization, burn-in, path dependence, continuation state | **pending; AV3-001/002/003 cross-cutting** | AV3-001 drops declared founder reproductive history in spatial annual M2. AV3-002 prevents M9 history replay from reconstructing declared founders. AV3-003 allows mutually contradictory declared reproductive chronology to enter execution and alter first-year fertility. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **pending** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **pending** | — |
| J | Identifiability, equifinality, calibration, discrimination | **pending** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending; AV3-002 cross-cutting** | AV3-002 is a fail-closed recorded-run integrity/replay limitation for declared-founder + M9 runs. |
| L | Observability, analysis outputs, statistical summaries | **pending; AV3-002 cross-cutting** | M9 event-history replay cannot reconstruct declared founder state. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001/002/003/004 cross-cutting** | AV3-001/002/003 involve explicit initialization crossing execution/replay boundaries. AV3-004 shows arbitrary canonical IDs can change post-fission kin topology that is subsequently causal for M3 resource sharing, M4 kin/migration and M9 household selection. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #387 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf`: female founder `lastBirthDay=-100`, minimum spacing 1278 days, certain fertility, zero mortality; core births = 0, spatial births = 1. Core uses founder-history-aware M2 finalizer; spatial host does not. |
| AV3-002 — M9 history validator cannot replay declared-founder runs | **P2** | K/L primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #392 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #390 head `3161ddd1269ad78bfb519f1d3eda3111c6e833e7`: M2 visitor/persistent locality assertions pass, then `RecordedRun::validate_invariants()` fails because `temporary_history.rs` always calls synthetic-only `Population::initialize`. Fail-closed, therefore P2 rather than P1. |
| AV3-003 — declared `lastBirthDay` can predate a later explicitly declared child | **P2** | B/G primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #396 | Frozen v21. Red test-only PR #395 head `662bf03c1b6b3908adea02e1f8f118d833404c7b`: female ID 1 declares `lastBirthDay=-2000` while child ID 3 declares ID 1 as mother at `birthDay=-100`; both event ages are individually schedule-valid. Format, Clippy and all 277 existing core tests pass, then the audit assertion fails with births **1 vs expected 0** because only 465 days have elapsed since the known child. This is an internally contradictory-input/fail-closed boundary defect, hence P2. |
| AV3-004 — dependency-aware household fission remains PersonId-sensitive through kin-role tie-breaking | **P2** | C primary; N cross-cutting | **demonstrated; open; deliberately unrepaired** | #399 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #397 head `b04d756a35e83fee3b294df74544b41b1f5bdd76`: two isomorphic founder graphs consistently swap IDs 3/4 between same-age/same-sex adult males while preserving the abstract father role. CI format/Clippy and all 277 existing core tests pass; the fresh assertion then fails with co-resident living parents **2 vs 1**. The v21 anchor sort uses `(birthDay, reproductiveSex, PersonId)` before dependent-parent structure is considered, violating the documented relationship-aware relabelling contract. |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

- AV3-001 means v3 is necessarily a non-clean convergence pass.
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

### 2026-09-01 — initialization and workflow

- v3 charter/ledger initialized from immutable `v0.3.3` with A–N reset to pending.
- User selected a frozen-baseline discovery pass: create issues immediately, but perform no production repairs until all A–N discovery is complete.
- Discovery-only policy was merged through PR #389 as `b626934def3af2aea3d3c66ea8665bb762972747`.

### 2026-09-01 — Area A complete / AV3-001

- Inspected core/spatial scheduler implementations, M3/M9 ordering, M4 frozen-snapshot application and founder-initialization contract.
- Fresh M3/M9 collision test passed half-open resource attribution at exact day-91/day-182 collisions.
- Fresh cross-host founder-history adversary demonstrated AV3-001/#387 (P1), preserved in closed test-only PR #386.
- A supplementary neutral identity-overlay host-parity adversary on PR #391 passed `Quality and tests` across 10 seeds × 4 years; it was closed unmerged because Area A had concurrently been authoritatively closed.

### 2026-09-01 — Area B complete / AV3-002 and late AV3-003

- Inspected M2 age exposure, fertility, parentage, birth spacing, partitioned background mortality and record-limit semantics on frozen v0.3.3.
- Fresh PR #390 M9/M2 locality counterfactual confirmed visitor physical co-presence does not change persistent-residence M2 parentage (0 births vs 1 persistent-control birth), then exposed AV3-002/#392 when declared-founder M9 history replay failed.
- Later fresh PR #395 tested cross-field founder chronology rather than repeating audit-v2 biological-age checks. A mother with `lastBirthDay=-2000` and an explicit child at day -100 was accepted and produced 1 first-boundary birth; coherent latest-known-birth spacing requires 0. Preserved as AV3-003/#396 (P2). No repair made.

### 2026-09-01 — Area C complete / AV3-004

- Reviewed v21 dependency-aware fission and audit-v2's obsolete contiguous-PersonId slicing defect. Confirmed current size ceiling is explicitly a target subordinate to dependency safety and temporary-away annual ineligibility is documented.
- Fresh test-only PR #397 constructed two isomorphic valid founder graphs differing only by a consistent canonical-ID swap between two same-age/same-sex adult males with different abstract kin roles. Adults were age 46 at epoch / 36 at the declared child births, mortality and resource pressure were disabled, migration was disabled, and one annual `deterministic_dependency_fission_v2(4,18)` boundary was isolated.
- Exact evidence head `b04d756a35e83fee3b294df74544b41b1f5bdd76`, CI run `33471585073`: format and Clippy passed; all 277 existing core tests passed; the fresh assertion then failed exactly **2 vs 1** co-resident living parents.
- Source inspection explains the result: independent adults are seeded using `(birthDay, reproductiveSex, PersonId)` before dependent parent structure is considered, so people with different relationship roles are incorrectly allowed to reach arbitrary-ID tie-breaking.
- Preserved as AV3-004/#399 (P2); PR #397 closed unmerged as red evidence. No repair made.

### Area D started

- Re-read the v20 M3 response contract and implementation rather than repeating audit-v2's repaired partial-supply ceiling defect.
- Rejected one false positive: the carried fractional M3 condition-loss remainder is explicitly retained through full-supply recovery and M4 whole-point travel loss unless condition saturates at the relevant bound. The implementation matches that declared causal contract.
- Fresh Area-D adversarial arithmetic and reachable-state checks are in progress against frozen v0.3.3.

### Initial Area E triage

- E: M4 permanent relocation explicitly uses Manhattan distance plus destination movement-cost excess; M9 is the route-cost graph mechanism. No defect assigned. Area E still needs fresh evidence.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. **Continue Area D from first principles against frozen v0.3.3.** Do not repeat Areas A–C unless new evidence requires it. Create issues for demonstrated findings and update this ledger. Do **not** repair #387, #392, #396, #399, or any later audit-v3 finding until the complete A–N discovery pass is finished.
