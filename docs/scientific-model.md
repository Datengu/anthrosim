# Scientific model specification (ODD-oriented)

**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v10
**Scientific status:** exploratory / unvalidated

This document began as the v0.1 ODD-oriented model specification and records the scientific meaning of the implemented baseline plus subsequent post-M9 scientific-hardening semantics. Historical M1–M4 sections remain relevant to the synthetic demographic/resource/permanent-migration baseline; M8 adds evidence-grounded spatial binding, M9 adds a separate temporary-mobility layer, and the hardening line makes previously ambiguous demographic/resource/response timing and condition-mortality causal contracts explicit. Software verification and successful capability benchmarks are not empirical validation of human prehistory.

## 1. Purpose

The original v0.1 methodological question remains part of the model:

> Can a spatially explicit population of persistent individuals and households produce interpretable demographic and permanent-migration patterns from local resource conditions without scripted historical destinations or higher-level social institutions?

M8 extends that engine so externally derived spatial evidence can be normalized, provenance-bound and transformed through declared model assumptions. M9 adds a second mobility question:

> Can persistent residence and intermittent temporary physical presence be represented as different causal histories, with deterministic travel/resource accounting and observability, without treating temporary aggregation as permanent settlement?

M9 is a generic capability/null mechanism. It does not encode why real people gathered, whether a focal place was political/ritual/economic/defensive, or whether any archaeological site had a particular use. Current benchmark success means the implemented mechanisms are distinguishable under declared synthetic assumptions; it is not a historical reconstruction.

The implemented causal layers are therefore:

```text
baseline environment or declared M8 landscape
        ↓
renewable local resources + declared spatial transformations
        ↓
household competition / sharing and individual condition
        ↓
M4 bounded permanent migration (changes residence)
        ↓
M9 temporary journeys (changes physical presence, not residence)
        ↓
duration-aware demand + separate residence/presence observability
```

Baseline age-specific mortality, fertility and genealogy continue through the M2 annual demographic process and remain residence-based under M9.

## 2. Entities, state variables, and scales

### People

Persistent individuals have stable identity, epoch-relative birth time/derived age, reproductive sex for the initial reproduction model, condition, household membership, parent references, birth-history state and death state. Dead individuals remain persistent records so genealogy references do not change when a person dies.

A living person's **persistent residence** is distinct from M9 **physical presence**. When temporary mobility is disabled or the household is home, presence is at residence. With M9 enabled an away household can instead be in outbound transit, visiting a resolved focal-region destination, or in return transit. Transit deliberately has no authoritative per-day world cell. Only M4 permanent migration changes residence.

Reproductive sex remains a deliberately limited biological state variable for the current birth mechanism; it is not a model of social gender. Condition is a bounded synthetic 0..1000 health/energetic mediator rather than a clinical or directly empirical measure. It is intentionally more general than nutritional status: current authoritative pathways include founder/newborn condition, M3 resource-driven recovery or loss, and M4 permanent-travel condition cost. The model does not preserve an additive decomposition of condition by upstream cause.

### Households

A household is a persistent resource-sharing and mobility unit, not a tribe, clan, lineage, settlement, marriage or universal nuclear-family structure. Parentage and household membership are separate relationships.

Birth/fertility semantics remain residence-based under M9. A newborn joins the female parent's current household and inherits that household's persistent residence even if the household is temporarily away; temporary physical-presence bookkeeping is kept consistent separately. Parent eligibility likewise uses residence rather than visitor co-presence.

M4 permanent migration relocates the living household and changes residence. M9 temporary mobility can move the household through transit/visiting states while preserving residence. A death while away removes the person from current physical-presence accounting, while demographic/spatial death attribution remains explicitly residence-based.

Resource sharing remains household-level. M9 changes **where/when** household demand is charged across elapsed person-days; it does not add within-household bargaining, storage, theft, exchange, status or preferential feeding.

### World cells

The baseline engine uses cells containing movement cost, water accessibility, food productivity/stock fields, seasonality, environmental stress and occupancy context. Synthetic runs generate those fields deterministically.

M8 additionally supports versioned normalized landscapes derived from external spatial inputs. Those source values remain distinct from the deterministic model-facing transformations used by the simulation; loading real-world-derived terrain therefore does not make the resource, mobility or demographic equations empirically validated. M9 focal regions bind to authoritative cell identities on either synthetic or transformed worlds.

