# Experiment execution in v0.1

This document records the M7.1–M7.2 batch/ensemble execution contract. It describes experiment orchestration and provenance only: it does not add an anthropological mechanism and does not create a second simulation implementation.

## Purpose

A useful research simulator must support repeated stochastic runs without requiring a person to relaunch each seed by hand, while making it impossible to confuse a retry with a different experiment. M7.1 added deterministic seed ensembles around the existing AnthroSim run lifecycle. M7.2 adds immutable experiment identity, explicit per-run lifecycle state, reconciliation after interruption and deterministic retry behaviour.

The scientific unit remains an ordinary AnthroSim run. Every child run has its own exact `ExperimentConfig`, authoritative M5 artifacts and final run manifest. The ensemble layer decides which runs were requested, records their immutable identities, tracks execution state and decides whether an exact retry is permitted.

## Seed definitions

An ensemble has exactly one of these seed definitions:

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

## Relationship to single-run execution

Batch execution does not fork the simulation logic. Both `run` and `ensemble` construct the same `ExperimentConfig`, initialize the same `Simulation`, execute the same `run_recorded()` lifecycle for completed bundles and use the same M5 artifact writer.

This is important scientifically. Batch/retry machinery must not create a subtly different model path merely because many seeds are being run.

Existing `anthrosim run` and `anthrosim resume` behaviour remains unchanged.

## Scientific use of statuses

Only runs in `completed` state with provenance-valid child bundles should be treated as successful experiment results. `failed`, `incomplete`, `planned` and `running` states are explicit non-results and must not be silently mixed into successful result sets.

M7.2 deliberately does not calculate experiment-level statistics. M7.3 will add parameter sweeps and aggregate analysis outputs and must consume this lifecycle/provenance boundary rather than averaging partial runs implicitly.

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

Running many seeds does not validate the model and does not turn synthetic assumptions into empirical parameters. An ensemble measures variation under a declared executable model and configuration. Interpretation still has to respect the provenance and synthetic-validation boundaries documented for demography, resources and migration.

Parameter sweeps and aggregate machine-readable analysis outputs remain deliberately deferred to M7.3 so derived experiment summaries stay distinct from authoritative per-run artifacts and M7.2 lifecycle state.
