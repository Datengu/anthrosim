# TRACE change record: M3 condition-mortality causality repair

**Date:** 2026-08-26  
**Issue:** #200  
**Model semantics:** `anthrosim-model-semantics-v10`  
**Status:** implementation in PR #242; final CI/reference evidence to be appended before merge

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
- manifest schema 10 -> 11.

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

The repair must review all derived surfaces that previously repeated `scarcityDeaths`. In particular, sweep-derived JSON/CSV is a scientific output and must not retain `resource_scarcity_deaths` after the core cause has become general condition-mediated mortality.

Resource-specific causal evidence remains separately observable through unmet need, resource stock, harvest, consumption and controlled resource interventions.

## 7. Reference-output expectation

The numerical mortality trajectory is intentionally not retuned in this repair. Nevertheless frozen references can legitimately change because:

- model-semantics identity changes;
- input serialization/schema changes;
- resource-state schema version participates in the resource/state digest;
- event/metric/artifact schemas change.

Any M7/M8/M9 reference update must therefore be inspected rather than automatically overwritten. Population/movement/scientific classifications should remain numerically consistent unless a changed artifact identity/digest is itself part of the compared output.

## 8. Initial CI evidence

On intermediate head `96319d3c12d18edc5e92b2a6d75e7a14f737032f`:

- M9.7 built release binaries, completed paired ensembles, derived M9.6 observability, proved identical intermittent replay and active-checkpoint resume exactness, then failed only at the preserved v9 scientific-reference comparison;
- core CI stopped at a rustfmt-only difference in `resources.rs` before Clippy/tests;
- source provenance, spatial/landscape, bundle/resume and cross-platform workflows that completed were green.

The rustfmt difference was corrected without a semantic change. Later intermediate test compilation found one isolated incorrect test import, also corrected. Final exact-head evidence remains required before merge.

## 9. Limitations retained

- `condition` does not preserve a quantitative decomposition by upstream causal source.
- `condition_mediated` is therefore not a claim that the model knows whether nutrition, travel, initialization or a combination explains an individual's deficit.
- resource and travel coefficients remain synthetic/unvalidated.
- #208 still governs coincident M3 condition-mediated and M2 demographic mortality competing-risk attribution.
- the repair does not calibrate or validate any archaeological population or a specific archaeological site scenario.

## 10. TRACE interpretation

If final acceptance/CI passes, the defensible claim is:

> AnthroSim no longer labels a death as resource scarcity solely because a shared condition deficit generated the M3 hazard. The event and derived count now state the executable general condition-mediated mechanism, while resource and travel pathways remain separately observable upstream.

This is a model-semantics/cause-attribution repair. It is not empirical validation of the mortality mechanism.
