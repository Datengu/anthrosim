# AnthroSim scientific audit v2 — status ledger

Audit target at initialization: AnthroSim `v0.3.2`, protected-main commit `eb240ab482d9683b64081d3d1ea8e151592927ee`.

Protocol: `docs/research/scientific-audit-protocol.md`

Purpose: durable, repository-authoritative state for the second independent/adversarial scientific audit of the integrated AnthroSim framework. This file must be updated during the audit so work can continue safely across agents and conversations without relying on chat history.

## Audit rules

- Verify live `main`, open issues, open PRs, and overlapping branches before each audit session.
- Record exact commit SHAs for evidence.
- Do not mark an area complete on code inspection alone; use adversarial or quantitative evidence where feasible.
- Search existing issues/PRs before creating findings.
- Preserve demonstrated findings before repair.
- Keep P0/P1 fixes distinct from independent re-verification.
- Do not upgrade AnthroSim to empirical research readiness merely because this audit passes.

## Current baseline

| Field | Value |
|---|---|
| Audit generation | v2 / second independent scientific audit |
| Initial release | `v0.3.2` |
| Initial protected-main SHA | `eb240ab482d9683b64081d3d1ea8e151592927ee` |
| Initial model semantics | v15 |
| Initial issue state | no open issues at audit initialization |
| Audit state | not started |

If `main` advances during the audit, add the new SHA to the session log and state whether earlier evidence remains applicable.

## Coverage matrix

Statuses: `not started`, `in progress`, `complete — no finding`, `complete — findings`, `blocked`, `repeat required`.

| ID | Audit area | Status | Evidence / notes | Findings / issues |
|---|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | not started | — | — |
| B | Demography, fertility, mortality, ageing, population structure | not started | — | — |
| C | Households, kinship, social links, lifecycle structure | not started | — | — |
| D | Resources, condition, subsistence, depletion/recovery | not started | — | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | not started | — | — |
| F | Aggregation and interaction mechanisms | not started | — | — |
| G | Initialization, burn-in, path dependence, continuation state | not started | — | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | not started | — | — |
| I | Sensitivity, uncertainty, convergence, robustness | not started | — | — |
| J | Identifiability, equifinality, calibration, discrimination | not started | — | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | not started | — | — |
| L | Observability, analysis outputs, statistical summaries | not started | — | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | not started | — | — |
| N | Cross-system integration | not started | — | — |

## Cross-system integration checklist

These checks are mandatory after the corresponding subsystem work is mature enough to support them.

| Interaction | Status | Evidence / findings |
|---|---|---|
| Demography × households | not started | — |
| Demography × resources | not started | — |
| Households × movement | not started | — |
| Movement × resources | not started | — |
| Aggregation × resources | not started | — |
| Initialization × demography | not started | — |
| Initialization × spatial placement | not started | — |
| Stochastic inference × censoring/extinction | not started | — |
| Sensitivity × hidden configuration | not started | — |
| Calibration × identifiability | not started | — |
| Checkpoint/resume × RNG | not started | — |
| Observability × scientific interpretation | not started | — |

## Finding register

Add every scientifically substantive finding here, including findings that are later closed.

| Finding | Severity | Area | Status | Issue/PR | Evidence location | Reverified? |
|---|---|---|---|---|---|---|
| — | — | — | — | — | — | — |

Suggested finding statuses: `hypothesis`, `demonstrated`, `issue open`, `fix in progress`, `fixed`, `reverified`, `not a defect`, `accepted limitation`.

## Session log

Each audit session must append an entry using the template below. Keep entries concise but sufficient for a fresh agent to reconstruct the audit state.

### Session template

**Date / agent:**

**Live main SHA:**

**Release / model semantics:**

**Audit area / sub-area:**

**Overlap check:**
- open issues:
- open PRs/branches:
- parallel-agent overlap:

**Implementation/docs inspected:**

**Tests/experiments performed:**

**Quantitative results:**

**Findings:**

**Issues/PRs created or relevant:**

**Unresolved hypotheses:**

**Explicitly not examined:**

**Does evidence need repeating after main changes?**

**Recommended next step:**

---

## Final audit synthesis

Do not fill this section until the coverage matrix and mandatory integration checks are complete.

The final synthesis must state:

- final audited protected-main SHA;
- release/model-semantics state;
- areas completed;
- P0/P1/P2/P3 finding counts;
- which P0/P1 findings were independently reverified;
- unresolved uncertainties/accepted limitations;
- whether a fresh audit still uncovered major scientific defects;
- whether the audit shows meaningful convergence relative to the previous audit;
- what empirical claims remain unsupported;
- recommended next scientific phase.
