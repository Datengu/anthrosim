# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed completed-area reports and executable discovery evidence remain preserved under `docs/research/audit-v6/` and on the referenced evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **14/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass: 14 findings — 7 P1, 7 P2** |
| Open Audit-v6 findings | **12** |
| Open P0/P1 | **5 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729** |
| Phase | **discovery complete — controlled remediation in progress** |
| Active ownership | **AV6-010/#729 is the next dependency-appropriate P1 remediation target unless live overlap/dependency state changes** |
| Production remediation | **AV6-013/#741 repaired by #751 and independently reverified by #752; AV6-012/#737 repaired by #754 and independently reverified by #755** |
| Convergence status | **v6 discovery remains non-clean because seven P1 findings were demonstrated; five P1 findings remain open after two verified repairs** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

Protected `main` after the verified AV6-012 production repair is `6c14decab9ce485e74bb83b58d2c0a2a2637d53a`. Scientific discovery remains attributed to immutable `v0.3.6` / v35 even though audit documentation and post-discovery remediation commits advance `main`.

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
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1, now repaired/reverified** | deterministic claim-driving outputs not bound to authoritative executions; equifinality control clean. `area-j-2026-09-09.md` |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1, now repaired/reverified** | finalized-study root verifier accepts semantically forged canonical rows; cross-root transplant control clean. `area-k-2026-09-09.md` |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | drifting initialization families collapse to identical status distributions; exposure/extinction denominator control clean. `area-l-2026-09-09.md` |
| M — documentation, TRACE/ODD/ODD+D and claim consistency | **complete — non-clean via existing findings; no new finding** | #747 claim-consistency adversary; TRACE empirical boundary remains conservative. `area-m-2026-09-09.md` |
| N — cross-system integration | **complete — non-clean via existing v6 findings; no new root finding** | #749 quantifies AV6-006 propagation from M9 destination into 5 visitor-person-days and 5 resource units. `area-n-2026-09-09.md` |

## Remediation progress

### AV6-013 / #741 P1 — repaired and independently reverified

Production repair PR **#751** merged to protected `main` as `a19fdaf98dc1d5e0bba1ed131347a7803bf8d856` after complete exact-head protected/scientific validation.

Mandatory independent post-merge re-verification used evidence-only PR **#752**, exact head `801f1d9436ff8eda5a71a08945be4e5fc022d849`, restoring the original #739 adversary byte-for-byte. Dedicated run `34425906603`, central CI `34425906576`, applicable scientific/security run `34425906837`, and the remaining triggered exact-head provenance/determinism/observability/bundle/resume workflows succeeded. #752 was closed unmerged and #741 closed only after re-verification.

### AV6-012 / #737 P1 — repaired and independently reverified

Production repair PR **#754** exact repair head `7fe94b811a84a62cd9f54487f7f50204dcb848b6` merged to protected `main` as `6c14decab9ce485e74bb83b58d2c0a2a2637d53a` after central CI, identifiability analysis, applicable scientific/security gates and the other triggered exact-head workflows passed.

The repair makes claim-driving deterministic AnthroSim-derived identifiability outputs resolve against authoritative executions before compatibility or identification is calculated, while retaining the distinction between model-derived outputs and external empirical observations.

GitHub initially auto-closed #737 on merge before mandatory P1 post-merge evidence was complete; the issue was reopened until verification finished.

Mandatory independent post-merge re-verification used evidence-only PR **#755**, exact head `a2341329d21a1747dcbce80f5c56b4a0f9b1a460`, based directly on merged `main`. The original #735 discovery workflow and adversary were restored byte-for-byte from Git blobs `66f4e370b2b7d3320951e0a593c225d9170a9eba` and `3fc3650d8b9b6a94d066b84feb308c1733f54f53` with no production changes.

Exact-head results on #755:

- dedicated `Audit v6 Area J deterministic output binding` run **34431930507** — success;
- central CI run **34431930476** — success;
- `Applicable scientific/security gates` run **34431930539** — success;
- all other triggered exact-head provenance/determinism/observability/bundle/resume workflows — success.

