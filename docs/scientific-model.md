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

Persistent individuals have stable identity, birth time/age, reproductive sex for the initial reproduction model, health/condition, location, household membership, parent references, and minimal resource/decision state. Reproductive sex is a deliberately limited biological state variable for the v0.1 birth mechanism; it is not a model of social gender.

### Households

A household is a co-resident resource-sharing unit. It is not assumed to be a tribe, clan, lineage, settlement, marriage, or universal nuclear-family structure. Parentage and household membership are separate relationships.

### World cells

Cells contain movement cost, water accessibility, baseline food productivity, renewable food stock, seasonality, temporary environmental stress, and occupancy/density information.

In M1, these values form a **synthetic engine-validation landscape**. Environmental ratios are dimensionless fixed-point permille values. Relative elevation, water access, productivity, movement cost, and seasonality are spatially autocorrelated synthetic fields; they do not correspond to measured palaeoenvironmental units and must not be interpreted as empirical geography.

### Time

Authoritative simulation time is represented in integer days. Systems execute at explicit event or periodic boundaries rather than through continuous real-time ticking.

### Space

v0.1 uses a synthetic rectangular grid with deterministic north/east/south/west adjacency and hard boundaries. Real Earth and palaeoenvironmental reconstruction are explicitly deferred.

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
- synthetic world dimensions and generator schema;
- initial population count and spatial clustering (from M2 onward);
- starting age/reproductive-sex distribution (from M2 onward);
- household initialization rule (from M2 onward);
- demographic schedule/preset and provenance status;
- food/environment parameters;
- migration parameters;
- duration/stop condition.

No default parameter should be described as empirically realistic until a source and validation rationale are documented.

A uniform random age distribution is not considered a scientifically neutral default. M2 will distinguish deterministic synthetic test initialization from schedule-consistent or empirical initialization.

## 6. Input data

v0.1 has no external anthropological or archaeological **runtime dataset**. The M1 environment is synthetic. M2 demographic mechanisms are informed by published comparative evidence, documented in `docs/research/demography-v0.1.md`, but the first executable demographic preset remains explicitly synthetic for engine validation.

Later empirical demographic presets may encode published age-specific schedules or derived tables. Such presets must retain source identity, transformation notes, units and provenance status.

## 7. Submodels planned for v0.1

### Synthetic environment (M1)

The initial environment is generated from several smoothly varying deterministic fields derived from the run's `world` random stream. Water accessibility depends partly on synthetic wetness and lowland favourability; productivity combines water, a separate fertility field, and lowland favourability; movement cost combines ruggedness and relative elevation. Seasonality uses a synthetic latitude gradient plus a spatial climate field.

These relationships exist to provide heterogeneous, causally inspectable test conditions for later systems. They are **not yet evidence-grounded ecological equations**.

### Food and resource renewal

Cells expose renewable food stock based on baseline productivity and seasonality. Households gather locally and consume shared resources. Scarcity reduces condition and changes mortality/migration pressure.

### Demography

M2 uses **replaceable age-specific schedules rather than hidden hard-coded anthropological constants**. Ageing is derived from birth time. Birth and death processes are parameterised, stochastic under named deterministic streams, and event-recorded.

The demographic research baseline is `docs/research/demography-v0.1.md`. It establishes that extant hunter-gatherer populations show substantial demographic diversity and should be used as comparative evidence/ranges rather than as one universal prehistoric proxy.

The first executable preset is therefore named `synthetic_validation_v1`, not `hunter_gatherer`. It may use evidence-informed ranges while remaining explicitly non-empirical. Future population-specific presets must identify their sources and validation targets.

Initial mortality is represented by a transparent piecewise age-specific annual hazard schedule. Initial fertility is represented by an age-specific opportunity/hazard schedule plus an explicit simplified postpartum spacing mechanism. Completed family size and life expectancy are outputs/validation quantities, not values directly forced on individuals.

### Households

Households share local resources and may coordinate migration. Formation/dissolution rules remain minimal until evidence-grounded social modules are introduced. Household membership is not equivalent to parentage or marriage.

### Migration

Households/individuals compare staying with a bounded set of locally plausible alternatives. Movement incurs distance/environment costs and uncertainty. No destination is selected because of its real-world historical importance.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting;
- valid IDs and genealogy references;
- no impossible ages or household membership states;
- no self-parent or duplicate-parent relationships;
- conservation/accounting rules for modelled resources where applicable;
- deterministic replay under the supported platform/build boundary;
- deterministic same-seed world generation and stable cell adjacency;
- isolation of named RNG streams so demographic changes cannot alter world generation;
- directional tests (for example, severe sustained resource scarcity must not systematically improve health).

## 9. Validation plan

The demographic literature baseline now documents comparative ranges and limitations, but v0.1 remains unvalidated. A research-capable model will require:

- population-specific or explicitly sampled demographic parameterizations where appropriate;
- comparison of simulated survivorship, age-specific mortality, fertility, birth spacing and growth against claimed calibration targets;
- comparison with known hunter-gatherer mobility/demographic patterns where appropriate;
- calibration only where justified by a stated research question;
- global and local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. First candidate experiment

A reasonable first research-style exercise after implementation is:

> How does the magnitude and temporal variability of local resource productivity affect population persistence, spatial fragmentation, and migration distance in the simplified v0.1 model?

This would be an engine/model validation exercise rather than a claim about actual prehistory.
