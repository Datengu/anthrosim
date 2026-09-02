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
| Current P1 findings | **2 open — AV4-001/#486; AV4-002/#488** |
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

## Coverage matrix

| ID | Audit area | Status | Fresh v4 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **in progress — 2 P1 findings open** | Scheduler attack: closed PR #483 / run `33686718180` checked **133,225** M3/M4 period-count pairs, collision range **1–365**, **128,400** pairs with >1 collision, no skipped/duplicated/reordered boundaries, and matching inspected host order; no finding. Fertility label-order attack: closed red PR #485 / evidence head `3168cd5547952c8eb2ae715447252785584bb84e` / run `33687262609` demonstrated **AV4-001/#486 (P1)**: pure canonical person relabelling changed the seed-1 first-year birth cell from `CellId(1)` to `CellId(2)`. Mortality label-order attack: closed red PR #487 / evidence head `c6f48d587fa7e469c97c4594c0e92d46176a004d` / run `33689235132` demonstrated **AV4-002/#488 (P1)**: pure canonical person relabelling changed the seed-1 death cell from `CellId(1)` to `CellId(2)`. Separate sequential fertility and mortality RNG streams are both attached to canonical record iteration, making arbitrary labels spatially causal. |
| B | Demography, fertility, mortality, ageing, population structure | **incomplete — AV4-001/002 cross-cutting** | — |
| C | Households, kinship, social links, lifecycle structure | **incomplete** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete — AV4-001/002 cross-cutting** | — |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **incomplete — AV4-001/002 cross-cutting** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **incomplete** | — |
| J | Identifiability, equifinality, calibration, discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete — AV4-001/002 cross-cutting** | — |

## Finding register

| Finding | Severity | Area | Status | Issue | Evidence / later repair requirement |
|---|---|---|---|---|---|
| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A primary; B/E/H/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #486 | Closed red PR #485, evidence head `3168cd5547952c8eb2ae715447252785584bb84e`, run `33687262609`: pure canonical-label permutation of two equivalent household-local female/male pairs changes same-seed birth cell at seed 1 from **CellId(1)** to **CellId(2)**. Later repair must make stochastic fertility realization label/order invariant without substituting another arbitrary storage key, preserve deterministic replay/provenance, and independently reverify after merge. |
| AV4-002 — background-mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A primary; B/E/H/N cross-cutting | **demonstrated; open; deliberately unrepaired** | #488 | Closed red PR #487, evidence head `c6f48d587fa7e469c97c4594c0e92d46176a004d`, run `33689235132`, job `100443882649`: pure canonical-label permutation of two otherwise-equivalent fixed-cell founders changes same-seed death cell at seed 1 from **CellId(1)** to **CellId(2)**. Later repair must make background-mortality stochastic assignment label/order invariant while preserving competing-risk semantics, stream separation, deterministic replay/provenance, and independent post-merge reverification. |

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
- Exact results: minimum same-day M3/M4 collisions **1**; maximum **365**; **128,400** period-count pairs had more than one same-day collision.
- Every merged dispatch matched the ordered union of the two boundary sets; configured M3 and M4 boundaries were each dispatched exactly once; day 365 was always a real shared M3/M4 boundary.
- Source-order attack verified both authoritative simulation hosts exposed the same inspected scheduler ordering markers.
- Dedicated workflow run `33686718180` completed successfully with `failures=0`.
- Disposition: this specific scheduler-collision/host-drift hypothesis was falsified. Area A remained incomplete.

### 2026-09-02 — Area A pass 2: canonical person-label fertility attack

- Closed red evidence PR #485 targeted immutable v0.3.4/v25 and constructed two founder states identical after erasing canonical person labels.
- Each arm had two fixed households/cells, one 30-year-old female and one 30-year-old male per household, mortality off, fertility 500,000 per million, zero birth spacing, zero resource need, migration off, and one-year horizon.
- The only transformation exchanged which canonical person labels identified the two otherwise-equivalent household-local pairs.
- Dedicated workflow `33687262609` compiled the adversary successfully under Rust 1.97.1 and failed at the intended scientific assertion immediately at seed 1: **A=[CellId(1)] vs B=[CellId(2)]**.
- Source inspection ties the failure to a shared sequential fertility RNG consumed while iterating canonical population records; the same random realization is therefore attached to different fixed spatial households after arbitrary relabelling.
- Finding preserved as **AV4-001/#486, P1** before any repair. The minimal construction preserves first-year total births but changes spatial attribution; downstream household/resource/migration/aggregation state can therefore diverge causally.
- Per Audit-v4 discovery rules, **do not repair #486 yet**. Continue fresh discovery against immutable v0.3.4/v25.

### 2026-09-02 — Area A pass 3: canonical person-label mortality attack

- Closed red evidence PR #487 targeted immutable v0.3.4/v25 with two fixed one-person households at cells 1 and 2, each containing an otherwise-equivalent 30-year-old male founder.
- The only transformation exchanged which canonical `PersonId` occupied which fixed household/cell; both arms used the same initialization identity and were scientifically identical after erasing record labels.
- Mortality was forced to 500,000 per million across all age bands; fertility was zero; annual resource need was zero; migration was disabled; horizon was one year.
- Dedicated workflow run `33689235132` / job `100443882649` compiled successfully under pinned Rust 1.97.1 and then failed at the intended scientific assertion immediately at seed 1: **A=[CellId(1)] vs B=[CellId(2)]**.
- This is a separate stochastic mechanism from AV4-001: background mortality has its own RNG stream, but sequential draws are likewise attached to canonical population-record iteration.
- Finding preserved as **AV4-002/#488, P1** before repair. Arbitrary bookkeeping identity can determine which spatially situated person dies and can therefore propagate into household, kin, resource, fertility, migration, temporary-mobility and spatial outputs.
- Per Audit-v4 discovery rules, **do not repair #486 or #488 yet**. Continue fresh discovery against immutable v0.3.4/v25; next Area-A attack should test household-label/order coupling in M4 migration and other stochastic simultaneous processes.
