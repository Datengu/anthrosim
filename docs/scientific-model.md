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

Persistent individuals have stable identity, epoch-relative birth time/derived age, reproductive sex for the initial reproduction model, health/condition, location, household membership, parent references, birth-history state and death state. Dead individuals remain persistent records so genealogy references do not change when a person dies.

Reproductive sex is a deliberately limited biological state variable for the v0.1 birth mechanism; it is not a model of social gender.

### Households

A household is a co-resident resource-sharing unit. It is not assumed to be a tribe, clan, lineage, settlement, marriage, or universal nuclear-family structure. Parentage and household membership are separate relationships.

In M2, a newborn is assigned to the female parent's current household and cell. This is a transparent synthetic co-residence rule required to place a newborn into valid state; it is not an empirical claim about universal residence systems.

### World cells

Cells contain movement cost, water accessibility, baseline food productivity, renewable food stock, seasonality, temporary environmental stress, and occupancy/density information.

In M1, these values form a **synthetic engine-validation landscape**. Environmental ratios are dimensionless fixed-point permille values. Relative elevation, water access, productivity, movement cost, and seasonality are spatially autocorrelated synthetic fields; they do not correspond to measured palaeoenvironmental units and must not be interpreted as empirical geography.

### Time

Authoritative simulation time is represented in integer days. M2 demographic hazards are evaluated at explicit annual boundaries. This is a temporal approximation, not a claim that real demographic events occur annually or synchronously.

### Space

v0.1 uses a synthetic rectangular grid with deterministic north/east/south/west adjacency and hard boundaries. Real Earth and palaeoenvironmental reconstruction are explicitly deferred.

## 3. Process overview and scheduling

The intended high-level v0.1 order at each due simulation boundary is:

1. advance world time;
2. apply environmental updates when implemented;
3. process scheduled demographic/life-history events;
4. update eligible resource, health, and household state when implemented;
5. reevaluate eligible migration decisions when implemented;
6. apply movements/state transitions in deterministic order;
7. emit events and aggregate metrics;
8. verify invariants at configured intervals;
9. write outputs/checkpoints when due.

For implemented M2 annual demographic boundaries, the order is specifically:

1. evaluate mortality for living person records that existed at the start of the boundary;
2. if nobody remains alive, stop with `population_extinct`;
3. evaluate fertility among surviving female records that existed at the start of the boundary;
4. apply minimum birth-spacing eligibility;
5. require at least one eligible living male in the female's current cell;
6. perform the configured fertility draw;
7. select one eligible local male uniformly by deterministic reservoir sampling;
8. create the newborn with stable parent IDs, the female parent's location/household, and a deterministic reproductive-sex draw;
9. rebuild the spatial occupancy index after births;
10. stop explicitly if the operational persistent-record limit is reached.

Newborns are therefore not exposed to mortality until a later annual boundary. Scheduling is part of the model definition and must be treated as scientifically consequential in later sensitivity/validation work.

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

M2 parent selection is intentionally minimal: a male parent must be alive, inside the configured age interval, and present in the same cell as the female parent. Among eligible males, one is selected uniformly. M2 does **not** model marriage, pair bonds, social paternity, mate preference, incest avoidance, kin exogamy, polygyny, or reproductive status beyond the stated rules.

### Stochasticity

Randomness represents unresolved variability in processes such as initialization and demographic/migration outcomes. All stochasticity comes from seeded named deterministic streams. M2 separates mortality, fertility, parentage and newborn-sex streams from world generation and from one another.

### Observation

The model emits aggregate run/population measurements and preserves authoritative persistent state. M2 records exact cumulative birth/death accounting in population state; a detailed chronological event stream is deferred to the later events/history milestone. Interpretive labels must not be confused with authoritative state or events.

## 5. Initialization

A run specifies:

- seed;
- synthetic world dimensions and generator schema;
- initial population count and spatial clustering;
- starting age/reproductive-sex distribution;
- household initialization rule;
- persistent person-record safety limit;
- demographic schedule/preset and provenance status;
- food/environment parameters when implemented;
- migration parameters when implemented;
- duration/stop condition.

No default parameter should be described as empirically realistic until a source and validation rationale are documented.

The current `synthetic_validation_v1` founder initialization samples ages from an explicitly configured synthetic range, assigns reproductive sex stochastically from a configured share, creates simple fixed-target-size co-resident households, and places households stochastically across synthetic world cells. Founder parentage and pre-run birth history are unknown/unset. This can create demographic transients and is not a scientifically neutral age structure.

