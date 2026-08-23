# M8.3 deterministic landscape loading boundary

## Purpose

M8.3 is the first stage at which a normalized M8.1 landscape becomes an input to an authoritative AnthroSim run.

The milestone deliberately separates **loading and identity** from **behavioural interpretation**. A successfully loaded landscape is immutable, validated, provenance-tracked simulation input, but its terrain, water and resource-opportunity values do not yet alter migration or resource equations. Those scientific transformations belong to M8.4.

This avoids introducing an unreviewed assumption merely because a spatial layer is available.

## Required behaviour

An M8.3 run must distinguish two spatial-input modes explicitly:

1. **Synthetic world mode** — the existing M1 world generator remains the complete spatial input and retains its existing deterministic behaviour.
2. **Normalized landscape mode** — a validated `LandscapeBundle` is bound to the run in addition to the existing model-facing world state. The landscape dimensions must match the authoritative world topology, and the exact landscape identity must be preserved in run/checkpoint provenance.

No implicit fallback is allowed. If an experiment declares a normalized landscape but that landscape is missing, modified, invalid or dimensionally incompatible, execution/resume must fail explicitly rather than regenerate or substitute a synthetic landscape silently.

## Why M8.3 does not yet map layer values into `Cell`

The current `World::Cell` fields have scientific meanings inherited from the synthetic M1-M4 model: `movement_cost`, `water_access`, `base_productivity`, seasonality and related quantities already feed resource and migration mechanisms.

A normalized M8 layer such as `terrain_traversal` is not automatically equivalent to `movement_cost`. Likewise, a source-derived water-accessibility layer is not automatically interchangeable with the current synthetic migration utility term, and a resource-opportunity surface is not automatically a calibrated productivity surface.

Therefore M8.3 must not perform mappings such as:

```text
terrain_traversal -> movement_cost
water_accessibility -> water_access
resource_opportunity -> base_productivity
```

without an explicit, versioned scientific transformation. M8.4 owns those mappings and their directional/sensitivity tests.

## Identity contract

A normalized-landscape run must preserve at minimum:

- landscape schema version;
- stable normalized landscape identity/digest;
- grid width/height;
- spatial reference metadata through the preserved landscape artifact;
- the experiment configuration and model semantics identity;
- evidence links already present in the `LandscapeBundle`/`EvidenceCatalog` boundary.

The landscape path on one machine is **not** scientific identity. Paths are runtime locators only. A copied landscape with the same normalized contents must retain the same identity; a modified normalized value must produce a different identity.

## Run artifacts

A controlled M8.3 run should preserve the exact normalized landscape alongside the ordinary run artifacts, using a stable artifact name such as `landscape.json`.

The run/checkpoint provenance must identify that artifact by normalized landscape identity rather than trusting the filename alone.

Existing synthetic run bundles must not gain a fake or empty landscape artifact merely to satisfy the new code path.

## Checkpoint/resume

Checkpoint/resume must preserve the same spatial-input mode.

For normalized-landscape runs:

- the checkpoint records the expected normalized landscape identity;
- resume requires the caller to supply or resolve a landscape with exactly that identity;
- schema, dimensions and content identity are revalidated before state restoration;
- a missing or different landscape is an integrity/compatibility error;
- the immutable landscape does not need to be duplicated inside every checkpoint if the controlled run bundle preserves it separately.

For synthetic runs, the existing deterministic reconstruction from experiment configuration + seed remains unchanged.

## Determinism boundary

M8.3 loading itself consumes no RNG draws and performs no order-dependent GIS processing. The normalized bundle is already row-major and deterministic by M8.1/M8.2.

Binding a landscape must therefore not perturb unrelated stochastic streams merely by being present. Before M8.4 introduces explicit layer-to-mechanism transformations, otherwise-identical synthetic model state should evolve identically when the bound landscape is behaviorally inert.

The landscape identity is still part of experiment/run provenance even when it is behaviorally inert, because it declares the exact external spatial input associated with the experiment.

## Validation requirements

M8.3 tests should cover at least:

- a valid normalized landscape binds successfully when dimensions match;
- invalid landscape schema/content is rejected;
- mismatched dimensions are rejected;
- a run that declares/requires a landscape cannot start without it;
- a synthetic run continues through the existing API unchanged;
- normalized-landscape checkpoint/resume matches uninterrupted execution when supplied the same landscape;
- resume rejects a landscape whose normalized content identity differs;
- the presence of an inert M8.3 landscape does not change pre-M8.4 demographic/resource/migration outcomes for an otherwise identical world/configuration;
- landscape identity appears in the appropriate run/checkpoint provenance;
- controlled run bundles preserve `landscape.json` only when applicable.

## Model-semantics identity

M8.3 adds a new supported input/binding mode but deliberately does not change the resource, migration or demographic equations. Existing synthetic runs must remain semantically and deterministically compatible.

Whether the global `MODEL_SEMANTICS_ID` needs to advance should therefore be decided by the implementation boundary actually chosen: if existing supported checkpoint semantics or authoritative state meaning changes, advance it; if M8.3 is an additive opt-in binding path that leaves existing synthetic semantics untouched, keep the existing identity for synthetic runs and make the new spatial binding explicit in its own provenance.

M8.4 is expected to change scientific model semantics when normalized spatial values begin affecting authoritative decisions/resources, and that transition must be versioned explicitly.

## Privacy/publication boundary

The public M8.3 implementation and tests use generic/synthetic fixtures. A private research project can bind an unpublished landscape without committing its location, source files or research question to the public repository.

Reproducibility for authorized collaborators still requires the private research archive to preserve the normalized landscape, recipe/provenance and exact experiment definition.
