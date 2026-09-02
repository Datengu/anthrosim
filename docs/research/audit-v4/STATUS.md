# AnthroSim scientific audit v4 — status ledger

Audit target: immutable AnthroSim `v0.3.4`, tag commit `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`, model semantics `anthrosim-model-semantics-v25`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v4/README.md`

Purpose: durable repository-authoritative state for the fourth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v3 convergence pass.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v4 / fourth independent scientific audit |
| Immutable discovery target | `v0.3.4` |
| Target tag SHA | `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` |
| Target software version | `0.3.4` |
| Target model semantics | `anthrosim-model-semantics-v25` |
| Coverage state | **1/14 Areas complete — Area A complete; Area B in progress** |
| Current P0 findings | none discovered |
| Current P1 findings | **7 open — AV4-001/#486; AV4-002/#488; AV4-003/#491; AV4-004/#493; AV4-005/#495; AV4-006/#497; AV4-007/#500** |
| Current P2 findings | none discovered |
| Current P3 findings | none discovered |
| Convergence classification | **non-clean candidate: v4 has discovered new P1 findings; full A–N discovery still pending** |
| Repair state | **discovery only; do not repair v4 findings until A–N discovery completes** |

## Discovery rule

- The immutable `v0.3.4` tag is the scientific discovery target.
- Audit v2/v3 evidence is historical context and regression-hypothesis material only.
- Each Area starts incomplete and requires fresh evidence.
- Preserve demonstrated defects in issues and this ledger before repair.
- Continue later Areas against the frozen v0.3.4 baseline even after findings are discovered.
- Production repair is deferred until A–N discovery is complete, absent an explicitly documented integrity emergency.

## Coverage matrix

| ID | Audit area | Status | Fresh v4 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — 7 P1 findings open** | Pass 1 scheduler collision attack: PR #483 / run `33686718180`, exhaustive **133,225** M3/M4 period-count pairs, no skipped/duplicated/reordered boundaries or inspected host-order drift. Passes 2–7 demonstrated independent label/order coupling in fertility (**AV4-001/#486**), background mortality (**AV4-002/#488**), M4 migration (**AV4-003/#491**), newborn sex (**AV4-004/#493**), parentage (**AV4-005/#495**), and condition-mediated mortality (**AV4-006/#497**). Pass 8 attacked M9's non-sequential keyed equal-cost destination policy and demonstrated **AV4-007/#500**. Frozen-source RNG inventory accounted for the runtime named streams: four demography streams, two migration streams and one resource/condition-mortality stream; M9's destination symmetry breaker is keyed separately. Initialization/world-generation streams are deferred to Areas G/E rather than silently treated as Area-A closure evidence. |
| B | Demography, fertility, mortality, ageing, population structure | **in progress — AV4-001/002/004/005/006 cross-cutting** | First fresh quantitative pass opened as PR #501: 30,000-person mortality partition Monte Carlo across 1/12/365 M3 periods/year. |
| C | Households, kinship, social links, lifecycle structure | **incomplete — AV4-003/005/007 cross-cutting** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete — AV4-006 cross-cutting** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete — AV4-001/002/003/004/006/007 cross-cutting** | — |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **incomplete — AV4-001 through AV4-007 cross-cutting** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **incomplete** | — |
| J | Identifiability, equifinality, calibration, discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete — AV4-001 through AV4-007 cross-cutting** | — |

## Finding register

| Finding | Severity | Area | Status | Issue | Fresh evidence |
|---|---|---|---|---|---|
| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #486 | Closed red PR #485 / run `33687262609`: seed-1 birth cell changes `CellId(1)` → `CellId(2)` under pure PersonId relabelling. |
| AV4-002 — background-mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #488 | Closed red PR #487 / run `33689235132`: seed-1 death cell changes `CellId(1)` → `CellId(2)`. |
| AV4-003 — migration RNG assignment is sensitive to arbitrary household labels | **P1** | A; C/E/H/N | **demonstrated; open; deliberately unrepaired** | #491 | Closed red PR #490 / run `33689659272`: seed 1 changes from move `(CellId(4), CellId(1))` to no move under pure HouseholdId relabelling. |
| AV4-004 — newborn-sex RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #493 | Closed red PR #492 / run `33689950122`: two forced births swap male/female assignment between fixed cells after PersonId relabelling. |
| AV4-005 — parentage RNG assignment is sensitive to arbitrary male person labels | **P1** | A; B/C/H/N | **demonstrated; open; deliberately unrepaired** | #495 | Closed red PR #494 / run `33690127728`: genealogy-preserving male-ID swap changes whether the newborn selects the existing child's father. |
| AV4-006 — condition-mediated mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/D/E/H/N | **demonstrated; open; deliberately unrepaired** | #497 | Closed red PR #496 / run `33690381210`, job `100447530200`: seed-1 condition-mediated death cell changes `CellId(2)` → `CellId(1)` with background mortality disabled. |
| AV4-007 — M9 equal-cost destination key is sensitive to arbitrary household labels | **P1** | A; C/E/H/N | **demonstrated; open; deliberately unrepaired** | #500 | Closed red PR #499 / run `33690671581`, job `100448444144`: same center-origin physical household selects `CellId(8)` as `HouseholdId(1)` and `CellId(2)` as `HouseholdId(2)` at seed 1. |

## Session log

### 2026-09-02 — Audit v4 initialization

- Frozen target selected: `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / semantics v25.
- Audit-v4 coverage initialized at zero; prior audits are historical context only.

### 2026-09-02 — Area A discovery

- Pass 1 exhaustively checked **365 × 365 = 133,225** M3/M4 schedule combinations. Same-day collision range was **1–365** and **128,400** pairs had more than one collision; no dispatch/order defect was demonstrated.
- Passes 2–7 attacked every persistent mutable runtime stochastic stream whose scientific realization is assigned to people/households: fertility, background mortality, migration choice/uncertainty, newborn sex, parentage and condition-mediated mortality. Six distinct P1 label/order defects were demonstrated and preserved as AV4-001 through AV4-006.
- Pass 8 attacked M9's explicit keyed equal-cost destination policy. In a uniform 3×3 geometry with exactly two equal-cost focal destinations, changing only the same physical household's canonical HouseholdId changed seed-1 destination `CellId(8)` ↔ `CellId(2)`, preserved as **AV4-007/#500 (P1)**.
- Frozen-source inventory accounted for the runtime named streams under Area A. Synthetic initialization and world-generation streams remain explicitly reserved for later initialization/spatial areas.
- All seven red evidence PRs were closed unmerged after issue preservation. No repairs were made.
- **Area A discovery is complete. Audit v4 advances to Area B while all seven findings remain deliberately unrepaired.**

### 2026-09-02 — Area B pass 1 started

- Open evidence PR #501 targets immutable v0.3.4/v25 with a **30,000-person** one-year Monte Carlo check of configured annual background mortality under **1, 12 and 365** M3 partitions/year.
- The test uses constant annual mortality `0.2`, zero fertility, zero condition-mediated mortality and no migration; observed annual deaths are checked against the binomial expectation and pairwise partition stability with conservative six-sigma envelopes.