Dynamic renewable-resource stock remains separate from immutable/model-facing world fields. Resource quantities are abstract integer units unless a future research configuration explicitly supplies defensible units and evidence.

### Time

Authoritative simulation time is represented in integer days. M2 baseline demography is an annual discrete transition.

For `P = resources.periodsPerYear`, M3 resource interval `i` is the exact half-open interval `[floor(i*365/P), floor((i+1)*365/P))` within a 365-day model year. A fixed annual integer quantity `Q` is allocated cumulatively as `floor(Q*t/365)` so period shares conserve the annual quantity exactly against actual elapsed model days. The synthetic seasonal curve is integrated over those same intervals and normalized by its complete-year weight so seasonal phase redistributes unconstrained annual regeneration potential rather than silently changing the annual baseline. Normative annual resource semantics are in [`research/m3-resource-time-contract-v1.md`](research/m3-resource-time-contract-v1.md).

M3 resource settlement, condition response and condition-mediated survival occur at the configured M3 interval ends. The v9 timing contract gives the historical condition recovery/loss coefficients and condition-dependent mortality probability an explicit **reference-quarter** interpretation against `[0,91)`, `[91,182)`, `[182,273)` and `[273,365)`. In v10 the public mortality parameter is `maxConditionMortalityProbabilityPerMillion`; the former scarcity-specific public name is not a v10 alias. Condition response is allocated cumulatively over actual elapsed M3 intervals, and fixed-condition survival is converted through exact integer-rational conditional survival. Thus changing only `P` does not multiply the complete-year response budget or fixed-condition mortality probability merely by adding more M3 boundaries.

M4 permanent relocation remains atomic at its decision boundary, but its opportunity clock is now independent of M3. For `D = migration.decisionPeriodsPerYear`, decision interval `j` uses `[floor(j*365/D), floor((j+1)*365/D))`; the synthetic default is `D = 4`. M4's resource-support term uses annual per-person need allocated over the current M4 decision interval using the same cumulative elapsed-day annual-allocation rule as M3. The runtime reconciles M4 decision index and actual decision day rather than assuming every resource boundary is a migration boundary.

M9 temporary journeys are explicitly duration-bearing: departure, arrival, visiting duration, return departure and completion occur on deterministic days, and journeys can remain active across an annual checkpoint. Within each year the authoritative hosts merge the independent M3 and M4 fixed schedules. At a shared M3/M4 day, elapsed M3 resource/condition/survival processing occurs first, then due M9 transitions/start processing, then M4 permanent migration. Either M3 or M4 may otherwise occur alone. M2 annual demography follows the year's subannual processing.

These schedules are model approximations and are scientifically consequential assumptions, not claims that real births, deaths, gathering, travel or physiological change occur synchronously. The v9 repair removes the specific hidden-rate artifact in which increasing M3 `periodsPerYear` automatically created more condition-response, condition-mortality or M4 opportunities. It does **not** make M3 resolution causally irrelevant: changing settlement timing can still alter stock, evolving condition, extinction timing, M9 demand attribution and state observed by later fixed M4 decisions. Normative response/decision timing is in [`research/m3-response-time-contract-v1.md`](research/m3-response-time-contract-v1.md).

The v10 causal repair changes what the low-condition hazard means, not its numerical v9 time-scaling rule. A death generated by that hazard serializes as `condition_mediated`. Resource shortage can causally raise that hazard by lowering condition, but M4 permanent-travel cost can lower the same condition scalar. Because the scalar does not retain source apportionment, the death event alone cannot establish that resource scarcity—or travel—was the unique upstream cause. Normative cause semantics are in [`research/m3-condition-mortality-contract-v1.md`](research/m3-condition-mortality-contract-v1.md).

### Space

The baseline world is a bounded rectangular grid. M4 permanent destination discovery uses a bounded Manhattan information radius rather than global optimization. M8 can replace the synthetic model-facing fields with deterministic transformations of a normalized provenance-tracked landscape while preserving the same stable cell identity contract.

