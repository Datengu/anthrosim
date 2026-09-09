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

- protected `main` at this session's fresh reconstruction: `34959751515f128949c8c045e7fad14a28248895`;
- immutable target: `v0.3.6` / `7d5e47309556e458477cd7283230871363b2c89a` / `anthrosim-model-semantics-v35`;
- open Audit-v6 issue: AV6-001 / #687 P1 from Area A;
- active Area-B evidence PR #690 was inspected at exact head `e387192ce7d2d5193e101d514e8c64d2806dce1c`, dispositioned, and closed unmerged;
- active ownership remains **Area B — demography, fertility, mortality, ageing and population structure**;
- #687 is cross-cutting to later movement/resource integration but does not overlap the independent Area-B demographic audit surface.

### Fresh Area-B evidence disposition: parentage-stream insertion locality

Evidence-only PR #690 constructed a deliberately isolated parentage-stream locality attack:

- the focal Cell-2 household was unchanged between arms and contained one certain-birth female plus two scientifically distinct eligible males;
- mortality was zero, fertility certain, spacing zero, resource need neutralized and migration disabled;
- the augmented arm added a separate Cell-1 household whose older female was processed first and whose only eligible male made that remote paternity outcome deterministic;
- the oracle required the unchanged focal two-male parentage choice to remain unchanged across identical seeds `0..1023` despite the separate-cell deterministic birth.

At exact evidence head `e387192ce7d2d5193e101d514e8c64d2806dce1c`, central CI run `34303025218` reached the controlled red scientific assertion in `Quality and tests` job `102313886432`; format, Clippy and the applicable scientific/security gates were green. Historical/current duplicate search showed that this is **not a distinct AV6 finding**: it is fresh v6 evidence for the already-declared sequential-stream/common-random-number limitation in closed #214, now demonstrated specifically in `demography/parentage`. The v6 evidence was recorded on #214 and PR #690 was closed unmerged.

This disposition is distinct from AV4-005/#495, which covered arbitrary male-label/order assignment within one local choice set, and AV5-002/#617, which covered close-kin eligibility scope. No production repair is authorized during v6 discovery.

Area B remains incomplete. The next fresh attack must be scientifically distinct from the accepted #214 cross-arm draw-consumption limitation and should preferentially target a different required Area-B surface such as age-boundary timing, mortality/fertility limiting cases, extinction/censoring, newborn initialization, or finite-population replacement behaviour.

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
| B | Demography, fertility, mortality, ageing, population structure | **in progress** | #690 fresh deterministic-remote parentage insertion-locality attack reproduced known #214 sequential-stream coupling and was closed unmerged with no new AV6 finding; continue with a scientifically distinct age/mortality/extinction/newborn/finite-population attack. |
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

Audit-v6 Area A is complete; **Area B remains actively owned and in discovery**. AV6-001/#687 remains open and unrepaired by design. Fresh Area-B evidence PR #690 has been dispositioned as additional evidence for known #214 sequential-stream coupling and closed unmerged; it does not create AV6-002.

Next action: execute a genuinely fresh Area-B adversary that is not merely another manifestation of cross-arm sequential RNG draw consumption. Prefer age-boundary timing, mortality/fertility limiting cases, extinction/censoring, newborn initialization, or finite-population replacement behaviour; search historical/current issues before any finding disposition.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active/next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.