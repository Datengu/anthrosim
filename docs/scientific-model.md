# Scientific model specification (ODD-oriented)

**Status:** v0.1 working model specification  
**Scientific status:** exploratory / unvalidated

This document describes the scientific meaning of AnthroSim v0.1 separately from its software architecture. It follows the spirit of the ODD (Overview, Design concepts, Details) protocol used for agent-based models.

The implementation is research-oriented, but the current executable demographic, resource and migration presets are still synthetic validation baselines. Verification of software behaviour is not empirical validation of human prehistory.

## 1. Purpose

v0.1 asks a deliberately narrow methodological question:

> Can a spatially explicit population of persistent individuals and households produce interpretable patterns of demographic growth, decline, fragmentation and migration when local resource conditions vary, without scripted destinations or higher-level social institutions?

v0.1 is not intended to reconstruct a particular prehistoric population or produce quantitative claims about real Homo sapiens.

The implemented M1–M4 causal loop is:

```text
synthetic environment
        ↓
renewable local resources
        ↓
local household competition / sharing
        ↓
individual condition and scarcity mortality
        ↓
local relocation pressure
        ↓
bounded household migration
        ↓
new local resource context
```

Baseline age-specific mortality, fertility and genealogy continue through the M2 annual demographic process.

## 2. Entities, state variables, and scales

### People

Persistent individuals have stable identity, epoch-relative birth time/derived age, reproductive sex for the initial reproduction model, condition, location, household membership, parent references, birth-history state and death state. Dead individuals remain persistent records so genealogy references do not change when a person dies.

Reproductive sex is a deliberately limited biological state variable for the v0.1 birth mechanism; it is not a model of social gender.

Condition is an integer permille state from 0 to 1000. It is a synthetic energetic/health mediator: adequate household resource supply can recover condition, unmet need can reduce it, and completed migration can impose a distance-dependent condition cost. Condition is not BMI, body-fat percentage, a nutritional biomarker or a clinical health score.

### Households

A household is a co-resident resource-sharing and M4 relocation unit. It is not assumed to be a tribe, clan, lineage, settlement, marriage or universal nuclear-family structure. Parentage and household membership are separate relationships.

A newborn is assigned to the female parent's current household and cell. This is a transparent synthetic co-residence rule required to place a newborn into valid state; it is not an empirical claim about universal residence systems.

Resource acquisition is accounted at household level after same-cell competition. All living members of a household receive the same resource-satisfaction fraction for that period. There is no within-household age, status, sex, bargaining, preferential feeding, storage, theft, exchange or food-waste model yet.

In M4, all currently living members of a selected household relocate together. People who died before a later household move retain their location at death rather than being retroactively moved with the current household. Household-coordinated movement is an explicit v0.1 modelling choice, not a claim that all real mobility decisions occur at that social scale.

### World cells

Cells contain movement cost, water accessibility, baseline food productivity, an M1 initial food-stock field, seasonality, environmental stress and occupancy/density context.

These values form a **synthetic engine-validation landscape**. Environmental ratios are dimensionless fixed-point permille values. Relative elevation, water access, productivity, movement cost and seasonality are spatially autocorrelated synthetic fields; they do not correspond to measured palaeoenvironmental units and must not be interpreted as empirical geography.

M3 keeps the M1 `World` immutable and creates a separate dynamic renewable-resource stock per cell. Resource quantities are abstract integer units. They are deliberately not labelled calories, kilograms, biomass or any other empirical energetic unit.

M4 reads dynamic local resource stock plus existing cell water/stress/movement fields when evaluating candidate destinations. It does not create a hidden geographic attractiveness field or label any cell as historically important.

### Time

Authoritative simulation time is represented in integer days.

Resource/condition processing occurs at explicit subannual boundaries; the default synthetic resource configuration uses four periods per year. Migration is evaluated immediately after each resource boundary when the population survives that period. Selected household moves complete atomically at the same boundary and impose an explicit travel-condition cost. The M2 baseline demographic process remains evaluated at annual boundaries after the final resource/migration period of the year.

These are temporal approximations, not claims that real gathering, physiological change, travel, births or deaths occur synchronously at those intervals.

### Space

v0.1 uses a synthetic rectangular grid with hard boundaries. Four-neighbour topology supports world structure, while M4 destination discovery uses bounded Manhattan-distance neighbourhoods. Real Earth and palaeoenvironmental reconstruction are explicitly deferred.

