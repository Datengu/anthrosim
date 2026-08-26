# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the historical M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics changed. The benchmark remains case-study-neutral and **is not archaeological validation**.

The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.

## Current regression reference — model semantics v10

The issue #200 condition-mortality causality repair advanced the authoritative model identity to `anthrosim-model-semantics-v10`. The frozen M8.6 experiment was rerun unchanged and reviewed before rebaselining. The repair changes public mortality cause/field semantics and artifact identities; it does not alter the numerical condition hazard itself.

Reviewed execution:

- workflow run: `32930245492`;
- branch head: `a329f68f3278a600dece4193b9a6179d4e981180`;
- pull-request merge-ref build: `4a224061e4f4387430a33215518503b064810a1f`;
- artifact: `9593020274`;
- artifact SHA-256: `61295c9c97a13b30879784fa94f613e2a53312b6db60b4b749239801a1c8d182`;
- aggregate canonical SHA-256: `bf078fdfd5a43673bfef0ab76203af5fda673868d8d18b81c754e9b8682a1d7f`.

The source terrain identity, evidence catalogue and spatial-transform semantics are unchanged. The M8 aggregate/reference schema is now version 2 because secondary mortality output uses `conditionMortalityDeaths` rather than a scarcity-attributed death field. Spatial observability itself is schema version 3 for the corresponding per-cell wire change.

All 32 runs completed the configured duration; no arm was degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

Current primary-metric results are:

| Primary metric | v10 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | fragile | 11.83% | 3 / 5 / 0 |
| cell-time occupied | not distinctive | 1.39% | 3 / 5 / 0 |
| terminal population Herfindahl | not distinctive | 9.24% | 7 / 1 / 0 |
| terminal largest-cell share | fragile | 11.38% | 5 / 3 / 0 |

The v10 result contains **no robust primary metric**. Total migration distance and terminal largest-cell share are fragile; cell-time occupied and terminal population Herfindahl are not distinctive under the predeclared criteria.

### Causal review of the v10 rebaseline

Every current primary metric, paired-effect fraction, sign count, robust criterion and overall classification matches the live v9 reference exactly. In particular, the strong-vs-flat migration-distance median absolute paired effect remains approximately **11.83%**, with signs **3 positive / 5 negative / 0 zero**. No older intermediate value is used as the reference.

The only required frozen-reference changes are the declared schema/model-semantics identity and the experiment/configuration/state/aggregate identities that follow from the new v10 wire/model contract. The secondary mortality label is also corrected from resource-scarcity attribution to general condition-mediated mortality. No numerical scientific change has been hidden by this rebaseline.

This exact stability is scientifically plausible for this particular repair: issue #200 corrects what a low-condition death is called and what can be causally inferred from it. The executable condition-mediated probability and its v9 elapsed-time scaling remain unchanged. Resource shortfall can still reduce condition, but a later low-condition death is no longer asserted to have resource scarcity as its unique cause because M4 travel can affect the same condition scalar.

The important benchmark-level conclusion therefore remains unchanged: the terrain-only null model does not produce a seed-stable robust primary spatial effect under the predeclared criteria.

## Previous verification references

### v9 — M3 response-time and M4 opportunity-clock repair

The v9 #204 response-time repair separated M3 integration resolution from elapsed condition response and from the M4 permanent-migration opportunity clock. Its reviewed M8.6 reference classified the overall benchmark as `fragile_spatial_structure`, with no robust primary metric. Total migration distance and terminal largest-cell share were fragile; cell-time occupied and terminal population Herfindahl were not distinctive. The complete v10 primary scientific result reproduces that v9 reference exactly.

### v8 — M3 resource-time accounting repair

The v8 reference followed the M3 annual-quantity/seasonal-accounting repair. It also classified the overall benchmark as `fragile_spatial_structure`, with no robust primary metric and terminal largest-cell share fragile. Total migration distance was not distinctive. That exact reference remains preserved in Git history and was superseded because v9 changed the authoritative response/opportunity clocks.

### v7 — M4 stay/relocation comparator repair

The v7 repair removed relocation-only costs from the stay counterfactual. Its reviewed M8.6 reference also classified the overall benchmark as `fragile_spatial_structure`, with no robust metrics and terminal largest-cell share fragile.

### v6 — M2 demographic-time repair

The v6 M2 repair changed demographic trajectories enough that total migration distance met the benchmark's robust criterion, while terminal largest-cell share remained fragile. This was preserved at the time rather than tuned away. Later rebaselines demonstrate why a preserved scientific regression suite is valuable: downstream classifications can reveal dependence on upstream causal semantics.

### Original first observation

The original corrected M8.6 observation used `anthrosim-model-semantics-v1`. All 32 runs completed and the overall result was also `fragile_spatial_structure`. That historical observation remains model-evaluation evidence but is not the current-code regression baseline.

## Interpretation

The defensible current interpretation is bounded:

- real-world-derived terrain can propagate through the deterministic model and alter simulated trajectories;
- exact spatial effects are conditional on upstream demographic, resource, condition and migration semantics;
- under current v10 semantics this terrain-only benchmark provides no robust primary spatial effect under its predeclared criteria;
- total migration distance and terminal largest-cell concentration are both fragile across seeds/arms;
- condition-mediated death counts cannot by themselves identify resource scarcity as the unique upstream cause;
- a visually plausible single run is inadequate evidence;
- none of these synthetic benchmark effects validates the terrain-cost transformation as a historical human movement law.

The v10 rebaseline is a causal-semantics/schema maintenance update with an explicitly reviewed null numerical effect on the primary benchmark result. That is different from the v9 rebaseline, where the scheduling repair legitimately changed a downstream classification.

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

M8 continues to demonstrate the generic evidence-grounded spatial execution path, while the v6→v7→v8→v9→v10 history demonstrates an equally important research property: **downstream benchmark claims are conditional on the complete upstream model definition**. Corrected causal semantics must be allowed to change those results; where a repair produces no numerical scientific change, that null effect should be reviewed and recorded just as explicitly rather than inferred from green CI alone.
