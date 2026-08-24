# M6 local explorer v0.1

## Purpose

The M6 explorer is a **read-only research-inspection tool** for completed or paused AnthroSim run bundles. It exists to make the M5 artifact set easier to inspect without moving visualization logic into the authoritative simulation engine.

The explorer does not run demographic, resource or migration transitions and has no write path back into a simulation. The Rust workspace remains `anthrosim-core` + `anthrosim-cli`; M6 adds no Cargo dependency and does not participate in the headless hot loop.

## Run-bundle contract

A completed M5 run contains six JSON artifacts:

- `manifest.json`;
- `world.json`;
- `initial-population.json`;
- `events.json`;
- `metrics.json`;
- `checkpoint.json`.

A deliberately paused `--checkpoint-year` run contains the same artifacts **except `manifest.json`**, because it has not completed. M6 therefore requires world, the day-zero founder population, events, metrics and checkpoint, while treating the manifest as optional. For a paused bundle the checkpoint is the authoritative current boundary; the explorer does not fabricate a completed manifest.

A current new-directory resume contains both `initial-population.json` and `resume-start-population.json`. The former is deterministically materialized as the true original day-zero founder state and remains the only baseline for full-history event replay. The latter records population state at the resume boundary for provenance/inspection and must never be substituted for founders. In-place resumes retain the original `initial-population.json`.

For completed runs, the explorer checks the manifest's declared artifact schema versions against the files actually loaded and reconciles event/metric totals with the terminal manifest. For paused runs, it checks the experiment and state schemas carried by the checkpoint, requires separately written event/metric history to agree with the history embedded in the checkpoint, and reconciles counts against checkpoint state. Unknown or mismatched required schemas are rejected instead of being silently interpreted.

JavaScript cannot exactly represent every Rust `u64` as a `Number`. The explorer therefore parses integers outside JavaScript's safe integer range as exact decimal strings. IDs, days and ordinary counters that fit safely remain numeric. This prevents state digests, RNG-related weights or future large counters from being silently rounded merely because they are displayed in a browser. A numeric visualization refuses a value that cannot be represented exactly rather than approximating it.

## Provenance classes in the UI

M6 keeps three display classes visibly distinct.

### Authoritative

These values are directly serialized engine facts, for example:

- `world.json` cell fields;
- event records in `events.json`;
- per-cell resource stock and person condition at the serialized checkpoint boundary;
- completed manifest summaries when a manifest exists;
- checkpoint state itself for a paused run.

An authoritative event says what the implemented simulation did. It does not establish a real historical fact or human motive.

### Derived

Annual/terminal `metrics.json` snapshots are labelled `derived`, matching M5. Summary cards on the timeline display these recorded derived values rather than recomputing replacement values when a snapshot exists.

### Reconstructed display

Historical living-cell occupancy is not separately serialized at every annual boundary. M6 reconstructs that view from:

1. founder locations in `initial-population.json`;
2. authoritative birth events;
3. authoritative death events;
4. authoritative household-migration events in sequence order.

This reconstruction is deterministic and CI checks that its boundary person locations, person-record count, living-population count and occupied-cell count agree with checkpoint/summary state. It is still labelled as a reconstructed display so the UI does not pretend a historical occupancy snapshot was directly serialized when it was not.

## What M6 does not infer

The explorer must not fill gaps in M5's recording model with invented historical state.

In particular:

- per-cell **dynamic food stock at arbitrary historical years** is not available in M5; the map can show immutable baseline productivity/water/movement fields at any selected time and authoritative dynamic food stock at the checkpoint boundary, but it does not interpolate a historical stock surface;
- arbitrary historical **individual condition** is not available for every person at every annual boundary; M6 shows condition when it is authoritative in a death event or the checkpoint and otherwise states that it was not serialized at that boundary;
- labels such as `collapse`, `recovery`, `migration wave`, `success`, `failure`, `settlement` or archaeological interpretations are not promoted into authoritative state.

If future milestones need those views, the recording model should be extended explicitly rather than having the explorer guess them.

## Views

### Timeline

The timeline exposes the initial boundary plus recorded M5 annual/terminal snapshots through the completed or paused boundary. Population, births, deaths, occupied cells, completed migrations and unmet need come from the selected derived snapshot. A raw-data disclosure shows the exact snapshot object and its source path.

### Map

Available overlays are:

