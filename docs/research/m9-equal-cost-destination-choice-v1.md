# M9 equal-cost destination choice semantics v1

## Scientific problem

M9.4 can find more than one focal-region destination with exactly the same minimum accumulated movement cost from an origin. Before model semantics v18, the routing table collapsed such a tie to the lower authoritative `CellId`. Because `CellId` follows grid storage order, that deterministic fallback created a causal spatial preference unrelated to the travel model.

## Equal-minimum preservation

M9.4 preserves the complete canonical set of exactly equal minimum-cost destinations for every origin. The set is sorted only for stable serialization; ordering is not intended to encode empirical preference. Each candidate also retains the minimum route-edge count among paths achieving that same minimum cost.

Model semantics v31 replaced the original canonical-`HouseholdId` tie component with `m9/equal-cost-destination-scientific-coupling-v2`, using the minimum persistent person `stochastic_coupling_rank` among a household's living members. Audit-v5 AV5-001/#606 demonstrated that those persistent ranks are global ordinals: adding an unrelated, unreachable founder can renumber an otherwise unchanged focal household and therefore alter its M9 tie destination.

Model semantics v34 repaired that nonlocal population coupling with:

- destination policy `m9/equal-cost-destination-local-household-coupling-v3`;
- household coupling policy `m9/household-local-demographic-equivalence-v1`.

For authoritative execution, the M9 household coupling key is derived only from the household's current living-member multiset of stable demographic attributes used by this null ambiguity policy:

- exact `birth_day`;
- reproductive sex.

The member tuples are sorted before hashing. `PersonId`, packed-record order, `HouseholdId`, persistent global stochastic-coupling rank, residence, condition, and unrelated population records are not inputs to this M9 household key. Exact demographic duplicate households may therefore share the same ambiguity realization; the policy does not manufacture a scientifically unsupported distinction merely to obtain globally unique keys.

Audit-v5 AV5-004/#629 then demonstrated that v34 still admitted a representation-dependent spatial result: the numeric origin/candidate representation entered the keyed destination mapping, so a pure reflection of the same physical M9 tie could select a non-reflected destination for the same seed and household state.

## Model semantics v35: spatially equivariant ambiguity coupling

Model semantics v35 replaced the destination policy with `m9/equal-cost-destination-local-household-spatial-equivalence-v4`.

The keyed draw combines:

- the authoritative M9 destination-tie seed stored with the travel table;
- the household-local M9 coupling key;
- the trigger index;
- the number of exactly tied destinations.

The raw origin `CellId` is not an input to that draw. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

The draw selects a rank in a destination-equivalence ordering derived from physical/model-facing spatial state. Under v35 that ordering used an exact canonical frame over the supported whole-grid reflection group: identity, horizontal reflection, vertical reflection, and horizontal-plus-vertical reflection. Its signature contained focal-region membership and authoritative `movement_cost` for every world cell, plus only explicitly transformed `water_access` and/or `base_productivity` fields for spatial-host runs. Residual synthetic fields were excluded.

The lexicographically minimal exact signature defined the canonical reflection frame. If multiple reflections produced the same signature, the declared state had an exact automorphism and all tied minimal frames were retained. For each origin and tied destination, v35 mapped the pair through every retained frame and derived a destination equivalence class from the minimum reflected origin/destination/route-distance tuple. Canonical `CellId` was permitted only to serialize members inside an exact equivalence class, preserving marginal exchangeability rather than creating a directional winner.

This repaired the Audit-v5 reflection failure and established the supported reflection-equivariance contract, but Audit-v6 AV6-006/#711 demonstrated a remaining locality defect: because the v35 canonical frame serialized the complete authoritative world, appending an unreachable and explicitly impassable cell outside an otherwise unchanged local travel problem could change the canonical frame and therefore the selected tied destination.

## Model semantics v38: reachable-component-local ambiguity coupling

Model semantics v38 replaces the destination policy with `m9/equal-cost-destination-local-household-spatial-equivalence-v5` and advances the derived M9 travel-table schema from v5 to v6.

The keyed draw itself is unchanged: tie seed, household-local coupling key, trigger index and tied-candidate count still determine the target rank. The change is how tied physical alternatives are assigned to scientific equivalence classes.

For each reachable origin, M9 now constructs its reflection-canonical frame only from the origin's **reachable traversable connected component**. Reachability is the authoritative M9.4 minimum-cost routing result under the declared traversability ceiling. The component is normalized to its own axis-aligned bounding box before reflection canonicalization, so absolute grid position, unrelated world width/height and cells outside that reachable component are not tie-coupling inputs.

Within the local bounding box, the canonical signature records:

- whether a slot belongs to the origin's reachable component;
- focal-region membership for component cells;
- authoritative `movement_cost` for component cells;
- explicitly transformed `water_access` and/or `base_productivity` when enabled by the persisted coupling context.

Slots inside the bounding box but outside the reachable component are encoded only as absent structure; their model-facing values are ignored. Cells outside the component bounding box are not serialized at all. The same identity/horizontal/vertical/both reflection group is evaluated over this normalized local component.

Destination coupling keys use the reflected **local coordinates** of the origin and destination within that component frame together with route distance. They do not use global row-major `CellId`, absolute world coordinates, a world digest, or a world-extent ordinal. If multiple local reflections are exact automorphisms, the alternatives remain one exchangeable class and canonical `CellId` is still used only for deterministic serialization within that scientifically indistinguishable class.

