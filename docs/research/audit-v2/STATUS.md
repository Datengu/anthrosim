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
| Latest executable protected-main baseline audited | `4b6a1ac1e239f262d960a6f063bd8f4288978c6a` |
| Model semantics | `anthrosim-model-semantics-v20` |
| Current P0 findings | none |
| Current P1 findings | none — #326, #334, #338, #340 closed and independently reverified |
| Current P2 findings | #314, #315, #320, #324, #327, #329, #332, #336, #342 |
| Current P3 findings | #344 |
| Non-scientific audit infrastructure | #317 |
| Coverage state | **A–N complete** |
| Closure state | **not closed/passed — remaining P2/P3 disposition + short closure pass required** |

P1 closure work advanced authoritative simulator semantics to v20 through #326. The #334, #338 and #340 repairs are analysis/provenance-layer changes and do not alter simulator trajectories. The latest P1-reverified protected main is `4b6a1ac1e239f262d960a6f063bd8f4288978c6a`; original discovery evidence remains tied to the exact SHA recorded in each area report.

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
| AV2-005 — per-boundary ceiling multiplies partial-supply condition deterioration with resolution | **P1** | D/I/N | closed — PR #347 | #326 | **reverified on v20: cadence metamorphics + coupled mortality/M4 checks; #304 v20 confirmation via #350/#351** |
| AV2-006 — M9 travel contract documents superseded lower-CellId equal-cost selection | P2 | E/M/L | open | #327 | n/a |
| AV2-007 — M9.7 narrative provenance/statistics stale relative to reference | P2 | F/K/L/M | open | #329 | n/a |
| AV2-008 — year-zero checkpoint/resume injects extra day-zero metric snapshot | P2 | G/K/L | open | #332 | n/a |
| AV2-009 — Monte Carlo quantile gate can certify under-covered intervals | **P1** | H/I/L/N | closed — PR #352 | #334 | **reverified: exact finite-sample coverage adversarial cases + full CI + fresh 384-run #304 confirmation** |
| AV2-010 — metadata-only coordinates can masquerade as structural sensitivity | P2 | I/K/N | open | #336 | n/a |
| AV2-011 — identifiability gate ignores stochastic uncertainty in calibration outputs | **P1** | J/H/N | closed — PR #353 | #338 | **reverified: fixed-estimate low/high-precision adversarial checker + dedicated workflow/full CI** |
| AV2-012 — downstream analysis arguments are not bound to executed analysis | **P1** | K/L/N | closed — PR #354 | #340 | **reverified: argv/config/RNG/observation-model binding + independent Area K + Rust E2E + M8.6/M9.7 gates** |
| AV2-013 — migration-quality point summaries leave run versus move weighting ambiguous | P2 | L/N | open | #342 | n/a |
| AV2-014 — documented sweep derived-analysis schema versions lag executable contracts | P3 | M | open | #344 | n/a |

## Cross-system integration matrix

| Interaction | Disposition |
|---|---|
| Demography × households | repeat after #324; current fission contrast strongly changes growth/mate limitation but partition rule is not neutral |
| Demography × resources | #326 repaired/reverified; fixed-point partial-supply response is subdivision-invariant and equivalent histories feed equivalent condition-mediated mortality inputs |
| Households × movement | repeat after #324; child/newborn-only fission units become autonomous M4/M9 units |
| Movement × resources | #326 repaired/reverified; equivalent partial-supply histories now feed equivalent M4 condition pressure instead of cadence-dependent threshold flips |
| Aggregation × resources | #326 condition-response defect repaired; #340 applicable-gate rerun also preserved the canonical M9.7 aggregation benchmark |
| Initialization × demography | #320 founder reproductive chronology remains open |
| Initialization × spatial placement | explicit path dependence demonstrated; no new defect |
| Stochastic inference × censoring/extinction | censoring/nullability governance remains positive; #334 repaired/reverified and quantile inference now fails closed when exact rank coverage is infeasible |
| Sensitivity × hidden configuration | complete — #336 metadata-only pseudo-structures remain open |
| Calibration × identifiability | #338 repaired/reverified; unresolved Monte Carlo precision blocks identification and adequate bound precision can legitimately resolve the compatible region |
| Checkpoint/resume × RNG | positive authoritative continuation evidence; #332 metric-history caveat at year zero |
| Observability × scientific interpretation | P1 dependencies #334/#338/#340 resolved; remaining interpretation dependencies are #327/#329/#332/#342/#344 plus other open P2 findings |

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

