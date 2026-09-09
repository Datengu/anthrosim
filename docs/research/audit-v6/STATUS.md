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
| Discovery coverage | **11/14 Areas A–N complete** |
| Discovery result | **non-clean: 13 findings so far — 7 P1, 6 P2; discovery continues through L–N** |
| Authoritative Audit-v6 findings | **13 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2; AV6-009/#726 P1; AV6-010/#729 P1; AV6-011/#733 P2; AV6-012/#737 P1; AV6-013/#741 P1** |
| Open Audit-v6 findings | **13** |
| Open P0/P1 | **7 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737; AV6-013/#741** |
| Phase | **Area L discovery active — observability, analysis outputs and statistical summaries** |
| Active ownership | **L — observability, analysis outputs and statistical summaries** |
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
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1** | #739/#741 finalized-study verifier semantic-parity failure; #740 clean cross-root binding-transplant control. Report: `area-k-2026-09-09.md`. |

## Area K completion

Area K is complete after one demonstrated P1 defect, one causally independent clean root-binding control and direct confirmation that the authoritative producer-side finalization repair remains fail-closed.

### AV6-013 / #741 — P1: finalized-study verifier semantic parity

Evidence-only PR #739 used a real exploratory study and real two-point research design with `resources.periodsPerYear = [4,12]`.

After untouched finalization and verification, the canonical research point/run treatment/configuration rows were changed to `[999,12]`. The immutable plan/state, child bundles, research identity and point/run identities were preserved. The modified artifact digests and a fresh producer-defined `study-result-v1-*` identity were then recomputed, producing a binding that was internally self-consistent rather than merely stale.

Exact evidence:

- PR #739 head: `86139b79459771a9dca6860907e6f3c9eec69627`;
- dedicated workflow: `34386105696`;
- dedicated job: `102582653156`;
- central CI: `34386105450` — success;
- all separate scientific/security/provenance/determinism/observability/bundle/resume workflows: success.

Dedicated output:

```text
immutable_plan_treatments=[4, 12]
forged_points_treatments=[999, 12]
forged_runs_treatments=[999, 12]
forged_result_identity=study-result-v1-6ce1e03b16ffd143
producer_finalize_rejects=true
root_verifier_accepted=true
```

The authoritative `anthrosim-study finalize` producer rejected the forged canonical analysis bytes against immutable research plan/state, confirming the AV4-013 producer repair remains effective. The root-aware finalized-study verifier nevertheless accepted the freshly reconstructed self-consistent binding over those contradictory treatment rows.

Finding: **AV6-013/#741 P1 — finalized-study root verifier accepts semantically forged canonical analysis rows.** PR #739 was closed unmerged after the finding was preserved; remediation remains deferred.

### Clean independent control — cross-root finalized binding transplant

Evidence-only PR #740 created two distinct frozen study/protocol roots with byte-identical research outputs, verified both untouched roots, then transplanted one valid `study-result-binding.json` into the other root.

Exact head `9566eb98116a68618c95271c7c672c864ebd72e1`; dedicated run `34386183485`; job `102582914744`; central CI `34386183517` and all separate protected/scientific workflows passed.

```text
research_artifacts_byte_identical=true
untouched_a_verified=true
untouched_b_verified=true
transplanted_binding_rejected=true
accepted=false
error=study result binding field studyExecutionId does not match the frozen study plan/protocol
```

Thus the root-aware verifier correctly binds finalized results to their own frozen study/protocol root. No new finding was demonstrated. PR #740 was closed unmerged.

### Frozen-target orchestration/provenance interpretation

The Area-K evidence confirms that v0.3.6/v35 retains strong producer-side semantic validation of canonical research indexes and strong study/protocol/root binding. AV6-013 is localized to missing verifier parity with the producer's canonical-analysis semantic contract.

AV6-010/#729 and AV6-012/#737 remain separate downstream value-binding defects and are not double-counted as Area-K findings.

Detailed Area-K completion evidence: `docs/research/audit-v6/area-k-2026-09-09.md`.

## Area L — active discovery

Primary scope: **observability, analysis outputs and statistical summaries**.

Start Area L at zero fresh coverage. High-value falsification directions include:

- denominator and weighting semantics across runs, agents, households, cells and time;
- censoring/extinction/incomplete-run handling in derived summaries;
- time-window and time-aggregation boundaries;
- nominal versus realized quantities and explicit missingness semantics;
- summary statistics that can conceal multimodality or incompatible regimes;
- whether uncertainty accompanies claim-driving estimates;
- accidental pooling of scientifically incompatible runs or configurations;
- whether AV6-007/#718, AV6-010/#729, AV6-012/#737 or AV6-013/#741 can propagate into analysis outputs without simply duplicating those findings.

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
| L | Observability, analysis outputs, statistical summaries | **active** | Fresh weighting/denominator/censoring/time-window/multimodality/pooling attack required. |
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
| `AV6-012` | **P1** | J primary; K/L/M/N | #735 head `fa46d25...`; same bound executions become falsely identifying after one contradictory deterministic output | **#737** | open; remediation deferred |
| `AV6-013` | **P1** | K primary; J/L/M/N | #739 head `86139b7...`; producer rejects forged `[999,12]` canonical rows while root-aware verifier accepts fresh self-consistent binding | **#741** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh discovery through Areas L–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–K are complete** once the Area-K completion documentation reaches protected `main`. Thirteen findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2, AV6-008/#721 P2, AV6-009/#726 P1, AV6-010/#729 P1, AV6-011/#733 P2, AV6-012/#737 P1 and AV6-013/#741 P1.

Next action: continue **Area L — observability, analysis outputs and statistical summaries** with a genuinely fresh adversarial attack against immutable v0.3.6/v35. Reconstruct live state/overlap before every new evidence branch; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.
