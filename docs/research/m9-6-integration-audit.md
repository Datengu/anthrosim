# M9.6 temporary-mobility integration audit

Status: M9.6 implementation in progress.

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

This separation means ensemble and sweep provenance should preserve the world-independent definition, not a seed-specific resolved table. Their existing identities already serialize full run/settings definitions, so subsequent CLI wiring can reuse the ordinary provenance machinery rather than introducing an M9-specific identity system.

## Remaining work after the second slice

M9.6 still requires:

- user-facing Run/Ensemble/Sweep loading of the versioned temporary-mobility definition;
- deterministic temporary-presence observability derived from authoritative event/checkpoint artifacts;
- bundle and derived-analysis integration for those observables;
- end-to-end M9.6 acceptance demonstrating ordinary reproducible experiment execution and replay.

## Scientific identity decision

The first two M9.6 slices do not introduce a new temporary-mobility mechanism. They expose the existing M9 v1 semantics through the transformed-landscape host and immutable experiment/provenance machinery using the same authoritative ordering, travel model, state and event vocabulary already represented by `anthrosim-model-semantics-v5`.

Accordingly, these slices do not by themselves require another `MODEL_SEMANTICS_ID` change. They also do not change the M8 landscape-to-model transformation semantics, so `SPATIAL_MODEL_SEMANTICS_ID` remains unchanged. The package version remains on the current released line during ordinary M9 implementation.

That identity decision must be reviewed again if later M9.6 work changes authoritative simulated meaning rather than experiment packaging, validation or downstream observability only.
