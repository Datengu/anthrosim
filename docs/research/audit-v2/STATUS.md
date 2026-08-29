# AnthroSim scientific audit v2 — status ledger

Audit target at initialization: AnthroSim `v0.3.2`, protected-main commit `eb240ab482d9683b64081d3d1ea8e151592927ee`.

Protocol: `docs/research/scientific-audit-protocol.md`

Purpose: durable, repository-authoritative cross-session state for the second independent/adversarial scientific audit. Detailed evidence lives in per-area reports; this ledger preserves current baseline, findings, dependencies, and handoff.

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
| Latest audited protected-main SHA | `17c28357d44d838b7dcc0e74279373767d4d66f6` |
| Latest audited model semantics | `anthrosim-model-semantics-v19` |
| Current P1 findings | #326, #334, #338, #340 |
| Current P2 findings | #314, #315, #320, #324, #327, #329, #332, #336, #342 |
| Non-scientific audit infrastructure | #317 |
| Audit state | in progress |

Protected `main` has advanced through audit-recording merges while executable v19 simulator semantics remain unchanged. Numerical evidence remains tied to the exact SHA recorded in each area report.

## Coverage matrix

| ID | Audit area | Status | Evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | complete — findings | `area-b-2026-08-29.md`; #320; #214 coupling evidence |
| C | Households, kinship, social links, lifecycle structure | complete — findings | `area-c-2026-08-29.md`; #324 |
| D | Resources, condition, subsistence, depletion/recovery | complete — findings | `area-d-2026-08-29.md`; #326 P1 |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | complete — findings | `area-e-2026-08-29.md`; #327; #214 relevant |
| F | Aggregation and interaction mechanisms | complete — findings | `area-f-2026-08-29.md`; #329 |
| G | Initialization, burn-in, path dependence, continuation state | complete — findings | `area-g-2026-08-29.md`; #332 |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | complete — findings | `area-h-2026-08-29.md`; #334 P1 |
| I | Sensitivity, uncertainty, convergence, robustness | complete — findings | `area-i-2026-08-29.md`; #336 |
| J | Identifiability, equifinality, calibration, discrimination | complete — findings | `area-j-2026-08-29.md`; #338 P1 |
| K | Experiment orchestration, configuration, provenance, reproducibility | complete — findings | `area-k-2026-08-29.md`; #340 P1 |
| L | Observability, analysis outputs, statistical summaries | complete — findings | `area-l-2026-08-29.md`; #342; earlier #183/#184/#222/#226/#229 substantially rechecked |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | #314/#315/#327/#329 already relevant |
| N | Cross-system integration | not started | mandatory dependencies below |

## Cross-system integration checklist

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | #324 requires post-repair integration |
| Demography × resources | not started | #326 can alter condition-mediated mortality trajectories |
| Households × movement | not started | #324 can create cohort/newborn-sorted M4/M9 units |
| Movement × resources | not started | #326 changes condition pressure seen by M4; M9 visitors change demand |
| Aggregation × resources | not started | Area F local coupling coherent; repeat after #326 repair |
| Initialization × demography | not started | #320 founder chronology defect |
| Initialization × spatial placement | not started | founder-layout persistence demonstrated in Area G |
| Stochastic inference × censoring/extinction | partial — findings | #334 invalidates supported quantile/tail confidence claims until repair |
| Sensitivity × hidden configuration | complete — findings | #336 metadata-only pseudo-structural coordinates; #324/#326/#334 dependencies |
| Calibration × identifiability | complete — findings | #338 shows stochastic precision invisible to hard acceptable-region decisions |
| Checkpoint/resume × RNG | partial — positive local evidence | v19 continuation identity binds named RNG positions; #332 is year-zero metric-history drift |
| Observability × scientific interpretation | complete — findings | #342 run-vs-move weighting ambiguity; #327/#329/#332/#334/#338/#340 remain interpretation dependencies |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence | Reverification |
|---|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift | P2 | A/M/K | open | #314 | Area A | n/a |
| AV2-002 — demographic-time contract retains superseded mortality execution | P2 | A/B/M | open | #315 | Area A | n/a |
| AV2-003 — founder genealogy admits impossible reproductive chronology | P2 | B/G/C | open | #320 | Area B | n/a |
| AV2-004 — household fission derives social composition from PersonId/birth order | P2 | C/N/I | open | #324 | Area C | n/a |
| AV2-005 — per-boundary ceiling multiplies partial-supply condition deterioration with resolution | **P1** | D/I/N | open | #326 | Area D | **required after repair** |
| AV2-006 — M9 travel contract documents superseded lower-CellId equal-cost selection | P2 | E/M/L | open | #327 | Area E | n/a |
| AV2-007 — M9.7 narrative provenance/statistics stale relative to reference | P2 | F/K/L/M | open | #329 | Area F | n/a |
| AV2-008 — year-zero checkpoint/resume injects extra day-zero metric snapshot | P2 | G/K/L | open | #332 | Area G | n/a |
| AV2-009 — Monte Carlo quantile gate can certify under-covered intervals | **P1** | H/I/L | open | #334 | Area H | **required after repair** |
| AV2-010 — metadata-only coordinates can masquerade as structural sensitivity | P2 | I/K | open | #336 | Area I | n/a |
| AV2-011 — identifiability gate ignores stochastic uncertainty in calibration outputs | **P1** | J/H/N | open | #338 | Area J | **required after repair** |
| AV2-012 — downstream analysis arguments are not bound to executed analysis | **P1** | K/L | open | #340 | Area K | **required after repair** |
| AV2-013 — migration-quality point summaries leave run versus move weighting ambiguous | P2 | L | open | #342 | Area L | n/a |

