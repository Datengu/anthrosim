# M8 normalized landscape contract v2 — explicit grid geometry

## Purpose

Landscape schema v2 removes an ambiguity in the original M8 normalized raster contract. A valid evidence-grounded landscape must now say exactly how its row-major cells occupy coordinate space rather than relying on a GIS convention that an independent reader has to guess.

This contract resolves issue #185. It deliberately does **not** resolve issue #203, which asks whether AnthroSim's resource and movement mechanisms are invariant to raster resolution. Schema v2 tells us exactly where a cell is and how large it is; it does not make a 20 m and a 40 m raster scientifically equivalent.

## Supported grid convention

`LandscapeBundle.gridConvention` is mandatory in schema version 2. The only currently supported convention is:

```json
{
  "originAnchor": "upper_left_outer_corner",
  "columnDirection": "increasing_x",
  "rowDirection": "decreasing_y",
  "cellInterpretation": "area"
}
```

Its meaning is normative:

- `originX`, `originY` is the **upper-left outer corner** of row 0, column 0;
- columns proceed left-to-right toward increasing coordinate X;
- rows proceed top-to-bottom toward decreasing coordinate Y;
- every cell represents an **area**, not a point sample located at the origin;
- row-major values therefore enumerate the highest-Y row first.

Landscape schema v1 is rejected rather than automatically upgraded because v1 did not contain enough information to prove which of several plausible orientation/anchor conventions was intended.

## Cell ID and grid coordinates

AnthroSim cell IDs are one-based. Let:

```text
n = CellId - 1
x = n mod width
y = floor(n / width)
```

Then `x` and `y` are zero-based grid coordinates. Increasing model/grid `y` moves down the stored raster and therefore toward decreasing CRS/reference Y under the v2 convention.

For a valid cell `(x, y)`:

```text
minX = originX + x * cellSizeX
maxX = minX + cellSizeX
maxY = originY - y * cellSizeY
minY = maxY - cellSizeY
```

The cell centre is:

```text
centreX = (minX + maxX) / 2
centreY = (minY + maxY) / 2
```

The core exposes canonical helpers for grid coordinates and cell extents. It also exposes the centre in **doubled coordinate units** (`xTwice`, `yTwice` conceptually) so an odd integer cell size can represent a half-unit centre exactly without introducing floating-point rounding into the authoritative geometry boundary.

## Extent and half-cell correctness

Because the origin is an outer corner, it is not a cell centre. A preparation that supplies a centre coordinate as `originX/originY` is shifted by half a cell and is invalid under v2 even if every layer has the correct number of values.

Similarly, a north/south mirrored input is not an alternative valid reading of the same file. Row 0 has one defined spatial meaning: the top/highest-Y row.

The core validates that the declared full grid extent can be represented in the supported signed coordinate range. Coordinate overflow fails closed.

## Coordinate unit and spatial reference

`coordinateUnit` is the authoritative unit of the integer values stored in `originX`, `originY`, `cellSizeX` and `cellSizeY`.

`spatialReference` identifies the spatial reference/basis used to interpret those coordinates, but AnthroSim does not currently contain a CRS engine and does not silently parse, reproject or prove native CRS-unit compatibility. For example, a preprocessing workflow may deliberately encode geographic positions as integer arcseconds while naming EPSG:4326 as the reference basis; the `coordinateUnit` must then explicitly say `arcsecond`.

Therefore:

- a non-empty `spatialReference` is preserved for traceability;
- a non-empty `coordinateUnit` states how the stored integers are measured;
- preprocessing is responsible for making those two declarations mutually intelligible and documenting any conversion from native CRS units;
- downstream AnthroSim output must not imply stronger coordinate precision or CRS validation than this contract provides.

This is an explicit limitation, not a hidden assumption.

## Rectangular cells and movement

Schema v2 can represent rectangular cells because rectangular rasters may be scientifically useful as data containers. However, current M4 permanent migration and M9 temporary travel measure cardinal movement in undifferentiated **grid steps**. One east/west step and one north/south step therefore receive the same step semantics.

If `cellSizeX != cellSizeY`, those equal grid steps represent unequal physical distances. Until issue #203 defines a physical-distance/resolution-normalization contract, AnthroSim therefore fails closed when either of these mechanisms is active on a rectangular evidence-grounded spatial grid:

- M4 permanent migration (`migration.enabled`);
- M9 temporary mobility/travel (`temporaryMobility` present).

A rectangular landscape can still be loaded and used by a spatial run with those grid-step movement mechanisms disabled, for example to test non-movement data/resource transformations.

Square cells remove this **directional anisotropy** problem, but they do not prove scale invariance. A 10 m square grid and a 100 m square grid still have different numbers of cells/edges per physical distance and remain part of #203.

## Preprocessing boundary

Recipe schema v2 requires the normalized convention and the convention of the immediately aligned CSV inputs to be declared explicitly. `build-landscape-bundle.py` accepts only the supported convention and does not silently flip, rotate or transpose an aligned input.

The external GIS workflow must perform and record any source reprojection, reorientation or resampling before the aligned CSV boundary. The reproduction record preserves the convention used for every aligned layer so a north/south flip cannot be hidden behind the phrase "row-major".

The M8.6 public HGT benchmark is migrated deliberately: HGT rows already run north-to-south and its 20-by-20 arcsecond sampled cells are square, so schema v2 changes its geometry/provenance identity but not its terrain/elevation values or intended movement mechanism.

## Identity and compatibility

The grid convention is included in `LandscapeBundle.digest64()` and therefore in `LandscapeBundle.identity()`. Changing orientation semantics changes landscape identity even when the layer value sequence is unchanged.

Spatial execution semantics advance to `anthrosim-spatial-transform-semantics-v3` because valid execution now includes the geometry-v2 and rectangular-movement rules. Core non-spatial model semantics are unchanged.

Existing v1 landscape JSON must be regenerated or explicitly migrated with its true orientation/anchor known. It must not be mass-upgraded by inserting v2 fields based on guesswork.

## Acceptance boundary

For a valid schema-v2 landscape, an independent implementation can determine exactly:

- which direction rows and columns proceed;
- which coordinate is the outer origin;
- the outer extent and centre of every cell;
- whether cells are interpreted as areas or points;
- whether current M4/M9 grid-step movement is permitted on that geometry.

This prevents a vertical mirror, half-cell shift or rectangular-step anisotropy from remaining an invisible implementation choice when evidence-grounded spatial output is interpreted.