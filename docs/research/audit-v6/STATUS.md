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
| Discovery coverage | **9/14 Areas A–N complete** |
| Discovery result | **non-clean: 11 findings so far — 5 P1, 6 P2; discovery continues through J–N** |
| Authoritative Audit-v6 findings | **11 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1; AV6-010/#729 P1; AV6-011/#733 P2** |
| Open Audit-v6 findings | **11** |
| Open P0/P1 | **5 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729** |
| Phase | **Area J discovery active — identifiability, equifinality, calibration and discrimination** |
| Active ownership | **J — identifiability, equifinality, calibration and discrimination** |
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
| E — spatial landscape, movement, migration, temporary mobility, boundaries | **complete — AV6-006/#711 P1** | #710/#711 unreachable impassable padding changes M9 tied destination; #713/#714 clean locality controls. Report: `area-e-2026-09-09.md`. |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | #716 clean lifecycle boundary control; #717/#718 touching half-open visits inflate `peakVisitors`. Report: `area-f-2026-09-09.md`. |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | #720/#721 unspecified founder genealogy consumed as structural absence in fission; #723 clean checkpoint/reproductive-history supersession control. Report: `area-g-2026-09-09.md`. |
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1** | #725/#726 sequential Wilson stopped-procedure undercoverage; #728/#729 confirmatory sample values not semantically bound to authoritative study outputs. Report: `area-h-2026-09-09.md`. |
| I — sensitivity, uncertainty, convergence and robustness | **complete — AV6-011/#733 P2** | #731/#733 vacuous no-op long-run sensitivity coverage; #732 clean late spatial-extent divergence invalidation. Report: `area-i-2026-09-09.md`. |

## Area I completion

Area I is complete after one demonstrated defect, one causally independent clean convergence attack and direct frozen-target review of current sensitivity/design contracts.

### AV6-011 / #733 — P2: vacuous long-run sensitivity coverage

Evidence-only PR #731 showed that the equilibrium-like long-run gate treats non-empty sensitivity arrays as mandatory coverage even when every entry exactly repeats the primary analysis. With primary start day 3285 and primary end day 8760, the no-op declarations:

```text
runLengthSensitivityEndDays = [8760]
analysisStartSensitivityDays = [3285]
analysisEndSensitivityDays = [8760]
```

produced:

```text
vacuous_coverage_complete=true
vacuous_gate_status=passed
vacuous_equilibrium_supported=true
```

On the exact same trajectories and primary analysis, genuine alternative run-length/start/end choices were detected and the gate failed. Exact evidence head `51b13e0b4239382fb4b71b600ce78666e3034b76`; dedicated run `34381701126`; job `102567877647`; central CI `34381701039`; `Quality and tests` `102567934940` passed. PR #731 was closed unmerged after AV6-011/#733 was preserved; remediation remains deferred.

### Clean independent control — late spatial-extent divergence

Evidence-only PR #732 tests a separate spatial/domain convergence mechanism. The first disposable run failed before test execution because the audit workflow pinned Rust 1.92.0 instead of the repository-required 1.97.1. The harness pin alone was corrected.

On corrected head `2e6b66c78716671fd947c95ba8d3c239c4d4b1cd`, dedicated run `34382359938` / job `102570080390` passed:

```text
prefix_adequate=true
prefix_trailing_stable=2
late_adequate=false
late_trailing_stable=0
late_absolute_difference=50
late_relative_difference_permille=334
material_boundary_dependence=true
```

Thus a later material domain divergence revokes an earlier adequate prefix rather than leaving a stale convergence classification. No new finding was demonstrated by this attack.

The completion-documentation PR must not merge until #732's ordinary exact-head checks have completed without an unrelated failure; once classified, #732 is evidence-only and must close unmerged.

### Frozen-target robustness/design inspection

The v35 research definition preserves a complete authoritative base configuration and supports deterministic Cartesian expansion across multiple generic dimensions. Current validation rejects duplicate/overlapping dimension paths and structural alternatives whose executable projections are not genuinely distinct, preserving repaired structural-sensitivity semantics.

The spatial-resolution contract is also explicit: `cell_space_resolution_dependent_v1` does not claim raster invariance, records `requiresResolutionSensitivity = true`, and requires resolution sensitivity/convergence or explicit conditional reporting before scale-independent physical interpretation.

AV6-006, AV6-008, AV6-009 and AV6-010 remain relevant cross-cutting constraints on robustness studies but are not double-counted as Area-I findings.

Detailed Area-I completion evidence: `docs/research/audit-v6/area-i-2026-09-09.md`.

## Area J — active discovery

Primary scope: **identifiability, equifinality, calibration and discrimination**.

Start Area J at zero fresh coverage. High-value falsification directions include:

- semantic binding between identifiability outputs and authoritative executed/finalized study results, not merely design-coordinate binding;
- multiple parameter/model combinations fitting the same calibration outputs;
- parameter compensation and nuisance dimensions;
- structural equifinality and compatible-region reporting;
- held-out discrimination and calibration/corroboration leakage;
- profile/conditional sensitivity and interaction surfaces;
- degenerate or vacuous claim declarations;
- exact numeric fidelity after v0.3.6's large-integer identifiability repair;
- whether upstream stochastic/provenance limitations are incorrectly treated as valid identifying evidence without duplicating AV6-009/AV6-010.

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
| J | Identifiability, equifinality, calibration, discrimination | **active** | Fresh compatible-region, compensation, structural-equifinality, held-out-discrimination and output-binding attacks required. |
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
| `AV6-011` | **P2** | I primary; K/L/M/N | #731 head `51b13e0...`; exact no-op horizon/window arrays satisfy complete sensitivity coverage and pass equilibrium gate | **#733** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas J–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–I are complete** once the Area-I completion documentation reaches protected `main`. Eleven findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2, AV6-009/#726 P1, AV6-010/#729 P1 and AV6-011/#733 P2.

Next action: continue **Area J — identifiability, equifinality, calibration and discrimination** with a genuinely fresh adversarial attack against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.