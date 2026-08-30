# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `d0986a833e68a3682e831a2ed1b9ffea174f7a9d`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

## Current regression reference — model semantics v20

The current machine-readable regression reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`. It records a reviewed execution under `anthrosim-model-semantics-v20`; this human-readable narrative is intentionally synchronized to that exact checked-in reference rather than to an older historical rebaseline.

The repository's executable `MODEL_SEMANTICS_ID` may advance beyond the reference semantics as later scientific repairs merge. That does not by itself authorize rewriting or rebaselining this result: a new reference must come from a reviewed benchmark execution, with its numerical result inspected before replacement. Exact historical identities are regression evidence, not calibration targets.

Reviewed reference execution:

- workflow run: `33260785876`;
- branch head: `e3ba2e12ea2ae38f5f4be0e0e2b6ff9ae1eb9513`;
- pull-request merge-ref build: `f6ecc05ceeb800ae442820f5ad87eb28951474da`;
- artifact: `9717232706`;
- artifact SHA-256: `a090a82a4a136cdf804c43549f73c758bf1541a2f0a506261da1c3af46668bc1`;
- aggregate canonical SHA-256: `19ec2e01b1a107897672c2b6edb16394bbf6cce0d9a88ab9939cc5014578c243`;
- reference model semantics: `anthrosim-model-semantics-v20`.

### v20 reference result

The predeclared capability remains distinguished:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment has exactly **270 days** with visitor presence;
- intermittent treatments complete roughly **990–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or condition-mediated death.

The aggregate values in the checked-in reference are:

- median total focal-person-day difference: **31 permille**;
- maximum paired total focal-person-day difference: **37 permille**;
- median intermittent peak-visitor share: **432 permille**;
- minimum intermittent peak-visitor share: **405 permille**.

The paired scientific criteria therefore continue to support `capability_distinguished`. The reference schema remains version 2 and uses the general condition-mediated mortality terminology (`conditionMortalityDeaths` / `noConditionMortalityDeaths`). Experiment identities, state digests, source identities and aggregate identities are properties of this reviewed execution and must not be treated as calibration targets for later model-semantic generations.

The workflow also gates non-statistical replay behavior separately from the stored classification, including deterministic replay and checkpoint/resume equivalence. The machine-readable classification records this as `workflow-gated-separately` rather than embedding those workflow checks as statistical benchmark evidence.

This remains an informative null sensitivity result for the mortality pathway exercised by the benchmark. Permanent migration is disabled, resource demand is deliberately low, and both arms record zero condition-mediated deaths, so this benchmark establishes a temporary-presence capability distinction without claiming empirical mortality calibration or resource-scarcity attribution.

## Historical references

Earlier v6, v8 and v10 executions are preserved in Git history as historical regression evidence. They demonstrated the same broad capability classification across earlier demographic-time, resource-time and condition-mortality semantic generations, but their exact workflow identities and aggregate values are not the current checked-in regression reference.

In particular, the former v10 narrative reported a maximum paired focal-person-day difference of **36 permille**, median peak-visitor share of **426 permille**, and minimum peak-visitor share of **387 permille**. Those values are historical and must not be cited as the current machine-readable reference.

These historical references are evaluation evidence, not calibration targets.

## What the current result establishes

Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve that difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

The checked-in v20 reference verifies that this capability distinction survived the model semantics represented by that reviewed execution. That is a regression/capability statement only; it is not empirical validation, and it does not imply that later semantics have been rebaselined until a later reviewed reference explicitly replaces it.

That does **not** establish invariance to all resource, demographic, condition or temporary-mobility alternatives. It establishes only the behavior of this frozen capability benchmark for the reviewed reference execution.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence or any social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation would require question-specific evidence, uncertainty propagation, structural sensitivity, calibration/corroboration separation and domain review.

Reference maintenance after a declared causal/model-semantics change is reproducibility work, not empirical calibration. A later model-semantics generation should replace this reference only after its benchmark execution and numerical scientific result have been independently reviewed.