M9 focal regions are identity-bearing bindings over declared cell sets. Temporary travel resolves deterministic cost/duration from household residence to the focal-region destination under the declared travel semantics. A journey's transit phase intentionally has no authoritative world cell, preventing false precision about unmodelled routes.

## 3. Process overview and scheduling

Within each model year, the main `Simulation` host and `SpatialLandscapeSimulation` use the same merged fixed-boundary contract:

1. identify the next due M3 resource boundary and M4 permanent-migration boundary;
2. process M9 temporary journey boundaries strictly before that fixed day;
3. if M3 is due, settle the elapsed resource interval using duration-aware residence/visitor/transit person-days, update condition, and apply the elapsed condition-mediated mortality probability;
4. process due M9 temporary transitions/start decisions for that day;
5. if M4 is due, evaluate permanent migration only for eligible households physically at residence, using the M4 decision interval's resource-support demand;
6. apply selected permanent moves simultaneously;
7. after the year's subannual schedules complete, run M2 annual demography.

Under the v8 annual resource-accounting contract, M3 uses exact scheduler intervals, cumulative elapsed-day allocation for fixed annual quantities, integrated/normalized seasonal regeneration and zero-demand condition neutrality. Under v9 timing semantics, condition recovery/loss and the condition-mediated mortality probability are converted from declared reference-quarter coefficients to the actual M3 interval. M4 receives its own opportunity count and its own current decision-interval demand share. It no longer maintains the former `ceil(annual/P)` approximation or inherits an opportunity from every M3 boundary. Under v10, the same numerical condition hazard has causal-neutral public semantics rather than a resource-scarcity-specific death cause.

Temporary lifecycle state progresses through departure, outbound transit, visiting, return departure, return transit and completion. Focal-region identity, residence, destination, timing and people affected remain stable across a journey's authoritative event history. Transit uses an explicit home-provisioning resource proxy rather than an invented route cell.

M2 mortality/fertility remains annual and residence-based. Births inherit household residence. If a person dies while the household is away, temporary-presence state is updated so visitor/transit counts remain correct; `Death.cell` remains a residence attribution field and must not be read as an observed physical death location.

Scheduling is part of the model definition and must be included in sensitivity/validation work. Detailed contracts are in [`research/m3-resource-time-contract-v1.md`](research/m3-resource-time-contract-v1.md), [`research/m3-response-time-contract-v1.md`](research/m3-response-time-contract-v1.md), [`research/m3-condition-mortality-contract-v1.md`](research/m3-condition-mortality-contract-v1.md), [`research/temporary-mobility-v1.md`](research/temporary-mobility-v1.md), [`research/m9-temporary-travel-semantics-v1.md`](research/m9-temporary-travel-semantics-v1.md), and [`research/m9-duration-aware-resource-semantics-v1.md`](research/m9-duration-aware-resource-semantics-v1.md).

## 4. Design concepts

### Emergence

Population clustering, concentration, fragmentation and repeated movement should emerge from local conditions. No rule may directly instruct a population to form a civilisation, tribe, village or historical migration route.

M3 allows local density and environmental productivity to create different condition/survival outcomes without scripting which population or cell should succeed. M4 adds a behavioural response: pressured households can relocate toward locally acceptable alternatives without any rule naming a real-world destination, route, settlement or population outcome.

### Adaptation / decision-making

M4 uses an interpretable bounded local utility model. A household only evaluates cells within a configured Manhattan radius (three cells by default), so destination discovery does not become a global best-cell search as the world grows. Remaining at the origin is represented by the origin utility against which candidate improvement is compared. Its dynamic-resource score compares stock to annual per-person need allocated over the current M4 decision interval.

M4 opportunity frequency is itself explicit model structure. `migration.decisionPeriodsPerYear` controls how many fixed opportunities occur in a complete year, independent of M3 resource partition. The synthetic validation default is four/year. This is not a calibrated estimate of how often historical households reconsidered permanent residence.

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

This is not deterministic optimization: several locally acceptable destinations can compete. It is also not a cognitive model of deliberation. The current factors, functional form, thresholds, weights and decision opportunity rate are transparent synthetic assumptions.

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

M4 permanent migration and M9 temporary mobility have different travel abstractions.

