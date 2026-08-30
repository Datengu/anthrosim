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
| Latest protected-main closure baseline examined | `9cb73cc405f76fe3c0d0692f60b4aca2bde377a0` |
| Model semantics | `anthrosim-model-semantics-v21` |
| Current P0 findings | none |
| Current P1 findings | none — #326, #334, #338, #340 and closure-discovered #350 repaired and reverified |
| Current P2 findings | #314, #315, #327, #329, #332, #336, #342 |
| Current P3 findings | #344 |
| Non-scientific audit infrastructure | #317 |
| Coverage state | **A–N complete** |
| Closure state | **P1 closure complete; remaining P2/P3 triage/repair and final closure pass required** |

The v2 audit originally completed A–N coverage on model semantics v19. Repair of #326 intentionally advanced authoritative causal semantics to v20. Repair of #324 then advanced authoritative household-assignment semantics to v21; subsequent inference/provenance and fail-closed input-validity contracts remain explicitly versioned.

## Coverage matrix

| ID | Audit area | Status | Evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | complete — P2 repaired/reverified | `area-b-2026-08-29.md`; #320 / PR #360; schedule-relative reproductive-age boundaries + independent checker; #214 coupling evidence |
| C | Households, kinship, social links, lifecycle structure | complete — P2 repaired/reverified | `area-c-2026-08-29.md`; #324 / PR #358; independent composition checker + #207/#304 reruns |
| D | Resources, condition, subsistence, depletion/recovery | complete — P1 reverified | `area-d-2026-08-29.md`; #326 / PR #347; independent Area D arithmetic checker |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | complete — findings | `area-e-2026-08-29.md`; #327; #214 relevant |
| F | Aggregation and interaction mechanisms | complete — findings | `area-f-2026-08-29.md`; #329 |
| G | Initialization, burn-in, path dependence, continuation state | complete — findings | `area-g-2026-08-29.md`; #320 repaired/reverified; #332 remains open |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | complete — P1 reverified | `area-h-2026-08-29.md`; #334 / PR #352; independent exact-coverage checker |
| I | Sensitivity, uncertainty, convergence, robustness | complete — findings | `area-i-2026-08-29.md`; #336 |
| J | Identifiability, equifinality, calibration, discrimination | complete — P1 reverified | `area-j-2026-08-29.md`; #338 / PR #353; independent Area J checker |
| K | Experiment orchestration, configuration, provenance, reproducibility | complete — P1 reverified | `area-k-2026-08-29.md`; #340 / PR #354; independent Area K checker + real end-to-end replay |
| L | Observability, analysis outputs, statistical summaries | complete — findings | `area-l-2026-08-29.md`; #342; earlier #183/#184/#222/#226/#229 rechecked |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | complete — findings | `area-m-2026-08-29.md`; #344 plus #314/#315/#327/#329 |
| N | Cross-system integration | complete — findings/dependencies | `area-n-2026-08-29.md`; P1 propagation repaired/rechecked; #320 initialization/demography boundary repaired; remaining P2 dependencies below |

## Finding register

