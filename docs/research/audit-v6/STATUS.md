# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed completed-area reports and executable evidence are preserved under `docs/research/audit-v6/` and on the referenced evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **14/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass: 14 findings — 7 P1, 7 P2** |
| Open Audit-v6 findings | **13** |
| Open P0/P1 | **6 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737** |
| Phase | **discovery complete — controlled remediation in progress** |
| Active ownership | **AV6-012/#737 is the next dependency-appropriate P1 remediation target** |
| Production remediation | **AV6-013/#741 repaired by #751 and independently reverified by #752; all remaining repairs require dedicated production PRs and preserved original evidence** |
| Convergence status | **v6 discovery remains non-clean because seven P1 findings were demonstrated; six P1 findings remain open after one verified repair** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

Protected `main` immediately before final Area-N evidence was `d341d0a6462f175bcfd1f26df148d074e9f80f4a`. Scientific discovery remains attributed to immutable `v0.3.6` / v35 even though audit documentation and post-discovery remediation commits advance `main`.

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
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1** | deterministic claim-driving outputs not bound to authoritative executions; equifinality control clean. `area-j-2026-09-09.md` |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1** | finalized-study root verifier accepts semantically forged canonical rows; cross-root transplant control clean. `area-k-2026-09-09.md` |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | drifting initialization families collapse to identical status distributions; exposure/extinction denominator control clean. `area-l-2026-09-09.md` |
| M — documentation, TRACE/ODD/ODD+D and claim consistency | **complete — non-clean via existing AV6-001, AV6-013, AV6-014; no new finding** | #747 claim-consistency adversary; TRACE empirical boundary remains conservative. `area-m-2026-09-09.md` |
| N — cross-system integration | **complete — non-clean via existing v6 findings; no new root finding** | #749 quantifies AV6-006 propagation from M9 destination into 5 visitor-person-days and 5 resource units. `area-n-2026-09-09.md` |

## Final Area-N evidence

Evidence-only PR **#749** composes AV6-006's M9 spatial-domain locality defect with actual temporary aggregation and M3 duration-aware resource accounting.

Final candidate evidence head: `e0b636735c486e0318004cb9cbe558de50d02ced`  
Dedicated workflow: `34409746294`

The corrected controlled fixture preserves the same reachable 3-cell local problem and adds only one outside-focal-region cell whose movement cost is `6000`, above the M9 traversal ceiling `5000`.

At process seed 0 the full authoritative host produces:

```text
baseline_destination=CellId(1)
padded_destination=CellId(3)
baseline_visitor_person_days=[5, 0, 0]
padded_visitor_person_days=[0, 0, 5]
baseline_food_stock=[9995, 9640, 10000]
padded_food_stock=[10000, 9640, 9995]
```

Therefore the causally irrelevant impassable padding moves **five actual visitor-person-days** and exactly **five units of M3 visitor resource demand** between unchanged local cells. This is a fresh cross-system consequence of AV6-006/#711, not a separate smallest defect, so no duplicate AV6 issue is created.

Two earlier #749 attempts are excluded from evidence: one compile-only fixture error and one zero-capacity fixture error. The scientific oracle was unchanged. See `area-n-2026-09-09.md` for the full evidence hygiene and integration synthesis.

## Remediation progress

### AV6-013 / #741 P1 — repaired and independently reverified

Production repair PR **#751** merged to protected `main` as `a19fdaf98dc1d5e0bba1ed131347a7803bf8d856`.

The repaired finalized-study root verifier now deterministically reconstructs the canonical `research/analysis/points.json` and `runs.json` tables from immutable research plan/state and requires complete semantic equality before returning a verified root. Existing artifact-digest, producer-defined `resultIdentity`, study/protocol/research lineage and replay/provenance checks remain in force.

PR #751's exact final repair head passed the complete protected/scientific suite before merge, including the dedicated study-result-binding chain and its real `anthrosim-study` producer-to-provenance integration.

