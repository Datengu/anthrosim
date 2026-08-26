# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the historical M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics changed. The benchmark remains case-study-neutral and **is not archaeological validation**.

The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.

## Current regression reference — model semantics v9

The #204 response-time repair advanced the authoritative model identity to `anthrosim-model-semantics-v9`. The frozen M8.6 experiment was therefore rerun unchanged rather than tuning the repaired timing implementation toward v8 outputs.

Reviewed execution:

- workflow run: `32923009999`;
- branch head: `dbd73404f2c2f9e65d58c32e4f495acee4bb7e30`;
- pull-request merge-ref build: `4711431704948c8f9f842c968aa113243b8a41a0`;
- artifact: `9590576288`;
- artifact SHA-256: `909a4d1032c2f3da5a4c7f5c719008a70b6c04e268e9cdb2893f1cef7c04525d`;
- aggregate canonical SHA-256: `fb90ad3a8870038d7f7e1ec42b34ffb3d1564be9255fc8f068b80673c35bb8c2`.

The source terrain identity, evidence catalogue and spatial-transform semantics are unchanged. The relevant upstream change is the global M3/M4 response-time model: M3 settlement resolution no longer multiplies physiological/scarcity response opportunity, and M4 has an independent permanent-migration decision clock.

All 32 runs completed the configured duration; no arm was degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

Current primary-metric results are:

| Primary metric | v9 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | fragile | 11.83% | 3 / 5 / 0 |
| cell-time occupied | not distinctive | 1.39% | 3 / 5 / 0 |
| terminal population Herfindahl | not distinctive | 9.24% | 7 / 1 / 0 |
| terminal largest-cell share | fragile | 11.38% | 5 / 3 / 0 |

The v9 result therefore contains **no robust primary metric**. Total migration distance and terminal largest-cell share are fragile; cell-time occupied and terminal population Herfindahl are not distinctive under the predeclared criteria.

This is a meaningful scientific-regression change, not a checksum-only update. Under v8, total migration distance was not distinctive in the then-current reference; earlier model-semantics versions had at times classified it as robust. Under v9 it exceeds the strong-arm median-effect magnitude threshold but fails sign-consistency and cross-arm direction criteria, so it is explicitly **fragile**, not robust. This is consistent with the repaired model definition: permanent-migration opportunities now arise from their own four-per-year default clock rather than being structurally tied to M3 settlement boundaries, while the resource/condition state observed at those decisions can still differ through legitimate elapsed-state dynamics.

The important benchmark-level conclusion is unchanged: the terrain-only null model does not produce a seed-stable robust primary spatial effect under the predeclared criteria.

## Previous verification references

### v8 — M3 resource-time accounting repair

The v8 reference followed the M3 annual-quantity/seasonal-accounting repair. It also classified the overall benchmark as `fragile_spatial_structure`, with no robust primary metric and terminal largest-cell share fragile. Total migration distance was not distinctive. That exact reference remains preserved in Git history and was superseded because v9 changes the authoritative response/opportunity clocks.

### v7 — M4 stay/relocation comparator repair

The v7 repair removed relocation-only costs from the stay counterfactual. Its reviewed M8.6 reference also classified the overall benchmark as `fragile_spatial_structure`, with no robust metrics and terminal largest-cell share fragile.

### v6 — M2 demographic-time repair

The v6 M2 repair changed demographic trajectories enough that total migration distance met the benchmark's robust criterion, while terminal largest-cell share remained fragile. This was preserved at the time rather than tuned away. Later rebaselines demonstrate why a preserved scientific regression suite is valuable: downstream classifications can reveal dependence on upstream causal semantics.

### Original first observation

The original corrected M8.6 observation used `anthrosim-model-semantics-v1`. All 32 runs completed and the overall result was also `fragile_spatial_structure`. That historical observation remains model-evaluation evidence but is not the current-code regression baseline.

## Interpretation

The defensible current interpretation is bounded:

- real-world-derived terrain can propagate through the deterministic model and alter simulated trajectories;
- exact spatial effects are sensitive to upstream demographic, resource and migration semantics;
- under current v9 semantics this terrain-only benchmark provides no robust primary spatial effect under its predeclared criteria;
- total migration distance and terminal largest-cell concentration are both fragile across seeds/arms;
- a visually plausible single run is inadequate evidence;
- none of these synthetic benchmark effects validates the terrain-cost transformation as a historical human movement law.

The v9 rebaseline is scientifically useful precisely because repairing a numerical/scheduling confound was allowed to alter downstream classifications instead of preserving an older result artificially.

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

M8 continues to demonstrate the generic evidence-grounded spatial execution path, while the v6→v7→v8→v9 history demonstrates an equally important research property: **downstream benchmark claims are conditional on the complete upstream model definition**. Corrected causal semantics must be allowed to change those results; they must not be tuned back toward an older reference merely for apparent stability.
