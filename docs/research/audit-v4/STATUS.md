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
| Coverage state | **11/14 Areas complete — Areas A-K complete; Area L next** |
| Current P0 findings | none discovered |
| Current P1 findings | **12 open — AV4-001/#486; AV4-002/#488; AV4-003/#491; AV4-004/#493; AV4-005/#495; AV4-006/#497; AV4-007/#500; AV4-008/#514; AV4-009/#518; AV4-010/#528; AV4-011/#535; AV4-012/#539** |
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
| A | Authoritative semantics and scheduler behaviour | **complete — AV4-001 through AV4-007** | PR #483 / run `33686718180` exhaustively checked **133,225** M3/M4 schedule combinations. Fresh adversaries then demonstrated arbitrary-label/order coupling in fertility, background mortality, M4 migration, newborn sex, parentage, condition-mediated mortality and M9 equal-cost destination selection: **AV4-001/#486 through AV4-007/#500**. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — AV4-001/002/004/005/006 cross-cutting** | PRs #501/#503/#505/#506/#507 covered mortality partitioning, fertility/birth-spacing edges, mortality-age edges, 200-seed extinction/censoring and mate-age/presence boundaries. No additional B-specific defect. |
| C | Households, kinship, social links, lifecycle structure | **complete — AV4-003/005/007 cross-cutting** | PR #509 passed the exact independent-age fission threshold; PR #510 quantified temporary-presence/lifecycle phase dependence. No additional C-specific defect. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — AV4-006/008 cross-cutting** | PR #512 passed 1/4/12/365 full-scarcity partition equivalence. PR #513 / run `33699132098`, job `100474385648` demonstrated **AV4-008/#514**: one-unit scarce-resource remainder assignment changes fixed-person condition under pure HouseholdId rotation. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — AV4-001/002/003/004/006/007/009 cross-cutting** | PR #516 passed 48 radius/buffer cases; PR #517 / run `33706244290`, job `100495847910` demonstrated **AV4-009/#518** M4 reflection/candidate-order dependence; PR #519 passed **2,240** M9 route-reflection comparisons; PR #520 passed **168** temporary-mobility/resource boundary cases. |
| F | Aggregation and interaction mechanisms | **complete — no additional finding demonstrated** | PR #522 exact-head CI passed; two one-person households aggregated for five days produced exactly 10 visitor person-days, 720 home person-days and 730 total person-days with expected resource pressure. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — no additional finding demonstrated** | PR #524 passed exact active-state continuation across **32 process seeds**. PR #525 / run `33740052632`, job `100599600258` quantified initial-stock path dependence: 743 vs 7430 after one year, exact 7430 vs 7430 convergence by the controlled ten-year horizon. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — AV4-001 through AV4-010 cross-cutting; new AV4-010 P1** | PR #527 / run `33740457897`, job `100600897791` demonstrated **AV4-010/#528**: binary64 conversion collapses exact integers around `2^53`, turning exact half-width `0.565792867038` into 0 and false `sufficient_stop`. PR #529 passed **136** safe-range estimator/seed-contract attacks. |
| I | Sensitivity, uncertainty, convergence, robustness | **complete — AV4-010 cross-cutting; no additional finding demonstrated** | PR #531 / run `33741469209`, job `100604140981` passed **48** transformed expansions / **576** coordinate checks, retaining exactly 12 executable treatments under all dimension/value declaration orders; ancestor/descendant overlaps failed closed. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — new AV4-011 P1; AV4-010 cross-cutting** | PR #534 / run `33743566077`, job `100610814705` demonstrated **AV4-011/#535**: an invented downstream parameter `fabricated_theta` absent from the preserved executed design can be given range `[0,1]`, compatible `[0,0]`, `identified=true`, and top-level `declared_claim_identified`. PR #536 passed **122** clean decision-matrix assertions across repaired identifiability boundaries. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete — new AV4-012 P1; AV4-010/011 cross-cutting** | PR #538 / head `fb852797b22dc0913ba0b543e44a345c26074187`, run `33744613505`, job `100614126094` demonstrated **AV4-012/#539**: downstream analysis provenance capture+verify accepts a tampered `study-result-binding.json` whose identity-covered `researchId` changed while stale `resultIdentity` remained. PR #540 / head `842e7387ba5bdbc5c2e6129d397d5bd7959642c2`, run `33745069053`, job `100615547018` passed **11** fresh end-to-end `anthrosim-research` retry/tamper scenarios: exact retry and one-copy recovery controls succeeded, while contradictory immutable metadata, changed definitions, source identity, checkpoint digest, event/metric schemas, world dimensions and completion seed all failed closed. Central CI's only #540 failure was disposable-fixture rustfmt; dedicated scientific execution passed. |
| L | Observability, analysis outputs, statistical summaries | **incomplete — AV4-010/012 cross-cutting** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete — AV4-001 through AV4-012 cross-cutting** | — |

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
| AV4-012 — downstream analysis provenance accepts a study-result binding with stale self-identity | **P1** | K; J/L/N | **demonstrated; open; deliberately unrepaired** | #539 | PR #538 / run `33744613505`, job `100614126094`: tampered `researchId` with stale binding `resultIdentity` was captured and verified as valid provenance. |

## Current discovery handoff

### Area K closure — 2026-09-03

- Frozen v0.3.4 source inspection covered `anthrosim-research` root creation/retry/reconciliation, completed child-bundle validation, exact source identity, checkpoint/event/metric/world validation, `anthrosim-study` result-binding production, and downstream analysis provenance capture/verify/replay.
- **AV4-012/#539 (P1)** is new and distinct: downstream provenance validates/fingerprints the exact binding bytes but does not recompute the producer-defined `StudyResultBinding.resultIdentity`, allowing a stale self-identity after an identity-covered field edit.
- PR #540 independently passed an 11-scenario end-to-end retry/tamper matrix, showing the authoritative research execution/root-reuse layer itself fails closed across contradictory immutable metadata and multiple completed-bundle tamper seams.
- AV4-010/#528 and AV4-011/#535 remain explicit cross-cutting K limitations for numeric inference fidelity and semantic binding of downstream identifiability coordinates.
- All Area-K evidence PRs are closed unmerged after classification. No v4 finding was repaired.
- **Area K discovery is complete with one new P1 finding, AV4-012/#539. Audit v4 advances to Area L — observability, analysis outputs, statistical summaries.**