This makes causally isolated impassable or disconnected padding invisible to an unchanged local tie. It does **not** erase genuine finite-boundary sensitivity: if an extent change adds a traversable route, changes reachability, changes the origin's connected component, changes a route cost, alters the candidate set, or changes declared model-facing state inside the reachable component, then the scientific input has changed and the resulting M9 destination is allowed to change.

The v35 reflection-equivariance guarantee remains in force under the local frame. The local component and its bounding-box coordinates transform with the reflected physical problem, so corresponding reflected runs retain corresponding destination outcomes for the same authoritative tie seed, household coupling key and trigger.

## Locality, symmetry and interpretation

The v34 AV5-001 locality guarantee remains in force: adding or removing unrelated population that does not change the focal household or its travel inputs must not perturb the focal household's M9 tie coupling key or tied destination merely through global ordinal renumbering.

The AV4-007 label-invariance guarantee also remains in force: canonical household labels are not causal tie inputs.

Under v38, corresponding reflected runs that preserve the declared M9 scientific state map a tied destination to its corresponding reflected destination for the same authoritative tie seed, household coupling key and trigger. Exact spatial automorphisms retain marginal exchangeability rather than selecting a deterministic directional winner. Causally isolated unreachable/impassable padding also cannot perturb the tie realization.

Non-tied minima are unchanged. Reachability, accumulated route cost, traversability, equal-minimum candidate construction and travel duration are unchanged by the v38 repair. Only the spatial equivalence classification used to resolve exactly tied M9 destinations changes.

The keyed policy remains a neutral ambiguity resolver, not evidence that historical households chose destinations randomly. If evidence supports destination preference, that preference requires a separate explicit model.

## Replay, validation and observability

The destination-policy identifier, coupling context and per-candidate coupling class are bound into travel-table/program identity. The M9 travel-table schema is v6 under model semantics v38 because the persisted candidate coupling classes now have reachable-component-local meaning. Historical v35 tables remain schema v5 and are not reinterpreted as v38 state.

Core program reconstruction uses the movement-plus-focal-region context. A spatial host derives its context from the authoritative spatial mechanism configuration and validates the persisted program against the same transformed-field flags. Validation recomputes per-origin reachability from persisted accumulated-cost state, reconstructs the same local connected component and canonical frame, and requires the persisted candidate classes to match. Downstream temporary-mobility observability reuses the coupling context persisted in the authoritative program when re-deriving expected travel state.

Authoritative tied-departure events continue to record the M9 household coupling key used so observability can independently replay the selected destination. A label-neutral compatibility resolver remains for callers that have only a canonical household ID; authoritative simulation execution does not use that ID as a causal tie key.

This choice is deterministic and platform independent and consumes no sequential RNG draw. Adding or removing a tied journey therefore cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative program, focal household state and trigger reproduces the same tie realization exactly.

Temporary-mobility observability v3 continues to report:

- the number of world origins with more than one equal-cost destination;
- the maximum equal-cost destination count;
- the number of started journeys whose origin was tied;
- `equalCostDestinationCount` for each started journey and origin-catchment row.

Researchers should inspect these fields before interpreting destination-level visitor concentration or resource pressure. A high tied-origin frequency means destination-level conclusions depend materially on the declared ambiguity policy even when total catchment participation is stable.

## Verification contract

Permanent regression coverage for the living v38 policy includes:

- the Audit-v6 #710/#711 impassable-padding locality adversary across tie seeds `0..128`;
- the Audit-v5 #628/#629 3x1 horizontal-reflection failure class across 256 process seeds;
- the analogous vertical-reflection sweep;
- exact spatial symmetry with deterministic replay and both marginal alternatives exercised;
- an asymmetric focal-region reflection on an otherwise flat world;
- AV5-001/#606 remote-founder locality;
- AV4-007 household-label invariance;
- preserved unique-destination route costs and candidate construction;
- finite-boundary controls where newly reachable paths are expected to change results;
- deterministic replay, checkpoint/resume, M9 history validation and cross-platform determinism.

Protected CI and the applicable scientific/security gates remain required on the exact production PR head. Because AV6-006/#711 is P1, production merge is not sufficient for closure: #711 remains open until an independent evidence-only derivative re-runs the preserved #710 adversarial contract against the exact merged production lineage and passes the required matrix.

## Provenance and compatibility boundary

The original equal-minimum preservation advanced `MODEL_SEMANTICS_ID` from v17 to v18. Audit-v4 AV4-007 remediation advanced the living line from v30 to v31 because future tied M9 destinations stopped using canonical `HouseholdId`. Audit-v5 AV5-001/#606 advanced the living line from v33 to v34 because future tied M9 destinations stopped using a globally ordinal household coupling rank. Audit-v5 AV5-004/#629 advanced the living line from v34 to v35 because future tied M9 destinations adopted reflection-equivariant spatial-equivalence coupling.

Audit-v6 AV6-006/#711 advances the living line from v37 to **v38** because future tied M9 destinations now ignore causally isolated world padding and use a reachable-component-local reflection frame. A v37 checkpoint must not silently resume under v38 because an unresolved future tied journey can select a different destination. The nested M9 travel-table schema advances from v5 to v6 to distinguish the changed persisted coupling-class semantics. No checkpoint schema, temporary-event schema, or temporary-mobility program schema bump is required: the model-semantics identity is the continuation barrier, the travel-table schema versions the derived tie state, and existing tied-departure events continue to carry the authoritative household tie key.

The repair does not change M9 travel-cost equations, traversability, minimum-cost candidate construction, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream. It changes only the scientific equivalence classification used to resolve exactly equal M9 destination minima and the persisted metadata required to validate/replay that resolution.
