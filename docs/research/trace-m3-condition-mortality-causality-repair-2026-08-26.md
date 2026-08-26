# TRACE change record: M3 condition-mortality causality repair

**Date:** 2026-08-26  
**Issue:** #200  
**Model semantics:** `anthrosim-model-semantics-v10`  
**Status:** implementation, downstream terminology review and frozen-reference causal review complete in draft PR #242; the PR's exact-head workflow suite remains the final software-verification gate before merge

## 1. Defect

The pre-v10 model exposed a death cause named `ResourceScarcity`, a parameter named `maxScarcityMortalityProbabilityPerMillion`, and summary counts named `scarcityDeaths`. The executable hazard, however, read only the shared `condition` scalar. M4 permanent travel also reduced that scalar.

Therefore a travel-created condition deficit could later generate a death scientifically attributed to resource scarcity even when the M3 period had full positive food supply. This was a TRACE model-structure and causal-observability defect, not merely a terminology preference.

## 2. Alternatives evaluated

### A. Introduce a resource-specific physiological state

One possible repair was to add a separate nutrition/resource condition state and make a scarcity-specific hazard read only that state.

This was rejected for this hardening slice because AnthroSim does not currently possess evidence or a validated conceptual model for two interacting physiological state variables. Adding one solely to preserve the old cause label would create a new scientific mechanism, new initialization assumptions, new recovery/interaction rules and new calibration requirements.

### B. Reinterpret the existing shared-condition hazard as general condition-mediated mortality

This was chosen because it matches the state the executable model actually contains. `condition` already has multiple declared upstream pathways, notably M3 resource response and M4 travel cost. The hazard can therefore legitimately be stated as conditional on that shared state without claiming unsupported event-level apportionment.

This choice is deliberately conservative: it removes a false causal claim without adding a new physiological model.

## 3. Executable changes

The numerical condition-to-mortality function and v9 survival-equivalent elapsed-time conversion are retained.

Run-facing semantics change to:

- config parameter: `maxConditionMortalityProbabilityPerMillion`;
- death cause: `condition_mediated`;
- resource summary / annual metric: `conditionMortalityDeaths`;
- checkpoint RNG position: `resourceConditionMortality`.

The historical deterministic random stream label `resources/scarcity_mortality` is retained internally so an otherwise-equivalent run does not receive a different random sequence merely because the scientific label was corrected.

Several Rust-private identifiers also retain historical `scarcity` names to minimize mechanical execution churn. The normative scientific meaning is the versioned serialized contract, not those private identifiers.

## 4. Compatibility changes

The repair is intentionally fail-closed rather than aliasing the old cause semantics:

- model semantics v9 -> v10;
- experiment schema 9 -> 10;
- resource config schema 3 -> 4;
- resource state/summary schema 1 -> 2;
- event log schema 1 -> 2;
- metric series/snapshot schema 1 -> 2;
- checkpoint schema 9 -> 10;
- manifest schema 10 -> 11;
- sweep derived-analysis schema 2 -> 3;
- M7.6 derived reference snapshot schema 2 -> 3;
- M8 spatial-observability schema 2 -> 3;
- M8 benchmark aggregate/reference schema 1 -> 2;
- M9 benchmark aggregate/reference schema 1 -> 2.

The old config key is not accepted as an alias for the new parameter. v9 checkpoints remain incompatible through the model-semantics/schema boundary.

## 5. Controlled acceptance design

Dedicated core acceptance tests cover:

- actual M4 relocation condition loss followed by full positive M3 food supply and deterministic condition-mediated death;
- resource-only condition deterioration and deterministic condition-mediated death;
- mixed M4 travel and M3 resource deterioration;
- deterministic M4 enabled/disabled comparison with full local resource support;
- serialized event/summary/config names and rejection of the v9 config key.

These tests are causal/mechanistic checks, not evidence that the coefficients are realistic.

## 6. Downstream observability review

Research-facing derived surfaces that previously repeated scarcity-attributed death names were reviewed and updated:

