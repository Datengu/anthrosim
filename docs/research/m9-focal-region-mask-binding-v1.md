# M9 focal-region landscape-mask binding v1

## Scientific purpose

An M9 focal region controls where temporary visits can occur. When a region says it came from a landscape mask, AnthroSim must prove that the cells used by the simulation are exactly the cells selected by that mask. A matching citation or layer name alone is not sufficient evidence binding.

This contract prevents a run from retaining plausible landscape provenance while executing a hand-edited or stale `memberCells` set.

## `LandscapeMask` contract

For `FocalRegionSource::LandscapeMask`, execution is valid only when all of the following are true in the exact bound `LandscapeBundle`:

- the declared layer exists;
- its role is `Auxiliary`;
- its declared value domain is exactly `0..=1`;
- every cell has a binary value and no cell is nodata;
- the layer's `evidenceInputId` equals the source recorded by the focal region;
- the set of cells whose mask value is `1` exactly equals the focal region's authoritative `memberCells`;
- the landscape grid and authoritative world dimensions agree.

The member-cell equality check is set-equivalent through the canonical sorted `FocalRegion` representation, so order cannot hide a mismatch.

## Validation boundaries

`SpatialLandscapeSimulation` revalidates the binding:

1. before a new spatial run constructs the M9 program;
2. when a spatial checkpoint is resumed against its supplied landscape;
3. when a recorded spatial run is cross-artifact validated.

A serialized focal region therefore cannot rely on the fact that it was originally created with `FocalRegion::from_landscape_mask`; the claim is re-proven against the current bound landscape whenever it matters.

## Core-only execution policy

The ordinary core `Simulation` host has no authoritative `LandscapeBundle` available. It therefore rejects any M9 definition whose focal region claims `LandscapeMask` provenance.

Use:

- `FocalRegionSource::Synthetic` for explicit synthetic/null-model regions executed by `Simulation`;
- `FocalRegionSource::LandscapeMask` only with `SpatialLandscapeSimulation`, where the cited mask can actually be checked.

This distinction is about evidence integrity, not about giving synthetic and landscape runs different behavioral rules.

## Model and schema implications

This repair does not change the behavior of a valid, correctly bound landscape region and does not change synthetic-region behavior. It therefore does not require a model-semantics identity change or a focal-region schema change.

The M9.7 controlled aggregation benchmark is unaffected: both its continuous-residence and intermittent-aggregation definitions explicitly use `source.kind = "synthetic"`.

## Adversarial coverage

Regression tests fail closed when, after a region has been derived, any of the following is changed:

- `memberCells`;
- the declared mask layer ID;
- the mask layer's evidence input ID;
- a mask membership value;
- a mask cell to nodata.

Positive coverage also verifies exact landscape binding, JSON round-trip revalidation, spatial construction, and checkpoint-resume revalidation.
