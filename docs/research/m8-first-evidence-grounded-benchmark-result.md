# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the M8.6 terrain null-model result and the reviewed regression rebaselines required when upstream authoritative model semantics change. The benchmark remains case-study-neutral and **is not archaeological validation**.

The checked-in machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references and the detailed per-generation review narrative remain preserved in Git history and associated Audit-v4 evidence/repair records.

**Current machine-readable reference: `anthrosim-model-semantics-v33`.**

## Current regression reference — model semantics v33

Audit-v4 AV4-009 / #518 removes arbitrary canonical M4 spatial-candidate ordering from uncertainty/proportional-choice assignment while preserving deterministic utility/distance semantics and total choice probability for exchangeable candidates. Because this changes authoritative permanent-migration trajectories, the frozen M8.6 experiment was rerun unchanged and independently reviewed before the checked-in scientific reference advanced to v33.

Current v33 reference provenance:

- workflow run: `33884800100`;
- artifact: `9941586776`;
- artifact SHA-256: `21b2f9dacdbf85c5b036bae7ca90d158cad5257a67cca1dae66bc17e54e9f9ba`;
- reviewed production head: `25c9a11dce8052fecfbb339114a4ba1c8da00b0c`;
- aggregate canonical SHA-256: `adf2033e68b5620ef7eb328b0ca5daea2951d3cf670d0cde19e486c06a43d97d`;
- model semantics: `anthrosim-model-semantics-v33`;
- landscape identity: `landscape-v2-6827044513b6c9fb`;
- spatial transformation semantics: `anthrosim-spatial-transform-semantics-v3`.

All declared arms remain non-degenerate. The overall predeclared classification remains:

> **fragile spatial structure**

The current v33 primary classifications are:

| Primary metric | v33 classification | Strong-vs-flat median absolute paired effect |
|---|---|---:|
| total migration distance | not distinctive | 9.44% |
| cell-time occupied | not distinctive | 1.85% |
| terminal population Herfindahl | not distinctive | 8.82% |
| terminal largest-cell share | **fragile** | above the predeclared effect threshold but without robust sign consistency |

No primary metric is robust in the current v33 reference. The benchmark therefore continues to support only the deliberately limited conclusion that the declared terrain transformation can perturb spatial outcomes, with sensitivity in direction/metric classification across seeds and upstream scientific-semantics generations.

## Historical Audit-v4 rebaselines

The frozen M8.6 design was repeatedly rerun when Audit-v4 authoritative repairs could causally change its population/migration trajectories. Those reviewed generations remain historical evidence rather than calibration targets:

- **v26 / AV4-001:** fertility draw assignment moved from arbitrary person-label order to scientific person coupling; overall class remained `fragile_spatial_structure`.
- **v27 / AV4-002:** background-mortality assignment moved to scientific person coupling; overall class remained fragile.
- **v28 / AV4-003:** M4 household migration RNG scheduling moved from `HouseholdId` order to household scientific coupling; terminal largest-cell share crossed to robust while the overall benchmark remained fragile.
- **v29 / AV4-005:** parentage assignment moved from arbitrary canonical male order to scientific coupling; largest-cell share returned to fragile and Herfindahl remained fragile.
- **v30 / AV4-006:** condition-mediated mortality coupling was corrected and applicable regression evidence was rerun.
- **v31 / AV4-007:** M9 equal-cost destination coupling changed, but applicability depends on whether the M8 design exercises that M9 pathway.
- **v32 / AV4-008:** scarce-resource remainder ties moved to household scientific coupling and the benchmark/reference applicability was rechecked.
- **v33 / AV4-009:** M4 candidate-order stochastic coupling was removed and the current checked-in reference was independently reviewed/rebaselined.

Exact historical run IDs, artifact hashes, per-seed tables and threshold crossings remain available in Git history and the corresponding Audit-v4 issue/PR evidence. They should be cited when discussing a specific semantics generation rather than relabelling an older execution as v33.

## What the current result establishes

Under the frozen M8.6 terrain-only design, AnthroSim can bind provenance-tracked spatial evidence, transform it through declared model assumptions, run paired deterministic ensembles and expose how those assumptions affect residence-based spatial outcomes. The current v33 checked-in result remains `fragile_spatial_structure` and therefore does not justify a robust terrain-only explanatory claim.

The benchmark is deliberately useful when classifications move after legitimate upstream causal repairs: those movements expose sensitivity of the model result to corrected scientific semantics rather than being tuned away.

## What the current result does not establish

This benchmark does **not** establish:

- archaeological validity of the terrain transformation;
- reconstruction of a historical population, settlement or route;
- calibration of mobility, demography or resource parameters to a real site;
- invariance to alternative demographic, household, resource or movement formulations;
- a robust causal claim that terrain alone explains an observed historical spatial pattern.

Reference maintenance after a declared model-semantics change is reproducibility work, not empirical calibration. A later semantics generation should replace the current reference only after the unchanged benchmark is rerun and its numerical scientific result is independently reviewed.
