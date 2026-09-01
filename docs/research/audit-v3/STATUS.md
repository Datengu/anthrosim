# AnthroSim scientific audit v3 — status ledger

Audit target at initialization: immutable AnthroSim `v0.3.3`, tag commit `358ae93b57a9b8f7053575dc6651aa959de2b4f9`, model semantics `anthrosim-model-semantics-v21`.

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
| Closure state | **in progress — AV3-001 preserved before repair** |

The immutable `v0.3.3` tag is the initial defect-discovery target. Audit documentation and any future repair commits may advance protected `main`; the ledger must distinguish evidence obtained on the frozen release from evidence obtained on a repaired successor head.

Audit v2 is historical context only. Its green Areas, repaired findings, protected CI and preserved benchmarks do not automatically satisfy any audit-v3 Area.

Because AV3-001 is a newly discovered P1 on the frozen release, audit v3 can no longer qualify as the desired **P1-clean convergence pass**, even if AV3-001 and any later findings are fully repaired and independently reverified. Audit v3 remains necessary to complete A–N and remove demonstrated defects; after that, a later frozen repair baseline and a fresh audit generation from zero coverage will be required to test for a genuinely P1-clean pass.

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **in progress — P1 demonstrated** | Fresh M3/M9 half-open boundary-collision adversary passed. Cross-host scheduler parity adversary demonstrated AV3-001: identical declared founder reproductive history yields 0 births in `Simulation` vs 1 birth in `SpatialLandscapeSimulation`. |
| B | Demography, fertility, mortality, ageing, population structure | **pending; AV3-001 cross-cutting dependency** | Spatial M2 fertility currently drops declared pre-run `lastBirthDay`; revisit after AV3-001 repair/reverification. |
| C | Households, kinship, social links, lifecycle structure | **pending** | — |
| D | Resources, condition, subsistence, depletion/recovery | **pending** | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **pending** | — |
| F | Aggregation and interaction mechanisms | **pending** | — |
| G | Initialization, burn-in, path dependence, continuation state | **pending; AV3-001 cross-cutting dependency** | AV3-001 shows declared founder pre-run reproductive history is not causally preserved by spatial annual M2 execution. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **pending** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **pending** | — |
| J | Identifiability, equifinality, calibration, discrimination | **pending** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending** | — |
| L | Observability, analysis outputs, statistical summaries | **pending** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **pending** | — |
| N | Cross-system integration | **pending; AV3-001 cross-cutting dependency** | AV3-001 is a direct authoritative-host integration divergence between declared initialization and spatial M2 execution. |

## Finding register

Finding IDs use `AV3-###` in discovery order and record severity, Area(s), exact affected baseline, issue, repair PR, semantics/provenance implications, and independent re-verification state.

| Finding | Severity | Area | Status | Issue | Evidence / repair / re-verification |
|---|---|---|---|---|---|
| AV3-001 — spatial host ignores declared founder reproductive history during annual M2 | **P1** | A primary; B/G/N cross-cutting | **demonstrated; open; unrepaired** | #387 | Affected immutable `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / semantics v21. Fresh test-only PR #386 head `230f652c4f1923fb6851f6a6433053267e0c60bf` uses the same declared founder config in both hosts with `lastBirthDay=-100`, minimum spacing 1278 days, certain fertility and zero mortality. At day 365, core host births = **0** while spatial host births = **1**. Source inspection shows core calls the founder-history-aware M2 finalizer with `founder_population.as_ref()`, while spatial calls the non-history finalizer. Prior #192/#213 are related repaired contracts but not duplicates. Repair must change authoritative spatial semantics, so a model-semantics identity advance is expected. Independent post-merge P1 re-verification required before closure. |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

Audit-v3 outcome rules:

- If v3 completes Areas A–N without discovering any P0/P1, classify it as a **P1-clean convergence pass**.
- If v3 discovers one or more P0/P1 findings, preserve and repair them normally, but classify v3 as a **non-clean convergence pass** even if every finding is later independently reverified as fixed.
- **AV3-001 means v3 is now necessarily a non-clean convergence pass.** This does not end the audit; v3 must continue A–N, repair/reverify all demonstrated defects, and produce a stable successor baseline.
- After a non-clean pass, a new frozen repair baseline and a new audit generation from zero coverage are required before claiming the desired clean convergence signal.
- Do not lower severity to improve the convergence classification.

