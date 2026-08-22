# AnthroSim

AnthroSim is an experimental, reproducible agent-based simulation framework for exploring human demography, migration, kinship, cultural transmission, and long-run social emergence.

The project begins deliberately small. **v0.1 is building toward a deterministic hunter-gatherer population simulator**, not a civilisation game and not a validated model of human prehistory. Its purpose is to establish a fast, inspectable foundation on which increasingly evidence-grounded anthropological models can be built.

## Core rule

> Model causes, not historical outcomes.

If a pattern appears in AnthroSim, the goal is for it to be explainable from simulated conditions and agent behaviour rather than from a scripted "civilisation" rule.

## v0.1

The first version focuses on:

- a synthetic spatial environment;
- persistent individuals and genealogy;
- households and local resource sharing;
- birth, ageing, condition, and death;
- renewable food, seasonality, environmental stress and local density competition;
- local, interpretable household migration decisions;
- deterministic experiment configuration and replay;
- versioned authoritative events and derived metric snapshots;
- resumable deterministic annual-boundary checkpoints;
- self-contained offline run bundles;
- a read-only local explorer for maps, timelines, entities, events and genealogy;
- headless batch execution;
- performance benchmarks and invariant validation.

Culture, language, trade, states, religion, warfare, AI-controlled agents, and real-Earth palaeoenvironmental data are intentionally deferred.

## Current milestone status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local migration:** complete; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.
- **M5 — Events, metrics, checkpoints and causal inspection:** complete; it adds observability/persistence rather than a new anthropological mechanism.
- **M6 — Local simulation explorer:** implemented as a read-only consumer of M5 run bundles; it does not participate in authoritative simulation state or the Rust hot loop.

M1–M4 establish the first closed spatial response loop: local synthetic productivity and seasonality create renewable resource supply; co-located households compete for finite stock; household supply affects individual condition and scarcity mortality; surviving households under local pressure can compare only bounded nearby alternatives and relocate together at an explicit travel cost; demographic births and baseline deaths continue through the M2 schedules.

M5 makes that loop inspectable. Births, deaths and completed household moves are emitted as versioned authoritative events; annual/terminal summaries are explicitly derived metrics; state digests fingerprint deterministic boundaries; and annual-boundary checkpoints preserve full dynamic state plus the exact positions of all named RNG streams for deterministic resumption.

M6 makes those artifacts navigable without changing them. It provides timeline, map, cell, household, person, genealogy and event views while visibly distinguishing serialized authoritative facts, recorded derived metrics and UI reconstructions. Historical state that M5 did not record is not silently invented.

No historical destination, route, settlement, tribe or migration outcome is scripted into that loop.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. From the repository root, a small headless run can be executed with:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

For M5/M6 causal inspection, write a controlled run bundle instead:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --seed 1 --run-dir runs/m6-example
```

The directory contains `manifest.json`, `world.json`, `initial-population.json`, `events.json`, `metrics.json` and a final `checkpoint.json`, so analysis does not require a live database.

Open that run in the M6 explorer with:

```text
python scripts/serve-explorer.py runs/m6-example
```

The server binds to `127.0.0.1:8765` by default and opens the local browser. It serves only the fixed explorer assets and six run files, exposes no directory listing or write API, and rejects POST/PUT/DELETE. Use `--no-browser` if you do not want it to open a browser automatically.

To deliberately pause at a resumable annual boundary:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --seed 1 --run-dir runs/m5-resume --checkpoint-year 10
cargo run --release -p anthrosim-cli -- resume --checkpoint runs/m5-resume/checkpoint.json --run-dir runs/m5-resume
```

The CLI also exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

The manifest records configuration, artifact schema versions, world/population/resource/migration summaries, state digest, runtime counters and stop reason. See [`docs/research/observability-v0.1.md`](docs/research/observability-v0.1.md) for the authoritative-event/derived-metric distinction and checkpoint compatibility rules, and [`docs/research/explorer-v0.1.md`](docs/research/explorer-v0.1.md) for M6 display provenance and reconstruction limits.

## Scientific status

AnthroSim is **research-oriented software under active model development**, not a validated anthropological model. The current executable demographic, resource and migration presets are deliberately named `synthetic_validation_v1`. Their role is to verify mechanisms, invariants, deterministic replay and directional causal behaviour before empirical calibration/validation claims are attempted.

See [`docs/`](docs/) for the project vision, architecture, scientific-model specification, provenance notes and v0.1 scope.
