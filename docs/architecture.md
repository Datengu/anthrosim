# Architecture

**Current framework identity:** software version v0.3.5 / current model semantics v35. The immutable v0.3.4/v25 release remains the frozen Audit-v4 discovery target; other historical release/audit baselines retain their own identities.

## Architectural objective

AnthroSim must support very long, reproducible simulations without tying scientific logic to a UI, database, cloud service, or AI provider.

The core architecture follows five rules:

1. authoritative state is owned by a headless deterministic engine;
2. hot simulation data is compact and data-oriented;
3. sparse changes are event-driven and periodic processes run at the coarsest defensible cadence;
4. observation/persistence is downstream from state transitions;
5. visualisation and analysis are read-only consumers of recorded state and outputs.

## Workspace and dependency direction

```text
Experiment definition
        ↓
anthrosim-cli
        ↓
anthrosim-core
        ├── deterministic time / IDs / named RNG streams
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
analysis / read-only explorer / research tooling
```

The explorer is deliberately outside the Rust workspace and must never write authoritative state during a research run. M8/M9 observability is downstream of authoritative world, population, event and checkpoint histories.

## Simulation execution model

The engine is a deterministic hybrid of sparse scheduled transitions and coarse periodic systems rather than a frame loop.

Current timing is intentionally split by causal process:

- M3 resource accounting and condition response are resolved over configured elapsed subannual intervals;
- background mortality is an annual risk partitioned across those elapsed M3 intervals;
- background and condition-mediated mortality are resolved together as order-invariant competing risks at those interval boundaries;
- M4 permanent-migration reevaluation occurs on its declared decision clock after due elapsed-resource/mortality work;
- M9 temporary journeys have deterministic departure, arrival, return-departure and completion days and may remain active across annual checkpoints;
- year-end M2 performs fertility/parentage for survivors and does **not** redraw background mortality;
- ageing/annual lifecycle state advances at its declared annual boundary.

With M9 enabled, same-day ordering remains explicit: settle any elapsed duration-aware resource/mortality interval, apply due temporary-mobility completions/starts, evaluate M4 permanent migration for eligible households physically at residence, then perform year-end M2 fertility/parentage when applicable. See [`research/m2-demographic-time-contract-v1.md`](research/m2-demographic-time-contract-v1.md) and [`research/temporary-mobility-v1.md`](research/temporary-mobility-v1.md).

## Data-oriented authoritative state

Persistent domain identities do not imply allocation-heavy objects. Hot state favours dense IDs, contiguous vectors, compact enums, bitsets/indices and reusable scratch storage; rich read models are constructed outside hot paths.

M1 stores cells contiguously with stable one-based `CellId` values. M2 stores person fields in packed/parallel structures addressed by stable `PersonId` values. M3 keeps immutable environmental geography separate from dynamic renewable-resource stock. M4 uses reusable household/cell scratch arrays for bounded candidate evaluation and applies selected relocations against one shared pre-move snapshot. M9 adds temporary-presence/journey state without changing persistent residence identity.

Dead records remain persistent. Permanent relocation changes residence only for living members. A death while temporarily away removes the deceased from active physical-presence/journey accounting, while residence-based death attribution remains explicitly distinct from a claimed physical death location.

## Scientific identity versus bookkeeping identity

Canonical `PersonId`, `HouseholdId`, packed-record order and candidate enumeration order are storage/provenance conveniences, not intended scientific causes.

Audit-v4 repairs introduced persistent **scientific stochastic-coupling ranks** so same-seed stochastic assignments remain attached to represented scientific roles/state rather than arbitrary labels:

- fertility draw assignment uses person stochastic-coupling rank (v26);
- background-mortality latent triggers use person coupling rank (v27);
- M4 household migration scheduling uses the minimum living-member coupling rank rather than `HouseholdId` (v28);
- residence-local parentage candidates are ordered by person coupling rank before the uniform reservoir sampler (v29);
- condition-mediated mortality triggers and simultaneous-cause attribution use the same person coupling-rank schedule while retaining independent named streams (v30);
- M9 equal-cost destination ties use household scientific coupling rather than `HouseholdId` (v31);
- scarce-resource largest-remainder ties use household scientific coupling plus the declared period/cell fairness rotation rather than claim-vector/household-label order (v32);
- M4 candidate uncertainty and proportional-choice assignment are invariant to canonical candidate ordering by coupling exact deterministic utility/distance equivalence classes and sampling exchangeable members symmetrically (v33).
- M9 equal-cost destination coupling uses a household-local identity rather than a globally ordinal population rank (v34).
- M9 equal-cost destination realization is equivariant under the supported grid-reflection group while preserving marginal exchangeability within scientifically indistinguishable alternatives (v35).

