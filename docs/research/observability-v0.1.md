# M5 observability and persistence model

AnthroSim M5 adds scientific observability and deterministic persistence. It does **not** add a new anthropological mechanism. The purpose is to make M1-M4 runs inspectable, resumable, reproducible and suitable for later analysis without treating summaries or interpretations as authoritative simulation facts.

## Evidence classes

AnthroSim now distinguishes three layers explicitly.

1. **Authoritative events** are state-changing lifecycle facts emitted by the simulation engine. M5 v1 records births, deaths and completed household migrations. Each event has a stable sequence number, simulated day, schema version through its containing event log, and `authoritative` provenance. Deaths distinguish demographic mortality from resource-scarcity mortality. Migration events include the selected destination and the utility/choice information used at the decision boundary.
2. **Derived metric snapshots** are recalculable summaries of authoritative state. M5 v1 emits snapshots at completed annual boundaries plus the terminal boundary when needed. Metrics include population, resource and migration aggregates and are explicitly marked `derived`.
3. **Interpretation** is outside the authoritative engine artifact. A later explorer or analysis workflow may label a pattern as a collapse, migration wave, bottleneck, recovery or other interpretation, but those labels must not be silently written back as simulated ground truth.

This distinction is important for archaeological use: the engine may know a person died from its implemented scarcity pathway, while an archaeological interpretation would have access only to a much narrower evidence model in a future milestone.

## Event schema

`events.json` contains a versioned `EventLog` (`schemaVersion = 1`). Event ordering is deterministic. `sequence` is one-based and represents authoritative emission order; `day` is the simulation day on which the event completed.

M5 v1 event types are:

- `birth`: stable newborn ID, both simulated biological parent IDs, household, cell and reproductive-sex state;
- `death`: stable person ID, household, cell, implemented cause, condition and mortality probability used by that pathway;
- `householdMigration`: household, people moved, origin/destination, distance, pressure, origin/destination utility decompositions, best locally known candidate, stochastic-choice weights/draw and per-person travel condition cost.

The event schema reports implemented causal state. It does not claim that the synthetic mortality probabilities, resource condition model or migration utilities are empirically calibrated.

## Derived metrics

`metrics.json` contains a versioned `MetricSeries` (`schemaVersion = 1`). The current cadence is `annual_boundary_plus_terminal`.

Each snapshot records:

- population counts, cumulative births/deaths, occupied-cell count and living condition;
- cumulative resource regeneration/harvest/unmet need, food stock and scarcity deaths;
- cumulative migration evaluations/moves/distance and spatial occupancy effect;
- a composite state digest for the same simulation boundary.

Snapshots intentionally use compact aggregates rather than duplicating the full decision-trace history. The final snapshot is regression-tested against the final authoritative population/resource/migration summaries.

## Deterministic checkpoints

`checkpoint.json` contains a versioned `SimulationCheckpoint` (`schemaVersion = 1`). M5 v1 resumable checkpoints are supported only at completed annual boundaries. This restriction is deliberate: an annual boundary has an unambiguous position in the resource -> migration -> demography schedule and avoids serializing a partially completed decision boundary.

A checkpoint stores:

- full experiment configuration;
- simulation time and completed-year count;
- full persistent population state;
- full dynamic resource state;
- persistent migration counters and retained decision traces (scratch buffers are reconstructed);
- positions of all seven named ChaCha8 random streams;
- event and metric history accumulated to the checkpoint;
- world and composite state digests.

The synthetic world itself is deterministically regenerated from configuration + master seed on resume. Its digest must match the checkpoint before execution can continue.

### RNG continuation

AnthroSim does not serialize opaque RNG implementation memory. Each named ChaCha8 stream is regenerated from the experiment master seed and its stable stream label, then restored to its recorded 128-bit word position. The seven M5 positions are:

- `demography/mortality`
- `demography/fertility`
- `demography/parentage`
- `demography/newborn_sex`
- `resources/scarcity_mortality`
- `migration/choice`
- `migration/uncertainty`

Regression tests require uninterrupted execution and checkpoint-resumed execution to produce identical authoritative final state, event history, metric series and state digests. Provenance-bearing manifest/checkpoint artifacts may differ because a resumed execution records explicit `resumeLineage` boundaries rather than pretending it was uninterrupted.

## Controlled offline run layout

`anthrosim run --run-dir <directory>` writes a self-contained analysis bundle:

```text
<run-dir>/
  manifest.json
  world.json
  initial-population.json
  events.json
  metrics.json
  checkpoint.json
```

The final checkpoint contains final persistent population/resource/migration state as well as event/metric history. `world.json` and `initial-population.json` preserve the initial authoritative inputs needed for offline causal inspection. No live database is required.

A resumable intermediate run can be written with `--checkpoint-year N`. It writes the deterministic annual-boundary checkpoint and accumulated events/metrics. It can later be continued with:

```text
anthrosim resume --checkpoint <run-dir>/checkpoint.json --run-dir <run-dir>
```

When the original run directory is reused, its `initial-population.json` founder artifact is retained. When resuming into a new directory, AnthroSim retains `resume-start-population.json` as the checkpoint-boundary population **and** deterministically materializes the true original `initial-population.json` from immutable initialization provenance. The two files describe different moments: full-history replay always starts from `initial-population.json`.

## Compatibility and provenance

The run manifest now reports the current schema versions for the manifest, events, metrics, checkpoint, world, population, resources and migration artifacts. Checkpoint loading rejects incompatible checkpoint/model/artifact versions, invalid annual boundaries, world-digest mismatch and state-digest mismatch.

Ordinary Git builds automatically embed source provenance in `gitCommit`: a clean tracked tree records the exact commit SHA and a tracked dirty tree records `<sha>-dirty-<working-tree-digest>`. Builds without resolvable Git metadata record `null`, while controlled build environments may override the value with `ANTHROSIM_GIT_COMMIT`. Formal versioned research sweeps preflight the exact supplied binary and reject missing or automatic dirty identities; see `docs/source-provenance.md`.

## Scope and limitations

M5 v1 is an engineering/research-observability milestone, not empirical validation. In particular:

- only births, deaths and completed migration moves are authoritative events; resource-period allocations and every rejected migration alternative are not yet emitted as first-class events;
- only annual-boundary checkpoints are resumable;
- JSON prioritizes inspectability and stable schemas rather than minimal storage size;
- event logs grow with state-changing activity and have not yet been designed for very large multi-million-agent archival workloads;
- the composite digest is a deterministic regression fingerprint, not a cryptographic integrity primitive;
- causal inspection can explain outcomes in terms of implemented mechanisms, but it does not turn synthetic-validation assumptions into claims about real human populations.

These limitations should be preserved when results are interpreted or compared with archaeological evidence.
