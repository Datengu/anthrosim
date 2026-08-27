# M8 external landscape preprocessing workflow v2

## Purpose

Preprocessing workflow v2 extends the original M8.2 reproducibility boundary with an explicit orientation contract. External GIS software may still reproject, resample, clip and derive layers, but the aligned grids handed to AnthroSim can no longer rely on an unstated raster orientation.

This document supplements `landscape-contract-v2.md`. The scientific transformations themselves remain project-specific and must still be justified independently.

## Recipe v2 requirements

Recipe schema version 2 adds two mandatory machine-readable fields:

- `gridConvention`: the convention the emitted `LandscapeBundle` will use;
- `alignedInputGridConvention`: the convention already used by the aligned CSV layer inputs.

Both currently must equal:

```json
{
  "originAnchor": "upper_left_outer_corner",
  "columnDirection": "increasing_x",
  "rowDirection": "decreasing_y",
  "cellInterpretation": "area"
}
```

The assembler does **not** transform orientation. If an input CSV is south-up, transposed, point-sampled, or anchored at cell centres, it must be corrected externally before assembly and that correction must appear in the recorded GIS steps.

## Required external orientation work

A reproducible preparation should establish, before CSV export:

1. the target spatial reference and the stored integer coordinate unit;
2. the target outer extent and resolution;
3. the upper-left outer-corner origin;
4. columns ordered toward increasing X;
5. rows ordered toward decreasing Y;
6. pixel-as-area interpretation;
7. any source-to-target flip, rotation, reprojection or resampling required to reach that convention.

For a north-up raster this normally means exporting the highest-Y row first. A source that uses another convention is not invalid, but the external workflow must transform it deliberately rather than expecting AnthroSim to infer what was meant.

## Validation and reproduction record

`build-landscape-bundle.py` now:

- requires recipe schema v2;
- requires both convention declarations;
- rejects any unsupported or mismatched convention;
- records the normalized convention;
- records the aligned-input convention;
- records that same convention alongside each aligned layer input digest;
- emits landscape schema v2.

Existing shape, value-domain, nodata, source-hash and deterministic-output validation remains unchanged.

The reproduction record therefore proves not only which bytes were assembled, but also the spatial row/column interpretation those bytes were asserted to have at the AnthroSim boundary.

## Coordinate units

The assembler treats `coordinateUnit` as the unit of the stored integer geometry. It does not parse the declared `spatialReference` or convert between native CRS units.

If a workflow stores an alternate integer representation—for example integer arcseconds referenced to EPSG:4326—that conversion must be explicit in the preparation and `coordinateUnit` must name the stored unit. A researcher must not describe the output as machine-validated native CRS coordinates merely because an EPSG identifier is present.

## Relationship to source provenance

The generic recipe can include raster, vector or derived sources, so it does not assume every source has one raster row convention. Source orientation and any transformation into the aligned grid belong in the recorded preprocessing steps and source-specific provenance.

The **validated machine boundary** is the aligned input: once a CSV is declared to use the supported convention, the assembler will preserve its row-major ordering exactly and will not silently flip or transpose it.

For sources with a well-defined raster convention, source-specific provenance should record it directly. The M8.6 HGT benchmark, for example, records its north-to-south source row order and emits the same supported normalized convention.

## Scientific boundary

Orientation reproducibility does not validate the underlying archaeological reconstruction. It only ensures that a GIS preparation cannot be spatially mirrored or shifted merely because two tools interpreted "row-major" or `originX/originY` differently.

Raster resolution effects, physical movement scaling and resource-area scaling remain issue #203 rather than being hidden inside preprocessing v2.