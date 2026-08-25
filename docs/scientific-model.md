# Scientific model specification (ODD-oriented)

**Status:** working specification for the AnthroSim v0.3.0 / completed-M9 release baseline
**Scientific status:** exploratory / unvalidated

This document began as the v0.1 ODD-oriented model specification and now records the scientific meaning of the implemented baseline through M9. Historical M1–M4 sections remain relevant to the synthetic demographic/resource/permanent-migration baseline; M8 adds evidence-grounded spatial binding and M9 adds a separate temporary-mobility layer. Software verification and successful capability benchmarks are not empirical validation of human prehistory.

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

Reproductive sex remains a deliberately limited biological state variable for the current birth mechanism; it is not a model of social gender. Condition remains a synthetic 0..1000 permille energetic/health mediator rather than a clinical or directly empirical measure.

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

Authoritative simulation time is represented in integer days. Resource/condition processing occurs at explicit subannual boundaries and M2 baseline demography at annual boundaries.

M4 permanent relocation remains atomic at its decision boundary. M9 temporary journeys are explicitly duration-bearing: departure, arrival, visiting duration, return departure and completion occur on deterministic days, and journeys can remain active across an annual checkpoint. When multiple processes share a day, elapsed resource accounting settles first, then due temporary transitions/start decisions, then eligible at-residence M4 permanent migration, then annual M2 demography where applicable.

These schedules are model approximations and are scientifically consequential assumptions, not claims that real births, deaths, gathering, travel or physiological change occur synchronously.

### Space

The baseline world is a bounded rectangular grid. M4 permanent destination discovery uses a bounded Manhattan information radius rather than global optimization. M8 can replace the synthetic model-facing fields with deterministic transformations of a normalized provenance-tracked landscape while preserving the same stable cell identity contract.

M9 focal regions are identity-bearing bindings over declared cell sets. Temporary travel resolves deterministic cost/duration from household residence to the focal-region destination under the declared travel semantics. A journey's transit phase intentionally has no authoritative world cell, preventing false precision about unmodelled routes.

## 3. Process overview and scheduling

The baseline M2–M4 process remains: subannual renewable-resource settlement updates household supply/condition/scarcity survival, surviving pressured households may make bounded permanent-migration decisions from a shared pre-move snapshot, selected permanent moves are applied simultaneously, and the final period is followed by the annual demographic boundary.

M9 overlays temporary mobility without changing the meaning of M4 permanent migration. The governing same-day order is:

1. settle the elapsed resource interval using duration-aware residence/visitor/transit person-days;
2. complete any due temporary transitions and evaluate/start due temporary journeys;
3. evaluate M4 permanent migration only for eligible households physically at residence;
4. run M2 annual demography if this is an annual boundary.

Temporary lifecycle state progresses through departure, outbound transit, visiting, return departure, return transit and completion. Focal-region identity, residence, destination, timing and people affected remain stable across a journey's authoritative event history. Transit uses an explicit home-provisioning resource proxy rather than an invented route cell.

M2 mortality/fertility remains annual and residence-based. Births inherit household residence. If a person dies while the household is away, temporary-presence state is updated so visitor/transit counts remain correct; `Death.cell` remains a residence attribution field and must not be read as an observed physical death location.

Scheduling is part of the model definition and must be included in sensitivity/validation work. Detailed contracts are in [`research/temporary-mobility-v1.md`](research/temporary-mobility-v1.md), [`research/m9-temporary-travel-semantics-v1.md`](research/m9-temporary-travel-semantics-v1.md), and [`research/m9-duration-aware-resource-semantics-v1.md`](research/m9-duration-aware-resource-semantics-v1.md).

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

M4 permanent migration and M9 temporary mobility have different travel abstractions.

For M4, candidate utility includes distance/terrain travel penalty and relocation risk, and a selected permanent move completes atomically at that boundary with a condition cost. It has no persistent en-route state.

For M9, temporary travel is explicitly duration-bearing. The household can remain in outbound or return transit for deterministic intervals before/after visiting. Travel cost/duration is derived from the declared world/focal-region binding and travel configuration; transit does not pretend to know a route cell. The current M9 null mechanism does not add route memory, camps, movement mortality, convoy structure or a cultural motive for travel.

### Interaction

v0.1 interaction is primarily household resource sharing, reproduction context, local density pressure, same-cell resource competition and bounded migration response. Global all-to-all interaction is prohibited.

Resource demand is aggregated only among households occupying the same cell. M4 candidate work is proportional to pressured households × bounded local candidate count, not households × all world cells. The hot path uses contiguous arrays indexed by cell/household/person plus reusable candidate buffers rather than pairwise person searches or a global household-interaction graph.

