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
| Latest audited protected-main SHA | `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4` |
| Latest audited model semantics | `anthrosim-model-semantics-v19` |
| Current audit findings | #326 P1; #314, #315, #320, #324, #327, #329, #332 P2; #317 is non-scientific CI infrastructure |
| Audit state | in progress |

`main` advanced from `06f1a059...` to `fdeb66ed...` only through merged Area B audit documentation. The executable tree audited by Areas C–G remained model semantics v19. Area C recording PR #325, Area D recording branch, Area E recording PR #328 and Area F recording PR #330 were repository-visible before Area G; their evidence is incorporated below so later agents do not duplicate those areas.

## Coverage matrix

Statuses: `not started`, `in progress`, `complete — no finding`, `complete — findings`, `blocked`, `repeat required`.

| ID | Audit area | Status | Evidence / notes | Findings / issues |
|---|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; exact source/docs review plus `area-a-scheduler-audit.py` | #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | complete — findings | `area-b-2026-08-29.md`; integrated acceptance suite plus adversarial PR #319 CI probes | #320; #315 documentation-relevant; #214 coupling evidence |
| C | Households, kinship, social links, lifecycle structure | complete — findings | `area-c-2026-08-29.md` on recording PR #325; adversarial PR #323 probes | #324; #320 founder-to-kin relevant |
| D | Resources, condition, subsistence, depletion/recovery | complete — findings | `area-d-2026-08-29.md` and `area-d-condition-rounding-audit.py` on `audit-v2/area-d-resources-record` | #326 P1; #324 household-unit interaction relevant |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | complete — findings | `area-e-2026-08-29.md`; `area-e-spatial-audit.py`; source/contracts plus existing spatial-boundary/grid tests | #327; #214 coupling relevant; #324/#326 cross-system dependencies |
| F | Aggregation and interaction mechanisms | complete — findings | `area-f-2026-08-29.md`; `area-f-aggregation-audit.py`; M9.7 raw-reference recomputation and same-total-use adversarial construction | #329; #327 destination semantics relevant; #324/#326 cross-system dependencies |
| G | Initialization, burn-in, path dependence, continuation state | complete — findings | `area-g-2026-08-29.md`; source/contracts review; controlled founder-layout evidence; adversarial year-zero checkpoint PR #331 | #332; #320 initialization chronology relevant; historical #168 substantially reverified |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | not started | Area B 2,000-seed PersonId relabelling diagnostic; Area E candidate-order reflection diagnostic; full inference audit deferred | #214 historical evidence relevant |
| I | Sensitivity, uncertainty, convergence, robustness | not started | Area D finite-capacity timing and Area E finite-boundary effects explicitly deferred to convergence testing | #326 repair dependency |
| J | Identifiability, equifinality, calibration, discrimination | not started | Area F confirms total focal person-days alone are non-identifying but M9 preserves additional temporal/journey structure | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | not started | Area F exposed stale current M9.7 benchmark narrative relative to machine-readable reference; Area G exposed year-zero resume output-history drift | #329, #332 |
| L | Observability, analysis outputs, statistical summaries | not started | Area C generation-span lower-bound hypothesis; Area E M9 destination-contract interpretation; Area F aggregation summaries/journey observability; Area G year-zero metric injection | #327, #329, #332 |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | Areas A/E/F exposed current-facing documentation/result drift, but Area M has not been systematically audited | #314, #315, #327, #329 |
| N | Cross-system integration | not started | Local subsystem passes have identified mandatory dependencies below | #324, #326 |

## Cross-system integration checklist

