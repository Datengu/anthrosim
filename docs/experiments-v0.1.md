# Experiment execution in v0.1

This document records the M7.1 batch/ensemble execution contract. It describes orchestration only: M7.1 does not add a new anthropological mechanism and does not create a second simulation implementation.

## Purpose

A useful research simulator must support repeated stochastic runs without requiring a person to relaunch each seed by hand. M7.1 therefore adds deterministic seed ensembles around the existing AnthroSim run lifecycle.

The scientific unit remains an ordinary AnthroSim run. Every child run has its own exact `ExperimentConfig`, authoritative M5 artifacts and final run manifest. The ensemble layer only decides which seeds to launch and where those ordinary bundles are written.

## Seed definitions

An ensemble has exactly one of these seed definitions:

- an explicit ordered seed set supplied with `--seeds`, for example `--seeds 2,5,11,19`;
- a consecutive range supplied with `--seed-start` and `--seed-count`, for example `--seed-start 100 --seed-count 20`.

Duplicate explicit seeds are rejected because two identical seeds would target the same stable child directory. Zero-length ranges and ranges that overflow `u64` are also rejected.

The same seed definition and shared run controls produce the same ordered plan. M7.1 intentionally preserves the order supplied by an explicit seed set rather than silently sorting it.

## Stable output layout

Before the first simulation is executed, AnthroSim writes `ensemble-plan.json` to the requested root. Its schema version, seed definition, shared simulation controls and stable child paths make the requested M7.1 plan inspectable before any run is counted as complete.

For seed `42`, the child directory is:

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

M7.1 then writes:

```text
completion.json
```

`completion.json` is a positive completion marker only. It is written after all ordinary M5 artifacts for that child have been written successfully and points back to `manifest.json`.

The ensemble command refuses to start if its requested root already contains files or directories. This is deliberately conservative: M7.1 will not overwrite an earlier run, merge two result sets implicitly, or infer that an existing partial directory is safe to reuse.

## Relationship to single-run execution

M7.1 does not fork the simulation logic. Both `run` and `ensemble` construct the same `ExperimentConfig`, initialize the same `Simulation`, execute the same `run_recorded()` lifecycle for completed bundles and use the same M5 artifact writer.

This is important scientifically. Batch execution must not create a subtly different model path merely because many seeds are being run.

Existing `anthrosim run` and `anthrosim resume` behaviour remains unchanged.

## Failure boundary in M7.1

M7.1 is intentionally simpler than the complete M7 experiment system.

If an ensemble is interrupted or one child returns an error:

- `ensemble-plan.json` still records every run that was requested;
- children that fully completed have `completion.json`;
- a child without `completion.json` must not be inferred to have completed successfully;
- M7.1 does not automatically retry, reconcile, resume or mutate the existing ensemble directory.

That conservative limitation prevents M7.1 from inventing incomplete retry semantics that would later become part of scientific provenance.

M7.2 is responsible for the stronger contract: immutable experiment manifests, explicit planned/running/completed/failed/incomplete lifecycle states, deterministic retries, duplicate-execution safeguards and status reconciliation after interruption.

## Example

```text
cargo run --release -p anthrosim-cli -- ensemble \
  --years 100 \
  --world-width 64 \
  --world-height 64 \
  --population 10000 \
  --seeds 1,2,3,4,5 \
  --run-dir runs/resource-baseline-seeds
```

All non-seed controls are shared across these child runs. Each child manifest remains the authoritative record of the exact configuration actually simulated.

## Scientific interpretation

Running many seeds does not validate the model and does not turn synthetic assumptions into empirical parameters. An ensemble measures variation under a declared executable model and configuration. Interpretation still has to respect the provenance and synthetic-validation boundaries documented for demography, resources and migration.

M7.1 also does not perform statistical aggregation. Parameter sweeps and aggregate machine-readable analysis outputs are deliberately deferred to M7.3 so derived experiment summaries remain distinct from authoritative per-run artifacts.
