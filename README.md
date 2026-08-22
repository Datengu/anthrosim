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
- deterministic multi-seed batch/ensemble execution with immutable provenance and retry state;
- deterministic parameter sweeps with derived machine-readable analysis tables;
- long-run correctness soaks and cross-artifact invariant validation;
- end-to-end performance and process-memory acceptance gates.

Culture, language, trade, states, religion, warfare, AI-controlled agents, and real-Earth palaeoenvironmental data are intentionally deferred.

## Current milestone status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local migration:** complete; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.
- **M5 — Events, metrics, checkpoints and causal inspection:** complete; it adds observability/persistence rather than a new anthropological mechanism.
- **M6 — Local simulation explorer:** complete as a read-only consumer of completed and paused M5 run bundles; it does not participate in authoritative simulation state or the Rust hot loop.
- **M7.1 — Batch / ensemble execution:** complete; one command launches deterministic explicit seed sets or seed ranges into isolated ordinary M5 run bundles.
- **M7.2 — Immutable experiment provenance and retry semantics:** complete; each planned run has an exact serialized configuration, explicit lifecycle state and deterministic reconciliation/retry behaviour.
- **M7.3 — Parameter sweeps and aggregate analysis outputs:** complete; explicit parameter grids expand deterministically into M7.2 experiments and produce separate derived CSV/JSON tables for downstream Python/R analysis.
- **M7.4 — Long-run soak and invariant hardening:** complete; automated long-duration/adversarial runs validate cross-artifact population, genealogy, household, resource, migration, event, metric and checkpoint consistency without weakening model invariants.
- **M7.5 — Performance and memory acceptance benchmarking:** implemented; the canonical 10,000-founder, 2,000-year observable lifecycle is measured end-to-end in release mode and CI gates material throughput/RSS regressions without introducing a special fast path.

M1–M4 establish the first closed spatial response loop: local synthetic productivity and seasonality create renewable resource supply; co-located households compete for finite stock; household supply affects individual condition and scarcity mortality; surviving households under local pressure can compare only bounded nearby alternatives and relocate together at an explicit travel cost; demographic births and baseline deaths continue through the M2 schedules.

M5 makes that loop inspectable. Births, deaths and completed household moves are emitted as versioned authoritative events; annual/terminal summaries are explicitly derived metrics; state digests fingerprint deterministic boundaries; and annual-boundary checkpoints preserve full dynamic state plus the exact positions of all named RNG streams for deterministic resumption.

M6 makes those artifacts navigable without changing them. It provides timeline, map, cell, household, person, genealogy and event views while visibly distinguishing serialized authoritative facts, recorded derived metrics and UI reconstructions. Historical state that M5 did not record is not silently invented.

M7.1–M7.3 add experiment orchestration around that same run path rather than a second simulator. Every ensemble or sweep child is created through the existing `Simulation` lifecycle and written with the existing completed M5 bundle format. The experiment layer records exactly which configurations were requested, whether each child is genuinely complete, and which derived summary rows were calculated from completed results; it does not change the underlying model.

M7.4 hardens that path rather than adding another model mechanism. Completed runs and checkpoints can now be subjected to a cross-artifact invariant validator, and ordinary CI includes long-duration stable, dynamic checkpoint/resume, adversarial scarcity, explicit terminal-state and multi-seed ensemble soaks. A checkpoint-schema ambiguity found around terminal record-limit boundaries is deliberately tracked as follow-up issue #31 rather than silently broadening the supported resume contract.

M7.5 establishes an explicit v0.1 engineering performance envelope. On the initial two-vCPU GitHub-hosted baseline, the ordinary event/metric-observable 10,000-founder workload completed all 2,000 requested years in about 73 seconds at about 27 simulated years/second and about 134 MiB peak RSS. CI keeps broad anti-regression floors around that measurement; no hot-path optimization or nondeterministic shortcut was justified by the evidence.

No historical destination, route, settlement, tribe or migration outcome is scripted into that loop.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. From the repository root, a small headless run can be executed with:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

For M5/M6 causal inspection, write a controlled completed run bundle instead:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --seed 1 --run-dir runs/m6-example
```

A completed directory contains `manifest.json`, `world.json`, `initial-population.json`, `events.json`, `metrics.json` and a final `checkpoint.json`, so analysis does not require a live database.

Open that run in the M6 explorer with:

```text
python scripts/serve-explorer.py runs/m6-example
```

The server binds to `127.0.0.1:8765` by default and opens the local browser. It serves only fixed explorer assets and expected run artifacts, exposes no directory listing or write API, and rejects POST/PUT/DELETE. Use `--no-browser` if you do not want it to open a browser automatically.

A deliberately paused run can be explored **before** resuming it:

```text
cargo run --release -p anthrosim-cli -- run --years 25 --population 10000 --seed 1 --run-dir runs/m5-resume --checkpoint-year 10
python scripts/serve-explorer.py runs/m5-resume
```

Paused checkpoint bundles do not yet have `manifest.json`; M6 recognises that form and treats `checkpoint.json` as the authoritative current boundary rather than fabricating a completed manifest. Resume later with:

```text
cargo run --release -p anthrosim-cli -- resume --checkpoint runs/m5-resume/checkpoint.json --run-dir runs/m5-resume
```

The CLI also exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

## Running deterministic ensembles

Launch an explicit deterministic seed set unattended:

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 25 \
  --population 10000 \
  --seeds 1,2,3,5,8 \
  --run-dir runs/example-ensemble
```