These checks are mandatory after the corresponding subsystem work is mature enough to support them.

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | Areas B/C locally verified ordering and exposed #324; dedicated integration pass remains required |
| Demography × resources | not started | #326 can alter condition-mediated mortality trajectories; repeat after repair |
| Households × movement | not started | #324 can create cohort/newborn-sorted autonomous M4/M9 household units; away-at-annual-boundary fission deferral remains relevant |
| Movement × resources | not started | #326 changes condition pressure seen by M4; M9 visitors alter destination demand seen by M4; dedicated post-repair pass required |
| Aggregation × resources | not started | Area F locally verified duration-weighted visitor demand and visitor-aware M4 cue; dedicated integration remains required after #326 repair |
| Initialization × demography | not started | #320 admits biologically impossible declared-founder reproductive chronology |
| Initialization × spatial placement | not started | Area G confirms exact declared placement and a controlled no-relaxation case where founder-layout dependence persists; dedicated empirical/sensitivity integration remains required |
| Stochastic inference × censoring/extinction | not started | — |
| Sensitivity × hidden configuration | not started | — |
| Calibration × identifiability | not started | — |
| Checkpoint/resume × RNG | not started | Area G source review confirms v19 continuation identity binds named RNG positions and ordinary year-3 resume equivalence; dedicated Area H adversarial inference check remains required |
| Observability × scientific interpretation | not started | Area C generation-span hypothesis; #327 stale M9 destination semantics; #329 stale current M9.7 result narrative; #332 year-zero checkpoint metric injection |

## Finding register

Add every scientifically substantive finding here, including findings that are later closed.

| Finding | Severity | Area | Status | Issue/PR | Evidence location | Reverified? |
|---|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift (v15 documented vs executable/tagged v19) | P2 | A / M / K | issue open | #314; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |
| AV2-002 — M2 demographic-time contract retains superseded annual-boundary mortality execution | P2 | A / B / M | issue open | #315; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |
| AV2-003 — declared founder reproductive chronology checks ordering but admits biologically impossible event ages | P2 | B / G / C | issue open | #320; adversarial harness #319 | `area-b-2026-08-29.md` | n/a — open |
| AV2-004 — deterministic household fission derives social composition from PersonId/birth-append order | P2 | C / N / I | issue open | #324; adversarial harness #323; recording PR #325 | `area-c-2026-08-29.md` | n/a — open |
| AV2-005 — per-boundary integer ceiling multiplies partial-supply condition deterioration as M3 resolution increases | P1 | D / I / N | issue open | #326; Area D recording branch | `area-d-2026-08-29.md`; `area-d-condition-rounding-audit.py` | required after repair |
| AV2-006 — active M9.4 travel contract still documents superseded lower-CellId equal-cost destination selection after keyed repair | P2 | E / M / L | issue open | #327; Area E recording PR #328 | `area-e-2026-08-29.md`; `area-e-spatial-audit.py` | n/a — open |
| AV2-007 — current M9.7 aggregation benchmark narrative reports obsolete v10 provenance/statistics relative to checked-in v18 reference | P2 | F / K / L / M | issue open | #329; Area F recording branch | `area-f-2026-08-29.md`; `area-f-aggregation-audit.py` | n/a — open |
| AV2-008 — year-zero checkpoint/resume injects a derived day-zero metric snapshot absent from uninterrupted execution | P2 | G / K / L | issue open | #332; adversarial harness PR #331 | `area-g-2026-08-29.md` | n/a — open |

Suggested finding statuses: `hypothesis`, `demonstrated`, `issue open`, `fix in progress`, `fixed`, `reverified`, `not a defect`, `accepted limitation`.

## Session log

### 2026-08-29 — Area A / authoritative semantics and scheduler behaviour

**Live main:** `698b0f79827ef9ea11d9eac2fe3ec23b7125e180`; model semantics v19.

**Evidence:** source/parity review plus independent `area-a-scheduler-audit.py`. All 365 allowed resource-clock partitions were valid; all `365 x 365 = 133,225` resource/migration clock pairs had correct merged scheduling; 2,555 exact annual-background-mortality partition compositions matched complete-year survival. Findings: AV2-001/#314 and AV2-002/#315; no executable scheduler defect demonstrated. Repeat if scheduler/mortality/M9 boundary semantics change.

**Handoff:** Area B.

---

### 2026-08-29 — Area B / demography, fertility, mortality, ageing and population structure

**Live main:** `06f1a059b23b46f6bab79672de7f9685529058e0`; model semantics v19.

