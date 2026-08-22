# Architecture

## Architectural objective

AnthroSim must support very long, reproducible simulations without tying scientific logic to a UI, database, cloud service, or AI provider.

The core architecture therefore follows five rules:

1. authoritative state is owned by a headless deterministic engine;
2. hot simulation data is compact and data-oriented;
3. sparse changes are event-driven and periodic processes run at the coarsest defensible cadence;
4. observation/persistence is downstream from state transitions;
5. visualisation and analysis are read-only consumers of recorded state and outputs.

## Initial workspace

```text
anthrosim/
├── crates/
│   ├── anthrosim-core/     # deterministic engine primitives and simulation lifecycle
│   └── anthrosim-cli/      # headless command-line runner
├── explorer/               # M6 read-only browser UI; outside the Rust workspace
├── scripts/                # local serving / validation helpers
├── docs/                   # design, research model, ADRs
├── examples/               # versioned example experiment definitions
├── experiments/            # research experiment definitions as models mature
├── analysis/               # notebooks/scripts later; not part of the hot loop
└── .github/workflows/      # reproducibility and quality gates
```

More crates should be created only when a real boundary exists. M6 deliberately does **not** add an explorer crate to the Rust workspace because the explorer is a downstream artifact consumer, not a simulation dependency.

## Direction of dependencies

```text
Experiment definition
        ↓
anthrosim-cli
        ↓
anthrosim-core
        ├── deterministic time / IDs / RNG streams
        ├── authoritative world state
        ├── persistent population state
        ├── dynamic resource state
        ├── bounded migration system
        ├── simulation systems
        └── observation records
        ↓
versioned output artifacts
        ↓
analysis / M6 explorer / research tooling
```

The arrow must never reverse from an explorer into authoritative simulation state during a research run.

## Simulation execution model

The target design is a hybrid of discrete events and coarse periodic systems.

- births, scheduled transitions, and sparse life-history events: event-driven or explicit demographic boundaries;
- environmental regeneration and health/resource processing: batched at explicit intervals;
- migration reevaluation: scheduled at explicit local-condition boundaries;
- slow cultural/language processes in future versions: much coarser intervals.

M4 uses configurable subannual resource periods (four per year by default). Each resource period is followed by a migration decision boundary for surviving households; the existing annual demographic boundary follows the final resource/migration period of the year. This ordering is part of the scientific model, not merely an implementation detail.

The engine should advance to the next moment at which something can change rather than pretend that every agent needs a 60 Hz game loop.

## Data-oriented state

Persistent domain identities do not imply allocation-heavy objects. Hot state should favour dense IDs, contiguous vectors, compact enums, bitsets/indices, and shared tables referenced by IDs. Rich read models may be constructed for inspection outside hot paths.

M1 applies this directly to geography: cells are stored contiguously and addressed by stable 1-based `CellId` values, while four-neighbour lookup is calculated from coordinates without allocating a neighbour collection per cell.

M2 applies the same approach to people: hot person fields are parallel contiguous vectors addressed by stable one-based `PersonId` values, and cell occupancy is a prefix index rather than a collection object per cell.

M3 keeps immutable environmental geography separate from dynamic renewable-resource state. `ResourceSystem` stores one contiguous integer stock value per cell. Each resource period uses compact arrays proportional to cell/household/person counts; it does not construct pairwise person searches or an allocation-heavy cell-to-household object graph.

M4 follows the same pattern. `MigrationSystem` owns reusable scratch arrays indexed by household and cell for living-member counts, condition totals, bounded kin-location hints, planned destinations, travel costs and pre/post-move occupancy counts. Candidate cells are generated from a bounded Manhattan radius into reusable buffers. The number of candidate destinations therefore depends on the configured local information radius, not on total world size.

Selected household moves are evaluated against one shared pre-move snapshot and then applied simultaneously in one packed scan of the living population. This prevents household-ID evaluation order from changing the information available to later households and avoids scanning the whole population separately for every move.