Or use a consecutive seed range:

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 25 \
  --population 10000 \
  --seed-start 100 \
  --seed-count 20 \
  --run-dir runs/example-range
```

A fresh ensemble writes `experiment-manifest.json` before child execution. That immutable, versioned manifest records the model identity and complete exact `ExperimentConfig` for every planned seed. `ensemble-plan.json` remains as the concise M7.1 planning view. Mutable per-run lifecycle records are written separately under `status/`.

Each seed receives its own stable directory such as `runs/seed-00000000000000000100/`, containing the ordinary six completed M5 artifacts plus `completion.json`. The positive completion marker is written only after the child bundle succeeds. A run status becomes `completed` only when that bundle reconciles with the exact immutable experiment definition.

If execution is interrupted or one run fails, rerun the **same command and exact configuration** with `--retry`:

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 25 \
  --population 10000 \
  --seed-start 100 \
  --seed-count 20 \
  --run-dir runs/example-range \
  --retry
```

Retry first requires exact equality with the stored immutable experiment manifest. It keeps provenance-valid completed runs without executing them again, reconciles interrupted/missing bundles as incomplete, and reruns only planned, failed or incomplete children. Partial child directories are removed before a retry attempt so old and new artifacts cannot be mixed. A completed bundle with conflicting provenance is treated as an integrity error instead of being silently overwritten.

The batch continues to later seeds after an individual child fails, but the overall command still exits unsuccessfully while any child is unsuccessful. Downstream analysis should treat only `completed` status records with provenance-valid bundles as successful results.

## Running deterministic parameter sweeps

M7.3 adds an explicit Cartesian parameter-grid layer. For example, this compares two M3 productivity settings and two annual food-need settings over the same four seeds:

```text
cargo run --release -p anthrosim-cli -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-annual-food-need 80,120 \
  --run-dir runs/resource-sweep
```

Supported sweep dimensions are founder population, target household size, M3 productivity scale, annual food need, migration enabled/disabled, and migration radius. A control that is not explicitly swept uses its ordinary base command value. The dimension order and value order are preserved deterministically when the Cartesian grid is expanded.

A fresh sweep writes immutable `sweep-manifest.json` before point execution. It records the exact base settings, declared dimension values, seed definition, model identity and every expanded parameter point. Each point then lives under `experiments/point-XXXXXX/` as a normal M7.2 experiment, with its own immutable `experiment-manifest.json`, status files, retries and completed M5 bundles. Retrying a sweep requires the exact same definition plus `--retry`; a changed grid, seed set or base control is rejected before child execution.

The sweep root also contains a deliberately separate `analysis/` directory:

```text
analysis/runs.json
analysis/runs.csv
analysis/points.json
analysis/points.csv
analysis/summary.json
```

These are **derived analysis artifacts**, not authoritative simulation state. `runs.*` contains one row for every planned run, including non-completed lifecycle states, exact point controls and source artifact paths. `points.*` contains per-parameter-point completion counts and simple completed-only descriptive means. Failed, incomplete, planned or otherwise non-completed runs remain explicit in the run table and point status counts; they are never silently folded into means. Each point summary lists the completed run IDs that contributed to its derived values.

The CSV files are intentionally ordinary rectangular tables with no special Rust tooling required. Python `pandas.read_csv(...)`, base R `read.csv(...)`, or equivalent tools can consume them directly. M7.3 does not add statistical inference, plotting or a general-purpose analysis framework to AnthroSim core.

See [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md) for the full M7.1–M7.3 provenance, retry, sweep and downstream-analysis contract, [`docs/research/soak-v0.1.md`](docs/research/soak-v0.1.md) for the M7.4 invariant and long-run soak boundary, and [`docs/benchmarks/m7-5-acceptance.md`](docs/benchmarks/m7-5-acceptance.md) for the M7.5 workload, measurements and acceptance limits.

For a completed run the manifest records configuration, artifact schema versions, world/population/resource/migration summaries, state digest, runtime counters and stop reason. For a paused run the checkpoint carries the current authoritative experiment/state boundary. See [`docs/research/observability-v0.1.md`](docs/research/observability-v0.1.md) for the authoritative-event/derived-metric distinction and checkpoint compatibility rules, and [`docs/research/explorer-v0.1.md`](docs/research/explorer-v0.1.md) for M6 display provenance and reconstruction limits.

## Scientific status

AnthroSim is **research-oriented software under active model development**, not a validated anthropological model. The current executable demographic, resource and migration presets are deliberately named `synthetic_validation_v1`. Their role is to verify mechanisms, invariants, deterministic replay and directional causal behaviour before empirical calibration/validation claims are attempted.

See [`docs/`](docs/) for the project vision, architecture, scientific-model specification, provenance notes and v0.1 scope.