For M4, candidate utility includes distance/terrain travel penalty and relocation risk, and a selected permanent move completes atomically at that decision boundary with a condition cost. It has no persistent en-route state. Under v10 that travel cost is an explicit upstream pathway into the same shared condition mediator later read by the M3 condition-mortality hazard; a later condition-mediated death does not by itself apportion the deficit back to travel or resources.

For M9, temporary travel is explicitly duration-bearing. The household can remain in outbound or return transit for deterministic intervals before/after visiting. Travel cost/duration is derived from the declared world/focal-region binding and travel configuration; transit does not pretend to know a route cell. The current M9 null mechanism does not add route memory, camps, movement mortality, convoy structure or a cultural motive for travel.

### Interaction

v0.1 interaction is primarily household resource sharing, reproduction context, local density pressure, same-cell resource competition and bounded migration response. Global all-to-all interaction is prohibited.

Resource demand is aggregated only among households occupying the same cell. M4 candidate work is proportional to pressured households × bounded local candidate count, not households × all world cells. The hot path uses contiguous arrays indexed by cell/household/person plus reusable candidate buffers rather than pairwise person searches or a global household-interaction graph.

M2 parent selection is intentionally minimal: a male parent must be alive, inside the configured age interval and share the same persistent residence cell as the female parent. M9 visitor co-presence does not change parent eligibility. Among eligible males, one is selected uniformly. v0.1 does **not** model marriage, pair bonds, social paternity, mate preference, incest avoidance, kin exogamy, polygyny or reproductive status beyond the stated rules.

### Simultaneous movement

All household decisions at one M4 boundary use the same pre-move snapshot. Planned moves are applied only after candidate evaluation has finished for every household.

This prevents household-ID order from changing the information seen by later households. It also means multiple households can independently choose the same destination without anticipating one another's simultaneous arrival. Resulting crowding is experienced during subsequent resource settlement rather than being solved by a hidden global coordinator.

That simultaneous-arrival assumption is scientifically consequential and should be included in later sensitivity work.

### Stochasticity

All stochasticity comes from seeded named deterministic streams.

- M1: world-generation stream used to derive stable field seeds;
- M2: independent baseline mortality, fertility, parentage and newborn-sex streams;
- M3: independent condition-mortality stream whose private historical implementation label remains `resources/scarcity_mortality` for trajectory compatibility;
- M4: independent `migration/choice` and `migration/uncertainty` streams.

Resource-period allocation, seasonal integration, resource regeneration, household demand/allocation, elapsed condition response, candidate enumeration and integer utility calculations are deterministic conditional on state/configuration. Condition-mediated mortality, candidate uncertainty and weighted destination choice are stochastic but reproducible through their named streams. The v9 timing rule uses the exact rational interval probability derived from the reference-quarter condition probability rather than first rounding the draw threshold to parts per million. The historical private stream name is not a v10 scientific cause label.

### Observation

Authoritative events are versioned ordered state-transition records. In addition to births, deaths and completed M4 permanent migrations, M9 records the temporary-journey lifecycle needed to replay departure/arrival/return/completion history. Core invariant validation replays that lifecycle independently of optional derived reports.

Derived metric snapshots remain explicitly downstream and reconcile against authoritative state. Completed/paused run bundles preserve the world, founders, event history, metrics and checkpoint required for deterministic inspection/resume.

M8 and M9 observability intentionally answer different location questions. Under v10, `spatial-observability.json` schema v3 is residence-based: occupancy/person-days/birth/death cell attribution describes persistent residence and excludes temporary visitors/transit; its condition-mediated death field does not assert resource scarcity as the unique upstream cause. `temporary-observability.json` derives physical-presence measures such as residents, visitors, transit, focal-region person-days, peak visitor share, journey counts/durations and travel/catchment. When both are present, the Explorer displays them side by side rather than collapsing them into one ambiguous occupancy concept.

Interpretation remains downstream. Labels such as settlement, aggregation, refuge, ritual gathering, migration wave or collapse are not silently promoted into authoritative simulated ground truth.

## 5. Initialization

A run records a complete versioned experiment configuration: seed, world/population/demographic/resource/permanent-migration settings, duration/stop conditions and evidence provenance where present. Landscape-bound runs additionally bind normalized landscape identity plus explicit spatial-mechanism transformations. M9-enabled runs additionally bind focal-region identity and the complete temporary-mobility/travel configuration in immutable experiment identity.

