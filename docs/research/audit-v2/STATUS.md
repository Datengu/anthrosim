# AnthroSim scientific audit v2 — status ledger

Audit target at initialization: AnthroSim `v0.3.2`, protected-main commit `eb240ab482d9683b64081d3d1ea8e151592927ee`.

Protocol: `docs/research/scientific-audit-protocol.md`

Purpose: durable repository-authoritative state for the second independent/adversarial scientific audit.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v2 / second independent scientific audit |
| Initial release | `v0.3.2` |
| Initial protected-main SHA | `eb240ab482d9683b64081d3d1ea8e151592927ee` |
| Latest executable protected-main baseline audited | `17c28357d44d838b7dcc0e74279373767d4d66f6` |
| Model semantics | `anthrosim-model-semantics-v19` |
| Current P0 findings | none |
| Current P1 findings | #326, #334, #338, #340 |
| Current P2 findings | #314, #315, #320, #324, #327, #329, #332, #336, #342 |
| Current P3 findings | #344 |
| Non-scientific audit infrastructure | #317 |
| Coverage state | **A–N complete** |
| Closure state | **not closed/passed — P1 repair + independent reverification required** |

The executable v19 simulator tree remained unchanged through the late audit-recording branches; numerical evidence remains tied to the exact SHA in each area report.

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
| L | Observability, analysis outputs, statistical summaries | complete — findings | `area-l-2026-08-29.md`; #342; earlier #183/#184/#222/#226/#229 rechecked |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | complete — findings | `area-m-2026-08-29.md`; #344 plus #314/#315/#327/#329 |
| N | Cross-system integration | complete — findings/dependencies | `area-n-2026-08-29.md`; no new root issue; P1 propagation quantified |

## Finding register

| Finding | Severity | Area | Status | Issue | Reverification |
|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift | P2 | A/M/K | open | #314 | n/a |
| AV2-002 — demographic-time contract retains superseded mortality execution | P2 | A/B/M | open | #315 | n/a |
| AV2-003 — founder genealogy admits impossible reproductive chronology | P2 | B/G/C | open | #320 | n/a |
| AV2-004 — household fission derives social composition from PersonId/birth order | P2 | C/N/I | open | #324 | n/a |
| AV2-005 — per-boundary ceiling multiplies partial-supply condition deterioration with resolution | **P1** | D/I/N | open | #326 | **required after repair** |
| AV2-006 — M9 travel contract documents superseded lower-CellId equal-cost selection | P2 | E/M/L | open | #327 | n/a |
| AV2-007 — M9.7 narrative provenance/statistics stale relative to reference | P2 | F/K/L/M | open | #329 | n/a |
| AV2-008 — year-zero checkpoint/resume injects extra day-zero metric snapshot | P2 | G/K/L | open | #332 | n/a |
| AV2-009 — Monte Carlo quantile gate can certify under-covered intervals | **P1** | H/I/L/N | open | #334 | **required after repair** |
| AV2-010 — metadata-only coordinates can masquerade as structural sensitivity | P2 | I/K/N | open | #336 | n/a |
| AV2-011 — identifiability gate ignores stochastic uncertainty in calibration outputs | **P1** | J/H/N | open | #338 | **required after repair** |
| AV2-012 — downstream analysis arguments are not bound to executed analysis | **P1** | K/L/N | open | #340 | **required after repair** |
| AV2-013 — migration-quality point summaries leave run versus move weighting ambiguous | P2 | L/N | open | #342 | n/a |
| AV2-014 — documented sweep derived-analysis schema versions lag executable contracts | P3 | M | open | #344 | n/a |

## Cross-system integration matrix

| Interaction | Disposition |
|---|---|
| Demography × households | repeat after #324; current fission contrast strongly changes growth/mate limitation but partition rule is not neutral |
| Demography × resources | **blocked by #326 P1**; resolution artifact changes condition path and condition-mediated mortality exposure |
| Households × movement | repeat after #324; child/newborn-only fission units become autonomous M4/M9 units |
| Movement × resources | **blocked by #326 P1**; 1-permille deficit gives condition 999 vs 635 and flips default M4 900-pressure threshold |
| Aggregation × resources | local accounting coherent; repeat after #326 because aggregation-induced partial supply uses defective condition response |
| Initialization × demography | #320 founder reproductive chronology remains open |
| Initialization × spatial placement | explicit path dependence demonstrated; no new defect |
| Stochastic inference × censoring/extinction | censoring/nullability governance positive; **tail inference blocked by #334** |
| Sensitivity × hidden configuration | complete — #336 metadata-only pseudo-structures remain open |
| Calibration × identifiability | **blocked by #338 P1** for stochastic calibration |
| Checkpoint/resume × RNG | positive authoritative continuation evidence; #332 metric-history caveat at year zero |
| Observability × scientific interpretation | complete with #327/#329/#332/#334/#338/#340/#342/#344 dependencies |

