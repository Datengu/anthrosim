# AnthroSim ODD 2020 model description

**Protocol:** ODD 2020 (Grimm et al. 2020)  
**AnthroSim baseline:** post-v0.3.2 scientific-hardening line / current model semantics v24 (immutable v0.3.2 release baseline: v19)  
**Status:** formal living ODD description  
**Scientific status:** exploratory / unvalidated

This document gives AnthroSim's model description in the seven-element ODD 2020 structure. The detailed normative semantics remain in [`../scientific-model.md`](../scientific-model.md); this document is the standards-facing description and index. It is intentionally explicit when a mechanism is synthetic, absent or not empirically validated. The repaired M2 annual transition semantics are specified more precisely in [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md), while day-zero founder reproductive/genealogical state is specified in [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md). M3 annual resource accounting remains specified in [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md), v9 elapsed condition-response and independent M4 decision timing are specified in [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md), and v10 shared-condition mortality cause semantics are specified in [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).

ODD describes the model. It does not by itself establish that the model is fit for a real archaeological or anthropological inference. Evaluation evidence is tracked separately in [`trace.md`](trace.md), while human decision assumptions are expanded in [`odd-d.md`](odd-d.md).

---

## 1. Purpose and patterns

### Purpose

AnthroSim is a deterministic/reproducible agent-based simulation framework for exploring whether explicit local causal mechanisms can generate interpretable population, resource, spatial and mobility patterns without scripting historical outcomes.

The current baseline supports three connected methodological purposes:

1. **Synthetic causal exploration:** test demographic, resource and permanent-migration mechanisms under transparent assumptions.
2. **Evidence-grounded spatial experimentation:** bind normalized external spatial evidence to declared model-facing transformations while keeping evidence, transformation and simulated result conceptually separate.
3. **Residence versus temporary presence:** test whether persistent residence and intermittent temporary aggregation can produce distinguishable physical-presence histories without treating temporary visitation as settlement.

The general purpose type is primarily **understanding/explanation and methodological demonstration**, not point prediction of a specific past society.

The baseline is not a validated reconstruction of human prehistory. It does not encode a tribe, polity, ethnic group, settlement function, ritual motive, war, market, language or historical route unless a future question-specific model explicitly introduces and justifies such a mechanism.

### Patterns used to evaluate suitability

AnthroSim distinguishes four kinds of patterns/criteria:

**A. Implementation-verification patterns**

Examples include exact population/resource accounting, deterministic replay, valid genealogy, bounded state, coherent residence/presence histories, checkpoint-resume equivalence and named RNG-stream isolation. These test whether the implementation matches its declared semantics; they do not validate anthropology.

**B. Directional/mechanism patterns**

Examples include the expectation that, under otherwise equal controlled conditions, worsening local resource/condition inputs must not reduce declared migration pressure, increasing relocation-only cost must not make relocation more attractive, and severe sustained resource deprivation must not improve condition/survival. These are falsification-oriented mechanism checks.

**C. Synthetic capability patterns**

The preserved M8 and M9 benchmarks test whether the engine can express and distinguish declared synthetic mechanisms reproducibly. The M8 terrain exercise found fragile spatial effects rather than a stable historical result. The M9 benchmark demonstrated distinguishable intermittent-presence versus continuous-residence histories under frozen synthetic assumptions. Neither is empirical validation.

**D. Future empirical/pattern-oriented validation criteria**

A research configuration may require simultaneous agreement with multiple independent patterns appropriate to its question, for example survivorship/age structure, fertility and birth spacing, population persistence, relocation frequency/distance, occupation/presence duration, spatial concentration or other independently justified observables. Such patterns must be predeclared and evidence-linked rather than selected only after results are known.

Detailed verification and validation targets are listed in [`../scientific-model.md`](../scientific-model.md) and tracked under TRACE in [`trace.md`](trace.md).

### Rationale

