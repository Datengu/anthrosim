# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

## Model-semantics v28 applicability re-verification — machine reference unchanged

Audit-v4 AV4-003 / #491 changes permanent M4 migration RNG assignment. The frozen M9.7 design was therefore rerun on the v28 production branch as part of applicable scientific/security run `33797427904`, job `100788647970`.

The complete M9.7 gate passed **without rebaselining**: paired ensembles, M9.6 observability, exact intermittent replay, active annual checkpoint/resume, aggregation, the preserved v27 scientific-reference comparison, and tamper rejection all succeeded. Every benchmark arm still records **zero permanent M4 migrations**, so AV4-003 is causally inapplicable to this benchmark's authoritative result.

The machine-readable M9.7 reference therefore remains the reviewed v27 reference rather than being relabelled as v28 evidence. This explicit non-change is itself the v28 applicability result.

## Current regression reference — model semantics v27

The current machine-readable regression reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`. Audit-v4 AV4-002 / #488 changes background-mortality stochastic coupling, so the frozen M9.7 design was rerun unchanged and its numerical result reviewed before replacement.

Reviewed v27 execution:

- workflow run: `33785449208`;
- branch head: `14b5290525f97c6404432e53fd91af5760f400cc`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9905367753`;
- artifact SHA-256: `350c6344ec721d201ee4e528de9daec9c5136faa0d6bf90727f5ec24a17c5020`;
- aggregate canonical SHA-256: `4c17ac0e9d1ee601f46baff8392203cc99ce267ed9e3028596a2f2871aaa65a9`;
- reference model semantics: `anthrosim-model-semantics-v27`.

### v27 reference result

The predeclared capability remains distinguished:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment has exactly **270 days** with visitor presence;
- intermittent treatments complete **990–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or condition-mediated death.

The aggregate v27 values are:

- median total focal-person-day difference: **31 permille** (v26: 32);
- maximum paired total focal-person-day difference: **36 permille** (v26: 37);
- median intermittent peak-visitor share: **432 permille** (v26: 441);
- minimum intermittent peak-visitor share: **396 permille** (v26: 398).

All predeclared paired criteria therefore continue to support `capability_distinguished`. The same exact-head workflow separately passed identical intermittent replay and active annual checkpoint/resume before reaching the frozen-reference comparison.

The independently preserved M9.6 travel-burden reference is **not rebaselined** for v27. For every seed, planned and observed transit days, planned and realized travel cost, and planned and realized route distance remain exactly equal to the existing travel-burden reference. The changes are population-dependent occupancy outcomes downstream of corrected background mortality, not changed travel-accounting semantics.

## Historical reviewed reference — model semantics v26

The machine-readable reference immediately preceding AV4-002 recorded the reviewed Audit-v4 AV4-001 / #486 execution under `anthrosim-model-semantics-v26`. Its exact values remain historical provenance for the v27 comparison.

Reviewed v26 execution:

- workflow run: `33759420404`;
- branch head: `b1553c3d3eb2273a831e517fde3daa0c35d0d6c3`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9894957071`;
- artifact SHA-256: `22ea8985687e7cdff2ad0a231a6765751a9889863f2481de8130c1cd6491e265`;
- aggregate canonical SHA-256: `d8ba60a84ac799ffa3c7cb54c808830d8ae4e0dd403bdba3fa5e86a3a79b63fa`;
- reference model semantics: `anthrosim-model-semantics-v26`.

### v26 reference result

The predeclared capability remains distinguished:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment has exactly **270 days** with visitor presence;
- intermittent treatments complete roughly **990–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or condition-mediated death.

The aggregate values in the v26 checked-in reference are:

- median total focal-person-day difference: **32 permille** (v20: 31);
- maximum paired total focal-person-day difference: **37 permille** (unchanged from v20);
- median intermittent peak-visitor share: **441 permille** (v20: 432);
- minimum intermittent peak-visitor share: **398 permille** (v20: 405).

All predeclared paired criteria therefore continue to support `capability_distinguished`. The workflow also separately verified deterministic intermittent replay and active-journey checkpoint/resume equivalence before reaching the frozen-reference comparison.

The independently preserved M9.6 travel-burden reference is **not rebaselined** for v26. For every declared seed, planned and observed transit days, planned and realized travel cost, and planned and realized route distance remain exactly equal to the existing `travel-burden-reference.json`. The changed values are population-dependent occupancy/journey outcomes, not changed travel-accounting semantics.

The v26 numerical shift is therefore preserved as a legitimate downstream consequence of corrected fertility stochastic coupling rather than tuned away. It does not alter the capability claim and is not archaeological or empirical validation.

## Historical reviewed reference — model semantics v20

The previous machine-readable reference recorded a reviewed execution under `anthrosim-model-semantics-v20`. Its exact identity remains available in Git history and provides the immediate numerical comparator for the v26 rebaseline.

Reviewed v20 execution:

- workflow run: `33260785876`;
- branch head: `b65b5ac45507c923a4cdba15ca26ca1165a8dc92`;
- pull-request merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9717232706`;
- artifact SHA-256: `a090a82a4a136cdf804c43549f73c758bf1541a2f0a506261da1c3af46668bc1`;
- aggregate canonical SHA-256: `19ec2e01b1a107897672c2b6edb16394bbf6cce0d9a88ab9939cc5014578c243`;
- model semantics: `anthrosim-model-semantics-v20`.

The v20 reference also classified the benchmark as `capability_distinguished` with **8/8** paired seeds passing. Its aggregate values were median focal-person-day difference **31 permille**, maximum **37 permille**, median intermittent peak-visitor share **432 permille**, and minimum **405 permille**.

## Earlier historical references

Earlier v6, v8 and v10 executions are preserved in Git history as historical regression evidence. They demonstrated the same broad capability classification across earlier demographic-time, resource-time and condition-mortality semantic generations, but their exact workflow identities and aggregate values are not the current checked-in regression reference.

In particular, the former v10 narrative reported a maximum paired focal-person-day difference of **36 permille**, median peak-visitor share of **426 permille**, and minimum peak-visitor share of **387 permille**. Those values are historical and must not be cited as the current machine-readable reference.

These historical references are evaluation evidence, not calibration targets.

## What the current result establishes

Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve that difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

The checked-in v27 reference verifies that this capability distinction survives the corrected background-mortality stochastic coupling represented by the reviewed AV4-002 execution. That is a regression/capability statement only; it is not empirical validation.

That does **not** establish invariance to all resource, demographic, condition or temporary-mobility alternatives. It establishes only the behavior of this frozen capability benchmark for the reviewed reference execution.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence or any social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation would require question-specific evidence, uncertainty propagation, structural sensitivity, calibration/corroboration separation and domain review.

Reference maintenance after a declared causal/model-semantics change is reproducibility work, not empirical calibration. A later model-semantics generation should replace this reference only after its benchmark execution and numerical scientific result have been independently reviewed.
