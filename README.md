# AnthroSim

AnthroSim is an experimental, reproducible agent-based simulation framework for exploring human demography, migration, kinship, cultural transmission, and long-run social emergence.

**v0.1 establishes a deterministic synthetic population-simulation and experiment baseline.** M8 extends that baseline with evidence-grounded spatial inputs, explicit model-facing transformations, spatial observability and a first real-world-derived terrain null-model benchmark. AnthroSim is not a civilisation game and it is not a validated model of human prehistory.

## Core rule

> Model causes, not historical outcomes.

If a pattern appears in AnthroSim, the goal is for it to be explainable from simulated conditions and agent behaviour rather than from a scripted "civilisation" rule.

## Current capabilities

AnthroSim includes:

- deterministic synthetic spatial environments;
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
- deterministic multi-seed batch/ensemble execution with immutable provenance and retry state;
- deterministic parameter sweeps with derived machine-readable analysis tables;
- long-run correctness soaks and cross-artifact invariant validation;
- end-to-end performance and process-memory acceptance gates;
- a versioned 144-run synthetic resource-variability reference experiment;
- versioned normalized landscape inputs with explicit provenance;
- deterministic transformations from declared landscape layers into movement cost, water access and resource opportunity;
- spatial observability and read-only explorer support for landscape-bound runs;
- evidence catalogues carried through immutable experiment identity;
- a reproducible 32-run evidence-grounded terrain null-model benchmark.

Culture, language, trade, states, religion, warfare, and AI-controlled agents remain deferred until a research question or validation target justifies adding them.

## Current milestone status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local migration:** complete; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.
- **M5 — Events, metrics, checkpoints and causal inspection:** complete; it adds observability/persistence rather than a new anthropological mechanism.
- **M6 — Local simulation explorer:** complete as a read-only consumer of completed and paused M5 run bundles; it does not participate in authoritative simulation state or the Rust hot loop.
- **M7.1 — Batch / ensemble execution:** complete.
- **M7.2 — Immutable experiment provenance and retry semantics:** complete.
- **M7.3 — Parameter sweeps and aggregate analysis outputs:** complete.
- **M7.4 — Long-run soak and invariant hardening:** complete.
- **M7.5 — Performance and memory acceptance benchmarking:** complete.
- **M7.6 — First resource-variability experiment and v0.1 closure:** complete.
- **M7 / v0.1 experiment-engine baseline:** complete.
- **M8 — Evidence-grounded spatial experiments:** complete. M8 adds reproducible normalized landscape inputs, explicit deterministic spatial transformations, exact landscape/evidence identity, machine-readable spatial observability, explorer support, M7 ensemble/sweep integration and a first evidence-grounded null-model benchmark. See [`docs/roadmap.md`](docs/roadmap.md).

M1–M4 establish the first closed spatial response loop: local synthetic productivity and seasonality create renewable resource supply; co-located households compete for finite stock; household supply affects individual condition and scarcity mortality; surviving households under local pressure can compare only bounded nearby alternatives and relocate together at an explicit travel cost; demographic births and baseline deaths continue through the M2 schedules.

M5 makes that loop inspectable. Births, deaths and completed household moves are emitted as versioned authoritative events; annual/terminal summaries are explicitly derived metrics; state digests fingerprint deterministic boundaries; and annual-boundary checkpoints preserve full dynamic state plus the exact positions of all named RNG streams for deterministic resumption.

M6 makes those artifacts navigable without changing them. It provides timeline, map, cell, household, person, genealogy and event views while visibly distinguishing serialized authoritative facts, recorded derived metrics and UI reconstructions. Historical state that M5 did not record is not silently invented.

M7 adds deterministic experiment orchestration around that same simulator rather than a second simulation engine. Ensembles, retries and parameter sweeps preserve immutable provenance, keep incomplete/failed runs explicit, and generate separate derived analysis tables without changing authoritative model state.