AnthroSim deliberately treats patterns as evaluation criteria rather than as outcomes that the simulator is instructed to produce. This preserves falsifiability and reduces the risk of embedding the desired archaeological conclusion in the model rules.

---

## 2. Entities, state variables and scales

### People

Persistent person records include stable identity, birth time/derived age, reproductive sex for the current reproduction mechanism, condition, household membership, parent references, birth-history state and death state. Dead people remain persistent records so genealogy does not change retrospectively.

A living person's persistent **residence** is distinct from M9 **physical presence**. Physical presence can be at residence, outbound transit, a focal-region visit, or return transit. Only permanent migration changes residence.

### Households

Households are persistent resource-sharing and mobility units. They are not asserted to be tribes, clans, lineages, marriages or universal nuclear families. Living household members normally share persistent residence; newborns join the female parent's household. M4 permanent migration moves the living household as a unit. M9 temporary mobility moves the household through a temporary journey state while preserving residence.

Household formation/dissolution remains intentionally minimal in the baseline and is therefore a model limitation to be evaluated when household lifecycle could affect a research claim.

### World cells

The world is a bounded rectangular grid of stable cells. Model-facing cell state/fields include movement cost, water accessibility, productivity/resource opportunity, seasonality, environmental stress and dynamic renewable resource stock, plus occupancy/presence relationships derived from people/households.

Synthetic runs generate environmental fields deterministically. M8 can instead bind normalized evidence-derived spatial inputs to explicit deterministic transformations. Real-world-derived inputs do not automatically validate the transformation or behavioural response.

### Focal regions and temporary journeys

M9 focal regions are identity-bearing declared sets of world cells. Temporary journeys preserve household, residence, destination/focal-region identity, timing and lifecycle state through departure, transit, visitation, return and completion.

### Time

Authoritative time is integer days.

- M2 baseline demography is an annual discrete transition evaluated at positive multiples of 365 days. At boundary `t`, age-specific mortality/fertility bands are selected from age at the start of `[t-365,t)`, not age at `t`.
- M2 mortality is drawn before fertility; the current fertility probability is therefore conditional on surviving the annual demographic mortality transition, subject also to spacing and parent-availability filters.
- Declared founders may carry signed pre-run birth-history timing before day 0; this initial-condition chronology can constrain later M2 birth spacing without being recorded as a model-period birth event.
- For `P = resources.periodsPerYear`, M3 resource interval `i` is the exact half-open interval `[floor(i*365/P), floor((i+1)*365/P))` within the model year. Fixed annual integer quantities are allocated by cumulative elapsed days so their complete-year shares conserve exactly.
- M3 resource settlement occurs at the end of those configured intervals. Seasonal regeneration integrates the synthetic daily seasonal curve over the actual interval and normalizes it to preserve unconstrained annual potential.
- M3 condition recovery/loss coefficients and the condition-mediated mortality probability are interpreted against four fixed reference-quarter intervals, then converted deterministically to the actual elapsed M3 interval. Changing only `P` therefore does not multiply the complete-year response budget or fixed-condition survival probability merely by creating more M3 boundaries.
- For `D = migration.decisionPeriodsPerYear`, M4 permanent-migration opportunity `j` occurs at `floor((j+1)*365/D)` within the model year. The synthetic default is four opportunities/year. This clock is independent of M3 `P`.
- M4 resource support allocates annual per-person need over its own current decision interval using the same cumulative elapsed-day allocation rule rather than requiring an M3 resource boundary.
- M9 transitions and starts can occur on deterministic journey days and can span annual checkpoints.

When M3 and M4 share a day, the declared subannual ordering is M3 settlement/condition/survival, then due M9 day processing, then M4 permanent migration. Either process may otherwise occur alone. The annual M2 transition follows the year's subannual processing. The annual M2 contract remains intentionally coarse and must not be described as continuous-time mortality/fertility hazard execution.