Dead records remain persistent. When a household later relocates, only living members move; dead people retain their location at death rather than being retroactively moved with the current household.

Dead or otherwise inactive historical state may eventually migrate out of hot memory while remaining queryable through archival outputs.

## Exact numerical state

Where practical, authoritative state uses integer/fixed-point representations. M1 environmental ratios and M2–M4 condition/resource/migration scores are stored as integers/permille rather than floating-point values. M3 resource stocks, demand, regeneration and consumption are integer abstract units. M4 utility components, candidate weights, distances and travel-condition costs are also integer-valued.

This improves compactness and exact cross-run comparison and reduces the risk that tiny platform-specific floating-point differences branch long deterministic histories.

Floating-point analysis remains appropriate downstream; avoiding it in authoritative state is not an ideological restriction where a later model genuinely requires it.

M6 has an additional display-boundary concern: JavaScript `Number` cannot exactly represent every Rust `u64`. The explorer therefore preserves integers outside JavaScript's safe integer range as exact decimal strings during JSON parsing instead of silently rounding authoritative artifact values. Numeric visualisations reject unsafe conversions rather than approximate them.

## Deterministic randomness

Randomness is explicit. The master seed derives named deterministic streams. A draw added to one system should not silently rewrite an unrelated system's stochastic history.

M1 consumes the `world` stream only to derive stable field seeds. Per-cell heterogeneity is then coordinate-derived, so generation order can change without changing the world itself.

M2 uses separate streams for mortality, fertility, parentage and newborn reproductive sex. M3 adds `resources/scarcity_mortality`; deterministic resource regeneration/allocation itself consumes no random draws. M4 adds independent `migration/choice` and `migration/uncertainty` streams.

Migration candidates are enumerated in a stable geometric order and household decisions are evaluated in stable household-ID order. Stochastic destination selection is therefore replayable without requiring global optimization.

Parallelism is introduced only with a declared deterministic strategy. Faster but nondeterministic execution may be offered later as a separate mode, never silently substituted for research runs.

## Resource accounting boundary

`World` describes the synthetic baseline environment; it is not mutated as food is consumed. `ResourceSystem` owns renewable stock and cumulative resource accounting. This separation makes it possible to compare the same generated geography under different resource-model parameters without conflating terrain generation with dynamic consumption state.

The M3/M4 accounting invariant is:

```text
initial dynamic stock + cumulative regeneration - cumulative harvest = current dynamic stock
```

Harvest is equal to consumption in the current baseline because household storage, spoilage and waste are not yet represented. If those mechanisms are later introduced, they must extend the accounting identity explicitly rather than silently changing the meaning of `harvested_food`.

Migration reads current local stock for its bounded destination comparison but does not alter resource accounting directly. Moving changes where households create demand in subsequent resource periods.

## Migration decision boundary

M4 separates **decision evaluation** from **relocation application**.

At one migration boundary:

1. compact household/cell state is derived from the current living population;
2. pressured households evaluate only candidates within their configured local radius;
3. each candidate receives an explicit integer utility decomposition for resources, water/security proxy, bounded kin proximity, travel cost, uncertainty and relocation risk;
4. candidates that do not improve sufficiently over staying are discarded;
5. an eligible destination is selected with a named deterministic stochastic stream;
6. all selected destinations are retained as plans;
7. plans are applied simultaneously to living household members in one packed pass;
8. distance-dependent travel condition cost is deducted and occupancy is rebuilt once.

The current move completes at the same decision boundary. There is no persistent en-route state, journey-duration model or movement mortality process yet. That limitation is explicit in `docs/research/migration-v0.1.md`.

For an interior cell, a Manhattan candidate radius `r` exposes at most `2r(r + 1)` move candidates. At the default radius three this is 24 candidates, independent of total world area.

## Persistence and observability

