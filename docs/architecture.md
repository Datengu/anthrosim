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

- births, scheduled transitions, and sparse life-history events: event-driven;
- environmental regeneration and health/resource processing: batched at explicit intervals;
- migration reevaluation: scheduled or triggered by meaningful local changes;
- slow cultural/language processes in future versions: much coarser intervals.

The engine should advance to the next moment at which something can change rather than pretend that every agent needs a 60 Hz game loop.

## Data-oriented state

Persistent domain identities do not imply allocation-heavy objects. Hot state should favour dense IDs, contiguous vectors, compact enums, bitsets/indices, and shared tables referenced by IDs. Rich read models may be constructed for inspection outside hot paths.

Dead or otherwise inactive historical state should eventually migrate out of hot memory while remaining queryable through archival outputs.

## Deterministic randomness

Randomness is explicit. The master seed derives named deterministic streams such as `world`, `demography`, and `migration`. A draw added to migration should not silently rewrite world generation. Streams must use an algorithm with documented portable deterministic behaviour.

Parallelism is introduced only with a declared deterministic strategy. Faster but nondeterministic execution may be offered later as a separate mode, never silently substituted for research runs.

## Persistence

A database is not the simulation loop. v0.1 writes versioned manifests, metrics, events, and later checkpoints at controlled boundaries. The authoritative state remains in memory while a run is executing.

The eventual persistence pattern is expected to combine:

- immutable run manifest;
- append-oriented event/metric outputs;
- periodic checkpoints;
- compact experiment summaries;
- optional analytical columnar formats for research workflows.

## Performance policy

Performance is part of correctness. Core metrics include:

- simulated years per wall-clock second;
- events processed per second;
- peak/resident memory;
- bytes per living individual;
- bytes per occupied world cell;
- allocations in hot loops;
- checkpoint size and throughput.

Optimisation follows measurement. Unsafe code, SIMD, GPU kernels, custom allocators, or distributed execution require benchmark evidence and an explicit architectural decision.
