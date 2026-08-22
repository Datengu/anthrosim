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
- structured events, aggregate metrics, and checkpoints;
- headless batch execution;
- performance benchmarks and invariant validation.

Culture, language, trade, states, religion, warfare, AI-controlled agents, and real-Earth palaeoenvironmental data are intentionally deferred.

## Current milestone status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local migration:** implemented in the current milestone branch; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.

M1–M4 establish the first closed spatial response loop: local synthetic productivity and seasonality create renewable resource supply; co-located households compete for finite stock; household supply affects individual condition and scarcity mortality; surviving households under local pressure can compare only bounded nearby alternatives and relocate together at an explicit travel cost; demographic births and baseline deaths continue through the M2 schedules.

No historical destination, route, settlement, tribe or migration outcome is scripted into that loop.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. From the repository root, a small headless run can be executed with:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

The CLI exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

The output manifest records the experiment configuration plus world, population, resource and migration summaries so runs can be compared and reproduced. M4 migration summaries include bounded decision traces that expose the implemented origin/destination utility factors for a capped sample of completed moves.

## Scientific status

AnthroSim is **research-oriented software under active model development**, not a validated anthropological model. The current executable demographic, resource and migration presets are deliberately named `synthetic_validation_v1`. Their role is to verify mechanisms, invariants, deterministic replay and directional causal behaviour before empirical calibration/validation claims are attempted.

See [`docs/`](docs/) for the project vision, architecture, scientific-model specification, provenance notes and v0.1 scope.