A database is not the simulation loop. The authoritative simulation remains in memory while a run executes; M5 writes versioned artifacts at controlled boundaries for offline analysis and deterministic resumption.

M5 introduces three explicit artifact classes:

- authoritative chronological events for births, deaths and completed household moves;
- derived annual/terminal metric snapshots that reconcile against authoritative state;
- deterministic annual-boundary checkpoints containing dynamic state, history and exact named-RNG stream positions.

A completed controlled run directory contains a manifest, generated world, founder population, event log, metric series and checkpoint. A deliberately paused `--checkpoint-year` directory contains world, founder population, event log, metric series and checkpoint but no completed-run manifest. The explorer and research tooling can inspect either form without a live database or simulation process.

Checkpoint restoration reconstructs the immutable synthetic world from experiment configuration + seed and verifies its digest, restores full population/resource state, reconstructs migration scratch buffers from persistent migration state, and restores all seven named ChaCha8 streams from stable stream labels plus their recorded word positions. A composite state digest is checked before execution continues.

M5 v1 deliberately supports resumable checkpoints only at completed annual boundaries. This keeps the resource -> migration -> demography schedule position unambiguous; partially completed subannual boundaries are not serialized as resumable states.

Migration still retains a bounded summary sample of detailed decision traces for ordinary manifests, while the authoritative event log records every completed move. Future analytical columnar formats may be added downstream without changing the in-memory simulation ownership boundary.

## M6 explorer boundary

M6 is intentionally **artifact-first and read-only**.

`scripts/serve-explorer.py` binds to loopback by default and exposes only fixed explorer assets plus the five run files common to completed and paused M5 bundles. `manifest.json` is additionally exposed only when it actually exists. The server implements GET/HEAD only and rejects write methods. The browser application performs no API call that can mutate simulation or artifact state.

For completed bundles, the manifest is the terminal summary and schema catalogue. For paused bundles, the checkpoint itself is the authoritative current boundary; M6 does not manufacture a completed manifest. Separately written events/metrics are checked against the history embedded in the checkpoint.

The explorer distinguishes three data classes:

- **authoritative** serialized engine values/events;
- **derived** M5 metric snapshots;
- **reconstructed display state**, such as historical living-cell occupancy replayed from founder locations plus authoritative birth/death/migration events.

M6 does not manufacture historical resource surfaces or historical individual condition where M5 did not serialize them. Checkpoint resource stock and condition are authoritative only at that checkpoint boundary; earlier unavailable values are labelled unavailable rather than interpolated.

Boundary reconstruction is verified against the checkpoint in CI: person count, living count, occupied cells, every person location/household and total checkpoint food stock must agree. CI generates both completed and genuinely paused sample bundles. Separate server smoke tests hash each bundle before and after access to verify no file changed.

The explorer has no Cargo dependency and no place in the authoritative dependency graph. Removing the entire `explorer/` and its serving scripts leaves the Rust simulation build and headless execution unchanged.

## Performance policy

Performance is part of correctness. Core metrics include:

- simulated years per wall-clock second;
- events/process boundaries per second;
- peak/resident memory;
- bytes per living individual;
- bytes per world cell;
- allocations in hot loops;
- checkpoint size and throughput.

M3 resource processing remains O(people + households + cells) per resource period. M4 adds bounded local migration work proportional to pressured households × local candidate count, plus one population/cell pass to apply simultaneous moves. It deliberately avoids global candidate scans and global pairwise searches.

CI benchmarks the 10,000-person full resource-migration-demographic lifecycle, population initialization, world generation, bounded radius-three candidate lookup and checkpoint persistence as regression baselines. Those Rust benchmark commands are unchanged by M6; explorer validation runs as downstream CI steps after the headless artifacts exist.

Optimisation follows measurement. Unsafe code, SIMD, GPU kernels, custom allocators, or distributed execution require benchmark evidence and an explicit architectural decision.