- reconstructed living population at the selected boundary;
- immutable baseline productivity;
- immutable water access;
- immutable movement cost;
- checkpoint food stock (explicitly labelled checkpoint-only when viewing an earlier boundary).

Clicking a cell opens its exported terrain values, reconstructed residents and relevant authoritative events.

### Entities

The inspector supports stable M2/M3/M4 identities:

- cells;
- households;
- people.

Person inspection exposes status, birth information, household, reconstructed location and available condition provenance. Parent/child navigation is derived from persistent parent IDs already represented by M2.

### Event browser

The event browser exposes all authoritative M5 events with type filters and optional `person:<id>`, `household:<id>` or `cell:<id>` filtering. Event rows link back to the relevant entity and can reveal the raw event record. This gives a direct path from an aggregate observation to the state-transition evidence supporting it.

## Sharing a completed run as one file

The multi-file run directory remains the canonical scientific bundle, but a completed run can be packaged into one deterministic ZIP for uploading, sharing or archiving:

```text
cargo run --release -p anthrosim-cli --bin anthrosim-pack -- runs/my-run
```

The default output is written beside the directory as `runs/my-run.zip`. A custom destination can be supplied with `--output`.

The packer:

- requires a **completed** bundle, including `manifest.json`; paused checkpoint bundles are rejected;
- requires the standard world/events/metrics/checkpoint artifacts and a resolvable day-zero founder population; current bundles carry `initial-population.json`, while semantic validation retains a reconstruction path for supported legacy resumed bundles that predate founder materialization;
- validates included artifacts as JSON before writing the archive;
- includes known landscape, spatial-mechanism, evidence, observability and ensemble-completion artifacts when they are present;
- ignores unrelated files in the run directory rather than accidentally sharing them;
- writes files in deterministic name order with fixed ZIP metadata, so the same unchanged bundle produces the same archive bytes;
- uses ordinary ZIP storage with no proprietary format, so standard ZIP readers can unpack the original JSON artifacts.

Packaging is a convenience layer only. It does not replace, rewrite or change the scientific meaning of the underlying run directory.

## Local server and security boundary

`scripts/serve-explorer.py` uses only the Python standard library. By default it binds to `127.0.0.1` and serves one explicitly selected run directory plus the fixed explorer assets.

The server:

- requires the five standard M5 reconstruction artifacts (`world.json`, `initial-population.json`, `events.json`, `metrics.json`, `checkpoint.json`), exposes `manifest.json` only when it exists, and may serve `resume-start-population.json` as optional resume-boundary provenance;
- exposes only the fixed M6 static assets;
- provides GET/HEAD only;
- rejects POST/PUT/DELETE;
- performs no directory listing;
- provides no API that mutates the run directory;
- sends a restrictive Content Security Policy.

CI hashes generated completed and paused sample runs before and after server smoke tests to ensure explorer access did not alter research artifacts.

## Verification

M6 has two distinct verification layers.

Pure JavaScript tests cover:

- completed and paused bundle reconciliation;
- rejection of incompatible or internally divergent history;
- lossless large-integer parsing and refusal of unsafe numeric visualization;
- chronological birth/migration/death reconstruction using the actual serialized event field names;
- genealogy traversal;
- entity/event filtering;
- population/environment/checkpoint-resource map-source separation;
- annual metric lookup.

CI then generates both a completed run and a genuinely paused checkpoint bundle with the release Rust binary and validates that:

- event counts reconcile with the manifest or checkpoint as appropriate;
- terminal metrics reconcile with the authoritative run boundary;
- reconstructed person count equals boundary person records;
- reconstructed living population and occupied-cell count equal boundary summaries;
- every reconstructed boundary person location/household agrees with the checkpoint;
- summed checkpoint cell resource stock agrees with the recorded resource summary;
- the loopback server opens both bundle forms, serves selected artifacts byte-for-byte and rejects writes;
- bundle hashes are unchanged after explorer access.

The run-bundle pack workflow separately generates a real completed bundle, packages it twice, requires byte-identical archives, validates the ZIP with Python's standard library, confirms unrelated files are excluded and confirms paused bundles are rejected.

The existing Rust headless benchmarks run unchanged. Explorer and packaging code are downstream and cannot change those benchmarked model semantics.

## Scientific status

M6 improves **inspectability**, not empirical validity. A clearer map, genealogy view or shareable ZIP does not make the underlying `synthetic_validation_v1` demographic, resource or migration parameters anthropologically calibrated. The same model-validation limits documented for M2–M4 remain in force.