Mandatory independent post-merge re-verification used evidence-only PR **#752**, exact head `801f1d9436ff8eda5a71a08945be4e5fc022d849`, based directly on the merged repair. The original #739 discovery workflow, Rust test and helper were restored **byte-for-byte from their original Git blobs**, with no production changes.

Dedicated re-verification run **34425906603**, job **102710963050**, passed. The original self-consistent attack still forged canonical treatment rows from `[4,12]` to `[999,12]`, recomputed both artifact digests and a fresh `resultIdentity`, and preserved immutable execution lineage. The producer continued to reject the forged root, and the repaired root-aware verifier now independently rejected it as well:

```text
producer_finalize_rejects=true
root_verifier_accepted=false
root_verifier_error=research analysis artifact differs from immutable research plan/state: .../research/analysis/points.json
```

The same evidence head also passed central CI **34425906576** in full, including Quality and tests **102710993310**, release build, core benchmarks, performance/memory acceptance, 1000-run ensemble soak, canonical M7.6 reference and M5/M6 integration. Applicable scientific/security run **34425906837** passed including M8.6 and M9.7, and all other exact-head provenance/determinism/observability/bundle/resume workflows succeeded.

Evidence PR #752 was closed unmerged after classification. Issue #741 was closed completed only after this independent evidence and full exact-head repository validation succeeded.

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
| AV6-010 | **P1** | H primary; K/L/M/N | #729 | open; remediation pending |
| AV6-011 | **P2** | I primary; K/L/M/N | #733 | open; remediation pending |
| AV6-012 | **P1** | J primary; K/L/M/N | #737 | **open; next active remediation target** |
| AV6-013 | **P1** | K primary; J/L/M/N | #741 | **closed — repaired by #751; independently reverified by #752** |
| AV6-014 | **P2** | L primary; G/I/M/N | #745 | open; remediation pending |

## Discovery conclusion and convergence

Scientific Audit v6 discovery is complete across the required A–N surface.

The discovery result is **non-clean**. Seven P1 findings mean the charter's desired P1-clean convergence pass was not achieved by v6. Post-discovery remediation does not retroactively turn the v6 discovery pass into a clean audit; it establishes a repaired line that must eventually be frozen and freshly audited again.

Required next path:

1. preserve the completed 14/14 discovery state and all evidence-only PRs/issues;
2. remediate the remaining 13 findings on dedicated production branches/PRs in dependency/severity order;
3. run exact-head protected/scientific CI for each repair;
4. independently re-run/re-verify the original adversarial contract for every remaining P0/P1 repair, using the version-drift addendum where live repository evolution matters;
5. disposition/retest P2 findings explicitly;
6. freeze the fully repaired line as a new immutable release;
7. run another genuinely fresh audit generation before the framework-convergence gate for empirical work can be satisfied.

A completed framework audit does **not** establish empirical or archaeological validity. Site-specific work still requires its own evidence, parameterization, uncertainty, sensitivity, identifiability and domain review.

## Current handoff

Audit-v6 discovery Areas **A–N are complete**. One P1 finding, AV6-013/#741, has now been repaired on protected `main`, independently reverified with the original adversary, and closed. **13 findings remain open: 6 P1 and 7 P2.**

Next action: remediate **AV6-012/#737**. Its executed-design binder already proves the authoritative treatment coordinates, structures and execution IDs, but the identifiability wrapper still passes analyst-supplied deterministic `outputs` through to legacy inference without binding those values to the authoritative executions. Preserve the exact #735 truthful/contradictory adversary as the acceptance test, keep externally authored empirical observations distinct from AnthroSim-derived outputs, and independently reverify the P1 repair after merge.

Before creating that production branch, reconstruct current protected `main`, open PRs/issues and overlapping work and treat this ledger as authoritative.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live protected `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35. AV6-013/#741 has been repaired by #751, independently reverified by evidence-only #752 and closed. Thirteen findings remain open, including six P1s. Continue controlled remediation with AV6-012/#737 unless live dependency/overlap state requires a different ordering; preserve original discovery evidence and independently reverify every P0/P1 repair before closure.