## 3. Process overview and scheduling

For each M4 simulated year, the implemented order is:

1. divide the year into the configured number of resource periods (four by default);
2. at each resource boundary, regenerate renewable cell stock from baseline productivity, configured productivity scale, synthetic seasonality and environmental stress, subject to cell stock capacity;
3. count living household members and calculate household resource need for the period;
4. aggregate household demand by cell;
5. allocate available stock among co-located households in proportion to household need, with bounded integer remainder resolved in stable household-ID order;
6. treat household harvest as immediate household consumption in the current baseline;
7. update each living member's condition from the household supply fraction;
8. apply an additional scarcity-mortality draw whose probability increases with condition deficit;
9. stop immediately if the population becomes extinct;
10. derive one shared pre-move household/cell snapshot for the surviving population;
11. calculate relocation pressure for living households from condition and local resource support;
12. pressured households compare staying with only the cells inside the configured local Manhattan radius;
13. decompose each candidate's utility into explicit resource, water/security, kin, travel, uncertainty and relocation-risk factors;
14. discard candidates that do not improve sufficiently over staying;
15. select among eligible alternatives with deterministic weighted stochastic choice;
16. retain all chosen destinations as plans so later household decisions at that boundary do not observe earlier planned moves;
17. apply selected household relocations simultaneously in one packed population pass, deduct distance-dependent travel condition cost and rebuild occupancy once;
18. after the final resource/migration period of the year, evaluate the M2 annual demographic boundary;
19. stop explicitly if extinction or the operational persistent-record limit occurs;
20. at run completion, validate authoritative invariants and emit world, population, resource and migration summaries.

For the M2 annual demographic boundary specifically:

1. evaluate baseline mortality for living person records that existed at the start of the boundary;
2. if nobody remains alive, stop with `population_extinct`;
3. evaluate fertility among surviving female records that existed at the start of the boundary;
4. apply minimum birth-spacing eligibility;
5. require at least one eligible living male in the female's current cell;
6. perform the configured fertility draw;
7. select one eligible local male uniformly by deterministic reservoir sampling;
8. create the newborn with stable parent IDs, the female parent's current location/household and a deterministic reproductive-sex draw;
9. rebuild the spatial occupancy index after births;
10. stop explicitly if the operational persistent-record limit is reached.

Newborns are not exposed to baseline annual mortality until a later annual boundary. They participate in later resource and migration periods once they exist. Scheduling is part of the model definition and must be treated as scientifically consequential in later sensitivity and validation work.

## 4. Design concepts

### Emergence

Population clustering, concentration, fragmentation and repeated movement should emerge from local conditions. No rule may directly instruct a population to form a civilisation, tribe, village or historical migration route.

M3 allows local density and environmental productivity to create different condition/survival outcomes without scripting which population or cell should succeed. M4 adds a behavioural response: pressured households can relocate toward locally acceptable alternatives without any rule naming a real-world destination, route, settlement or population outcome.

### Adaptation / decision-making

M4 uses an interpretable bounded local utility model. A household only evaluates cells within a configured Manhattan radius (three cells by default), so destination discovery does not become a global best-cell search as the world grows. Remaining at the origin is represented by the origin utility against which candidate improvement is compared.

The implemented synthetic candidate utility is conceptually:

```text
candidate utility
  = local dynamic-resource score × resource weight
  + water/security proxy × water weight
  + bounded direct-parent-location score × kin weight
  - distance/terrain travel penalty × travel weight
  - deterministic uncertainty penalty
  - relocation-risk penalty
```

A candidate must exceed the configured minimum improvement over staying. Eligible alternatives receive weights proportional to utility improvement, and one is selected through the named deterministic `migration/choice` random stream.

This is not deterministic optimization: several locally acceptable destinations can compete. It is also not a cognitive model of deliberation. The current factors, functional form, thresholds and weights are transparent synthetic assumptions.

### Relocation pressure

Relocation pressure is derived from two explicit deficits:

- mean living household condition below a configured condition threshold;
- local resource-support score below a configured resource threshold.

The deficits add and are bounded to 0..1000 permille. Thus, under otherwise equal inputs, worsening condition or local resource support cannot reduce the implemented pressure value.

Pressure is a model trigger, not a calibrated psychological or ethnographic measurement of desire to migrate.

### Knowledge and uncertainty

