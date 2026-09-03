# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the historical M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics change. The benchmark remains case-study-neutral and **is not archaeological validation**.

The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.

## Current regression reference — model semantics v26

Audit-v4 AV4-001 / #486 changes the same-seed stochastic coupling used to assign annual fertility draws across scientifically distinguishable founder roles. Because that correction changes causal demographic trajectories, the frozen M8.6 experiment was rerun unchanged and its generated artifact was reviewed before replacing the scientific reference.

Reviewed v26 execution:

- workflow run: `33759420404`;
- branch head: `b1553c3d3eb2273a831e517fde3daa0c35d0d6c3`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9894955334`;
- artifact SHA-256: `7b64a4d18e05359bafd0b0a23e6c57c50e5383166ebdbd217883488875178e39`;
- aggregate canonical SHA-256: `f606287151b09f936182ccdc1124e277889d01ec9395f4f5c093545cdf305a44`;
- model semantics: `anthrosim-model-semantics-v26`.

All **32/32** declared runs completed the configured duration and all four arms remained non-degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

The v26 primary-metric results are:

| Primary metric | v26 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | not distinctive | 3.40% | 5 / 3 / 0 |
| cell-time occupied | not distinctive | 2.45% | 3 / 5 / 0 |
| terminal population Herfindahl | not distinctive | 9.50% | 3 / 5 / 0 |
| terminal largest-cell share | **fragile** | **16.71%** | 2 / 6 / 0 |

No primary metric is robust under v26. Terminal largest-cell share remains fragile. Total migration distance and cell-time occupancy remain not distinctive. Terminal population Herfindahl moves from the immediate pre-v26 reference's fragile classification to **not distinctive** because its strong-vs-flat median absolute relative effect is now 9.50%, below the predeclared 10% threshold.

### Causal review of the v26 rebaseline

This reference was not refreshed merely because a regression check failed. The authoritative change is the AV4-001 fertility stochastic-coupling correction, and the benchmark's upstream demographic histories are therefore expected to change even though its terrain inputs and spatial rules are unchanged.

The reviewed v26 run preserves the benchmark design and interpretation boundaries:

- the declared seeds remain 8601–8608;
- the same evidence package and `landscape-v2-6827044513b6c9fb` are used;
- `anthrosim-spatial-transform-semantics-v3` is unchanged;
- all mechanism-file identities are unchanged;
- all four arms remain non-degenerate;
- the overall benchmark class remains `fragile_spatial_structure`;
- the only classification-level change relative to the immediate pre-v26 reference is terminal population Herfindahl moving from fragile to not distinctive;
- terminal largest-cell share remains fragile;
- no metric becomes robust.

The changed Herfindahl classification is scientifically meaningful and is preserved rather than tuned away. Correcting how fertility draws are coupled to scientifically represented founder roles legitimately changes downstream population histories, which can in turn alter spatial ensemble summaries. The 9.50% result falling just below a predeclared 10% threshold is precisely the kind of downstream sensitivity the frozen benchmark is intended to expose.

This remains a result about the declared terrain-only null model under the complete v26 upstream model definition. It is not archaeological validation and does not imply that the exact pre-v26 trajectory should be reproduced after a causal-semantics repair.

## Immediate historical reference — model semantics v20

The machine reference immediately preceding AV4-001 used `anthrosim-model-semantics-v20`. It also classified the overall benchmark as `fragile_spatial_structure`, with no robust primary metric. Terminal population Herfindahl and terminal largest-cell share were fragile; migration distance and cell-time occupancy were not distinctive. That exact reference remains preserved in Git history and is the relevant classification-level comparator for the v26 rebaseline.

## Historical reviewed rebaseline — model semantics v13

Issue #188 repaired the M4 kin-residence proxy. The null model then treated a living parent-child relationship that crosses household boundaries as a reciprocal first-degree kin tie: each household could receive the other household's current residence as a kin anchor. Female-parent and male-parent links used the same rule. Parent-child links within one household created no spatial anchor because M4 moves the household as one unit, and all unique qualifying kin cells were retained rather than truncating to the first four encountered records.

This was a symmetry-preserving null rule. It was not a claim for matrilocality, patrilocality, descent rules, household authority, or an empirically calibrated strength of kin preference.

Because the repair changed authoritative residence utility and could alter permanent-migration trajectories, `MODEL_SEMANTICS_ID` advanced to `anthrosim-model-semantics-v13`.

### Reviewed v13 execution

The frozen M8.6 experiment was rerun unchanged and its generated artifact was reviewed **before** rebaselining.

Reviewed v13 execution:

- workflow run: `33090696389`;
- branch head: `f1b1839dff60f120e98a214a0ac729b846a97659`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9654333775`;
- artifact SHA-256: `c10a47b7e5509ac97f64320fb561a0bc6c6206db0b9f67eb7e43f13988d0b30d`;
- aggregate canonical SHA-256: `8c1b906539a74af80c9fa4f3d6500339feefe28201e6559b71081e4fb89c9725`.

