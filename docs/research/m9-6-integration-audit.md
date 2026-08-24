# M9.6 temporary-mobility integration audit

Status: M9.6 capability implementation complete; protected final validation is required before closing the umbrella issue.

This note records the capability boundary identified while implementing M9.6. It is intentionally generic and does not encode a named archaeological interpretation or case-study rule.

## Existing authoritative foundations

M9.3-M9.5 already provide more of the M9.6 substrate than the roadmap summary alone implies:

- temporary-mobility transition events are authoritative and distinct from permanent `HouseholdMigration` events;
- transition records carry journey, household, focal-region and timing identity, with travel metadata where available;
- `TemporaryMobilityState` persists presence state, active journeys, the immutable resolved temporary-mobility program, processed triggers, journey identity allocation and the duration-aware resource ledger;
- the core simulation state digest includes temporary-mobility state;
- core checkpoint/resume can continue an active temporary journey deterministically;
- run bundles already preserve authoritative events and the complete core checkpoint.

M9.6 therefore extends these existing boundaries rather than creating a second mobility state or analysis engine.

## Gaps identified for M9.6

The capability gaps identified at the start of M9.6 were:

1. **Transformed spatial execution.** The M8 transformed-landscape host had to execute and resume the same authoritative M9 lifecycle as the core host. Before the first slice it validated temporary state only to reject any active M9 configuration and serialized a disabled at-residence state of its own.
2. **Experiment identity and ordinary execution.** Ordinary experiment definitions need a world-independent M9 input that can be preserved across runs while each run derives its resolved travel table from its own authoritative world. Ensemble and sweep retry identity must reject changes to that definition.
3. **Derived observability.** M8.5 event replay currently recognizes M9 event variants only to ignore them. It must derive temporary-use observables from preserved authoritative artifacts while keeping M4 permanent migration separate.
4. **Bundle/experiment validation.** Ordinary completed/paused bundle and retry validation must reconcile the M9 definition/program/state and any new derived artifact schemas without weakening existing provenance checks.

## First implementation slice: transformed spatial host

The first M9.6 prerequisite extends `SpatialLandscapeSimulation` to own `TemporaryMobilityState` directly and to use the same ordering already established by the core M9 host:

1. process temporary boundaries before a fixed resource boundary;
2. settle duration-aware resource demand for the elapsed period;
3. reset/reconcile the duration ledger;
4. apply temporary transitions due on that boundary;
5. evaluate M4 permanent migration with temporary presence as an eligibility constraint;
6. run annual demography using the existing residence-based semantics;
7. validate and preserve the resulting temporary state in checkpoints and state digests.

Acceptance for this slice includes a transformed-landscape run whose temporary journey is active across an annual checkpoint. The resumed run must reproduce the uninterrupted authoritative population, temporary-mobility state, resources, migration state, RNG positions, events, metrics and final state digest.

This slice was merged in PR #133.

## Second implementation slice: immutable experiment definition

A resolved M9.4 travel table is **not** a world-independent experiment input. The frozen M9.4 contract derives that table once per `(world, focal region, travel model)`. Synthetic worlds can have different model-facing movement costs under different seeds, so copying one resolved `TemporaryMobilityProgram` across an ensemble would silently apply routing derived from the wrong world.

M9.6 therefore introduces a versioned `TemporaryMobilityConfig` containing only the world-independent assumptions:

- one immutable `FocalRegion`;
- one exogenous `TemporaryMobilitySchedule`;
- one explicit `TemporaryTravelModel`.

`ExperimentConfig` can carry this definition. Each core or transformed-spatial run then derives its own authoritative `TemporaryMobilityProgram` from that run's reconstructed/transformed `World`. The resolved program remains persisted inside `TemporaryMobilityState`, so checkpoint resume can rederive the expected program from the immutable experiment definition and fail closed if the two no longer match.

The lower-level explicit-program constructors remain available for isolated lifecycle tests, but supplying both an ordinary experiment definition and an explicit resolved program is rejected as ambiguous.

Evidence provenance is also fail-closed at this boundary. If a focal region claims `LandscapeMask` provenance, its `evidence_input_id` must identify an external input in the experiment's attached evidence catalogue. A synthetic focal region does not require a catalogue.

This separation means ensemble and sweep provenance preserve the world-independent definition, not a seed-specific resolved table. Their existing identities already serialize full run/settings definitions, so subsequent CLI wiring can reuse the ordinary provenance machinery rather than introducing an M9-specific identity system.

This slice was merged in PR #135.

## Third implementation slice: ordinary runner integration

The ordinary user-facing runner interface uses one versioned file input rather than distributing M9 assumptions across independent CLI flags:

- `anthrosim run --temporary-mobility <json>`;
- `anthrosim ensemble --temporary-mobility <json>`;
- `anthrosim sweep --temporary-mobility <json>`;
- `anthrosim-landscape run --temporary-mobility <json>`.