- sweep derived JSON uses `conditionMortalityDeaths` and `meanConditionMortalityDeathsCompletedOnly`;
- sweep derived CSV uses `condition_mortality_deaths` and `mean_condition_mortality_deaths_completed_only`;
- the canonical M7.6 derived reference and CI projection use `meanConditionMortalityDeathsCompletedOnly`;
- M8 spatial observability uses `conditionMortalityDeaths` at per-cell and summary level;
- M8 benchmark aggregation uses `conditionMortalityDeaths`;
- M9 benchmark replay/aggregation uses `conditionMortalityDeaths` and `noConditionMortalityDeaths`;
- ODD, ODD+D, the scientific model, resource-model documentation, M8 observability documentation and M9 benchmark documentation now state the v10 shared-condition causal boundary.

Regression assertions require the new JSON/CSV names and reject the former derived scarcity-death names where those wire surfaces changed. The M7.6 CI gate additionally requires derived-analysis schema 3 and a v10/schema-3 frozen reference.

Legitimate upstream resource terminology remains. `resource scarcity`, unmet need, stock, harvest, consumption and resource interventions still describe the M3 resource mechanism. Private Rust identifiers and the historical RNG stream label may also retain `scarcity` for execution compatibility; they are not public scientific cause labels.

## 7. Frozen-reference causal review

The numerical condition hazard was intentionally not retuned. Frozen M7.6, M8.6 and M9.7 outputs were therefore regenerated and inspected before rebaselining rather than being overwritten automatically.

### M7.6

Reviewed execution before rebaseline:

- workflow run: `32931457083`;
- branch head: `f18b3d1d3242f363891affb9d1e55892b74fc6df`;
- merge-ref build: `de045e09e4a9550ea1d964ac644c22e0dac31e44`;
- derived artifact: `9593578301`;
- artifact SHA-256: `1356300fe21d029f7a5c0a8e1f0c3db36d23fbf1d906cb5bdd522b24f1a2667e`;
- generated sweep: `anthrosim-sweep-v2-e119a09bf3eb0393`.

All **144/144** planned runs completed across all 18 parameter points. Every frozen point-level scientific value matches the v9 reference exactly, including living population, occupied cells, mean condition, mortality count, unmet need, permanent-migration counts/distances and pooled move distance. The workflow failure occurred only because its projection still requested the removed `meanResourceScarcityDeathsCompletedOnly` key from schema-3 `points.json`.

The M7.6 reference therefore changes only declared reference schema/model-semantics identity, sweep/source identities and the mortality field name. The unchanged numerical count is now correctly described as `meanConditionMortalityDeathsCompletedOnly`; it is not an event-level resource-scarcity death estimate.

### M8.6

Reviewed execution before rebaseline:

- workflow run: `32930245492`;
- branch head: `a329f68f3278a600dece4193b9a6179d4e981180`;
- merge-ref build: `4a224061e4f4387430a33215518503b064810a1f`;
- artifact: `9593020274`;
- artifact SHA-256: `61295c9c97a13b30879784fa94f613e2a53312b6db60b4b749239801a1c8d182`;
- aggregate canonical SHA-256: `bf078fdfd5a43673bfef0ab76203af5fda673868d8d18b81c754e9b8682a1d7f`.

All 32 runs completed and the current classification remains `fragile_spatial_structure` with no robust primary metric. Every primary metric, paired-effect fraction, sign count and robust criterion matches the live v9 reference exactly. In particular, strong-vs-flat total migration distance remains approximately **11.83%** median absolute paired effect with signs **3 positive / 5 negative / 0 zero**. Terminal largest-cell share remains fragile; cell-time occupancy and terminal population Herfindahl remain not distinctive.

The required M8 reference changes are schema/model-semantics identity, experiment/configuration/state/aggregate identities, and the secondary mortality field name. No numerical primary scientific change is hidden by the rebaseline.

### M9.7

Reviewed execution before rebaseline:

