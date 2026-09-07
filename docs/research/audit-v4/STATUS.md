# AnthroSim scientific audit v4 — status ledger

Audit target: immutable AnthroSim `v0.3.4`, tag commit `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`, model semantics `anthrosim-model-semantics-v25`.

Protocol: `docs/research/scientific-audit-protocol.md`

Re-verification addendum: `docs/research/audit-reverification-version-drift.md`

Charter: `docs/research/audit-v4/README.md`

Purpose: durable repository-authoritative state for the fourth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v3 convergence pass.

> This ledger is intentionally compact current-state documentation. Earlier expanded per-session narratives, exact discovery constructions, production changes and evidence classifications remain preserved in Git history and the referenced issues/PRs.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v4 / fourth independent scientific audit |
| Immutable discovery target | `v0.3.4` |
| Target tag SHA | `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` |
| Target software version | `0.3.4` |
| Target model semantics | `anthrosim-model-semantics-v25` |
| Coverage state | **14/14 Areas complete — Areas A-N complete; discovery complete** |
| Frozen-target findings | **0 P0; 13 P1; 2 P2; 0 P3** |
| Current open Audit-v4 findings | **none** |
| Verified remediated findings | **15/15 — AV4-001 through AV4-015** |
| Living remediation line at Audit-v4 completion | `anthrosim-model-semantics-v33`; checkpoint schema `21` |
| Convergence classification | **non-clean historical convergence pass: frozen v0.3.4/v25 discovery demonstrated 13 P1 and 2 P2 findings** |
| Repair state | **post-discovery remediation complete; every Audit-v4 finding is closed after production repair and required independent current/merged-main re-verification** |
| Empirical readiness implication | **none — Audit v4 does not establish empirical validity or archaeological research readiness for a specific case** |

The historical convergence classification remains non-clean even though all findings are repaired: it describes what the independent discovery pass found on the immutable v0.3.4/v25 target. The living repository is not rolled back to that target after remediation.

## Discovery and remediation rules

- The immutable `v0.3.4` tag remains the scientific discovery target and historical evidence identity.
- Audit v2/v3 evidence is historical context and regression-hypothesis material only; Audit v4 coverage was established independently.
- Production repair was deferred until A-N discovery completed.
- Every demonstrated finding was preserved in its original issue before remediation.
- Production repairs used dedicated PRs and required the applicable exact-head protected/scientific gates before merge.
- Evidence-only re-verification PRs are closed unmerged after classification.
- Historical adversaries remain preserved in Git history. When a frozen-target harness assumption becomes intrinsically obsolete on the living remediation line, current-state re-verification follows `docs/research/audit-reverification-version-drift.md`: only the minimum harness/positive-control adaptation is permitted, every adaptation is enumerated, and the substantive scientific oracle may not be weakened.
- Living ODD, ODD+D, scientific-model, provenance and other current-facing surfaces describe the current repaired repository; historical v25 language remains only where it is explicitly a frozen Audit-v4/v0.3.4 identity.

## Coverage matrix

| ID | Audit area | Final discovery status | Finding relationship |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete** | AV4-001 through AV4-007 |
| B | Demography, fertility, mortality, ageing, population structure | **complete** | AV4-001/002/004/005/006 cross-cutting; no additional B-specific finding |
| C | Households, kinship, social links, lifecycle structure | **complete** | AV4-003/005/007 cross-cutting; no additional C-specific finding |
| D | Resources, condition, subsistence, depletion/recovery | **complete** | AV4-006/008 cross-cutting |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete** | AV4-001/002/003/004/006/007/009 cross-cutting |
| F | Aggregation and interaction mechanisms | **complete** | no additional finding demonstrated |
| G | Initialization, burn-in, path dependence, continuation state | **complete** | no additional finding demonstrated |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete** | AV4-001 through AV4-010 cross-cutting; AV4-010 new P1 |
| I | Sensitivity, uncertainty, convergence, robustness | **complete** | AV4-010 cross-cutting; no additional finding demonstrated |
| J | Identifiability, equifinality, calibration, discrimination | **complete** | AV4-011 P1; AV4-010/013 cross-cutting |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete** | AV4-012/013 P1; AV4-010/011 cross-cutting |
| L | Observability, analysis outputs, statistical summaries | **complete** | AV4-014 P2; AV4-010/012/013 cross-cutting |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **complete** | AV4-015 P2; AV4-014 cross-cutting |
| N | Cross-system integration | **complete** | AV4-001 through AV4-015 integrated; no additional independent defect demonstrated |

## Finding register

All discovery findings below refer to the immutable v0.3.4/v25 target. `verified repaired` refers to the living remediation line and does not erase the historical finding.

