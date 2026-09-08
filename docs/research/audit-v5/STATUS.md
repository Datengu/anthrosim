# AnthroSim scientific audit v5 — status ledger

Audit target: immutable AnthroSim `v0.3.5`, tag commit `e7667af52d48a1ffbae2bf7713a2388e65994b42`, model semantics `anthrosim-model-semantics-v33`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v5/README.md`

This is the repository-authoritative compact ledger. Detailed completed-area reports, discovery evidence and earlier chronological ledger revisions remain preserved in Git history.

## Current state

| Field | Value |
|---|---|
| Audit generation | v5 / fifth independent scientific audit |
| Immutable discovery target | `v0.3.5` |
| Target tag SHA | `e7667af52d48a1ffbae2bf7713a2388e65994b42` |
| Target software version | `0.3.5` |
| Target model semantics | `anthrosim-model-semantics-v33` |
| Discovery coverage | **14/14 Areas A–N complete** |
| Discovery result | **non-clean: 8 findings — 4 P1, 4 P2; no P0/P3** |
| Production `main` reconciled for this update | `2231faab99fe33c59cac96d06a8874023f8db694` |
| Living production model semantics | `anthrosim-model-semantics-v35` |
| Closed Audit-v5 findings | **8/8 — #606, #617, #627, #629, #640, #648, #651, #658** |
| Open Audit-v5 findings | **0/8 — none** |
| Open P1 | **0** |
| Open P2 | **0** |
| Repair state | **post-discovery remediation complete; 8/8 findings closed** |
| Next repair phase | **none — Audit-v5 remediation is complete** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.5` / v33 target remains the discovery baseline even though production repairs advanced living semantics through v35. Audit-v5 closure means the eight software/scientific-method findings received their required production dispositions and evidence chains. It does **not** establish empirical, archaeological or case-specific validity.

## Phase rules after closure

- Do **not** restart Audit-v5 discovery or create duplicate findings for AV5-001 through AV5-008.
- Preserve the immutable `v0.3.5` / v33 discovery evidence as historical controls.
- Treat production repairs and evidence-only re-verification PRs as separate evidence classes.
- Evidence-only re-verification PRs remain unmerged.
- Any future audit generation must establish a new explicit immutable target rather than rewriting Audit-v5 history.
- Do not infer empirical, archaeological or case-specific readiness from Audit-v5 closure.

## Discovery coverage matrix

| ID | Area | Discovery status | Fresh v5 evidence / disposition |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | #605 demonstrated AV5-001/#606 P1. #608 and #610 no new finding. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | #612 documented coupling limitation; #614 no finding; #616 demonstrated AV5-002/#617 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via AV5-002** | #619 and #620 no new finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | #622 and #624 no finding; #623 cadence sensitivity bounded in Area I. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — non-clean** | #626 demonstrated AV5-003/#627 P2; #628 demonstrated AV5-004/#629 P1; AV5-001 also cross-cutting. |
| F | Aggregation and interaction mechanisms | **complete — clean** | #631, #632 and #633 no finding. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — non-clean only via AV5-003** | #635, #636 and #637 no new finding. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — non-clean** | #639 demonstrated AV5-005/#640 P1; #642 matched Wilson control no finding. |
| I | Sensitivity, uncertainty, convergence, robustness | **complete — non-clean via cross-cutting findings** | #644 cadence refinement and #645 analysis-end sensitivity produced no new finding. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — non-clean** | #647 demonstrated AV5-006/#648 P2; #650 demonstrated AV5-007/#651 P1; #653 valid discriminator control no finding. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete — non-clean via cross-cutting findings** | #655 operational relocation/retry preserved identity; no new finding. |
| L | Observability, analysis outputs, statistical summaries | **complete — non-clean** | #657 demonstrated AV5-008/#658 P2; AV5-003, AV5-005, AV5-006 and AV5-007 also cross-cutting. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **complete — non-clean via existing findings** | #660 demonstrated missing disclosure for AV5-002, AV5-006 and AV5-008; no duplicate finding. |
| N | Cross-system integration | **complete — non-clean via AV5-001 through AV5-008** | #662 M2/M4/M9 integration control passed; no ninth finding. |

## Finding register and final remediation state