The v9 response-time repair removes the specific artifact where finer M3 partition automatically created more physiological, condition-mortality or M4 opportunities. It does **not** make `resources.periodsPerYear` scientifically irrelevant: changing settlement times can still change stock, evolving condition, extinction timing, M9 demand attribution and the state observed by later M4 decisions. Such remaining scheduling sensitivity is part of model evaluation rather than hidden rate multiplication.

Under v10, the M3 low-condition hazard is scientifically a **general condition-mediated mortality pathway**. Resource shortfall can reduce condition, but M4 permanent-travel cost can reduce the same condition scalar. Because the model does not preserve an additive decomposition of that scalar by upstream cause, a later low-condition death cannot be attributed uniquely to resource scarcity (or uniquely to travel) from the death event alone.

### Space

The baseline space is a bounded rectangular grid. M4 candidate discovery uses a bounded Manhattan radius in cells. M9 uses deterministic travel-cost/duration semantics over the declared landscape/focal-region binding. M8 provides normalized grid/evidence binding and declared transformations.

Cell size, grid extent, boundary conditions and the translation between cell units and physical distance are scientific assumptions for evidence-grounded applications and must be included in resolution/boundary sensitivity work.

### Rationale

Persistent identity and explicit residence/presence separation allow causal histories to be inspected rather than inferred from aggregate snapshots. The grid is a computational abstraction; it must not be mistaken for an intrinsically meaningful archaeological spatial scale.

---

## 3. Process overview and scheduling

Within a model year, AnthroSim merges the independent fixed M3 and M4 schedules. At each next due fixed day:

1. process any due M9 temporary journey boundaries strictly before that fixed day;
2. if an M3 resource boundary is due, settle elapsed resource demand/regeneration, update condition and apply the elapsed M3 condition-mediated survival semantics;
3. process due M9 temporary journey transitions/start decisions for that day;
4. if an M4 decision boundary is due, evaluate eligible permanent-migration decisions from the declared shared pre-move state, comparing an explicit zero-action-cost stay utility with candidate residence utility minus relocation-only travel, uncertainty and relocation-risk costs;
5. apply selected permanent moves according to the simultaneous-movement contract;
6. after the year's subannual schedules complete, execute the M2 discrete transition for `[t-365,t)`: use interval-start age bands, draw mortality, then evaluate conditional fertility/parentage among survivors;
7. update authoritative events/checkpoint/derived observability as specified by the run lifecycle.

Under the v8 resource-time contract, M3 uses exact elapsed-day resource intervals, cumulative elapsed-day allocation for fixed annual quantities, integrated/normalized seasonal regeneration, and zero-demand condition neutrality. Under v9, condition response and the condition-mediated mortality probability are converted from explicit reference-quarter coefficients to the actual M3 interval, while M4 receives its own fixed decision schedule. M4's resource-support denominator is the annual need allocated over the M4 decision interval, not an independent `ceil(annual/P)` approximation and not an M3-period side effect. Under v10, the same numerical low-condition hazard is explicitly general condition-mediated mortality rather than a resource-scarcity-specific death cause.

For a zero-demand M3 interval, condition remains unchanged rather than treating `0/0` as fully supplied recovery. For positive demand, the existing supply-dependent recovery/loss rule is applied to the elapsed response amount. At fixed condition, condition-mediated survival across a full year composes identically under tested `P = 1, 4, 12, 365`; evolving-condition trajectories may still differ because state changes occur at different times.

M9 duration-aware resource accounting can attribute elapsed person-days to residence, focal-region visitation or transit according to its declared provisioning proxy. Temporary mobility does not by itself redefine persistent residence or M2 parentage locality.

When M4 and M2 share annual-boundary day `t`, a just-entered M4 destination contributes zero elapsed exposure to `[t-365,t)`. M2 therefore reconstructs parentage locality from persistent residence immediately before that same-day M4 relocation. A newborn is nevertheless stored at the female parent's current boundary-state residence after M4. M2 demographic `Death.cell` retains its existing boundary-state residence meaning because current M2 mortality is not spatially parameterized.

Permanent M4 relocation is atomic at its decision boundary; M9 temporary travel is duration-bearing.

