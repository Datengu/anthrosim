# AnthroSim scientific audit v2 — status ledger

Audit target at initialization: AnthroSim `v0.3.2`, protected-main commit `eb240ab482d9683b64081d3d1ea8e151592927ee`.

Protocol: `docs/research/scientific-audit-protocol.md`

Purpose: durable, repository-authoritative state for the second independent/adversarial scientific audit of the integrated AnthroSim framework. This file must be updated during the audit so work can continue safely across agents and conversations without relying on chat history.

## Audit rules

- Verify live `main`, open issues, open PRs, and overlapping branches before each audit session.
- Record exact commit SHAs for evidence.
- Do not mark an area complete on code inspection alone; use adversarial or quantitative evidence where feasible.
- Search existing issues/PRs before creating findings.
- Preserve demonstrated findings before repair.
- Keep P0/P1 fixes distinct from independent re-verification.
- Do not upgrade AnthroSim to empirical research readiness merely because this audit passes.

## Current baseline

| Field | Value |
|---|---|
| Audit generation | v2 / second independent scientific audit |
| Initial release | `v0.3.2` |
| Initial protected-main SHA | `eb240ab482d9683b64081d3d1ea8e151592927ee` |
| Initial model semantics | `anthrosim-model-semantics-v19` — corrected from the ledger's erroneous initial v15 label after exact-tag verification; see #314 |
| Initial issue state | no open issues at audit initialization |
| Latest audited protected-main SHA | `698b0f79827ef9ea11d9eac2fe3ec23b7125e180` |
| Latest audited model semantics | `anthrosim-model-semantics-v19` |
| Current audit findings | #314, #315 open (both P2) |
| Audit state | in progress |

If `main` advances during the audit, add the new SHA to the session log and state whether earlier evidence remains applicable.

## Coverage matrix

Statuses: `not started`, `in progress`, `complete — no finding`, `complete — findings`, `blocked`, `repeat required`.

| ID | Audit area | Status | Evidence / notes | Findings / issues |
|---|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; exact source/docs review plus `area-a-scheduler-audit.py` | #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | not started | — | — |
| C | Households, kinship, social links, lifecycle structure | not started | — | — |
| D | Resources, condition, subsistence, depletion/recovery | not started | — | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | not started | — | — |
| F | Aggregation and interaction mechanisms | not started | — | — |
| G | Initialization, burn-in, path dependence, continuation state | not started | — | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | not started | — | — |
| I | Sensitivity, uncertainty, convergence, robustness | not started | — | — |
| J | Identifiability, equifinality, calibration, discrimination | not started | — | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | not started | — | — |
| L | Observability, analysis outputs, statistical summaries | not started | — | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | Area A exposed two documentation/provenance findings, but Area M has not been systematically audited | #314, #315 relevant |
| N | Cross-system integration | not started | — | — |

## Cross-system integration checklist

These checks are mandatory after the corresponding subsystem work is mature enough to support them.

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | — |
| Demography × resources | not started | — |
| Households × movement | not started | — |
| Movement × resources | not started | — |
| Aggregation × resources | not started | — |
| Initialization × demography | not started | — |
| Initialization × spatial placement | not started | — |
| Stochastic inference × censoring/extinction | not started | — |
| Sensitivity × hidden configuration | not started | — |
| Calibration × identifiability | not started | — |
| Checkpoint/resume × RNG | not started | — |
| Observability × scientific interpretation | not started | — |

## Finding register

Add every scientifically substantive finding here, including findings that are later closed.

| Finding | Severity | Area | Status | Issue/PR | Evidence location | Reverified? |
|---|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift (v15 documented vs executable/tagged v19) | P2 | A / M / K | issue open | #314; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |
| AV2-002 — M2 demographic-time contract retains superseded annual-boundary mortality execution | P2 | A / B / M | issue open | #315; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |

Suggested finding statuses: `hypothesis`, `demonstrated`, `issue open`, `fix in progress`, `fixed`, `reverified`, `not a defect`, `accepted limitation`.

## Session log

Each audit session must append an entry using the template below. Keep entries concise but sufficient for a fresh agent to reconstruct the audit state.

### 2026-08-29 — Area A / authoritative semantics and scheduler behaviour

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** `698b0f79827ef9ea11d9eac2fe3ec23b7125e180` (rechecked before ledger branch work; unchanged from session start)

**Release / model semantics:** `v0.3.2` tag -> `eb240ab482d9683b64081d3d1ea8e151592927ee`; executable semantics at both tag and audited main = `anthrosim-model-semantics-v19`

**Audit area / sub-area:** Area A — authoritative fixed scheduler, M2/M3 mortality timing/attribution, M9 same-day boundaries, core/spatial host parity, scheduler-facing documentation

