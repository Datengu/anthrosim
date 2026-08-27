# Spatial raster-resolution dependence v1

Status: implementation contract for issue #203.

## Scientific decision

AnthroSim does **not** currently claim that evidence-grounded spatial behaviour is invariant to
raster resolution. The supported contract is therefore explicit cell-space semantics:

`cell_space_resolution_dependent_v1`

This is the conservative branch of #203's acceptance criterion. It records raster resolution as a
scientifically consequential model input and requires resolution sensitivity/convergence work before
spatial conclusions are described as scale-independent physical inference.

No artificial metre-to-food, metre-to-reproductive-contact, metre-to-migration or metre-to-travel
conversion is introduced in this contract. Those conversions would themselves be model assumptions
requiring evidence, units and validation. A future physically normalized model may add a distinct
semantics identity only when it has executable, tested meaning.

## Machine-readable preservation

`LandscapeBinding` schema v2 preserves `SpatialScaleAssessment` alongside the exact landscape
identity. The assessment records:

- `cellSizeX` and `cellSizeY`;
- coordinate unit;
- `m2InteractionBasis = exact_persistent_residence_cell`;
- `resourceQuantityBasis = per_cell_total`;
- `m4DistanceBasis = grid_steps`;
- `m9TravelCostBasis = grid_edges`;
- `status = resolution_dependent`;
- `requiresResolutionSensitivity = true`.

The binding is present in landscape-backed manifests and checkpoints. Resume validation reconstructs
it from the supplied landscape and rejects mismatches, so a different resolution or forged scale
assessment cannot silently inherit the original run identity.

This assessment is intentionally separate from evidence closure. A run may have fully closed,
content-bound empirical inputs while its executable spatial model remains resolution-dependent.
`evidenceClosure = closed` therefore does **not** imply that a spatial result is physically
scale-independent.

## Current quantity meanings

### M2 reproductive locality

M2 builds its eligible male-parent pool by exact equality of the female parent's persistent-residence
`CellId`. It has no physical reproductive-contact radius or settlement-unit abstraction. Raster
partitioning is therefore an interaction boundary: two people at unchanged nearby physical
positions can share one coarse cell but occupy different fine cells, changing whether a local male
parent is available before fertility probability is even drawn.

This contract records that rule as `exact_persistent_residence_cell`. It does not change M2
parent-selection behavior. #228 remains responsible for richer observability of fertility
suppression caused by absence of a local eligible male.

### M3 resources

Model-facing `base_productivity` and the derived food stock/capacity/regeneration quantities are
cell totals. They are not currently interpreted as densities per square metre or other unit area.
Subdividing one physical cell into several cells while retaining the same model-facing productivity
value therefore increases aggregate resource opportunity.

### M4 permanent migration

`candidateRadiusCells`, Manhattan distance, relocation risk per cell and travel-condition cost per
cell are grid-step quantities. Physical cell size is not consumed by M4. The same three-cell search
radius therefore spans 300 m on a 100 m raster and 150 m on a 50 m raster.

The #203 contract does not alter M4 production code. This keeps the scale finding independent of
#188's active kin-proxy repair while preserving the exact current causal semantics.

### M9 temporary travel

M9 route cost sums model-facing movement cost over grid edges. Edge length is not multiplied into
the cost. A 200 m uniform route represented by two 100 m edges therefore accumulates half the cost
of the same physical distance represented by four 50 m edges under otherwise identical settings.

## Metamorphic non-convergence fixture

`spatial_resolution_dependence.rs` fixes simple physical scenarios and varies only the raster
representation. It proves the currently declared dependence:

1. Two hypothetical residents remain 50 m apart. They share one 100 m cell but occupy separate 50 m
   cells, while M2's interaction basis remains exact persistent-residence-cell equality. The same
   physical reproductive neighbourhood can therefore change its eligibility structure solely from
   raster partitioning.
2. A 200 m x 200 m surface represented as 2x2 100 m cells versus 4x4 50 m cells produces four times
   the aggregate initial M3 food stock at the finer resolution when per-cell productivity is held
   constant.
3. Two 700 m x 700 m rasters preserve the same `candidateRadiusCells = 3`, but the M4 physical
   horizon changes from 300 m to 150 m.
4. Two uniform 200 m cardinal M9 routes contain two 100 m edges versus four 50 m edges. Accumulated
   cost changes from 2,000 to 4,000 abstract units and, under the synthetic validation travel model,
   travel duration changes from one to two model days.
5. Tampering with the preserved scale assessment causes landscape-binding validation to fail.

These are intentionally **failing convergence properties expressed as passing tests of the declared
model contract**. They ensure a future change cannot silently claim that cell size is incidental
while retaining the same behaviour.

## Research-use rule

Before interpreting evidence-grounded reproductive opportunity, spatial absence, migration
catchment, route accessibility, resource pressure, settlement concentration or related quantities
physically, a study must do one of the following:

- run a predeclared raster-resolution sensitivity/convergence design over plausible resolutions and
  show that the claim is stable enough for its intended interpretation; or
- explicitly report material resolution dependence and condition the claim on the selected raster;
  or
- use a later, separately identified physically normalized spatial model whose reproductive-contact,
  resource-conservation and distance semantics have been validated.

Resolution changes must preserve the same physical study extent and derive source fields
consistently. Raster resolution and study-area extent are separate variables: #211 handles boundary
extent/crop effects and should not be confounded with this scale test.

## What this contract does not claim

This work does not establish a physically calibrated reproductive-contact radius, resource density,
walking speed, mobility radius, relocation cost or travel-energy model. It does not make the M8.6
terrain transformation an empirical movement law. It does not make one raster resolution preferable
for Bulstrode Camp or any other case study.

Its scientific purpose is narrower and necessary: raster resolution is no longer a hidden numerical
choice that can be mistaken for irrelevant GIS metadata.

## Provenance and semantics identities

No authoritative population/resource/migration/M9 trajectory rule changes in this contract, so the
core `MODEL_SEMANTICS_ID` is unchanged. The existing spatial transformation semantics identity is
also unchanged because the transformation and movement algorithms are unchanged.

The new interpretation is versioned independently through `SpatialScaleAssessment` schema v1 and
`LandscapeBinding` schema v2. A future physical-normalization mechanism that changes executable
resource, reproductive-contact or movement behavior must receive its own appropriate model/spatial
semantics identity.
