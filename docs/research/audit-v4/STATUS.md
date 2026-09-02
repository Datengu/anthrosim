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
| Current P1 findings | **6 open — AV4-001/#486; AV4-002/#488; AV4-003/#491; AV4-004/#493; AV4-005/#495; AV4-006/#497** |
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
| A | Authoritative semantics and scheduler behaviour | **in progress — 6 P1 findings open** | Pass 1: exhaustive scheduler collision attack, PR #483 / run `33686718180`, checked **133,225** M3/M4 period-count pairs with no skipped/duplicated/reordered boundaries and matching inspected host order; no finding. Pass 2: fertility PersonId relabel, PR #485 / run `33687262609` → **AV4-001/#486**. Pass 3: background-mortality PersonId relabel, PR #487 / run `33689235132` → **AV4-002/#488**. Pass 4: M4 HouseholdId relabel, PR #490 / run `33689659272` → **AV4-003/#491**. Pass 5: newborn-sex PersonId relabel, PR #492 / run `33689950122` → **AV4-004/#493**. Pass 6: parentage PersonId relabel, PR #494 / run `33690127728` → **AV4-005/#495**. Pass 7: condition-mediated mortality PersonId relabel, PR #496 / run `33690381210` → **AV4-006/#497**. All six red evidence PRs were closed unmerged after preserving the findings. |
| B | Demography, fertility, mortality, ageing, population structure | **incomplete — AV4-001/002/004/005/006 cross-cutting** | — |
| C | Households, kinship, social links, lifecycle structure | **incomplete — AV4-003/005 cross-cutting** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete — AV4-006 cross-cutting** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete — AV4-001/002/003/004/006 cross-cutting** | — |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **incomplete — AV4-001 through AV4-006 cross-cutting** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **incomplete** | — |
| J | Identifiability, equifinality, calibration, discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete — AV4-001 through AV4-006 cross-cutting** | — |

## Finding register

| Finding | Severity | Area | Status | Issue | Fresh evidence |
|---|---|---|---|---|---|
| AV4-001 — fertility RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #486 | Closed red PR #485, head `3168cd5547952c8eb2ae715447252785584bb84e`, run `33687262609`: seed 1 birth cell changes `CellId(1)` → `CellId(2)` under pure PersonId relabelling. |
| AV4-002 — background-mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #488 | Closed red PR #487, head `c6f48d587fa7e469c97c4594c0e92d46176a004d`, run `33689235132`: seed 1 death cell changes `CellId(1)` → `CellId(2)` under pure PersonId relabelling. |
| AV4-003 — migration RNG assignment is sensitive to arbitrary household labels | **P1** | A; C/E/H/N | **demonstrated; open; deliberately unrepaired** | #491 | Closed red PR #490, head `52d739562f0dcba3be1569ef4f41108c6f0d83f2`, run `33689659272`: seed 1 changes from move `(CellId(4), CellId(1))` to no move under pure HouseholdId relabelling. |
| AV4-004 — newborn-sex RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/E/H/N | **demonstrated; open; deliberately unrepaired** | #493 | Closed red PR #492, head `156a3836e85a325fa5986bd090ec340f663c25b4`, run `33689950122`: with two forced births, seed 1 swaps male/female assignment between fixed cells under PersonId relabelling. |
| AV4-005 — parentage RNG assignment is sensitive to arbitrary male person labels | **P1** | A; B/C/H/N | **demonstrated; open; deliberately unrepaired** | #495 | Closed red PR #494, head `c48ebe8ddc45a4f3762552545139b74a085d80e6`, run `33690127728`: seed 1 changes whether the newborn selects the male already father of an existing child after a genealogy-preserving male-ID swap. |
| AV4-006 — condition-mediated mortality RNG assignment is sensitive to arbitrary founder person labels | **P1** | A; B/D/E/H/N | **demonstrated; open; deliberately unrepaired** | #497 | Closed red PR #496, head `21775e28406d9c6af803aeaa8cbbb595a1eb3e2d`, run `33690381210`, job `100447530200`: seed 1 death cell changes `CellId(2)` → `CellId(1)` under pure PersonId relabelling with background mortality disabled. |

## Session log

### 2026-09-02 — Audit v4 initialization

- Frozen target selected: `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / semantics v25.
- Audit-v4 coverage initialized at zero and prior audits treated as historical context only.
- Area A selected first for fresh scheduler, simultaneity and order-invariance attacks.

### 2026-09-02 — Area A pass 1: exhaustive scheduler collision attack

- Closed unmerged evidence PR #483 targeted the frozen release.
- Enumerated **365 × 365 = 133,225** supported M3/M4 period-count combinations.
- Same-day collision range was **1–365**; **128,400** pairs had more than one collision.
- No configured boundary was skipped, duplicated or reordered; inspected host scheduling markers agreed.
- Dedicated run `33686718180` completed with `failures=0`; this specific hypothesis was falsified.

### 2026-09-02 — Area A passes 2–7: stochastic label/order attacks

- Pass 2 / AV4-001/#486: separate fertility RNG draws are consumed in canonical person-record order; a pure relabelling moved the seed-1 birth between fixed cells.
- Pass 3 / AV4-002/#488: separate background-mortality RNG draws are likewise attached to canonical person order; a pure relabelling moved the seed-1 death between fixed cells.
- Pass 4 / AV4-003/#491: M4's shared migration choice/uncertainty streams are attached to stable HouseholdId evaluation order; a pure household relabelling changed seed-1 from one migration to no migration even though the pre-move snapshot was scientifically identical.
- Pass 5 / AV4-004/#493: with fertility occurrence forced identical, the independent newborn-sex RNG stream assigned male/female outcomes to opposite fixed cells after PersonId relabelling.
- Pass 6 / AV4-005/#495: reservoir-sampled parentage over the eligible-male vector inherited canonical record order; a genealogy-preserving male-ID swap changed whether the newborn shared the existing child's father.
- Pass 7 / AV4-006/#497: independent condition-mediated mortality draws were attached to canonical person iteration; with background mortality disabled, seed 1 moved the death between fixed cells after pure relabelling.
- Every evidence test compiled and ran under pinned Rust 1.97.1; each red result above was the intended scientific assertion rather than setup failure.
- Each defect was preserved as a P1 issue before the corresponding deliberately failing evidence PR was closed unmerged.
- **No repair is authorized yet. Area A remains in progress and discovery continues against immutable v0.3.4/v25.**