A future schedule-consistent or empirical initialization method is required before demographic experiments claim correspondence to a real population.

## 6. Input data

v0.1 has no external anthropological or archaeological **runtime dataset**. The M1 environment is synthetic. M2 demographic mechanisms are informed by published comparative evidence, documented in `docs/research/demography-v0.1.md`, but the first executable demographic preset remains explicitly synthetic for engine validation.

Later empirical demographic presets may encode published age-specific schedules or derived tables. Such presets must retain source identity, transformation notes, units and provenance status.

## 7. Implemented and planned v0.1 submodels

### Synthetic environment (M1 — implemented)

The initial environment is generated from several smoothly varying deterministic fields derived from the run's `world` random stream. Water accessibility depends partly on synthetic wetness and lowland favourability; productivity combines water, a separate fertility field, and lowland favourability; movement cost combines ruggedness and relative elevation. Seasonality uses a synthetic latitude gradient plus a spatial climate field.

These relationships exist to provide heterogeneous, causally inspectable test conditions for later systems. They are **not yet evidence-grounded ecological equations**.

### Demography (M2 — implemented baseline)

M2 uses **replaceable age-specific schedules rather than hidden hard-coded anthropological constants**. Age is derived from birth time. Mortality and fertility are stochastic under separate named deterministic random streams, and exact births/deaths are reconciled against persistent population totals.

The demographic research baseline is `docs/research/demography-v0.1.md`. It establishes that extant hunter-gatherer populations show substantial demographic diversity and should be used as comparative evidence/ranges rather than as one universal prehistoric proxy.

The first executable preset is therefore named `synthetic_validation_v1`, not `hunter_gatherer`. Its qualitative mortality/fertility shapes are evidence-informed, but the complete schedule is not calibrated to a real population and carries provenance status `synthetic_validation`.

Mortality is represented by a transparent piecewise age-specific **annual event probability** schedule. Fertility is represented by an age-specific **annual live-birth opportunity probability** plus an explicit minimum birth-spacing rule. Probabilities are stored as integer parts per million. Completed family size, survivorship and life expectancy are outputs/validation quantities rather than values directly forced on individuals.

The M2 baseline deliberately does not couple nutrition, environment, disease, workload or social institutions into demographic hazards. Those links must be introduced explicitly in later modules rather than hidden inside the age schedules.

### Food and resource renewal (planned M3)

Cells expose renewable food stock based on baseline productivity and seasonality. Households gather locally and consume shared resources. Scarcity reduces condition and changes mortality/migration pressure only through explicitly documented coupling rules.

### Households (minimal M2 baseline; deeper rules deferred)

Households currently provide co-residence/location consistency and a future resource-sharing container. Formation/dissolution rules remain minimal until evidence-grounded social modules are introduced. Household membership is not equivalent to parentage or marriage.

### Migration (planned M4)

Households/individuals compare staying with a bounded set of locally plausible alternatives. Movement incurs distance/environment costs and uncertainty. No destination is selected because of its real-world historical importance.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting: `initial + births - deaths = living`;
- persistent record accounting: `initial + births = person records`;
- valid stable IDs and genealogy references after parent death;
- no death before birth;
- no self-parent, duplicate-parent, wrong-reproductive-sex parent or non-older parent relationships;
- valid household membership/location relationships;
- condition values within declared bounds;
- occupancy index reconciliation with authoritative locations;
- deterministic replay under the supported platform/build boundary;
- deterministic same-seed world generation and stable cell adjacency;
- isolation of named RNG streams so demographic changes cannot alter world generation;
- explicit non-scientific stop reason when the configured persistent-record safety ceiling is reached;
- directional tests for later coupled systems (for example, severe sustained resource scarcity must not systematically improve health).

## 9. Validation plan

The demographic literature baseline documents comparative ranges and limitations, but the implemented M2 baseline remains **unvalidated**. Passing software verification is not equivalent to empirical validation.

A research-capable demographic preset will require:

- population-specific or explicitly sampled parameterizations where appropriate;
- comparison of simulated survivorship, age-specific mortality, fertility, birth spacing and growth against claimed calibration targets;
- explicit examination of annual-boundary approximation effects;
- explicit examination of initialization transients;
- calibration only where justified by a stated research question;
- global and local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. First candidate experiment

A reasonable first research-style exercise after the remaining v0.1 mechanisms are implemented is:

> How does the magnitude and temporal variability of local resource productivity affect population persistence, spatial fragmentation, and migration distance in the simplified v0.1 model?

This would be an engine/model validation exercise rather than a claim about actual prehistory.