Households do not know the whole world. Candidate discovery is bounded by the configured local radius. For an interior cell, a Manhattan radius `r` contains at most:

```text
2r(r + 1)
```

move candidates after excluding the origin. At the default radius three, that is 24 cells regardless of total world area. Edge and corner households see fewer candidates.

The model currently treats the candidate's dynamic resource stock and cell environmental fields as locally inspectable proxies. It does not yet model landscape memory, learned routes, scouts, hearsay, map-like knowledge, seasonal movement traditions or culturally transmitted geographic knowledge.

Candidate uncertainty is represented by an independent deterministic stochastic penalty. That term is a mechanism-testing proxy, not a fitted measure of human uncertainty.

### Kin proximity

M4 uses only genealogical state that already exists in the model. For each household, it can retain a small bounded set of cells containing living direct parents of living household members when those parents reside outside the household. A candidate receives a limited kin contribution when it matches one of those cells.

This is deliberately narrow. It is not a model of clans, lineages, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial groups or culturally defined obligations.

### Travel and relocation risk

Movement is not free. Candidate utility includes a travel penalty from Manhattan distance and destination movement-cost excess, plus a relocation-risk penalty. After a move is selected, each living mover loses condition according to travelled distance and the configured travel-condition cost.

The current relocation-risk term affects decision utility rather than creating a separate injury/death event. The journey itself is atomic rather than persistent: there is no en-route state, journey duration, camp sequence, route choice or movement mortality process yet.

These are explicit M4 simplifications and are not calibrated travel energetics or mortality estimates.

### Interaction

v0.1 interaction is primarily household resource sharing, reproduction context, local density pressure, same-cell resource competition and bounded migration response. Global all-to-all interaction is prohibited.

Resource demand is aggregated only among households occupying the same cell. M4 candidate work is proportional to pressured households × bounded local candidate count, not households × all world cells. The hot path uses contiguous arrays indexed by cell/household/person plus reusable candidate buffers rather than pairwise person searches or a global household-interaction graph.

M2 parent selection is intentionally minimal: a male parent must be alive, inside the configured age interval and present in the same cell as the female parent. Among eligible males, one is selected uniformly. v0.1 does **not** model marriage, pair bonds, social paternity, mate preference, incest avoidance, kin exogamy, polygyny or reproductive status beyond the stated rules.

### Simultaneous movement

All household decisions at one boundary use the same pre-move snapshot. Planned moves are applied only after candidate evaluation has finished for every household.

This prevents household-ID order from changing the information seen by later households. It also means multiple households can independently choose the same destination without anticipating one another's simultaneous arrival. Resulting crowding is experienced during subsequent resource periods rather than being solved by a hidden global coordinator.

That simultaneous-arrival assumption is scientifically consequential and should be included in later sensitivity work.

### Stochasticity

All stochasticity comes from seeded named deterministic streams.

- M1: world-generation stream used to derive stable field seeds;
- M2: independent baseline mortality, fertility, parentage and newborn-sex streams;
- M3: independent `resources/scarcity_mortality` stream;
- M4: independent `migration/choice` and `migration/uncertainty` streams.

Resource regeneration, household demand/allocation, condition updates, candidate enumeration and integer utility calculations are deterministic conditional on state/configuration. Scarcity mortality, candidate uncertainty and weighted destination choice are stochastic but reproducible through their named streams.

### Observation

The run manifest records aggregate world, population, resource and migration measurements while preserving authoritative persistent state.

Resource summaries include initial/final stock, cumulative regeneration, harvest/consumption, unmet need, household-periods with unmet need, scarcity deaths, final mean living condition, count below half condition and a deterministic resource-state digest.

M4 migration summaries include:

- decision boundaries;
- households evaluated and households under pressure;
- completed household moves and people moved;
- total Manhattan distance;
- north/east/south/west step totals;
- realized travel-condition cost;
- mean origin/destination resource scores;
- mean origin/destination water/security scores;
- migration-attributable occupied-cell delta;
- deterministic migration digest;
- a bounded sample of detailed completed-move traces.

Each retained trace contains the household, origin/destination, distance, pressure, local candidate count, factor-by-factor origin and selected-destination utility, best visible candidate, stochastic choice weights/draw and travel condition cost. The default trace cap is 256 so ordinary manifests remain bounded even if many moves occur.

A trace explains why the implemented model selected a move. It is not evidence of a real person's motive.