M8 keeps the same separation between evidence, model assumptions and results. Externally derived landscape values remain distinct from model-facing transformed fields; transformed runs retain exact landscape and mechanism identity; spatial metrics are downstream derived artifacts; and visualisation remains read-only. The first M8.6 reference exercise used an open terrain input and four movement-cost mappings over eight paired seeds. Its predeclared result was **fragile spatial structure**: terrain materially perturbed migration distance and largest-cell concentration in some runs, but the direction was not stable across seeds. This is a result about the declared terrain-only null model, not a historical reconstruction or archaeological validation. See [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md).

No historical destination, route, settlement, tribe or migration outcome is scripted into that loop.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. The CLI package contains multiple binaries, so local `cargo run` commands should explicitly select the main `anthrosim` binary.

From the repository root, a small headless run can be executed with:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

For causal inspection, write a controlled completed run bundle instead:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m6-example
```

A completed directory contains `manifest.json`, `world.json`, `initial-population.json`, `events.json`, `metrics.json` and a final `checkpoint.json`, so analysis does not require a live database.

Open that run in the explorer with:

```text
python scripts/serve-explorer.py runs/m6-example
```

The server binds to `127.0.0.1:8765` by default and opens the local browser. It serves only fixed explorer assets and expected run artifacts, exposes no directory listing or write API, and rejects POST/PUT/DELETE. Use `--no-browser` if you do not want it to open a browser automatically.

A deliberately paused run can be explored **before** resuming it:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m5-resume --checkpoint-year 10
python scripts/serve-explorer.py runs/m5-resume
```

Resume later with:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- resume --checkpoint runs/m5-resume/checkpoint.json --run-dir runs/m5-resume
```

The CLI exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--resource-seasonality-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

## Testing M8 landscape mode

A committed generic M8 landscape/mechanism fixture can be run directly from the repository:

```text
cargo run --release -p anthrosim-cli --bin anthrosim-landscape -- run \
  --landscape examples/landscape-loading/landscape.json \
  --mechanisms examples/landscape-loading/spatial-mechanisms.json \
  --years 25 \
  --population 1000 \
  --seed 1 \
  --run-dir runs/m8-landscape-example
```

This produces the ordinary run artifacts plus the preserved normalized landscape, spatial transformation configuration and landscape/spatial wrapper provenance. The source landscape and transformed authoritative `world.json` remain separate by design.

For the full M8.6 public benchmark definition and result, see [`docs/research/m8-first-evidence-grounded-benchmark.md`](docs/research/m8-first-evidence-grounded-benchmark.md), [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md), and `examples/m8-first-evidence-grounded-benchmark/`.

## Running deterministic ensembles

Launch an explicit deterministic seed set unattended:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 25 \
  --population 10000 \
  --seeds 1,2,3,5,8 \
  --run-dir runs/example-ensemble
```

Or use a consecutive seed range:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 25 \
  --population 10000 \
  --seed-start 100 \
  --seed-count 20 \
  --run-dir runs/example-range
```

A fresh ensemble writes `experiment-manifest.json` before child execution. That immutable, versioned manifest records the model identity and complete exact `ExperimentConfig` for every planned seed. `ensemble-plan.json` remains as the concise M7.1 planning view. Mutable per-run lifecycle records are written separately under `status/`.

Each seed receives its own stable directory such as `runs/seed-00000000000000000100/`, containing the ordinary completed run artifacts plus `completion.json`. The positive completion marker is written only after the child bundle succeeds. A run status becomes `completed` only when that bundle reconciles with the exact immutable experiment definition.

If execution is interrupted or one run fails, rerun the **same command and exact configuration** with `--retry`. Retry first requires exact equality with the stored immutable experiment manifest. It keeps provenance-valid completed runs without executing them again, reconciles interrupted/missing bundles as incomplete, and reruns only planned, failed or incomplete children. Partial child directories are removed before a retry attempt so old and new artifacts cannot be mixed. A completed bundle with conflicting provenance is treated as an integrity error instead of being silently overwritten.

The batch continues to later seeds after an individual child fails, but the overall command still exits unsuccessfully while any child is unsuccessful. Downstream analysis should treat only `completed` status records with provenance-valid bundles as successful results.

## Running deterministic parameter sweeps

M7 adds an explicit Cartesian parameter-grid layer. For example, this compares two M3 productivity settings and two seasonal-amplitude settings over the same four seeds:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-resource-seasonality-scale-permille 0,1000 \
  --run-dir runs/resource-sweep
```

