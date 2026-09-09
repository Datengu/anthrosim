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
| Discovery coverage | **8/14 Areas A–N complete** |
| Discovery result | **non-clean: 10 findings so far — 5 P1, 5 P2; discovery continues through I–N** |
| Authoritative Audit-v6 findings | **10 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1; AV6-010/#729 P1** |
| Open Audit-v6 findings | **10** |
| Open P0/P1 | **5 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729** |
| Phase | **Area I discovery active — sensitivity, uncertainty, convergence and robustness** |
| Active ownership | **I — sensitivity, uncertainty, convergence and robustness** |
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
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1** | #725/#726 sequential Wilson stopped-procedure undercoverage; #728/#729 confirmatory sample values not semantically bound to authoritative study outputs. Report: `area-h-2026-09-09.md`. |

## Area H completion

Area H is complete after two causally distinct fresh attacks against immutable v0.3.6/v35.

### AV6-009 / #726 — P1: sequential Wilson precision stopping

Evidence-only PR #725 demonstrated exact stopped-procedure coverage `0.918976766485` for a declared 95% sequential Bernoulli precision procedure even though the fixed n=30, n=100 and n=300 Wilson intervals each independently met nominal 95% coverage. The width-based repeated stopping rule therefore under-covers by about 3.10 percentage points. PR #725 was closed unmerged and remediation remains deferred.

### AV6-010 / #729 — P1: confirmatory sample-value semantic binding

Evidence-only PR #728 used a fixed 30-seed finalized study and the full official study/research/finalize/confirmatory/provenance path. Canonical `research/analysis/runs.json` contained 30/30 completed runs. The positive sample produced estimate `1.0` / `sufficient_stop`; a contradictory sample using the same exact seeds but values `0` for all 30 runs produced estimate `0.0` / `sufficient_stop`, while analysis-provenance verification and isolated replay both passed.

Exact evidence:

- PR #728 head: `44101264bcbfeb114099addfb8ba23914270c174`;
- dedicated workflow: `34379137518`;
- dedicated job: `102559348237`;
- central CI: `34379137573`;
- `Quality and tests`: `102559410153`;
- analysis provenance identity: `analysis-provenance-v2-sha256-529bc993a39c736e4e9d04b45f02e4cea8936d49bab2820322e0ab85c5765f69`.

The dedicated scientific oracle alone was intentionally red; protected exact-head CI and the other scientific/security/determinism/provenance workflows passed. The defect is distinct from AV6-009: byte provenance and seed identity do not prove that submitted per-seed estimand values were derived from authoritative frozen study outputs. PR #728 was closed unmerged after AV6-010/#729 was preserved; remediation remains deferred.

Detailed Area-H completion evidence: `docs/research/audit-v6/area-h-2026-09-09.md`.

## Area I — active discovery

Primary scope: **sensitivity, uncertainty, convergence and robustness**.

Start Area I at zero fresh coverage. High-value falsification directions include:

- parameter-range and parameter-interaction sensitivity rather than one-factor examples;
- structural alternatives and hidden fixed mechanism choices;
- horizon and analysis-window sensitivity;
- spatial/temporal resolution sensitivity where outputs or claims imply comparability;
- initialization sensitivity and persistent alternative regimes;
- ensemble-size/replicate robustness without merely duplicating AV6-009's optional-stopping defect;
- threshold discontinuities, non-monotonic responses, multimodality and regime switching;
- whether robustness summaries or gates silently collapse scientifically distinct plausible settings;
- whether stated robustness survives plausible nuisance-parameter variation.

Existing findings must not be double-counted. In particular, AV6-009 owns the repeated-look confidence defect and AV6-010 owns missing semantic binding of confirmatory sample values.

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
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — AV6-009 P1 + AV6-010 P1** | #725/#726 stopped-procedure undercoverage; #728/#729 sample-value semantic-binding failure. |
| I | Sensitivity, uncertainty, convergence, robustness | **active** | Fresh parameter/structure/horizon/resolution/initialization/replicate robustness attacks required. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination, tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, identities, artifact integrity, replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality, incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases, benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | State |
|---|---:|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F/D/N | #686 head `bf94ed0...`; M4 seq1 then M9 seq2 day91 | **#687** | open; remediation deferred |
| `AV6-002` | **P2** | B primary; C/G/M/N | #693 head `579cc00...`; dynamic/founder male-age reference inversion | **#694** | open; remediation deferred |
| `AV6-003` | **P2** | C primary; E/N | #698 head `3733700...`; external-kin orientation flips under PersonId relabelling | **#699** | open; remediation deferred |
| `AV6-004` | **P1** | D primary; E/H/N | #705 head `f8ee562...`; condition flips under cell reflection | **#707** | open; remediation deferred |
| `AV6-005` | **P2** | D primary; C/E/F/N | #706 head `ae667fe...`; duration split flips under household-index relabelling | **#708** | open; remediation deferred |
| `AV6-006` | **P1** | E primary; F/H/I/N | #710 head `cd1dd83...`; unreachable padding changes M9 tied destination | **#711** | open; remediation deferred |
| `AV6-007` | **P2** | F primary; E/L/N | #717 head `416da690...`; touching visits report peak 2 despite no positive-duration overlap | **#718** | open; remediation deferred |
| `AV6-008` | **P2** | G primary; C/M/N | #720 head `550a8ed...`; unspecified founder genealogy treated as structural absence by fission | **#721** | open; remediation deferred |
| `AV6-009` | **P1** | H primary; K/L/M/N | #725 head `0104b5d...`; exact stopped coverage `0.918976766485` vs declared `0.95` | **#726** | open; remediation deferred |
| `AV6-010` | **P1** | H primary; K/L/M/N | #728 head `4410126...`; 30/30 authoritative completions accepted as contradictory 0/30 sample with provenance verify/replay pass | **#729** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas I–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–H are complete**. Area I is **active**. Ten findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2, AV6-009/#726 P1 and AV6-010/#729 P1.

Next action: continue **Area I — sensitivity, uncertainty, convergence and robustness** with a genuinely fresh adversarial attack against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.