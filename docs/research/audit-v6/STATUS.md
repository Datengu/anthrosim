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
| Discovery result | **non-clean: 8 findings so far — 3 P1, 5 P2; discovery continues through H–N** |
| Authoritative Audit-v6 findings | **8 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1; AV6-007/#718 P2; AV6-008/#721 P2** |
| Open Audit-v6 findings | **8** |
| Open P0/P1 | **3 — AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Phase | **Area H discovery next — stochasticity, RNG, ensembles and Monte Carlo inference** |
| Active ownership | **H — stochasticity, RNG, ensembles and Monte Carlo inference** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings have been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

### Area A — authoritative semantics and scheduler behaviour

Status: **complete — AV6-001 / #687 P1 open**  
Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Fresh v6 evidence: #684 clean scheduler-equivalence controls; #686/#687 demonstrated same-day M4-before-newly-due-M9 inversion on day 91, **AV6-001 P1**.

### Area B — demography, fertility, mortality, ageing and population structure

Status: **complete — AV6-002 / #694 P2 open**  
Completion report: `docs/research/audit-v6/area-b-2026-09-09.md`

Fresh v6 evidence: #692 clean birthday-crossing × mortality-cadence challenge; #693/#694 demonstrated incompatible male-parent age-window time references between dynamic M2 and declared-founder genealogy, **AV6-002 P2**. #690 was retained as known sequential-RNG/common-random-number scope rather than a new finding.

### Area C — households, kinship, social links and lifecycle structure

Status: **complete — AV6-003 / #699 P2 open**  
Completion report: `docs/research/audit-v6/area-c-2026-09-09.md`

Fresh v6 evidence: #698/#699 demonstrated external-kin context collapse before the final PersonId fission tie-break, **AV6-003 P2**; corrected #701 preserved daughter partitions under source-HouseholdId relabelling, no finding.

### Area D — resources, condition, subsistence, depletion/recovery

Status: **complete — AV6-004 / #707 P1 and AV6-005 / #708 P2 open**  
Completion report: `docs/research/audit-v6/area-d-2026-09-09.md`

Fresh v6 evidence:
- #705/#707: resource equal-remainder allocation changed condition `(1000,0) -> (0,1000)` under pure cell reflection; final head `f8ee562...`; CI `34356842006`, job `102483639065`; **AV6-004 P1**.
- #706/#708: exact 50/50 M9 duration split changed `(home=1,visitor=0) -> (0,1)` under canonical household-index relabelling; final head `ae667fe...`; CI `34356930794`, job `102483966686`; **AV6-005 P2**.

### Area E — spatial landscape, movement, migration, temporary mobility and boundaries

Status: **complete — AV6-006 / #711 P1 open**  
Completion report: `docs/research/audit-v6/area-e-2026-09-09.md`

Fresh v6 evidence:
- #710/#711: one unreachable impassable padding cell outside an unchanged local two-way M9 tie changed authoritative destination `CellId(1) -> CellId(3)` at seed 0; final head `cd1dd83...`; CI `34358664992`, job `102489797483`; **AV6-006 P1**.
- #713: M4 physical candidate geometry remained invariant across outer-domain padding, clean.
- #714: boundary-adjacent unique-destination M9 route cost/duration remained invariant behind impassable padding, clean.

### Area F — aggregation and interaction mechanisms

Status: **complete — AV6-007 / #718 P2 open**  
Completion report: `docs/research/audit-v6/area-f-2026-09-09.md`

Fresh v6 evidence:
- #716: repeated aggregation triggers at active-return/exact-completion boundaries behaved as declared, clean.
- #717/#718: touching half-open visit intervals `[102,112)` and `[112,122)` reported `peakVisitors=2` despite no positive-duration overlap, while visitor person-days remained correct; final head `416da690...`; CI `34366149670`, job `102515335302`; **AV6-007 P2**.

### Area G — initialization, burn-in, path dependence and continuation state

Status: **complete — AV6-008 / #721 P2 open**  
Completion report: `docs/research/audit-v6/area-g-2026-09-09.md`

Fresh v6 evidence:
- #720/#721: dependency-aware household fission accepted `FounderGenealogyStatus::Unspecified` and therefore consumed epistemically unknown missing parent links as structural absence. Final controlled head `550a8edbce9f9b6a96e86a8f32a27bf0d46d528c`; CI `34370338510`; `Quality and tests` job `102529686398`; format/Clippy and all **284 pre-existing core tests** passed before only the fresh fail-closed oracle failed. **AV6-008 P2**.
- #723: model-period reproductive history correctly superseded older immutable founder reproductive history across checkpoint/resume. Final controlled head `6f2ce4bd8acf1f67819a6eaba2f025b07d93209b`; CI `34371950740`; `Quality and tests` job `102535198421`; all **284 pre-existing core tests**, the fresh oracle and the complete downstream CI tail passed. Maternal births remained exactly `[365,1095]`, day 730 remained spacing-blocked, and resumed causal state matched uninterrupted execution. Clean no finding.

