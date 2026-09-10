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
| Open Audit-v6 findings | **10** |
| Open P0/P1 | **3 — AV6-001/#687; AV6-004/#707; AV6-006/#711** |
| Phase | **discovery complete — controlled remediation in progress** |
| Verified P1 remediations | **4/7 — AV6-013/#741; AV6-012/#737; AV6-010/#729; AV6-009/#726** |
| Active ownership | **AV6-001/#687 is the next dependency-appropriate P1 remediation target unless live overlap/dependency state changes** |
| Convergence status | **v6 discovery remains non-clean because seven P1 findings were demonstrated; three P1 findings remain open after four verified repairs** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

Protected `main` after the verified AV6-009 production repair is `9e1ff460b25a59bf63220899f10a7f661a5d8b2a`. Scientific discovery remains attributed to immutable `v0.3.6` / v35 even though post-discovery remediation advances `main`.

## Completed Areas

| Area | Discovery status | Fresh v6 evidence |
|---|---|---|
| A — authoritative semantics and scheduler behaviour | **complete — AV6-001/#687 P1** | #684 clean scheduler-equivalence control; #686/#687 same-day M4-before-newly-due-M9 inversion. `area-a-2026-09-09.md` |
| B — demography, fertility, mortality, ageing and population structure | **complete — AV6-002/#694 P2** | #692 clean chronology/cadence; #693/#694 inconsistent male-parent age-window references. `area-b-2026-09-09.md` |
| C — households, kinship, social links and lifecycle structure | **complete — AV6-003/#699 P2** | #698/#699 external-kin context collapse; #701 clean relabelling control. `area-c-2026-09-09.md` |
| D — resources, condition, subsistence, depletion/recovery | **complete — AV6-004/#707 P1; AV6-005/#708 P2** | scarce-resource reflection and duration-split household-index defects. `area-d-2026-09-09.md` |
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1** | unreachable impassable padding changes tied M9 destination; locality controls clean. `area-e-2026-09-09.md` |
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
| AV6-001 | **P1** | A primary; D/E/F/M/N | #687 | **open — next P1 remediation target** |
| AV6-002 | **P2** | B primary; C/G/M/N | #694 | open; remediation pending |
| AV6-003 | **P2** | C primary; E/N | #699 | open; remediation pending |
| AV6-004 | **P1** | D primary; E/H/N | #707 | open; remediation pending |
| AV6-005 | **P2** | D primary; C/E/F/N | #708 | open; remediation pending |
| AV6-006 | **P1** | E primary; F/H/I/N | #711 | open; remediation pending |
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
2. remediate the remaining three P1 findings in dependency/severity order;
3. require exact-head protected/scientific CI for each production repair;
4. independently re-run/re-verify every P1 adversarial contract after merge, using the version-drift addendum only where necessary;
5. disposition and retest all seven P2 findings explicitly;
6. freeze the fully repaired line as a new immutable release;
7. run another genuinely fresh audit generation before the framework-convergence gate for empirical work can be satisfied.

A completed framework audit does **not** establish empirical or archaeological validity. Site-specific work still requires its own evidence, parameterization, uncertainty, sensitivity, identifiability and domain review.

## Current handoff

Audit-v6 discovery Areas **A–N are complete**. Four of seven P1 findings are now repaired, independently reverified and closed: AV6-013/#741, AV6-012/#737, AV6-010/#729 and AV6-009/#726. **10 findings remain open: 3 P1 and 7 P2.** There are no live open PRs at this handoff.

The next dependency-appropriate P1 is **AV6-001/#687** because it concerns the authoritative same-day scheduler order itself (`M3 -> M9 -> M4 -> annual M2` where applicable). Repair it before relying on downstream M9/resource interaction semantics in AV6-004/#707 and AV6-006/#711.

Preserve #686 as the acceptance adversary. The repair must eliminate the same-day M4-then-newly-due-M9 inversion without undoing #197's valid future target-arrival reconsideration behavior. Cover `new departure <`, `==`, and `>` the M4 day, coincident M3/M9/M4 boundaries, core/spatial parity, checkpoint/resume and duration/resource-ledger consistency. Because AV6-001 is P1, keep #687 open through production merge and close it only after independent post-merge re-verification.

Before creating the AV6-001 production branch, reconstruct current protected `main`, open PRs/issues and overlapping work and treat this ledger plus live GitHub state as authoritative.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live protected `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35; do not redo discovery. AV6-013/#741, AV6-012/#737, AV6-010/#729 and AV6-009/#726 are repaired, independently reverified and closed. Ten findings remain open: three P1 and seven P2. Continue controlled remediation with AV6-001/#687 unless live dependency/overlap state requires a different ordering; preserve original discovery evidence and independently reverify every P1 repair before closure.