| Finding | Severity | Area | Status | Issue | Reverification |
|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift | P2 | A/M/K | open | #314 | n/a |
| AV2-002 — demographic-time contract retains superseded mortality execution | P2 | A/B/M | open | #315 | n/a |
| AV2-003 — founder genealogy admits impossible reproductive chronology | P2 | B/G/C | **fixed + independently reverified** | #320 | PR #360; one-day parent rejection; exact female/male boundaries; custom schedule; resealed-checkpoint resume adversary |
| AV2-004 — household fission derives social composition from PersonId/birth order | P2 | C/N/I | **fixed + independently reverified** | #324 | PR #358; relabelling/newborn/dependency adversaries; M4/M9/checkpoint; #207 + 130-seed #304 evidence |
| AV2-005 — per-boundary ceiling multiplies partial-supply condition deterioration with resolution | **P1** | D/I/N | **fixed + independently reverified** | #326 | PR #347; deficits 1/10/100/500/1000‰ over every P=1..365; mortality/M4 integration checked |
| AV2-006 — M9 travel contract documents superseded lower-CellId equal-cost selection | P2 | E/M/L | open | #327 | n/a |
| AV2-007 — M9.7 narrative provenance/statistics stale relative to reference | P2 | F/K/L/M | open | #329 | n/a |
| AV2-008 — year-zero checkpoint/resume injects extra day-zero metric snapshot | P2 | G/K/L | open | #332 | n/a |
| AV2-009 — Monte Carlo quantile gate can certify under-covered intervals | **P1** | H/I/L/N | **fixed + independently reverified** | #334 | PR #352; exact binomial/order-statistic coverage; infeasible tails fail closed |
| AV2-010 — metadata-only coordinates can masquerade as structural sensitivity | P2 | I/K/N | open | #336 | n/a |
| AV2-011 — identifiability gate ignores stochastic uncertainty in calibration outputs | **P1** | J/H/N | **fixed + independently reverified** | #338 | PR #353; ±0.20 unresolved vs identical estimates at ±0.01 identified |
| AV2-012 — downstream analysis arguments are not bound to executed analysis | **P1** | K/L/N | **fixed + independently reverified** | #340 | PR #354; single authoritative argv/config binding; CLI/config/RNG/observation selectors rechecked |
| AV2-013 — migration-quality point summaries leave run versus move weighting ambiguous | P2 | L/N | open | #342 | n/a |
| AV2-014 — documented sweep derived-analysis schema versions lag executable contracts | P3 | M | open | #344 | n/a |
| AV2-015 — model-semantics changes can leave #304 confirmatory reference stale | **P1** | B/D/H/K/L/N | **fixed + independently reverified** | #350 | PR #351; clean v20 384-run rebaseline + semantics-aware workflow trigger |

## P1 closure evidence

### #326 / PR #347 — partial-supply temporal rounding

- `MODEL_SEMANTICS_ID` advanced v19 → v20.
- Fixed-point per-person residual removes independent upward rounding at each M3 boundary.
- Independent closure arithmetic verified deficits 1, 10, 100, 500, 1000‰ for every `resources.periodsPerYear = 1..365`.
- At 1‰ deficit and annual maximum-loss budget 400, every partition yields `0` whole condition points plus residual `400/1000`, rather than 1→365 whole points.
- Equivalent partial-supply histories feed the same authoritative integer condition into condition-mediated mortality and M4 pressure.
- M7.6, M8.6 and M9.7 references were causally rebaselined and protected CI passed.

### #350 / PR #351 — confirmatory reference semantics binding

- A clean v20 rerun of the frozen #304 design completed **384/384** simulations.
- All three predeclared Monte Carlo precision gates returned `sufficient_stop`.
- High-level recommendation remained `no_universal_demographic_baseline`, while v20 quantitative changes were preserved explicitly rather than hidden.
- The #304 workflow now watches all `anthrosim-core/src/**` changes and binds its canonical result to authoritative `MODEL_SEMANTICS_ID`, preventing future silent semantics drift.

### #334 / PR #352 — exact quantile coverage

- Replaced clipped normal rank approximation with exact finite-sample binomial/order-statistic support.
- At nominal 95% confidence, minimum finite sample support is n=6 for p=.50, n=29 for p=.90, n=59 for p=.95 and n=299 for p=.99 under the declared sample-only interval contract.
- Infeasible combinations emit no valid precision width and fail closed instead of returning false `sufficient_stop`.
- #304 integration reran 384/384 simulations; a machine field diff proved its only compatibility changes were three nested Monte Carlo diagnostic `schemaVersion: 1 -> 2` values, with no numerical/scientific changes.

### #338 / PR #353 — stochastic identifiability

- Identifiability schema v2 distinguishes deterministic outputs from content-addressed Monte Carlo diagnostics.
- Same point estimates 0.00 and 0.10 against target 0.00 ±0.05 are unresolved/non-identifying with ±0.20 process uncertainty, but become acceptable/rejected respectively with adequate ±0.01 uncertainty.
- Unresolved points remain in the compatible region; process noise can widen or preserve uncertainty but cannot spuriously narrow parameter support.
- Dedicated protected identifiability workflow and independent Area J adversarial checker passed.

