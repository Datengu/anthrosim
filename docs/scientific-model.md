# Scientific model specification (ODD-oriented)

**Status:** v0.1 working model specification  
**Scientific status:** exploratory / unvalidated

This document describes the scientific meaning of AnthroSim v0.1 separately from its software architecture. It follows the spirit of the ODD (Overview, Design concepts, Details) protocol used for agent-based models, while remaining concise until the model is implemented.

## 1. Purpose

v0.1 asks a deliberately narrow methodological question:

> Can a spatially explicit population of persistent individuals and households produce interpretable patterns of demographic growth, decline, fragmentation, and migration when local resource conditions vary, without scripted destinations or higher-level social institutions?

v0.1 is not intended to reconstruct a particular prehistoric population or produce quantitative claims about real Homo sapiens.

## 2. Entities, state variables, and scales

### People

Persistent individuals have stable identity, birth time/age, sex for the initial reproduction model, health/condition, location, household membership, parent references, and minimal resource/decision state.

### Households

A household is a co-resident resource-sharing unit. It is not assumed to be a tribe, clan, lineage, settlement, or political institution.

### World cells

Cells contain movement cost, water accessibility, baseline food productivity, renewable food stock, seasonality, temporary environmental stress, and occupancy/density information.

### Time

Authoritative simulation time is represented in integer days. Systems execute at explicit event or periodic boundaries rather than through continuous real-time ticking.

### Space

v0.1 uses a synthetic grid. Real Earth and palaeoenvironmental reconstruction are explicitly deferred.

## 3. Process overview and scheduling

The intended order at each due simulation boundary is:

1. advance world time;
2. apply environmental updates;
3. process scheduled demographic/life-history events;
4. update eligible resource, health, and household state;
5. reevaluate eligible migration decisions;
6. apply movements/state transitions in deterministic order;
7. emit events and aggregate metrics;
8. verify invariants at configured intervals;
9. write outputs/checkpoints when due.

Exact scheduling is a model component and will be versioned.

## 4. Design concepts

### Emergence

Population clusters, fragmentation, and migration corridors should emerge from local conditions. No rule may directly instruct a population to form a civilisation, tribe, village, or historical migration route.

### Adaptation / decision-making

Migration uses an interpretable local utility or probability model. Candidate destinations are knowledge-limited rather than globally optimal.

A starting conceptual form is:

```text
move utility = expected resources
             + water/security benefit
             + kin proximity
             - travel cost
             - uncertainty
             - relocation risk
```

Weights are explicit experimental parameters, not hidden tuning constants.

### Interaction

v0.1 interaction is primarily household resource sharing, reproduction context, local density pressure, and spatial competition for resources. Global all-to-all interaction is prohibited.

### Stochasticity

Randomness represents unresolved variability in processes such as initialization and demographic/migration outcomes. All stochasticity comes from seeded named deterministic streams.

### Observation

The model emits both ground-truth events and aggregate measurements. Interpretive labels are derived outputs and must not be confused with authoritative events.

## 5. Initialization

A run specifies:

- seed;
- synthetic world dimensions and generator parameters;
- initial population count and spatial clustering;
- starting age/sex distribution;
- household initialization rule;
- demographic parameters;
- food/environment parameters;
- migration parameters;
- duration/stop condition.

No default parameter should be described as empirically realistic until a source and validation rationale are documented.

## 6. Input data

v0.1 has no external anthropological or archaeological input dataset. Synthetic environment and placeholder demographic parameters are generated/configured internally. This is intentional so engine verification can precede empirical claims.

## 7. Submodels planned for v0.1

### Food and resource renewal

Cells expose renewable food stock based on baseline productivity and seasonality. Households gather locally and consume shared resources. Scarcity reduces condition and changes mortality/migration pressure.

### Demography

Ageing is derived from birth time. Birth and death processes are parameterised and event-recorded. Fertility, spacing, and mortality models begin simple and replaceable.

### Households

Households share local resources and may coordinate migration. Formation/dissolution rules remain minimal until evidence-grounded social modules are introduced.

### Migration

Households/individuals compare staying with a bounded set of locally plausible alternatives. Movement incurs distance/environment costs and uncertainty. No destination is selected because of its real-world historical importance.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting;
- valid IDs and genealogy references;
- no impossible ages or household membership states;
- conservation/accounting rules for modelled resources where applicable;
- deterministic replay under the supported platform/build boundary;
- directional tests (for example, severe sustained resource scarcity must not systematically improve health).

## 9. Validation plan (future)

v0.1 defaults are placeholders until grounded. A research-capable model will require:

- literature-backed demographic ranges;
- comparison with known hunter-gatherer mobility/demographic patterns where appropriate;
- calibration only where justified by a stated research question;
- global and local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

## 10. First candidate experiment

A reasonable first research-style exercise after implementation is:

> How does the magnitude and temporal variability of local resource productivity affect population persistence, spatial fragmentation, and migration distance in the simplified v0.1 model?

This would be an engine/model validation exercise rather than a claim about actual prehistory.
