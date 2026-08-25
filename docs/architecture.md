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
        ├── persistent population + residence state
        ├── temporary physical-presence / journey state
        ├── dynamic resource state
        ├── bounded permanent-migration system
        ├── focal-region temporary-mobility/travel system
        ├── simulation systems
        └── authoritative + derived observation records
        ↓
versioned output artifacts
        ↓
analysis / M6 explorer / research tooling
```

The arrow must never reverse from an explorer into authoritative simulation state during a research run. M8/M9 derived observability is downstream of the authoritative world, population, event and checkpoint histories.

## Simulation execution model

The engine is a deterministic hybrid of sparse scheduled transitions and coarse periodic systems rather than a game-style frame loop.

- births, deaths and annual demographic transitions occur at explicit demographic boundaries;
- environmental regeneration and resource/condition processing occur at configured subannual boundaries;
- M4 permanent-migration reevaluation occurs at explicit local-condition boundaries;
- M9 temporary journeys have deterministic departure, arrival, return-departure and completion days and may remain active across annual checkpoints;
- cultural/language/social-institution processes remain deferred rather than being implied by the temporary-mobility mechanism.

M4 permanent migration still completes atomically at its decision boundary and changes household residence. M9 overlays a separate physical-presence lifecycle without redefining that mechanism. With M9 enabled, same-day ordering is explicit: settle any elapsed duration-aware resource interval; apply due temporary-mobility completions/starts; evaluate M4 permanent migration only for eligible households physically at residence; then run the annual M2 demographic boundary when applicable. M2 fertility/parentage remains residence-based rather than visitor-presence-based.

The engine advances only to moments at which represented state can change. See [`research/temporary-mobility-v1.md`](research/temporary-mobility-v1.md) for the governing M9 ordering contract.

## Data-oriented state

Persistent domain identities do not imply allocation-heavy objects. Hot state should favour dense IDs, contiguous vectors, compact enums, bitsets/indices, and shared tables referenced by IDs. Rich read models may be constructed for inspection outside hot paths.

M1 applies this directly to geography: cells are stored contiguously and addressed by stable 1-based `CellId` values, while four-neighbour lookup is calculated from coordinates without allocating a neighbour collection per cell.

M2 applies the same approach to people: hot person fields are parallel contiguous vectors addressed by stable one-based `PersonId` values, and cell occupancy is a prefix index rather than a collection object per cell.

M3 keeps immutable environmental geography separate from dynamic renewable-resource state. `ResourceSystem` stores one contiguous integer stock value per cell. Each resource period uses compact arrays proportional to cell/household/person counts; it does not construct pairwise person searches or an allocation-heavy cell-to-household object graph.

M4 follows the same pattern. `MigrationSystem` owns reusable scratch arrays indexed by household and cell for living-member counts, condition totals, bounded kin-location hints, planned destinations, travel costs and pre/post-move occupancy counts. Candidate cells are generated from a bounded Manhattan radius into reusable buffers. The number of candidate destinations therefore depends on the configured local information radius, not on total world size.

Selected household moves are evaluated against one shared pre-move snapshot and then applied simultaneously in one packed scan of the living population. This prevents household-ID evaluation order from changing the information available to later households and avoids scanning the whole population separately for every move.

Dead records remain persistent. M4 permanent relocation changes residence only for living members. Under M9, a death can occur while a household is temporarily away: physical-presence accounting removes the deceased from the actual active journey/presence state, while `Death.cell` and M8 spatial death attribution remain explicitly tied to persistent residence rather than claiming a physical death location.

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

`World` describes the baseline/model-facing environment; it is not mutated as food is consumed. `ResourceSystem` owns renewable stock and cumulative resource accounting. This separation allows identical geography to be compared under different resource assumptions without conflating terrain construction with dynamic consumption state.

The core stock invariant remains:

```text
initial dynamic stock + cumulative regeneration - cumulative harvest = current dynamic stock
```

Harvest equals consumption in the current baseline because household storage, spoilage and waste are not represented. A later mechanism must extend the identity explicitly rather than silently changing `harvested_food` semantics.

M4 permanent migration changes residence and therefore where later demand originates. M9 adds duration-aware accounting while a household is temporarily away: at-residence person-days are charged to residence, visitor person-days to the visitor/focal cell, and transit days use the declared home-provisioning proxy because transit deliberately has no authoritative world cell. The accounting implementation therefore settles elapsed intervals before same-day temporary transitions. See [`research/m9-duration-aware-resource-semantics-v1.md`](research/m9-duration-aware-resource-semantics-v1.md).

## Permanent and temporary mobility boundaries

M4 permanent migration separates **decision evaluation** from **relocation application**. Pressured households evaluate bounded nearby candidates against one shared pre-move snapshot, retain plans, then apply selected relocations simultaneously. A completed M4 move changes persistent residence and imposes the configured travel-condition cost at that boundary; it does not create an en-route state.

M9 is intentionally a different mechanism. A configured household may start a temporary journey while retaining its residence. Authoritative physical presence progresses through:

```text
at residence → outbound transit → visiting → return transit → at residence
```

Transit has journey timing and resource semantics but no authoritative per-day world cell. Focal-region identity is preserved independently of its resolved destination cells, and travel duration/cost is derived deterministically from the authoritative world plus declared travel configuration. Permanent migration is not allowed to move an away household; once the household returns, later permanent-migration boundaries may change residence normally.

This separation is scientifically important: a visitor concentration is not a settlement relocation, and repeated temporary presence must not be reconstructed by pretending that M4 permanent-migration events occurred. See [`research/migration-v0.1.md`](research/migration-v0.1.md), [`research/temporary-mobility-v1.md`](research/temporary-mobility-v1.md), and [`research/m9-temporary-travel-semantics-v1.md`](research/m9-temporary-travel-semantics-v1.md).

## Persistence and observability

A database is not the simulation loop. Authoritative state remains in memory during execution; versioned artifacts are written at controlled boundaries for offline analysis, validation and deterministic resumption.

The authoritative history now includes ordinary demographic/resource/permanent-migration state plus M9 temporary-journey transitions when configured. Annual-boundary checkpoints retain complete dynamic population/resource/migration/temporary-mobility state and the exact positions of all named deterministic RNG streams. A checkpoint may therefore contain active outbound, visiting or return journeys; deterministic resume must reproduce uninterrupted authoritative state, events and metrics exactly from that boundary onward.

A fresh completed controlled run directory contains the run manifest, authoritative world, day-zero founder population, chronological event log, derived metrics and final checkpoint. A deliberately paused directory contains the same reconstruction inputs without a completed-run manifest. A new-directory resume additionally preserves `resume-start-population.json` as boundary provenance while deterministically reconstructing the true founder artifact.

M8 `spatial-observability.json` and M9 `temporary-observability.json` are **derived companion artifacts**, not alternative authoritative state. Spatial-observability schema v2 explicitly attributes occupancy/person-days/births/deaths to persistent residence and excludes temporary visitors/transit. Temporary observability separately reconstructs residents, visitors, transit, journey counts/durations, person-days, peaks and catchment from the M9 event/state history. Core invariants validate the M9 event lifecycle independently of whether the derived temporary report is requested.

Checkpoint restoration verifies experiment/model/source identity, reconstructed world identity, complete persistent and temporary state and the composite state digest before execution continues. Checkpoints remain resumable at completed annual boundaries; M9 makes the represented within-journey state explicit rather than making the schedule ambiguous.

## M6 explorer boundary

M6 remains intentionally **artifact-first and read-only**. `scripts/serve-explorer.py` binds to loopback and serves only fixed explorer assets plus an explicit allowlist of expected run artifacts; it implements GET/HEAD only and rejects write methods. M8 landscape/spatial artifacts and M9 `temporary-observability.json` are optional allowlisted extensions.

The browser distinguishes serialized authoritative facts, recorded derived metrics and reconstructed display state. It never manufactures unavailable historical resource/condition values. For landscape-bound M9 runs, residence-based M8 spatial quantities and M9 visitor/physical-presence quantities are shown as separate concepts rather than merged into an ambiguous occupancy field.

The explorer has no Cargo dependency and no place in the authoritative dependency graph. Removing the entire explorer and its serving scripts leaves the Rust simulation build and headless execution unchanged.

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
