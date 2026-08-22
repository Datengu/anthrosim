# Experiment execution in v0.1

This document records the M7.1–M7.3 batch, ensemble, sweep and downstream-analysis contract. It describes experiment orchestration and provenance only: it does not add an anthropological mechanism and does not create a second simulation implementation.

## Purpose

A useful research simulator must support repeated stochastic runs without requiring a person to relaunch each seed by hand, while making it impossible to confuse a retry with a different experiment. M7.1 added deterministic seed ensembles around the existing AnthroSim run lifecycle. M7.2 added immutable experiment identity, explicit per-run lifecycle state, reconciliation after interruption and deterministic retry behaviour. M7.3 adds deterministic parameter-grid expansion and explicitly derived analysis tables.

The scientific unit remains an ordinary AnthroSim run. Every child run has its own exact `ExperimentConfig`, authoritative M5 artifacts and final run manifest. The orchestration layer decides which runs were requested, records their immutable identities, tracks execution state and decides whether an exact retry is permitted. Sweep-level analysis consumes those run results; it never becomes authoritative simulation state.

## Seed definitions

An ensemble or sweep has exactly one of these seed definitions:

- an explicit ordered seed set supplied with `--seeds`, for example `--seeds 2,5,11,19`;
- a consecutive range supplied with `--seed-start` and `--seed-count`, for example `--seed-start 100 --seed-count 20`.

Duplicate explicit seeds are rejected because two identical seeds would target the same stable child directory. Zero-length ranges and ranges that overflow `u64` are also rejected.

The same seed definition and shared run controls produce the same ordered plan. AnthroSim intentionally preserves the order supplied by an explicit seed set rather than silently sorting it.

## Immutable experiment manifest

Before any child simulation starts, a fresh ensemble writes `experiment-manifest.json`. This is the authoritative experiment identity for M7.2. It contains:

- an experiment-manifest schema version;
- a deterministic experiment ID;
- the AnthroSim model/package version;
- the build git commit when supplied through `ANTHROSIM_GIT_COMMIT`;
- every planned run ID and stable output path;
- the complete exact `ExperimentConfig` for every child, including its seed.

The experiment ID is a stable FNV-1a fingerprint of the versioned identity payload. It is an identifier and accidental-change detector, not a cryptographic signature. Retry safety does not rely on the fingerprint alone: AnthroSim deserializes the stored manifest and requires exact structural equality with the experiment definition requested by the retry command.

AnthroSim never rewrites `experiment-manifest.json` during retry. Changing a seed, duration, world size, population setting, resource control, migration control, model version, git identity or any other serialized run configuration causes `--retry` to fail before child artifacts are touched.

`ensemble-plan.json` is retained as the concise M7.1 planning view for compatibility. `experiment-manifest.json` is the authoritative M7.2 provenance record.

## Stable output layout

For seed `42`, the child directory remains:

```text
runs/seed-00000000000000000042/
```

Each completed child directory contains the normal M5 completed bundle:

```text
manifest.json
world.json
initial-population.json
events.json
metrics.json
checkpoint.json
```

It also contains the M7.1-compatible positive completion marker:

```text
completion.json
```

`completion.json` is written only after all ordinary M5 child artifacts have been written successfully.

Mutable M7.2 lifecycle state is deliberately kept outside the child bundle:

```text
status/seed-00000000000000000042.json
```

This separation lets an incomplete child directory be discarded and deterministically rebuilt on retry without mutating the immutable experiment definition.

## Run lifecycle

Every planned child has an explicit versioned status record with:

- experiment ID, run ID and seed;
- lifecycle state;
- execution attempt number;
- optional failure/reconciliation message;
- for completed runs, the child manifest path and final state digest.

The lifecycle states are:

- `planned` — requested but not yet started;
- `running` — an attempt has begun;
- `completed` — the completed bundle and its provenance have reconciled successfully;
- `failed` — the last execution attempt returned an error;
- `incomplete` — a prior state or bundle does not prove successful completion and must be reconciled/retried.

A fresh experiment writes all planned status records before executing the first child. Immediately before a child starts, its status moves to `running` and its attempt number increments. It becomes `completed` only after the full child bundle and completion marker have been written. An execution error becomes `failed`, with the error text retained in the status record.