The immediate pre-#188 control was the final successful M8.6 execution from PR #272:

- workflow run: `33091589938`;
- branch head: `7dcfa3550716d8b5e25148ff3f47b9784b4905bb`;
- artifact: `9654664618`;
- artifact SHA-256: `a08f78be7b9a7434f4d37ff767cda786235076d753309413a6ff4a251d8e4ee3`.

That control and the v13 run used the same declared seeds, evidence input, `landscape-v2-6827044513b6c9fb`, and `anthrosim-spatial-transform-semantics-v3`. The relevant authoritative difference was therefore model semantics v12 versus v13.

All 32 runs completed the configured duration and no arm was degenerate.

The overall predeclared classification remained `fragile_spatial_structure`.

The v13 primary-metric results were:

| Primary metric | v13 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | not distinctive | 6.04% | 3 / 5 / 0 |
| cell-time occupied | not distinctive | 0.76% | 5 / 3 / 0 |
| terminal population Herfindahl | not distinctive | 9.98% | 3 / 5 / 0 |
| terminal largest-cell share | **robust** | **23.72%** | 2 / 6 / 0 |

Under v13, terminal largest-cell share was the benchmark's one robust primary metric. Total migration distance, cell-time occupied, and terminal population Herfindahl were not distinctive under the predeclared criteria. No metric was classified as fragile.

### Causal review of the v13 rebaseline

The v13 artifact was compared run-by-run against the immediate v12 control rather than treating the frozen-reference failure as permission to overwrite the baseline.

All 32 terminal trajectories changed. However, the **first authoritative divergence was tightly localized to the repaired M4 kin term in every run**:

- all 32 first differing events were `householdMigration` events;
- the same household, day, origin cell and event sequence were reached in both versions before that decision;
- the origin `kinScorePermille` changed from `0` under v12 to `250` under v13 in all 32 cases;
- every other origin-utility component was identical at that first decision;
- origin total utility therefore rose by exactly `250` in all 32 cases;
- 28 of the 32 first affected migrations consequently selected a different destination; in the other four, the selected destination remained the same even though the corrected choice weights differed.

The first affected household/day was also invariant across the four terrain arms for each seed:

| Seed | First affected day | Household | Origin cell |
| ---: | ---: | ---: | ---: |
| 8601 | 1368 | 10 | 37 |
| 8602 | 1825 | 198 | 200 |
| 8603 | 1460 | 134 | 35 |
| 8604 | 1003 | 4 | 109 |
| 8605 | 1368 | 75 | 19 |
| 8606 | 1825 | 89 | 55 |
| 8607 | 1642 | 135 | 97 |
| 8608 | 1186 | 20 | 20 |

For a concrete example, flat / seed 8601 was identical through event sequence 135. At sequence 136 on day 1368, household 10 evaluated origin cell 37. A child in household 161 had a living male parent in household 10, and household 161 resided at cell 37. Under the old one-way representation, household 10 received no reciprocal kin anchor there. Under v13, that cross-household parent-child relationship correctly contributed the declared `250` kin score to household 10's origin. Its origin utility changed from 3447 to 3697, while all non-kin origin terms were unchanged.

This first-difference pattern was exactly what #188 was intended to change: the formerly missing reciprocal side of a cross-household first-degree kin relationship became visible to M4. It was not the earlier rejected implementation that rewarded a household merely for remaining with co-resident relatives.

The downstream benchmark changes were scientifically meaningful and were therefore preserved rather than tuned away:

