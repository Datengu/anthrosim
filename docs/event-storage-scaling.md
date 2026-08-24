# Event storage scaling boundary

## Status

Design boundary only. The current monolithic `events.json` format remains the default and no persisted schema changes are introduced by this document.

AnthroSim should move to chunked/streaming event storage only when measurements from a concrete target experiment show that the existing representation is a material write, memory, validation or inspection bottleneck.

## Why this boundary exists

M5 deliberately chose a simple inspectable event representation:

```text
EventLog {
  schemaVersion,
  events: [EventRecord, ...]
}
```

Each authoritative `EventRecord` has a stable `sequence`, simulation `day`, provenance and typed event payload. That logical event stream is scientifically important; the fact that it is currently serialized as one JSON document is not.

There are two related scaling pressures in the current implementation:

1. completed/paused run bundles write the complete event history as `events.json`;
2. `SimulationCheckpoint` also embeds the complete `EventLog`, so checkpoint size and resume serialization currently duplicate that historical event volume.

A future change that merely splits `events.json` while leaving the entire history embedded in every checkpoint would therefore not solve the underlying persistence problem.

## Non-goal: changing storage pre-emptively

Small and moderate runs should keep the current representation because it is easy to inspect, copy, validate and consume with ordinary JSON tooling.

Before implementing a second physical representation, collect measurements for the actual workload that motivates it, including at least:

- event count and `events.json` byte size;
- checkpoint byte size attributable to historical events;
- peak memory while reading/validating events;
- write and parse time;
- explorer load latency and browser memory;
- analysis-tool cost when only a narrow sequence/time range is required.

The implementation trigger should be a demonstrated failure of an explicit workload/acceptance budget, not a guessed event-count threshold.

## Core rule: event semantics are storage-independent

The authoritative logical contract remains an ordered stream of `EventRecord` values.

Physical storage must not change:

- event schema or field meaning;
- authoritative/derived provenance meaning;
- stable sequence numbers;
- chronological ordering/invariants;
- event counts used by run/checkpoint reconciliation;
- replay results used by spatial observability or the M6 explorer;
- deterministic simulation outcomes.

A storage migration is therefore an artifact/persistence change, not a new anthropological model semantic. If the same logical event records are stored monolithically or in chunks, scientific interpretation must be identical.

## Reader boundary before storage migration

Before a chunked representation is introduced, event consumers should converge on a storage-neutral reader boundary rather than opening `events.json` directly throughout the codebase.

The conceptual interface should support operations equivalent to:

```text
metadata()
iter_all()
iter_sequence_range(first, last)
iter_day_range(first_day, last_day)
```

The initial backend can simply wrap the existing `EventLog`/`events.json`. A later backend can resolve a chunk manifest and read only the relevant files.

Consumers that should eventually use the common boundary include:

- completed-run semantic validation;
- checkpoint/run reconciliation;
- `anthrosim-pack` bundle discovery;
- spatial-observability replay;
- the M6 explorer model/server boundary;
- downstream analysis helpers.

This prevents a later storage migration from requiring every consumer to reinterpret event semantics independently.

## Candidate future chunked layout

The precise encoding should be benchmarked before implementation, but a future bundle may use a structure conceptually like:

```text
events/
  manifest.json
  chunks/
    00000000000000000001-00000000000000100000.<encoding>
    00000000000000100001-00000000000000200000.<encoding>
    ...
```

The chunk manifest should be versioned separately from the `EventRecord` schema and should record, for every chunk:

- relative path;
- first and last event sequence;
- event count;
- first and last simulation day where useful for range selection;
- byte size;
- cryptographic content hash;
- physical encoding/version.

At the top level it should record at least:

- storage-manifest schema version;
- logical event-record/log schema version;
- total event count;
- first/last sequence when non-empty;
- deterministic chunking policy/configuration;
- ordered chunk descriptors.

The existing research-integrity manifest remains the archive-wide integrity layer. Per-chunk hashes serve the narrower operational purpose of validating a chunked event source before replay/resume; they do not replace the archive-level manifest.

## Deterministic chunk boundaries

Chunk boundaries must not depend on wall-clock timing, thread scheduling, IO buffer size or whether a run was interrupted.

A suitable policy is derived from the authoritative sequence number, for example fixed sequence windows. If a configured window holds `N` events, sequence `s` belongs to a deterministic window derived from `(s - 1) / N`.

The actual `N` should be selected from measurements and declared in storage provenance; it is not a model parameter.

This matters for checkpoint/resume. An uninterrupted run and an equivalent resumed run must converge on the same completed chunk partition for the same logical event stream and storage configuration.

A paused run may persist an incomplete tail window. Completed windows can be immutable; the active tail may be rewritten/continued at a later checkpoint. Final completed output must canonicalize the same sequence windows regardless of pause/resume history.

## Ordering and validation invariants

A chunked event source must fail closed unless the ordered chunks reconstruct one valid logical stream.

Validation should require at least:

- no duplicate chunk path;
- no missing or duplicated sequence range;
- first event sequence is the same value required by the current event contract;
- every subsequent event sequence is exactly contiguous;
- chunk descriptor counts/ranges agree with decoded records;
- records satisfy the same chronological/event invariants as monolithic `EventLog`;
- declared chunk byte sizes/hashes match the files;
- aggregate count/last sequence match the storage manifest;
- run/checkpoint summaries reconcile against the reconstructed logical stream.

Chunk filename order alone is never authoritative. Sequence values and the versioned manifest define the stream.

## Checkpoint/resume migration

