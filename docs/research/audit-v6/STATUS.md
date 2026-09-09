# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed completed-area reports and executable evidence are preserved under `docs/research/audit-v6/` and on the referenced evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target software version | `0.3.6` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **6/14 Areas A–N complete** |
| Discovery result | **non-clean: 7 findings so far — 3 P1, 4 P2; discovery continues through G–N** |
| Authoritative Audit-v6 findings | **7 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2** |
| Open Audit-v6 findings | **7** |
| Open P0/P1 | **3 — AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Phase | **Area G discovery next — initialization, burn-in, path dependence and continuation state** |
| Active ownership | **G — initialization, burn-in, path dependence and continuation state** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings have been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

### Area A — authoritative semantics and scheduler behaviour

Status: **complete — AV6-001 / #687 P1 open**  
Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Fresh evidence: #684 produced 20 exact scheduler-equivalence checkpoint comparisons with no finding; #686/#687 demonstrated same-day M4-before-newly-due-M9 inversion on day 91, **AV6-001 P1**.

### Area B — demography, fertility, mortality, ageing and population structure

Status: **complete — AV6-002 / #694 P2 open**  
Completion report: `docs/research/audit-v6/area-b-2026-09-09.md`

Fresh evidence: #690 was known sequential-RNG/common-random-number scope; #692 passed the birthday-crossing × mortality-cadence quantitative challenge; #693/#694 demonstrated incompatible male-parent age-window time references between dynamic M2 and declared-founder genealogy, **AV6-002 P2**.

### Area C — households, kinship, social links and lifecycle structure

Status: **complete — AV6-003 / #699 P2 open**  
Completion report: `docs/research/audit-v6/area-c-2026-09-09.md`

Fresh evidence: #698/#699 demonstrated external-kin context collapse before the final PersonId fission tie-break, **AV6-003 P2**; corrected #701 preserved physical daughter partitions under source-HouseholdId relabelling, no finding.

### Area D — resources, condition, subsistence, depletion/recovery

Status: **complete — AV6-004 / #707 P1 and AV6-005 / #708 P2 open**  
Completion report: `docs/research/audit-v6/area-d-2026-09-09.md`

Fresh evidence:
- #705/#707: resource equal-remainder allocation changed condition `(1000,0) -> (0,1000)` under pure cell reflection; final head `f8ee562027ae162afb9ce7a5f00a6ca3382b0f33`, CI `34356842006`, job `102483639065`; 284 pre-existing core tests passed; **AV6-004 P1**.
- #706/#708: exact 50/50 M9 duration split changed `(home=1,visitor=0) -> (0,1)` under canonical household-index relabelling; final head `ae667febd0930f43b50eae0af459c5fc8201f13f`, CI `34356930794`, job `102483966686`; 284 pre-existing core tests passed; **AV6-005 P2**.

### Area E — spatial landscape, movement, migration, temporary mobility and boundaries

Status: **complete — AV6-006 / #711 P1 open**  
Completion report: `docs/research/audit-v6/area-e-2026-09-09.md`

Fresh evidence:
- #710/#711: one unreachable impassable padding cell outside an unchanged local two-way M9 tie changed the authoritative destination `CellId(1) -> CellId(3)` at seed 0. Final head `cd1dd83abf1a83911c4c11e3d47fc662175d580a`; CI `34358664992`, job `102489797483`; format/Clippy and all **284 pre-existing core tests** passed before the fresh locality oracle failed. **AV6-006 P1**.
- #713: M4 physical radius-2 candidate geometry remained exactly invariant across right, bottom and combined outer-domain padding after canonical CellIds were erased. Final head `6dff18c0a4488ba329ca5e52f6e02c195b71c63b`; CI `34359990493`, job `102494331225`; clean no finding.
- #714: a boundary-adjacent unique-destination M9 corridor retained exact `2000` accumulated cost, two-edge route distance and two-day outbound/return duration after embedding behind impassable padding. Final head `3b628b01857a145a35edfa5dd9ab641bf85d5df9`; CI `34364050562`, job `102508144241`; all **284 pre-existing core tests** and the fresh oracle passed; clean no finding.

Area E introduced no production semantics changes. AV6-006 remains open and deferred behind the A–N discovery barrier.

### Area F — aggregation and interaction mechanisms

Status: **complete — AV6-007 / #718 P2 open**  
Completion report: `docs/research/audit-v6/area-f-2026-09-09.md`

Fresh evidence:
- #716: repeated aggregation triggers at the active-return/exact-completion boundary behaved exactly as declared. Final head `71a0b48b8bb38a1afffb05ac50585202661f2dbd`; CI `34367396021`, job `102519609766`; format, Clippy, all **284 pre-existing core tests**, the fresh recurrence oracle and the complete workspace passed. A day-106 trigger recorded one explicit `ActiveJourney` skip; a day-107 trigger departed again after same-day completion. Clean no finding.
- #717/#718: two one-person half-open visit intervals touched at day 112 without positive-duration overlap, yet derived observability reported global and cell `peakVisitors=2` instead of the physical maximum 1 while correctly reporting `visitor_person_days=20`. Final head `416da690fa9e1a7e3ab99644096b300f3cce1d39`; CI `34366149670`, job `102515335302`; format/Clippy and all **284 pre-existing core tests** passed before only the fresh peak-occupancy oracle failed. **AV6-007 P2**.

Area F introduced no production semantics changes. AV6-007 remains open and deferred behind the A–N discovery barrier.

