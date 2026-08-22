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

Condition is an integer permille state from 0 to 1000. In M3 it is a synthetic energetic/health mediator: adequate household resource supply can recover condition and unmet need can reduce it. Condition is not currently a calibrated physiological measurement, body-mass index, nutritional score, or clinical health measure.

### Households

A household is a co-resident resource-sharing unit. It is not assumed to be a tribe, clan, lineage, settlement, marriage, or universal nuclear-family structure. Parentage and household membership are separate relationships.

In M2/M3, a newborn is assigned to the female parent's current household and cell. This is a transparent synthetic co-residence rule required to place a newborn into valid state; it is not an empirical claim about universal residence systems.

In M3, resource acquisition is accounted at the household level after same-cell competition. All living members of a household receive the same resource-satisfaction fraction for that period. There is no within-household age, status, sex, bargaining, preferential feeding, storage, theft, exchange, or food-waste model yet.

### World cells

Cells contain movement cost, water accessibility, baseline food productivity, an M1 initial food-stock field, seasonality, temporary environmental stress, and occupancy/density information.

In M1, these values form a **synthetic engine-validation landscape**. Environmental ratios are dimensionless fixed-point permille values. Relative elevation, water access, productivity, movement cost, and seasonality are spatially autocorrelated synthetic fields; they do not correspond to measured palaeoenvironmental units and must not be interpreted as empirical geography.

M3 keeps the M1 `World` immutable and creates a separate dynamic renewable-resource stock per cell. Resource quantities are abstract integer units. They are deliberately not labelled calories, kilograms, biomass, or any empirical energetic unit.

### Time

Authoritative simulation time is represented in integer days. M3 resource/condition processing occurs at explicit subannual boundaries: the default synthetic resource configuration uses four periods per year. M2 demographic hazards remain evaluated at explicit annual boundaries after the resource periods for that year. These are temporal approximations, not claims that real gathering, physiological change, births, or deaths occur synchronously at those intervals.

### Space

v0.1 uses a synthetic rectangular grid with deterministic north/east/south/west adjacency and hard boundaries. Real Earth and palaeoenvironmental reconstruction are explicitly deferred.

## 3. Process overview and scheduling

For each M3 simulated year, the implemented order is:

1. divide the year into the configured number of resource periods (four by default);
2. at each resource boundary, regenerate renewable cell stock from baseline productivity, the configured productivity scale, synthetic seasonality and environmental stress, subject to a cell stock capacity;
3. count living household members and calculate household resource need for the period;
4. aggregate household demand by cell;
5. allocate available cell stock among co-located households in proportion to household need, with bounded integer remainder resolved in stable household-ID order;
6. treat household harvest as immediate household consumption in the M3 baseline;
7. update each living member's condition from the household supply fraction;
8. apply an additional scarcity-mortality draw whose probability increases with condition deficit;
9. stop immediately if the population becomes extinct;
10. after the final resource period of the year, evaluate the existing M2 annual demographic boundary: baseline mortality first, then fertility/births among survivors;
11. stop explicitly if extinction or the operational persistent-record limit occurs;
12. at run completion, validate population/resource accounting and emit manifest summaries.

For the implemented M2 annual demographic boundary specifically:

1. evaluate baseline mortality for living person records that existed at the start of the boundary;
2. if nobody remains alive, stop with `population_extinct`;
3. evaluate fertility among surviving female records that existed at the start of the boundary;
4. apply minimum birth-spacing eligibility;
5. require at least one eligible living male in the female's current cell;
6. perform the configured fertility draw;
7. select one eligible local male uniformly by deterministic reservoir sampling;
8. create the newborn with stable parent IDs, the female parent's location/household, and a deterministic reproductive-sex draw;
9. rebuild the spatial occupancy index after births;
10. stop explicitly if the operational persistent-record limit is reached.

Newborns are not exposed to baseline annual demographic mortality until a later annual boundary. They participate in later resource periods once they exist. Scheduling is part of the model definition and must be treated as scientifically consequential in later sensitivity/validation work.

## 4. Design concepts

### Emergence

Population clusters, fragmentation, and migration corridors should emerge from local conditions. No rule may directly instruct a population to form a civilisation, tribe, village, or historical migration route.

M3 now allows local density and environmental productivity to create different condition/survival outcomes without scripting which population or cell should succeed.

### Adaptation / decision-making

Migration uses an interpretable local utility or probability model. Candidate destinations are knowledge-limited rather than globally optimal. Migration is still deferred to M4, so M3 households currently cannot leave a poor cell.

A starting conceptual form for M4 is:

```text
move utility = expected resources
             + water/security benefit
             + kin proximity
             - travel cost
             - uncertainty
             - relocation risk
```

Weights will be explicit experimental parameters, not hidden tuning constants.

### Interaction