**Evidence:** integrated workspace/core suite on temporary audit PR #319; one-day impossible-parent-age founder counterexample; 2,000-seed PersonId relabelling experiment. Impossible founder genealogy with a one-day-old parent was accepted. Relabelling produced `976/2000 = 48.8%` paired terminal-count disagreement against a 48% sequential-stream expectation, additional evidence for historical #214 rather than marginal bias. Finding AV2-003/#320. No additional age-band, competing-mortality, spacing, newborn, locality, extinction or growth-control defect demonstrated.

**Handoff:** Area C.

---

### 2026-08-29 — Area C / households, kinship, social links and lifecycle structure

**Repository-visible recording state:** PR #325, head `a2a6128d6b5178dbdf9c9e8b2489041b84875717`, based on executable v19 main. Temporary adversarial PR #323 was closed without merge after its CI evidence was preserved.

**Evidence:** fission composition relabelling and same-boundary birth/fission probes plus all 260 pre-existing core tests and 3 household-lifecycle sensitivity tests on the audit harness. Finding AV2-004/#324: `deterministic_size_fission_v1` slices stable PersonId/birth-append order, making supposedly neutral fission cohort/generation dependent and permitting same-day newborn-only households. Existing #304 structural-sensitivity magnitude must be reverified after repair. No second Area C mechanism defect demonstrated.

**Handoff:** Area D. If PR #325 has not merged, do not duplicate Area C; rebase/preserve its documentation instead.

---

### 2026-08-29 — Area D / resources, condition, depletion and recovery

**Repository-visible recording state:** branch `audit-v2/area-d-resources-record`, report head `20f8ca1c3d9429b0ac19e440a55ce0218fa45dbd`; executable baseline `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`, model semantics v19.

**Evidence:** source/contracts review plus independent integer checker `area-d-condition-rounding-audit.py`. The annual maximum condition-loss budget for reference-quarter max 100 remains exactly 400 under all `P=1..365`, but a constant 1-permille supply deficit realizes losses `1..365` as P increases because every interval independently ceilings a positive fractional loss. Representative deficit/P tables are preserved in `area-d-2026-08-29.md`.

**Finding:** AV2-005/#326, P1. This violates the declared temporal-response contract and can propagate to condition-mediated mortality and M4 pressure. No second Area D mechanism defect demonstrated.

**Handoff:** #326 requires an independent post-fix re-verification. Do not duplicate Area D while its recording branch remains active; proceed to Area E/F or non-overlapping work.

---