The detailed same-day ordering and lifecycle contracts are specified in:

- [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md)
- [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md)
- [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md)
- [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md)
- [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md)
- [`temporary-mobility-v1.md`](temporary-mobility-v1.md)
- [`m9-temporary-travel-semantics-v1.md`](m9-temporary-travel-semantics-v1.md)
- [`m9-duration-aware-resource-semantics-v1.md`](m9-duration-aware-resource-semantics-v1.md)

### Rationale

Scheduling is part of the scientific model, not an implementation detail. AnthroSim therefore records it explicitly and requires temporal-resolution/scheduling sensitivity before strong claims when multiple mechanisms interact on different frequencies.

---

## 4. Design concepts

### 4.1 Basic principles

AnthroSim follows the principle **model causes, not historical outcomes**. Local mechanisms and constraints generate trajectories; no rule may directly command a population to produce a desired archaeological narrative.

The model is generative and mechanism-oriented. Deterministic reproducibility is treated as software/research infrastructure, not evidence of empirical validity.

### 4.2 Emergence

Potentially emergent outputs include population concentration/dispersion, local crowding, differential condition/survival, migration frequency/direction, residence distributions, temporary aggregation histories and resource-pressure patterns.

Higher-level labels such as settlement, ritual centre, polity, refuge or migration wave are not authoritative model state and must be justified downstream.

### 4.3 Adaptation

M4 provides a limited adaptive response: pressured households may relocate to locally evaluated alternatives when an explicit M4 decision opportunity occurs. M9 baseline temporary mobility follows configured trigger/scheduling semantics rather than a general adaptive theory of gathering motives.

No general behavioural learning, cultural adaptation or evolving strategy is present in v0.3.0.

### 4.4 Objectives

M4 uses an explicit synthetic bounded utility comparison. Resource support, water/security and a reciprocal cross-household living parent-child location proxy are treated as residence-state terms. Each living parent-child edge that crosses households connects both households to one another's persistent residence regardless of reproductive-sex role; same-household relations add no spatial preference, and there is no fixed record-order-dependent cap. The resource-support term compares available dynamic stock against annual per-person resource need allocated over the current **M4 decision interval**. Staying evaluates those terms at the current residence with zero travel, candidate uncertainty and relocation-risk costs. Candidate relocations evaluate the same residence-state terms for the destination and additionally pay travel/terrain, uncertainty and relocation-risk costs. Candidates must improve sufficiently over the explicit stay utility before participating in weighted choice.

The M4 opportunity clock itself is a model assumption: `migration.decisionPeriodsPerYear` is independently configurable and defaults to four/year in the synthetic validation baseline. Neither the utility equation nor that opportunity frequency is claimed to reproduce real human decision cognition/rates.

This is a mechanism-testing objective function, not a claim that real people maximize this utility or consciously calculate or partition these terms in this way.

### 4.5 Learning

No persistent learning or updating of behavioural rules is implemented in the current baseline. Agents do not learn routes, revise utility weights, accumulate social memory or infer hidden environmental states.

Absence of learning is a declared null assumption, not evidence that learning was historically unimportant.

### 4.6 Prediction

Households do not run explicit forward simulations of future resource or social states. M4 evaluates current/proxy candidate conditions and deterministic uncertainty at its current decision boundary. M9 follows its configured journey timing/travel semantics.

### 4.7 Sensing

M4 household information is bounded spatially by the candidate radius and uses only declared model-facing state/proxies. Households do not have global omniscience. Candidate environmental/resource fields are treated as locally available proxies for the synthetic decision mechanism.

M9 focal-region/travel configuration is model structure, not necessarily agent knowledge in a cognitive sense.

### 4.8 Interaction

Major interactions include:

- within-household resource sharing;
- same-cell competition for renewable resources;
- reproduction through pre-same-boundary-M4 persistent-residence parent eligibility;
- a narrow reciprocal cross-household living parent-child-location contribution to M4 utility, including declared founder parent state available from day 0 when supplied;
- crowding/resource consequences after multiple households relocate or visit.