v0.1 interaction is primarily household resource sharing, reproduction context, local density pressure, and spatial competition for resources. Global all-to-all interaction is prohibited.

M3 cell competition is local: household demand is aggregated only among households occupying the same cell. The hot path uses contiguous arrays indexed by cell/household/person rather than pairwise person searches or a global household-interaction graph.

M2 parent selection is intentionally minimal: a male parent must be alive, inside the configured age interval, and present in the same cell as the female parent. Among eligible males, one is selected uniformly. M2/M3 do **not** model marriage, pair bonds, social paternity, mate preference, incest avoidance, kin exogamy, polygyny, or reproductive status beyond the stated rules.

### Stochasticity

Randomness represents unresolved variability in processes such as initialization and mortality/reproduction outcomes. All stochasticity comes from seeded named deterministic streams. M2 separates baseline mortality, fertility, parentage and newborn-sex streams from world generation and from one another. M3 adds an independent `resources/scarcity_mortality` stream so scarcity-death draws do not consume demographic or world RNG state.

Resource regeneration, household demand, allocation, condition changes, and integer accounting are deterministic conditional on state/configuration; the current M3 scarcity-mortality coupling is stochastic.

### Observation

The run manifest records aggregate world, population and resource measurements while preserving authoritative persistent state. M3 resource summaries include initial/final stock, cumulative regeneration, harvest/consumption, unmet need, household-periods with unmet need, scarcity deaths, final mean living condition, count below half condition, and a deterministic resource-state digest.

Detailed chronological event streams remain deferred to the later events/history milestone. Interpretive labels must not be confused with authoritative state or events.

## 5. Initialization

A run specifies:

- seed;
- synthetic world dimensions and generator schema;
- initial population count and spatial clustering;
- starting age/reproductive-sex distribution;
- household initialization rule;
- persistent person-record safety limit;
- demographic schedule/preset and provenance status;
- resource model/preset, productivity scale, need and scarcity-response parameters;
- migration parameters when implemented;
- duration/stop condition.

No default parameter should be described as empirically realistic until a source and validation rationale are documented.

The current `synthetic_validation_v1` founder initialization samples ages from an explicitly configured synthetic range, assigns reproductive sex stochastically from a configured share, creates simple fixed-target-size co-resident households, and places households stochastically across synthetic world cells. Founder parentage and pre-run birth history are unknown/unset. This can create demographic transients and is not a scientifically neutral age structure.

M3 initializes dynamic resource stock from each synthetic world's initial food-stock/productivity fields, the resource productivity scale, and the configured stock-capacity rule. This is an engine-validation initial condition, not reconstructed environmental carrying capacity.

A future schedule-consistent or empirical initialization method is required before demographic/resource experiments claim correspondence to a real population.

## 6. Input data

v0.1 has no external anthropological, archaeological, energetic, or palaeoecological **runtime dataset**. The M1 environment and M3 resource quantities are synthetic. M2 demographic mechanisms are informed by published comparative evidence, documented in `docs/research/demography-v0.1.md`, but the first executable demographic preset remains explicitly synthetic for engine validation.

The M3 executable resource preset is also named `synthetic_validation_v1` and carries provenance `synthetic_validation`. Current need, regeneration, stock-capacity, condition and scarcity-mortality values are modelling placeholders selected to exercise causal machinery and directional tests. They are not estimates of hunter-gatherer caloric demand, foraging return rates, biomass productivity, starvation physiology, or Palaeolithic carrying capacity.

Later empirical presets must retain source identity, transformations, units, uncertainty and provenance status.

## 7. Implemented and planned v0.1 submodels

### Synthetic environment (M1 — implemented)

The initial environment is generated from several smoothly varying deterministic fields derived from the run's `world` random stream. Water accessibility depends partly on synthetic wetness and lowland favourability; productivity combines water, a separate fertility field, and lowland favourability; movement cost combines ruggedness and relative elevation. Seasonality uses a synthetic latitude gradient plus a spatial climate field.

These relationships exist to provide heterogeneous, causally inspectable test conditions for later systems. They are **not evidence-grounded ecological equations**.

### Demography (M2 — implemented synthetic baseline)

M2 uses **replaceable age-specific schedules rather than hidden hard-coded anthropological constants**. Age is derived from birth time. Mortality and fertility are stochastic under separate named deterministic random streams, and exact births/deaths are reconciled against persistent population totals.

The demographic research baseline is `docs/research/demography-v0.1.md`. It establishes that extant hunter-gatherer populations show substantial demographic diversity and should be used as comparative evidence/ranges rather than as one universal prehistoric proxy.

The first executable preset is therefore named `synthetic_validation_v1`, not `hunter_gatherer`. Its qualitative mortality/fertility shapes are evidence-informed, but the complete schedule is not calibrated to a real population and carries provenance status `synthetic_validation`.

