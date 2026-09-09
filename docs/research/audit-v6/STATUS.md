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
| Discovery coverage | **12/14 Areas A–N complete** |
| Discovery result | **non-clean: 14 findings so far — 7 P1, 7 P2; discovery continues through M–N** |
| Authoritative Audit-v6 findings | **14 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1; AV6-010/#729 P1; AV6-011/#733 P2; AV6-012/#737 P1; AV6-013/#741 P1; AV6-014/#745 P2** |
| Open Audit-v6 findings | **14** |
| Open P0/P1 | **7 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737; AV6-013/#741** |
| Phase | **Area M discovery active — documentation, TRACE/ODD/ODD+D and claim consistency** |
| Active ownership | **M — documentation, TRACE/ODD/ODD+D and claim consistency** |
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
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1** | #710/#711 unreachable impassable padding changes M9 tied destination; #713/#714 clean locality controls. Report: `area-e-2026-09-09.md`. |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | #716 clean lifecycle boundary control; #717/#718 touching half-open visits inflate `peakVisitors`. Report: `area-f-2026-09-09.md`. |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | #720/#721 unspecified founder genealogy consumed as structural absence in fission; #723 clean checkpoint/reproductive-history supersession control. Report: `area-g-2026-09-09.md`. |
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1** | #725/#726 sequential Wilson stopped-procedure undercoverage; #728/#729 confirmatory sample values not semantically bound to authoritative study outputs. Report: `area-h-2026-09-09.md`. |
| I — sensitivity, uncertainty, convergence and robustness | **complete — AV6-011/#733 P2** | #731/#733 vacuous no-op long-run sensitivity coverage; #732 clean late spatial-extent divergence invalidation. Report: `area-i-2026-09-09.md`. |
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1** | #735/#737 real-study deterministic output-binding failure; #736 clean narrow-claim/equifinality control. Report: `area-j-2026-09-09.md`. |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1** | #739/#741 finalized-study verifier semantic-parity failure; #740 clean cross-root binding-transplant control. Report: `area-k-2026-09-09.md`. |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | #743/#745 drifting initialization-dependence false negative; #744 clean cumulative-versus-realized-exposure ranking/extinction control. Report: `area-l-2026-09-09.md`. |

## Area L completion

Area L is complete after one demonstrated P2 defect, one causally independent clean denominator/censoring control and frozen-target review of the remaining analysis-summary surfaces.

### AV6-014 / #745 — P2: non-stable path-dependence summary collapse

Evidence-only PR #743 exercised the production long-run diagnostics under an explicitly transient protocol. A stable positive control correctly detected initialization dependence between population-100 and population-1000 regimes.

The falsification arm used two same-treatment drifting initialization families:

```text
A = 100,110,...,210
B = 1000,1100,...,2100
```

The two trajectories remain exactly 10× separated at every annual observation. All four runs correctly classify `drifting`, but the aggregate dependence layer reduces each run to `status:drifting` and reports:

```text
drifting_initialization_frequencies={'founder_state="A"': {'status:drifting': 2}, 'founder_state="B"': {'status:drifting': 2}}
drifting_initialization_dependence=false
terminal_population_ratio=10.0
```

Exact evidence:

- PR #743 head: `1441156b3c013aaf21d1c81bd8998b044f2e2038`;
- dedicated run: `34403272155`;
- dedicated job: `102640061320`;
- central CI: `34403272035` — success;
- `Quality and tests`: `102640137709` — success;
- all separate scientific/security/provenance/determinism/observability/bundle/resume workflows: success.

Only the deliberately red predeclared scientific oracle failed. Finding: **AV6-014/#745 P2 — long-run initialization-dependence summary collapses materially distinct drifting trajectories.** PR #743 was closed unmerged after preservation; remediation remains deferred.

The mechanism is the non-stable fallback in `run_outcome_label(...)`: stable/cyclic-stable runs receive metric-derived `regimeSignature` values, while any run with no signature is represented only as `status:<classification>`. Material trajectory information is therefore discarded before initialization/environment distribution comparison.

This also contradicts the normative `long-run-regime-diagnostics-v1.md` statement that initialization/environment dependence compares the **full normalized outcome distributions, not just which regime labels appear**. The contract explicitly supports `claimMode: explicitly_transient`, so this is a supported non-stationary analysis path rather than an irrelevant failed-equilibrium corner.

### Clean independent control — cumulative versus realized exposure

Evidence-only PR #744 tested a separate denominator/censoring surface.

