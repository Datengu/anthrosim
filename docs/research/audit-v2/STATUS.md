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
| Current audit findings | #314, #315, #320, #324 open (all P2); #317 is non-scientific CI infrastructure |
| Audit state | in progress |

If `main` advances during the audit, add the new SHA to the session log and state whether earlier evidence remains applicable.

## Coverage matrix

Statuses: `not started`, `in progress`, `complete — no finding`, `complete — findings`, `blocked`, `repeat required`.

| ID | Audit area | Status | Evidence / notes | Findings / issues |
|---|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | complete — findings | `area-a-2026-08-29.md`; exact source/docs review plus `area-a-scheduler-audit.py` | #314, #315 |
| B | Demography, fertility, mortality, ageing, population structure | complete — findings | `area-b-2026-08-29.md`; existing integrated acceptance suite plus adversarial PR #319 CI probes | #320; #315 documentation-relevant; #214 additional coupling evidence |
| C | Households, kinship, social links, lifecycle structure | complete — findings | `area-c-2026-08-29.md`; source/integration review plus adversarial PR #323 CI probes | #324; #320 founder-to-kin relevant |
| D | Resources, condition, subsistence, depletion/recovery | not started | — | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | not started | — | — |
| F | Aggregation and interaction mechanisms | not started | — | — |
| G | Initialization, burn-in, path dependence, continuation state | not started | — | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | not started | Area B produced a 2,000-seed PersonId relabelling coupling experiment; full RNG/inference audit remains deferred | #214 historical evidence relevant |
| I | Sensitivity, uncertainty, convergence, robustness | not started | — | — |
| J | Identifiability, equifinality, calibration, discrimination | not started | — | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | not started | — | — |
| L | Observability, analysis outputs, statistical summaries | not started | Area C exposed a hypothesis that recorded-genealogy generation span may be a lower bound when founder genealogy is unspecified | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | Area A exposed two documentation/provenance findings, but Area M has not been systematically audited | #314, #315 relevant |
| N | Cross-system integration | not started | Area C exposed an households × M9 timing hypothesis when an oversized household is away exactly at the annual fission boundary | — |

## Cross-system integration checklist

These checks are mandatory after the corresponding subsystem work is mature enough to support them.

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | Areas B/C locally verified birth/fission ordering and exposed #324; dedicated integration pass remains required |
| Demography × resources | not started | — |
| Households × movement | not started | Area C showed child/newborn-only fission households are eligible future household units; away-at-annual-boundary fission deferral remains for dedicated integration |
| Movement × resources | not started | — |
| Aggregation × resources | not started | — |
| Initialization × demography | not started | Area B finding #320 shows declared founder reproductive chronology can be biologically impossible; full Area G integration remains required |
| Initialization × spatial placement | not started | — |
| Stochastic inference × censoring/extinction | not started | — |
| Sensitivity × hidden configuration | not started | — |
| Calibration × identifiability | not started | — |
| Checkpoint/resume × RNG | not started | — |
| Observability × scientific interpretation | not started | Area C generation-span lower-bound hypothesis carried to Area L |

## Finding register

Add every scientifically substantive finding here, including findings that are later closed.

| Finding | Severity | Area | Status | Issue/PR | Evidence location | Reverified? |
|---|---|---|---|---|---|---|
| AV2-001 — current-facing model-semantics identity drift (v15 documented vs executable/tagged v19) | P2 | A / M / K | issue open | #314; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |
| AV2-002 — M2 demographic-time contract retains superseded annual-boundary mortality execution | P2 | A / B / M | issue open | #315; audit ledger PR #316 | `area-a-2026-08-29.md` | n/a — open |
| AV2-003 — declared founder reproductive chronology checks ordering but admits biologically impossible event ages | P2 | B / G / C | issue open | #320; adversarial harness #319 | `area-b-2026-08-29.md` | n/a — open |
| AV2-004 — deterministic household fission derives social composition from PersonId/birth-append order | P2 | C / N / I | issue open | #324; adversarial harness #323 | `area-c-2026-08-29.md` | n/a — open |

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

### 2026-08-29 — Area B / demography, fertility, mortality, ageing and population structure

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** `06f1a059b23b46f6bab79672de7f9685529058e0`; rechecked after adversarial work and unchanged.

**Release / model semantics:** `v0.3.2`; `anthrosim-model-semantics-v19`.

**Audit area / sub-area:** Area B — age-band execution, competing/background mortality, fertility conditioning, spacing, newborn state, parentage locality, founder reproductive chronology/genealogy, extinction/growth controls and demographic RNG assignment under identity relabelling.

