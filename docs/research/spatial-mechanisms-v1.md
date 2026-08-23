# M8.4 spatial mechanisms v1

## Purpose

M8.4 is the boundary at which selected normalized landscape values are allowed to change authoritative AnthroSim model state.

M8.1 defines the normalized landscape contract. M8.2 defines reproducible external GIS preprocessing. M8.3 binds an exact normalized landscape to a run while keeping its layer values behaviourally inert. M8.4 adds an explicit, versioned transformation from selected normalized layers into model-facing fields that existing M3 resource and M4 migration mechanisms already consume.

This is a scientific-model boundary, not a rendering feature. Loading a DEM, water surface or reconstructed environmental layer does not by itself make AnthroSim archaeologically valid.

## Explicit targets

The v1 transformation layer can replace three existing `World::Cell` fields:

- `movement_cost` from a declared `terrain_traversal` layer;
- `water_access` from a declared `water_accessibility` layer;
- `base_productivity` from a declared `resource_opportunity` layer.

A target omitted from `SpatialMechanismConfig` retains its deterministic synthetic-world value. This makes partial/null-model experiments possible without manufacturing data for every environmental field.

When `base_productivity` is replaced, initial `food_stock` is recalculated using the same `INITIAL_FOOD_STOCK_MULTIPLIER` relationship used by synthetic world generation. The existing M3 productivity scale remains an independent experiment parameter.

M8.4 does not add settlement attractiveness, known-site preference, defence value, territoriality, exchange value, route importance or any other historically targeted utility.

## Transformation configuration

A `SpatialMechanismConfig` is versioned, serializable and stored in transformed run/checkpoint provenance. Every transform declares:

- an exact source `layerId`;
- the expected landscape role;
- the expected source unit;
- the exact source value domain;
- one model-facing target field;
- target minimum and maximum;
- direct or inverse direction;
- an explicit nodata policy;
- an optional `evidenceId` linking the transformation assumption to the experiment `EvidenceCatalog`.

Broad layer role alone never selects data. Exact layer IDs prevent an experiment from silently changing meaning when a bundle contains multiple plausible layers.

## Integer mapping

Authoritative v1 transformations use integer arithmetic only.

For a source domain `[Smin, Smax]`, source value `S`, and target domain `[Tmin, Tmax]`:

```text
source_span = Smax - Smin
position    = S - Smin                  # direct
position    = source_span - (S - Smin)  # inverse
mapped      = Tmin + floor(position * (Tmax - Tmin) / source_span)
```

The source value must already lie inside the declared source domain. The transform does not normalize, interpolate, reproject or infer palaeoenvironmental meaning. Those operations belong in the documented M8.2 preprocessing recipe.

The initial v1 family is intentionally simple and monotonic. More sophisticated cost surfaces are not automatically more defensible; they should be introduced only when a scientific question and evidence justify the additional assumptions.

## Target constraints

- movement cost must remain at or above the existing `BASE_MOVEMENT_COST` baseline;
- water accessibility is constrained to `0..=1000`;
- base productivity is constrained to `0..=1000`.

These constraints preserve the domains expected by the existing resource/migration mechanisms.

## Nodata

Nodata is never silently interpreted as zero.

Each transform chooses one of:

- `reject`: any missing source cell makes the transform fail;
- `constant`: replace missing cells with one explicitly declared source-domain value.

Interpolation, gap filling or source reconstruction should normally happen in M8.2 so the method and provenance remain visible. A constant replacement is part of transformation identity and therefore part of reproducibility.

## Evidence linkage

A transform may omit `evidenceId` for synthetic validation and exploratory/null-model work.

If `evidenceId` is present:

1. the experiment must include an `EvidenceCatalog`;
2. the catalogue must itself validate;
3. the exact evidence record must exist;
4. the evidence identifier contributes to spatial-mechanism identity.

This records support for the **mapping assumption** separately from provenance for the source spatial layer. Evidence traceability does not prove that the estimate or mapping is correct.

## Authoritative world construction

Transformed execution uses `SpatialLandscapeSimulation`, not the legacy synthetic `Simulation` constructor.

The sequence is deterministic:

1. validate experiment, landscape, evidence and transformation contracts;
2. generate the same deterministic synthetic baseline `World` for the experiment seed/grid;
3. transform selected landscape layers into model-facing vectors;
4. overlay only declared targets on the baseline world;
5. validate the resulting world;
6. initialize population, resources and migration against that transformed world;
7. run the existing M2-M4 lifecycle with the same named RNG streams.

The original normalized `LandscapeBundle` is retained unchanged. `world.json` is the model-facing transformed state. They are intentionally separate artifacts.

## Why the synthetic baseline still exists underneath

M8.4 does not yet provide evidence-grounded replacements for every environmental property used by the current world model, such as seasonality. Keeping undeclared fields on the deterministic synthetic baseline makes missing assumptions explicit and allows controlled incremental experiments.

A real-landscape result must therefore state which fields were evidence-grounded and which retained synthetic/default semantics. M8.6 must not describe a partially grounded environment as a complete palaeolandscape reconstruction.

