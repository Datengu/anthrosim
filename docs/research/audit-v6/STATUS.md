# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed completed-area reports and executable evidence are preserved separately under `docs/research/audit-v6/` and on identified audit evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target software version | `0.3.6` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **4/14 Areas A–N complete** |
| Discovery result | **non-clean: 5 findings so far — 2 P1, 3 P2; discovery continues through E–N** |
| Authoritative Audit-v6 findings | **5 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2** |
| Open Audit-v6 findings | **5** |
| Open P0/P1 | **2 — AV6-001/#687 P1; AV6-004/#707 P1** |
| Phase | **Area E discovery next — spatial landscape, movement, migration, temporary mobility, boundaries** |
| Active ownership | **E — spatial landscape, movement, migration, temporary mobility, boundaries** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings have been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

### Area A — authoritative semantics and scheduler behaviour

Status: **complete — AV6-001 / #687 P1 open**  
Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Fresh v6 evidence:

- #684: 20 exact core-vs-identity-spatial checkpoint comparisons across cadence pairs, all equal; no finding;
- #686/#687: same-day target-arrival/M4 adversary demonstrated M4 before newly due M9 on day 91, opposite the declared fixed-boundary order; **AV6-001 P1**.

### Area B — demography, fertility, mortality, ageing and population structure

Status: **complete — AV6-002 / #694 P2 open**  
Completion report: `docs/research/audit-v6/area-b-2026-09-09.md`

Fresh v6 evidence:

- #690: remote deterministic birth shifted paired focal paternity in 500/1024 seeds; dispositioned as known sequential-RNG/common-random-number scope under #214, no new finding;
- #692: birthday-crossing × M3 mortality-cadence × fertility challenge passed quantitative exact-law tolerances across 4096 seeds/cadence; no finding;
- #693/#694: dynamic M2 and declared-founder genealogy apply one male-parent age window at incompatible time references; **AV6-002 P2**.

### Area C — households, kinship, social links and lifecycle structure

Status: **complete — AV6-003 / #699 P2 open**  
Completion report: `docs/research/audit-v6/area-c-2026-09-09.md`

Fresh v6 evidence:

- #698/#699: pure linked-adult PersonId relabelling flips which external-kin orientation enters a daughter household because source-local fission relationship refinement collapses distinct living external-parent contexts; **AV6-003 P2**;
- #701 corrected head: heterogeneous simultaneous source-HouseholdId relabelling preserved physical/member daughter partitions after bookkeeping IDs were erased; no finding.

### Area D — resources, condition, subsistence, depletion/recovery

Status: **complete — AV6-004 / #707 P1 and AV6-005 / #708 P2 open**  
Completion report: `docs/research/audit-v6/area-d-2026-09-09.md`

Fresh v6 evidence:

- #705/#707: exact 2×1 horizontal reflection of a one-unit scarce-resource tie flips abstract household condition from `(1000,0)` to `(0,1000)` because the equal-remainder award phase includes canonical `cell_index`. Final head `f8ee562027ae162afb9ce7a5f00a6ca3382b0f33`; CI `34356842006`, job `102483639065`; format/Clippy and all **284 pre-existing core tests** passed before the new oracle failed. **AV6-004 P1**;
- #706/#708: exact 50/50 one-unit M9 duration-weighted home/visitor tie changes from `(home=1,visitor=0)` to `(0,1)` when only canonical household index changes. Final head `ae667febd0930f43b50eae0af459c5fc8201f13f`; CI `34356930794`, job `102483966686`; format/Clippy and all **284 pre-existing core tests** passed before the new oracle failed. **AV6-005 P2**.

Area D introduced no production semantics changes. Both findings remain open and deferred behind the A–N discovery barrier.

## Area E ownership / next discovery session

Area E starts from zero after the Area-D disposition is merged. Before substantive evidence, reconstruct live `main`, open PRs/issues and overlapping spatial/movement audit work.

Fresh Area-E attacks should challenge at least some of:

- spatial reflection/rotation/isomorphism beyond already-known historical M4/M9 tie defects;
- boundary and unreachable-state semantics;
- M4 permanent migration candidate/utility/uncertainty behaviour;
- M9 travel reachability, route/duration and temporary-state transitions;
- transformed landscape equivalence and local/nonlocal coupling;
- persistent residence versus temporary physical presence;
- same-day or neighbouring mechanism interactions not already counted under AV6-001;
- spatial keys/ordering that can survive prior label/coupling repairs.