The file contains the same world-independent `TemporaryMobilityConfig` that is embedded in `ExperimentConfig`. `EnsembleRunSettings` therefore carries the definition directly, and the existing immutable experiment/sweep identity machinery automatically includes it. Changing the M9 definition changes retry identity rather than being treated as an out-of-band runtime option.

Ensemble execution validates the definition before creating run outputs and then lets each seed derive its own resolved travel program from its stored authoritative world. Acceptance coverage reads completed two-seed run bundles back, rederives the program separately from each `world.json`, and requires equality with the program persisted in that run's checkpoint. Sweep coverage requires every Cartesian point to preserve the definition and verifies that changing it changes the sweep identity.

The dedicated landscape runner exposes the same file input so evidence-bound single runs do not require an ensemble/sweep wrapper merely to exercise M9.

## Fourth implementation slice: deterministic temporary-mobility observability

PR #137 establishes `TemporaryMobilityObservabilityReport` as a separate downstream artifact rather than changing M8 spatial-observability meaning. It replays authoritative event history household-by-household and reconciles the terminal result against the checkpoint.

The v1 report provides:

- exact persistent-residence and physical-presence person-day accounting;
- separate at-residence, visiting, outbound-transit and return-transit categories;
- trigger outcomes and explicit non-start reasons, including unreachable households;
- starts, arrivals, return departures, completions and journeys active at the report boundary;
- visit-duration distributions, visitor household/person-days and peak/mean visitor presence;
- travel duration/cost and origin catchment;
- derived route edge distance only when deterministic minimum-cost recomputation agrees with the authoritative M9.4 cost and destination.

Transit remains deliberately non-spatial because M9 v1 does not contain an authoritative per-day en-route cell. The resource subsystem's home-provisioning proxy for transit therefore does not become a claim of physical residence occupancy.

Focused acceptance proves exact physical person-day partitioning and exact report equality between uninterrupted execution and execution resumed from a checkpoint with an active temporary journey. Resume-lineage provenance may differ; the scientific state and derived report must not.

## Fifth implementation slice: report CLI, bundle validation and Explorer

PR #138 integrates the established report into ordinary downstream workflows:

- `anthrosim-temporary-observability run` derives or verifies one run;
- `anthrosim-temporary-observability tree` discovers and processes M9 runs beneath experiment/sweep roots;
- completed bundle validation treats `temporary-observability.json` as an optional derived artifact and requires exact deterministic regeneration when it is present;
- the existing deterministic bundle packer includes the report automatically through the normal optional-artifact path;
- a paused-run regression proves a report can be derived and verified when only `resume-start-population.json` is preserved, by validating that artifact and deterministically reconstructing day-zero founders from immutable experiment identity and the authoritative world;
- the read-only Explorer optionally surfaces the derived M9 summary, identities and authoritative temporary-event family while leaving its residence map semantics unchanged.

Explorer support uses the machine-readable report rather than independently re-deriving scientific quantities in JavaScript. It validates basic provenance and exact person-day identities for display, while Rust bundle/report validation remains the authority for deterministic regeneration.

## M9.6 acceptance reconciliation

The umbrella acceptance criteria are satisfied by the combined M9.3-M9.6 state and the M9.6 slices above:

- temporary transition events are separately versioned and remain distinct from `HouseholdMigration`;
- active journeys, resolved program, resource ledger and RNG/state continuity survive checkpoints and state digests;
- core and transformed-spatial uninterrupted/resumed execution reconcile exactly at the scientific-state boundary;
- ordinary run, ensemble and sweep identity preserve the immutable world-independent M9 definition, with routing re-derived per world;
- completed bundles validate the optional derived report and reject tampering;
- paused runs regenerate/verify the report from preserved provenance;
- resident/visitor/transit observables, journey outcomes, duration, peaks/means, travel burden, catchment and non-participation are machine-readable;
- transit is not assigned to arbitrary cells;
- M4 permanent-migration observables remain a separate category;
- read-only Explorer support is downstream from the established machine-readable semantics.

M9.6 therefore adds inspectability and experiment integration without creating an archaeological observation/preservation model or assigning a social motive to temporary aggregation.

## Scientific identity decision

The M9.6 slices do not introduce a new temporary-mobility mechanism. They expose the existing M9 v1 semantics through the transformed-landscape host, immutable experiment/provenance machinery, ordinary runners and downstream derived observability using the same authoritative ordering, travel model, state and event vocabulary already represented by `anthrosim-model-semantics-v5`.

Accordingly, M9.6 does not require another `MODEL_SEMANTICS_ID` change. It also does not change the M8 landscape-to-model transformation semantics, so `SPATIAL_MODEL_SEMANTICS_ID` remains unchanged. The package version remains on the current released line during ordinary M9 implementation. M9.7 must review scientific identity again if its benchmark work reveals an authoritative semantic change rather than merely exercising the completed M9 capability.