### #340 / PR #354 — downstream analysis execution provenance

- Schema v2 has one authoritative executed representation: complete `command` argv, with config files separately content-bound as declared artifacts.
- Independently editable `arguments`, `analysisRngSeeds`, and downstream `observationModelIdentity` execution claims were removed from the downstream-analysis schema; descriptive metadata is explicitly non-executed annotation.
- CLI option, config-file mutation, RNG configuration and observation-model selection paths were exercised adversarially.
- Focused Python tests, independent Area K checker, real Rust end-to-end execution/replay, normal CI and applicable scientific/security gates passed.

### #324 / PR #358 — dependency-aware household fission

- `MODEL_SEMANTICS_ID` advanced v20 → v21 and the opt-in structural-sensitivity treatment is now `deterministic_dependency_fission_v2`.
- Household composition no longer comes from contiguous PersonId/storage-order slices: daughter households require independent-age anchors when feasible, dependents preferentially remain with living parents, and fission defers instead of manufacturing child-only autonomous units when there are too few independent anchors.
- Adversarial tests cover scientifically equivalent PersonId relabelling, a 5-adult + 4-newborn threshold crossing, a 1-adult + 8-child defer case, deterministic replay/checkpoint, explicit M4 daughter-household participation and downstream M9 behavior. An independent Area C arithmetic checker reproduces the legacy cohort-sorting failure and repaired invariants without importing AnthroSim.
- The frozen #207 structural-sensitivity design reran 8 paired seeds × 2 arms × 40 years: 16/16 completed, zero extinctions. The qualitative household-structure sensitivity survived, while historical fission-v1 total M3 unmet need 449 fell to 118 under dependency-aware v2 (−73.7%), demonstrating that the old membership artifact materially distorted at least one mechanism-level outcome.
- The first v21 #304 64-seed confirmation completed 384/384 runs but correctly failed its fixed Monte Carlo extinction-precision gate (4/64; Wilson half-width 0.08747 > predeclared 0.085). It was not extended post hoc.
- A new confirmation prospectively froze 130 entirely fresh seeds (`3042001..3042130`), with 130 chosen from the worst-case Wilson precision bound. All three exact precision plans were checked in before execution. The resulting 780/780 arm-runs completed and all three gates returned `sufficient_stop`.
- Under the v21 positive-growth comparison, fixed-founder households ended at mean N=107.31 with late growth −0.0817%/yr and 0/130 extinctions; dependency-aware fission ended at mean N=20.92 with late growth −1.029%/yr and 4/130 extinctions. The paired terminal-population effect was −86.38 people with 95% Monte Carlo half-width 7.04 (<15). The high-level #304 recommendation remains `no_universal_demographic_baseline`.

### #320 / PR #360 — founder reproductive chronology

- Declared founder parenthood is now validated against the experiment's own `DemographyConfig` rather than only requiring the parent to be older than the child or introducing a hidden universal age constant.
- Female parenthood and a female founder's pre-run `lastBirthDay` must fall inside configured fertility-age bands with positive probability support; male parenthood must fall inside `[maleParentMinAgeYears, maleParentMaxAgeYearsExclusive)`.
- Under the default synthetic-validation schedule this means female **18≤age<45** and male **18≤age<70**. Exact one-day-below/at/one-day-below-upper/at-upper boundary tests pass, and a deliberately narrow custom female **21≤age<22** schedule changes admissibility exactly as declared.
- Fresh standard runs, spatial runs, CLI bundle/demography reconstruction and checkpoint resume all bind the same rule to the embedded experiment demography.
- The checkpoint adversary starts from a valid persisted founder genealogy, changes the embedded mother to be one day old at the child's birth, reseals continuation identity, and still fails specifically with `ParentOutsideConfiguredReproductiveAge`; rejection is therefore scientific validation rather than merely a digest mismatch.
- The independent Area B checker reproduces the legacy one-day-parent loophole and all repaired age boundaries without importing AnthroSim. Focused tests, full protected CI, cross-platform/spatial determinism, and the guarded 130-seed / **780-run** #304 confirmation all passed on the final PR head.
- `MODEL_SEMANTICS_ID` remains v21 because this is fail-closed input-validity hardening: previously admissible executable trajectories retain their meaning, while scientifically impossible declared initial histories are rejected before execution.

