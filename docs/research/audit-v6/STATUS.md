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
| Discovery coverage | **7/14 Areas A–N complete** |
| Discovery result | **non-clean: 9 findings so far — 4 P1, 5 P2; discovery continues through H–N** |
| Authoritative Audit-v6 findings | **9 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1** |
| Open Audit-v6 findings | **9** |
| Open P0/P1 | **4 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726** |
| Phase | **Area H discovery active — stochasticity, RNG, ensembles and Monte Carlo inference** |
| Active ownership | **H — stochasticity, RNG, ensembles and Monte Carlo inference** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings have been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

| Area | Status | Fresh v6 evidence |
|---|---|---|
| A — authoritative semantics and scheduler behaviour | **complete — AV6-001/#687 P1** | #684 clean scheduler-equivalence controls; #686/#687 same-day M4-before-newly-due-M9 inversion. Report: `area-a-2026-09-09.md`. |
| B — demography, fertility, mortality, ageing and population structure | **complete — AV6-002/#694 P2** | #692 clean chronology/cadence control; #693/#694 inconsistent male-parent age-window time references. Report: `area-b-2026-09-09.md`. |
| C — households, kinship, social links and lifecycle structure | **complete — AV6-003/#699 P2** | #698/#699 external-kin context collapse before final PersonId fission tie-break; #701 clean relabelling control. Report: `area-c-2026-09-09.md`. |
| D — resources, condition, subsistence, depletion/recovery | **complete — AV6-004/#707 P1; AV6-005/#708 P2** | #705/#707 scarce-resource reflection failure; #706/#708 duration-split household-index dependence. Report: `area-d-2026-09-09.md`. |
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1** | #710/#711 unreachable impassable padding changes M9 equal-cost destination; #713/#714 clean locality controls. Report: `area-e-2026-09-09.md`. |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | #716 clean lifecycle boundary control; #717/#718 touching half-open visits inflate `peakVisitors`. Report: `area-f-2026-09-09.md`. |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | #720/#721 unspecified founder genealogy consumed as structural absence in fission; #723 clean checkpoint/reproductive-history supersession control. Report: `area-g-2026-09-09.md`. |

## Area H — active discovery

Primary scope: **stochasticity, RNG, ensembles and Monte Carlo inference**.

### Fresh v6 evidence recorded so far

#### AV6-009 / #726 — P1

Evidence-only PR #725 tested the supported sequential Bernoulli `probability` precision gate against immutable v0.3.6/v35.

Exact controlled head: `0104b5d8fd830d827cd291b8eb5d3c293b117cd9`  
Dedicated workflow: `34374791210`  
Central CI: `34374791165`, `Quality and tests` job `102544806010`.

Controlled design: true `p=0.305`, declared confidence `0.95`, `maxHalfWidth=0.1685`, predeclared cumulative boundaries `[30,100,300]`. Production Wilson interval/stopping logic was evaluated for every attainable success count and exact finite Bernoulli path probabilities were independently enumerated.

Fixed-boundary exact coverage controls all met nominal 95%:

- n=30: `0.953754214943`;
- n=100: `0.950153992240`;
- n=300: `0.955430640776`.

But the interval reported at the gate's own width-based stopping time covered only `0.918976766485`, a **3.1023 percentage-point shortfall** from the declared 95% confidence level. Early-stop probability was `0.038654964824` at n=30, with remaining paths stopping at n=100 under the chosen threshold.

All ordinary/protected exact-head workflows passed: format, Clippy, full workspace, all **284 pre-existing `anthrosim-core` tests**, release/benchmark/downstream CI, and scientific/security/determinism/provenance gates. Only the predeclared dedicated sequential-coverage oracle failed.

Finding: **AV6-009/#726 P1 — sequential Wilson precision stopping under-covers its declared confidence level.** The current procedure repeatedly reuses an ordinary fixed-sample Wilson interval at data-dependent stopping times without a confidence-sequence, alpha-spending or equivalent sequential-validity adjustment.

