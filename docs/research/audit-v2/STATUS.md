# AnthroSim scientific audit v2 — status ledger

Audit target at initialization: AnthroSim `v0.3.2`, protected-main commit `eb240ab482d9683b64081d3d1ea8e151592927ee`.

Protocol: `docs/research/scientific-audit-protocol.md`

Purpose: durable, repository-authoritative cross-session state for the second independent/adversarial scientific audit. Detailed evidence lives in the per-area reports; this ledger preserves the current baseline, findings, dependencies, and handoff without relying on chat context.

## Audit rules

- Verify live `main`, open issues, open PRs, and overlapping branches before every session.
- Record exact SHAs and model-semantics identity for evidence.
- Do not mark an area complete on inspection alone; use adversarial/quantitative evidence where feasible.
- Search existing issues/PRs before creating findings.
- Preserve demonstrated findings before repair.
- P0/P1 fixes require independent re-verification.
- Do not imply empirical/archaeological validation from software/scientific-method audit success.

## Current baseline

| Field | Value |
|---|---|
| Audit generation | v2 / second independent scientific audit |
| Initial release | `v0.3.2` |
| Initial protected-main SHA | `eb240ab482d9683b64081d3d1ea8e151592927ee` |
| Initial model semantics | `anthrosim-model-semantics-v19` — corrected from stale v15 label; see #314 |
| Latest audited protected-main SHA | `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4` |
| Latest audited model semantics | `anthrosim-model-semantics-v19` |
| Current P1 findings | #326, #334 |
| Current P2 findings | #314, #315, #320, #324, #327, #329, #332 |
| Non-scientific audit infrastructure | #317 |
| Audit state | in progress |

`main` advanced to `fdeb66ed...` through audit documentation only after the Area B executable baseline. The executable v19 tree audited by Areas C–H remained unchanged. Repository-visible stacked recording PRs/branches preserve Areas C–H; do not restart an area merely because its recording PR has not yet merged to protected `main`.

## Coverage matrix

Statuses: `not started`, `in progress`, `complete — no finding`, `complete — findings`, `blocked`, `repeat required`.

| ID | Audit area | Status | Evidence / notes | Findings / issues |
|---|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; scheduler arithmetic/adversarial checks | #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | complete — findings | `area-b-2026-08-29.md`; adversarial PR #319; 2,000-seed relabelling diagnostic | #320; #214 coupling evidence |
| C | Households, kinship, social links, lifecycle structure | complete — findings | `area-c-2026-08-29.md`; adversarial PR #323; recording PR #325 | #324 |
| D | Resources, condition, subsistence, depletion/recovery | complete — findings | `area-d-2026-08-29.md`; `area-d-condition-rounding-audit.py` | #326 P1 |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | complete — findings | `area-e-2026-08-29.md`; symmetry/boundary/keyed-tie checks; PR #328 | #327; #214 relevant |
| F | Aggregation and interaction mechanisms | complete — findings | `area-f-2026-08-29.md`; M9.7 raw-reference recomputation; PR #330 | #329 |
| G | Initialization, burn-in, path dependence, continuation state | complete — findings | `area-g-2026-08-29.md`; year-zero adversarial PR #331; PR #333 | #332; #168 substantially reverified locally |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | complete — findings | `area-h-2026-08-29.md`; exact binomial quantile-coverage checker | #334 P1; #214/#231 historical contracts reviewed |
| I | Sensitivity, uncertainty, convergence, robustness | not started | Preserve #326 and #334 as convergence/inference dependencies | #326, #334 dependencies |
| J | Identifiability, equifinality, calibration, discrimination | not started | Area F already shows total focal person-days alone are non-identifying | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | not started | #329/#332 already relevant | #329, #332 |
| L | Observability, analysis outputs, statistical summaries | not started | #327/#329/#332/#334 already relevant | #327, #329, #332, #334 |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | Earlier areas exposed multiple documentation/result drifts | #314, #315, #327, #329 |
| N | Cross-system integration | not started | Mandatory dependencies below | #324, #326, #334 |

## Cross-system integration checklist

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | Areas B/C locally checked; #324 requires post-repair integration |
| Demography × resources | not started | #326 can alter condition-mediated mortality trajectories |
| Households × movement | not started | #324 can create cohort/newborn-sorted autonomous M4/M9 units |
| Movement × resources | not started | #326 changes condition pressure seen by M4; M9 visitors change demand |
| Aggregation × resources | not started | Area F local coupling coherent; repeat after #326 repair |
| Initialization × demography | not started | #320 founder chronology defect |
| Initialization × spatial placement | not started | Area G confirms persistent founder-layout dependence in a no-relaxation control |
| Stochastic inference × censoring/extinction | partial — local governance positive, tail gate defective | survivor-conditioned/undefined-extinction contract is explicit; #334 invalidates supported quantile/tail confidence claims until repair |
| Sensitivity × hidden configuration | not started | Area I |
| Calibration × identifiability | not started | Area J |
| Checkpoint/resume × RNG | partial — positive local evidence | v19 continuation identity binds named RNG positions; ordinary year-3 resume exact; #332 is separate year-zero metric-history drift |
| Observability × scientific interpretation | not started | #327, #329, #332 and #334 all cross into interpretation/analysis |

## Finding register