| Finding | Severity | Discovery evidence | Final state |
|---|---:|---|---|
| **AV5-001 / #606** — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination | P1 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; destination divergence `503/1024` | **closed; repaired by #676 as `f20b7f78103884c6982c86a5d70472153e3a524d`, semantics v34; independently reverified by evidence-only #677** |
| **AV5-002 / #617** — parentage ignores declared close kin and permits first-degree mating | P2 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233` | **closed; explicit null-model scope disposition merged in #669 as `96b4efd07f6c78a5a0c80dfc2c883ee2bd5c908f`** |
| **AV5-003 / #627** — spatial M9 history replay uses process seed instead of population seed | P2 | PR #626, head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100` | **closed; repaired by #671 as `9cf8131240d1e8dce45306773a54b68a3964c64b`** |
| **AV5-004 / #629** — M9 equal-cost destination choice is not spatial-reflection equivariant | P1 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; `256/256` mismatches | **closed; repaired by #679 as `2231faab99fe33c59cac96d06a8874023f8db694`, semantics v35; independently reverified by evidence-only #680 with `0/256` mismatches** |
| **AV5-005 / #640** — small-n normal-CLT mean gate can certify severely under-covered intervals | P1 | PR #639, head `97fa8ee6636b89a7bdb3660583a2484216a9ef5f`, run `34179975096`, job `101916912997`; maximum coverage `0.19` vs nominal `0.95` | **closed; repaired by #664; independently reverified by evidence-only #665** |
| **AV5-006 / #648** — negative held-out discrimination tolerance can label overlapping structural envelopes as discriminating | P2 | PR #647, head `5e57dd95bf268c7bb13bf27d711bd2ad1a414e0f`, run `34183461176`, job `101927040185`; zero-gap overlap accepted at tolerance `-1` | **closed; repaired by #666** |
| **AV5-007 / #651** — binary64 parameter-coordinate collapse can falsely certify a wide compatible region as identified | P1 | PR #650, head `6147ae898f959aec9068f0ae2e3d654a2509afca`, run `34183710162`, job `101927759593`; exact width `0.5`, reported width `0.0` | **closed; repaired by #673 as `b4ac5a46164b3abe7e5d2336858c399f0f7bf94d`; independently reverified by evidence-only #674** |
| **AV5-008 / #658** — survivor-conditioning gate accepts mismatched survival analysis window | P2 | PR #657, head `58f7b6bd5dd7d20330804e6091ce7b3e65c8a4ab`, run `34187243024`, job `101937940280` | **closed; repaired by #667; independently reverified by evidence-only #668** |

## Completed remediation evidence

### AV5-001 / #606 — M9 household-local equal-cost coupling

Production PR **#676** merged as `f20b7f78103884c6982c86a5d70472153e3a524d` and advanced living model semantics from v33 to v34. M9 equal-cost destination coupling now uses versioned household-local demographic equivalence rather than a population-wide ordinal rank. The 1,024-seed locality regression preserves the positive control that the old global rank renumbers while requiring the focal household's authoritative tie key and destination to remain unchanged. Canonical M7.6 was rebound as provenance-only after 144/144 declared runs reproduced its numerical `pointResults`. Evidence-only PR **#677** then independently reverified the original locality attack and was closed unmerged before #606 was closed.

### AV5-002 / #617 — M2 parentage relatedness scope

Production PR **#669** merged as `96b4efd07f6c78a5a0c80dfc2c883ee2bd5c908f`. The accepted disposition explicitly defines M2 parentage as a residence-local eligible-male null model without marriage/incest/relatedness-exclusion semantics, states that first-degree pairings can occur, constrains downstream scientific interpretation, and preserves the complete-genealogy father/daughter construction as executable regression evidence. No model-semantics or checkpoint-schema change was required.

### AV5-003 / #627 — spatial M9 replay population seed

Production PR **#671** merged as `9cf8131240d1e8dce45306773a54b68a3964c64b`. Synthetic-founder realization identity is explicit at the M9 history-replay seam and transformed spatial validation supplies the verified bound population seed. Ordinary non-spatial behavior, declared-founder population-seed inertness, checkpoint schema and model semantics remain unchanged.

### AV5-004 / #629 — spatially equivariant M9 equal-cost destination coupling

Production PR **#679** merged to `main` as `2231faab99fe33c59cac96d06a8874023f8db694` from exact final production head `083ee888577a7cebafc476de4310d5c7c411003d` after all exact-head PR workflows completed successfully.

The repair advances living model semantics from v34 to **v35** and the derived M9 travel-table schema from v4 to v5. Destination policy identity is `m9/equal-cost-destination-local-household-spatial-equivalence-v4`. The keyed destination draw no longer couples to raw origin `CellId` or a canonically ordered physical alternative. It uses the authoritative tie seed, household-local M9 coupling key, trigger index, tied-candidate count and a locally derived spatial-equivalence ordering. The canonical spatial frame is constructed over the supported grid reflection group and includes focal-region membership plus authoritative movement cost and declared transformed spatial fields. Exact automorphic alternatives remain one scientific equivalence class; canonical IDs may serialize indistinguishable members but do not create a directional scientific winner.

