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
| Initial protected-main SHA | `358ae93b57a9b8f7053575dc6651aa959de2b4f9` |
| Audit initialization merge on `main` | `181fc294f40191d9915d60d3a22f6567591ff220` |
| Target software version | `0.3.3` |
| Target model semantics | `anthrosim-model-semantics-v21` |
| Required protected-main contexts at initialization | 24 |
| Open issues at initialization | none |
| Open PRs at initialization | none |
| Overlapping audit work at initialization | none observed |
| Current P0 findings | none discovered |
| Current P1 findings | **1 open — AV3-001 / #387** |
| Current P2 findings | none discovered yet |
| Current P3 findings | none discovered yet |
| Coverage state | **Area A in progress; B–N pending** |
| Audit-v3 convergence classification | **non-clean convergence pass already established: v3 discovered a new P1** |
| Closure state | **in progress — discovery-only; findings accumulate unrepaired until A–N discovery is complete** |

The immutable `v0.3.3` tag is the sole scientific discovery target for audit v3. Audit documentation may advance protected `main`, but executable audit evidence must continue to interrogate the frozen tag/commit rather than a repaired successor.

Audit v2 is historical context only. Its green Areas, repaired findings, protected CI and preserved benchmarks do not automatically satisfy any audit-v3 Area.

## Discovery-only rule

Audit v3 is a coherent frozen-baseline discovery pass.

- Demonstrated defects are preserved immediately as GitHub issues with normal P0/P1/P2/P3 severity and exact evidence.
- Findings are recorded in this ledger as they are discovered.
- **No production repair of audit-v3 findings occurs until the A–N discovery pass is complete.**
- Do not advance model semantics, alter executable behaviour, rebaseline scientific references, or close findings as repaired during discovery.
- Intentionally failing adversarial evidence may be preserved in a test-only branch/closed PR, but knowingly red evidence is not merged into protected `main`.
- Later Areas continue against immutable `v0.3.3` even when an earlier finding affects them; record the dependency/limitation rather than switching to repaired semantics.
- After A–N discovery is complete, enter a separate repair/re-verification phase for the complete backlog, then freeze a successor baseline and begin a new audit generation from zero coverage.

This separation is required for the convergence question: **can one untouched frozen baseline survive a full independent audit without producing a new P0/P1?**

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **in progress — P1 demonstrated** | Fresh M3/M9 half-open boundary-collision adversary passed. Cross-host scheduler parity adversary demonstrated AV3-001: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 birth in `SpatialLandscapeSimulation`. |
| B | Demography, fertility, mortality, ageing, population structure | **pending; AV3-001 cross-cutting limitation** | Spatial M2 fertility drops declared pre-run `lastBirthDay`; Area B must still be audited on frozen v0.3.3 and record affected conclusions explicitly. |
| C | Households, kinship, social links, lifecycle structure | **pending** | — |
| D | Resources, condition, subsistence, depletion/recovery | **pending** | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **pending** | — |
| F | Aggregation and interaction mechanisms | **pending** | — |
| G | Initialization, burn-in, path dependence, continuation state | **pending; AV3-001 cross-cutting limitation** | AV3-001 shows declared founder pre-run reproductive history is not causally preserved by spatial annual M2 execution. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **pending** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **pending** | — |
| J | Identifiability, equifinality, calibration, discrimination | **pending** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending** | — |
| L | Observability, analysis outputs, statistical summaries | **pending** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001 cross-cutting limitation** | AV3-001 is a direct authoritative-host integration divergence between declared initialization and spatial M2 execution. |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; deliberately unrepaired during discovery** | #387 | Affected immutable `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / semantics v21. Fresh test-only PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf` uses the same declared founder config in both hosts with `lastBirthDay=-100`, minimum spacing 1278 days, certain fertility and zero mortality. At day 365, core host births = **0** while spatial host births = **1**. Source inspection shows core calls the founder-history-aware M2 finalizer with `founder_population.as_ref()`, while spatial calls the non-history finalizer. Closed #192/#213 are related repaired contracts but not duplicates. Repair and independent P1 re-verification are deferred until after A–N discovery. |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

Audit-v3 outcome rules:

- If a full audit generation completes Areas A–N without discovering any P0/P1, classify it as a **P1-clean convergence pass**.
- If it discovers one or more P0/P1 findings, preserve them normally and classify that generation as a **non-clean convergence pass**, even if every finding is later repaired.
- **AV3-001 means v3 is necessarily a non-clean convergence pass.** This does not end v3: discovery continues through all Areas A–N on the same frozen target so the complete defect backlog is visible before repair.
- After v3 discovery closes, repair and independently reverify the accumulated findings in a separate phase, freeze a successor release/baseline, and run a fresh audit generation from zero coverage to test for the desired clean convergence signal.
- Do not lower severity to improve the convergence classification.

## Freshness requirements for Area completion

Every Area A–N must record:

- authoritative implementation and documentation inspected;
- exact target SHA and model-semantics identity used;
- at least one fresh falsification-oriented question or construction;
- tests/experiments and quantitative results where feasible;
- neighbouring-system interactions considered;
- explicit comparison against relevant audit-v2 repaired contracts without using v2 as sole evidence;
- findings/dispositions and unresolved uncertainties;
- limitations introduced by already-known unrepaired v3 findings.

## Initial repository verification — 2026-09-01

- `refs/tags/v0.3.3` resolves exactly to `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.
- Protected `main` was the same SHA at audit initialization.
- `crates/anthrosim-core/src/provenance.rs` declares `MODEL_SEMANTICS_ID = "anthrosim-model-semantics-v21"` on the frozen target.
- `v0.3.3` release documentation records software version `0.3.3` and model semantics v21.
- The release-candidate line had 24 required protected-main contexts.
- Exact-SHA release verification passed protected CI, RustSec, manually dispatched M8.6 and M9.7, and the fail-closed named-release tag workflow before audit-v3 initialization.
- Open issues at initialization: none. Open PRs at initialization: none.

These facts establish the audit target and release provenance. They are **not** scientific completion evidence for Areas A–N.

## Session / handoff log

### 2026-09-01 — audit-v3 initialization

- Frozen target established: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / semantics v21.
- Audit-v3 charter created with stricter independence rules: v2 evidence is historical context, not v3 completion evidence; each Area requires fresh adversarial work.
- Coverage deliberately reset to A–N pending.
- Initialization docs merged through PR #385 as `181fc294f40191d9915d60d3a22f6567591ff220`; this documentation merge does not alter the immutable audit target.

### 2026-09-01 — Area A start and AV3-001 discovery

- Inspected tagged core/spatial authoritative scheduler implementations, M3 resource timing, M9 transition timing, M4/M9 ordering documentation, and the founder initialization contract.
- Rejected repeating audit-v2's all-frequency clock-partition adversary as sole v3 evidence.
- Fresh host-level M3/M9 collision test passed: zero-travel M9 departure exactly at M3 day 91 and return exactly at day 182 preserves `[0,91)` home, `[91,182)` visitor, `[182,273)` home resource attribution.
- Cross-host comparison demonstrated AV3-001. With identical declared founder state, female `lastBirthDay=-100`, 1278-day spacing, certain fertility, zero mortality, no migration and no resource demand, expected day-365 spacing is blocked because `465 < 1278`; core births = **0**, spatial births = **1**.
- Exact failing evidence is preserved in closed test-only PR #386 at head `230f652c4f1923fb6851f6a6433053267e0c60bf`.
- AV3-001 preserved as P1 issue #387 and in the ledger before any production repair.

### 2026-09-01 — audit workflow correction

- User explicitly selected frozen-baseline discovery over find-and-repair iteration.
- Protected `main` at correction: `d4a8b510f88f032c5ecd5d8164d45e86210365de`; no open PRs; only open issue is AV3-001 / #387.
- Any previously-created repair branch is non-authoritative and must remain unused during audit-v3 discovery.
- Immediate next action: **continue Area A against immutable v0.3.3; create issues for any additional demonstrated defects; do not repair #387 or any later finding until A–N discovery is complete.**

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live protected `main`, immutable `v0.3.3`, open issues, open PRs and overlapping work. Continue the next incomplete Area from first principles against the frozen tag. Do not mark an Area complete solely because audit v2 or CI previously passed it. Create issues for demonstrated findings, update this ledger, and **do not repair findings until the full A–N discovery pass is complete**.