**Overlap check:**
- open issues at session start: #314, #315 and non-scientific CI issue #317;
- open PRs at session start: none;
- parallel-agent overlap: none visible;
- temporary audit-only draft PR #319 was created for adversarial execution and closed without merge after evidence preservation;
- new scientific finding issue #320 created;
- final overlap recheck before ledger work: main unchanged, open scientific findings #314/#315/#320, no open PRs.

**Implementation/docs inspected:** `demography.rs`, `founder_initialization.rs`, `config.rs`, `population.rs` behaviour through tests, `m2_demographic_acceptance_tests.rs`, current core regression output, prior #239/PR #303 growth-control evidence, prior #304/PR #307/#308 demographic-baseline evidence, #214/PR #302 paired-seed semantics, and relevant historical M2 findings/repairs.

**Tests/experiments performed:**
- reviewed and re-executed the current integrated workspace/core acceptance surface on the PR #319 merge head; all 260 pre-existing core tests passed before the deliberately failing audit assertion;
- constructed an executable one-day parent/child founder-genealogy counterexample and confirmed current `Simulation::new` accepts it;
- ran a 2,000-seed PersonId relabelling metamorphic experiment under isolated one-year background mortality, with an exact analytical sequential-stream expectation;
- searched existing issues/PRs before dispositioning both results.

**Quantitative results:**
- impossible founder genealogy: parent born day -101 and child day -100 accepted; one-day parent age at childbirth;
- PersonId relabelling experiment: seeds `1..=2000`, 2 founders, 1 year, 1 cell, one M3 period, mortality risks 0.2 and 0.8, fertility/condition mortality/M4 disabled;
- exact sequential-stream prediction for paired terminal-count disagreement: `2 * 0.6 * 0.4 = 48%`;
- observed paired disagreement: `976/2000 = 48.8%`; first mismatch seed 2 (`1` survivor vs `0`); binomial Monte Carlo SE around the 48% reference is about 1.12 percentage points, so observed excess is about 0.7 SE;
- paired population-difference expectation is zero by exchange symmetry, so the result diagnoses common-random-number/identity coupling rather than marginal ensemble bias;
- prior demographic control evidence reviewed: #239 controls bracket intrinsic replacement; #304 preserved 288-run exploratory plus independent 384-run confirmation showing strong household-lifecycle dependence of realized growth/mate limitation.

**Findings:**
- AV2-003 / P2: declared founder reproductive chronology validates ordering but not biological plausibility; impossible parent age at child birth and analogous implausible pre-run `lastBirthDay` can pass — #320;
- #315 remains a current documentation defect but executable M2 mortality/fertility ordering reviewed here agrees with the later competing-risk implementation;
- no new defect demonstrated in age-band execution, mortality competition, spacing normalization, newborn condition, parentage locality, extinction accounting or explicit demographic growth-control policy;
- the 48.8% PersonId relabelling effect is additional evidence for historical #214/PR #302 sequential-stream coupling, not a separate demographic finding.

**Issues/PRs created or relevant:** #320; closed-unmerged audit harness #319; #214 received quantitative relabelling evidence; #315 remains relevant documentation drift. #239/PR #303 and #304/PR #307/#308 provide existing growth/structural evidence.

**Unresolved hypotheses:** whether sequential draw assignment plus state-dependent eligibility can create marginal estimator bias or invalid paired uncertainty in realistic multi-agent comparisons remains for Area H; whether founder chronology validation and household/kin lifecycle interactions create further structural constraints belongs to Areas C/G.

**Explicitly not examined:** empirical calibration of fertility/mortality/parent ages; full household lifecycle and kin graph semantics; burn-in/initialization sensitivity; RNG/Monte Carlo inference beyond the controlled relabelling diagnostic; global sensitivity; identifiability; archaeological demographic validity.

**Does evidence need repeating after main changes?** Repeat the founder counterexample if founder/demography validation changes, and repeat the relabelling experiment if RNG draw assignment or mortality iteration semantics change. Purely documentation-only changes do not invalidate the executable evidence. #320 requires independent re-verification after repair if its eventual severity/scope warrants it; Area H should independently revisit #214 coupling evidence under its own inference questions.

**Recommended next step:** Area C — households, kinship, social links and lifecycle structure. Attack household fission/dissolution invariants, reciprocal kin graph consistency, irrelevant relationship/person-order invariance, and whether household lifecycle changes reproduction/mobility through hidden structural constraints. Keep #320 visible as a founder-to-kin integration dependency without double-counting it.

---

### 2026-08-29 — Area C / households, kinship, social links and lifecycle structure

**Date / agent:** 2026-08-29 / ChatGPT scientific-audit agent

**Live main SHA:** Area C executable audit began on `06f1a059b23b46f6bab79672de7f9685529058e0`; during the session Area B documentation-only PR #321 merged, advancing protected main to `fdeb66ed0e05683fd5092f3e1ec8407df1bbcfe4`. The executable household/kin/lifecycle tree remained unchanged, so prior Area C evidence remains applicable to the latest audited main.