The `synthetic_validation_v1` founder/demographic/resource/permanent-migration presets remain mechanism-testing baselines rather than neutral empirical priors. M9 temporary-mobility defaults likewise remain synthetic unless a research-specific configuration supplies a defensible evidence basis.

No historical route, settlement role, aggregation motive or destination outcome is initialized implicitly. Focal-region cells define where the temporary mechanism can act; they do not assert why a real place was used or that a historical aggregation occurred there.

## 6. Input data and provenance

The original v0.1 M1/M3/M4 runtime baseline is synthetic. M2 mechanisms are informed by published comparative evidence but its executable preset remains explicitly synthetic validation.

M8 adds a separate evidence/provenance layer: normalized external spatial inputs can be preserved with source identity, transformations and uncertainty/provenance metadata, then mapped deterministically into model-facing fields. Evidence parameter links are validated against the actual versioned serialized `ExperimentConfig` so a typo or obsolete path cannot masquerade as machine-readable provenance.

Real-world-derived input does not automatically validate the model transformation or downstream behavioural mechanism. Every empirical interpretation still requires source/units/uncertainty, a declared transformation and question-specific validation rationale.

M9's generic temporary-mobility mechanism is not itself an empirical dataset or behavioural claim. Its controlled aggregation benchmark is synthetic capability validation. Research-specific temporal frequency, duration, catchment or focal-region assumptions require their own evidence/uncertainty treatment before archaeological interpretation.

## 7. Implemented baseline and extensions

### Synthetic environment (M1 — implemented)

The environment is generated from smoothly varying deterministic fields derived from the run's `world` random stream. Water accessibility depends partly on synthetic wetness and lowland favourability; productivity combines water, a separate fertility field and lowland favourability; movement cost combines ruggedness and relative elevation. Seasonality uses a synthetic latitude gradient plus a spatial climate field.

These relationships exist to provide heterogeneous, causally inspectable test conditions. They are **not evidence-grounded ecological equations**.

### Demography (M2 — implemented synthetic baseline)

M2 uses replaceable age-specific schedules rather than hidden hard-coded anthropological constants. Age is derived from birth time. Mortality and fertility are stochastic under separate named deterministic streams, and exact births/deaths reconcile against persistent population totals.

The demographic research baseline is `docs/research/demography-v0.1.md`. It establishes that extant hunter-gatherer populations show substantial demographic diversity and should be used as comparative evidence/ranges rather than as one universal prehistoric proxy.

The first executable preset is therefore named `synthetic_validation_v1`, not `hunter_gatherer`. Its qualitative mortality/fertility shapes are evidence-informed, but the complete schedule is not calibrated to a real population and carries provenance status `synthetic_validation`.

Mortality is represented by a transparent piecewise age-specific annual event-probability schedule. Fertility is represented by an age-specific annual live-birth opportunity probability plus explicit minimum birth spacing. Probabilities are integer parts per million. Completed family size, survivorship and life expectancy are outputs/validation quantities rather than values directly forced on individuals.

M3 condition-mediated mortality remains separate rather than silently changing baseline demographic schedules. There is no direct food-to-fertility multiplier; introducing one requires an explicit hypothesis/evidence basis.

### Food, resource renewal, condition and condition-mediated mortality (M3 — implemented synthetic baseline)

M3 maintains one dynamic integer food-stock value per world cell. Its normative annual resource-accounting contract is [`research/m3-resource-time-contract-v1.md`](research/m3-resource-time-contract-v1.md); its v9 response-time contract is [`research/m3-response-time-contract-v1.md`](research/m3-response-time-contract-v1.md); and its v10 shared-condition mortality-cause contract is [`research/m3-condition-mortality-contract-v1.md`](research/m3-condition-mortality-contract-v1.md).

For `P` resource periods/year, period `i` covers:

```text
[ floor(i * 365 / P), floor((i + 1) * 365 / P) )
```

relative to the model-year start. For a fixed annual integer quantity `Q`, cumulative allocation after `t` elapsed days is:

```text
C_Q(t) = floor(Q * t / 365)
```