- terminal largest-cell share changed from fragile to **robust**;
- total migration distance changed from fragile to **not distinctive**;
- cell-time occupancy remained not distinctive;
- terminal population Herfindahl remained not distinctive;
- the overall benchmark class remained `fragile_spatial_structure`;
- all arms remained non-degenerate.

## Previous verification references

### v12 — immediate pre-#188 control

Model semantics v12 is the immediate control used for the v13 causal comparison. PR #272's final validation passed the preserved M8.6 scientific reference without rebaselining, while advancing evidence-closure and spatial provenance/readiness machinery independently of M4 kin behavior. Its successful artifact therefore provides the correct same-landscape, same-evidence, same-spatial-semantics comparator for #188.

### v11 — scarce-resource apportionment repair

The v11 #182 repair replaced stable first-claim remainder assignment with largest-remainder apportionment and deterministic rotation of exact ties. Its reviewed M8.6 reference remained `fragile_spatial_structure`, with no robust primary metric. Total migration distance and terminal largest-cell share were fragile; cell-time occupied and terminal population Herfindahl were not distinctive.

### v10 — condition-mortality causal-attribution repair

The v10 #200 repair corrected the interpretation of low-condition deaths from uniquely resource-scarcity-attributed deaths to a general condition-mediated cause. The M8.6 primary numerical result was unchanged from v9; its rebaseline updated causal/schema identity rather than trajectories.

### v9 — M3 response-time and M4 opportunity-clock repair

The v9 #204 repair separated M3 integration resolution from elapsed condition response and from the M4 permanent-migration opportunity clock. Its reviewed reference classified the benchmark as `fragile_spatial_structure`, with no robust primary metric.

### v8 — M3 resource-time accounting repair

The v8 reference followed the M3 annual-quantity/seasonal-accounting repair. It also classified the benchmark as `fragile_spatial_structure`, with no robust primary metric.

### v7 — M4 stay/relocation comparator repair

The v7 repair removed relocation-only costs from the stay counterfactual. Its M8.6 reference also classified the overall benchmark as `fragile_spatial_structure`.

### v6 — M2 demographic-time repair

The v6 M2 repair changed demographic trajectories enough that total migration distance met the benchmark's robust criterion, while terminal largest-cell share remained fragile. This was preserved at the time rather than tuned away.

### Original first observation

The original corrected M8.6 observation used `anthrosim-model-semantics-v1`. All 32 runs completed and the overall result was also `fragile_spatial_structure`. That historical observation remains model-evaluation evidence but is not the current-code regression baseline.

## Interpretation

The defensible current interpretation is bounded:

- real-world-derived terrain can propagate through the deterministic model and alter simulated trajectories;
- exact spatial effects are conditional on upstream demographic, resource, condition and migration semantics;
- under v26, the predeclared terrain contrast produces a fragile terminal largest-cell-share response in this synthetic benchmark, while the other three primary metrics are not distinctive;
- the change in Herfindahl classification across the v26 fertility-coupling repair demonstrates that ensemble-level spatial conclusions can be sensitive to corrected upstream demographic stochastic coupling;
- a visually plausible single run is inadequate evidence;
- none of these synthetic benchmark effects validates the terrain-cost transformation, kin-weight parameter, fertility assumptions, or any other mechanism as a historical human behavior law.

The v26 rebaseline is deliberately different from a cosmetic reference refresh: the causal semantics changed, the frozen experiment was rerun, the resulting classification-level change was reviewed, and the changed secondary conclusion is recorded rather than forced back to the old result.

## What this does not establish

This benchmark does not establish that:

- the simulated population represents a historical population;
- the selected terrain patch represents an ancient landscape state;
- the terrain transformation is a calibrated human travel-cost function;
- the M4 kin weight or reciprocal null rule is calibrated to a historical society;
- water, vegetation, soils, land use or resource geography are historically realistic;
- demographic, resource or migration rules are empirically valid for a real society;
- a similar-looking simulated pattern explains an archaeological pattern;
- terrain, kinship or fertility had one universal historical effect because this remains a synthetic sensitivity benchmark.

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

M8 continues to demonstrate the generic evidence-grounded spatial execution path. The v6→v7→v8→v9→v10→v11→v12→v13→v20→v26 history demonstrates an equally important research property: **downstream benchmark claims are conditional on the complete upstream model definition**. Corrected causal or numerical semantics must be allowed to change those results, but any rebaseline must be causally reviewed rather than automatically accepted.
