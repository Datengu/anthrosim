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
| Discovery coverage | **4/14 Areas A–N complete** |
| Discovery result | **non-clean: 6 findings so far — 3 P1, 3 P2; Area E remains in progress** |
| Authoritative Audit-v6 findings | **6 — AV6-001/#687 P1; AV6-002/#694 P2; AV6-003/#699 P2; AV6-004/#707 P1; AV6-005/#708 P2; AV6-006/#711 P1** |
| Open Audit-v6 findings | **6** |
| Open P0/P1 | **3 — AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Phase | **Area E discovery in progress — spatial landscape, movement, migration, temporary mobility, boundaries** |
| Active ownership | **E — spatial landscape, movement, migration, temporary mobility, boundaries** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings have been demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`.

## Completed Areas

### Area A — authoritative semantics and scheduler behaviour

Status: **complete — AV6-001 / #687 P1 open**  
Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Fresh evidence: #684 produced 20 exact scheduler-equivalence checkpoint comparisons with no finding; #686/#687 demonstrated same-day M4-before-newly-due-M9 inversion on day 91, **AV6-001 P1**.

### Area B — demography, fertility, mortality, ageing and population structure

Status: **complete — AV6-002 / #694 P2 open**  
Completion report: `docs/research/audit-v6/area-b-2026-09-09.md`

Fresh evidence: #690 was known sequential-RNG/common-random-number scope; #692 passed the birthday-crossing × mortality-cadence quantitative challenge; #693/#694 demonstrated incompatible male-parent age-window time references between dynamic M2 and declared-founder genealogy, **AV6-002 P2**.

### Area C — households, kinship, social links and lifecycle structure

Status: **complete — AV6-003 / #699 P2 open**  
Completion report: `docs/research/audit-v6/area-c-2026-09-09.md`

Fresh evidence: #698/#699 demonstrated external-kin context collapse before the final PersonId fission tie-break, **AV6-003 P2**; corrected #701 preserved physical daughter partitions under source-HouseholdId relabelling, no finding.

### Area D — resources, condition, subsistence, depletion/recovery

Status: **complete — AV6-004 / #707 P1 and AV6-005 / #708 P2 open**  
Completion report: `docs/research/audit-v6/area-d-2026-09-09.md`

Fresh evidence:
- #705/#707: resource equal-remainder allocation changed condition `(1000,0) -> (0,1000)` under pure cell reflection; final head `f8ee562027ae162afb9ce7a5f00a6ca3382b0f33`, CI `34356842006`, job `102483639065`; 284 pre-existing core tests passed; **AV6-004 P1**.
- #706/#708: exact 50/50 M9 duration split changed `(home=1,visitor=0) -> (0,1)` under canonical household-index relabelling; final head `ae667febd0930f43b50eae0af459c5fc8201f13f`, CI `34356930794`, job `102483966686`; 284 pre-existing core tests passed; **AV6-005 P2**.

## Area E — discovery in progress

Area E has one fresh demonstrated finding so far and remains open for at least one independent second attack before disposition.

### AV6-006 / #711 — P1

Evidence-only PR #710 tested whether causally isolated spatial-domain padding can change a local M9 equal-cost tie.

Controlled construction:
- baseline 3×1 world, movement costs `[1000,1000,1000]`;
- origin `CellId(2)`, focal destinations `{CellId(1), CellId(3)}`;
- both destinations remain one edge away at accumulated cost `1000`;
- padded arm preserves cells 1–3 and appends only `CellId(4)` with movement cost `6000` above the declared M9 traversability ceiling `5000`;
- same tie seed, household coupling key and trigger;
- positive controls prove identical local route cost, candidate count and route distances and prove the padding cell non-traversable.

Final controlled evidence head `cd1dd83abf1a83911c4c11e3d47fc662175d580a`; central CI run `34358664992`, `Quality and tests` job `102489797483`. Format and Clippy passed; all **284 pre-existing anthrosim-core unit tests** passed; only the fresh locality oracle failed:

```text
seed=0 local M9 padding control: baseline=CellId(1) padded=CellId(3)
```

Mandatory duplicate search found no prior record of this failure. It is distinct from #190 lower-CellId bias, AV4-007/#500 HouseholdId-label dependence, AV5-001/#606 remote-founder/global-rank dependence and AV5-004/#629 whole-problem reflection non-equivariance. The v35 whole-grid canonical frame leaks causally isolated spatial extent into a local M9 tie. Preserved as **AV6-006/#711 P1**; #710 closed unmerged.

Next Area-E work should attack an independent spatial mechanism rather than merely widen AV6-006. A preferred direction is a local M4 permanent-migration choice under causally isolated impassable padding, because v35 M4 uses bounded local candidate/equivalence semantics and should provide an independent clean control if its locality contract holds.

Known cross-cutting context not to double-count:
- AV6-001/#687 same-day M9/M4 scheduler inversion;
- AV6-003/#699 external-kin fission context;
- AV6-004/#707 resource-cell reflection;
- AV6-005/#708 home/visitor resource attribution;
- historical M4/M9 reflection, HouseholdId and stochastic-coupling findings are controls/hypothesis sources only.

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
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1** | #684 no finding; #686/#687 P1. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV6-002 P2** | #690 known coupling scope; #692 no finding; #693/#694 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV6-003 P2** | #698/#699 P2; #701 no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV6-004 P1 + AV6-005 P2** | `area-d-2026-09-09.md`; #705/#707; #706/#708. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **in progress — AV6-006 P1 open** | #710/#711 impassable-padding M9 locality failure; next independent M4/locality or boundary attack. |
| F | Aggregation and interaction mechanisms | **not started** | Trigger/timing, concentration vs relocation, interaction accounting, crowding/recovery, distinguishability. |
| G | Initialization, burn-in, path dependence, continuation state | **not started** | Alternative starts, transient/stationary interpretation, checkpoint continuation, path dependence. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **not started** | Seed/stream identity, draw ordering/coupling, rare events, stopping rules, replicate sufficiency, censoring, precision. |
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

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:
1. finish fresh discovery through Areas E–N against immutable v0.3.6/v35;
2. disposition all additional findings;
3. only after discovery, remediate by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–D are complete**. Area E is **in progress**. Six findings are open and deliberately unrepaired: AV6-001/#687 P1, AV6-002/#694 P2, AV6-003/#699 P2, AV6-004/#707 P1, AV6-005/#708 P2 and AV6-006/#711 P1.

Next action: continue Area E with a genuinely independent attack against immutable v0.3.6/v35, preferably a local M4 permanent-migration padding/locality control. Reconstruct live state/overlap before new evidence and do not repair AV6-006 during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and update this ledger before handoff.
