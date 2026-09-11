# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed discovery evidence remains in the per-area reports, finding issues and referenced evidence-only PRs.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **14/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass: 14 findings — 7 P1, 7 P2** |
| Open Audit-v6 findings | **5** |
| Open P0/P1 | **0** |
| Phase | **discovery complete — controlled P2 remediation in progress** |
| Verified P1 remediations | **7/7 — AV6-013/#741; AV6-012/#737; AV6-010/#729; AV6-009/#726; AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Active ownership | **AV6-005/#708 is the active P2 remediation on `audit/repair-708-av6-005`; AV6-002/#694 and AV6-003/#699 are merged/closed via #771/#772** |
| Convergence status | **v6 discovery remains non-clean because seven P1 findings were demonstrated; all seven P1s are now repaired, independently reverified and closed, with five P2 findings still open** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

Protected `main` after merged AV6-003/#699 is `8f69fd951ae62773fa2a1f228efa00d17e9b6e53`, with living model semantics `anthrosim-model-semantics-v40`. Active AV6-005/#708 advances this repair branch to `anthrosim-model-semantics-v41`; immutable Audit-v6 discovery remains attributed to `v0.3.6` / v35.

## Completed Areas

| Area | Discovery status | Fresh v6 evidence |
|---|---|---|
| A — authoritative semantics and scheduler behaviour | **complete — AV6-001/#687 P1; repaired/reverified** | #684 clean scheduler-equivalence control; #686/#687 same-day M4-before-newly-due-M9 inversion. `area-a-2026-09-09.md` |
| B — demography, fertility, mortality, ageing and population structure | **complete — AV6-002/#694 P2** | #692 clean chronology/cadence; #693/#694 inconsistent male-parent age-window references. `area-b-2026-09-09.md` |
| C — households, kinship, social links and lifecycle structure | **complete — AV6-003/#699 P2** | #698/#699 external-kin context collapse; #701 clean relabelling control. `area-c-2026-09-09.md` |
| D — resources, condition, subsistence, depletion/recovery | **complete — AV6-004/#707 P1 repaired/reverified; AV6-005/#708 P2** | scarce-resource reflection and duration-split household-index defects. `area-d-2026-09-09.md` |
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1 repaired/reverified** | unreachable impassable padding changes tied M9 destination; locality controls clean. `area-e-2026-09-09.md` |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | touching half-open visits inflate `peakVisitors`; lifecycle control clean. `area-f-2026-09-09.md` |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | incomplete founder genealogy consumed as absence; reproductive-history resume control clean. `area-g-2026-09-09.md` |
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1; both repaired/reverified** | sequential Wilson undercoverage; confirmatory sample values not bound to authoritative outputs. `area-h-2026-09-09.md` |
| I — sensitivity, uncertainty, convergence and robustness | **complete — AV6-011/#733 P2** | no-op long-run sensitivity declarations can satisfy coverage; spatial extent late-divergence control clean. `area-i-2026-09-09.md` |
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1; repaired/reverified** | deterministic claim-driving outputs not bound to authoritative executions; equifinality control clean. `area-j-2026-09-09.md` |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1; repaired/reverified** | finalized-study root verifier accepts semantically forged canonical rows; cross-root transplant control clean. `area-k-2026-09-09.md` |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | drifting initialization families collapse to identical status distributions; exposure/extinction denominator control clean. `area-l-2026-09-09.md` |
| M — documentation, TRACE/ODD/ODD+D and claim consistency | **complete — non-clean via existing findings; no new finding** | #747 claim-consistency adversary; TRACE empirical boundary remains conservative. `area-m-2026-09-09.md` |
| N — cross-system integration | **complete — non-clean via existing v6 findings; no new root finding** | #749 quantifies AV6-006 propagation from M9 destination into visitor-person-days and resource units. `area-n-2026-09-09.md` |

## Verified P1 remediation progress

### AV6-006 / #711 — repaired and independently reverified