The unchanged original adversary accepted the truthful control and rejected the contradictory analyst-supplied deterministic output. Evidence PR #755 was closed unmerged and #737 closed only after that evidence was complete.

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Issue | State |
|---|---:|---|---|---|
| AV6-001 | **P1** | A primary; D/E/F/M/N | #687 | open; remediation pending |
| AV6-002 | **P2** | B primary; C/G/M/N | #694 | open; remediation pending |
| AV6-003 | **P2** | C primary; E/N | #699 | open; remediation pending |
| AV6-004 | **P1** | D primary; E/H/N | #707 | open; remediation pending |
| AV6-005 | **P2** | D primary; C/E/F/N | #708 | open; remediation pending |
| AV6-006 | **P1** | E primary; F/H/I/N | #711 | open; remediation pending |
| AV6-007 | **P2** | F primary; E/L/N | #718 | open; remediation pending |
| AV6-008 | **P2** | G primary; C/M/N | #721 | open; remediation pending |
| AV6-009 | **P1** | H primary; K/L/M/N | #726 | open; remediation pending |
| AV6-010 | **P1** | H primary; K/L/M/N | #729 | **open; next active remediation target unless live dependencies/overlap require otherwise** |
| AV6-011 | **P2** | I primary; K/L/M/N | #733 | open; remediation pending |
| AV6-012 | **P1** | J primary; K/L/M/N | #737 | **closed — repaired by #754; independently reverified by #755** |
| AV6-013 | **P1** | K primary; J/L/M/N | #741 | **closed — repaired by #751; independently reverified by #752** |
| AV6-014 | **P2** | L primary; G/I/M/N | #745 | open; remediation pending |

## Discovery conclusion and convergence

Scientific Audit v6 discovery is complete across the required A–N surface. The discovery result remains **non-clean**: seven P1 findings were demonstrated. Post-discovery remediation does not retroactively make v6 a clean convergence pass.

Required path:

1. preserve the completed 14/14 discovery state and all evidence-only PRs/issues;
2. remediate the remaining findings on dedicated production branches/PRs in dependency/severity order;
3. require exact-head protected/scientific CI for each repair;
4. independently re-run/re-verify the original adversarial contract for every remaining P0/P1 repair, using the version-drift addendum when needed;
5. disposition/retest P2 findings explicitly;
6. freeze the fully repaired line as a new immutable release;
7. run another genuinely fresh audit generation before the framework-convergence gate for empirical work can be satisfied.

A completed framework audit does **not** establish empirical or archaeological validity. Site-specific work still requires its own evidence, parameterization, uncertainty, sensitivity, identifiability and domain review.

## Current handoff

Audit-v6 discovery Areas **A–N are complete**. AV6-013/#741 and AV6-012/#737 have both been repaired on protected `main`, independently reverified using their preserved original adversaries, and closed. **12 findings remain open: 5 P1 and 7 P2.**

With no live open PR or competing remediation branch, the next dependency-appropriate P1 is **AV6-010/#729**. It is the stochastic analogue of the authoritative-output seam just repaired for AV6-012: the confirmatory Monte Carlo path binds seed identities and byte-level provenance but still accepts per-seed values without proving that those values were derived from the authoritative frozen study outputs. Preserve #728 as the acceptance adversary, keep external/user-authored observations distinct from AnthroSim-derived values, and independently reverify the P1 repair after merge.

AV6-009/#726 remains a separate statistical-validity defect in sequential stopping/coverage and must not be silently conflated with AV6-010's semantic sample-value binding.

Before creating the AV6-010 production branch, reconstruct current protected `main`, open PRs/issues and overlapping work and treat this ledger plus live GitHub state as authoritative.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live protected `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35; do not redo discovery. AV6-013/#741 and AV6-012/#737 are repaired, independently reverified and closed. Twelve findings remain open, including five P1s. Continue controlled remediation with AV6-010/#729 unless live dependency/overlap state requires a different ordering; preserve original discovery evidence and independently reverify every P0/P1 repair before closure.