Mortality is represented by a transparent piecewise age-specific **annual event probability** schedule. Fertility is represented by an age-specific **annual live-birth opportunity probability** plus an explicit minimum birth-spacing rule. Probabilities are stored as integer parts per million. Completed family size, survivorship and life expectancy are outputs/validation quantities rather than values directly forced on individuals.

M3 adds scarcity mortality separately rather than silently changing those baseline demographic schedules. There is still no direct food-to-fertility multiplier; introducing such a relationship would require an explicit hypothesis and evidence rather than being smuggled into the M2 fertility schedule.

### Food, resource renewal, condition and scarcity survival (M3 — implemented synthetic baseline)

M3 maintains one dynamic integer food-stock value per world cell. At each configured resource period:

```text
annual baseline regeneration
    = cell baseline productivity
    × configured regeneration scale
    × experiment productivity scale

period regeneration
    = annual baseline regeneration
    × synthetic seasonal factor
    × (1 - environmental stress)
    ÷ periods per year
```

All arithmetic is integer/fixed-point. Regeneration cannot raise a cell above its configured stock capacity.

Living-person resource need is aggregated into households and then cells. If total cell supply is insufficient, co-located households receive a proportional share according to household need. Within a household, that period's supply fraction is shared equally among living members for the purpose of condition change. Harvest is treated as consumption immediately; explicit household food storage, spoilage, waste and exchange are deferred.

Full supply allows bounded condition recovery. A supply deficit causes condition loss proportional to the missing fraction. Scarcity mortality is a separate probability proportional to the individual's condition deficit, capped by the configured maximum. These functions are transparent **synthetic response rules**, not calibrated human physiology.

The key causal chain now available is:

```text
cell productivity / seasonality / stress
             ↓
renewable resource stock
             ↓
local household competition
             ↓
household supply fraction
             ↓
individual condition
             ↓
additional scarcity mortality
             ↓
population persistence
```

M3 deliberately does not let households migrate away from scarcity. That response is M4.

### Households (minimal M2/M3 baseline; deeper rules deferred)

Households provide co-residence/location consistency and the M3 resource-sharing container. Formation/dissolution rules remain minimal until evidence-grounded social modules are introduced. Household membership is not equivalent to parentage or marriage.

### Migration (planned M4)

Households/individuals compare staying with a bounded set of locally plausible alternatives. Movement incurs distance/environment costs and uncertainty. No destination is selected because of its real-world historical importance.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting: `initial + births - deaths = living`;
- persistent record accounting: `initial + births = person records`;
- exact cumulative resource accounting: `initial stock + regeneration - harvest = final stock`;
- resource need accounting: period demand must reconcile to harvest plus unmet need;
- valid stable IDs and genealogy references after parent death;
- no death before birth;
- no self-parent, duplicate-parent, wrong-reproductive-sex parent or non-older parent relationships;
- valid household membership/location relationships;
- condition values within declared bounds;
- occupancy index reconciliation with authoritative locations;
- deterministic replay under the supported platform/build boundary;
- deterministic same-seed world/resource/population evolution;
- isolation of named RNG streams so resource changes cannot alter world generation;
- explicit non-scientific stop reason when the configured persistent-record safety ceiling is reached;
- directional scarcity test: severe sustained zero-resource conditions must not improve condition/survival;
- directional productivity test: under otherwise identical synthetic assumptions designed to isolate the resource mechanism, a resource-rich environment must support at least as much condition/survival as a zero-productivity environment.

Passing these targets verifies implementation properties. It does not validate the resource model against reality.

## 9. Validation plan

The implemented M2 and M3 baselines remain **unvalidated**. Passing software verification is not equivalent to empirical validation.

A research-capable demographic/resource configuration will require:

- population-specific or explicitly sampled demographic parameterizations where appropriate;
- defensible energetic/ecological units and source provenance for any claimed resource interpretation;
- evidence-grounded or explicitly hypothetical relationships between resource availability, condition, fertility and mortality;
- comparison of simulated survivorship, age-specific mortality, fertility, birth spacing and growth against claimed calibration targets;
- comparison of resource/condition responses against independent empirical or archaeological patterns appropriate to the research question;
- explicit examination of annual demographic and subannual resource scheduling effects;
- explicit examination of initialization transients and initial resource stock;
- calibration only where justified by a stated research question;
- global and local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. First candidate experiment

A reasonable first research-style exercise after migration is implemented is:

> How does the magnitude and temporal variability of local resource productivity affect population persistence, spatial fragmentation, and migration distance in the simplified v0.1 model?

M3 now provides the resource-to-condition-to-survival half of that causal experiment. M4 will add behavioural movement responses. Until empirical parameterization/validation exists, this remains an engine/model validation exercise rather than a claim about actual prehistory.