Full persistence scaling eventually requires changing how historical events are represented by checkpoints, because `SimulationCheckpoint` currently contains the complete `EventLog`.

That change must be explicit and versioned. A future checkpoint design may replace embedded historical records with a binding roughly equivalent to:

```text
EventHistoryBinding {
  storage_schema_version,
  event_schema_version,
  event_count,
  last_sequence,
  storage_manifest_identity,
  next_sequence
}
```

The exact type is intentionally not fixed here, but the following rules are required:

1. The checkpoint must still contain enough information to continue sequence numbering deterministically.
2. Resume must validate the complete prior event history binding before continuing; it must not silently continue from a missing/different archive.
3. If a checkpoint depends on companion chunk files, it is a **checkpoint bundle contract**, not the current self-contained single-file checkpoint contract. The schema/version and CLI documentation must say so explicitly.
4. New-directory resume must carry or reconstruct the same validated history and continue the deterministic tail window; it must not relabel the resume boundary as the start of event history.
5. A resumed completed run must expose the same logical event stream as the equivalent uninterrupted run.
6. Migration from an old monolithic checkpoint must be deterministic and explicit if supported; unsupported old/new combinations must fail rather than discard history.

The simulation's dynamic state should not require replaying old events to advance. Historical events are observational provenance. That separation should be preserved so event-archive growth does not force historical replay into the hot simulation loop.

## Explorer and analysis range access

M6 currently benefits from the simplicity of whole-file JSON, but a large chunked archive should not require loading every event merely to inspect one period.

A future M6 event reader can remain read-only and static-file-oriented:

1. load the event storage manifest;
2. use declared sequence/day ranges to identify relevant chunks;
3. fetch only those chunk files;
4. apply the same existing event interpretation/replay rules.

No write API or live database is required.

For historical occupancy replay, some views inherently need history from day zero to the selected boundary unless a separately validated derived/index/checkpoint snapshot is available. Chunking reduces transport/parser memory and enables incremental replay; it does not justify inventing random-access historical state that was never recorded.

Downstream Python/R tooling should likewise be able to stream chunks or request ranges through a small adapter instead of first materializing one giant JSON object.

## Optional indices

An index may later accelerate queries by day, event type, person, household or cell, but indices are **derived acceleration structures**, not authoritative event history.

Any index must:

- identify the exact event-storage manifest it was built from;
- be deterministically rebuildable;
- be discardable without losing authoritative history;
- never override the event records when disagreement occurs.

Do not add indexing until a measured query workload needs it.

## Bundle, pack and archive behavior

Semantic bundle validation must accept exactly one declared event storage representation for a completed run: the current monolithic form or a future versioned chunked form. Ambiguous mixtures should fail unless a migration/compatibility mode explicitly defines which is authoritative.

`anthrosim-pack` should consume the same storage-neutral bundle resolver and include every authoritative event chunk plus its manifest in deterministic relative-path order. If extremely large archives exceed classic ZIP constraints, that is an archive-container scaling decision and must not change event semantics.

The cryptographic research-integrity tooling can hash the resulting chunk files/manifest or a final publication package independently of the event-storage implementation.

## Compatibility and migration strategy

A safe migration path is staged:

### Stage 0 — current default

- `events.json` and checkpoint-embedded `EventLog` remain unchanged.
- Measure real workloads.
- Avoid introducing new consumers that require the physical filename/layout for semantic reasons.

### Stage 1 — storage-neutral readers

- Introduce one event-source abstraction in Rust and equivalent downstream adapters when a concrete implementation project begins.
- Keep only the monolithic backend initially if useful for risk reduction.
- Prove byte/record-equivalent replay through existing tests.

### Stage 2 — optional chunked completed-run output

- Add a versioned chunk manifest and deterministic chunk backend.
- Preserve the monolithic default for ordinary small runs.
- Validate both backends against the same logical event-stream invariants.
- Add conversion/equivalence tests from a monolithic fixture to chunked storage.

### Stage 3 — scalable checkpoint history

- Only if checkpoint duplication itself is measured as a material bottleneck, introduce a new checkpoint schema with an explicit event-history binding.
- Preserve strict resume provenance and uninterrupted/resumed logical equivalence.

### Stage 4 — range-aware explorer/analysis

- Use manifest ranges/chunks for incremental reads.
- Add optional derived indices only for measured workloads.

These stages need not all be implemented together; each must preserve compatibility guarantees appropriate to its scope.

## Required test properties for a future implementation

A storage implementation is not complete until CI demonstrates at least:

- monolithic and chunked representations decode to exactly the same ordered `EventRecord` values;
- chunking the same event stream twice produces the same completed storage manifest/chunk boundaries;
- uninterrupted and checkpoint-resumed execution produce the same logical event stream and final chunk partition under the same storage configuration;
- missing, reordered, duplicated, truncated or modified chunks are rejected;
- range reads return exactly the same records as filtering a fully decoded event stream;
- semantic bundle validation, observability and Explorer boundary reconstruction agree across storage backends;
- small-run monolithic bundles remain supported unless a separately reviewed migration deliberately changes the default.

## Decision summary

AnthroSim's event **sequence and record semantics** are authoritative; `events.json` is the current physical representation, not the permanent semantic interface.

No storage migration is justified today by this design alone. When measurements demonstrate the need, the project should first centralize event reading, then add deterministic chunked storage, and only later externalize checkpoint history if checkpoint duplication is also a measured problem.

This keeps simple runs inspectable now while preventing future scale work from either breaking scientific provenance or being blocked by accidental dependence on one monolithic JSON file.
