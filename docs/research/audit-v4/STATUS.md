# AnthroSim scientific audit v4 — status ledger

Audit target: immutable AnthroSim `v0.3.4`, tag commit `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`, model semantics `anthrosim-model-semantics-v25`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v4/README.md`

Purpose: durable repository-authoritative state for the fourth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v3 convergence pass.

> This ledger is intentionally compact current-state documentation. Earlier expanded per-session narratives remain preserved in the Git history and the closed evidence PRs/issues referenced below.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v4 / fourth independent scientific audit |
| Immutable discovery target | `v0.3.4` |
| Target tag SHA | `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` |
| Target software version | `0.3.4` |
| Target model semantics | `anthrosim-model-semantics-v25` |
| Coverage state | **10/14 Areas complete — Areas A-J complete; Area K next** |
| Current P0 findings | none discovered |
| Current P1 findings | **11 open — AV4-001/#486; AV4-002/#488; AV4-003/#491; AV4-004/#493; AV4-005/#495; AV4-006/#497; AV4-007/#500; AV4-008/#514; AV4-009/#518; AV4-010/#528; AV4-011/#535** |
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
- Evidence-only PRs are closed unmerged after classification.
- Harness/compiler/formatting failures are not scientific findings; correct and rerun before classification.

## Coverage matrix

| ID | Audit area | Status | Fresh v4 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — AV4-001 through AV4-007** | PR #483 / run `33686718180` exhaustively checked **133,225** M3/M4 schedule combinations with no scheduler collision defect. Independent fresh adversaries then demonstrated arbitrary-label/order coupling in fertility (**AV4-001/#486**), background mortality (**AV4-002/#488**), M4 migration (**AV4-003/#491**), newborn sex (**AV4-004/#493**), parentage (**AV4-005/#495**), condition-mediated mortality (**AV4-006/#497**) and M9 equal-cost destination selection (**AV4-007/#500**). |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV4-001/002/004/005/006 cross-cutting** | PR #501 / run `33690928936` tested 30,000-person background mortality under 1/12/365 partitions; PR #503 / `33691311786` passed fertility-age and birth-spacing edges; PR #505 / `33693515792` passed mortality-age boundaries; PR #506 / `33694660962` quantified 200-seed finite-population extinction/censoring (N=20: 28/200 extinct, survivor-only terminal mean 13.070 vs all-run 11.240; N=200: 0/200 extinct); PR #507 / `33696946835` passed mate-presence and male `[18,70)` boundaries. No additional B-specific defect. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV4-003/005/007 cross-cutting** | PR #509 / `33697318347` passed the exact 18-year independent-age fission threshold. PR #510 / `33697866190` demonstrated the declared temporary-presence/lifecycle phase contrast (returned household fissioned; household away at lifecycle boundary did not). No additional C-specific defect. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV4-006/008 cross-cutting** | PR #512 / `33698192834` passed full-scarcity 1/4/12/365 partition equivalence. PR #513 / run `33699132098`, job `100474385648` demonstrated **AV4-008/#514**: a one-unit shortage among equal colocated households changes which fixed person receives the condition penalty under pure HouseholdId rotation (`[1000,1000,996]` vs `[1000,996,1000]`). |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — AV4-001/002/003/004/006/007/009 cross-cutting** | PR #516 / run `33705627132`, job `100493999886` passed 48 radius/buffer boundary cases. PR #517 / final run `33706244290`, job `100495847910` demonstrated **AV4-009/#518**: one-household M4 weighted choice fails horizontal-reflection equivariance (`CellId(1)` stays `CellId(1)` instead of reflecting to `CellId(3)`). PR #519 / `33706705885`, job `100497268016` passed **2,240** M9 route-reflection comparisons. PR #520 / `33706870747`, job `100497764665` passed **168** temporary-mobility/resource boundary cases. |
| F | Aggregation and interaction mechanisms | **complete — no additional finding demonstrated** | PR #522 head `4b883f8b65f45a4bddc02b98bd0e26a10b3e083f`; central CI `33709978856` and applicable scientific/security gates `33709979126` passed. Two one-person households at one destination for five days produced exactly **10 visitor person-days**, **720 home person-days**, **730 total person-days** and the expected destination resource reduction. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — no additional finding demonstrated** | PR #524 head `59fc6f08caec6c5cf9eeb76c87859f929624f93e` passed exact continuation across **32 process seeds** with an active M9 journey, nonzero resources, demography, RNG state, events and metrics across a year-1 checkpoint. PR #525 final head `958a8d78a61e45706fd56a8288664528fdadbb69`, run `33740052632`, job `100599600258` quantified initialization path dependence: empty vs fully stocked start was **743 vs 7430** after one year but converged exactly to **7430 vs 7430** by the controlled ten-year refill/capacity horizon. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — AV4-001 through AV4-010 cross-cutting; new AV4-010 P1** | PR #527 head `369548384761942fff1151d1ffa35460e678c0b0`, run `33740457897`, job `100600897791` demonstrated **AV4-010/#528**: exact replicates `[2^53,2^53+1,2^53,2^53+1]` have exact 95% normal-CLT half-width **0.565792867038**, but binary64 coercion collapses all values, reports half-width **0** and false `sufficient_stop` at threshold 0.1. PR #529 head `3a9df0f122e72f1cd19cc1770deaa4d7f7541f8a`, run `33740878371`, job `100602226718` passed **136** safe-range estimator/seed-contract attacks with maximum half-width discrepancy 0. |
| I | Sensitivity, uncertainty, convergence, robustness | **complete — AV4-010 cross-cutting; no additional finding demonstrated** | PR #531 head `a2b9d4a4b3cc198afc0ec1e68113fe64a43e0408`, run `33741469209`, job `100604140981` passed all **48** transformed expansions of one 2×3×2 design (**576** coordinate checks), retaining exactly **12** executable treatments under all six dimension orders and eight value-order reversal masks; ancestor/descendant overlaps failed closed in both orders. Stale weaker overlap PR #532 was later closed unmerged after its only central-CI failure was disposable-fixture rustfmt. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — new AV4-011 P1; AV4-010 cross-cutting** | PR #534 head `93f6d306afd9c8831e785d38eaa77207daaaab0e`, run `33743566077`, job `100610814705` demonstrated **AV4-011/#535**: the preserved identifiability benchmark contains no `fabricated_theta`, yet adding that invented coordinate only to free-form downstream `point.parameters` rows gives explored range `[0,1]`, compatible range `[0,0]`, `identified=true` and top-level `researchGate.passes=true / declared_claim_identified`. The analyzer does not bind parameter rows back to immutable executed `ResearchPoint`/run configurations. PR #536 head `0b5f3dd30b37ce1239a4bc5b807f594a3b7fbf5a`, run `33743862940`, job `100611767504` then passed **122** fresh decision-matrix assertions covering deterministic boundaries, Monte Carlo unresolved compatibility, numeric/categorical identifiability, structural equifinality, conservative held-out envelopes, nuisance compensation, all 24 row-order permutations, evidence-role overlap rejection and duplicate point-ID rejection. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete — AV4-010/011 cross-cutting** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete — AV4-010 cross-cutting** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete — AV4-001 through AV4-011 cross-cutting** | — |

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
| AV4-008 — scarce-resource remainder assignment is sensitive to arbitrary household labels | **P1** | D; C/H/N | **demonstrated; open; deliberately unrepaired** | #514 | Closed red PR #513 / run `33699132098`, job `100474385648`: equal colocated household claims under a one-unit shortage change fixed-person terminal condition `[1000,1000,996]` → `[1000,996,1000]` under pure HouseholdId rotation. |
| AV4-009 — M4 weighted migration choice is sensitive to canonical spatial candidate order | **P1** | E; A/H/N | **demonstrated; open; deliberately unrepaired** | #518 | Closed red PR #517 / run `33706244290`, job `100495847910`, head `fce8503489bddfaffea93df1c5bc5ac6eb2d1bfa`: one fixed centre household chooses `CellId(1)` before and after pure horizontal reflection; mirrored physical destination should be `CellId(3)`. |
| AV4-010 — Monte Carlo precision gate can erase large exact-integer variance during float conversion | **P1** | H; I/J/K/L/N | **demonstrated; open; deliberately unrepaired** | #528 | Closed red PR #527 / head `369548384761942fff1151d1ffa35460e678c0b0`, run `33740457897`, job `100600897791`: exact integer variation gives half-width `0.565792867038`; binary64 coercion reports 0 and false `sufficient_stop`. |
| AV4-011 — identifiability gate can certify a fabricated parameter coordinate not bound to the executed design | **P1** | J; I/K/N | **demonstrated; open; deliberately unrepaired** | #535 | Closed red PR #534 / head `93f6d306afd9c8831e785d38eaa77207daaaab0e`, run `33743566077`, job `100610814705`: original benchmark has no `fabricated_theta`; injected analysis-table coordinate yields full range `[0,1]`, compatible `[0,0]`, `identified=true`, `researchGate.passes=true`, `declared_claim_identified`. |

## Current discovery handoff

### Area J closure — 2026-09-03

- Frozen source inspection covered `scripts/research-identifiability.py`, the identifiability/equifinality normative contract, its protected workflow, and downstream analysis-provenance v2.
- Historical Audit-v2/v3 findings were used only as attack hypotheses. Current v25 repairs were confirmed for Monte Carlo uncertainty coupling, fixed-by-design numeric parameters, typed structure identifiers, conservative held-out interval envelopes, nuisance compensation and overlap/evidence-role protections.
- **AV4-011/#535 (P1)** is new and distinct: free-form downstream parameter coordinates are not semantically resolved to immutable executed research points/configurations, permitting a parameter invented after execution to be positively certified as identified.
- The generic downstream provenance layer does not close AV4-011: it proves exact input bytes and replay lineage, but its own normative boundary explicitly states that provenance does not establish identifiability validity.
- PR #536 independently passed **122** clean decision-matrix assertions, isolating AV4-011 rather than indicating general failure of the repaired identifiability logic.
- All Area-J evidence PRs were closed unmerged after classification. No v4 finding was repaired.
- **Area J discovery is complete with one new P1 finding, AV4-011/#535. Audit v4 advances to Area K — experiment orchestration, configuration, provenance, reproducibility.**
