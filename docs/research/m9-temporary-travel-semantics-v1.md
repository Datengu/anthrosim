# M9.4 temporary-travel semantics v1

**Milestone:** M9.4  
**Scientific status:** synthetic/null-model semantics; not an empirical walking model  
**Purpose:** deterministically derive focal-region destination, accumulated travel cost and integer travel duration from AnthroSim's authoritative model-facing `movement_cost` grid.

## Scope

M9.4 supplies the minimum travel model required by the M9 temporary-journey lifecycle. It does not infer roads, entrances, camps, historically preferred routes or GIS-quality least-cost paths. External GIS remains responsible for exploratory routing/cartography that is not part of authoritative simulated behaviour.

The authoritative solver operates only on the normalized AnthroSim world grid and one already-validated `FocalRegion`.

## Grid topology

Travel uses the existing authoritative four-neighbour topology in stable north, east, south, west cell order. Diagonal moves are not edges.

A cell is traversable for this travel model when its model-facing `movement_cost` is less than or equal to the configured `maximum_traversable_movement_cost`. This threshold is an explicit model assumption. It is not a hidden nodata rule and it does not change the meaning of `World::movement_cost` for other mechanisms.

Every focal-region member cell must be traversable under the configured threshold. A travel model that makes its own target region impassable is invalid and fails closed.

## Symmetric edge cost

For adjacent traversable cells `a` and `b`, the authoritative M9.4 edge cost is:

```text
edge_cost(a, b) = ceil((movement_cost(a) + movement_cost(b)) / 2)
```

The implementation uses integer arithmetic only. The formula is symmetric by construction, so `edge_cost(a, b) == edge_cost(b, a)`.

Consequences:

- two baseline cells (`1000`, `1000`) cost exactly `1000` units to cross;
- both endpoint costs contribute equally;
- no directional slope, road, entrance or downhill/uphill preference is implied by the edge-cost equation;
- accumulated route cost is the exact sum of edge costs.

The cost unit is an abstract model-facing traversal-cost unit. It is not metres, calories, minutes or any other empirical physical unit.

## Minimum-cost destination

The travel table is derived once per `(world, focal region, travel model)` using a deterministic multi-source minimum-cost search seeded by every focal-region member cell.

For each origin cell, the solver returns the minimum accumulated cost to the focal region. If more than one focal-region destination has exactly that same minimum accumulated cost, M9.4 preserves the complete canonical set of equal-cost destinations instead of collapsing the tie to one `CellId`. The stored candidate set is sorted only to make serialization and replay stable; that ordering is not an empirical destination-preference statement.

When a household evaluates a temporary-travel trigger from a tied origin, model semantics v34 uses the versioned destination policy `m9/equal-cost-destination-local-household-coupling-v3`. Its household component is produced by `m9/household-local-demographic-equivalence-v1` from the sorted multiset of the current living household members' exact birth days and reproductive sexes.

The M9 household key deliberately excludes:

- `HouseholdId`;
- `PersonId` and packed-record order;
- the globally ordinal persistent `stochastic_coupling_rank`;
- unrelated population records;
- residence and mutable condition.

This means insertion or removal of an unreachable, otherwise non-interacting founder cannot change an unchanged focal household's tie key merely by renumbering global coupling ordinals. Exact demographic duplicate households can share a key; the null ambiguity policy does not create arbitrary uniqueness from storage labels.

The destination mapping combines:

- the authoritative M9 destination-tie seed stored with the travel table;
- the origin cell under the current travel-table representation;
- the household-local M9 coupling key;
- the trigger index.

Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed declared by the spatial-realization provenance contract. The destination-policy identifier is included in travel-table/program identity, and tied authoritative departure events record the coupling key used for downstream verification.

This choice is deterministic and platform-independent and does **not** consume a mutable sequential RNG stream. Therefore an added, removed or reordered tied journey cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative program, household state and trigger reproduces the same destination exactly.

The policy is an ambiguity resolver, not an empirical claim that historical households chose equally accessible destinations randomly. If evidence later supports a directional, entrance, social or destination preference, that requires a separately declared model assumption.

Audit-v5 AV5-004/#629 is intentionally not folded into the v34 locality repair: the current tie mapping is not yet guaranteed to be spatial-reflection equivariant. Exact-tie destination-level interpretation therefore remains ambiguity-policy-sensitive pending that separate repair.

Internal minimum-cost-search queue ordering remains fully ordered for reproducibility, but queue/candidate order must not be interpreted as destination preference.