This progression is summarized in `crates/anthrosim-core/src/provenance.rs`, whose `MODEL_SEMANTICS_ID` is the authoritative current compatibility identity.

## Deterministic randomness

Randomness is explicit. The master seed derives named deterministic streams; adding a draw to one subsystem must not silently rewrite unrelated stochastic history.

M2/M3 use separate streams for background mortality, condition-mediated mortality, fertility, parentage and newborn reproductive sex. M4 uses independent migration choice and uncertainty streams. M9 uses its declared deterministic/tie semantics. Stream separation is necessary but not sufficient: draw **assignment** also follows the scientific coupling rules above so arbitrary storage labels/order do not become hidden causes.

M4 candidate discovery may use deterministic enumeration internally, but under current v35 semantics (rule introduced at v33) candidate enumeration order is not the scientific stochastic-coupling key. Exact deterministic utility and movement distance define exchangeability classes for uncertainty/choice assignment.

Parallelism is introduced only with a declared deterministic strategy. Faster nondeterministic execution may be offered later only as an explicitly separate mode, never silently substituted for research runs.

## Exact numerical state

Where practical, authoritative state uses integer/fixed-point representations. Environmental ratios, condition, resource accounting, migration utilities/costs and many derived model-facing quantities are integer or permille values. This supports exact replay/comparison and reduces platform-dependent floating-point branching.

Floating-point analysis remains appropriate downstream. The browser also preserves integers beyond JavaScript's safe integer range as exact decimal strings instead of silently rounding authoritative `u64` values.

## Resource accounting boundary

`World` describes baseline/model-facing environment; `ResourceSystem` owns dynamic renewable stock and cumulative accounting. The core identity remains:

```text
initial dynamic stock + cumulative regeneration - cumulative harvest = current dynamic stock
```

Harvest equals consumption in the current baseline because storage/spoilage/waste are not represented.

Under scarcity, indivisible remainder units are allocated using the current v32 scientific household coupling/fairness rule rather than arbitrary household or claim-vector order. M9 duration-aware accounting charges at-residence person-days to residence, visitor person-days to visitor/focal cells, and transit through the declared home-provisioning proxy because transit has no authoritative world cell. See [`research/m3-resource-time-contract-v1.md`](research/m3-resource-time-contract-v1.md) and [`research/m9-duration-aware-resource-semantics-v1.md`](research/m9-duration-aware-resource-semantics-v1.md).

## Permanent and temporary mobility boundaries

M4 permanent migration separates decision evaluation from relocation application. Pressured households evaluate bounded nearby candidates against one shared pre-move snapshot; selected relocations are applied simultaneously. A completed M4 move changes persistent residence and applies declared travel-condition cost at that boundary; there is no en-route M4 state.

M9 is a distinct mechanism. A household may retain residence while progressing through:

```text
at residence → outbound transit → visiting → return transit → at residence
```

Transit has journey timing/resource semantics but no authoritative per-day world cell. A visitor concentration is therefore not a settlement relocation. Permanent migration cannot move an away household; later M4 boundaries may act normally once it returns.

## Persistence, provenance and observability

Authoritative state remains in memory during execution; versioned artifacts are written at controlled boundaries for offline analysis, validation and deterministic resumption.

A completed controlled run directory contains the run manifest, authoritative world, day-zero founder population, chronological event log, derived metrics and final checkpoint. Paused/resumed workflows preserve explicit resume-boundary provenance and source lineage. Checkpoint restoration validates model/package/source identity, reconstructed world identity, complete state and deterministic continuation identity before execution continues.

M8 `spatial-observability.json` and M9 `temporary-observability.json` are derived companion artifacts, not alternative authoritative state. Spatial observability remains residence-based; temporary observability reconstructs residents, visitors, transit, journey counts/durations, person-days, peaks and catchment from preserved authoritative history.

## Explorer boundary

The local explorer is artifact-first and read-only. `scripts/serve-explorer.py` binds to loopback and serves fixed explorer assets plus an explicit allowlist of expected run artifacts. It exposes no authoritative write path. Removing the explorer leaves the headless Rust simulation build/execution unchanged.

## Performance policy

Performance is part of correctness. Core metrics include simulated years per wall-clock second, process/event throughput, memory, bytes per living person/cell, hot-loop allocations and checkpoint throughput.

M3 resource processing remains O(people + households + cells) per resource period. M4 adds bounded local candidate work proportional to pressured households × local candidate count plus shared scans/application. The model deliberately avoids global pairwise searches or allocation-heavy hot-path object graphs.

CI benchmarks world generation, population initialization, bounded M4 candidate lookup, full resource/migration/demographic lifecycle and checkpoint persistence. Optimisation follows measurement; unsafe code, SIMD, GPU kernels, custom allocators or distributed execution require benchmark evidence and explicit architectural review.