M4 decisions at one boundary use a common pre-move snapshot so household iteration order does not become a hidden information advantage. Households do not anticipate one another's simultaneous moves.

### 4.9 Stochasticity

All stochastic processes use seeded named deterministic random streams. Separate streams exist for the model's declared demographic, condition-mortality, migration-choice and uncertainty processes. The condition-mortality stream retains a private historical scarcity-oriented identifier for trajectory compatibility; that implementation label is not the public v10 cause semantics. Conditional on configuration/state and supported determinism boundary, runs are reproducible.

Reproducibility does not imply that a stochastic mechanism is scientifically well parameterized.

### 4.10 Collectives

Households are the principal collective entity in the current model. Focal regions are spatial constructs rather than social groups. Tribes, clans, polities, markets, armies, ritual institutions and other higher-order human collectives are not represented in the baseline.

### 4.11 Observation

Authoritative state/events record simulated model history. Derived metrics and observability layers are downstream views that must reconcile with authoritative state.

M8 spatial observability is residence-based. M9 temporary observability records physical presence and journey-derived measures separately. The Explorer must not collapse these into one ambiguous concept.

Condition-mediated deaths identify execution of the shared-condition hazard. They do not, without a controlled upstream intervention, identify resource scarcity as the unique cause of the condition deficit.

Simulated presence/activity is not automatically equivalent to preserved archaeological evidence. A question requiring comparison with material remains needs an explicit observation/taphonomic/sampling model or a justified downstream comparison layer.

### Rationale

The design concepts intentionally expose absences as well as implemented mechanisms. An unrepresented human process is not a hidden constant; it is a boundary on what the model can legitimately explain.

For human-decision detail, see [`odd-d.md`](odd-d.md).

---

## 5. Initialization

Every run records a complete versioned experiment configuration including seed, world/population/demography/resource/permanent-migration controls, duration/stop conditions and applicable evidence/spatial/M9 configuration.

The default founder, demographic, resource and migration presets are explicitly synthetic validation baselines. They are not neutral prehistoric priors.

AnthroSim now distinguishes two founder-initialization meanings. `synthetic_validation_v1` remains the deterministic engineering/null-model generator for founder population, target household grouping, synthetic age distribution, reproductive-sex distribution and household locations; it intentionally carries no claim to realistic prehistory. `declared_founder_state_v1` instead materializes an explicit versioned founder definition containing exact founder chronology, reproductive sex, household/residence, condition, optional signed pre-run last-birth timing and optional living direct-parent links.

Declared pre-run birth timing constrains M2 birth spacing until superseded by a model-period birth without creating fictitious pre-run child records or incrementing runtime birth accounting. Declared living direct-parent links are authoritative Population state and can therefore affect the M4 kin proxy on its first eligible migration boundary. When M4 has non-zero active kin weighting, declared genealogy marked `unspecified` fails closed rather than being interpreted as evidence of no living direct kin.

The declared path removes the requirement that research-facing founder history begin implicitly at zero, but it is not itself a stable-population generator or evidence of empirical adequacy. The procedure that creates a founder declaration must be justified separately, and initialization/burn-in sensitivity remains required. Normative semantics are in [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md).

World/resource initial state is generated deterministically or supplied through the M8 evidence-grounded landscape path. M9-enabled runs bind focal-region identity and temporary-mobility/travel configuration as part of immutable experiment identity.

Initial conditions are scientifically consequential. Question-specific research must examine initialization/burn-in sensitivity where founder age structure, reproductive history, genealogy, condition, household structure, resource stock, start season or starting locations could affect conclusions.

### Rationale

Explicit initialization prevents day-zero assumptions from being mistaken for emergent history and allows initialization transients to be measured rather than ignored.

---

## 6. Input data

### Synthetic inputs

