# M9.6 temporary-mobility integration audit

Status: M9.6 implementation in progress.

This note records the capability boundary identified before implementing M9.6. It is intentionally generic and does not encode a named archaeological interpretation or case-study rule.

## Existing authoritative foundations

M9.3-M9.5 already provide more of the M9.6 substrate than the roadmap summary alone implies:

- temporary-mobility transition events are authoritative and distinct from permanent `HouseholdMigration` events;
- transition records carry journey, household, focal-region and timing identity, with travel metadata where available;
- `TemporaryMobilityState` persists presence state, active journeys, the immutable temporary-mobility program, processed triggers, journey identity allocation and the duration-aware resource ledger;
- the core simulation state digest includes temporary-mobility state;
- core checkpoint/resume can continue an active temporary journey deterministically;
- run bundles already preserve authoritative events and the complete core checkpoint.

M9.6 therefore should extend these existing boundaries rather than create a second mobility state or analysis engine.

## Gaps identified for M9.6

The remaining capability gaps are:

1. **Transformed spatial execution.** The M8 transformed-landscape host must execute and resume the same authoritative M9 lifecycle as the core host. Before this slice it validated temporary state only to reject any active M9 configuration and serialized a disabled at-residence state of its own.
2. **Experiment identity and ordinary execution.** Ensemble and sweep definitions must be able to carry an immutable `TemporaryMobilityProgram`, execute it in both core and transformed-spatial runs, and reject retries whose M9 definition differs.
3. **Derived observability.** M8.5 event replay currently recognizes M9 event variants only to ignore them. It must derive temporary-use observables from preserved authoritative artifacts while keeping M4 permanent migration separate.
4. **Bundle/experiment validation.** Ordinary completed/paused bundle and retry validation must reconcile the M9 program/state and any new derived artifact schemas without weakening existing provenance checks.

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

This slice does not by itself complete M9.6. Ensemble/sweep program identity and M9 temporary-presence observability remain subsequent implementation work under issue #120.

## Scientific identity decision

This slice does not introduce a new temporary-mobility mechanism. It makes the existing M9 v1 semantics available through the transformed-landscape host using the same authoritative ordering, state and event vocabulary already represented by `anthrosim-model-semantics-v5`.

Accordingly, this slice does not by itself require another `MODEL_SEMANTICS_ID` change. It also does not change the M8 landscape-to-model transformation semantics, so `SPATIAL_MODEL_SEMANTICS_ID` remains unchanged. The package version remains on the current released line during ordinary M9 implementation.

That identity decision must be reviewed again if later M9.6 work changes authoritative simulated meaning rather than experiment packaging or downstream observability only.