Frozen-source and interpretation review also confirmed that initialization is treated as causal rather than neutral, elapsed time alone is not treated as burn-in/equilibrium evidence, and persistent initialization effects/multiple regimes are required to remain visible rather than silently pooled.

Area G introduced no production semantics changes. AV6-008 remains open and deferred behind the A–N discovery barrier.

## Area H ownership / next discovery session

Area H starts from zero after the Area-G disposition is merged. Before substantive evidence, reconstruct live `main`, open PRs/issues and overlapping stochastic/inference audit work.

Primary scope: **stochasticity, RNG, ensembles and Monte Carlo inference**.

Fresh attacks should challenge genuinely new seams such as:

- seed-role and named-stream identity without replaying repaired AV5-003/#627;
- draw ordering and coupling locality;
- conditional draw consumption and limits of common-random-number interpretation;
- representation/remote-state perturbations that should not reassign local stochastic outcomes;
- rare-event behavior, stopping rules and censoring interaction;
- ensemble replicate sufficiency and precision claims;
- Monte Carlo aggregation and uncertainty reporting;
- large-integer/count fidelity in statistical summaries;
- paired-seed designs after structural divergence, where agent-level coupling may cease to be meaningful;
- random-event endpoints and low-probability limiting cases.

Known v6 context not to double-count:

- AV6-004/#707 is cross-cutting to H because a spatial reflection changes scarce-resource tie allocation and condition, but it is primarily an Area-D resource/fairness defect;
- AV6-006/#711 is cross-cutting to H because remote spatial padding perturbs an M9 tie realization, but it is primarily an Area-E spatial-locality defect;
- AV6-002/#694 touches demographic chronology, not a fresh RNG/inference defect;
- existing permanent RNG, label-invariance, ensemble soak and Monte Carlo script tests are controls/attack-design context, not fresh Area-H credit.

Historical Audit-v5 stochastic/inference findings and experiments must be reviewed before opening new evidence so repaired or already-classified defects are not duplicated.

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
| G | Initialization, burn-in, path dependence, continuation state | **complete — AV6-008 P2** | `area-g-2026-09-09.md`; #720/#721 P2; #723 clean. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **next/active** | Fresh RNG/coupling, rare-event, stopping/censoring, ensemble precision and Monte Carlo attacks. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Parameter/structure/horizon/resolution/initialization/replicate sensitivity and hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Compatible regions, parameter compensation, structural equifinality, held-out discrimination, tolerances. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Defaults, sweeps, retry/resume/crash recovery, identities, artifact integrity, replay. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Denominators, weighting, censoring/missingness, time windows, multimodality, incompatible-run mixing. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Executable v35 semantics versus living claims, null-model boundaries, releases, benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across neighbouring mechanisms and scientific-analysis surfaces. |

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | State |
|---|---:|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F/D/N | #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; M4 seq1 then M9 seq2 day91 | **#687** | open; remediation deferred |
| `AV6-002` | **P2** | B primary; C/G/M/N | #693 head `579cc00...`; dynamic/founder male-age reference inversion | **#694** | open; remediation deferred |
| `AV6-003` | **P2** | C primary; E/N | #698 head `3733700...`; external-kin orientation flips under PersonId relabelling | **#699** | open; remediation deferred |
| `AV6-004` | **P1** | D primary; E/H/N | #705 head `f8ee562...`; CI `34356842006` job `102483639065`; condition flips under cell reflection | **#707** | open; remediation deferred |
| `AV6-005` | **P2** | D primary; C/E/F/N | #706 head `ae667fe...`; CI `34356930794` job `102483966686`; duration split flips under household-index relabelling | **#708** | open; remediation deferred |
| `AV6-006` | **P1** | E primary; F/H/I/N | #710 head `cd1dd83...`; CI `34358664992` job `102489797483`; unreachable padding changes `CellId(1) -> CellId(3)` at seed0 | **#711** | open; remediation deferred |
| `AV6-007` | **P2** | F primary; E/L/N | #717 head `416da690...`; CI `34366149670` job `102515335302`; touching visits report peak 2 instead of physical maximum 1 | **#718** | open; remediation deferred |
| `AV6-008` | **P2** | G primary; C/M/N | #720 head `550a8ed...`; CI `34370338510` job `102529686398`; dependency-aware fission accepts `Unspecified` founder genealogy as structural absence | **#721** | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:
1. finish fresh discovery through Areas H–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–G are complete**. Area H is **next/active**. Eight findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2, AV6-006/#711 P1, AV6-007/#718 P2 and AV6-008/#721 P2.

Next action: begin **Area H — stochasticity, RNG, ensembles and Monte Carlo inference** from zero against immutable v0.3.6/v35. Reconstruct live state and overlap first; review prior stochastic/inference audit findings before designing fresh evidence; do not repair any v6 finding during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and update this ledger before handoff.