The baseline M1 environment and default M2/M3/M4 parameter sets are synthetic mechanism-validation inputs, although some qualitative demographic shapes are evidence-informed. Synthetic inputs must remain labelled as such.

### Evidence-grounded spatial inputs

M8 supports normalized external spatial evidence with declared provenance, units/coverage where available, transformations and content identity. The source input is preserved separately from model-facing transformed fields.

Relevant contracts include:

- [`landscape-contract-v1.md`](landscape-contract-v1.md)
- [`landscape-loading-v1.md`](landscape-loading-v1.md)
- [`landscape-preprocessing-v1.md`](landscape-preprocessing-v1.md)
- [`spatial-mechanisms-v1.md`](spatial-mechanisms-v1.md)
- [`evidence-provenance.md`](evidence-provenance.md)

### Parameter/evidence provenance

The evidence catalogue can record source identity, original variable/units, transformation, simulation units, uncertainty, applicability, competing estimates, parameter links and external-input links.

A real-world source does not validate a transformation merely because the source itself is empirical. The transformation and downstream behavioural interpretation require their own rationale/evaluation.

### Calibration and corroboration roles

For inferential studies, evidence must be assigned a declared role: model construction/parameterisation, calibration, model-output verification, or independent corroboration. Evidence used to tune a model cannot later be presented as independent confirmation of that same fit.

### Rationale

AnthroSim keeps source evidence, transformations and simulation outputs separable so that uncertainty and interpretation can be traced through the causal chain.

---

## 7. Submodels

### M1 — synthetic environment

Creates deterministic heterogeneous environmental fields for mechanism testing. Current relationships among wetness/elevation/ruggedness/productivity/movement are synthetic and not empirical ecological laws.

Primary implementation: `crates/anthrosim-core/src/world.rs` and related world/config code.

### M2 — demography and genealogy

Maintains persistent people, age-derived state, mortality/fertility schedules, parentage and births. The default schedule is `synthetic_validation_v1`; it is not calibrated to one prehistoric population. The current implementation is an explicit annual discrete transition: age bands are selected from interval-start age, fertility is conditional on surviving annual M2 mortality, requested day-valued birth spacing is normalized to executable annual boundaries, same-day M4 relocation does not redefine prior parentage locality, and newborn condition inherits the female parent's boundary condition. Founder initialization can remain explicitly synthetic or use an exact declared state whose signed pre-run reproductive timing and living direct-parent links are available from the start of the run.

Primary specifications: [`demography-v0.1.md`](demography-v0.1.md), [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md), [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md).  
Primary implementation: `crates/anthrosim-core/src/population.rs`, `crates/anthrosim-core/src/demography.rs`, `crates/anthrosim-core/src/founder_initialization.rs` and demographic/config code.

### M3 — renewable resources, condition and condition-mediated mortality

Maintains dynamic cell food stock, regeneration, household/cell demand, supply allocation, individual condition response and configured condition-mediated survival mechanism. Quantities are abstract units unless a future evidence-grounded configuration establishes a defensible unit mapping.

Under the v8 time/accounting contract, resource-period boundaries are exact half-open integer-day intervals. Fixed annual integer quantities are allocated by cumulative elapsed days so the complete-year total is conserved. The synthetic seasonal curve redistributes unconstrained annual regeneration potential through those intervals using integrated daily weights rather than a single endpoint sample. An interval with zero executable demand is condition-neutral rather than a free recovery event.

Under v9 timing semantics, the condition recovery/loss coefficients and the condition-dependent mortality probability are explicit reference-quarter quantities converted over the actual elapsed M3 interval. At fixed supply/condition, changing only `resources.periodsPerYear` does not multiply the annual condition-response budget or fixed-condition survival probability. Under v10 the public resource parameter is `maxConditionMortalityProbabilityPerMillion`, and a death produced by this shared-condition hazard is `condition_mediated`, not `resource_scarcity`.

