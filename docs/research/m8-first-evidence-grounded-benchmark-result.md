# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the historical M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics changed. The benchmark remains case-study-neutral and **is not archaeological validation**.

The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.

## Current regression reference — model semantics v11

Issue #182 repairs the M3 indivisible-unit allocation rule used when several household claims compete for insufficient stock in one resource cell. Under v11, proportional floor shares are completed with largest-remainder apportionment, while exact fractional ties rotate deterministically with the persisted M3 resource-period sequence rather than permanently favoring the first/lower-ID claim.

Because that rule can change who receives the final scarce integer unit, it can change condition and later trajectories even while aggregate cell resource accounting remains exactly conserved. The authoritative model identity therefore advances to `anthrosim-model-semantics-v11`.

The frozen M8.6 experiment was rerun unchanged and its generated artifact was reviewed **before** rebaselining.

Reviewed execution:

- workflow run: `33023888296`;
- branch head: `f0914493835bd513383bf8de7f88ba39f9d83c34`;
- pull-request merge-ref build: `f833232a575397d7833b0a76d5eedac5df055cf2`;
- artifact: `9627671700`;
- artifact SHA-256: `94b2f21754a9b3bad3d2cadff0bdf32cce03c81194cec8e28b11466d7e62b935`;
- aggregate canonical SHA-256: `b013ebbd6004165a317fd471acf201089c78fb3dd538b9b9d84a1f8e8c849ad4`.

The source terrain, evidence catalogue, declared seeds, experiment design and spatial-transform semantics are unchanged. All 32 runs completed the configured duration and no arm was degenerate.

The overall predeclared classification remains:

> **fragile spatial structure**

Current primary-metric results are:

| Primary metric | v11 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | fragile | 11.83% | 3 / 5 / 0 |
| cell-time occupied | not distinctive | 1.39% | 3 / 5 / 0 |
| terminal population Herfindahl | not distinctive | 9.24% | 7 / 1 / 0 |
| terminal largest-cell share | fragile | 11.38% | 5 / 3 / 0 |

The v11 result therefore still contains **no robust primary metric**. Total migration distance and terminal largest-cell share are fragile; cell-time occupied and terminal population Herfindahl are not distinctive under the predeclared criteria.

### Causal review of the v11 rebaseline

The v10 and v11 benchmark artifacts were compared run-by-run rather than treating a red reference check as permission to overwrite the baseline.

Of the 32 declared runs, **31 reproduce the v10 scientific trajectory exactly**. The only changed run is the `moderate` terrain arm at seed `8604`.

That run is identical through day `29,930`. The first authoritative causal divergence occurs at day `30,112`: under v10, person `2035` dies through the condition-mediated mortality mechanism at condition `995`; under v11 that death does not occur. By the next annual snapshot at day `30,295`, v11 therefore has one additional living person.

Crucially, the resource-accounting control remains intact. At that annual snapshot total resource unmet need is **36 units in both versions**. The repair has changed the *distribution* of indivisible scarce units among competing claims, not created or destroyed food. That is the exact mechanism #182 was intended to correct. The altered allocation changes condition exposure for particular households and can consequently alter a later stochastic survival event and downstream migration/demographic history.

The resulting benchmark-level changes are correspondingly narrow:

- only the `moderate / 8604` terminal state digest changes;
- strong-vs-flat headline results and sign counts remain unchanged;
- the overall benchmark class remains `fragile_spatial_structure`;
- the same two metrics remain fragile and the same two remain not distinctive;
- no arm becomes degenerate and no previously non-robust metric becomes robust.

The moderate-arm exact paired fractions change where that one trajectory contributes to the median, but no scientific classification is tuned or forced back to its earlier numerical value. The corrected v11 result is frozen because the upstream allocation semantics are now different, not because the previous reference was inconvenient.

## Previous verification references

### v10 — condition-mortality causal-attribution repair

The v10 #200 repair corrected the interpretation of low-condition deaths from uniquely resource-scarcity-attributed deaths to a general condition-mediated cause. The M8.6 primary numerical result was unchanged from v9; its rebaseline updated causal/schema identity rather than trajectories. The v10 exact reference is preserved in Git history and provides the direct comparator used for the v11 review above.

### v9 — M3 response-time and M4 opportunity-clock repair

The v9 #204 repair separated M3 integration resolution from elapsed condition response and from the M4 permanent-migration opportunity clock. Its reviewed reference classified the benchmark as `fragile_spatial_structure`, with no robust primary metric. Total migration distance and terminal largest-cell share were fragile; cell-time occupied and terminal population Herfindahl were not distinctive.

### v8 — M3 resource-time accounting repair

The v8 reference followed the M3 annual-quantity/seasonal-accounting repair. It also classified the benchmark as `fragile_spatial_structure`, with no robust primary metric and terminal largest-cell share fragile. Total migration distance was not distinctive.

### v7 — M4 stay/relocation comparator repair

The v7 repair removed relocation-only costs from the stay counterfactual. Its reviewed M8.6 reference also classified the overall benchmark as `fragile_spatial_structure`, with no robust metrics and terminal largest-cell share fragile.

### v6 — M2 demographic-time repair

The v6 M2 repair changed demographic trajectories enough that total migration distance met the benchmark's robust criterion, while terminal largest-cell share remained fragile. This was preserved at the time rather than tuned away.

### Original first observation

The original corrected M8.6 observation used `anthrosim-model-semantics-v1`. All 32 runs completed and the overall result was also `fragile_spatial_structure`. That historical observation remains model-evaluation evidence but is not the current-code regression baseline.

## Interpretation

The defensible current interpretation is bounded:

- real-world-derived terrain can propagate through the deterministic model and alter simulated trajectories;
- exact spatial effects are conditional on upstream demographic, resource, condition and migration semantics;
- under current v11 semantics this terrain-only benchmark provides no robust primary spatial effect under its predeclared criteria;
- total migration distance and terminal largest-cell concentration are both fragile across seeds/arms;
- indivisible resource rounding is now explicit model semantics rather than an accidental stable-ID priority;
- condition-mediated death counts cannot by themselves identify resource scarcity as the unique upstream cause;
- a visually plausible single run is inadequate evidence;
- none of these synthetic benchmark effects validates the terrain-cost transformation as a historical human movement law.

The v11 rebaseline is deliberately different from a cosmetic reference refresh: one corrected allocation decision can propagate into an individual survival difference and later history. Preserving that consequence is necessary if the regression suite is to test the current model rather than reproduce a known defect.

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

M8 continues to demonstrate the generic evidence-grounded spatial execution path, while the v6→v7→v8→v9→v10→v11 history demonstrates an equally important research property: **downstream benchmark claims are conditional on the complete upstream model definition**. Corrected causal or numerical semantics must be allowed to change those results, but any rebaseline must be causally reviewed rather than automatically accepted.
