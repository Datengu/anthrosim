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
| Live `main` reconciled for this update | `9cf8131240d1e8dce45306773a54b68a3964c64b` |
| Closed Audit-v5 findings | **5/8 — #617, #627, #640, #648, #658** |
| Open Audit-v5 findings | **3/8 — #606, #629, #651** |
| Open P1 | **3 — AV5-001/#606; AV5-004/#629; AV5-007/#651** |
| Open P2 | **0** |
| Repair state | **post-discovery remediation in progress; 5/8 findings closed** |
| Next selected repair | **AV5-007/#651 — exact parameter-coordinate fidelity in identifiability/equifinality diagnostics** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.5` / v33 target remains the discovery baseline even as production fixes advance `main`. A finding being closed means its required production disposition has been completed; the ledger separately records independent post-merge re-verification where it was required or performed.

## Phase rules

- Do **not** restart Audit-v5 discovery or create duplicate findings for AV5-001 through AV5-008.
- Repair current `main`, while preserving the immutable discovery target and original evidence as historical controls.
- Use one dedicated production repair PR per authoritative finding.
- Require exact-head protected/scientific CI before merge.
- P0/P1 repairs require independent post-merge adversarial re-verification before final closure. Apply re-verification to P2 findings when required by their acceptance contract or when it materially strengthens the evidence chain.
- If an issue auto-closes before required re-verification is complete, reopen it until the evidence chain is complete.
- Do not infer empirical, archaeological or case-specific readiness from framework/software audit closure.

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

## Finding register and live remediation state

| Finding | Severity | Discovery evidence | Live state |
|---|---:|---|---|
| **AV5-001 / #606** — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination | P1 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; destination divergence `503/1024` | **open; unrepaired** |
| **AV5-002 / #617** — parentage ignores declared close kin and permits first-degree mating | P2 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233` | **closed; explicit null-model scope disposition merged in #669 as `96b4efd07f6c78a5a0c80dfc2c883ee2bd5c908f`** |
| **AV5-003 / #627** — spatial M9 history replay uses process seed instead of population seed | P2 | PR #626, head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100` | **closed; repaired by #671 and merged as `9cf8131240d1e8dce45306773a54b68a3964c64b`** |
| **AV5-004 / #629** — M9 equal-cost destination choice is not spatial-reflection equivariant | P1 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; `256/256` mismatches | **open; unrepaired** |
| **AV5-005 / #640** — small-n normal-CLT mean gate can certify severely under-covered intervals | P1 | PR #639, head `97fa8ee6636b89a7bdb3660583a2484216a9ef5f`, run `34179975096`, job `101916912997`; maximum coverage `0.19` vs nominal `0.95` | **closed; repaired by #664 and independently reverified by evidence-only #665** |
| **AV5-006 / #648** — negative held-out discrimination tolerance can label overlapping structural envelopes as discriminating | P2 | PR #647, head `5e57dd95bf268c7bb13bf27d711bd2ad1a414e0f`, run `34183461176`, job `101927040185`; zero-gap overlap accepted at tolerance `-1` | **closed; repaired by #666** |
| **AV5-007 / #651** — binary64 parameter-coordinate collapse can falsely certify a wide compatible region as identified | P1 | PR #650, head `6147ae898f959aec9068f0ae2e3d654a2509afca`, run `34183710162`, job `101927759593`; exact width `0.5`, reported width `0.0` | **open; next selected repair** |
| **AV5-008 / #658** — survivor-conditioning gate accepts mismatched survival analysis window | P2 | PR #657, head `58f7b6bd5dd7d20330804e6091ce7b3e65c8a4ab`, run `34187243024`, job `101937940280` | **closed; repaired by #667 and independently reverified by evidence-only #668** |

## Completed remediation evidence

### AV5-002 / #617 — M2 parentage relatedness scope

Production PR **#669** was merged to `main` as `96b4efd07f6c78a5a0c80dfc2c883ee2bd5c908f` after exact-head protected/scientific CI passed. The accepted disposition deliberately does **not** invent a new genealogy-aware mate-choice mechanism. Instead it:

