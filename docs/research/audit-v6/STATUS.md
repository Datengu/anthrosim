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
| Discovery coverage | **3/14 Areas A–N complete** |
| Discovery result | **non-clean: 3 findings so far — 1 P1, 2 P2; discovery continues through D–N** |
| Authoritative Audit-v6 findings | **3 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2** |
| Open Audit-v6 findings | **3** |
| Open P0/P1 | **1 — AV6-001/#687 P1** |
| Phase | **Area D discovery next — resources, condition, subsistence, depletion/recovery** |
| Active ownership | **D — resources, condition, subsistence, depletion/recovery** |
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

### Area B — demography, fertility, mortality, ageing and population structure

Status: **complete — AV6-002 / #694 P2 open**  
Completion report: `docs/research/audit-v6/area-b-2026-09-09.md`

Fresh v6 evidence included:

- evidence-only PR #690: adding a separate-cell deterministic one-male birth shifted the unchanged focal two-male paternity result in **500/1024** paired seeds. Final disposition: **no new finding / known stochastic-coupling scope** under #214 because the arms contain a different causal event sequence and v35 does not promise agent/event-level common random numbers after such divergence; closed unmerged;
- evidence-only PR #692: integrated birthday-crossing × M3 background-mortality cadence × survival-conditioned fertility challenge. Across **4096 seeds per cadence**, focal birth counts were `2041`, `2091`, `2073`, `2025` for `1`, `4`, `12`, `365` M3 periods/year, respectively; max deviation from the exact 0.5 expectation was 43 and max pairwise difference 66, well inside predeclared 160/220 tolerances. Exact zero-risk boundary controls and all exact-head workflows passed. Final disposition: **clean no finding**; closed unmerged;
- evidence-only PR #693: demonstrated that the same configured male-parent age window is evaluated at annual interval start by dynamic M2 but at child birth by declared-founder genealogy. Final controlled head `579cc00...`; dedicated run `34306372696` job `102323804239`; upper/lower controls were `dynamic Some(MALE) / founder rejected` and `dynamic None / founder accepted`. Central CI run `34306372699`, job `102323830454`, passed format, Clippy and all **284 pre-existing core tests** before the dedicated adversary failed. Preserved as **AV6-002/#694 P2** and closed unmerged.

Area B introduced no production semantics changes. AV6-002 remains open and deferred behind the A–N discovery barrier.

### Area C — households, kinship, social links and lifecycle structure

Status: **complete — AV6-003 / #699 P2 open**  
Completion report: `docs/research/audit-v6/area-c-2026-09-09.md`

Fresh v6 evidence included:

- evidence-only PR #698: cross-household external-kin relabelling adversary against `deterministic_dependency_fission_v2`. Final controlled head `3733700bc1353cce768f41e9f4932f96766fd45f`; central CI run `34307746048`, `Quality and tests` job `102327881921`. Formatting and Clippy passed, all **284 pre-existing core tests** passed, then the new oracle failed. The same canonical moved-ID vector `[PersonId(2), PersonId(4)]` was produced in both arms, but pure canonical relabelling flipped the abstract externally linked adult entering the daughter household from east to west: `(west_moved=false,east_moved=true) -> (true,false)`. Full duplicate search found no existing issue for this residual cross-household-context failure. Preserved as **AV6-003/#699 P2** and PR #698 closed unmerged;
- evidence-only PR #701: heterogeneous simultaneous-fission source-HouseholdId relabelling challenge. The initial red head was rejected as a harness error because founder households were serialized non-canonically. Corrected head `40f92be8b591e0dc1bc046884962c3a89db6d70f`; central CI `34309228460`, `Quality and tests` job `102332243271`. Formatting, Clippy, all **284 pre-existing core tests**, the fresh relabelling oracle and applicable scientific/security gate passed. After erasing bookkeeping household IDs, one west daughter plus two east daughters had identical physical/member partitions in both arms. Final disposition: **clean no finding**; PR #701 closed unmerged.

AV6-003 is distinct from historical #324 and AV3-004/#399. #399's v25 repair derives relationship classes over the living **source-household** parent/child graph; v35 still collapses any living parent outside that source household to one generic external-parent state before the final `PersonId` tie-break. External direct-parent location is causal downstream to M4 kin utility, so the collapsed adults are not scientifically interchangeable.

Area C introduced no production semantics changes. AV6-003 remains open and deferred behind the A–N discovery barrier.

Known cross-cutting context to preserve without double-counting:

- AV6-002/#694 affects authoritative parentage/genealogy near male age thresholds and may propagate into later kin-mediated household or movement behaviour;
- AV6-001/#687 remains a scheduler/temporary-mobility P1 and must not be repaired during discovery;
- prior audits contain household/fission/kinship repairs and parentage-scope decisions, but they are historical controls only.

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
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV6-002 P2 open** | `area-b-2026-09-09.md`; #690 known coupling scope; #692 quantitative birthday/cadence no finding; #693/#694 demonstrated male-parent age time-reference ambiguity. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV6-003 P2 open** | `area-c-2026-09-09.md`; #698/#699 demonstrated cross-household external-kin context collapse before final PersonId fission tie-break; #701 source-HouseholdId/global-daughter-ID relabelling no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **not started — next** | Depletion/replenishment cadence, allocation order/ties, realized-vs-nominal effects and initialization dependence. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **not started** | Symmetry/isomorphism, boundaries, unreachable/equal-cost choices, transformed-input and local-coupling attacks. |
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
| `AV6-001` | **P1** | A primary; E/F and D/N cross-cutting | `v0.3.6` / `7d5e473...`; #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; M4 seq 1 then M9 seq 2 on day 91 | **#687** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-002` | **P2** | B primary; C/G/M/N cross-cutting | `v0.3.6` / `7d5e473...`; #693 head `579cc00...`; dedicated run `34306372696` job `102323804239`; upper dynamic accepted/founder rejected, lower dynamic excluded/founder accepted | **#694** | **demonstrated; open** | **deferred until A–N discovery completes** |
| `AV6-003` | **P2** | C primary; E/N cross-cutting | `v0.3.6` / `7d5e473...`; #698 head `3733700...`; CI `34307746048` job `102327881921`; external-kin orientation `(false,true) -> (true,false)` under pure linked-adult PersonId relabelling | **#699** | **demonstrated; open** | **deferred until A–N discovery completes** |

## Discovery/remediation barrier and convergence

Because AV6-001 is P1, Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas D–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A, B and C are complete**. Area D is **next**. Open findings are AV6-001/#687 P1, AV6-002/#694 P2 and AV6-003/#699 P2; all remain unrepaired by design.

Next action: begin **Area D — resources, condition, subsistence, depletion/recovery** from zero with genuinely fresh evidence against immutable v0.3.6/v35. Reconstruct live state and overlapping work first, use historical resource findings only as hypothesis/duplicate-avoidance controls, and do not repair existing v6 findings during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active/next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.