Production repair PR **#768**, final repair head `8e9a1e88eb002f739377e8242f0243c6096a5bc2`, merged to protected `main` as `e729866f4a46d833a566a557e38570daf854d5ae`. The accepted v38 M9 contract localizes equal-cost destination coupling to the origin's reachable traversable connected component, canonicalized in component-local coordinates, so unreachable/impassable domain padding is excluded from the tie frame while genuine route/reachability changes remain causal.

Mandatory post-merge evidence used evidence-only PR **#769**, exact evidence head `a4d15b61664e8a35006dbb0b8942a8cfe0071963`, based directly on the production merge. The original #710 128-seed impassable-padding adversary was restored byte-for-byte unchanged and passed. Exact-head central CI **34560791773** passed, including `Quality and tests` job **103143004031**; applicable scientific/security gates **34560791926** also passed with determinism, provenance, spatial observability, bundle, resume, landscape and spatial-mechanism workflows. #769 was closed unmerged as required, and #711 was closed completed only after independent evidence was green.

### AV6-004 / #707 — repaired and independently reverified

Production repair PR **#765**, final repair head `cdc9a4d82e58fabfb061123db134d26113c7bca3`, merged to protected `main` as `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` after the complete exact-head protected/scientific matrix passed. The accepted v37 M3 contract removes canonical resource-cell index from equal-remainder tie phase while retaining persistent scientific household coupling identity and period-sequence fairness. The repair also adds permanent horizontal and vertical reflection regressions and preserves prior HouseholdId-relabelling and repeated-period fairness controls.

Because this authoritative allocation change legitimately altered some downstream trajectories, the production repair did not paper over evidence drift. A fresh v37 780-run demographic confirmation was guarded and refreshed, and a separate fail-closed comparison established the exact M7.6 numerical delta before updating its reference. Final production CI **34552939008** then reproduced the refreshed 18-point / 144-run canonical M7.6 reference exactly; the fresh 780-run demographic confirmation and M8.6/M9.7 scientific gates also passed.

Mandatory post-merge evidence used evidence-only PR **#766**, final evidence head `2f65f1915bba2d9ad45fc06b6bd1347230c740da`, based directly on the production merge. The original #705 Rust adversary was restored byte-for-byte unchanged from historical evidence head `f8ee562027ae162afb9ce7a5f00a6ca3382b0f33`; no version-drift adaptation of the scientific oracle was required. Dedicated run **34553739865**, job **103121848831**, proved production ancestry, byte identity and living v37 semantics, then passed the unchanged horizontal-reflection oracle. Ordinary exact-head central CI **34553739838**, demographic confirmation **34553739983**, applicable scientific/security gates **34553740059**, determinism, provenance, bundle, resume and spatial workflows all passed. #766 was closed unmerged only after the complete evidence matrix was green; #707 may therefore be closed completed as independently reverified.

### AV6-001 / #687 — repaired and independently reverified

Production repair PR **#762**, final repair head `2f07a8b4e88f26857823bd0522a082563f8affbc`, merged to protected `main` as `5c9cc73a26b4e400fbbc90e89ef9c0b4194371ca` after the complete exact-head protected/scientific matrix passed. The accepted v36 scheduler contract prevents between-boundary M9 scans from re-entering an already completed positive fixed day after M4, while preserving valid future target-arrival reconsideration and day-zero behaviour.

Mandatory post-merge evidence used evidence-only PR **#763**, final evidence head `59661e7c10cf73f37ce4744de3fd615019cc2330`, based directly on the merged repair. The derivative preserved the #686 controlled landscape, founder state, day-91 M4 relocation and target-arrival geometry, adapting only the obsolete defect-presence oracle under the version-drift protocol. Dedicated run **34526823928** passed, and every ordinary exact-head workflow on the same evidence head passed, including central CI **34526830471** and applicable scientific/security gates **34526831137**.

