# M3 condition-mediated mortality contract v1

**Status:** normative executable contract for `anthrosim-model-semantics-v10`  
**Scope:** shared condition state, condition-mediated mortality cause semantics, M3 resource and M4 travel pathways  
**Scientific status:** implementation/model-contract specification; **not empirical validation**

## Purpose

Before v10, AnthroSim labelled the M3 condition-dependent death mechanism `ResourceScarcity` and exposed `scarcityDeaths`, even though the executable probability depended only on the person's shared `condition` scalar. M4 permanent travel also reduces that same condition state. A person could therefore be fully supplied with food at the next M3 settlement and still die from a travel-created condition deficit while the event was scientifically labelled as resource scarcity.

Issue #200 identified this as a causal-attribution defect. v10 resolves it by defining the existing shared-condition hazard as **general condition-mediated mortality**. It does not introduce a second resource-only physiology state and does not claim that travel, nutrition or condition are empirically calibrated.

## 1. What `condition` means in v10

`condition` is a bounded synthetic health/energetic mediator on `0..1000`.

It is intentionally more general than nutritional status. Current authoritative pathways that can determine or change it include:

- founder condition supplied by the selected population initialization mode;
- newborn condition under the M2 newborn-inheritance rule;
- M3 resource supply, recovery and shortfall response;
- M4 permanent-migration travel condition cost.

A future mechanism may affect condition only if that pathway is made explicit in the model contract and provenance.

The model does **not** currently maintain a decomposition such as "300 points resource-derived + 200 points travel-derived". Once multiple mechanisms have changed the scalar, the exact causal contribution of each to the current deficit is not recoverable from `condition` alone.

## 2. The mortality hazard

At an M3 settlement, the person's current condition determines a reference-quarter condition-mediated probability using the existing v9 equation:

`q = condition_deficit_fraction × maxConditionMortalityProbabilityPerMillion`

where zero condition receives the configured maximum reference-quarter probability and condition 1000 receives probability zero.

The v9 elapsed-time conversion remains unchanged: arbitrary M3 intervals use the survival-equivalent rational conversion defined by [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md). v10 changes the scientific cause/parameter meaning, not the numerical time-scaling rule.

The deterministic random stream sequence is also deliberately preserved. Its private historical stream label remains `resources/scarcity_mortality` for trajectory comparability; that internal implementation identifier is **not** the scientific semantics of v10 output.

## 3. Authoritative cause semantics

A death from this hazard serializes as:

`cause = "condition_mediated"`

This means:

> the person's shared condition state generated the configured condition-mediated mortality hazard at this boundary.

It does **not** mean:

- resource scarcity specifically caused the death;
- the condition deficit was nutritional;
- travel specifically caused the death;
- the event identifies the relative contribution of multiple upstream mechanisms; or
- the probability is an empirical estimate of mortality at that condition value.

Resource shortage can still causally increase condition-mediated mortality by lowering condition. The event cause simply does not overclaim that resource shortage was the unique cause when the executable state cannot support that attribution.

## 4. Run-facing observability

v10 uses causal-neutral mortality naming in authoritative and derived artifacts:

- resource configuration: `maxConditionMortalityProbabilityPerMillion`;
- death event cause: `condition_mediated`;
- resource summary and annual resource metric: `conditionMortalityDeaths`;
- checkpoint RNG position: `resourceConditionMortality`.

The old run-facing names `maxScarcityMortalityProbabilityPerMillion`, `resource_scarcity`, and `scarcityDeaths` are not v10 scientific output names.

Some Rust-private identifiers retain historical `scarcity` wording to avoid unnecessary execution-code churn. Those identifiers are implementation history only and must not be used to interpret artifacts.

## 5. Compatibility boundary

The semantics change is deliberately fail-closed:

- `MODEL_SEMANTICS_ID`: v9 -> v10;
- `ExperimentConfig`: schema 9 -> 10;
- `ResourceConfig`: schema 3 -> 4;
- `ResourceSystem` / `ResourceSummary`: schema 1 -> 2;
- event log: schema 1 -> 2;
- metric series/snapshot: schema 1 -> 2;
- checkpoint: schema 9 -> 10;
- manifest: schema 10 -> 11.

The v10 resource configuration does not accept the former scarcity-specific parameter name as an alias. A v9 checkpoint cannot be scientifically resumed as v10 through the existing semantics/schema checks.

## 6. Required causal acceptance cases

The implementation must verify at minimum:

1. **Travel-only deficit + full positive food supply:** M4 travel lowers condition, M3 records no unmet food need, and a deterministic condition death serializes as `condition_mediated`, never `resource_scarcity`.
2. **Resource-only deficit:** unmet food need lowers condition and can still directionally produce condition-mediated mortality.
3. **Mixed travel + resource deficit:** both upstream pathways are demonstrably present, while the death event remains general rather than inventing unsupported cause apportionment.
4. **Migration enabled vs disabled under full local resource support:** enabling the controlled M4 relocation is the only source of the travel-condition decrement.
5. **Wire contract:** v10 artifacts/configuration expose the general condition-mediated names and reject the old scarcity-specific config field.
6. **Probability reconciliation:** the recorded event probability continues to correspond to the existing elapsed-time condition hazard contract.

## 7. Relationship to resource analysis

This repair does not make resource scarcity scientifically invisible. Resource-specific outputs remain available and should be used when asking resource questions, including:

- unmet resource need;
- household periods with unmet need;
- food regeneration, stock, harvest and consumption;
- condition trajectories under controlled resource interventions.

A defensible resource-causality analysis therefore asks whether manipulating resource inputs changes condition-mediated mortality while other relevant pathways are controlled. It must not treat `conditionMortalityDeaths` as a direct food-scarcity death count.

## 8. Relationship to M4 travel

M4 travel condition cost remains an explicit synthetic movement consequence. Under v10 it is legitimate for sufficiently low post-travel condition to affect the general condition-mediated hazard, because that pathway is now declared rather than disguised as resource mortality.

This does **not** validate the travel-condition coefficient, imply that real migration raises mortality by the simulated amount, or establish a journey injury/energetics model.

## 9. Remaining competing-risk limitation

Issue #208 remains separate. On a day when the condition-mediated M3 hazard and annual M2 demographic mortality coincide, total and cause-specific mortality still require an explicit competing-risk attribution contract. v10 corrects what the M3 hazard itself means; it does not by itself solve same-boundary competition between M3 and M2.

## 10. Interpretation rule

Valid v10 language is:

> Under the stated synthetic assumptions, intervention X changed the shared condition state and therefore changed condition-mediated mortality within the model.

When a controlled experiment isolates resources, it can additionally say:

> Under otherwise fixed assumptions, the resource intervention changed condition and condition-mediated mortality within the model.

Invalid language is:

> The simulator observed N resource-scarcity deaths,

because the executable condition state does not preserve sufficient causal provenance to justify that event-level attribution.