Because the result is precomputed for every world cell, temporary-journey scheduling performs an indexed lookup rather than a global route search per household.

## Unreachable origins

An origin is explicitly `Unreachable` when no four-neighbour path of traversable cells connects it to the focal region.

Unreachability is therefore caused only by the declared model-facing traversal threshold and grid connectivity. M9.4 does not silently reinterpret high-but-traversable costs as impossible travel.

## Travel capacity and duration

The travel model records an explicit `travel_capacity_cost_units_per_day` and a `ParameterProvenance` classification.

For accumulated cost `C` and capacity `K > 0`:

```text
travel_days = ceil(C / K)
```

using integer arithmetic only.

A region-resident origin has cost `0` and duration `0`; M9.3 separately classifies such households as focal-region residents rather than temporary visitors.

The capacity parameter is an abstract conversion between AnthroSim traversal-cost units and simulation days. A synthetic value must not be described as a walking speed. Evidence-informed or empirical calibration, if later justified, requires its own evidence/provenance work.

M9.4 v1 is symmetric: outbound and return travel use the same destination, accumulated cost and duration.

## Condition boundary

M9.4 v1 applies **no temporary-travel condition loss**.

This is deliberate. M4's `travel_condition_cost_per_cell` belongs to permanent-relocation semantics and is not silently reused. If a later experiment requires temporary-travel energetic/condition effects, it must introduce a separately named, documented function with its own provenance and sensitivity tests.

## Persisted identity

The travel model has a versioned schema, model identifier, provenance classification, travel-capacity parameter and maximum-traversable-cost parameter. Its deterministic identity is stored alongside the pre-resolved travel table.

An M9.4-derived table stores, for every origin:

- reachable/unreachable status;
- the canonical equal-minimum focal-region destination set when reachable;
- the selected destination implied by the keyed household/trigger resolution when a journey is evaluated;
- outbound and return travel duration;
- accumulated symmetric travel cost when reachable;
- the authoritative destination-tie seed needed for exact replay;
- travel-model identity.

Legacy hand-authored M9.3 validation tables remain constructible for lifecycle tests, but they carry no M9.4 travel-model identity or authoritative accumulated-cost table and must not be presented as M9.4-derived routing output.

## Compatibility boundary

AV5-001/#606 intentionally changes future equal-cost M9 destination realization and therefore advances `MODEL_SEMANTICS_ID` from `anthrosim-model-semantics-v33` to `anthrosim-model-semantics-v34`.

No checkpoint, event, travel-table or program schema changes are required. Existing artifacts already bind model semantics, the travel-table/program identity already binds the destination-policy identifier, and tied departure events already carry the authoritative tie key. A v33 checkpoint therefore fails the existing scientific compatibility boundary rather than being silently continued under v34.

The v34 change does not alter traversability, edge costs, route-cost minimization, equal-minimum candidate construction, travel-duration conversion or any sequential RNG stream.

## Interpretation limits

M9.4 establishes deterministic route-cost semantics, not historical route reconstruction. A low-cost model path does not imply that people used that exact path, that the focal-region destination was an archaeological entrance, or that the configured capacity reflects a real population's daily travel ability.

Equal-cost destination resolution likewise does not imply empirical indifference. Researchers interpreting destination-level visitor concentration or resource pressure should inspect the equal-cost-destination observability described in `m9-equal-cost-destination-choice-v1.md`; a high tied-origin frequency means those destination-level results depend materially on the declared ambiguity policy even when total catchment participation is unchanged.

The scientific value is narrower: otherwise identical experiments can make travel burden and arrival timing respond reproducibly to declared model-facing landscape cost while explicitly exposing where destination choice is an ambiguity-policy decision rather than an empirical inference.

## Acceptance

M9.4 is accepted when deterministic tests demonstrate:

- the frozen symmetric edge formula;
- minimum accumulated route cost;
- preservation of every exactly equal minimum-cost destination;
- deterministic keyed equal-cost destination resolution using the declared tie seed, local household coupling key and trigger index without consuming sequential RNG state or canonical IDs;
- locality under insertion/removal of unrelated population that does not alter the focal household or travel inputs;
- integer duration conversion;
- explicit unreachable origins;
- different route cost/duration when the authoritative M8 movement-cost overlay changes;
- identical derived travel tables and keyed resolutions on supported platforms for identical inputs;
- M9.3 can consume the derived table without changing persistent residence or permanent-migration meaning.