## Area G ownership / next discovery session

Area G starts from zero after the Area-F disposition is merged. Before substantive evidence, reconstruct live `main`, open PRs/issues and overlapping initialization/continuation audit work.

Fresh Area-G attacks should challenge genuinely new seams such as:

- alternative scientifically meaningful initial states and whether persistent differences are exposed rather than silently interpreted as burn-in or convergence;
- path dependence under matched later environments/process settings, with explicit transient versus stationary interpretation;
- checkpoint/resume at a continuation boundary that composes multiple causal subsystems in a way not already covered by permanent tests or Audit v5;
- hidden future-defining state that could be lost, regenerated or reinterpreted at resume;
- initialization assumptions whose downstream influence should remain visible in provenance and diagnostics rather than being mistaken for model equilibrium;
- continuation under the repaired v35 M9 coupling/spatial-equivalence semantics without replaying the old v5 population-seed defect.

Historical v5 Area-G evidence is control/duplicate-avoidance context only:

- #635 composed post-fission topology with three simultaneously active M9 journeys across checkpoint/resume and matched uninterrupted execution exactly;
- #636 showed declared founders remain invariant to dormant synthetic-only initialization knobs through active downstream fertility;
- #637 showed a future declared stop horizon does not alter the shared pre-horizon causal trajectory;
- AV5-003/#627 demonstrated spatial M9 history replay using the process seed instead of the population-realization seed, but that defect was repaired by PR #671 before v0.3.6 and has permanent regression coverage.

Do not repeat those as fresh v6 credit.

Known cross-cutting v6 context not to double-count:

- AV6-002/#694 is cross-cutting to Area G because founder/dynamic chronology semantics can affect initialization interpretation;
- other existing v6 findings remain separate unless a fresh Area-G experiment demonstrates a distinct initialization/path-dependence or continuation mechanism.

## Discovery phase rules

- Start every Area at zero coverage.
- Attribute discovery to immutable `v0.3.6` / v35.
- Prior audits/regressions are controls and attack-design context, not v6 completion evidence.
- Each Area requires genuinely fresh falsification-oriented evidence.
- Preserve demonstrated findings before production repair and search open/closed issues/PRs before creating one.
- Assign sequential `AV6-*` identifiers only after distinct scientific defects are demonstrated.
- Continue A–N discovery after findings are recorded; ordinary production remediation begins only after full discovery completes.
- Keep evidence from semantically different heads explicitly separated.
- Do not infer empirical/archaeological validity from framework audit results.

## Discovery coverage matrix

| ID | Area | Status | Fresh v6 evidence / direction |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1** | #684 no finding; #686/#687 P1. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV6-002 P2** | #690 known coupling scope; #692 no finding; #693/#694 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV6-003 P2** | #698/#699 P2; #701 no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV6-004 P1 + AV6-005 P2** | `area-d-2026-09-09.md`; #705/#707; #706/#708. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — AV6-006 P1** | `area-e-2026-09-09.md`; #710/#711 P1; #713 and #714 clean controls. |
| F | Aggregation and interaction mechanisms | **complete — AV6-007 P2** | `area-f-2026-09-09.md`; #716 clean recurrence control; #717/#718 P2. |
| G | Initialization, burn-in, path dependence, continuation state | **next/active** | Fresh alternative-start, path-dependence and continuation-state attacks on repaired v35 baseline. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **not started** | Seed/stream identity, draw ordering/coupling, rare events, stopping rules, replicate sufficiency, censoring, precision. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Parameter/structure/horizon/resolution/initialization/replicate sensitivity and hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination, tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, identities, artifact integrity, replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality, incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases, benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | State |
|---|---:|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F/D/N | #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; M4 seq1 then M9 seq2 day91 | **#687** | open; remediation deferred |
| `AV6-002` | **P2** | B primary; C/G/M/N | #693 head `579cc00...`; dynamic/founder male-age reference inversion | **#694** | open; remediation deferred |
| `AV6-003` | **P2** | C primary; E/N | #698 head `3733700...`; external-kin orientation flips under PersonId relabelling | **#699** | open; remediation deferred |
| `AV6-004` | **P1** | D primary; E/H/N | #705 head `f8ee562...`; CI `34356842006` job `102483639065`; condition flips under cell reflection | **#707** | open; remediation deferred |
| `AV6-005` | **P2** | D primary; C/E/F/N | #706 head `ae667fe...`; CI `34356930794` job `102483966686`; duration split flips under household-index relabelling | **#708** | open; remediation deferred |
| `AV6-006` | **P1** | E primary; F/H/I/N | #710 head `cd1dd83...`; CI `34358664992` job `102489797483`; unreachable padding changes `CellId(1) -> CellId(3)` at seed0 | **#711** | open; remediation deferred |
| `AV6-007` | **P2** | F primary; E/L/N | #717 head `416da690...`; CI `34366149670` job `102515335302`; touching visits `[102,112)` and `[112,122)` report peak 2 instead of physical maximum 1 | **#718** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:
1. finish fresh discovery through Areas G–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–F are complete**. Area G is **next/active**. Seven findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1 and AV6-007/#718 P2.

Next action: begin **Area G — initialization, burn-in, path dependence and continuation state** from zero against immutable v0.3.6/v35. Reconstruct live state and overlap before substantive evidence; use v5 Area-G experiments, repaired AV5-003/#627 and permanent continuation regressions only for duplicate avoidance, controls and attack design.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and update this ledger before handoff.