## Semantics identity

The existing core `MODEL_SEMANTICS_ID` remains the compatibility identity for unchanged M1-M7 demographic/resource/migration equations and synthetic checkpoints.

M8.4 adds a separate:

```text
anthrosim-spatial-transform-semantics-v1
```

This `SPATIAL_MODEL_SEMANTICS_ID`, the full `SpatialMechanismConfig`, its deterministic config identity, and the transformed-world digest are stored in the outer landscape run/checkpoint wrappers.

This split prevents an opt-in transformed landscape mode from unnecessarily invalidating old synthetic checkpoints while still making transformed spatial semantics strict and resume-safe.

## Checkpoint and resume

A transformed checkpoint preserves:

- exact normalized landscape identity;
- spatial transformation semantics identity;
- full transformation configuration;
- deterministic transformation-config identity;
- transformed authoritative world digest;
- unchanged core checkpoint state and RNG positions.

Resume requires the same normalized landscape. AnthroSim reconstructs the transformed world from landscape + stored transformation configuration + experiment seed, verifies the spatial semantics/config/world identities, then restores dynamic state. A modified landscape, altered transform, unsupported spatial semantics ID or mismatched world digest is rejected.

The transformation configuration is recovered from the checkpoint; callers cannot resume by supplying a different mechanism file.

## CLI modes

`anthrosim-landscape run` remains backwards compatible with M8.3:

```text
# M8.3 inert landscape control
anthrosim-landscape run --landscape landscape.json ...

# M8.4 transformed landscape
anthrosim-landscape run --landscape landscape.json --mechanisms spatial-mechanisms.json ...
```

For M8.4 bundles, `spatial-mechanisms.json` is preserved alongside `landscape.json`, `world.json`, the core artifacts, and the landscape wrapper artifacts.

`resume` auto-detects whether the wrapper contains a spatial binding. No mechanism path is accepted on resume because the exact transformation is already part of the checkpoint contract.

## M7 ensemble and sweep integration

M8.4 spatial execution is also available through the ordinary M7 experiment machinery:

```text
anthrosim ensemble \
  --landscape landscape.json \
  --mechanisms spatial-mechanisms.json \
  ...

anthrosim sweep \
  --landscape landscape.json \
  --mechanisms spatial-mechanisms.json \
  --sweep-annual-food-need 80,100 \
  ...
```

The filesystem paths are runtime locators only. They are deliberately excluded from experiment and sweep identity. The immutable scientific identity contains instead:

- the exact `LandscapeBinding` derived from landscape content;
- `SPATIAL_MODEL_SEMANTICS_ID`;
- the full `SpatialMechanismConfig`.

A spatial ensemble uses experiment-manifest schema v2, while unchanged synthetic ensembles retain the existing schema-v1 serialized form. This prevents an older reader from silently treating a spatial experiment as an ordinary synthetic experiment while preserving the identity of existing synthetic experiment definitions.

A fresh spatial experiment preserves `landscape.json` and `spatial-mechanisms.json` at the experiment root as well as inside completed run bundles. Retry validates the immutable definition and exact preserved landscape before retaining or rerunning child runs. Runtime machine paths therefore do not need to remain stable across machines.

A spatial sweep carries one immutable landscape/mechanism binding through all of its M7 parameter points. Ordinary M7 dimensions such as population, resource scale, annual need and migration settings can still vary. Competing spatial transformation models should normally be represented as separate declared sweep/experiment definitions rather than hiding multiple scientific models behind path-valued sweep parameters.

## Validation boundary

M8.4 verifies engineering and model-direction properties such as:

- exact layer/unit/domain selection;
- monotonic direct/inverse transformation;
- explicit nodata handling;
- deterministic transformed `World` fields;
- productivity affecting M3 initial resource state through the existing resource path;
- movement cost and water accessibility entering the existing M4 migration utility through the authoritative transformed `World`;
- controlled migration traces selecting higher water security when water is the isolated positive utility and lower travel penalty when transformed terrain cost is the isolated differentiator;
- same input producing identical output;
- checkpoint/resume matching uninterrupted execution;
- transformation parameters changing spatial identity and authoritative world digest;
- transformed execution and exact retry through M7 ensemble/sweep machinery;
- cross-platform byte-identical canonical transformed outputs.

These checks demonstrate that declared spatial assumptions are applied consistently. They do **not** demonstrate that a mapping is anthropologically or archaeologically correct.

## Interpretation boundary

A transformed run is still a model result. In particular:

- a normalized layer may be uncertain or reconstructed;
- the linear transformation may be only one plausible model;
- the existing migration/resource mechanisms may be incomplete;
- synthetic fields may remain in the world;
- agreement with a known archaeological pattern does not by itself establish causation;
- mismatch is a valid result and may indicate missing mechanisms or unsuitable assumptions.

M8.4 exists so those assumptions can be represented, varied and falsified explicitly. M8.5 adds spatial observability; M8.6 uses the machinery in a predeclared evidence-grounded null-model benchmark.
