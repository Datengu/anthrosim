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

Model semantics v35 replaces the destination policy with `m9/equal-cost-destination-local-household-spatial-equivalence-v4`.

The keyed draw now combines:

- the authoritative M9 destination-tie seed stored with the travel table;
- the household-local M9 coupling key;
- the trigger index;
- the number of exactly tied destinations.

The raw origin `CellId` is no longer an input to that draw. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

The draw selects a rank in a local destination-equivalence ordering derived from the physical/model-facing spatial state. The ordering is constructed in two stages.

First, M9 constructs an exact canonical frame over the supported grid-reflection group:

- identity;
- horizontal reflection;
- vertical reflection;
- horizontal-plus-vertical reflection.

For each frame, it serializes the reflected state in row-major order. The signature always includes:

- focal-region membership for every cell;
- authoritative `movement_cost` for every cell.

For spatial-host runs it additionally includes only model-facing world fields explicitly supplied by the declared M8 transforms:

- `water_access`, when a `WaterAccess` transform is declared;
- `base_productivity`, when a `BaseProductivity` transform is declared.

Residual synthetic world fields that were not declared as transformed spatial inputs are deliberately excluded from the spatial-host correspondence signature. Grid-coordinate labels and `CellId` values are not themselves scientific state in the signature.

The lexicographically minimal exact signature defines the canonical reflection frame. If more than one reflection produces exactly the same signature, the declared focal-region/world state has an exact automorphism under those reflections; all tied minimal frames are retained rather than manufacturing an arbitrary orientation.

Second, for each origin and each exactly tied destination, M9 maps the origin/destination pair through every retained canonical frame and derives a local equivalence key from the minimum reflected `(origin, destination, route_distance_edges)` tuple. Equal keys form one `destination_coupling_class`. Class order therefore follows the canonical physical representation when the state distinguishes an orientation. If exact automorphism makes alternatives scientifically indistinguishable, they remain in one class and canonical `CellId` is used only to serialize members inside that indistinguishable class.

This distinction is important: v35 does not claim that canonical numeric cell labels are empirical destination preferences. Numeric ordering is allowed only as a deterministic enumeration inside an exact equivalence class where the declared model state supplies no basis for distinguishing the alternatives. Across seeds/household coupling keys, those alternatives remain marginally exchangeable rather than acquiring a fixed lower-ID winner.

The focal-region mask is part of the canonical signature because it is itself model-facing spatial input. A symmetric world with an asymmetric focal region must not be treated as having a world automorphism that the actual M9 destination definition does not possess.

The v35 contract is reflection equivariance for the supported grid-reflection group, not a claim of invariance under arbitrary coordinate transformations, rotations between unequal grid dimensions, resampling, or changes to the declared scientific inputs.

## Locality, symmetry and interpretation

The v34 AV5-001 locality guarantee remains in force: adding or removing unrelated population that does not change the focal household or its travel inputs must not perturb the focal household's M9 tie coupling key or tied destination merely through global ordinal renumbering.

The AV4-007 label-invariance guarantee also remains in force: canonical household labels are not causal tie inputs.

Under v35, corresponding reflected runs that preserve the declared M9 scientific state map a tied destination to its corresponding reflected destination for the same authoritative tie seed, household coupling key and trigger. Exact spatial automorphisms retain marginal exchangeability rather than selecting a deterministic directional winner.

Non-tied minima are unchanged. Reachability, accumulated route cost, traversability, equal-minimum candidate construction and travel duration are unchanged by the v35 repair. Only the ambiguity coupling for exactly tied M9 destinations changes.

The keyed policy remains a neutral ambiguity resolver, not evidence that historical households chose destinations randomly. If evidence supports destination preference, that preference requires a separate explicit model.

## Replay, validation and observability

The destination-policy identifier, coupling context and per-candidate coupling class are bound into travel-table/program identity. The M9 travel-table schema advances from v4 to v5 so the derived ambiguity-coupling context/classes are explicit persisted state rather than hidden reconstruction assumptions.

Core program reconstruction uses the movement-plus-focal-region context. A spatial host derives its context from the authoritative spatial mechanism configuration and validates the persisted program against the same transformed-field flags. Downstream temporary-mobility observability reuses the coupling context persisted in the authoritative program when re-deriving expected travel state.

Authoritative tied-departure events continue to record the M9 household coupling key used so observability can independently replay the selected destination. A label-neutral compatibility resolver remains for callers that have only a canonical household ID; authoritative simulation execution does not use that ID as a causal tie key.

This choice is deterministic and platform independent and consumes no sequential RNG draw. Adding or removing a tied journey therefore cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative program, focal household state and trigger reproduces the same tie realization exactly.

Temporary-mobility observability v3 continues to report:

- the number of world origins with more than one equal-cost destination;
- the maximum equal-cost destination count;
- the number of started journeys whose origin was tied;
- `equalCostDestinationCount` for each started journey and origin-catchment row.

Researchers should inspect these fields before interpreting destination-level visitor concentration or resource pressure. A high tied-origin frequency means destination-level conclusions depend materially on the declared ambiguity policy even when total catchment participation is stable.

## Verification contract

Permanent regression coverage for v35 includes:

- the Audit-v5 #628/#629 3x1 horizontal-reflection failure class across 256 process seeds;
- the analogous vertical-reflection sweep;
- exact spatial symmetry with deterministic replay and both marginal alternatives exercised;
- an asymmetric focal-region reflection on an otherwise flat world, proving that focal-region geometry participates in the equivalence frame;
- AV5-001/#606 remote-founder locality;
- AV4-007 household-label invariance.

Protected CI and the applicable scientific/security gates remain required on the exact production PR head. Because AV5-004/#629 is P1, merge is not sufficient for final closure: independent post-merge adversarial re-verification is required under the scientific audit protocol.

## Provenance and compatibility boundary

The original equal-minimum preservation advanced `MODEL_SEMANTICS_ID` from v17 to v18. Audit-v4 AV4-007 remediation advanced the living line from v30 to v31 because future tied M9 destinations stopped using canonical `HouseholdId`. Audit-v5 AV5-001/#606 advanced the living line from v33 to v34 because future tied M9 destinations stopped using a globally ordinal household coupling rank and instead use the local household policy above.

Audit-v5 AV5-004/#629 advances the living line from v34 to v35 because future tied M9 destinations now use the reflection-equivariant spatial-equivalence policy rather than representation-dependent origin/candidate identity.

This is intentionally a **model-semantics change**. A v34 checkpoint must not silently resume under v35 because an unresolved future tied M9 journey can select a different destination. The derived M9 travel-table schema advances from v4 to v5 to persist the new coupling context and candidate equivalence classes. No checkpoint schema, temporary-event schema, or temporary-mobility program schema bump is required: the model-semantics identity is the continuation barrier, the nested travel-table schema versions the new derived state, and the existing event field continues to carry the authoritative household tie key.

The repair does not change M9 travel-cost equations, traversability, minimum-cost candidate construction, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream. It changes only the resolution of exactly equal M9 destination minima and the persisted metadata required to validate/replay that resolution.