Supported sweep dimensions include founder population, target household size, M3 productivity scale, M3 seasonal-amplitude scale, annual food need, migration enabled/disabled, migration radius and the M8 spatial execution path. A control that is not explicitly swept uses its ordinary base command value.

A fresh sweep writes immutable `sweep-manifest.json` before point execution. It records the exact base settings, declared dimension values, seed definition, model identity and every expanded parameter point. Each point then lives under `experiments/point-XXXXXX/` as a normal M7 experiment, with its own immutable `experiment-manifest.json`, status files, retries and completed bundles. Retrying a sweep requires the exact same definition plus `--retry`; a changed grid, seed set or base control is rejected before child execution.

The sweep root also contains a deliberately separate `analysis/` directory:

```text
analysis/runs.json
analysis/runs.csv
analysis/points.json
analysis/points.csv
analysis/summary.json
```

These are **derived analysis artifacts**, not authoritative simulation state. Failed, incomplete, planned or otherwise non-completed runs remain explicit in the run table and point status counts; they are never silently folded into means.

The CSV files are intentionally ordinary rectangular tables with no special Rust tooling required. Python `pandas.read_csv(...)`, base R `read.csv(...)`, or equivalent tools can consume them directly.

## Reproducing the v0.1 reference experiment

The canonical source definition is [`experiments/v0.1-resource-variability.json`](experiments/v0.1-resource-variability.json). Build with an explicit source revision and launch it through the ordinary sweep path:

```text
ANTHROSIM_GIT_COMMIT="$(git rev-parse HEAD)" cargo build --locked --workspace --release
python3 scripts/run-versioned-sweep.py \
  experiments/v0.1-resource-variability.json \
  --binary target/release/anthrosim \
  --run-dir runs/v0.1-resource-variability
```

The launcher copies the exact definition, records its SHA-256, verifies the immutable sweep manifest against the requested seeds/settings/dimensions, and writes a reproduction record containing the model and source identity. The full contract and Windows/PowerShell equivalent are in [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md).

See [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md) for the M7 provenance/retry/sweep contract, [`docs/research/resource-variability-v0.1.md`](docs/research/resource-variability-v0.1.md) for the synthetic reference experiment, [`docs/research/spatial-mechanisms-v1.md`](docs/research/spatial-mechanisms-v1.md) for M8 transformation semantics, [`docs/research/spatial-observability-v1.md`](docs/research/spatial-observability-v1.md) for M8 observability, and [`docs/roadmap.md`](docs/roadmap.md) for the question-led development direction after M8.

## Scientific status

AnthroSim is a **research-oriented simulation framework, not a validated anthropological or archaeological model**. Its synthetic demographic, resource and migration presets remain explicit model assumptions. M8 demonstrates that the same deterministic experiment engine can bind provenance-tracked real-world-derived spatial evidence, transform it through declared assumptions and analyse spatial outcomes reproducibly.

That does not make a spatially grounded run a reconstruction of a real past population. Strong archaeological or anthropological claims still require question-specific evidence, calibration/validation where appropriate, uncertainty and sensitivity analysis, comparison against independent observations, and domain review.

Post-M8 development is intentionally question-led rather than a fixed feature list. The first evidence-grounded benchmark should inform which missing mechanism, alternative assumption, comparison method or archaeological observation layer is most scientifically useful next. See [`docs/roadmap.md`](docs/roadmap.md).

## Contributing and security

Contribution expectations are documented in [`CONTRIBUTING.md`](CONTRIBUTING.md). Please report security-sensitive issues according to [`SECURITY.md`](SECURITY.md) rather than publishing exploit details in a normal issue.

## License

AnthroSim is licensed under the [Apache License 2.0](LICENSE).