## High-value quantitative evidence

- Area A: 365 allowed resource partitions; 133,225 resource/migration clock pairs; 2,555 exact mortality compositions; zero scheduler arithmetic failures.
- Area B: 2,000-seed pure PersonId relabelling changed paired terminal count in 976/2000 runs = 48.8%, consistent with exact 48% sequential-stream coupling prediction.
- Area C: implemented fission treatment changed late growth about `-0.002%/yr` → `-0.994%/yr`, mate limitation `11.3%` → `38.7%`, and mean year-240 population `108.2` → `28.9`; repeat after #324.
- Area D: constant 1-permille supply deficit yields annual condition loss 1→365 as M3 settlements/year rise 1→365 with identical annual maximum-loss budget 400.
- Area E: 100,000 equal-cost keyed M9 selections split 49,886/50,114 while active documentation claimed lower-ID always wins.
- Area G: year-zero resume preserves terminal state but adds a day-zero metric observation.
- Area H: nominal 95% interval for p=.95,n=8 has only 33.08% exact coverage under current quantile gate.
- Area I: `k` metadata-only structural labels can create `k` nominal structures while executable structures remain 1.
- Area J: fixed point estimates receive the same identifiability verdict when unrepresented Monte Carlo SE changes 0.001→1.0.
- Area K: `arguments.scale` can change 2→3 while executed argv remains `--scale 2`, output remains 10 and replay passes.
- Area L: 1/99 moves with run means 0/1000 produce 500 run-weighted versus 990 move-weighted; a 600 comparator reverses ranking.
- Area M: current source emits derived-analysis schemas 5/6 while current docs state 4/5.
- Area N: composing #326 with default M4 threshold 900 gives condition 999 (pressure off) at P=1 versus 635 (pressure on) at P=365 under the same 1-permille annual deficit.

## Required closure work

Before audit v2 can be declared closed/passed:

1. Repair #326 and independently repeat its partial-supply arithmetic across representative deficits/partitions, then rerun coupled condition-mortality, M4 pressure and aggregation-resource checks.
2. Repair #334 and independently verify exact/valid coverage for supported quantile/tail intervals.
3. Repair #338 and independently show stochastic identifiability responds appropriately to insufficient versus adequate Monte Carlo precision.
4. Repair #340 and independently prove machine-readable downstream analysis configuration is bound to execution/replay.
5. Triage/repair the remaining P2/P3 findings and rerun any scientific reference whose interpretation depends on them, especially #324.
6. Perform a short audit-v2 closure/reverification pass; do **not** restart A–N from scratch unless executable semantics materially change.

## Final audit synthesis

The second comprehensive audit has completed coverage of every required surface A–N on executable semantics v19. No P0 defect was found. Four P1 defects remain open, so the audit is **coverage-complete but not scientifically closed**.

The strongest positive result is that AnthroSim now has substantial deterministic/provenance/research-governance infrastructure: scheduler arithmetic, explicit scientific configuration, checkpoint identity, typed censoring/nullability, research protocol binding, acceptable-region/equifinality machinery and downstream artifact lineage all survived meaningful adversarial reinspection in substantial part.

The strongest negative result is that correctness is not yet compositional in four research-critical places: resource temporal discretization can activate different downstream mechanisms (#326), one Monte Carlo tail gate can assert invalid precision (#334), stochastic identifiability can ignore that precision altogether (#338), and downstream analysis provenance can preserve configuration metadata that was not actually executed (#340).

Accordingly, no current audit statement should be presented as empirical validation of a prehistoric reconstruction. After the P1 repair/reverification cycle and remaining backlog triage, the appropriate next scientific phase is a bounded empirical case-study workflow with explicit evidence roles, sensitivity, identifiability, Monte Carlo precision and held-out corroboration rather than another immediate whole-repository audit.