The preserved boundary arm now records the day-91 M4 migration without any retroactive same-day M9 departure and reports the target as `DepartureWindowMissed` on day 100. A neighbouring positive control changing only travel capacity preserves a legitimate future M9 departure on day 92 with arrival on day 100. The dedicated derivative tests passed 2/2 and the permanent repair regressions passed 6/6. #763 was closed unmerged and #687 was closed completed only after that independent P1 evidence was green.

### AV6-013 / #741 — repaired and independently reverified

Production repair PR **#751** merged to protected `main` as `a19fdaf98dc1d5e0bba1ed131347a7803bf8d856` after complete exact-head protected/scientific validation. Evidence-only PR **#752**, exact head `801f1d9436ff8eda5a71a08945be4e5fc022d849`, independently re-ran the preserved adversarial contract; its dedicated, central and applicable scientific/security checks passed. #752 was closed unmerged and #741 closed only after re-verification.

### AV6-012 / #737 — repaired and independently reverified

Production repair PR **#754**, exact repair head `7fe94b811a84a62cd9f54487f7f50204dcb848b6`, merged as `6c14decab9ce485e74bb83b58d2c0a2a2637d53a`. The repair binds claim-driving deterministic AnthroSim-derived identifiability outputs to authoritative executions before compatibility/identification calculation. Evidence-only PR **#755**, exact head `a2341329d21a1747dcbce80f5c56b4a0f9b1a460`, restored the original #735 adversary and passed the required exact-head matrix before being closed unmerged; #737 was then closed.

### AV6-010 / #729 — repaired and independently reverified

Production repair PR **#757**, exact repair head `d2875b19456a4bce04401b7741dbdbc3c1126771`, merged to protected `main` as `9e0827bb1f971a7061e1a713079dd2b51dd9b53d` after its complete exact-head matrix passed. The repair resolves AnthroSim-derived confirmatory per-seed values against authoritative finalized study/run state while preserving the distinction from externally authored observations.

Mandatory post-merge evidence used evidence-only PR **#758**, final evidence head `918c7f841c2e87677d49552638250879946ec4cd`. The original #728 adversary remained preserved; the documented version-drift wrapper required the repaired semantic-binding rejection and absence of published contradictory provenance. Dedicated evidence, central CI and the triggered scientific/provenance/determinism/bundle/resume checks passed. #758 was closed unmerged and #729 closed completed.

### AV6-009 / #726 — repaired and independently reverified

Production repair PR **#759**, final repair head `6130d19234bf486f8d3b775d08402ff5e17a4b9c`, merged to protected `main` as `9e1ff460b25a59bf63220899f10a7f661a5d8b2a` after the complete exact-head matrix passed, including workspace tests, release/benchmarks, 1000-run soak, canonical M7.6, performance/memory, M5/M6 integration, the 780-run demographic confirmation and applicable M8.6/M9.7 scientific gates.

The accepted statistical contract makes ordinary fixed-sample intervals descriptive at intermediate predeclared sequential boundaries and forbids them from authorizing inferential early stopping; inferential precision is decided only at the predeclared terminal boundary. Existing estimator-specific validity warnings remain additive.

Mandatory independent re-verification used evidence-only PR **#760**, final evidence head `faa371bd2d6ac3b87fc8ca82a7c2f657782d7416`, based directly on the merged repair. The original #725 discovery script remained byte-for-byte unchanged; a narrow version-drift wrapper converted its historical defect-presence terminal assertions into current-state repair assertions. Dedicated run **34501404515** and every ordinary exact-head workflow passed. Exact repaired result:

```text
fixed_coverages=n30:0.953754214943,n100:0.950153992240,n300:0.955430640776
stop_probabilities=n30:0.000000000000,n100:0.000000000000,n300:1.000000000000
reached_probabilities=n30:1.000000000000,n100:1.000000000000,n300:1.000000000000
repaired_stopped_procedure_coverage=0.955430640776
declared_confidence=0.950000000000
av6_009_reverification=pass
```