Point A reached a full 10-year duration with cumulative unmet need 10,000. Point B became scientifically extinct after one year with cumulative unmet need 2,000. The raw cumulative ranking is therefore A > B, while realized-time intensity reverses the ranking to B > A.

Dedicated output:

```text
raw_cumulative_ranking=long-survivor(10000.0)>early-extinction(2000.0)
realized_time_rate_ranking=early-extinction(2000.0)>long-survivor(1000.0)
early_extinction_fraction=1.0
time_denominator=realizedSimulatedDays
exposure_ranking_control=pass
```

Exact head `0c52be7cf63657efdbbccba4e984157cdbf24dc3`; dedicated run `34403881292`; job `102642062415`; central CI `34403881201`. The dedicated scientific control and `Quality and tests` passed; the evidence PR is to close unmerged after its remaining downstream CI tail finishes green.

### Frozen-target analysis-summary review

The v35 analysis surface is explicit on several historically risky boundaries:

- undefined denominator statistics use `null`, not numeric-zero sentinels;
- scientifically meaningful extinction remains separate from operational person-record censoring;
- exposure-aware summaries preserve realized simulated time and terminal-outcome fractions;
- migration occurrence/frequency remains separate from move-conditioned quality means;
- pooled-per-move migration quantities are distinguished from run-weighted quantities where both are meaningful;
- cumulative since-start metrics are not documented as analysis-window totals without differencing/raw-event derivation;
- temporary mobility separates planned, realized and unrealized travel duration/cost/distance and includes reconciliation checks.

No additional independent Area-L defect was demonstrated on those surfaces.

Detailed Area-L completion evidence: `docs/research/audit-v6/area-l-2026-09-09.md`.

## Area M — active discovery

Primary scope: **documentation, TRACE/ODD/ODD+D and claim consistency**.

Start Area M at zero fresh coverage. High-value falsification directions include:

- executable v35 semantics versus normative scientific-model/ODD/ODD+D wording;
- the long-run contract's claim of full normalized outcome-distribution comparison after AV6-014;
- finalized-study/provenance documentation that claims semantically forged canonical artifacts cannot become authoritative after recomputing a fresh identity, after AV6-013 demonstrated otherwise;
- whether current TRACE statements accurately represent the active non-clean v6 convergence state;
- benchmark/release narratives that could imply empirical validity or stronger verification than actually established;
- null-model limitations and supported-use boundaries exposed by v6 Areas A–L;
- stale or over-broad claims introduced by historical repairs/version drift.

Area M must distinguish a documentation manifestation of an already-preserved v6 finding from a genuinely new causal/scientific defect. Do not create duplicate findings merely because documentation needs synchronization after discovery.

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
| I | Sensitivity, uncertainty, convergence, robustness | **complete — AV6-011 P2** | #731/#733 no-op sensitivity-coverage failure; #732 clean late domain-divergence control. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — AV6-012 P1** | #735/#737 deterministic output-binding failure; #736 clean equifinality control. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete — AV6-013 P1** | #739/#741 verifier semantic-parity failure; #740 clean cross-root control. |
| L | Observability, analysis outputs, statistical summaries | **complete — AV6-014 P2** | #743/#745 drifting path-dependence summary failure; #744 clean exposure/extinction control. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **active** | Fresh executable-vs-claim consistency attacks required. |
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
| `AV6-011` | **P2** | I primary; K/L/M/N | #731 head `51b13e0...`; no-op horizon/window arrays satisfy complete sensitivity coverage | **#733** | open; remediation deferred |
| `AV6-012` | **P1** | J primary; K/L/M/N | #735 head `fa46d25...`; same bound executions become falsely identifying after one contradictory deterministic output | **#737** | open; remediation deferred |
| `AV6-013` | **P1** | K primary; J/L/M/N | #739 head `86139b7...`; producer rejects forged `[999,12]` canonical rows while root-aware verifier accepts fresh self-consistent binding | **#741** | open; remediation deferred |
| `AV6-014` | **P2** | L primary; G/I/M/N | #743 head `1441156...`; two same-treatment drifting initialization families remain 10× apart but aggregate dependence is false | **#745** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas M–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–L are complete** once the Area-L completion documentation reaches protected `main`. Fourteen findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2, AV6-009/#726 P1, AV6-010/#729 P1, AV6-011/#733 P2, AV6-012/#737 P1, AV6-013/#741 P1 and AV6-014/#745 P2.

Next action: continue **Area M — documentation, TRACE/ODD/ODD+D and claim consistency** with a genuinely fresh adversarial comparison against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.
