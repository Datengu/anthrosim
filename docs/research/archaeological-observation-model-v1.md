# Archaeological observation-model contract v1

**Status:** normative downstream research contract  
**Scope:** empirical archaeological comparison, TRACE model-output verification/corroboration  
**Scientific status:** framework contract; no archaeological proxy or taphonomic coefficient is supplied as a universal default

## 1. Boundary

AnthroSim state is not the archaeological record.

A research study that compares simulated occupancy, presence, journeys, person-days, resource use, mortality, movement or other model quantities with archaeological evidence must place an explicit observation model between the simulated quantity and the empirical observable unless it justifies that the measured quantity is already directly comparable.

The required conceptual separation is:

```text
AnthroSim authoritative/derived state
        ↓
explicit observation model
        ↓
hypothesized deposition / material production
        ↓
preservation / taphonomic survival
        ↓
survey or excavation sampling
        ↓
recovery / recording
        ↓
archaeological observable used in comparison
```

The observation model is **downstream analysis**. It does not alter a run, model semantics, checkpoints, events, metrics, landscape state or deterministic simulation identity.

## 2. Why this is required

The following implications are not valid without a declared observation model:

```text
high simulated person-days -> high artefact density
no persistent residence     -> no archaeology
high visitor presence       -> strong excavation signal
low simulated activity      -> archaeological absence
```

Material production, cleaning, discard practice, preservation, later disturbance, sampling and recovery can all break those direct correspondences. In some cases there is no defensible proxy at all.

A negative archaeological observation is therefore not automatically a simulated absence. It may instead represent non-deposition, preservation loss, unsampled space or non-detection.

## 3. Versioned machine-readable contract

`scripts/research-archaeological-observation.py` accepts:

```text
--model <observation-model.json>
--simulated <simulated-values.json>
--output <observation-result.json>
```

The model uses strict JSON: unknown fields fail closed.

Top-level observation-model fields are:

- `schemaVersion`: currently `1`;
- `observationModelId`: human-readable versioned model identifier;
- `comparisonId`: identity of the empirical comparison being attempted;
- `simulationSource`: provenance for the AnthroSim-derived input;
- `evidenceSource`: provenance for the archaeological evidence;
- `evidenceRole`: one of `calibration`, `validation`, `corroboration`, or `descriptive`;
- `mappings`: one or more explicit simulated-variable-to-observable mappings.

Both source records require:

- `id`;
- `kind`;
- `contentSha256`;
- `reference`.

The hash is the content identity supplied by the study. The observation layer does not silently fetch or mutate either source.

This keeps three identities separate:

1. simulation source identity;
2. archaeological evidence identity;
3. observation-model identity.

Changing a taphonomic or sampling assumption therefore creates a different observation-model/result identity without changing the underlying AnthroSim run.

## 4. Mapping types

### `independent_detection_count`

This is the deliberately simple v1 verification mapping. It is appropriate only when the study can defend a count of exchangeable simulated opportunities and an independent staged detection approximation.

A mapping declares four integer probabilities in parts per million:

```text
depositionPerMillion
preservationPerMillion
samplingPerMillion
recoveryPerMillion
```

The effective probability is exactly:

```text
p = deposition × preservation × sampling × recovery
```

with each term divided by `1,000,000`.

For simulated count `N`, the declared observable distribution is:

```text
Detected ~ Binomial(N, p)
```

The tool emits exact rational `successProbability` and `expectedDetectedCount`; it does not use floating-point arithmetic or make a stochastic draw.

This mapping is a framework capability, not an archaeological law. A real study must justify whether the independence assumption, count unit and each stage are defensible. More complex observation models may be implemented as separately versioned downstream methods when the question requires them.

### `no_direct_observable`

This is scientifically first-class, not an error condition.

Use it when a simulated variable has no defensible direct archaeological proxy. The mapping must set:

```json
"archaeologicalObservable": null
```

and must not supply deposition/preservation/sampling/recovery probabilities.

The result is `not_comparable` with absence semantics `no_defensible_direct_archaeological_mapping`.

This prevents the framework from forcing every model quantity into a pseudo-empirical comparison.

## 5. Required assumptions and uncertainty

Every mapping requires:

- a non-empty `assumptions` array;
- a non-empty `uncertaintyNote`.

These fields are intentionally mandatory even for synthetic verification fixtures. A real study should use them to record at least the material-production hypothesis, relevant taphonomic assumptions, sampling/recovery coverage and known bias or uncertainty.

Where more than one observation model is defensible, run each model separately. Because observation-model identity is content-derived, those alternatives remain provenance-distinct and can be treated as structural sensitivity rather than silently selecting one preferred mapping.

## 6. Evidence roles and the held-out firewall

`evidenceRole` is explicit so the same observed pattern cannot quietly move between calibration and independent corroboration during downstream analysis.

This contract complements the study evidence-role firewall. It does not by itself prove that evidence was held out correctly; the study protocol and evidence-role audit remain responsible for that lifecycle. The observation result preserves the declared role so later analysis can audit the distinction.

## 7. Absence semantics

For an `independent_detection_count` mapping:

- if the simulated source count is zero, the result records `simulated_absence`;
- if the simulated source count is positive, the result records `non_detection_possible_after_deposition_preservation_sampling_recovery`.

The latter means an observed zero cannot be interpreted as proof of no simulated activity merely from this mapping.

For `no_direct_observable`, absence or presence in the model has no direct archaeological comparison under the declared contract.

A study with spatially explicit unsampled areas, heterogeneous preservation, false positives, excavation-unit geometry or other detection structure should implement those mechanisms explicitly rather than overloading this simple v1 mapping.

## 8. Synthetic verification fixture

The regression test supplies `100` simulated activity units and declares:

```text
deposition   = 0.5
preservation = 0.8
sampling     = 0.5
recovery     = 0.5
```

Therefore:

```text
p = 0.5 × 0.8 × 0.5 × 0.5 = 0.1
Detected ~ Binomial(100, 0.1)
E[Detected] = 10
```

The test requires the tool to emit exact probability `1/10` and exact expected count `10/1`.

It also verifies:

- an explicit `no_direct_observable` mapping;
- zero simulated activity being distinguished from non-detection;
- missing detection stages being rejected;
- unknown fields being rejected;
- simulation-source identity mismatch being rejected;
- a null mapping being forbidden from inventing an archaeological observable.

This fixture verifies the observation-model machinery. It is not a calibrated taphonomic model.

## 9. Interpretation rule

A research result may compare AnthroSim with archaeology only through one of these declared routes:

1. a versioned observation-model mapping that preserves simulation, observation-model and evidence provenance separately; or
2. an explicit study justification that the measured empirical quantity is already directly comparable and no additional observation transformation is required.

Otherwise the simulated quantity remains a model observable, not archaeological evidence.

## 10. Research gate

Before archaeological pattern matching is presented as empirical validation or independent corroboration, the study must document which target quantities are:

- directly comparable;
- indirectly comparable through a declared observation model;
- not defensibly comparable.

Changing the observation model is part of analysis sensitivity and must not mutate or re-identify the underlying simulation run.
