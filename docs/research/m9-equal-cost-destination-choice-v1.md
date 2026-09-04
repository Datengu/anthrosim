# M9 equal-cost destination choice semantics v1

## Scientific problem

M9.4 can find more than one focal-region destination with exactly the same minimum accumulated movement cost from an origin. Before model semantics v18, the routing table collapsed such a tie to the lower authoritative `CellId`. Because `CellId` follows grid storage order, that deterministic fallback created a causal spatial preference unrelated to the travel model.

## v18 rule

M9.4 now preserves the complete canonical set of exactly equal minimum-cost destinations for every origin. The set is sorted only for stable serialization; ordering is not the causal choice rule. Each candidate also retains the minimum route-edge count among paths achieving that same minimum cost.

Model semantics v31 replaces the original canonical-`HouseholdId` component of that rule with the versioned policy `m9/equal-cost-destination-scientific-coupling-v2`. Authoritative execution keys a tied destination from the authoritative M9 tie seed, origin cell, trigger index and the household's scientific coupling key: the minimum persistent person `stochastic_coupling_rank` among its living members. Canonical `HouseholdId` is bookkeeping only and is not a causal input. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

The policy identifier is bound into the travel-table/program identity, and authoritative tied-departure events record the scientific coupling key used so observability can independently verify the selected destination. A label-neutral compatibility resolver exists for callers that have only a canonical household ID; authoritative simulation execution never uses that ID as the tie key.

This choice is deterministic and platform independent, but it consumes no sequential RNG draw. Therefore adding or removing a tied journey cannot shift M2, M3, M4 or other stochastic streams. Replaying the same scientific household/trigger under the same program reproduces the same destination exactly.

The keyed policy is a neutral ambiguity resolver, not evidence that historical households chose destinations randomly. If evidence supports destination preference, that preference requires a separate explicit model.

## Symmetry and interpretation

No candidate receives priority because it is north, west, first in row-major storage or lower in `CellId`. Across households/seeds, symmetric alternatives receive both outcomes under the keyed mapping. A single deterministic run can still contain sampling imbalance; that is stochastic/keyed realization, not a fixed directional rule.

Non-tied minima are unchanged. Reachability, accumulated route cost and travel duration are unchanged by the tie key. Only which scientifically indistinguishable minimum-cost destination receives the visit can change.

## Observability

Temporary-mobility observability v3 reports:

- the number of world origins with more than one equal-cost destination;
- the maximum equal-cost destination count;
- the number of started journeys whose origin was tied;
- `equalCostDestinationCount` for each started journey and origin-catchment row.

Researchers should inspect these fields before interpreting destination-level visitor concentration or resource pressure. A high tied-origin frequency means destination-level conclusions depend materially on the declared ambiguity policy even when total catchment participation is stable.

Downstream observability regeneration reads the authoritative tie seed preserved in the M9 travel table rather than guessing the seed role from the outer experiment config. Core and spatial execution hosts remain responsible for validating that the stored seed matches their declared seed-role provenance. Any validation or replay check that re-derives an executed M9 program must therefore use that same authoritative tie seed when testing program equality.

## Provenance boundary

The original equal-minimum preservation advanced `MODEL_SEMANTICS_ID` from v17 to v18. Audit-v4 AV4-007 remediation advances the current line from v30 to v31 because future tied M9 destinations now use scientific household coupling identity rather than canonical `HouseholdId`. It does not change travel-cost equations, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream.