**Release / model semantics:** `v0.3.2`; `anthrosim-model-semantics-v19`.

**Audit area / sub-area:** Area C — household lifecycle partition semantics, fission state/events/topology, reciprocal first-degree kin anchors, M4/M9 household-unit implications, lifecycle observability and structural-sensitivity interpretation.

**Overlap check:**
- open scientific findings entering Area C: #314, #315, #320; #317 was CI infrastructure only;
- no parallel product PR overlapped the household lifecycle surface;
- disposable audit-only PR #323 was created for adversarial execution and closed without merge after evidence preservation;
- new Area C scientific finding #324 created;
- main advance during session was Area B audit documentation only.

**Implementation/docs inspected:** `household_lifecycle.rs`, `population.rs` fission implementation, `migration.rs` reciprocal kin-anchor construction, `temporary_mobility.rs` household trigger/topology semantics, `events.rs` fission record, `household_observability.rs`, `simulation.rs` annual lifecycle scheduling, `household_lifecycle_sensitivity.rs`, historical #188 kin repair, #207/PR #305 lifecycle structural sensitivity, and #304/PR #307/#308 replicated demographic-baseline analysis.

**Tests/experiments performed:**
- inspected existing balanced-cap, checkpoint/resume and future-M9 household-lifecycle acceptance;
- verified reciprocal parent/child kin anchors retain all unique locations independent of person-record order;
- ran a pure PersonId relabelling metamorphic test on an eight-person one-household founder population with all substantive state held fixed;
- ran a controlled same-boundary birth/fission test with four certain births into a five-adult household and fission cap 5;
- strengthened audit head `d298ffe1cf4c3f623458672315774f8fcc531546` passed format, Clippy and complete workspace tests in CI run `33235309035`, quality job `99055150212`.

**Quantitative results:**
- PersonId relabelling: same eight ages `[60, 50, 40, 30, 20, 15, 10, 5]`, same reproductive sex/residence/condition, zero mortality/fertility/resource mortality/M4; under cap 4, reversing only age-to-ID assignment changed the source household from the four oldest records to the four youngest — probe passed;
- same-day birth/fission: 5 adults + 4 certain births = 9 living people; cap 5 yields balanced `5 + 4`; IDs 1–5 founders remained source household while newborn IDs 6–9 all became the new household — probe passed;
- audit harness: 2/2 adversarial tests passed; all 260 pre-existing core tests and 3/3 existing household-lifecycle sensitivity tests also passed;
- prior #304 confirmation context: positive-schedule late growth about `-0.002%/yr` fixed vs `-0.994%/yr` fission; mate limitation `11.3%` vs `38.7%`; mean year-240 population `108.2` vs `28.9`.

**Findings:**
- AV2-004 / P2: `deterministic_size_fission_v1` treats stable PersonId order as a “neutral” household partition, but PersonId is birth-append order for model-born people, producing arbitrary cohort/generation sorting and potentially newborn-only autonomous household units — #324;
- #320 remains relevant founder-to-kin validity evidence from Area B but is not double-counted as a second Area C finding;
- no separate defect demonstrated in reciprocal kin anchoring, exact fission event membership, balanced cap enforcement, M9 topology extension or checkpoint/resume determinism.

**Issues/PRs created or relevant:** #324; closed-unmerged adversarial PR #323; #320 cross-area relevant; historical #188, #207/PR #305 and #304/PR #307/#308 supply comparison/interpretation evidence.

**Unresolved hypotheses:**
- an oversized household temporarily away on the exact annual lifecycle boundary is deliberately skipped and may defer fission until a later year; assess under households × movement integration rather than treating as a subsystem defect here;
- household generation-span observability follows recorded genealogy and may be a lower bound when founder genealogy is `Unspecified`; assess under Area L.

**Explicitly not examined:** empirical/culturally specific household formation, marriage/residence systems, archaeological validation of household size/composition, dedicated demography × household and households × movement integration counterfactuals beyond the controlled #324 probes.

**Does evidence need repeating after main changes?** Repeat if household partition membership, annual lifecycle timing, kin-anchor construction, M4/M9 household eligibility, fission event replay, or household topology continuation changes. After #324 repair, rerun #207 structural sensitivity and relevant #304 replicated comparisons; prior fission-arm effect sizes should not be assumed to survive unchanged.

**Recommended next step:** Area D — resources, condition, subsistence, depletion and recovery. Start from current live main, verify overlap, then attack exact stock/need conservation, remainder/tie allocation, initialization and storage capacity, seasonal/elapsed-time integration, condition response and recovery, condition-mediated mortality, and temporal observability under adversarial scarcity/recovery regimes.

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