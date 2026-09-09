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
| Discovery coverage | **10/14 Areas A–N complete** |
| Discovery result | **non-clean: 12 findings so far — 6 P1, 6 P2; discovery continues through K–N** |
| Authoritative Audit-v6 findings | **12 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1; AV6-010/#729 P1; AV6-011/#733 P2; AV6-012/#737 P1** |
| Open Audit-v6 findings | **12** |
| Open P0/P1 | **6 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737** |
| Phase | **Area K discovery active — experiment orchestration, configuration, provenance and reproducibility** |
| Active ownership | **K — experiment orchestration, configuration, provenance and reproducibility** |
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
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1** | #735/#737 real-study deterministic output-binding failure; #736 clean narrow-claim/equifinality control. Report: `area-j-2026-09-09.md`. |

## Area J completion

Area J is complete after one demonstrated P1 defect, one causally independent clean equifinality control and direct frozen-target review of current identifiability contracts.

### AV6-012 / #737 — P1: deterministic outputs are not bound to authoritative executions

Evidence-only PR #735 used a real two-point `anthrosim-research` design with `duration_years = [1,2]` and two seeds per point. The production binder correctly reconstructed the immutable research identity, point coordinates, canonical structures and all four execution IDs.

The truthful deterministic completion outputs `[1.0,1.0]` left both points compatible and the claimed duration parameter unidentifiable:

```text
truthful_outputs=1.0,1.0 gate=false compatible=2 identified=false
```

Changing only the downstream deterministic output for the two-year point to `0.0`, while preserving the same real research root and exact executed-design binding, produced:

```text
contradictory_outputs=1.0,0.0 executed_design_bound=true gate=true compatible=1 identified=true
```

Exact evidence head `fa46d254f5e5c53dfce9fc5741f3a3047dba1c4b`; dedicated run `34383829078`; job `102574961144`; central CI `34383829226`; `Quality and tests` `102575010365` passed formatting, Clippy and the complete workspace suite. The separate scientific/security/provenance/determinism workflows were green. PR #735 was closed unmerged after AV6-012/#737 was preserved.

The mechanism is downstream of the repaired AV4-011 coordinate binding: point/parameter/structure/execution identities are proven, but `outputEvidence={"kind":"deterministic"}` does not prove that the supplied claim-driving value was derived from those executions. Remediation remains deferred.

### Clean independent control — narrow claim preserves broader equifinality

Evidence-only PR #736 uses a full factorial over claimed `theta`, nuisance parameter and two structures. Calibration legitimately identifies `theta=0` while leaving four compatible scientific states.

Dedicated run `34384083382` / job `102575855189` passed:

```text
gate=true compatible=4 theta_identified=true nuisance_identified=false
equifinality_present=true parameter_combination_equifinality=true structural_equifinality=true nuisance_compensation=['nuisance']
```

Thus a successful narrow parameter claim does not erase nuisance compensation, parameter-combination equifinality or structural equifinality. No new finding was demonstrated. The completion-documentation PR must not merge until #736's ordinary exact-head suite is green and the evidence PR is closed unmerged.

### Frozen-target identifiability inspection

The v0.3.6 front end retains strong manifest-derived parameter/structure/execution binding, exact JSON-integer parameter arithmetic after AV5-007, non-negative held-out discrimination tolerance after AV5-006, disjoint calibration/held-out evidence roles and conservative compatible-region/equifinality reporting.

AV6-010/#729 remains a separate stochastic value-binding defect and is not double-counted as Area-J evidence.

Detailed Area-J completion evidence: `docs/research/audit-v6/area-j-2026-09-09.md`.

## Area K — active discovery

Primary scope: **experiment orchestration, configuration, provenance and reproducibility**.

Start Area K at zero fresh coverage. Do not replay v5's successful relocation + missing `research-state.json` retry control. High-value fresh directions include:

- consistency among immutable root plan/manifest, mutable orchestration state, validated child bundles and canonical derived indexes under partial recovery;
- retry/reconciliation when individually valid artifacts are stale or cross-bound to another execution generation;
- transactional publication/recovery when interrupted states leave scientifically ambiguous but structurally valid candidates;
- source/revision and model-semantics identity across research, study finalization and downstream provenance;
- canonical analysis/provenance replay when inputs are each valid but mutually inconsistent;
- artifact-set completeness and provenance identity under stale, duplicate or substituted optional scientific artifacts;
- whether AV6-010/#729 or AV6-012/#737 can be propagated as apparently authoritative downstream results without counting those same defects twice.

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
| K | Experiment orchestration, configuration, provenance, reproducibility | **active** | Fresh orchestration/provenance/recovery attack required. |
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
| `AV6-011` | **P2** | I primary; K/L/M/N | #731 head `51b13e0...`; no-op horizon/window arrays satisfy complete sensitivity coverage | **#733** | open; remediation deferred |
| `AV6-012` | **P1** | J primary; K/L/M/N | #735 head `fa46d25...`; same four bound executions flip from non-identifying to identified after one contradictory deterministic output | **#737** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas K–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–J are complete** once the Area-J completion documentation reaches protected `main`. Twelve findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2, AV6-009/#726 P1, AV6-010/#729 P1, AV6-011/#733 P2 and AV6-012/#737 P1.

Next action: continue **Area K — experiment orchestration, configuration, provenance and reproducibility** with a genuinely fresh adversarial attack against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.