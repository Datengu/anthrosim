# M6 local explorer v0.1

## Purpose

The M6 explorer is a **read-only research-inspection tool** for completed or paused AnthroSim run bundles. It exists to make the M5 artifact set easier to inspect without moving visualization logic into the authoritative simulation engine.

The explorer does not run demographic, resource or migration transitions and has no write path back into a simulation. The Rust workspace remains `anthrosim-core` + `anthrosim-cli`; M6 adds no Cargo dependency and does not participate in the headless hot loop.

## Run-bundle contract

M6 opens the six M5 JSON artifacts:

- `manifest.json`;
- `world.json`;
- `initial-population.json`;
- `events.json`;
- `metrics.json`;
- `checkpoint.json`.

Before display, the explorer checks the manifest's declared artifact schema versions against the files actually loaded and reconciles event/metric totals with the terminal manifest. Unknown or mismatched required schemas are rejected instead of being silently interpreted.

JavaScript cannot exactly represent every Rust `u64` as a `Number`. The explorer therefore parses integers outside JavaScript's safe integer range as exact decimal strings. IDs, days and ordinary counters that fit safely remain numeric. This prevents state digests, RNG-related weights or future large counters from being silently rounded merely because they are displayed in a browser.

## Provenance classes in the UI

M6 keeps three display classes visibly distinct.

### Authoritative

These values are directly serialized engine facts, for example:

- `world.json` cell fields;
- event records in `events.json`;
- final per-cell resource stock and final person condition in `checkpoint.json`;
- final manifest summaries.

An authoritative event says what the implemented simulation did. It does not establish a real historical fact or human motive.

### Derived

Annual/terminal `metrics.json` snapshots are labelled `derived`, matching M5. Summary cards on the timeline display these recorded derived values rather than recomputing replacement values when a snapshot exists.

### Reconstructed display

Historical living-cell occupancy is not separately serialized at every annual boundary. M6 reconstructs that view from:

1. founder locations in `initial-population.json`;
2. authoritative birth events;
3. authoritative death events;
4. authoritative household-migration events in sequence order.

This reconstruction is deterministic and CI checks that its terminal person locations, person-record count, living-population count and occupied-cell count agree with the final checkpoint/manifest. It is still labelled as a reconstructed/derived display so the UI does not pretend a historical occupancy snapshot was directly serialized when it was not.

## What M6 does not infer

The explorer must not fill gaps in M5's recording model with invented historical state.

In particular:

- per-cell **dynamic food stock at arbitrary historical years** is not available in M5; the map can show immutable baseline productivity/water/movement fields at any selected time and authoritative **final** dynamic food stock from the checkpoint, but it does not interpolate a historical stock surface;
- arbitrary historical **individual condition** is not available for every person at every annual boundary; M6 shows condition when it is authoritative in a death event or the final checkpoint and otherwise states that it was not serialized at that boundary;
- labels such as `collapse`, `recovery`, `migration wave`, `success`, `failure`, `settlement` or archaeological interpretations are not promoted into authoritative state.

If future milestones need those views, the recording model should be extended explicitly rather than having the explorer guess them.

## Views

### Timeline

The timeline exposes the initial boundary plus recorded M5 annual/terminal snapshots. Population, births, deaths, occupied cells, completed migrations and unmet need come from the selected derived snapshot. A raw-data disclosure shows the exact snapshot object and its source path.

### Map

Available overlays are:

- reconstructed living population at the selected boundary;
- immutable baseline productivity;
- immutable water access;
- immutable movement cost;
- final checkpoint food stock (explicitly labelled final-only when viewing an earlier boundary).

Clicking a cell opens its exported terrain values, reconstructed residents and relevant authoritative events.

### Entities

The inspector supports stable M2/M3/M4 identities:

- cells;
- households;
- people.

Person inspection exposes status, birth information, household, reconstructed location and available condition provenance. Parent/child navigation is derived from persistent parent IDs already represented by M2.

### Event browser

The event browser exposes all authoritative M5 events with type filters and optional `person:<id>`, `household:<id>` or `cell:<id>` filtering. Event rows link back to the relevant entity and can reveal the raw event record. This gives a direct path from an aggregate observation to the state-transition evidence supporting it.

## Local server and security boundary

`scripts/serve-explorer.py` uses only the Python standard library. By default it binds to `127.0.0.1` and serves one explicitly selected run directory plus the fixed explorer assets.

The server:

- exposes only the six expected run artifact filenames;
- exposes only the fixed M6 static assets;
- provides GET/HEAD only;
- rejects POST/PUT/DELETE;
- performs no directory listing;
- provides no API that mutates the run directory;
- sends a restrictive Content Security Policy.

CI hashes the generated sample run before and after the server smoke test to ensure the explorer did not alter the research artifacts.

## Verification

M6 has two distinct verification layers.

Pure JavaScript tests cover:

- schema reconciliation and rejection of incompatible bundles;
- lossless large-integer parsing;
- chronological birth/migration/death reconstruction;
- genealogy traversal;
- entity/event filtering;
- population/environment/final-resource map-source separation;
- annual metric lookup.

CI then generates a real versioned M5 run bundle with the release Rust binary and validates that:

- event counts reconcile with the manifest;
- terminal metrics reconcile with the manifest;
- reconstructed person count equals terminal person records;
- reconstructed living population and occupied-cell count equal terminal summaries;
- every reconstructed final person location/household agrees with the final checkpoint;
- summed final cell resource stock agrees with the resource summary;
- the loopback server serves the selected bundle byte-for-byte and rejects writes;
- the bundle hashes are unchanged after explorer access.

The existing Rust headless benchmarks run unchanged. Explorer code is downstream and cannot change those benchmarked binaries unless a separate core/CLI change is made.

## Scientific status

M6 improves **inspectability**, not empirical validity. A clearer map or genealogy view does not make the underlying `synthetic_validation_v1` demographic, resource or migration parameters anthropologically calibrated. The same model-validation limits documented for M2–M4 remain in force.