- defines M2 parentage as a residence-local eligible-male null model with no marriage/incest/relatedness-exclusion semantics;
- states explicitly that first-degree pairings can be generated;
- constrains scientific interpretation of generated genealogy and downstream kin-driven patterns;
- preserves the complete-genealogy father/daughter construction as a permanent executable regression.

No causal model-semantics or checkpoint-schema change was required.

### AV5-003 / #627 — spatial M9 replay population seed

Production PR **#671** was merged to `main` as `9cf8131240d1e8dce45306773a54b68a3964c64b` after every PR-triggered workflow on exact repair head `f2761bb5353ecba4bbb8b0f49a7062c037efe8dc` completed successfully. The repair:

- makes the synthetic-founder realization seed explicit at the M9 temporary-history replay seam;
- carries that identity through the shared core invariant validator without rewriting the checkpoint process seed;
- makes transformed spatial validation supply the verified bound `populationSeed`;
- preserves ordinary non-spatial synthetic behavior and declared-founder population-seed inertness;
- adds permanent regressions for fixed environment/population with varying process seeds, population-seed variation, declared-founder inertness, and explicit-split M9 checkpoint/resume equivalence.

Protected CI, cross-platform/spatial determinism checks, the M8.6 terrain benchmark and the M9.7 aggregation/replay benchmark all passed. No checkpoint schema, core model-semantics identity, spatial-transform-semantics identity or authoritative execution/RNG trajectory change was required. Because AV5-003 is P2 and its acceptance contract did not require an independent post-merge adversarial evidence PR, the exact-head production regression and protected/scientific gate evidence complete its required disposition.

### AV5-005 / #640 — small-n mean stopping validity

Production PR **#664** added executable validity guards for normal-CLT mean-family stopping, including a minimum replicate floor and fail-closed zero-observed-variance handling. Because AV5-005 is P1, evidence-only PR **#665** independently restored the original adversary against merged `main` and confirmed the repaired behavior before closure.

### AV5-006 / #648 — discrimination threshold domain

Production PR **#666** rejects negative `corroborationDiscriminationTolerance` before the discrimination calculation while preserving zero/positive thresholds, conservative structural envelopes and the calibration/held-out firewall. The issue is closed.

### AV5-008 / #658 — survivor-window alignment

Production PR **#667** requires survivor-conditioned condition observables and their accompanying survival/population observables to share the same `analysisWindowId`. Evidence-only PR **#668** independently restored the original mismatch adversary against merged `main` and confirmed the repaired behavior before closure.

## Remaining remediation set

1. **AV5-007 / #651 (P1) — next selected repair.** Preserve sufficient exact numeric fidelity for parameter coordinates used by identifiability/equifinality diagnostics and research gates. The original adversary uses exact integer levels above binary64's safe-integer boundary, where distinct coordinates collapse after conversion and a true compatible width of `0.5` can be reported as `0.0`. Repair must either retain exact/sufficient coordinate identity through the analysis path or fail closed when the representation cannot support the claim, while preserving ordinary floating-point designs and executed-design binding. As a P1, it requires independent post-merge adversarial re-verification before final closure.
2. **AV5-001 / #606 (P1).** Remove nonlocal M9 tie dependence on globally renumbered stochastic-coupling ranks while preserving deterministic and label-invariant coupling.
3. **AV5-004 / #629 (P1).** Make M9 equal-cost destination selection spatially equivariant under reflection/isomorphism without collapsing to an arbitrary deterministic canonical winner.

AV5-001 and AV5-004 both touch M9 stochastic destination identity and may require coordinated design review; do not repair them concurrently on overlapping branches unless their shared keying contract is explicitly reconciled. AV5-007 is selected first because it is an independent analysis-layer repair and does not create overlap with that shared M9 design work.

## Next action

Begin dedicated production remediation for **AV5-007/#651** from the current live `main` after this ledger synchronization is merged. Preserve the original #650 evidence as the historical adversary, add permanent exact-coordinate/fail-closed regression coverage, run exact-head protected/scientific CI, merge only after all required checks pass, and then perform an independent post-merge adversarial re-verification before final closure because AV5-007 is P1.

Audit-v5 discovery remains complete. No further discovery pass is implied by remediation, and no empirical or archaeological readiness claim follows from closing these software/scientific-method findings.
