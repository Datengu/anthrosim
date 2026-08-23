# M8.6 first evidence-grounded spatial null-model benchmark

Status: **predeclared before benchmark execution**.

This document specializes the generic M8.0 contract into the first Level-D benchmark. It is intentionally unrelated to any private or intended future case study.

## Question

> When the existing AnthroSim M1-M4 mechanisms are run on a fixed real-world-derived terrain surface, how much does terrain-dependent movement cost alter long-run migration and spatial concentration relative to the same simulated world with terrain made behaviourally flat?

This is a null-model question. It does not ask the simulation to reproduce a known settlement, route, boundary, event or archaeological pattern.

## Evidence-grounded input

The only evidence-grounded environmental input in this benchmark is terrain elevation/contrast.

Source:

- Mapzen/Tilezen Terrain Tiles, Skadi HGT tile `N46E007`;
- public AWS Open Data bucket `elevation-tiles-prod`;
- source URL fixed in `scripts/prepare-m8-benchmark-landscape.py`;
- exact downloaded bytes must be identified by SHA-256 before benchmark execution;
- Mapzen/Tilezen source attribution applies; global SRTM terrain data are attributed by that project to the U.S. Geological Survey.

The benchmark uses a fixed 16×16 sample patch:

- source row start: `1300`;
- source column start: `1300`;
- sampling stride: `20` arcseconds;
- source HGT row order: north to south;
- no interpolation;
- no smoothing;
- no void filling;
- no clipping of terrain contrast;
- source void or out-of-domain contrast is a hard preprocessing error.

The normalized `terrain_contrast` value for a sampled cell is the maximum absolute elevation difference, in metres, between that cell and its sampled north/east/south/west neighbours. This is a transparent terrain proxy, not a calibrated human travel-cost model.

The patch was selected as a neutral, publishable high-relief test surface so that a terrain mechanism has non-trivial variation to act on. It was not selected by inspecting AnthroSim outcomes.

## Inputs that remain synthetic

The following are **not** evidence-grounded in this benchmark:

- founder locations and ages;
- demography;
- water accessibility;
- renewable resource productivity and seasonality;
- annual food need;
- migration behavioural coefficients other than the tested terrain-dependent movement-cost field.

Those fields retain the existing synthetic-validation mechanisms. Therefore this benchmark is not a historical landscape reconstruction and cannot validate settlement suitability, subsistence, population history or archaeological interpretation.

Using the same seed across terrain alternatives keeps founder initialization and synthetic environmental fields paired, allowing the terrain transformation to be varied while those stochastic components remain identical within each pair.

## Fixed simulation settings

Every arm uses ordinary M7 spatial ensemble execution with:

- seeds: `8601,8602,8603,8604,8605,8606,8607,8608`;
- duration: `100` simulated years;
- grid: `16 × 16` cells;
- initial population: `1000` synthetic founders;
- target household size: `5`;
- maximum persistent person records: `100000`;
- resource productivity scale: `1000` permille;
- resource seasonality scale: `1000` permille;
- annual food need: `100` abstract units/person;
- migration enabled;
- migration candidate radius: `3` cells.

No setting may be changed in response to benchmark outcomes without defining a new benchmark version.

## Terrain-to-movement sensitivity arms

All arms consume the identical normalized landscape and evidence catalogue. Only the M8.4 mapping from `terrain_contrast` to authoritative `movementCost` differs.

`terrain_contrast` source domain is fixed at `0..=2500` metres of maximum sampled-neighbour elevation difference.

| Arm | movementCost target range | Purpose |
| --- | --- | --- |
| `flat` | `1000..=1000` | causal control: terrain values preserved but behaviourally inert |
| `weak` | `1000..=1500` | weak terrain penalty |
| `moderate` | `1000..=2500` | broader terrain penalty |
| `strong` | `1000..=4000` | deliberately strong sensitivity bound |

These ranges are **sensitivity assumptions**, not empirically calibrated prehistoric travel costs. No arm is designated the historically correct arm.

## Authoritative and derived outputs

Each arm must be executed through the ordinary M7 ensemble/retry machinery. Each run must preserve:

- complete `ExperimentConfig`, including the `EvidenceCatalog`;
- seed;
- core model-semantics identity;
- landscape identity/digest;
- spatial transformation identity;
- transformed world identity;
- terminal state digest;
- ordinary events, metrics and checkpoints.

M8.5 `spatial-observability.json` must then be regenerated downstream for every run.

## Primary observables

The predeclared primary derived metrics are:

1. `migrationTotalDistanceCells` — cumulative distance of completed household migrations;
2. `cellTimeOccupiedPermille` — fraction of available cell-time occupied by living simulated people;
3. `terminalPopulationHerfindahlPerMillion` — terminal population concentration across cells;
4. `terminalLargestCellSharePermille` — terminal share of living population in the largest cell.

Secondary/descriptive observables include:

- terminal living population;
- terminal occupied cells;
- births/deaths/resource-scarcity deaths;
- migration moves and people moved;
- origin→destination flow distribution;
- migration-distance distribution;
- per-cell occupancy persistence/person-days.

Secondary observables may explain a result but must not replace the primary metrics after results are known.

## Paired analysis

Comparison is paired by seed. For every metric and each non-flat arm:

```text
effect(seed, arm) = arm(seed) - flat(seed)
```

When the flat value is non-zero, the analysis also reports:

```text
relative_effect = effect / abs(flat)
```

Zero flat baselines remain explicit; they are not assigned an arbitrary percentage.

The analysis reports all eight paired effects, medians, sign counts and zero-baseline counts. No run is discarded because its direction is inconvenient.

## Predeclared outcome classification

A run is **terminally degenerate** when it fails, is incomplete, or stops before 100 years for a reason other than ordinary duration completion. Population extinction therefore remains visible rather than being filtered out.

The benchmark-level classification is evaluated in this order.

### 1. `degenerate`

Classify the benchmark as degenerate if any terrain arm has at least 4 of 8 terminally degenerate runs.

### 2. Metric-level `robust`

For a primary metric, call the terrain effect robust when all are true:

- the `strong` arm has non-zero paired effects with the same sign in at least 6 of 8 seeds;
- the median absolute relative effect for `strong` is at least 10% among seeds with non-zero flat baselines;
- the median paired effect for `moderate` has the same sign as the median paired effect for `strong`.

If a primary metric has too many zero flat baselines to calculate a meaningful relative median, it cannot satisfy the 10% criterion and is not labelled robust.

### 3. Metric-level `fragile`

A primary metric is fragile when the median absolute relative effect for `strong` is at least 10% but the robust sign/ordering criteria are not all met.

### 4. Benchmark-level spatial class

If the degenerate rule did not trigger:

- `robust_spatial_structure`: at least 2 of 4 primary metrics are robust;
- `fragile_spatial_structure`: fewer than 2 are robust, but at least 1 primary metric is robust or fragile;
- `no_distinctive_spatial_structure`: no primary metric is robust or fragile.

The labels describe sensitivity of **this declared model** to the tested terrain constraint. They are not historical or archaeological conclusions.

## Interpretation limits

A result may support statements such as:

- terrain-dependent movement costs materially changed or did not materially change simulated migration/spatial concentration under these assumptions;
- effects were stable or unstable across stochastic seeds and transformation strengths;
- the current lower-level mechanisms produced persistence, concentration, extinction or little spatial differentiation on this terrain input.

A result may **not** support claims that:

- any simulated cell corresponds to a real historical settlement;
- the terrain transformation is empirically correct;
- the synthetic water/resource surface represents the real landscape;
- the simulated population resembles a specific past population;
- a real archaeological pattern has been explained or predicted.

## Reproducibility acceptance

M8.6 is complete only when the repository contains enough information for a third party to:

1. retrieve the permitted public source tile;
2. verify its pinned SHA-256;
3. regenerate byte-identical normalized benchmark inputs;
4. execute all four arms through ordinary M7 machinery;
5. regenerate M8.5 observability outputs;
6. regenerate the benchmark aggregate/classification without manual editing;
7. see every failed/extinct/negative/null result rather than only favourable examples.