Mandatory duplicate search distinguished this from AV5-005/#640 (small-n zero-variance normal-CLT mean stopping), AV2-009/#334 (quantile coverage), AV3-006/#410 (paired/independent mean contrasts), AV4-010/#528 (large-integer statistical fidelity), and #214 (conditional draw/common-random-number alignment). #725 is closed unmerged; remediation is deferred.

### Area-H next work

Area H remains **incomplete**. Continue with at least one genuinely independent fresh stochastic attack before disposition. High-value directions include:

- stochastic identity/coupling for model-born individuals after founder-generation label-invariance repairs;
- remote or representation-only perturbations that should not reassign local stochastic outcomes;
- rare-event and low-probability endpoint behavior;
- sequential validity of the other supported estimator families, while avoiding merely duplicating AV6-009's underlying repeated-look defect;
- seed-role and ensemble-replicate semantics not already covered by AV5-003/#627 or #214;
- aggregation/precision behavior under missing, extinct or operationally censored replicates, only where current explicit censoring contracts do not already resolve the question.

Known cross-cutting v6 controls not to double-count:

- AV6-004/#707 is primarily an Area-D resource/fairness defect;
- AV6-006/#711 is primarily an Area-E spatial-locality defect;
- AV6-009/#726 now owns the repeated-look nominal-confidence failure; a new Area-H finding must demonstrate a distinct causal defect rather than another parameterization of the same optional-stopping problem.

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
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1** | #684 clean; #686/#687 P1. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV6-002 P2** | #692 clean; #693/#694 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV6-003 P2** | #698/#699 P2; #701 clean. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV6-004 P1 + AV6-005 P2** | #705/#707; #706/#708. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — AV6-006 P1** | #710/#711 P1; #713/#714 clean. |
| F | Aggregation and interaction mechanisms | **complete — AV6-007 P2** | #716 clean; #717/#718 P2. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — AV6-008 P2** | #720/#721 P2; #723 clean. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **active — AV6-009 P1 so far** | #725/#726 sequential Wilson stopped-procedure undercoverage; continue independent attack. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Parameter/structure/horizon/resolution/initialization/replicate sensitivity and hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination, tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, identities, artifact integrity, replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality, incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases, benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | State |
|---|---:|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F/D/N | #686 head `bf94ed0...`; CI `34301664858`; M4 seq1 then M9 seq2 day91 | **#687** | open; remediation deferred |
| `AV6-002` | **P2** | B primary; C/G/M/N | #693 head `579cc00...`; dynamic/founder male-age reference inversion | **#694** | open; remediation deferred |
| `AV6-003` | **P2** | C primary; E/N | #698 head `3733700...`; external-kin orientation flips under PersonId relabelling | **#699** | open; remediation deferred |
| `AV6-004` | **P1** | D primary; E/H/N | #705 head `f8ee562...`; CI `34356842006`; condition flips under cell reflection | **#707** | open; remediation deferred |
| `AV6-005` | **P2** | D primary; C/E/F/N | #706 head `ae667fe...`; duration split flips under household-index relabelling | **#708** | open; remediation deferred |
| `AV6-006` | **P1** | E primary; F/H/I/N | #710 head `cd1dd83...`; CI `34358664992`; unreachable padding changes M9 tied destination | **#711** | open; remediation deferred |
| `AV6-007` | **P2** | F primary; E/L/N | #717 head `416da690...`; touching visits report peak 2 despite no positive-duration overlap | **#718** | open; remediation deferred |
| `AV6-008` | **P2** | G primary; C/M/N | #720 head `550a8ed...`; unspecified founder genealogy treated as structural absence by fission | **#721** | open; remediation deferred |
| `AV6-009` | **P1** | H primary; K/L/M/N | #725 head `0104b5d...`; dedicated run `34374791210`; exact stopped coverage `0.918976766485` vs declared `0.95` while all fixed-boundary controls pass | **#726** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:
1. finish fresh discovery through Areas H–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–G are complete**. Area H is **active and incomplete**. Nine findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2 and AV6-009/#726 P1.

Next action: continue **Area H** with a genuinely independent stochastic attack against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.