## Freshness requirements for Area completion

Every Area A–N must record:

- authoritative implementation and documentation inspected;
- exact target/head SHA and model-semantics identity used;
- at least one fresh falsification-oriented question or construction;
- tests/experiments and quantitative results where feasible;
- neighbouring-system interactions considered;
- explicit comparison against any relevant audit-v2 repaired contract without using v2 as sole evidence;
- findings/dispositions and unresolved uncertainties;
- whether evidence must be repeated after a material repair.

## Initial repository verification — 2026-09-01

- `refs/tags/v0.3.3` resolves exactly to `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.
- Protected `main` was the same SHA at audit initialization.
- `crates/anthrosim-core/src/provenance.rs` declares `MODEL_SEMANTICS_ID = "anthrosim-model-semantics-v21"` on the frozen target.
- `v0.3.3` release documentation records software version `0.3.3` and model semantics v21.
- The release-candidate line had 24 required protected-main contexts.
- Exact-SHA release verification had passed protected CI, RustSec, manually dispatched M8.6 and M9.7, and the fail-closed named-release tag workflow before audit-v3 initialization.
- Open issues: none.
- Open pull requests: none.
- No overlapping audit-v3 agent/branch was observed before creating the initialization branch.

These facts establish the audit target and release provenance. They are **not** scientific completion evidence for Areas A–N.

## Session / handoff log

### 2026-09-01 — audit-v3 initialization

- Frozen target established: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / semantics v21.
- Reusable scientific-audit protocol reviewed; its convergence section already defines a fresh no-new-P0/P1 pass as the useful maturity signal.
- Audit-v3 charter created with stricter independence rules: v2 evidence is historical context, not v3 completion evidence; each Area requires fresh adversarial work.
- Coverage deliberately reset to A–N pending.
- Initialization docs merged through PR #385 as `181fc294f40191d9915d60d3a22f6567591ff220`; this documentation merge does not alter the immutable audit target.

### 2026-09-01 — Area A start and AV3-001 discovery

- Inspected the tagged core and spatial authoritative scheduler implementations, M3 resource timing, M9 transition timing, current M4/M9 ordering documentation, and the founder initialization contract.
- Rejected the tempting all-frequency fixed-clock partition enumeration as sole v3 evidence because audit v2 had already used essentially that adversary; v3 requires fresh evidence.
- Added a fresh host-level M3/M9 collision test: M9 zero-travel departure exactly at M3 day 91 and zero-travel return exactly at day 182. The required half-open attribution `[0,91)` home, `[91,182)` visitor, `[182,273)` home passed under full workspace tests.
- Cross-host code comparison found a divergence: ordinary `Simulation` passes `self.config.founder_population.as_ref()` to `process_demographic_year_after_competing_mortality_recorded_with_founder_history`, while `SpatialLandscapeSimulation` calls `process_demographic_year_after_competing_mortality_recorded` without founder history.
- Added a fresh parity adversary using the same declared founder state in both hosts. Female founder: age 25, `lastBirthDay=-100`; male founder: age 30; minimum spacing 1278; certain fertility; zero mortality; no migration; no resource demand; one-year horizon.
- Expected spacing state at day 365 is blocked because `365 - (-100) = 465 < 1278`.
- Exact CI result on test head `230f652c4f1923fb6851f6a6433053267e0c60bf`: core host births **0**, spatial host births **1**. Formatting and Clippy passed; the workspace failed specifically at `audit_v3_spatial_founder_history::spatial_host_preserves_declared_founder_birth_spacing_history`.
- Searched prior issues. Closed #192 established declared founder reproductive/genealogical history; closed #213 established explicit/evidence-aware spatial founder initialization. Neither covers the newly demonstrated host-integration omission.
- Preserved the defect as **AV3-001 / issue #387 / P1** before any production repair.
- Immediate next action: repair AV3-001 on a dedicated reviewable PR, advance authoritative model semantics, run local/adjacent/full protected validation, merge exact-green, then perform an independent post-merge P1 re-verification from a fresh branch before continuing Area A.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live protected `main`, immutable `v0.3.3`, open issues, open PRs and overlapping work. Continue the next incomplete Area from first principles. Do not mark an Area complete solely because audit v2 or CI previously passed it.

Current priority: preserve/repair/reverify AV3-001 (#387) before declaring Area A complete or using affected spatial-demographic behaviour as evidence in later Areas.