Known cross-cutting context to preserve without double-counting:

- AV6-001/#687 is a same-day M9/M4 scheduler-order P1 and must not be rediscovered as Area-E credit;
- AV6-003/#699 affects external kin context during fission and may propagate into M4;
- AV6-004/#707 is a resource-cell reflection P1 and is cross-cutting to E, but its primary defect is M3 allocation;
- AV6-005/#708 affects physical home/visitor resource attribution but is primary Area D;
- historical M4/M9 reflection, HouseholdId and coupling-key findings are controls/hypothesis sources only.

## Discovery phase rules

- Start every Area at zero coverage.
- Attribute discovery to immutable `v0.3.6` / v35.
- Prior audits and regressions are historical controls/hypothesis sources, not v6 completion evidence.
- Each Area requires genuinely fresh falsification-oriented evidence.
- Preserve demonstrated findings before production repair and search open/closed issues/PRs before creating one.
- Assign sequential identifiers `AV6-001`, `AV6-002`, ... only after distinct scientific defects are demonstrated.
- Continue A–N discovery after findings are recorded; ordinary production remediation begins only after full discovery completes.
- Keep evidence from semantically different heads explicitly separated.
- Do not infer empirical/archaeological validity from framework audit results.

## Discovery coverage matrix

| ID | Area | Status | Fresh v6 direction / evidence |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1 open** | #684 no finding; #686/#687 demonstrated same-day M9/M4 inversion. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV6-002 P2 open** | #690 known coupling scope; #692 quantitative no finding; #693/#694 male-parent age time-reference ambiguity. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV6-003 P2 open** | #698/#699 external-kin fission relabelling defect; #701 HouseholdId simultaneous-fission no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV6-004 P1 + AV6-005 P2 open** | `area-d-2026-09-09.md`; #705/#707 resource cell-reflection defect; #706/#708 duration-split household-identity defect. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **next/active** | Fresh spatial isomorphism, boundaries, unreachable/equal-cost choices, transformed-input and local-coupling attacks. |
| F | Aggregation and interaction mechanisms | **not started** | Trigger/timing, concentration vs relocation, interaction accounting, crowding/recovery and mechanism distinguishability. |
| G | Initialization, burn-in, path dependence, continuation state | **not started** | Alternative starts, transient/stationary interpretation, checkpoint continuation and path-dependence attacks. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **not started** | Seed/stream identity, draw ordering/coupling, rare events, stopping rules, replicate sufficiency, censoring and precision; revisit #690 as coupling-contract evidence. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Parameter/structure/horizon/resolution/initialization/replicate sensitivity and hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination and tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, partial handling, identities, artifact integrity and replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality and incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases and benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across all neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | Discovery state | Remediation / re-verification |
|---|---:|---|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F and D/N cross-cutting | #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; M4 seq 1 then M9 seq 2 on day 91 | **#687** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-002` | **P2** | B primary; C/G/M/N cross-cutting | #693 head `579cc00...`; CI `34306372696` job `102323804239`; dynamic/founder age-boundary inversion | **#694** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-003` | **P2** | C primary; E/N cross-cutting | #698 head `3733700...`; CI `34307746048` job `102327881921`; external-kin orientation flips under PersonId relabelling | **#699** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-004` | **P1** | D primary; E/H/N cross-cutting | #705 head `f8ee562...`; CI `34356842006` job `102483639065`; condition `(1000,0) -> (0,1000)` under cell reflection | **#707** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-005` | **P2** | D primary; C/E/F/N cross-cutting | #706 head `ae667fe...`; CI `34356930794` job `102483966686`; duration split `(1,0) -> (0,1)` under household-index relabelling | **#708** | **demonstrated; open** | **deferred until A–N discovery completes** |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas E–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–D are complete**. Area E is **next/active**. Open findings are AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1 and AV6-005/#708 P2; all remain unrepaired by design.

Next action: begin **Area E — spatial landscape, movement, migration, temporary mobility and boundaries** from zero against immutable v0.3.6/v35. Reconstruct live state and overlap before substantive evidence; use prior spatial findings only for duplicate avoidance and attack design.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active/next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.
