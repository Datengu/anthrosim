# M8.3 deterministic landscape loading boundary

## Purpose

M8.3 is the first stage at which a normalized M8.1 landscape becomes an immutable input bound to an authoritative AnthroSim run.

The milestone deliberately separates **loading and identity** from **behavioural interpretation**. A successfully loaded landscape is validated, provenance-tracked spatial input, but its terrain, water and resource-opportunity values do not yet alter migration or resource equations. Those scientific transformations belong to M8.4.

This avoids introducing an unreviewed behavioural assumption merely because a spatial layer is available.

## Supported spatial-input modes

M8.3 preserves two explicit execution paths:

1. **Synthetic world mode** — the existing `anthrosim` runner and M1 world generator remain unchanged.
2. **Normalized landscape mode** — the new `anthrosim-landscape` runner validates an M8.1 `LandscapeBundle`, requires its dimensions to match the authoritative simulation grid, and binds its exact identity to the run through versioned M8.3 wrapper artifacts.

No implicit fallback is allowed. If a landscape-bound run is missing its landscape, receives an invalid bundle, or is resumed with modified contents, execution fails instead of silently substituting or regenerating spatial input.

## Why M8.3 does not map layer values into `Cell`

The existing `World::Cell` fields already have model meanings inherited from the synthetic M1-M4 mechanisms: `movement_cost`, `water_access`, `base_productivity`, seasonality and related quantities feed resource and migration behaviour.

A normalized layer named `terrain_traversal` is not automatically equivalent to `movement_cost`. A source-derived water-accessibility layer is not automatically interchangeable with the current synthetic migration utility term, and a resource-opportunity surface is not automatically a calibrated productivity surface.

Therefore M8.3 deliberately does **not** perform mappings such as:

```text
terrain_traversal -> movement_cost
water_accessibility -> water_access
resource_opportunity -> base_productivity
```

M8.4 owns those mappings, their explicit versioned parameters, evidence/provenance links, alternative plausible transformations and directional/sensitivity tests.

## Landscape identity

`LandscapeBinding` records the normalized spatial identity used by a run:

- binding schema version;
- normalized landscape identity;
- deterministic landscape digest;
- width and height;
- spatial reference;
- coordinate unit.

The original local filesystem path is deliberately absent from scientific identity. Copying identical normalized contents to another machine therefore preserves identity, while changing even one normalized landscape value changes the bundle digest and causes resume validation to fail.

Where an experiment contains an `EvidenceCatalog`, the landscape's evidence-input references are revalidated before execution or resume.

## Compatibility-preserving wrapper artifacts

M8.3 does not add optional fields to the existing M1-M7 `RunManifest` or `SimulationCheckpoint`. Doing so would unnecessarily alter their serialized form and weaken the project's byte-stable synthetic regression boundary.

Instead, landscape-bound runs use two additive versioned wrappers:

- `LandscapeRunManifest` contains a `LandscapeBinding` plus the unchanged core `RunManifest`;
- `LandscapeCheckpoint` contains the same `LandscapeBinding` plus the unchanged core `SimulationCheckpoint`.

This lets existing tooling continue to consume the familiar core artifacts while M8.3 adds a strict spatial-input identity layer around them.

A completed landscape-bound run directory contains:

```text
landscape.json
world.json
initial-population.json
manifest.json
landscape-manifest.json
events.json
metrics.json
checkpoint.json
landscape-checkpoint.json
```

`manifest.json` and `checkpoint.json` retain the pre-M8 core schemas. `landscape-manifest.json` and `landscape-checkpoint.json` are the M8.3 authoritative provenance wrappers tying those artifacts to the normalized landscape.

Synthetic `anthrosim` run bundles do not gain fake or empty landscape artifacts.

## CLI boundary

The dedicated runner makes the opt-in spatial mode explicit:

```text
anthrosim-landscape run \
  --landscape path/to/landscape.json \
  --seed 8301 \
  --years 100 \
  --population 1000 \
  --run-dir path/to/run
```

