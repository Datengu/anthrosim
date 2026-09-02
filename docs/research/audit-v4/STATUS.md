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
| Coverage state | **2/14 Areas complete — Areas A-B complete; Area C next** |
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
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV4-001/002/004/005/006 cross-cutting** | Fresh v4 passes: PR #501 / run `33690928936` tested 30,000-person annual background mortality under 1/12/365 M3 partitions and found all observed death counts and pairwise differences well inside six-sigma binomial envelopes; PR #503 / run `33691311786` passed exact female fertility-age and executable birth-spacing boundaries; PR #505 / run `33693515792` passed exact background-mortality age-band boundaries; PR #506 / run `33694660962` measured finite-population extinction/censoring over 200 seeds × 120 years (N=20: 28/200 extinct, all-run terminal mean 11.240 vs survivor-only 13.070; N=200: 0/200 extinct, mean 119.255), retained as an explicit analysis obligation rather than a new defect; PR #507 / run `33696946835` passed mate presence and exact male `[18,70)` eligibility boundaries. Authoritative demography/config/founder chronology code was inspected; the existing no-universal-demographic-baseline policy remains the correct interpretation boundary. No additional Area-B defect beyond the cross-cutting Area-A findings was demonstrated. |
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

### 2026-09-02/03 — Area B discovery

- PR #501 / run `33690928936`: 30,000-person Monte Carlo at annual background mortality 0.2 gave 6022, 5984 and 6015 deaths for 1, 12 and 365 M3 periods/year respectively; z-scores against the 6000 expectation were 0.318, 0.231 and 0.217 and pairwise partition differences were small. No time-basis finding.
- PR #503 / run `33691311786`: exact female fertility-band lower/upper edges and the rounded executable birth-spacing threshold passed. No finding.
- PR #505 / run `33693515792`: exact background-mortality age-band lower/upper edges passed. No finding.
- PR #506 / run `33694660962`, job `100460844628`: 200-seed finite-population stress test over 120 years found **14.0% extinction at N=20** versus **0% at N=200**. Survivor-only terminal population at N=20 was 13.070 versus all-run 11.240, a +1.830 (+16.28%) censoring shift. Extinction is an explicit terminal state and current scientific-analysis contracts already require censoring/extinction treatment, so this is retained as a material model/analysis obligation rather than a new implementation defect. Central CI's only failure was rustfmt line wrapping in the disposable evidence fixture; the dedicated experiment itself compiled and passed.
- PR #507 / run `33696946835`, job `100467763467`: no-male, lower-age-edge and upper-age-edge mate-limitation cases passed exactly: 0, 0, 1, 1, 0 births for no male, 18y−1d, 18y, 70y−1d and 70y respectively. Central CI again failed only disposable-fixture formatting; dedicated scientific execution passed.
- Source inspection covered the authoritative demography schedule lookup, annual transition semantics, competing-mortality integration, founder reproductive chronology, birth-spacing normalization, mate gating and stop reasons. Neighbouring household/resource/spatial dependencies are carried forward to Areas C/D/E/N rather than treated as marginal-demography closure evidence.
- **Area B discovery is complete. No new Area-B-specific finding was demonstrated beyond AV4-001/002/004/005/006 already preserved from Area A. Audit v4 advances to Area C.**