so the period share is `C_Q(end) - C_Q(start)`. This conserves the annual total exactly despite unequal integer-day period lengths. Under the default four-period schedule, annual need `100` therefore executes as `24, 25, 25, 26` rather than four independently rounded equal quarters.

The annual regeneration baseline is determined from cell baseline productivity, configured annual regeneration scale, experiment productivity scale and environmental stress. The synthetic triangular seasonal curve then changes **when** this annual potential is allocated. Its daily integer weights are integrated over the actual resource interval and normalized by the complete-year weight. Changing phase or tested resource-period resolution therefore does not by itself change unconstrained annual potential; finite stock capacity can still clip realized regeneration.

All arithmetic remains integer/fixed-point. Regeneration cannot raise a cell above configured stock capacity.

Living-person resource need is aggregated into households and cells. If total cell supply is insufficient, co-located households receive a proportional share according to household need. Within a household, the period supply fraction is shared equally among living members for condition change. Harvest is treated as immediate consumption; explicit storage, spoilage, waste and exchange are deferred. M9 may split a household's current-period demand between residence and visitor destination under its separate duration-aware provisioning contract.

For positive-demand intervals, full supply allows bounded condition recovery and deficit causes condition loss proportional to missing supply. A zero-demand interval is condition-neutral: it cannot turn an integer `0/0` into free recovery. The historical recovery/loss coefficients retain their v9 reference-quarter timing interpretation; an elapsed M3 interval receives only the corresponding fraction of that response budget. A continuously applicable response therefore sums to four reference-quarter quantities over a full year regardless of tested M3 partition.

General condition-mediated mortality uses the current condition deficit to calculate a reference-quarter probability from `maxConditionMortalityProbabilityPerMillion`. The v9 elapsed-time rule converts that probability to the actual M3 interval with exact rational conditional survival. At fixed condition, complete-year survival is the same composition of four reference-quarter survivals under tested `P = 1, 4, 12, 365`; finer M3 partition does not itself multiply mortality opportunity. The recorded event probability is a deterministic parts-per-million view of the exact rational interval probability.

Under v10, a death from this hazard has `cause = "condition_mediated"` and derived outputs use `conditionMortalityDeaths`. This means the shared condition state generated the configured hazard at that boundary. It does **not** mean resource scarcity uniquely caused the death, that the deficit was purely nutritional, or that travel uniquely caused it. Resource shortfall can lower condition and thereby causally increase condition-mediated mortality; M4 travel can also lower the same state. Because condition does not retain source shares, event-level apportionment between those upstream mechanisms is unavailable.

Issue #200 is therefore repaired by making the cause semantics match the executable shared mediator rather than inventing unsupported source attribution. Coincident M3 condition-mediated mortality and M2 annual demographic mortality remain a separate competing-risk attribution issue (#208). Evolving condition/resource trajectories can also remain resolution-sensitive because state is settled at different days even though the v9 hidden rate multiplier has been removed.

### Households (minimal M2–M4 baseline)

Households provide co-residence/location consistency, resource sharing and the M4 relocation unit. Formation/dissolution rules remain minimal. Household membership is not equivalent to parentage or marriage.

### Migration (M4 — implemented synthetic baseline)

Surviving households whose condition and/or local resource support create positive relocation pressure compare staying with cells inside a bounded Manhattan information radius at explicit fixed M4 opportunity boundaries.

`migration.decisionPeriodsPerYear` defines the M4 opportunity clock independently of M3 resource resolution; the synthetic validation default is four opportunities/year. The decision interval uses the same deterministic 365-day boundary construction as other fixed schedules. Changing `resources.periodsPerYear` alone therefore does not change how many permanent-migration decisions are available.

Candidate utility uses:

- current dynamic resource stock relative to annual per-person need allocated over the current M4 decision interval, after adding the moving household;
- a synthetic water/security score using water accessibility and inverse environmental stress;
- a deliberately narrow bounded living-direct-parent location proxy;
- distance and terrain movement cost;
- deterministic stochastic uncertainty;
- relocation-risk penalty.

Candidate evaluation is not global optimization. Alternatives must improve sufficiently over staying, then compete through weighted deterministic stochastic choice.