The simulation grid dimensions are taken from the validated landscape bundle. A caller cannot accidentally specify conflicting world dimensions on this command path.

A resumable run uses the same command with `--checkpoint-year`. Resume requires both the landscape-aware checkpoint wrapper and a normalized landscape:

```text
anthrosim-landscape resume \
  --checkpoint path/to/run/landscape-checkpoint.json \
  --landscape path/to/landscape.json \
  --run-dir path/to/run
```

The supplied landscape must reproduce the exact binding stored in the checkpoint. The ordinary core checkpoint alone is intentionally not sufficient to claim a landscape-bound resume.

## Checkpoint/resume contract

For normalized-landscape runs:

- `LandscapeCheckpoint` records the expected normalized landscape identity;
- resume requires an explicitly supplied landscape;
- schema, dimensions and normalized content identity are revalidated before core state restoration;
- a different or modified landscape is a compatibility error;
- no fallback to the M1 synthetic generator occurs;
- once binding validation succeeds, the unchanged core checkpoint is restored through the ordinary deterministic `Simulation::from_checkpoint` path.

For synthetic runs, the existing reconstruction from experiment configuration and seed remains unchanged.

## Cross-artifact invariants

`validate_landscape_recorded_run_invariants` requires:

- current landscape wrapper schemas;
- identical landscape bindings in landscape manifest and checkpoint;
- matching experiment identity in the embedded core manifest and checkpoint;
- matching terminal state digest in the embedded core manifest and checkpoint;
- every existing M1-M7 recorded-run invariant to pass on the wrapped core artifacts.

This adds spatial-input integrity without weakening or duplicating the established invariant validator.

## Determinism boundary

M8.3 loading consumes no RNG draws and performs no order-dependent GIS processing. M8.1/M8.2 have already normalized the landscape to deterministic row-major integer data.

Because landscape layers are behaviorally inert until M8.4, binding a landscape does not perturb demographic, resource or migration stochastic streams. The exact landscape is nevertheless authoritative provenance for the landscape-bound experiment.

A dedicated `Landscape loading determinism` GitHub Actions workflow runs a generic landscape fixture on Linux, Windows and macOS. On every platform it:

1. performs an uninterrupted run;
2. performs an otherwise-identical run paused at an annual checkpoint;
3. resumes using the exact same normalized landscape;
4. requires uninterrupted and resumed core/wrapper artifacts to reconcile;
5. emits a canonical landscape-bound golden record;
6. requires the three platform golden records to be byte-identical.

The existing synthetic cross-platform golden workflow remains independent and unchanged.

## Validation coverage

M8.3 automated coverage includes:

- same seed/config/landscape produces identical landscape-bound results;
- checkpoint/resume matches uninterrupted execution;
- a modified landscape is rejected on resume;
- mismatched landscape/world dimensions are rejected;
- wrapper manifest/checkpoint identities reconcile machine-readably;
- the generic landscape-bound fixture is byte-identical across supported CI platforms;
- all pre-M8 workspace tests, invariant checks, reference experiments and performance gates remain regression requirements.

## Model-semantics identity

M8.3 adds an opt-in input/provenance mode but deliberately does not change resource, migration or demographic equations. The existing synthetic execution path and its core artifacts remain semantically and deterministically compatible, so M8.3 does not advance the global `MODEL_SEMANTICS_ID` merely for the presence of an inert landscape binding.

M8.4 is expected to change scientific model semantics when normalized spatial values begin affecting authoritative model state and decisions. That transition must be versioned explicitly.

## Privacy/publication boundary

The public M8.3 implementation, documentation and tests use generic synthetic fixtures. No named locality, unpublished dataset or private archaeological question is required by the public core.

A private or restricted research project can keep its raw GIS and normalized landscape outside the public repository while preserving the exact M8.2 recipe/provenance, normalized bundle and experiment definition for authorized reproduction.
