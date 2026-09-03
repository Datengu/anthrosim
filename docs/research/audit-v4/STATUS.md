# AnthroSim scientific audit v4 — status ledger

Audit target: immutable AnthroSim `v0.3.4`, tag commit `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`, model semantics `anthrosim-model-semantics-v25`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v4/README.md`

Purpose: durable repository-authoritative state for the fourth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v3 convergence pass.

> This ledger is intentionally compact current-state documentation. Earlier expanded per-session narratives remain preserved in Git history and the closed evidence PRs/issues referenced below.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v4 / fourth independent scientific audit |
| Immutable discovery target | `v0.3.4` |
| Target tag SHA | `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` |
| Target software version | `0.3.4` |
| Target model semantics | `anthrosim-model-semantics-v25` |
| Coverage state | **13/14 Areas complete — Areas A-M complete; Area N next** |
| Current P0 findings | none discovered |
| Current P1 findings | **13 open — AV4-001/#486; AV4-002/#488; AV4-003/#491; AV4-004/#493; AV4-005/#495; AV4-006/#497; AV4-007/#500; AV4-008/#514; AV4-009/#518; AV4-010/#528; AV4-011/#535; AV4-012/#539; AV4-013/#543** |
| Current P2 findings | **2 open — AV4-014/#546; AV4-015/#549** |
| Current P3 findings | none discovered |
| Convergence classification | **non-clean candidate: v4 has discovered new P1/P2 findings; final Area N discovery remains pending** |
| Repair state | **discovery only; do not repair v4 findings until A–N discovery completes** |

## Discovery rule

- The immutable `v0.3.4` tag is the scientific discovery target.
- Audit v2/v3 evidence is historical context and regression-hypothesis material only.
- Each Area starts incomplete and requires fresh evidence.
- Preserve demonstrated defects in issues and this ledger before repair.
- Continue later Areas against the frozen v0.3.4 baseline even after findings are discovered.
- Production repair is deferred until A–N discovery is complete, absent an explicitly documented integrity emergency.
- Evidence-only PRs are closed unmerged after classification.
- Harness/compiler/formatting failures are not scientific findings; correct/rerun or rely on a separate clean dedicated scientific execution before classification.

## Coverage matrix

| ID | Audit area | Status | Fresh v4 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — AV4-001 through AV4-007** | PR #483 / run `33686718180` exhaustively checked **133,225** M3/M4 schedule combinations. Fresh adversaries then demonstrated arbitrary-label/order coupling in fertility, background mortality, M4 migration, newborn sex, parentage, condition-mediated mortality and M9 equal-cost destination selection: **AV4-001/#486 through AV4-007/#500**. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV4-001/002/004/005/006 cross-cutting** | PRs #501/#503/#505/#506/#507 covered mortality partitioning, fertility/birth-spacing edges, mortality-age edges, 200-seed extinction/censoring and mate-age/presence boundaries. No additional B-specific defect. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV4-003/005/007 cross-cutting** | PR #509 passed the exact independent-age fission threshold; PR #510 quantified temporary-presence/lifecycle phase dependence. No additional C-specific defect. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV4-006/008 cross-cutting** | PR #512 passed 1/4/12/365 full-scarcity partition equivalence. PR #513 / run `33699132098`, job `100474385648` demonstrated **AV4-008/#514**: one-unit scarce-resource remainder assignment changes fixed-person condition under pure HouseholdId rotation. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — AV4-001/002/003/004/006/007/009 cross-cutting** | PR #516 passed 48 radius/buffer cases; PR #517 / run `33706244290`, job `100495847910` demonstrated **AV4-009/#518** M4 reflection/candidate-order dependence; PR #519 passed **2,240** M9 route-reflection comparisons; PR #520 passed **168** temporary-mobility/resource boundary cases. |
| F | Aggregation and interaction mechanisms | **complete — no additional finding demonstrated** | PR #522 exact-head CI passed; two one-person households aggregated for five days produced exactly 10 visitor person-days, 720 home person-days and 730 total person-days with expected resource pressure. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — no additional finding demonstrated** | PR #524 passed exact active-state continuation across **32 process seeds**. PR #525 / run `33740052632`, job `100599600258` quantified initial-stock path dependence: 743 vs 7430 after one year, exact 7430 vs 7430 convergence by the controlled ten-year horizon. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — AV4-001 through AV4-010 cross-cutting; new AV4-010 P1** | PR #527 / run `33740457897`, job `100600897791` demonstrated **AV4-010/#528**: binary64 conversion collapses exact integers around `2^53`, turning exact half-width `0.565792867038` into 0 and false `sufficient_stop`. PR #529 passed **136** safe-range estimator/seed-contract attacks. |
| I | Sensitivity, uncertainty, convergence, robustness | **complete — AV4-010 cross-cutting; no additional finding demonstrated** | PR #531 / run `33741469209`, job `100604140981` passed **48** transformed expansions / **576** coordinate checks, retaining exactly 12 executable treatments under all dimension/value declaration orders; ancestor/descendant overlaps failed closed. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — AV4-011 P1; AV4-010/013 cross-cutting** | PR #534 / run `33743566077`, job `100610814705` demonstrated **AV4-011/#535**: invented downstream parameter `fabricated_theta` absent from the executed design can be certified as identified. PR #536 passed **122** clean decision-matrix assertions. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete — AV4-012/013 P1; AV4-010/011 cross-cutting** | PR #538 / run `33744613505`, job `100614126094` demonstrated **AV4-012/#539**: downstream provenance accepts a study-result binding whose identity-covered `researchId` changed while stale `resultIdentity` remained. PR #540 / run `33745069053`, job `100615547018` passed **11** end-to-end research retry/tamper scenarios. PR #541 / run `33745315422`, job `100616318921` demonstrated **AV4-013/#543**: `anthrosim-study finalize` can create a fresh official result binding over tampered canonical analysis point/run treatment rows that contradict the immutable executed design. |
| L | Observability, analysis outputs, statistical summaries | **complete — AV4-014 P2; AV4-010/012/013 cross-cutting** | PR #544 / run `33745675477`, job `100617455028` passed **64** paired-summary row orderings, exact per-statistic contributing counts 6/3/4 under asymmetric missingness and three malformed pairing fail-closed attacks. PR #545 / run `33745956159`, job `100618351717` demonstrated **AV4-014/#546 (P2)**: the survivor-conditioning joint-population safeguard accepts `derived.not_a_real_mortality_observable` solely because its free-form source label contains `mortality`. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **complete — AV4-015 P2; AV4-014 cross-cutting** | PR #548 / final head `3da64b46f91ceb22637ce5588a05f8e2e64e0b5a`, run `33746630336`, job `100620484701` demonstrated **AV4-015/#549 (P2)**: living v0.3.4/v25 ODD has two pre-v15 annual-boundary mortality claims and ODD+D has one current claim that annual M2 mortality has declared priority, while frozen demography source/scientific-model use elapsed M3-interval order-invariant competing mortality and year-end fertility/parentage only. Current lifecycle/baseline identity and empirical-readiness surfaces were separately inspected and were synchronized/conservative. TRACE remains appropriately `NOT YET EMPIRICALLY RESEARCH-READY`, but its model-description row calls ODD/ODD+D established for v25, making the drift a current claim inconsistency. |
| N | Cross-system integration | **incomplete — AV4-001 through AV4-015 cross-cutting** | — |

## Finding register

| Finding | Severity | Area | Status | Issue | Fresh evidence |
|---|---|---|---|---|---|
| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #486 | PR #485 / run `33687262609`. |
| AV4-002 — background-mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #488 | PR #487 / run `33689235132`. |
| AV4-003 — migration RNG assignment is sensitive to arbitrary household labels | **P1** | A; C/E/H/N | **demonstrated; open; deliberately unrepaired** | #491 | PR #490 / run `33689659272`. |
| AV4-004 — newborn-sex RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #493 | PR #492 / run `33689950122`. |
| AV4-005 — parentage RNG assignment is sensitive to arbitrary male person labels | **P1** | A; B/C/H/N | **demonstrated; open; deliberately unrepaired** | #495 | PR #494 / run `33690127728`. |
| AV4-006 — condition-mediated mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/D/E/H/N | **demonstrated; open; deliberately unrepaired** | #497 | PR #496 / run `33690381210`, job `100447530200`. |
| AV4-007 — M9 equal-cost destination key is sensitive to arbitrary household labels | **P1** | A; C/E/H/N | **demonstrated; open; deliberately unrepaired** | #500 | PR #499 / run `33690671581`, job `100448444144`. |
| AV4-008 — scarce-resource remainder assignment is sensitive to arbitrary household labels | **P1** | D; C/H/N | **demonstrated; open; deliberately unrepaired** | #514 | PR #513 / run `33699132098`, job `100474385648`. |
| AV4-009 — M4 weighted migration choice is sensitive to canonical spatial candidate order | **P1** | E; A/H/N | **demonstrated; open; deliberately unrepaired** | #518 | PR #517 / run `33706244290`, job `100495847910`. |
| AV4-010 — Monte Carlo precision gate can erase large exact-integer variance during float conversion | **P1** | H; I/J/K/L/N | **demonstrated; open; deliberately unrepaired** | #528 | PR #527 / run `33740457897`, job `100600897791`. |
| AV4-011 — identifiability gate can certify a fabricated parameter coordinate not bound to the executed design | **P1** | J; I/K/N | **demonstrated; open; deliberately unrepaired** | #535 | PR #534 / run `33743566077`, job `100610814705`. |
| AV4-012 — downstream analysis provenance accepts a study-result binding with stale self-identity | **P1** | K; J/L/N | **demonstrated; open; deliberately unrepaired** | #539 | PR #538 / run `33744613505`, job `100614126094`. |
| AV4-013 — study finalization can bind canonical analysis rows that contradict the immutable executed design | **P1** | K; J/L/N | **demonstrated; open; deliberately unrepaired** | #543 | PR #541 / run `33745315422`, job `100616318921`: immutable treatment values `[4,12]` were changed to `[999,12]` in canonical analysis rows and `anthrosim-study finalize` still succeeded with a fresh result identity. |
| AV4-014 — survivor-conditioning gate accepts fabricated population observables by source substring | **P2** | L; M/N | **demonstrated; open; deliberately unrepaired** | #546 | PR #545 / run `33745956159`, job `100618351717`: genuine population source passes, unrelated source fails, but `derived.not_a_real_mortality_observable` passes with zero failures solely through substring matching. |
| AV4-015 — living ODD/ODD+D retain superseded annual-boundary mortality semantics | **P2** | M; B/N | **demonstrated; open; deliberately unrepaired** | #549 | PR #548 / final head `3da64b46f91ceb22637ce5588a05f8e2e64e0b5a`, run `33746630336`, job `100620484701`: two stale ODD annual-mortality claims plus one ODD+D mortality-priority claim contradict frozen v25 competing-risk/year-end-finalizer semantics. |

## Current discovery handoff

### Area M closure — 2026-09-03

- Fresh current-facing documentation review rechecked the prior v3 repair surfaces rather than treating old findings as closure evidence. The living demographic baseline correctly identifies `deterministic_dependency_fission_v2`, 130 fresh seeds per arm and explicitly separates the historical v1/64-seed page. `docs/scientific-model.md` correctly identifies v0.3.4/v25, dependency-aware lifecycle v2, elapsed competing mortality and exploratory/unvalidated status. TRACE remains explicitly `NOT YET EMPIRICALLY RESEARCH-READY`.
- PR #548 demonstrated **AV4-015/#549 (P2)** across the formal standards-facing model descriptions. The frozen ODD retains two pre-v15 statements that background mortality is drawn at the annual M2 transition; ODD+D retains a statement that mortality has declared priority in that annual transition. Frozen source/scientific-model positive controls instead show annual risk partitioned over elapsed M3 intervals, order-invariant competing mortality and year-end fertility/parentage only.
- TRACE rates model description as established for the v0.3.4/v25 baseline and cites ODD/ODD+D, so the stale wording is a current claim-consistency defect even though TRACE's empirical-readiness status remains conservative.
- Closed #315 is related historical context: it repaired the dedicated M2 time contract and called for ODD cross-checking, but did not remove the separate living ODD/ODD+D drift demonstrated on immutable v0.3.4.
- PR #548 was closed unmerged after preserving AV4-015. No v4 finding was repaired.
- **Area M discovery is complete with one new P2 finding, AV4-015/#549. Audit v4 advances to final Area N — cross-system integration.**