| Finding | Severity | Area | Issue | Final status |
|---|---|---|---|---|
| AV4-001 — fertility RNG assignment sensitive to arbitrary founder person labels | P1 | A; B/E/H/N | #486 | **verified repaired; closed** |
| AV4-002 — background-mortality RNG assignment sensitive to arbitrary founder person labels | P1 | A; B/E/H/N | #488 | **verified repaired; closed** |
| AV4-003 — migration RNG assignment sensitive to arbitrary household labels | P1 | A; C/E/H/N | #491 | **verified repaired; closed** |
| AV4-004 — newborn-sex RNG assignment sensitive to arbitrary founder person labels | P1 | A; B/E/H/N | #493 | **verified repaired; closed** |
| AV4-005 — parentage RNG assignment sensitive to arbitrary male person labels | P1 | A; B/C/H/N | #495 | **verified repaired; closed** |
| AV4-006 — condition-mediated mortality RNG assignment sensitive to arbitrary founder person labels | P1 | A; B/D/E/H/N | #497 | **verified repaired; closed** |
| AV4-007 — M9 equal-cost destination key sensitive to arbitrary household labels | P1 | A; C/E/H/N | #500 | **verified repaired; closed** |
| AV4-008 — scarce-resource remainder assignment sensitive to arbitrary household labels | P1 | D; C/H/N | #514 | **verified repaired; closed** |
| AV4-009 — M4 destination selection sensitive to reflected/candidate ordering | P1 | E; H/N | #518 | **verified repaired; closed** |
| AV4-010 — binary64 conversion can collapse large-integer uncertainty and falsely satisfy stopping | P1 | H; I/J/K/L/N | #528 | **verified repaired; closed** |
| AV4-011 — identifiability certification can accept a fabricated downstream parameter absent from executed design | P1 | J; K/N | #535 | **verified repaired; closed** |
| AV4-012 — downstream study-result provenance can accept changed `researchId` with stale result identity | P1 | K; L/N | #539 | **verified repaired; closed** |
| AV4-013 — study finalization can bind tampered canonical analysis rows contradicting immutable executed design | P1 | K; J/L/N | #543 | **verified repaired; closed** |
| AV4-014 — survivor-conditioning safeguard recognized fabricated mortality/survival source labels by substring | P2 | L; M/N | #546 | **verified repaired; closed** |
| AV4-015 — living ODD/ODD+D retained superseded annual-boundary mortality semantics | P2 | M; B/N | #549 | **verified repaired; closed** |

### Final three remediation chains

The complete exact evidence histories remain on the issues; these last three entries are repeated here because they were the final stale ledger rows at Audit-v4 closure.

**AV4-013 / #543**

- production PR #589 exact head `8275fd40dd1a95387d84271097706097699491a8`, merged as `78dd0930936d1e7e18064e7314ce2e07d7608c66`;
- evidence PR #591 exact head `db6a53b1a80d4b266269bc7ff1feabaa02e4b19c`;
- dedicated evidence run/job `33986578732` / `101361134203`: success;
- evidence PR #591 closed unmerged; #543 closed after verification.

**AV4-014 / #546**

- primary production PR #592 exact head `a8081006e4462b0637a639afdd4f68ceba6e8abe`, merged as `70fc008290b55dae8846740387fa343194228c73`;
- diagnostic-compatibility production PR #594 exact final head `509da56de275500004e1204edebc7b04bbb71b24`, merged as `4d1457b165867ef7c0327fb1ff3a5095de2d9693`;
- evidence PR #595 exact head `84d1fbc07f98f16bd30c73dc18c48c16a2af04f3`, original #545 adversary restored byte-for-byte;
- dedicated evidence run `34020753555`: success;
- evidence PR #595 closed unmerged; #546 closed after verification.

**AV4-015 / #549**

- production PR #596 exact head `b4d557934ca5f2d2ae840c2525e47f5a5a65eb22`, merged as `0588816f1bd23101d6f0e6d5c9c61c8b3bf6f685`;
- exact historical replay PR #597 head `34866a6622772dfd5714dbbaf7c40f3535036f20` was closed unmerged after showing that the discovery-era `current model semantics v25` positive control was no longer forward-compatible with living v33 documentation and blocked the substantive mortality oracle;
- protocol clarification PR #598 exact head `b3126bd8e9c4e262fdbabaed187a0b75c33237d1`, merged as `bd92c6b3708cb5fb06eb9446c5f5a0a4b1b5e646` after a fully green exact-head matrix;
- current-state evidence PR #599 exact head `9e62cda1f091aaef06ae2799730fd347abaf8eba`, based directly on protocol-merged `main`;
- #599 changed only the obsolete frozen/current version positive controls and related diagnostics while preserving the original substantive mortality positive controls, the exact three stale-claim strings, and the terminal scientific oracle;
- dedicated evidence run/job `34134061659` / `101780748744`: success;
- central CI run `34134061605` plus applicable scientific/security, provenance, observability, preprocessing, bundle, resumed-Explorer and determinism workflows: success;
- evidence PR #599 closed unmerged; #549 closed as completed.

## Audit-v4 conclusion

Scientific Audit v4 discovery and post-discovery remediation are complete.

The immutable v0.3.4/v25 discovery result remains a **non-clean convergence pass** because it independently demonstrated 13 P1 and 2 P2 scientific defects. All 15 findings have subsequently been repaired and independently re-verified/dispositioned on the living remediation line under the documented protocol.

This completion state means the Audit-v4 finding backlog is closed. It does **not** mean AnthroSim is proven correct, empirically validated, or automatically ready for a specific archaeological inference. A future independent audit, empirical validation programme, or case-specific research phase must be started explicitly rather than inferred from Audit-v4 closure.