The v10 causal contract treats `condition` as a shared synthetic health/energetic mediator. Current pathways include founder/newborn condition, M3 resource supply/recovery/shortfall and M4 permanent-travel condition cost. Resource shortage can therefore increase condition-mediated mortality by lowering condition, but the death event does not preserve enough provenance to claim scarcity was the unique cause. Issue #200 is repaired by this semantic boundary; coincident M3/M2 competing-risk attribution remains separately tracked under #208.

Primary specifications: [`resources-v0.1.md`](resources-v0.1.md), [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md), [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md), [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).  
Primary implementation: `crates/anthrosim-core/src/resources.rs` and resource/config/scheduler code.

### M4 — permanent household migration

Evaluates pressured households against bounded local alternatives on an independent fixed annual decision schedule controlled by `migration.decisionPeriodsPerYear` (synthetic default four/year). The explicit stay counterfactual contains residence-state resource/water/kin terms only; relocation candidates evaluate their destination residence terms and then subtract relocation-only travel/terrain, uncertainty and relocation-risk costs before the minimum-improvement and stochastic weighted-choice steps.

The resource-support term uses annual per-person need allocated over the current M4 decision interval using the same cumulative elapsed-day annual-allocation rule as M3. M4 therefore does not depend on the number of M3 resource boundaries for its opportunity count or demand denominator. Selected moves change persistent residence and can reduce the shared condition state through the configured travel-condition cost.

Primary specifications: [`migration-v0.1.md`](migration-v0.1.md), [`m4-kin-proxy-v1.md`](m4-kin-proxy-v1.md), [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md), [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).  
Primary implementation: `crates/anthrosim-core/src/migration.rs`, `crates/anthrosim-core/src/simulation.rs`, `crates/anthrosim-core/src/spatial_simulation.rs` and related spatial/migration code.

### M5–M7 — observability and experiment infrastructure

Authoritative events, metrics, checkpoints, deterministic replay, run bundles, ensembles, retries and sweeps make the scientific unit inspectable/reproducible but do not add anthropological mechanisms.

Primary experiment contract: [`../experiments-v0.1.md`](../experiments-v0.1.md).

### M8 — evidence-grounded spatial execution

Binds normalized spatial evidence to declared deterministic model-facing transformations and residence-based spatial observability while preserving source/transformation/result separation. Under v10, per-cell mortality observability uses condition-mediated terminology; it does not reinterpret that death count as a resource-specific causal count.

Primary specifications: [`spatial-mechanisms-v1.md`](spatial-mechanisms-v1.md), [`spatial-observability-v1.md`](spatial-observability-v1.md).

### M9 — temporary mobility and aggregation

Separates persistent residence from temporary physical presence, adds focal regions and duration-bearing journeys, and records duration-aware resource attribution plus temporary-presence observability.

Primary specifications: [`temporary-mobility-v1.md`](temporary-mobility-v1.md), [`m9-temporary-travel-semantics-v1.md`](m9-temporary-travel-semantics-v1.md), [`m9-duration-aware-resource-semantics-v1.md`](m9-duration-aware-resource-semantics-v1.md), [`temporary-mobility-observability-v1.md`](temporary-mobility-observability-v1.md).

### Model evaluation

ODD 2020 encourages the model's fitness for purpose to be made explicit. AnthroSim keeps the evaluation record in the TRACE dossier rather than pretending evaluation is part of the mechanism definition itself. See [`trace.md`](trace.md).

---

## ODD completeness declaration

For the v0.3.0 package baseline and subsequent model-semantics hardening line, this document explicitly covers all seven ODD 2020 elements and all eleven standard design concepts. A concept that is absent from the model (for example learning) is documented as absent rather than omitted silently.

This completeness declaration means **the model is formally described under ODD**. It does **not** mean the behavioural model has passed empirical validation.

## Reference

Grimm, V. et al. (2020). *The ODD protocol for describing agent-based and other simulation models: A second update to improve clarity, replication, and structural realism.* Journal of Artificial Societies and Social Simulation 23(2):7. DOI: `10.18564/jasss.4259`.