### 2026-08-29 — Area E / spatial landscape, permanent movement, temporary mobility and boundaries

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`; rechecked before recording work and unchanged.

**Release / model semantics:** `v0.3.2`; `anthrosim-model-semantics-v19`.

**Audit area / sub-area:** Area E — grid/CRS geometry, movement cell-size contract, M4 candidate/utility/choice geometry, M9 edge/routing/tie/presence semantics, finite boundaries and extent sensitivity.

**Overlap check:**
- open scientific issues before Area E included #314/#315/#320/#324/#326; #317 non-scientific CI;
- PR #325 was active Area C documentation-only work;
- `audit-v2/area-d-resources-record` actively owned Area D and contained #326 evidence;
- no open executable Area E product PR was found;
- historical #190/#211/#214 were checked before dispositioning possible rediscoveries.

**Implementation/docs inspected:** `landscape.rs`, `migration.rs`, `temporary_travel.rs`, `temporary_mobility.rs`, `spatial_boundary.rs`, `spatial_grid_geometry.rs`, `spatial_boundary_dependence.rs`, `migration-v0.1.md`, `m9-temporary-travel-semantics-v1.md`, and relevant historical issue contracts.

**Tests/experiments performed:** independent `area-e-spatial-audit.py` candidate/reflection and keyed-tie checker; source-level adversarial review against existing grid/boundary regression fixtures; existing issue search before #327 creation.

**Quantitative results:**
- M4 radii `1..=32`: exact interior count `2r(r+1)` in every case; 0 reflected candidate-set failures;
- corner/interior counts at r=1/2/3/4/8/16/32: `2/4`, `5/12`, `9/24`, `14/40`, `44/144`, `152/544`, `560/2112`;
- stable M4 candidate-order X-reflection mismatches at the same radii: `2/4`, `8/12`, `18/24`, `32/40`, `128/144`, `512/544`, `2048/2112`; this is recorded as #214-style common-random-number coupling evidence, not a marginal spatial-bias finding;
- M9 flat 3x1 two-way equal-minimum keyed selection across tie seeds `1..=100000`: `49,886` vs `50,114` selections (49.886% / 50.114%), versus the stale contract's implied 100% smaller-CellId choice.

**Findings:**
- AV2-006 / P2 / #327: `m9-temporary-travel-semantics-v1.md` still presents the superseded smaller-CellId equal-cost winner although current v19 code preserves all equal minima and resolves destination by keyed seed+origin+household+trigger policy;
- no new executable movement/routing defect demonstrated in the audited surfaces.

**Issues/PRs created or relevant:** #327 created. Historical #190 is the repaired executable antecedent; #211 covers explicit finite-boundary dependence; #214 covers sequential-stream paired-seed interpretation. #324 and #326 remain mandatory cross-system dependencies rather than duplicate Area E findings.

**Unresolved hypotheses:** whether M4/M9 conclusions converge under empirical landscape buffers/resolutions belongs to Area I; candidate-order common-random-number consequences belong to Area H; household/movement and movement/resource feedback require post-#324/#326 integration tests in Area N.

**Explicitly not examined:** empirical validity/calibration of movement-cost transforms, M4 radius/frequency/weights, M9 travel capacity, historical routes/entrances, real focal-region definition, or real landscape boundary placement; systematic Areas F–N remain incomplete.

**Does evidence need repeating after main changes?** Repeat if grid geometry, M8 movement binding, M4 candidate/choice mechanics, M9 travel/tie/presence semantics, or spatial-boundary machinery changes. After #327 repair, recheck all current-facing M9/ODD/TRACE statements. After #324/#326 repair, rerun affected cross-system movement checks.

**Recommended next step:** Area F — aggregation and interaction mechanisms. Attack aggregation trigger/occupancy semantics, duration/exposure accounting, resource feedback, equality/boundary cases and whether the controlled aggregation benchmark distinguishes continuous residence from intermittent aggregation without hidden weighting/timing artifacts.

---

### 2026-08-29 — Area F / aggregation and interaction mechanisms

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`; model semantics v19.

**Repository-visible audit state:** Area C PR #325 and Area E PR #328 open; Area D recording branch active; no overlapping Area F branch/PR found before work began.

**Implementation/docs inspected:** M9 temporary mobility, observability and resource ledgers; M9.5 resource acceptance fixture; temporary-mobility and M9 duration-aware resource contracts; visitor-aware M4 demand contract; frozen M9.7 benchmark contract, human-readable result, machine-readable reference and regression verifier.

**Tests/experiments performed:** independent `area-f-aggregation-audit.py` raw-reference recomputation plus an adversarial same-total-use construction (`20 x 30 days = 10 x 60 days = 600 visitor-person-days`). Existing historical #190/#194/#196 were checked before filing new work.

**Quantitative results:** for 8 paired seeds `9701..9708`, 3650-day horizon, independent recomputation gives median added focal exposure `31.260722` permille, maximum `36.572557`; median peak-visitor share `432.332993` permille and minimum `404.507517`. All preserved pairs retain equal resident person-days, zero control visitor use, exactly 270 treatment visitor-presence days, positive completed journeys and zero permanent migration/condition-mediated deaths. The frozen <=50 permille exposure and >=250 permille peak thresholds remain satisfied.

**Finding:** AV2-007/#329 P2. `m9-controlled-aggregation-benchmark-result.md` presents a v10 run as the current reference and reports 36/426/387 permille max/median/min summary values, while the checked-in machine reference is v18 and reports 37/432/405 after rounding. The capability classification remains supported; the defect is current-facing TRACE/result provenance and quantitative narrative drift.