All households decide from one pre-move snapshot. Selected households then relocate simultaneously. Living members move together, their condition pays the travel cost, current household location changes, and occupancy is rebuilt. Dead records keep location at death. That travel-condition decrement is an explicit upstream input to the shared v10 condition mediator and can therefore affect later condition-mediated mortality without making the later death event a uniquely travel-attributed death.

There is no hard-coded historical destination or route, route memory, seasonal mobility tradition, clan/tribe institution or claim that synthetic M4 weights or four-per-year default reproduce real mobility behaviour. M4 itself has no persistent en-route state; M9 temporary mobility adds a separate duration-bearing transit/visiting lifecycle without changing permanent-migration semantics.

### Evidence-grounded spatial execution (M8 — implemented; v0.2.0 release baseline)

M8 preserves normalized landscape inputs and provenance separately from explicit deterministic model-facing transformations. Landscape/mechanism identity is carried through experiment/run artifacts, and residence-based spatial observability is derived downstream. Its first evidence-grounded terrain null-model benchmark found fragile rather than seed-stable spatial effects; this validates the experiment/reproduction capability, not a historical reconstruction. Under v10, spatial mortality observability uses general condition-mediated terminology rather than treating the death count as a resource-specific causal count.

### Temporary mobility and aggregation (M9 — implemented; v0.3.0 release baseline)

M9 separates persistent residence from physical presence and adds identity-bearing focal regions, deterministic temporary journey scheduling/travel, duration-aware resource attribution, checkpointable active journeys and separate physical-presence observability. The frozen M9.7 benchmark distinguished intermittent aggregation from continuous residence across its paired synthetic seeds while preserving exact duplicate and checkpoint/resume replay. This is a capability result only; no social motive or archaeological interpretation is encoded. Under the v10 rebaseline, its control criterion records zero condition-mediated deaths rather than claiming zero uniquely resource-scarcity deaths.

## 8. Model verification targets

Before scientific validation, implementation must satisfy:

- exact population accounting: `initial + births - deaths = living`;
- persistent record accounting: `initial + births = person records`;
- exact cumulative resource accounting: `initial stock + regeneration - harvest = final stock`;
- exact conservation of fixed annual integer resource quantities across scheduler-aligned periods;
- exact allocation of M4 annual demand over the declared M4 decision interval, with decision index/day reconciliation;
- zero-amplitude seasonal allocation reducing exactly to fixed elapsed-day allocation;
- unconstrained seasonal annual potential remaining invariant to phase and tested resource-period resolutions while non-zero seasonality can change within-year timing;
- resource need accounting: positive demand must reconcile to consumption plus unmet need;
- zero-demand intervals must not create condition recovery;
- reference-quarter condition-response budgets must remain invariant to tested M3 partitions when supply regime is held fixed;
- fixed-condition condition-mediated survival must compose identically across tested `P = 1, 4, 12, 365`, and `P = 4` must reproduce the configured reference-quarter probability exactly;
- a controlled travel-only condition deficit under full positive resource supply must be able to exercise the general `condition_mediated` cause without being labelled resource scarcity;
- a controlled resource-only deficit must still be able to change condition and condition-mediated mortality directionally;
- mixed resource + travel deficits must retain the general condition-mediated cause rather than inventing unsupported causal apportionment;
- v10 wire artifacts must use the condition-mortality names and reject the old scarcity-specific resource mortality config field;
- changing only M3 `resources.periodsPerYear` while holding M4 `decisionPeriodsPerYear` fixed must not change the number of configured M4 opportunities per complete year;
- changing M4 `decisionPeriodsPerYear` must change M4 opportunity count independently of M3 resolution;
- both ordinary and spatial-landscape simulation hosts must obey the same independent-clock scheduler;
- valid stable IDs and genealogy references after parent death;
- no death before birth;
- no self-parent, duplicate-parent, wrong-reproductive-sex parent or non-older parent relationships;
- valid current household membership/location relationships for living people;
- condition values within declared bounds;
- occupancy index reconciliation with authoritative locations;
- deterministic replay under the supported platform/build boundary;
- isolation of named RNG streams;
- explicit non-scientific stop reason when the persistent-record safety ceiling is reached;
- directional resource-scarcity intervention test: severe sustained zero-resource conditions must not improve condition/survival;
- directional productivity test: an otherwise-equal positive-resource case designed to isolate M3 must support at least as much condition/survival as a zero-productivity case;
- bounded migration-candidate discovery independent of total-world search;
- deterministic migration replay including migration digest and retained decision traces;
- directional migration-pressure test: worsening local resource/condition inputs must not reduce relocation pressure under otherwise equal inputs;
- completed migration traces must remain within the configured local radius and expose origin/destination utility factors;
- migration-enabled and migration-disabled otherwise-equal tests can diverge spatially through the implemented movement mechanism;
- household relocation and occupancy invariants remain valid after simultaneous moves;
- migration candidate lookup and the full v0.1 target workload remain benchmarked in CI;
- every M9 temporary journey has a unique coherent departure → arrival → return-departure → completion history with stable household/region/residence/destination identity;
- M9 state/events replay exactly across uninterrupted and annual-boundary resumed execution, including checkpoints taken while journeys are active;
- transformed spatial runs receive the same core invariant validation as ordinary runs;
- M8 residence-based spatial observability and M9 physical-presence observability declare and preserve their different location semantics;
- compact enabled-M9 cross-platform golden outputs remain byte-identical across supported Linux, Windows and macOS CI runners.