A batch continues to later planned seeds after one child fails, so unattended ensembles do not lose all remaining work because of one failed run. The overall command still exits unsuccessfully if any child remains failed, preventing a partially successful experiment from masquerading as fully successful.

## Retry and reconciliation

Retry uses the same ensemble command and exact original definition plus `--retry`:

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4,5 \
  --run-dir runs/resource-baseline-seeds \
  --retry
```

The requested definition must exactly match the stored immutable manifest. AnthroSim then reconciles each run independently.

A run is kept without re-execution only when its ordinary M5 bundle is complete and provenance-valid. Reconciliation requires all six normal completed artifacts plus `completion.json`, then verifies that the child `manifest.json` and final `checkpoint.json` agree with the exact planned `ExperimentConfig`, model identity and final state digest.

A valid completed bundle is retained even if its mutable status was left at `running` by an interruption after the bundle finished. A `completed` status with a missing artifact is downgraded to `incomplete` and rerun. Failed, incomplete and genuinely unfinished runs are retried from the exact planned configuration. Before such a retry, the incomplete child directory is removed so partial artifacts cannot be mixed with the new attempt.

A completed bundle whose serialized provenance conflicts with the immutable experiment is treated as an integrity error, not silently deleted or overwritten.

Re-running `--retry` after every child is already valid is idempotent: completed bundles are kept, attempt numbers do not increase and `experiment-manifest.json` is unchanged.

## Duplicate execution safeguards

M7.2 uses three layers to avoid accidental duplicate or ambiguous execution:

1. A fresh ensemble still refuses a non-empty output root.
2. Retry requires exact equality with the existing immutable experiment manifest.
3. Reconciliation keeps a provenance-valid completed child instead of executing it again.

These rules mean a retry can repair an interrupted experiment without silently changing what experiment was requested or overwriting a valid completed result.

## M7.3 parameter sweeps

A sweep is an explicit deterministic Cartesian product over supported experiment controls. In v0.1 the sweepable controls are:

- founder population;
- target household size;
- M3 resource productivity scale;
- annual food need;
- M4 migration enabled/disabled state;
- M4 local migration radius.

A dimension that is not supplied uses its ordinary base command value. Supplying a dimension preserves the explicit value order. The Cartesian expansion order is fixed by the implementation and therefore produces the same point sequence for the same sweep definition. Duplicate values inside one dimension are rejected because they would create scientifically redundant parameter points with indistinguishable configurations.

For example:

```text
cargo run --release -p anthrosim-cli -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-annual-food-need 80,120 \
  --run-dir runs/resource-sweep
```

This expands to four parameter points. Each point is then executed over seeds 1–4, for 16 planned ordinary AnthroSim runs.

M7.3 deliberately limits planning to 100,000 parameter points so an accidental combinatorial explosion is rejected before expensive execution begins. This is a planning safeguard, not a performance claim.

## Immutable sweep manifest

Before any point experiment starts, a fresh sweep writes `sweep-manifest.json`. It records:

- a sweep-manifest schema version;
- a deterministic sweep ID;
- model/package and optional git identity;
- the exact seed definition;
- all base controls;
- every explicitly declared sweep dimension and value order;
- every expanded point ID, stable point path and exact point settings.

The sweep ID fingerprints the complete versioned identity payload. As with experiment IDs, the fingerprint is an identifier rather than a cryptographic signature. Retry safety requires exact structural equality with the stored sweep manifest.

A sweep retry uses the identical command plus `--retry`. Changing a seed, base control, dimension, dimension value order, model version or build git identity causes retry to fail before point execution. The stored `sweep-manifest.json` is never rewritten during retry.

## Sweep point layout and relationship to M7.2

M7.3 does not invent a new child-run format. Each expanded point is an ordinary M7.2 experiment:

```text
experiments/point-000000/
  experiment-manifest.json
  ensemble-plan.json
  status/
  runs/