Full chronological event streams remain deferred to M5.

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
- migration preset, local-information radius, pressure thresholds, utility weights, uncertainty/risk and travel-cost parameters;
- duration and stop conditions.

No default parameter should be described as empirically realistic until a source and validation rationale are documented.

The current `synthetic_validation_v1` founder initialization samples ages from an explicitly configured synthetic range, assigns reproductive sex stochastically from a configured share, creates simple fixed-target-size co-resident households and places households stochastically across synthetic world cells. Founder parentage and pre-run birth history are unknown/unset. This can create demographic transients and is not a scientifically neutral age structure.

M3 initializes dynamic resource stock from the synthetic world's initial food-stock/productivity fields, the resource productivity scale and stock-capacity rule. This is an engine-validation initial condition, not reconstructed environmental carrying capacity.

M4 initializes no historical route, migration corridor or destination knowledge. Households begin with current location/co-residence and only the bounded genealogical information already represented in persistent person state.

A future schedule-consistent or empirical initialization method is required before experiments claim correspondence to a real population.

## 6. Input data and provenance

v0.1 has no external anthropological, archaeological, energetic, mobility or palaeoecological **runtime dataset**. The M1 environment, M3 resource quantities and M4 migration parameters are synthetic.

M2 demographic mechanisms are informed by published comparative evidence documented in `docs/research/demography-v0.1.md`, but the first executable demographic preset remains explicitly synthetic for engine validation.

The M3 executable resource preset is `synthetic_validation_v1`, documented in `docs/research/resources-v0.1.md`. Current need, regeneration, stock-capacity, condition and scarcity-mortality values are modelling placeholders selected to exercise causal machinery and directional tests.

The M4 executable migration preset is also `synthetic_validation_v1`, documented in `docs/research/migration-v0.1.md`. Its information radius, pressure thresholds, utility weights, uncertainty, relocation-risk and travel-condition costs are mechanism-testing assumptions rather than estimates of observed mobility.

Later empirical presets must retain source identity, transformations, units, uncertainty and provenance status.

## 7. Implemented v0.1 submodels

### Synthetic environment (M1 — implemented)

The environment is generated from smoothly varying deterministic fields derived from the run's `world` random stream. Water accessibility depends partly on synthetic wetness and lowland favourability; productivity combines water, a separate fertility field and lowland favourability; movement cost combines ruggedness and relative elevation. Seasonality uses a synthetic latitude gradient plus a spatial climate field.

These relationships exist to provide heterogeneous, causally inspectable test conditions. They are **not evidence-grounded ecological equations**.

### Demography (M2 — implemented synthetic baseline)

M2 uses replaceable age-specific schedules rather than hidden hard-coded anthropological constants. Age is derived from birth time. Mortality and fertility are stochastic under separate named deterministic streams, and exact births/deaths reconcile against persistent population totals.

The demographic research baseline is `docs/research/demography-v0.1.md`. It establishes that extant hunter-gatherer populations show substantial demographic diversity and should be used as comparative evidence/ranges rather than as one universal prehistoric proxy.

The first executable preset is therefore named `synthetic_validation_v1`, not `hunter_gatherer`. Its qualitative mortality/fertility shapes are evidence-informed, but the complete schedule is not calibrated to a real population and carries provenance status `synthetic_validation`.

Mortality is represented by a transparent piecewise age-specific annual event-probability schedule. Fertility is represented by an age-specific annual live-birth opportunity probability plus explicit minimum birth spacing. Probabilities are integer parts per million. Completed family size, survivorship and life expectancy are outputs/validation quantities rather than values directly forced on individuals.

M3 scarcity mortality remains separate rather than silently changing baseline demographic schedules. There is no direct food-to-fertility multiplier; introducing one requires an explicit hypothesis/evidence basis.

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

All arithmetic is integer/fixed-point. Regeneration cannot raise a cell above configured stock capacity.

Living-person resource need is aggregated into households and cells. If total cell supply is insufficient, co-located households receive a proportional share according to household need. Within a household, the period supply fraction is shared equally among living members for condition change. Harvest is treated as immediate consumption; explicit storage, spoilage, waste and exchange are deferred.

Full supply allows bounded condition recovery. Deficit causes condition loss proportional to missing supply. Scarcity mortality is a separate probability proportional to condition deficit, capped by the configured maximum. These are transparent synthetic response rules, not calibrated human physiology.

### Households (minimal M2–M4 baseline)

