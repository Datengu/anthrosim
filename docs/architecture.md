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
├── docs/                   # design, research model, ADRs
├── examples/               # versioned example experiment definitions
├── experiments/            # research experiment definitions as models mature
├── analysis/               # notebooks/scripts later; not part of the hot loop
└── .github/workflows/      # reproducibility and quality gates
```

More crates should be created only when a real boundary exists. v0.1 should not begin as a large framework of empty modules.

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
        ├── simulation systems
        └── observation records
        ↓
versioned output artifacts
        ↓
analysis / explorer / research tooling
```

The arrow must never reverse from an explorer into authoritative simulation state during a research run.

## Simulation execution model

The target design is a hybrid of discrete events and coarse periodic systems.

- births, scheduled transitions, and sparse life-history events: event-driven or explicit demographic boundaries;
- environmental regeneration and health/resource processing: batched at explicit intervals;
- migration reevaluation: scheduled or triggered by meaningful local changes;
- slow cultural/language processes in future versions: much coarser intervals.

M3 currently uses configurable subannual resource periods (four per year by default) followed by the existing annual demographic boundary. This schedule is part of the scientific model, not merely an implementation detail.

The engine should advance to the next moment at which something can change rather than pretend that every agent needs a 60 Hz game loop.

## Data-oriented state

Persistent domain identities do not imply allocation-heavy objects. Hot state should favour dense IDs, contiguous vectors, compact enums, bitsets/indices, and shared tables referenced by IDs. Rich read models may be constructed for inspection outside hot paths.

M1 applies this directly to geography: cells are stored contiguously and addressed by stable 1-based `CellId` values, while four-neighbour lookup is calculated from coordinates without allocating a neighbour collection per cell.

M2 applies the same approach to people: hot person fields are parallel contiguous vectors addressed by stable one-based `PersonId` values, and cell occupancy is a prefix index rather than a collection object per cell.

M3 keeps immutable environmental geography separate from dynamic renewable-resource state. `ResourceSystem` stores one contiguous integer stock value per cell. Each resource period allocates temporary contiguous arrays proportional to cell/household counts for demand/allocation; it does not construct pairwise person searches or an allocation-heavy cell-to-household object graph.

Dead or otherwise inactive historical state should eventually migrate out of hot memory while remaining queryable through archival outputs.

## Exact numerical state

Where practical, authoritative state uses integer/fixed-point representations. M1 environmental ratios and M2/M3 condition/resource ratios are stored as integers/permille rather than floating-point values. M3 resource stocks, demand, regeneration and consumption are integer abstract units. This improves compactness and exact cross-run comparison and reduces the risk that tiny platform-specific floating-point differences branch long deterministic histories.

Floating-point analysis remains appropriate downstream; avoiding it in authoritative state is not an ideological restriction where a later model genuinely requires it.

## Deterministic randomness

Randomness is explicit. The master seed derives named deterministic streams. A draw added to one system should not silently rewrite an unrelated system's stochastic history.

M1 consumes the `world` stream only to derive stable field seeds. Per-cell heterogeneity is then coordinate-derived, so generation order can change without changing the world itself.

M2 uses separate streams for mortality, fertility, parentage and newborn reproductive sex. M3 adds `resources/scarcity_mortality`; deterministic resource regeneration/allocation itself consumes no random draws.

Parallelism is introduced only with a declared deterministic strategy. Faster but nondeterministic execution may be offered later as a separate mode, never silently substituted for research runs.

## Resource accounting boundary

`World` describes the synthetic baseline environment; it is not mutated as food is consumed. `ResourceSystem` owns renewable stock and cumulative resource accounting. This separation makes it possible to compare the same generated geography under different resource-model parameters without conflating terrain generation with dynamic consumption state.

The M3 accounting invariant is:

```text
initial dynamic stock + cumulative regeneration - cumulative harvest = current dynamic stock
```

Harvest is equal to consumption in the M3 baseline because household storage, spoilage and waste are not yet represented. If those mechanisms are later introduced, they must extend the accounting identity explicitly rather than silently changing the meaning of `harvested_food`.

## Persistence

A database is not the simulation loop. v0.1 writes versioned manifests, metrics, events, and later checkpoints at controlled boundaries. The authoritative state remains in memory while a run is executing.

The eventual persistence pattern is expected to combine:

- immutable run manifest;
- append-oriented event/metric outputs;
- periodic checkpoints;
- compact experiment summaries;
- optional analytical columnar formats for research workflows.

M3 currently emits aggregate resource state/accounting in the run manifest; chronological resource events/metrics are deferred to M5.

## Performance policy

Performance is part of correctness. Core metrics include:

- simulated years per wall-clock second;
- events/process boundaries per second;
- peak/resident memory;
- bytes per living individual;
- bytes per world cell;
- allocations in hot loops;
- checkpoint size and throughput.

M3 retains O(people + households + cells) resource-period processing. It deliberately avoids global pairwise searches. The 10,000-person resource-demographic lifecycle is benchmarked in CI as a regression baseline.

Optimisation follows measurement. Unsafe code, SIMD, GPU kernels, custom allocators, or distributed execution require benchmark evidence and an explicit architectural decision.