Passing these targets verifies implementation properties. It does not validate demographic, resource, condition-mortality or migration assumptions against reality.

## 9. Validation plan

The implemented M2, M3 and M4 baselines remain **unvalidated**. Passing software verification is not equivalent to empirical validation.

A research-capable demographic/resource/migration configuration will require, as appropriate to the bounded research question:

- population-specific or explicitly sampled demographic parameterizations;
- defensible energetic/ecological units and source provenance for claimed resource interpretations;
- evidence-grounded or explicitly hypothetical relationships between resources, condition, fertility and mortality;
- comparison of survivorship, age-specific mortality, fertility, birth spacing and growth against declared calibration/validation targets;
- independent empirical or archaeological patterns appropriate to resource/condition claims;
- evidence on mobility scale, settlement duration/relocation frequency, **decision-opportunity frequency** and information horizon where movement is part of the claim;
- explicit treatment of analogy limits when ethnographic mobility evidence is used;
- defensible travel/terrain costs and social/kin assumptions;
- sensitivity to simultaneous-arrival crowding, M4 atomic permanent relocation, M4 `decisionPeriodsPerYear`, and M9 temporary-journey duration/transit assumptions;
- explicit examination of annual/subannual scheduling effects even after the v9 hidden-rate repair, because different M3 settlement days can still change evolving state and later process exposure;
- controlled intervention designs when distinguishing resource versus travel effects on the shared condition state, because a `condition_mediated` death does not itself identify the upstream source;
- explicit examination of coincident M3/M2 competing-risk attribution (#208) where cause-specific mortality outputs matter;
- explicit examination of initialization transients and initial resource stock;
- calibration only where justified by a stated research question;
- global/local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. Research progression

The first v0.1 resource-variability experiment, the M8 evidence-grounded terrain benchmark and the M9 controlled aggregation benchmark are completed reference exercises rather than future candidates.

The M8 result showed that a declared terrain-only transformation could perturb spatial outcomes without yielding a stable directional effect across seeds. The M9 result showed that continuous residence and intermittent temporary aggregation can produce measurably different physical-presence histories under frozen synthetic assumptions while remaining deterministic and checkpoint-replayable.

Post-M9 scientific hardening is repairing known causal/model-contract defects before those reference capabilities are used for stronger inference. The v8 M3 resource-time repair established coherent annual quantity and seasonal timing semantics. The v9 #204 response-time repair then separated M3 integration resolution from elapsed condition response and from the M4 permanent-migration opportunity clock, removing one hidden temporal confound. The v10 #200 repair now makes the shared low-condition hazard causally explicit: resource scarcity can reduce condition, M4 travel can reduce the same condition, and a later death is recorded as condition-mediated rather than uniquely scarcity-caused. Coincident M3/M2 competing-risk attribution (#208) remains separately open.

None of these results or repairs establishes an archaeological explanation. The next mechanism or experiment is intentionally question-led: identify a real discriminating research question, state competing hypotheses and observables, determine which assumptions/evidence can constrain them, then add only the missing capability required for that comparison. Negative, fragile and equifinal results remain valid outcomes and must not be tuned away.