M2 parent selection is intentionally minimal: a male parent must be alive, inside the configured age interval and share the same persistent residence cell as the female parent. M9 visitor co-presence does not change parent eligibility. Among eligible males, one is selected uniformly. v0.1 does **not** model marriage, pair bonds, social paternity, mate preference, incest avoidance, kin exogamy, polygyny or reproductive status beyond the stated rules.

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

Authoritative events are versioned ordered state-transition records. In addition to births, deaths and completed M4 permanent migrations, M9 records the temporary-journey lifecycle needed to replay departure/arrival/return/completion history. Core invariant validation replays that lifecycle independently of optional derived reports.

Derived metric snapshots remain explicitly downstream and reconcile against authoritative state. Completed/paused run bundles preserve the world, founders, event history, metrics and checkpoint required for deterministic inspection/resume.

M8 and M9 observability intentionally answer different location questions. `spatial-observability.json` schema v2 is residence-based: occupancy/person-days/birth/death cell attribution describes persistent residence and excludes temporary visitors/transit. `temporary-observability.json` derives physical-presence measures such as residents, visitors, transit, focal-region person-days, peak visitor share, journey counts/durations and travel/catchment. When both are present, the Explorer displays them side by side rather than collapsing them into one ambiguous occupancy concept.

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

There is no hard-coded historical destination or route, route memory, seasonal mobility tradition, clan/tribe institution or claim that synthetic M4 weights reproduce real mobility behaviour. M4 itself has no persistent en-route state; M9 temporary mobility adds a separate duration-bearing transit/visiting lifecycle without changing permanent-migration semantics.

### Evidence-grounded spatial execution (M8 — implemented; v0.2.0 release baseline)

M8 preserves normalized landscape inputs and provenance separately from explicit deterministic model-facing transformations. Landscape/mechanism identity is carried through experiment/run artifacts, and residence-based spatial observability is derived downstream. Its first evidence-grounded terrain null-model benchmark found fragile rather than seed-stable spatial effects; this validates the experiment/reproduction capability, not a historical reconstruction.

### Temporary mobility and aggregation (M9 — implemented; v0.3.0 release baseline)

M9 separates persistent residence from physical presence and adds identity-bearing focal regions, deterministic temporary journey scheduling/travel, duration-aware resource attribution, checkpointable active journeys and separate physical-presence observability. The frozen M9.7 benchmark distinguished intermittent aggregation from continuous residence across its paired synthetic seeds while preserving exact duplicate and checkpoint/resume replay. This is a capability result only; no social motive or archaeological interpretation is encoded.

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
- migration candidate lookup and the full v0.1 target workload remain benchmarked in CI;
- every M9 temporary journey has a unique coherent departure → arrival → return-departure → completion history with stable household/region/residence/destination identity;
- M9 state/events replay exactly across uninterrupted and annual-boundary resumed execution, including checkpoints taken while journeys are active;
- transformed spatial runs receive the same core invariant validation as ordinary runs;
- M8 residence-based spatial observability and M9 physical-presence observability declare and preserve their different location semantics;
- compact enabled-M9 cross-platform golden outputs remain byte-identical across supported Linux, Windows and macOS CI runners.

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
- sensitivity to simultaneous-arrival crowding, M4 atomic permanent relocation, and M9 temporary-journey duration/transit assumptions;
- explicit examination of annual/subannual scheduling effects;
- explicit examination of initialization transients and initial resource stock;
- calibration only where justified by a stated research question;
- global/local sensitivity analysis;
- uncertainty quantification;
- pattern-oriented validation across multiple observables;
- external domain review.

A preset fails validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerance/uncertainty. Failure is reported rather than tuned away invisibly.

## 10. Research progression

The first v0.1 resource-variability experiment, the M8 evidence-grounded terrain benchmark and the M9 controlled aggregation benchmark are now completed reference exercises rather than future candidates.

The M8 result showed that a declared terrain-only transformation could perturb spatial outcomes without yielding a stable directional effect across seeds. The M9 result showed that continuous residence and intermittent temporary aggregation can produce measurably different physical-presence histories under frozen synthetic assumptions while remaining deterministic and checkpoint-replayable.

Neither result establishes an archaeological explanation. The next mechanism or experiment is intentionally question-led: identify a real discriminating research question, state competing hypotheses and observables, determine which assumptions/evidence can constrain them, then add only the missing capability required for that comparison. Negative, fragile and equifinal results remain valid outcomes and must not be tuned away.
