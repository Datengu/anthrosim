# AnthroSim scientific audit v4 — status ledger

Audit target: immutable AnthroSim `v0.3.4`, tag commit `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`, model semantics `anthrosim-model-semantics-v25`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v4/README.md`

Purpose: durable repository-authoritative state for the fourth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v3 convergence pass.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v4 / fourth independent scientific audit |
| Immutable discovery target | `v0.3.4` |
| Target tag SHA | `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` |
| Target software version | `0.3.4` |
| Target model semantics | `anthrosim-model-semantics-v25` |
| Coverage state | **0/14 Areas complete — Area A in progress** |
| Current P0 findings | none discovered |
| Current P1 findings | none discovered |
| Current P2 findings | none discovered |
| Current P3 findings | none discovered |
| Convergence classification | **pending full A–N discovery** |
| Repair state | **discovery only; do not repair v4 findings until A–N discovery completes** |

## Discovery rule

- The immutable `v0.3.4` tag is the scientific discovery target.
- Audit v2/v3 evidence is historical context and regression-hypothesis material only.
- Each Area starts incomplete and requires fresh evidence.
- Preserve demonstrated defects in issues and this ledger before repair.
- Continue later Areas against the frozen v0.3.4 baseline even after findings are discovered.
- Production repair is deferred until A–N discovery is complete, absent an explicitly documented integrity emergency.

## Coverage matrix

| ID | Audit area | Status | Fresh v4 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **in progress — first scheduler/collision adversary passed; broader Area-A attacks still required** | Closed evidence PR #483, head `879d08e7e59c0128c739238c49279e1722c9961b`, workflow run `33686718180`: exhaustive M3/M4 fixed-clock enumeration checked **133,225 period-count pairs**; same-day collision count ranged **1–365**, with **46,791** pairs having >1 collision. The merged dispatcher preserved the exact ordered set union with no skipped/duplicated boundaries; both `Simulation` and `SpatialLandscapeSimulation` exposed the same inspected order markers: temporary pre-boundary → M3 resource processing → resource-period completion → M4 migration → annual M2 demography. `failures=0`. No finding from this attack. |
| B | Demography, fertility, mortality, ageing, population structure | **incomplete** | — |
| C | Households, kinship, social links, lifecycle structure | **incomplete** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete** | — |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | — |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | — |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete** | — |

## Finding register

No Audit-v4 findings yet.

## Session log

### 2026-09-02 — Audit v4 initialization

- Frozen target selected: `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / semantics v25.
- Reusable scientific audit protocol reviewed.
- Audit-v3 charter and ledger reviewed as historical context only.
- Audit-v4 coverage initialized at zero.
- Recommended first substantive area: **Area A — authoritative semantics and scheduler behaviour**, beginning with fresh scheduler/simultaneity/order-invariance attacks against v25 and targeted regression attacks on the integrated v3 repair line.

### 2026-09-02 — Area A pass 1: exhaustive scheduler collision attack

- Live protected `main` after Audit-v4 initialization: `5655ec64393d4d849cc2261d34d91d42da13b925`; scientific discovery remained fixed to immutable `v0.3.4` / `8996e99...` / v25.
- Closed unmerged evidence PR #483 targeted the frozen release rather than living `main` semantics.
- Fresh checker exhaustively enumerated **365 × 365 = 133,225** supported M3/M4 period-count combinations.
- Exact results: minimum same-day M3/M4 collisions **1**; maximum **365**; **46,791** period-count pairs had more than one same-day collision.
- Every merged dispatch matched the ordered union of the two boundary sets; configured M3 and M4 boundaries were each dispatched exactly once; day 365 was always a real shared M3/M4 boundary.
- Source-order attack verified both authoritative simulation hosts exposed the same inspected scheduler ordering markers.
- Dedicated workflow run `33686718180` completed successfully with `failures=0`.
- Disposition: this specific scheduler-collision/host-drift hypothesis was falsified. **Area A remains incomplete** pending fresh simultaneous-process, permutation/order-invariance, tie-breaking and v25 repair-integration attacks.
