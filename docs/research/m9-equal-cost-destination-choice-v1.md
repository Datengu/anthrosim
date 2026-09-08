# M9 equal-cost destination choice semantics v1

## Scientific problem

M9.4 can find more than one focal-region destination with exactly the same minimum accumulated movement cost from an origin. Before model semantics v18, the routing table collapsed such a tie to the lower authoritative `CellId`. Because `CellId` follows grid storage order, that deterministic fallback created a causal spatial preference unrelated to the travel model.

## Equal-minimum preservation

M9.4 preserves the complete canonical set of exactly equal minimum-cost destinations for every origin. The set is sorted only for stable serialization; ordering is not intended to encode empirical preference. Each candidate also retains the minimum route-edge count among paths achieving that same minimum cost.

Model semantics v31 replaced the original canonical-`HouseholdId` tie component with `m9/equal-cost-destination-scientific-coupling-v2`, using the minimum persistent person `stochastic_coupling_rank` among a household's living members. Audit-v5 AV5-001/#606 demonstrated that those persistent ranks are global ordinals: adding an unrelated, unreachable founder can renumber an otherwise unchanged focal household and therefore alter its M9 tie destination.

Model semantics v34 repairs that nonlocal coupling with two explicit policy identities:

- destination policy: `m9/equal-cost-destination-local-household-coupling-v3`;
- household coupling policy: `m9/household-local-demographic-equivalence-v1`.

For authoritative execution, the M9 household coupling key is derived only from the household's current living-member multiset of stable demographic attributes used by this null ambiguity policy:

- exact `birth_day`;
- reproductive sex.

The member tuples are sorted before hashing. `PersonId`, packed-record order, `HouseholdId`, persistent global stochastic-coupling rank, residence, condition, and unrelated population records are not inputs to this M9 household key. Exact demographic duplicate households may therefore share the same ambiguity realization; the policy does not manufacture a scientifically unsupported distinction merely to obtain globally unique keys.

The destination policy then combines:

- the authoritative M9 destination-tie seed stored with the travel table;
- the origin cell under the current travel-table representation;
- the household-local M9 coupling key;
- the trigger index.

Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

The destination-policy identifier is bound into travel-table/program identity, and authoritative tied-departure events record the M9 coupling key used so observability can independently replay the selected destination. A label-neutral compatibility resolver remains for callers that have only a canonical household ID; authoritative simulation execution does not use that ID as a causal tie key.

This choice is deterministic and platform independent and consumes no sequential RNG draw. Adding or removing a tied journey therefore cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative program, focal household state and trigger reproduces the same tie realization exactly.

The keyed policy is a neutral ambiguity resolver, not evidence that historical households chose destinations randomly. If evidence supports destination preference, that preference requires a separate explicit model.

## Locality, symmetry and interpretation

The v34 locality contract is deliberately narrow: adding or removing unrelated population that does not change the focal household or its travel inputs must not perturb the focal household's M9 tie coupling key or tied destination merely through global ordinal renumbering.

Non-tied minima are unchanged. Reachability, accumulated route cost and travel duration are unchanged by the v34 repair. Only the ambiguity coupling identity for exactly tied M9 destinations changes.

Audit-v5 AV5-004/#629 remains a separate open defect at the point this v34 repair is introduced: the current destination mapping is not yet guaranteed to be equivariant under spatial reflection because spatial representation enters the tie-resolution mapping. The v34 locality repair must therefore not be interpreted as proof of full spatial symmetry. Destination-level analyses should continue to treat exact ties as ambiguity-policy-sensitive until #629 is separately remediated and reverified.

## Observability

Temporary-mobility observability v3 reports:

- the number of world origins with more than one equal-cost destination;
- the maximum equal-cost destination count;
- the number of started journeys whose origin was tied;
- `equalCostDestinationCount` for each started journey and origin-catchment row.

Researchers should inspect these fields before interpreting destination-level visitor concentration or resource pressure. A high tied-origin frequency means destination-level conclusions depend materially on the declared ambiguity policy even when total catchment participation is stable.

Downstream observability regeneration reads the authoritative tie seed preserved in the M9 travel table rather than guessing the seed role from the outer experiment config. Core and spatial execution hosts remain responsible for validating that the stored seed matches their declared seed-role provenance. Any validation or replay check that re-derives an executed M9 program must therefore use that same authoritative tie seed and the recorded M9 household coupling key when testing program equality.

## Provenance and compatibility boundary

The original equal-minimum preservation advanced `MODEL_SEMANTICS_ID` from v17 to v18. Audit-v4 AV4-007 remediation advanced the living line from v30 to v31 because future tied M9 destinations stopped using canonical `HouseholdId`. Audit-v5 AV5-001/#606 now advances the living line from v33 to v34 because future tied M9 destinations stop using a globally ordinal household coupling rank and instead use the local policy above.

This is intentionally a **model-semantics change**. A v33 checkpoint must not silently resume under v34 because an unresolved future tied M9 journey could select a different destination. No checkpoint, event, travel-table or program schema bump is required for this repair: the already-versioned model-semantics identity provides the compatibility barrier, the existing event field continues to carry the authoritative tie key, and the destination-policy identifier remains bound into program identity.

The repair does not change M9 travel-cost equations, traversability, minimum-cost candidate construction, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream.