## Cross-system integration matrix

| Interaction | Current disposition |
|---|---|
| Demography × households | #324 repaired/reverified under v21; #207 and fresh 130-seed #304 reruns confirm strong structural sensitivity without PersonId cohort slicing |
| Demography × resources | #326 P1 repaired and coupled condition/mortality behavior rechecked; remaining interpretation depends on open P2s rather than temporal-rounding defect |
| Households × movement | #324 repaired/reverified; explicit M4 daughter-household participation plus existing M9/checkpoint lifecycle integration pass under dependency-aware fission |
| Movement × resources | #326 P1 repaired; M4 pressure no longer flips solely because the same partial exposure is subdivided differently |
| Aggregation × resources | #326 temporal condition-response defect repaired; remaining aggregation interpretation not blocked by that P1 |
| Initialization × demography | #320 repaired/reverified; founder genealogy and pre-run birth history now fail closed against the experiment's declared reproductive-age support on fresh and resumed paths |
| Initialization × spatial placement | explicit path dependence demonstrated; no new root defect |
| Stochastic inference × censoring/extinction | #334 P1 repaired; existing censoring/nullability governance remains positive |
| Sensitivity × hidden configuration | #336 metadata-only pseudo-structures remains open |
| Calibration × identifiability | #338 P1 repaired; stochastic claims now fail closed on inadequate simulation precision |
| Checkpoint/resume × RNG | positive authoritative continuation evidence; #332 metric-history caveat remains open |
| Observability × scientific interpretation | remaining dependencies are #327/#329/#332/#342/#344 plus broader P2 backlog |

## Historical high-value discovery evidence

These values record the original adversarial failures and should not be mistaken for current v21 behavior after repair:

- Area B: legacy declared founder genealogy accepted a parent only one day older than the child; #320 now rejects that state against the experiment's declared reproductive-age support.
- Area D: legacy 1‰ deficit produced annual condition loss 1→365 as M3 settlements/year rose 1→365; #326 removed this partition-induced multiplication.
- Area H: legacy nominal 95% interval for p=.95,n=8 had only 33.08% exact coverage; #334 now fails such unsupported combinations closed.
- Area J: legacy fixed point estimates were invariant to unrepresented Monte Carlo SE 0.001→1.0; #338 now makes precision part of the decision.
- Area K: legacy `arguments.scale` could change 2→3 while executed argv stayed `--scale 2`, output stayed 10 and replay passed; #340 removed that duplicated executable description.
- Area N: legacy #326 composition could move condition from 999 to 635 and cross default M4 pressure solely through resource-clock subdivision; this specific artefact is repaired under v20.

## Remaining closure work

Before audit v2 is declared fully closed/passed:

1. Triage/repair the remaining P2 findings #314, #315, #327, #329, #332, #336 and #342.
2. Resolve P3 #344 or explicitly disposition it if no longer applicable after intervening schema changes.
3. Repeat any integration/scientific references whose interpretation changes under the remaining repairs.
4. Perform one short final closure/reverification pass against then-current protected main; do **not** restart A–N from scratch unless causal executable semantics materially change.

## Final audit synthesis — current handoff

Audit v2 remains **coverage-complete but not yet backlog-closed**. No P0 finding is open. All P1 findings discovered so far are repaired and independently reverified. P2 #324 and #320 are also repaired/reverified on the v21 line, with #324 supported by fresh structural/confirmatory stochastic evidence and #320 by schedule-relative reproductive-age and resealed-checkpoint adversaries.

The remaining work is now P2/P3 scientific robustness, interpretation, documentation, sensitivity, metric semantics and continuation-observability cleanup. #320 is closed; the next issue should be selected from #314, #315, #327, #329, #332, #336 and #342 only after checking live overlap and dependencies.

No audit result should be interpreted as empirical validation of a prehistoric reconstruction. After the remaining backlog and closure pass, the appropriate next phase remains a bounded empirical case study with explicit evidence roles, sensitivity, identifiability, Monte Carlo precision and held-out corroboration.