## Condensed quantitative evidence

- Area A: 365 allowed resource partitions and 133,225 resource/migration clock pairs reconciled; 2,555 exact mortality compositions; zero scheduler arithmetic failures.
- Area B: one-day-old declared founder parent accepted; 2,000-seed relabelling changed paired terminal count in 976/2000 runs (48.8%), consistent with the exact 48% sequential-stream coupling prediction.
- Area D: a constant 1-permille deficit realizes annual condition loss from 1 to 365 as resource settlement count rises 1→365 despite the same annual max-loss budget.
- Area E: 100,000 equal-cost keyed M9 selections split 49,886/50,114 while active documentation still claimed lower-ID always wins.
- Area G: year-zero resume preserves terminal state but changes metric days from the uninterrupted cadence by injecting day 0.
- Area H: exact nominal-95% quantile coverage can be only 33.08% for p=.95,n=8.
- Area I: `k` metadata-only structural labels can create `k` nominal structures while executable structures remain exactly 1.
- Area J: changing unrepresented Monte Carlo SE from 0.001 to 1.0 leaves the hard identifiability verdict unchanged for fixed point estimates.
- Area K: existing #232 test changes only declared `arguments.scale` 2→3 while command remains `--scale 2`; output remains scale-2 `scaledTotal=10`, replay passes, and only provenance identity changes.
- Area L: with run move counts 1/99 and run quality means 0/1000, current run-weighted point mean is 500 while pooled move-weighted mean is 990; a comparator at 600 reverses ranking between the two estimands.

## 2026-08-29 — Area L handoff

**Live executable baseline / semantics:** `17c28357d44d838b7dcc0e74279373767d4d66f6`; `anthrosim-model-semantics-v19`. Area K PR #341 is documentation-only and pending branch-protection completion.

**Positive evidence:** historical censoring, self-describing point design, undefined empty-set means, realized exposure and survivor-conditioning work was substantially rechecked in current `sweep.rs`. Operationally censored runs are excluded from default scientific means, extinction remains an explicit scientific outcome, achieved duration is exposed, and undefined denominator-based means remain null.

**Finding:** AV2-013/#342 P2. Migration-quality point fields are equal-weight means of per-run move means, but names/docs do not state run weighting. A 1-vs-99 move construction gives 500 run-weighted versus 990 move-weighted and reverses ranking against a 600 comparator.

**Repeat:** after #342 repair, preserve the 1/99 unequal-event fixture and make the estimator/weighting unit explicit. If both run- and move-weighted summaries exist, they should return 500 and 990 respectively.

**Next:** **Area M — documentation, TRACE/ODD/ODD+D, and claim consistency.** Reconstruct live repository state and inspect current-facing docs against executable v19 semantics and checked-in evidence. Treat #314, #315, #327 and #329 as known findings rather than duplicates.

---

## Final audit synthesis

Do not fill until Areas M–N and mandatory integration checks are complete. Final synthesis must state final audited SHA/semantics, finding counts, independent P0/P1 reverification state, unresolved uncertainties, audit convergence, unsupported empirical claims, and recommended next scientific phase.