| Finding | Severity | Area | Status | Issue/PR | Evidence location | Reverified? |
|---|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift | P2 | A / M / K | issue open | #314 | `area-a-2026-08-29.md` | n/a — open |
| AV2-002 — M2 demographic-time contract retains superseded mortality execution | P2 | A / B / M | issue open | #315 | `area-a-2026-08-29.md` | n/a — open |
| AV2-003 — declared founder genealogy admits biologically impossible reproductive chronology | P2 | B / G / C | issue open | #320; probe #319 | `area-b-2026-08-29.md` | n/a — open |
| AV2-004 — deterministic household fission derives social composition from PersonId/birth-append order | P2 | C / N / I | issue open | #324; probe #323; PR #325 | `area-c-2026-08-29.md` | n/a — open |
| AV2-005 — per-boundary ceiling multiplies partial-supply condition deterioration with M3 resolution | P1 | D / I / N | issue open | #326 | `area-d-2026-08-29.md` | **required after repair** |
| AV2-006 — active M9 travel contract documents superseded lower-CellId equal-cost selection | P2 | E / M / L | issue open | #327; PR #328 | `area-e-2026-08-29.md` | n/a — open |
| AV2-007 — current M9.7 narrative reports obsolete provenance/statistics relative to checked-in reference | P2 | F / K / L / M | issue open | #329; PR #330 | `area-f-2026-08-29.md` | n/a — open |
| AV2-008 — year-zero checkpoint/resume injects extra day-zero metric snapshot | P2 | G / K / L | issue open | #332; probe #331; PR #333 | `area-g-2026-08-29.md` | n/a — open |
| AV2-009 — Monte Carlo quantile gate can certify severely under-covered nominal confidence intervals | **P1** | H / I / L | issue open | #334; Area H recording branch | `area-h-2026-08-29.md`; `area-h-quantile-coverage-audit.py` | **required after repair** |

## Condensed session history

- **Area A:** exact scheduler/contract review; all 365 allowed resource partitions and 133,225 resource/migration clock pairs reconciled; findings #314/#315.
- **Area B:** impossible one-day-old founder parent accepted; 2,000-seed relabelling produced 976/2000 paired terminal-count disagreements consistent with #214 sequential-stream coupling; finding #320.
- **Area C:** PersonId/birth-order household fission can segregate generations/newborns; finding #324.
- **Area D:** annual max-loss budget remains 400, but a 1-permille constant deficit realizes annual loss from 1 to 365 as settlement count rises 1→365; finding #326 P1.
- **Area E:** M4 reflection candidate sets were symmetric; finite boundaries explicit; 100,000 M9 equal-cost keyed selections split 49,886/50,114 while active documentation still claimed lower-ID always wins; finding #327.
- **Area F:** independent 8-seed M9.7 recomputation retained capability distinction; current narrative provenance/statistics were stale; finding #329.
- **Area G:** founder-layout persistence remained visible (1000 vs 500 permille largest-cell share); year-zero resume added metric day 0 while terminal present state stayed equal; finding #332.

### 2026-08-29 — Area H / stochasticity, RNG, ensembles, and Monte Carlo inference

**Live main / semantics:** `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`; `anthrosim-model-semantics-v19`.

**Overlap check:** newest handoff was PR #333; no pre-existing Area H audit branch. Historical #214 and #231 were closed and reviewed before new issue creation. Existing #326 remains a separate P1 dependency.

**Implementation/docs inspected:** `crates/anthrosim-core/src/rng.rs`, `scripts/research-monte-carlo-sufficiency.py`, `docs/research/monte-carlo-sufficiency-v1.md`, `docs/research/paired-seed-semantics-v1.md`, `docs/research/survivor-conditioned-estimands-v1.md`, plus Area G continuation evidence.

**Positive evidence:** deterministic named streams and exact positions are explicit; paired seeds are correctly documented as replicate blocking rather than agent-level common random numbers; fixed/sequential seed batches are frozen and partial-batch peeking rejected; survivor-conditioned condition and extinction undefinedness are explicit.

**Adversarial quantitative result:** the current nominal 95% quantile rank interval has exact finite-sample coverage far below 95% in supported cases. Examples from the independent checker: p=.90,n=8 → 56.45%; p=.95,n=8 → **33.08%**; p=.95,n=20 → **62.56%**; p=.99,n=20 → **18.11%**; p=.99,n=100 → **61.56%**. The executable gate can nevertheless decide `sufficient_stop` from narrow observed value-space width without testing whether requested rank coverage is attainable.

**Finding:** AV2-009/#334 P1 — invalid stochastic inference can be certified as sufficiently precise. This is analysis-layer only; simulation trajectories/model semantics do not change.

**Repeat conditions:** independently reverify #334 after repair, including exact finite-sample rank coverage and fail-closed behavior for infeasible confidence/quantile/sample-size combinations. Repeat broader Area H only if RNG derivation/stream consumption, seed-role decomposition, ensemble pairing, Monte Carlo gate methods, stopping rules, or extinction conditioning materially change.

**Handoff:** **Area I — sensitivity, uncertainty, convergence, and robustness.** Do not treat a passing current quantile precision diagnostic as convergence evidence until #334 is repaired/reverified. Preserve #326 as the separate resource-temporal convergence dependency.

---

## Final audit synthesis

Do not fill this section until the coverage matrix and mandatory integration checks are complete. Final synthesis must state the final audited protected-main SHA/model semantics, completed areas, finding counts, independent P0/P1 reverification state, unresolved uncertainties, audit convergence, unsupported empirical claims, and recommended next scientific phase.