Households provide co-residence/location consistency, resource sharing and the M4 relocation unit. Formation/dissolution rules remain minimal. Household membership is not equivalent to parentage or marriage.

### Migration (M4 — implemented synthetic baseline)

Surviving households whose condition and/or local resource support create positive relocation pressure compare staying with cells inside a bounded Manhattan information radius.

Candidate utility uses:

- current dynamic resource stock relative to local demand after adding the moving household;
- a synthetic water/security score using water accessibility and inverse environmental stress;
- a deliberately narrow bounded living-direct-parent location proxy;
- distance and terrain movement cost;
- deterministic stochastic uncertainty;
- relocation-risk penalty.

Candidate evaluation is not global optimization. Alternatives must improve sufficiently over staying, then compete through weighted deterministic stochastic choice.

All households decide from one pre-move snapshot. Selected households then relocate simultaneously. Living members move together, their condition pays the travel cost, current household location changes, and occupancy is rebuilt. Dead records keep location at death.

There is no hard-coded historical destination or route, persistent en-route state, route memory, seasonal mobility tradition, clan/tribe institution or claim that synthetic weights reproduce real mobility behaviour.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting: `initial + births - deaths = living`;
- persistent record accounting: `initial + births = person records`;
- exact cumulative resource accounting: `initial stock + regeneration - harvest = final stock`;
- resource need accounting: demand must reconcile to consumption plus unmet need;
- valid stable IDs and genealogy references after parent death;
- no death before birth;
- no self-parent, duplicate-parent, wrong-reproductive-sex parent or non-older parent relationships;
- valid current household membership/location relationships for living people;
- condition values within declared bounds;
- occupancy index reconciliation with authoritative locations;
- deterministic replay under the supported platform/build boundary;
- isolation of named RNG streams;
- explicit non-scientific stop reason when the persistent-record safety ceiling is reached;
- directional scarcity test: severe sustained zero-resource conditions must not improve condition/survival;
- directional productivity test: an otherwise-equal positive-resource case designed to isolate M3 must support at least as much condition/survival as a zero-productivity case;
- bounded migration-candidate discovery independent of total-world search;
- deterministic migration replay including migration digest and retained decision traces;
- directional migration-pressure test: worsening local resource/condition inputs must not reduce relocation pressure under otherwise equal inputs;
- completed migration traces must remain within the configured local radius and expose origin/destination utility factors;
- migration-enabled and migration-disabled otherwise-equal tests can diverge spatially through the implemented movement mechanism;
- household relocation and occupancy invariants remain valid after simultaneous moves;
- migration candidate lookup and the full v0.1 target workload remain benchmarked in CI.

Passing these targets verifies implementation properties. It does not validate demographic, resource or migration assumptions against reality.

## 9. Validation plan

The implemented M2, M3 and M4 baselines remain **unvalidated**. Passing software verification is not equivalent to empirical validation.

A research-capable demographic/resource/migration configuration will require, as appropriate to the bounded research question:

- population-specific or explicitly sampled demographic parameterizations;
- defensible energetic/ecological units and source provenance for claimed resource interpretations;
- evidence-grounded or explicitly hypothetical relationships between resources, condition, fertility and mortality;
- comparison of survivorship, age-specific mortality, fertility, birth spacing and growth against declared calibration/validation targets;
- independent empirical or archaeological patterns appropriate to resource/condition claims;
- evidence on mobility scale, settlement duration/relocation frequency and information horizon where movement is part of the claim;
- explicit treatment of analogy limits when ethnographic mobility evidence is used;
- defensible travel/terrain costs and social/kin assumptions;
- sensitivity to simultaneous-arrival crowding and the atomic-travel approximation;
- explicit examination of annual/subannual scheduling effects;
- explicit examination of initialization transients and initial resource stock;
- calibration only where justified by a stated research question;
- global/local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. First candidate experiment

Now that the complete M1–M4 environmental-response loop exists, a reasonable first research-style model exercise is:

> How does the magnitude and temporal variability of local resource productivity affect population persistence, spatial fragmentation and migration distance in the simplified v0.1 model?

M3 supplies the resource-to-condition-to-survival pathway and M4 adds bounded behavioural movement. Factorial comparisons can also turn migration on/off or vary information radius to distinguish survival effects caused by resources from those caused by mobility response.

Until empirical parameterization and validation exist, such experiments remain engine/model validation exercises rather than claims about actual prehistory.
