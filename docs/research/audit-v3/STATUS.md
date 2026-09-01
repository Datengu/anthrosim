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
| Current P2 findings | **1 open — AV3-002 / #392** |
| Current P3 findings | none discovered |
| Coverage state | **Areas A–B complete; Area C next; D–N pending** |
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
| A | Authoritative semantics and scheduler behaviour | **complete — P1 finding open** | Fresh M3/M9 half-open boundary-collision adversary passed. Source inspection confirmed explicit M9 within-day order and frozen-snapshot/simultaneous M4 application. Cross-host parity adversary demonstrated AV3-001/#387: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 birth in `SpatialLandscapeSimulation`. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — no additional causal M2 defect; cross-cutting P2 found** | Fresh certain-fertility limiting case confirmed documented M2 persistent-residence parentage: male physically visiting the female's cell across day 365 still gives 0 births, while persistent co-residence gives 1. Source review confirmed interval-start age exposure, partitioned background mortality, explicit birth-spacing quantization and operational record-limit censoring. The valid M9 arm then exposed AV3-002/#392 in integrity replay, not M2 execution. AV3-001 remains a spatial-host limitation on declared founder reproductive history. |
| C | Households, kinship, social links, lifecycle structure | **next / in progress** | Initial v21 review rejected two false positives: `maxLivingMembers` is explicitly a target subordinate to dependency safety, and at-residence-only annual fission is explicitly declared. A fresh executable/counterfactual Area C adversary is still required before completion. |
| D | Resources, condition, subsistence, depletion/recovery | **pending** | Initial v20/v21 review confirmed fractional condition-loss remainder persistence through recovery is explicit contract, not hidden drift. Fresh independent Area D evidence still required. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **pending** | Initial review confirmed M4's Manhattan distance + destination movement-cost excess is explicitly documented; M9 is the routed path-cost mechanism. Fresh independent Area E evidence still required. |
| F | Aggregation and interaction mechanisms | **pending** | — |
| G | Initialization, burn-in, path dependence, continuation state | **pending; AV3-001/002 cross-cutting** | AV3-001 drops declared founder reproductive history in spatial annual M2. AV3-002 prevents M9 history replay from reconstructing declared founders. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **pending** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **pending** | — |
| J | Identifiability, equifinality, calibration, discrimination | **pending** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending; AV3-002 cross-cutting** | AV3-002 is a fail-closed recorded-run integrity/replay limitation for declared-founder + M9 runs. |
| L | Observability, analysis outputs, statistical summaries | **pending; AV3-002 cross-cutting** | M9 event-history replay cannot reconstruct declared founder state. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001/002 cross-cutting** | Both findings are cross-system integration failures involving explicit initialization. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #387 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf`: female founder `lastBirthDay=-100`, minimum spacing 1278 days, certain fertility, zero mortality; core births = 0, spatial births = 1. Core uses founder-history-aware M2 finalizer; spatial host does not. |
| AV3-002 — M9 history validator cannot replay declared-founder runs | **P2** | K/L primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #392 | Frozen `v0.3.3` / `358ae93b...` / v21. Closed red evidence PR #390 head `3161ddd1269ad78bfb519f1d3eda3111c6e833e7`: M2 assertions pass (visitor arm births 0, persistent control births 1), then `RecordedRun::validate_invariants()` fails because `temporary_history.rs` always calls synthetic-only `Population::initialize`. CLI bundle reconstruction already contains the correct initialization-mode switch. Fail-closed, therefore P2 rather than P1. |

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
- Area A closed for discovery with AV3-001 left open and unrepaired.

### 2026-09-01 — Area B complete / AV3-002

- Inspected M2 age exposure, fertility, parentage, birth spacing, partitioned background mortality and record-limit semantics on frozen v0.3.3.
- Fresh test-only PR #390 constructed a deterministic M9/M2 locality counterfactual: a male is physically visiting the female's cell over the annual fertility boundary, but persistent residences differ. With certain fertility and zero mortality/resources, temporary visitor arm births = **0**; persistent co-residence control births = **1**, confirming the documented residence-based M2 rule.
- The valid visitor run then failed only at invariant replay. Source inspection demonstrated `temporary_history.rs` uses synthetic-only founder reconstruction while the CLI bundle validator already implements the correct initialization-mode-aware reconstruction.
- Preserved as AV3-002/#392 (P2); PR #390 closed unmerged as red evidence. No repair made.

### Initial Area C/D/E triage

- C: dependency-aware fission's size target is explicitly subordinate to parent/dependency safety; temporary-away households are explicitly ineligible at annual fission boundaries. These are declared assumptions, not demonstrated defects. Area C still needs a fresh executable/counterfactual test.
- D: v20 fixed-point condition-loss remainder semantics explicitly preserve latent deterioration through full-supply recovery unless condition saturates; no defect assigned. Area D still needs fresh evidence.
- E: M4 permanent relocation explicitly uses Manhattan distance plus destination movement-cost excess; M9 is the route-cost graph mechanism. No defect assigned. Area E still needs fresh evidence.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. **Continue Area C from first principles against frozen v0.3.3.** Do not repeat A/B unless new evidence requires it. Create issues for demonstrated findings and update this ledger. Do **not** repair #387, #392, or any later audit-v3 finding until the complete A–N discovery pass is finished.
