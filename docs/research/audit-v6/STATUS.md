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
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **14/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass: 14 findings — 7 P1, 7 P2** |
| Open Audit-v6 findings | **14** |
| Open P0/P1 | **7 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737; AV6-013/#741** |
| Phase | **discovery complete — remediation/disposition next** |
| Active ownership | **no discovery Area active; begin controlled finding remediation after this completion record is on protected `main`** |
| Production remediation | **allowed only after authoritative 14/14 discovery completion; use dedicated repair PRs and preserve original evidence** |
| Convergence status | **v6 cannot be P1-clean because seven new P1 findings were demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

Protected `main` immediately before final Area-N evidence was `d341d0a6462f175bcfd1f26df148d074e9f80f4a`. Scientific discovery remains attributed to immutable `v0.3.6` / v35 even though audit-only documentation commits advanced `main`.

## Completed Areas

| Area | Status | Fresh v6 evidence |
|---|---|---|
| A — authoritative semantics and scheduler behaviour | **complete — AV6-001/#687 P1** | #684 clean scheduler-equivalence control; #686/#687 same-day M4-before-newly-due-M9 inversion. `area-a-2026-09-09.md` |
| B — demography, fertility, mortality, ageing and population structure | **complete — AV6-002/#694 P2** | #692 clean chronology/cadence; #693/#694 inconsistent male-parent age-window references. `area-b-2026-09-09.md` |
| C — households, kinship, social links and lifecycle structure | **complete — AV6-003/#699 P2** | #698/#699 external-kin context collapse; #701 clean relabelling control. `area-c-2026-09-09.md` |
| D — resources, condition, subsistence, depletion/recovery | **complete — AV6-004/#707 P1; AV6-005/#708 P2** | scarce-resource reflection and duration-split household-index defects. `area-d-2026-09-09.md` |
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1** | unreachable impassable padding changes tied M9 destination; locality controls clean. `area-e-2026-09-09.md` |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | touching half-open visits inflate `peakVisitors`; lifecycle control clean. `area-f-2026-09-09.md` |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | incomplete founder genealogy consumed as absence; reproductive-history resume control clean. `area-g-2026-09-09.md` |
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1** | sequential Wilson undercoverage; confirmatory sample values not bound to authoritative outputs. `area-h-2026-09-09.md` |
| I — sensitivity, uncertainty, convergence and robustness | **complete — AV6-011/#733 P2** | no-op long-run sensitivity declarations can satisfy coverage; spatial extent late-divergence control clean. `area-i-2026-09-09.md` |
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1** | deterministic claim-driving outputs not bound to authoritative executions; equifinality control clean. `area-j-2026-09-09.md` |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1** | finalized-study root verifier accepts semantically forged canonical rows; cross-root transplant control clean. `area-k-2026-09-09.md` |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | drifting initialization families collapse to identical status distributions; exposure/extinction denominator control clean. `area-l-2026-09-09.md` |
| M — documentation, TRACE/ODD/ODD+D and claim consistency | **complete — non-clean via existing AV6-001, AV6-013, AV6-014; no new finding** | #747 claim-consistency adversary; TRACE empirical boundary remains conservative. `area-m-2026-09-09.md` |
| N — cross-system integration | **complete — non-clean via existing v6 findings; no new root finding** | #749 quantifies AV6-006 propagation from M9 destination into 5 visitor-person-days and 5 resource units. `area-n-2026-09-09.md` |

## Final Area-N evidence

Evidence-only PR **#749** composes AV6-006's M9 spatial-domain locality defect with actual temporary aggregation and M3 duration-aware resource accounting.

Final candidate evidence head: `e0b636735c486e0318004cb9cbe558de50d02ced`  
Dedicated workflow: `34409746294`

The corrected controlled fixture preserves the same reachable 3-cell local problem and adds only one outside-focal-region cell whose movement cost is `6000`, above the M9 traversal ceiling `5000`.

At process seed 0 the full authoritative host produces:

```text
baseline_destination=CellId(1)
padded_destination=CellId(3)
baseline_visitor_person_days=[5, 0, 0]
padded_visitor_person_days=[0, 0, 5]
baseline_food_stock=[9995, 9640, 10000]
padded_food_stock=[10000, 9640, 9995]
```

Therefore the causally irrelevant impassable padding moves **five actual visitor-person-days** and exactly **five units of M3 visitor resource demand** between unchanged local cells. This is a fresh cross-system consequence of AV6-006/#711, not a separate smallest defect, so no duplicate AV6 issue is created.

Two earlier #749 attempts are excluded from evidence: one compile-only fixture error and one zero-capacity fixture error. The scientific oracle was unchanged. See `area-n-2026-09-09.md` for the full evidence hygiene and integration synthesis.

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Issue | State |
|---|---:|---|---|---|
| AV6-001 | **P1** | A primary; D/E/F/M/N | #687 | open; remediation next |
| AV6-002 | **P2** | B primary; C/G/M/N | #694 | open; remediation next |
| AV6-003 | **P2** | C primary; E/N | #699 | open; remediation next |
| AV6-004 | **P1** | D primary; E/H/N | #707 | open; remediation next |
| AV6-005 | **P2** | D primary; C/E/F/N | #708 | open; remediation next |
| AV6-006 | **P1** | E primary; F/H/I/N | #711 | open; remediation next |
| AV6-007 | **P2** | F primary; E/L/N | #718 | open; remediation next |
| AV6-008 | **P2** | G primary; C/M/N | #721 | open; remediation next |
| AV6-009 | **P1** | H primary; K/L/M/N | #726 | open; remediation next |
| AV6-010 | **P1** | H primary; K/L/M/N | #729 | open; remediation next |
| AV6-011 | **P2** | I primary; K/L/M/N | #733 | open; remediation next |
| AV6-012 | **P1** | J primary; K/L/M/N | #737 | open; remediation next |
| AV6-013 | **P1** | K primary; J/L/M/N | #741 | open; remediation next |
| AV6-014 | **P2** | L primary; G/I/M/N | #745 | open; remediation next |

## Discovery conclusion and convergence

Scientific Audit v6 discovery is complete across the required A–N surface.

The result is **non-clean**. Seven P1 findings mean the charter's desired P1-clean convergence pass is impossible for v6, even if every defect is subsequently repaired.

Required next path:

1. preserve this 14/14 discovery state and all evidence-only PRs/issues;
2. remediate the 14 findings on dedicated production branches/PRs in dependency/severity order;
3. run exact-head protected/scientific CI for each repair;
4. independently re-run/re-verify the original adversarial contract for every P0/P1 repair, using the version-drift addendum where live repository evolution matters;
5. disposition/retest P2 findings explicitly;
6. freeze the fully repaired line as a new immutable release;
7. run another genuinely fresh audit generation before the framework-convergence gate for empirical work can be satisfied.

A completed framework audit does **not** establish empirical or archaeological validity. Site-specific work still requires its own evidence, parameterization, uncertainty, sensitivity, identifiability and domain review.

## Current handoff

Audit-v6 discovery Areas **A–N are complete** with 14 open findings. No discovery Area remains active.

Next action: begin **post-Audit-v6 remediation**, starting with the highest-severity/dependency-appropriate unresolved P1 rather than repairing findings in arbitrary issue-number order. Reconstruct live `main`, open PRs/issues and overlap before selecting the first repair.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and `docs/research/audit-v6/area-n-2026-09-09.md`. Verify live `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35; do not redo discovery. Begin controlled remediation by severity/dependency, preserve original discovery evidence, and independently reverify every P0/P1 repair before closure.
