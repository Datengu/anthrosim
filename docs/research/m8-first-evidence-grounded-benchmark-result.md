# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the historical M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics changed. The benchmark remains case-study-neutral and **is not archaeological validation**.

The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.

## Current regression reference — model semantics v8

The M3 resource-time repair (#180, #189, #199) advanced the authoritative model identity to `anthrosim-model-semantics-v8`. The frozen M8.6 experiment was therefore rerun unchanged rather than tuning the repaired M3 implementation back toward v7 outputs.

Reviewed execution:

- workflow run: `32917412247`;
- branch head: `7e13d5ee82db0c65d5ac52e4e5501c812fc968b0`;
- pull-request merge-ref build: `bdee1f2831d8c18a9798acc5756cc10d21df1d04`;
- artifact: `9588696469`;
- artifact SHA-256: `7beb866c91f36be7c26b2195e2b07a5910e0cb563d2da4ea690522d908255f8b`;
- aggregate canonical SHA-256: `61f7965f875ba212778f6911261334c39cb9a340bd4717317441526fc80be811`.

The source terrain identity, evidence catalogue and spatial-transform semantics are unchanged. Only the upstream global model semantics changed.

All 32 runs completed the configured duration; no arm was degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

Current primary-metric results are:

| Primary metric | v8 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | not distinctive | 8.81% | 2 / 6 / 0 |
| cell-time occupied | not distinctive | 2.28% | 4 / 4 / 0 |
| terminal population Herfindahl | not distinctive | 8.16% | 5 / 3 / 0 |
| terminal largest-cell share | fragile | 24.54% | 5 / 3 / 0 |

The v8 result therefore contains **no robust primary metric**. Terminal largest-cell share remains fragile; the other three metrics are not distinctive under the predeclared criteria.

This is a meaningful change from the v7 reference rather than a cosmetic checksum update. Correcting seasonal resource timing and making M3/M4 share one current-period demand changes resource exposure and relocation decisions throughout the 100-year trajectories. The strong-terrain migration-distance comparison, for example, changes from a small positive median under v7 to a small negative median under v8, while still failing the predeclared robust-effect magnitude threshold. The broader benchmark conclusion remains fragile rather than robust.

## Previous verification references

### v7 — M4 stay/relocation comparator repair

The v7 repair removed relocation-only costs from the stay counterfactual. Its reviewed M8.6 reference also classified the overall benchmark as `fragile_spatial_structure`, with no robust metrics and terminal largest-cell share fragile. That result is preserved in Git history and was superseded because v8 changes upstream M3 resource timing.

### v6 — M2 demographic-time repair

The v6 M2 repair changed demographic trajectories enough that total migration distance temporarily met the benchmark's robust criterion, while terminal largest-cell share remained fragile. This was preserved at the time rather than tuned away. The later v7 and v8 repairs demonstrate why a preserved scientific regression suite is valuable: downstream classifications can reveal dependence on upstream causal semantics.

### Original first observation

The original corrected M8.6 observation used `anthrosim-model-semantics-v1`. All 32 runs completed and the overall result was also `fragile_spatial_structure`. That historical observation remains model-evaluation evidence but is not the current-code regression baseline.

## Interpretation

The defensible current interpretation is bounded:

- real-world-derived terrain can propagate through the deterministic model and alter simulated trajectories;
- exact spatial effects are sensitive to upstream demographic, resource and migration semantics;
- under current v8 semantics this terrain-only benchmark provides no robust primary spatial effect under its predeclared criteria;
- terminal largest-cell concentration remains fragile across seeds;
- a visually plausible single run is inadequate evidence;
- none of these synthetic benchmark effects validates the terrain-cost transformation as a historical human movement law.

The v8 rebaseline is scientifically useful precisely because the repaired resource clock weakened/reoriented some previously observed effects instead of preserving them artificially.

## What this does not establish

This benchmark does not establish that:

- the simulated population represents a historical population;
- the selected terrain patch represents an ancient landscape state;
- the terrain transformation is a calibrated human travel-cost function;
- water, vegetation, soils, land use or resource geography are historically realistic;
- demographic, resource or migration rules are empirically valid for a real society;
- a similar-looking simulated pattern explains an archaeological pattern;
- terrain had no historical effect, or one universal effect, because this null model is fragile.

The benchmark is evidence-grounded environmental constraint plus reproducible ensemble sensitivity, not case-study validation.

## Reproduction

A third party can reproduce the current benchmark by:

1. regenerating the pinned public input package with `scripts/prepare-m8-benchmark-landscape.py`;
2. requiring byte-identical equality with `examples/m8-first-evidence-grounded-benchmark/` inputs;
3. running the four predeclared ordinary AnthroSim ensembles for seeds 8601–8608;
4. deriving M8.5 spatial observability for every run;
5. aggregating with `scripts/aggregate-m8-spatial-benchmark.py`;
6. comparing the aggregate to the machine-readable reference for the current `MODEL_SEMANTICS_ID`.

The dedicated workflow preserves its generated artifact even when a frozen-reference comparison fails, allowing a declared model-semantics change to be reviewed before deliberate rebaselining.

## M8 scientific conclusion

M8 continues to demonstrate the generic evidence-grounded spatial execution path, while the v6→v7→v8 history demonstrates an equally important research property: **downstream benchmark claims are conditional on the complete upstream model definition**. Corrected causal semantics must be allowed to change those results; they must not be tuned back toward an older reference merely for apparent stability.