#760 was closed unmerged and #726 closed completed only after that independent P1 evidence was green.

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Issue | State |
|---|---:|---|---|---|
| AV6-001 | **P1** | A primary; D/E/F/M/N | #687 | **closed — repaired by #762; independently reverified by #763** |
| AV6-002 | **P2** | B primary; C/G/M/N | #694 | **closed — repaired/verified by #771** |
| AV6-003 | **P2** | C primary; E/N | #699 | **closed — repaired/verified by #772** |
| AV6-004 | **P1** | D primary; E/H/N | #707 | **closed — repaired by #765; independently reverified by #766** |
| AV6-005 | **P2** | D primary; C/E/F/N | #708 | **open — active production repair** |
| AV6-006 | **P1** | E primary; F/H/I/N | #711 | **closed — repaired by #768; independently reverified by #769** |
| AV6-007 | **P2** | F primary; E/L/N | #718 | open; remediation pending |
| AV6-008 | **P2** | G primary; C/M/N | #721 | open; remediation pending |
| AV6-009 | **P1** | H primary; K/L/M/N | #726 | **closed — repaired by #759; independently reverified by #760** |
| AV6-010 | **P1** | H primary; K/L/M/N | #729 | **closed — repaired by #757; independently reverified by #758** |
| AV6-011 | **P2** | I primary; K/L/M/N | #733 | open; remediation pending |
| AV6-012 | **P1** | J primary; K/L/M/N | #737 | **closed — repaired by #754; independently reverified by #755** |
| AV6-013 | **P1** | K primary; J/L/M/N | #741 | **closed — repaired by #751; independently reverified by #752** |
| AV6-014 | **P2** | L primary; G/I/M/N | #745 | open; remediation pending |

## Discovery conclusion and convergence

Scientific Audit v6 discovery is complete across A–N and remains a **non-clean** discovery pass: seven P1 findings were demonstrated on immutable `v0.3.6`. Remediation does not retroactively make that discovery generation clean.

Required path:

1. preserve completed discovery and all evidence-only findings/PRs;
2. disposition and retest all seven P2 findings explicitly;
3. require exact-head protected/scientific CI for each production repair;
4. preserve completed independent re-verification for every P1 repair;
5. freeze the fully repaired line as a new immutable release;
6. run another genuinely fresh audit generation before the framework-convergence gate for empirical work can be satisfied.

A completed framework audit does **not** establish empirical or archaeological validity. Site-specific work still requires its own evidence, parameterization, uncertainty, sensitivity, identifiability and domain review.

## Current handoff

Audit-v6 discovery Areas **A–N are complete**. All seven P1 findings are now repaired, independently reverified and closed. **7 findings remain open, all P2.** Protected `main` is `e729866f4a46d833a566a557e38570daf854d5ae` / living semantics v38 before the active AV6-002 repair.

Active work is **AV6-002/#694** in production repair PR **#771**. The repair gives the male-parent configured age window one temporal meaning—completed age at the recorded child-birth boundary—while retaining female-fertility and annual-mortality schedule lookup at interval start and preserving pre-same-day-M4 parentage locality. The branch advances living semantics v38 → v39, includes the permanent threshold-crossing regression, and has fresh v39 demographic, M7.6, M8.6 and M9.7 evidence/reference propagation. #694 is P2, so it may close after production merge plus verification; no independent P1 evidence-only derivative is required.

If #771 merges cleanly, the next queued Audit-v6 remediation is **AV6-003/#699**, unless live overlap/dependency state changes. Preserve #698 as its external-kin relabelling adversary and #399 as the earlier in-source relationship-invariance regression; do not substitute arbitrary storage/index/global-ordinal tie-breaks for the missing causal external-kin context.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live protected `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35; do not redo discovery. All seven P1 findings are repaired, independently reverified and closed. Seven P2 findings remain open. Continue controlled remediation with active AV6-002/#694 PR #771; after it merges and verifies, continue with AV6-003/#699 unless live dependency/overlap state requires a different ordering. Preserve all original discovery evidence and the established remediation/evidence protocol.
