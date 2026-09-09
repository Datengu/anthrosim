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
| Discovery result | **non-clean: 8 findings so far — 3 P1, 5 P2; discovery continues through G–N** |
| Authoritative Audit-v6 findings | **8 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2** |
| Open Audit-v6 findings | **8** |
| Open P0/P1 | **3 — AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Phase | **Area G discovery in progress — initialization, burn-in, path dependence and continuation state** |
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

## Area G ownership / active discovery

Area G remains **in progress**. Fresh evidence has already demonstrated **AV6-008 / #721 P2**; this does not by itself complete the Area.

### Fresh pass 1 — unspecified founder genealogy × dependency-aware household lifecycle

Evidence-only PR: **#720**, closed unmerged.  
Final controlled head: `550a8edbce9f9b6a96e86a8f32a27bf0d46d528c`.  
Central CI: `34370338510`; `Quality and tests` job `102529686398`.

The controlled construction held one six-person founder household, zero mortality/fertility/resource pressure and disabled M4 fixed while varying only founder-genealogy completeness and whether `deterministic_dependency_fission_v2(3,18)` was active.

The negative control (`Unspecified`, lifecycle off) was admissible. The positive control (identical omitted links marked `CompleteLivingDirectParents`, lifecycle on) ran and produced two households. All **284 pre-existing core tests** passed. The fresh oracle then demonstrated that `Unspecified` genealogy with dependency-aware fission active is incorrectly accepted instead of failing closed.

This is **AV6-008 / #721 P2**: `Unspecified` explicitly says missing founder parent links cannot be interpreted as absence of living direct kin, yet the relationship-sensitive fission rule consumes those missing links as structural state. Ordinary and spatial construction currently gate genealogy completeness only for active M4 kin weighting, not for dependency-aware household lifecycle.

Mandatory duplicate review distinguished this from #324, AV3-004/#399, AV6-003/#699 and AV5-002/#617. No production repair is permitted during discovery.

### Remaining Area-G work

At least one further independent fresh attack is required before Area G can be considered complete. Good remaining targets include hidden/superseded future-defining state across checkpoint continuation and alternative-start/path-dependence behavior not already covered by historical v3/v4/v5 probes.

Historical v5 Area-G evidence remains control/duplicate-avoidance context only:

- #635 composed post-fission topology with three simultaneously active M9 journeys across checkpoint/resume and matched uninterrupted execution exactly;
- #636 showed declared founders remain invariant to dormant synthetic-only initialization knobs through active downstream fertility;
- #637 showed a future declared stop horizon does not alter the shared pre-horizon causal trajectory;
- AV5-003/#627 demonstrated spatial M9 history replay using the process seed instead of the population-realization seed, but that defect was repaired by PR #671 before v0.3.6 and has permanent regression coverage.

Do not repeat those as fresh v6 credit.

Known cross-cutting v6 context not to double-count:

- AV6-002/#694 is cross-cutting to Area G because founder/dynamic chronology semantics can affect initialization interpretation;
- AV6-008/#721 is now the primary Area-G finding and cross-cuts C/M/N;
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
| G | Initialization, burn-in, path dependence, continuation state | **in progress — AV6-008 P2** | #720/#721 P2; further independent fresh continuation/path-dependence evidence required. |
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
| `AV6-008` | **P2** | G primary; C/M/N | #720 head `550a8ed...`; CI `34370338510` job `102529686398`; dependency-aware fission accepts `Unspecified` founder genealogy as if missing parent links were structural absence | **#721** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:
1. finish fresh discovery through Areas G–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–F are complete**. Area G is **in progress**. Eight findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2 and AV6-008/#721 P2.

Next action: continue **Area G — initialization, burn-in, path dependence and continuation state** against immutable v0.3.6/v35 with another independent fresh adversary. Do not repair AV6-008 during discovery; use historical v3/v4/v5 Area-G experiments and permanent continuation regressions only for duplicate avoidance, controls and attack design.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and update this ledger before handoff.