**Accepted capability boundary:** M9 models temporary presence, travel and resource consequences, not social motive, encounter networks or interaction effects. Journey-level household timing is preserved so co-presence can be reconstructed externally; no defect was filed solely for intentionally absent social interaction semantics.

**Cross-system dependency:** local Aggregation x resources behaviour is internally coherent, but the mandatory integration check remains not started until #326 is repaired because resource-resolution dependence can change crowded-period condition consequences. #324 can also change the household units participating in M9.

**Does evidence need repeating after main changes?** Repeat if M9 trigger/presence timing, duration-aware M3 claims, visitor-aware M4 resource cues, temporary observability or M9.7 reference machinery changes. Recheck #329 after repair and perform dedicated aggregation/resource integration after #326 repair.

**Recommended next step:** Area G — initialization, burn-in, path dependence and continuation state. Do not duplicate any newer repository-visible branch that has already claimed it.

---

### 2026-08-29 — Area G / initialization, burn-in, path dependence and continuation state

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`; model semantics v19.

**Repository-visible audit state:** Area F PR #330 was the latest audit handoff and explicitly assigned Area G. Historical checkpoint-integrity issue #168 was closed; no overlapping Area G branch or duplicate year-zero checkpoint issue was found before the adversarial probe.

**Implementation/docs inspected:** `simulation.rs`, `checkpoint.rs`, `metrics.rs`, checkpoint persistence tests, spatial founder initialization tests, founder/resource/spatial initialization contracts, long-run-regime diagnostics, and historical #168.

**Tests/experiments performed:** controlled founder-layout persistence evidence plus temporary adversarial PR #331 comparing an uninterrupted two-year run with the same run checkpointed/resumed at year 0. The probe required equal terminal `stateDigest64`, equal metric-snapshot days and complete final-checkpoint equality after removing only resume lineage. Formatting and Clippy passed; workspace tests failed on the adversarial equivalence probe.

**Quantitative results:** controlled no-relaxation spatial founders remain materially distinct after two years: concentrated terminal largest-cell share `1000` permille vs split-layout `500` permille. For the checkpoint probe, source control flow gives uninterrupted metric days `[365, 730]` versus year-zero-resumed `[0, 365, 730]`; the extra observation is introduced by `checkpoint_at_year(0)` calling `ensure_terminal_metric_snapshot()` before any model time elapses.

**Finding:** AV2-008/#332 P2. A valid year-zero checkpoint/resume changes retained authoritative metric history despite preserving the terminal scientific present state. This is output/observability continuation drift, not a demonstrated dynamics divergence.

**Positive continuation evidence:** the #168 continuation-identity repair is present and binds experiment, time/stop boundary, population/world/resources/migration/M9 state, all named RNG positions, retained events/metrics, state digest and lineage. Ordinary year-3 checkpoint persistence requires exact uninterrupted/resumed manifest/checkpoint equivalence after removing lineage.

**Initialization/burn-in disposition:** founder/resource/spatial contracts correctly expose initialization as causal uncertainty, and the long-run contract rejects duration alone as equilibrium evidence. Existing #320 remains the founder chronology defect; no second initialization/burn-in mechanism defect was demonstrated.

**Cross-system dependency:** #320 affects initialization×demography; #326 can alter resource-mediated initialization sensitivity; #332 crosses into Areas K/L. Checkpoint/resume×RNG receives positive local evidence but systematic stochastic-inference testing remains Area H.

**Does evidence need repeating after main changes?** Repeat if founder chronology/materialization, resource initialization, spatial founder binding, checkpoint metric cadence, continuation identity/resume validation or long-run diagnostic semantics change. Recheck #332 after repair and resource-mediated initialization claims after #326 repair.

**Recommended next step:** Area H — stochasticity, RNG, ensembles and Monte Carlo inference. Do not duplicate historical #168; attack current seed/stream semantics, paired designs, ensemble estimands and stochastic inference directly.

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
