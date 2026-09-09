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
| Discovery coverage | **1/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass already established by AV6-001 P1; discovery continues through B–N** |
| Authoritative Audit-v6 findings | **1** |
| Open Audit-v6 findings | **1** |
| Open P0/P1 | **1** |
| Phase | **Area B discovery in progress** |
| Active ownership | **B — demography, fertility, mortality, ageing and population structure** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because a new P1 has been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

### Area A — authoritative semantics and scheduler behaviour

Status: **complete — AV6-001 / #687 P1 open**  
Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Fresh v6 evidence included:

- evidence-only PR #684: exact core-vs-identity-spatial scheduler equivalence across four seeds × five M3/M4 cadence pairs = **20 complete checkpoint comparisons**, all equal; closed unmerged as no-finding evidence;
- evidence-only PR #686: controlled target-arrival/M4 same-day adversary demonstrated **M4 migration day 91 sequence 1 followed by newly due M9 departure day 91 sequence 2**, opposite to the declared M9-before-M4 fixed-boundary order; preserved as AV6-001 / #687 P1 and closed unmerged after ledger disposition.

Area A documentation disposition merged via PR #688 at `94b8ecb5d9b262a96792c4bbb243ae280c0a248e`.

## Active discovery session — 2026-09-09 / Area B

Area B starts from zero after reconstructing live state again:

- protected `main`: `94b8ecb5d9b262a96792c4bbb243ae280c0a248e`;
- immutable target: `v0.3.6` / `7d5e47309556e458477cd7283230871363b2c89a` / `anthrosim-model-semantics-v35`;
- open issues: **1**, AV6-001 / #687 P1 from Area A;
- open PRs: **0**;
- Audit-v6 branches present are Area-A/groundwork evidence or documentation branches; none claims substantive Area B;
- active ownership: **Area B — demography, fertility, mortality, ageing and population structure**;
- #687 is cross-cutting to later movement/resource integration but does not overlap the independent Area-B demographic audit surface.

Area B must generate fresh v6 evidence beyond prior demographic regressions. Primary attack directions are limiting cases and structural controls for fertility/mortality competition, age-boundary timing, mate limitation, extinction/censoring, newborn state, finite-population behaviour and demographic dependence on scientifically irrelevant representation/order.

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
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1 open** | `area-a-2026-09-09.md`; #684 no-finding scheduler-equivalence evidence; #686/#687 demonstrated P1 same-day M9/M4 inversion. |
| B | Demography, fertility, mortality, ageing, population structure | **in progress** | Fresh limiting cases/structural controls for fertility-mortality competition, mate limitation, age/lifecycle timing, extinction/censoring, newborn state and finite-population effects. |
| C | Households, kinship, social links, lifecycle structure | **not started** | Formation/fission/parentage lifecycle invariants, relationship-order dependence and downstream demographic/mobility coupling. |
| D | Resources, condition, subsistence, depletion/recovery | **not started** | Depletion/replenishment cadence, allocation order/ties, realized-vs-nominal effects and initialization dependence. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **not started** | Symmetry/isomorphism, boundaries, unreachable/equal-cost choices, transformed-input and local-coupling attacks. |
| F | Aggregation and interaction mechanisms | **not started** | Trigger/timing, concentration vs relocation, interaction accounting, crowding/recovery and mechanism distinguishability. |
| G | Initialization, burn-in, path dependence, continuation state | **not started** | Alternative starts, transient/stationary interpretation, checkpoint continuation and path-dependence attacks. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **not started** | Seed/stream identity, draw ordering/coupling, rare events, stopping rules, replicate sufficiency, censoring and precision. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Parameter/structure/horizon/resolution/initialization/replicate sensitivity and hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination and tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, partial handling, identities, artifact integrity and replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality and incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases and benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across all neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | Discovery state | Remediation / re-verification |
|---|---:|---|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F and D/N cross-cutting | `v0.3.6` / `7d5e473...`; #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; M4 seq 1 then M9 seq 2 on day 91 | **#687** | **demonstrated; open** | **deferred until A–N discovery completes** |

## Discovery/remediation barrier and convergence

Because AV6-001 is P1, Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas B–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Area A is complete; **Area B is actively owned and in discovery**. AV6-001/#687 remains open and unrepaired by design. No open PR or other branch owns substantive Area B at session start.

Next action: inspect immutable v0.3.6 demographic implementation/documentation and prior finding history only to avoid duplicates, then execute genuinely fresh Area-B adversarial evidence before any Area-B disposition.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active/next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.