```

That means every sweep run retains the same exact per-run `ExperimentConfig`, M5 bundle, completion marker, lifecycle status and retry/reconciliation guarantees as an ordinary M7.2 ensemble.

On retry, a point that already has its experiment manifest is reconciled through M7.2. A point that had never begun can be created fresh from the immutable top-level sweep definition. A non-empty point directory without its expected experiment manifest is not silently repurposed.

## Derived analysis outputs

After point execution AnthroSim writes a separate `analysis/` directory:

```text
analysis/runs.json
analysis/runs.csv
analysis/points.json
analysis/points.csv
analysis/summary.json
```

These files are explicitly labelled `derived`. They are not checkpoints, run manifests, events, metrics or any other authoritative simulation state.

`runs.json` and `runs.csv` contain one row for **every planned run**, not just successes. Each row records:

- sweep, point, experiment and run identity;
- seed and exact swept control values;
- current lifecycle state and attempt count;
- relative source status path;
- for completed runs, the source run-manifest path;
- selected terminal descriptive values copied from the provenance-valid completed run manifest.

If a run is failed, incomplete, planned, running or otherwise not completed, that row remains present and terminal result fields remain empty. M7.3 therefore never hides a missing run by simply omitting it from the dataset.

`points.json` and `points.csv` contain one row per parameter point. They report planned, completed, failed, incomplete and other non-completed counts. The small descriptive means currently emitted—final living population, births and deaths—are calculated **only from completed runs**. Each point row also lists the exact completed run IDs that contributed to those means.

This completed-only rule is intentionally visible in field names such as `meanFinalLivingPopulationCompletedOnly`. A point with two successful and two failed seeds therefore reports `completedRuns = 2`, `failedRuns = 2`, and its mean is explicitly a mean of those two completed source runs—not a four-run result.

`analysis/summary.json` records how many run and point rows were emitted and how many planned runs were completed versus non-completed.

## Python and R consumption

The CSV artifacts are ordinary rectangular comma-separated tables. No AnthroSim-specific library is required. Typical downstream loading is therefore as simple as:

```python
import pandas as pd
runs = pd.read_csv("runs/resource-sweep/analysis/runs.csv")
points = pd.read_csv("runs/resource-sweep/analysis/points.csv")
```

or in base R:

```r
runs <- read.csv("runs/resource-sweep/analysis/runs.csv")
points <- read.csv("runs/resource-sweep/analysis/points.csv")
```

The JSON equivalents preserve null values and nested source-run ID lists for consumers that prefer structured records.

M7.3 intentionally does **not** add statistical inference, plotting, notebook machinery, sensitivity-analysis algorithms or a general-purpose statistics dependency to AnthroSim core. More sophisticated inference belongs in downstream scientific workflows where assumptions and methods can be chosen explicitly.

## Relationship to single-run execution

Batch and sweep execution do not fork the simulation logic. `run`, `ensemble`, and every sweep point construct ordinary `ExperimentConfig` values, initialize the same `Simulation`, execute the same `run_recorded()` lifecycle for completed bundles and use the same M5 artifact writer.

This is important scientifically. Experiment orchestration must not create a subtly different model path merely because many configurations or seeds are being run.

Existing `anthrosim run` and `anthrosim resume` behaviour remains unchanged.

## Scientific use of statuses and aggregates

Only runs in `completed` state with provenance-valid child bundles are successful experiment results. `failed`, `incomplete`, `planned` and `running` states are explicit non-results and must not be silently mixed into successful result sets.

M7.3 consumes that boundary rather than weakening it. Run-level derived tables preserve non-completed rows; point-level descriptive means use completed runs only and publish both completion counts and source run IDs. A researcher can therefore filter differently downstream if a study design requires it, but AnthroSim never silently pretends missing runs were successful.

## Example fresh ensemble

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 100 \
  --world-width 64 \
  --world-height 64 \
  --population 10000 \
  --seeds 1,2,3,4,5 \
  --run-dir runs/resource-baseline-seeds
```

All non-seed controls are shared across these child runs. Each exact per-run configuration is nevertheless expanded and serialized independently in the immutable experiment manifest, and each child manifest remains the authoritative record of what that run actually simulated.

## Scientific interpretation

Running many seeds or parameter points does not validate the model and does not turn synthetic assumptions into empirical parameters. An ensemble or sweep measures behaviour under declared executable models and configurations. Interpretation still has to respect the provenance and synthetic-validation boundaries documented for demography, resources and migration.

M7.3 provides reproducible experiment design and clean descriptive outputs; it does not establish that the parameter ranges themselves are archaeologically or anthropologically justified.