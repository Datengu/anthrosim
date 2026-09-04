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
- no directional slope, road, entrance or downhill/uphill preference is implied;
- accumulated route cost is the exact sum of edge costs.

The cost unit is an abstract model-facing traversal-cost unit. It is not metres, calories, minutes or any other empirical physical unit.

## Minimum-cost destination

The travel table is derived once per `(world, focal region, travel model)` using a deterministic multi-source minimum-cost search seeded by every focal-region member cell.

For each origin cell, the solver returns the minimum accumulated cost to the focal region. If more than one focal-region destination has exactly that same minimum accumulated cost, M9.4 preserves the complete canonical set of equal-cost destinations instead of collapsing the tie to one `CellId`. The stored candidate set is sorted only to make serialization and replay stable; that ordering is not the causal destination-choice rule.

When a household actually evaluates a temporary-travel trigger from a tied origin, authoritative execution chooses among the equal minima with the versioned policy `m9/equal-cost-destination-scientific-coupling-v2`. The key is composed from:

- the authoritative M9 destination-tie seed stored with the travel table;
- the origin `CellId`;
- the household scientific coupling key, defined as the minimum persistent person `stochastic_coupling_rank` among its living members;
- the trigger index.

Canonical `HouseholdId` is excluded from the causal key. The keyed value is passed through the fixed integer avalanche defined by that policy and reduced to the candidate count. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed declared by the spatial-realization provenance contract. The tie-policy identifier is included in travel-table/program identity, and tied authoritative departure events record the coupling key used for downstream verification.

This choice is deterministic and platform-independent, but it does **not** consume a mutable sequential RNG stream. Therefore an added, removed or reordered tied journey cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative tie seed, origin, scientific household coupling key and trigger index reproduces the same destination exactly.

The policy is an ambiguity resolver, not an empirical claim that historical households chose equally accessible destinations randomly. If evidence later supports a directional, entrance, social or destination preference, that requires a separately declared model assumption.

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
- the selected destination implied by the keyed scientific-household/trigger resolution when a journey is evaluated;
- outbound and return travel duration;
- accumulated symmetric travel cost when reachable;
- the authoritative destination-tie seed needed for exact replay;
- travel-model identity.

Legacy hand-authored M9.3 validation tables remain constructible for lifecycle tests, but they carry no M9.4 travel-model identity or authoritative accumulated-cost table and must not be presented as M9.4-derived routing output.

## Interpretation limits

M9.4 establishes deterministic route-cost semantics, not historical route reconstruction. A low-cost model path does not imply that people used that exact path, that the focal-region destination was an archaeological entrance, or that the configured capacity reflects a real population's daily travel ability.

Equal-cost destination resolution likewise does not imply empirical indifference. Researchers interpreting destination-level visitor concentration or resource pressure should inspect the equal-cost-destination observability described in `m9-equal-cost-destination-choice-v1.md`; a high frequency of tied origins means those destination-level results depend materially on the declared ambiguity policy even when total catchment participation is unchanged.

The scientific value is narrower: otherwise identical experiments can now make travel burden and arrival timing respond reproducibly to declared model-facing landscape cost while avoiding a permanent row-major/low-`CellId` spatial preference in exact ties.

## Acceptance

M9.4 is accepted when deterministic tests demonstrate:

- the frozen symmetric edge formula;
- minimum accumulated route cost;
- preservation of every exactly equal minimum-cost destination;
- deterministic keyed equal-cost destination resolution using the declared tie seed, origin, scientific household coupling key and trigger index without consuming sequential RNG state or canonical HouseholdId;
- integer duration conversion;
- explicit unreachable origins;
- different route cost/duration when the authoritative M8 movement-cost overlay changes;
- identical derived travel tables and keyed resolutions on supported platforms for identical inputs;
- M9.3 can consume the derived table without changing persistent residence or permanent-migration meaning.