**Overlap check:**
- open issues at session start: none;
- open PRs at session start and immediately before ledger work: none;
- repository-visible `audit`/`scientific` branches were historical branches with no open PR or active v2-ledger ownership;
- parallel-agent overlap: none visible in repository state;
- findings created during this session: #314 and #315;
- audit evidence/ledger PR created during this session: #316.

**Implementation/docs inspected:** `simulation.rs`, `spatial_simulation.rs`, `demography.rs`, `mortality.rs`, `rng.rs`, `temporary_mobility.rs`, `provenance.rs`, `founder_initialization.rs`, `docs/scientific-model.md`, M2 demographic-time contract, M3 resource/response-time contracts, competing-mortality contract, audit protocol and this ledger.

**Tests/experiments performed:** independent exact-arithmetic adversarial scheduler/mortality checker at `docs/research/audit-v2/area-a-scheduler-audit.py`; live-head GitHub check-run state inspected (25 checks exposed; no failing conclusion found); source parity reviewed across both authoritative hosts. A fresh local Rust checkout could not be executed because the audit runtime could not resolve GitHub DNS, so the independent checker plus live CI/source evidence is recorded explicitly rather than implying a local build occurred.

**Quantitative results:**
- all 365 allowed fixed resource-clock partitions: 0 invalid/duplicate/non-increasing boundary schedules;
- all `365 x 365 = 133,225` resource/migration clock pairs: 0 merge/count/order/collision failures;
- same-day collisions ranged from 1 to 365; 128,400 clock pairs had more than one shared boundary, so collision handling was exercised broadly rather than only at year end;
- annual M2 background probabilities `[0, 1, 50000, 200000, 500000, 999999, 1000000]` across every `P=1..365`: 2,555 exact rational partition compositions, 0 complete-year survival mismatches.

**Findings:**
- AV2-001 / P2: current-facing scientific/audit documentation labels v0.3.2/current semantics v15 although executable/tagged semantics are v19 — #314;
- AV2-002 / P2: `m2-demographic-time-contract-v1.md` still presents superseded annual-boundary M2 mortality execution after #208/PR #283 introduced elapsed competing risks — #315;
- no new executable scheduler defect demonstrated on the audited SHA.

**Issues/PRs created or relevant:** #314, #315, #316; historical #208 / PR #283 is relevant evidence for the superseded mortality contract but is already closed/merged.

**Unresolved hypotheses:** sequential named RNG streams consume variates conditionally; whether scientifically irrelevant entity-ID/record permutations can alter materially interpreted demographic outcomes should be attacked explicitly in Areas B/H. Internal household-lifecycle semantics were not audited beyond their scheduler position.

**Explicitly not examined:** systematic Areas B–N; detailed demographic distributions/structure; household topology semantics; resource depletion/recovery science; movement choice surfaces; aggregation; burn-in/path dependence; Monte Carlo sufficiency; sensitivity/convergence; identifiability; full orchestration/provenance; complete observability/statistical weighting; comprehensive documentation convergence; mandatory cross-system integration passes.

**Does evidence need repeating after main changes?** Repeat the Area A source/parity/adversarial checks if authoritative scheduler, mortality arithmetic, M9 boundary code, or host scheduling changes. Purely unrelated changes do not invalidate the exact arithmetic result, but the ledger must still record the new main SHA. #314/#315 remain open until separately repaired/dispositioned.

**Recommended next step:** Area B — demography, fertility, mortality, ageing and population structure. Begin by verifying live main/issues/PRs/overlap, then attack entity-ID/record permutation sensitivity, annual age-band boundaries, fertility/parentage conditioning and demographic structure under adversarial founder states.

---

### Session template

**Date / agent:**

**Live main SHA:**

**Release / model semantics:**

**Audit area / sub-area:**

**Overlap check:**
- open issues:
- open PRs/branches:
- parallel-agent overlap:

**Implementation/docs inspected:**

**Tests/experiments performed:**

**Quantitative results:**

**Findings:**

**Issues/PRs created or relevant:**

**Unresolved hypotheses:**

**Explicitly not examined:**

**Does evidence need repeating after main changes?**

**Recommended next step:**

---

## Final audit synthesis

Do not fill this section until the coverage matrix and mandatory integration checks are complete.

The final synthesis must state:

- final audited protected-main SHA;
- release/model-semantics state;
- areas completed;
- P0/P1/P2/P3 finding counts;
- which P0/P1 findings were independently reverified;
- unresolved uncertainties/accepted limitations;
- whether a fresh audit still uncovered major scientific defects;
- whether the audit shows meaningful convergence relative to the previous audit;
- what empirical claims remain unsupported;
- recommended next scientific phase.