Permanent regression coverage includes the original 3x1 horizontal-reflection failure class across 256 seeds, analogous vertical reflection, exact-symmetry marginal exchangeability, asymmetric reflected focal regions, AV5-001 remote-founder locality and AV4-007 HouseholdId-label invariance. Route cost, reachability, equal-minimum construction, travel duration, M4 decisions, mortality, resource allocation and sequential RNG streams were not changed.

The v35 semantics transition also reconciled living scientific-reference provenance. Canonical M7.6 does not exercise M9; exact-head production CI reran its 18 points / 144 declared simulations under v35 and reproduced the preserved numerical `pointResults`. M8.6 and M9.7 scientific/security gates passed, including replay, active checkpoint/resume, strict preserved-reference and tamper-rejection checks.

Because AV5-004 is P1, GitHub's automatic closure of #629 at production merge was reversed until independent evidence completed. Evidence-only PR **#680** started exactly from production merge `2231faab99fe33c59cac96d06a8874023f8db694`.

The original #628 discovery adversary was first restored byte-for-byte at evidence head `56e1247e0dbcb5bd35c8a341f07e0fb1943e41ea`; dedicated run `34286067298`, job `102261842555`, passed. Current rustfmt required three mechanical layout-only hunks, so final evidence head `aa3c9fccd404be6c63388e9da05acefc415f4cad` retained identical scientific logic and reran the same adversary:

- workflow: `Audit v5 Area E M9 spatial reflection adversary`;
- run `34286393659`;
- job `102262880816`;
- conclusion: **success**;
- result: `M9 reflection mismatches=0/256; first=[]`;
- the same test continues to require equality of the authoritative household destination-coupling key between canonical and reflected arms.

The immutable discovery result was `256/256` reflection mismatches; merged v35 therefore changes that exact controlled oracle to `0/256`. All other workflows on final evidence head also completed successfully, including Applicable scientific/security gates run `34286393764` and central CI run `34286393532` with full workspace tests, release build, core benchmarks, Canonical M7.6, 1000-run soak, M5/M6 integration and performance/memory acceptance. The evidence was recorded on #629, PR #680 was closed **unmerged**, and #629 was closed as completed only after that independent P1 evidence chain was complete.

### AV5-005 / #640 — small-n mean stopping validity

Production PR **#664** added executable validity guards for normal-CLT mean-family stopping, including a minimum replicate floor and fail-closed zero-observed-variance handling. Evidence-only PR **#665** independently restored the original adversary against merged `main` and confirmed the repaired behavior before closure.

### AV5-006 / #648 — discrimination threshold domain

Production PR **#666** rejects negative `corroborationDiscriminationTolerance` before discrimination calculations while preserving zero/positive thresholds, conservative structural envelopes and the calibration/held-out firewall.

### AV5-007 / #651 — exact parameter-coordinate fidelity

Production PR **#673** merged as `b4ac5a46164b3abe7e5d2336858c399f0f7bf94d`. All-integer research parameter coordinates remain exact through explored/compatible range arithmetic; normalized width uses exact rational span arithmetic; ordinary floating coordinates remain explicitly binary64-approximate; unsafe mixed integer/float domains fail closed. Evidence-only PR **#674**, exact evidence head `2d2e3642a73017fa1779fc488984a2738c572117`, re-ran the large-integer adversary in run `34251741737`, job `102147571292`, and confirmed exact width `0.5`, three explored levels and `identified=false` before closing unmerged.

### AV5-008 / #658 — survivor-window alignment

Production PR **#667** requires survivor-conditioned condition observables and their survival/population observables to share the same `analysisWindowId`. Evidence-only PR **#668** independently restored the original mismatch adversary against merged `main` and confirmed the repaired behavior before closure.

## Remediation completion

Audit-v5 discovery and post-discovery remediation are both complete:

- discovery coverage: **14/14 Areas A–N**;
- authoritative findings: **8 total**;
- P1: **4/4 closed with independent post-merge adversarial re-verification**;
- P2: **4/4 closed with required production disposition**;
- open Audit-v5 findings: **0**;
- living production model semantics after the final repair: **`anthrosim-model-semantics-v35`**.

No additional Audit-v5 repair branch is authorized by this ledger because there is no remaining Audit-v5 finding. Future scientific audit work must use a separately declared audit generation and immutable target if another independent audit is undertaken.

## Next action

Preserve the completed Audit-v5 evidence chain and continue ordinary repository development from live `main`. Do not reopen Audit-v5 discovery merely because production continues to evolve. If a later independent audit is commissioned, define a new immutable software/model-semantics target and a new audit generation before discovery begins.

Audit-v5 closure remains a statement about framework/software scientific-method defects identified by this audit. It is **not** a claim that AnthroSim is empirically calibrated, archaeologically validated, or ready to answer a specific case-study question without the separate evidence, parameterization, validation and uncertainty work required for that study.