- workflow run: `32930245559`;
- branch head: `a329f68f3278a600dece4193b9a6179d4e981180`;
- merge-ref build: `4a224061e4f4387430a33215518503b064810a1f`;
- artifact: `9593016422`;
- artifact SHA-256: `561f828adec030fba9879b9a354f285b4b10b9f0431b16591134e539d866bb08`;
- aggregate canonical SHA-256: `4a7e4a95edbb01f0ab7371d313bd24e5f989cf2371a022ed6dd62fe4426f8a07`.

The benchmark remains `capability_distinguished`; all **8/8** paired seeds pass every predeclared criterion. Paired resident person-days, every seed-level outcome and all aggregate values are numerically unchanged. The preserved aggregates remain median total focal-person-day difference **31 permille**, maximum **36 permille**, median intermittent peak-visitor share **426 permille**, and minimum **387 permille**. Duplicate replay remains exact and active-journey checkpoint/resume remains exact.

The required M9 reference changes are schema/model-semantics identity, experiment/state/aggregate identities, and the causal-neutral mortality fields. The control criterion now establishes zero condition-mediated deaths, not zero uniquely resource-caused deaths.

## 8. CI evidence and exact-head gate

Early intermediate heads exposed ordinary implementation/maintenance defects rather than a contradictory scientific result:

- `96319d3c12d18edc5e92b2a6d75e7a14f737032f` reached M9 reference comparison after successful execution/replay/resume but still had a rustfmt difference and one later-corrected test import;
- inherited head `a329f68f3278a600dece4193b9a6179d4e981180` passed Clippy and the broad workspace test body, including all five issue-#200 causal acceptance tests, but two stale spatial-observability assertions still expected schema 2 instead of 3;
- the M8.6 and M9.7 workflows on `a329f68...` completed their scientific executions successfully and failed only because the deliberately frozen references still expected benchmark schema 1. Those artifacts were the reviewed material described above.

On later head `f18b3d1d3242f363891affb9d1e55892b74fc6df`, every dedicated PR workflow passed: M8.6, M9.7, spatial observability, M8.6 data preparation, cross-platform determinism, spatial mechanism determinism, landscape loading, landscape preprocessing, source provenance, resumed-Explorer compatibility and deterministic bundle integration. Umbrella CI passed formatting, Clippy, the complete workspace tests, Explorer/script validation, release build, core benchmarks, the 1000-run soak, performance/memory acceptance and M5/M6 bundle integration. Its sole failure was the stale M7.6 scarcity-named projection described above; the underlying 144-run M7.6 execution itself completed successfully and supplied the reviewed v10 artifact.

The stale spatial and M7.6 schema/terminology guards were updated only after confirming production output had intentionally advanced and the numerical scientific outputs were unchanged. M7/M8/M9 references were rebaselined only after the causal/numerical reviews above.

The authoritative final software-verification evidence is the GitHub Actions suite attached to the eventual PR branch head after these documentation/reference changes. Before merge it must be green on that exact head, including umbrella CI/Clippy/workspace tests and the dedicated spatial, M8.6, M9.7, soak/performance/memory/bundle checks that the repository requires. A green suite is implementation/regression evidence only; it is not empirical validation.

## 9. Limitations retained

- `condition` does not preserve a quantitative decomposition by upstream causal source.
- `condition_mediated` is therefore not a claim that the model knows whether nutrition, travel, initialization or a combination explains an individual's deficit.
- resource and travel coefficients remain synthetic/unvalidated.
- #208 still governs coincident M3 condition-mediated and M2 demographic mortality competing-risk attribution.
- the repair does not calibrate or validate any archaeological population or case-study scenario.

## 10. TRACE interpretation

If the exact-head software-verification gate passes, the defensible claim is:

> AnthroSim no longer labels a death as resource scarcity solely because a shared condition deficit generated the M3 hazard. The event and derived count now state the executable general condition-mediated mechanism, while resource and travel pathways remain separately observable upstream.

This is a model-semantics/cause-attribution repair. It is not empirical validation of the mortality mechanism.