### P1 closure / independent reverification evidence

- #326 / PR #347 / main `83878a82bf68049af6389ffc1cb477bf035345c8`: under a 1-permille deficit and annual maximum-loss budget 400, P=1/4/12/52/365 all resolve to 0 whole condition loss plus a 400/1000 carried residual; representative 1/10/100/500/1000-permille cadence metamorphics and coupled mortality/M4 checks pass. #350 / PR #351 independently reran the affected #304 confirmatory baseline on v20.
- #334 / PR #352 / main `57fada3b5a5c666302132e274dfc1316bd1a5c4c`: nominal 95% exact distribution-free order-statistic support requires at least n=6/29/59/299 for p=.50/.90/.95/.99; infeasible cases fail closed. The fresh #304 confirmation completed 384/384 runs and preserved the v20 scientific result, with only three independently proven diagnostic schema-version changes.
- #338 / PR #353 / main `cf6759ebd3f45f66eb4bf2ef7703246f18fd6207`: holding estimates 0.00 and 0.10 fixed against target 0.00 ± 0.05, ±0.20 Monte Carlo intervals leave both points unresolved/compatible and block identification, while ±0.01 makes the first acceptable, the second rejected and the parameter identified. The independent Area J checker and protected identifiability workflow pass.
- #340 / PR #354 / main `4b6a1ac1e239f262d960a6f063bd8f4288978c6a`: authoritative `--scale 2` → `--scale 3` changes output 10 → 15; content-bound config scale 3 also yields 15 and later config mutation fails verification. Independent executed-configuration aliases are rejected, and focused/Area K/Rust E2E plus protected M8.6 and M9.7 neighboring scientific gates all pass.

## Required closure work

All four P1 closure requirements are complete and independently reverified. Before audit v2 can be declared closed/passed:

1. Triage/repair the remaining P2/P3 findings and rerun any scientific reference whose interpretation depends on them, especially #324 and its #304 structural-sensitivity implications.
2. Reconcile remaining current-facing documentation/provenance findings and any affected historical/current reference narratives.
3. Perform a short audit-v2 closure/reverification pass on the latest protected main; do **not** restart A–N from scratch unless executable semantics materially change.

## Final audit synthesis

The second comprehensive audit has completed coverage of every required surface A–N. No P0 defect was found. All four P1 defects (#326, #334, #338 and #340) are now repaired and independently reverified on the v20/post-repair line. The audit is **coverage-complete and P1-clear, but not yet scientifically closed** because the remaining P2/P3 backlog still requires disposition and a final closure pass.

The strongest positive result is that AnthroSim now has substantial deterministic/provenance/research-governance infrastructure: scheduler arithmetic, explicit scientific configuration, checkpoint identity, typed censoring/nullability, research protocol binding, acceptable-region/equifinality machinery and downstream artifact lineage all survived meaningful adversarial reinspection in substantial part.

The strongest remaining negative result is now concentrated in the P2/P3 backlog rather than an open P1: especially the #324 household-fission partition rule and its dependence with #304 structural-sensitivity interpretation, alongside unresolved founder-genealogy, checkpoint-observability, sensitivity-classification, weighting and documentation/provenance findings.

Accordingly, no current audit statement should be presented as empirical validation of a prehistoric reconstruction. After the remaining P2/P3 backlog is disposed and the short closure pass is complete, the appropriate next scientific phase is a bounded empirical case-study workflow with explicit evidence roles, sensitivity, identifiability, Monte Carlo precision and held-out corroboration rather than another immediate whole-repository audit.
