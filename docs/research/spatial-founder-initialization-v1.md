# Spatial founder initialization contract v1

**Status:** normative scientific contract for spatial founder initialization  
**Applies to:** post-M9 scientific-hardening line  
**Issue repaired:** #213  
**Scientific status:** initialization/research-design verification; not empirical settlement validation

## 1. Purpose

An evidence-grounded landscape does not, by itself, make the population that begins on that landscape evidence-grounded.

Before this repair, `SpatialLandscapeSimulation` reused the synthetic founder initializer. Founder households were therefore placed by a deterministic uniform random draw over world cells even when the landscape itself was provenance-bound to external evidence. That was useful for engineering tests, but it could silently make an arbitrary starting residence pattern part of a purportedly empirical spatial experiment.

Initial residence can affect later resource pressure, co-residence, migration, kin geography, temporary-mobility catchment, aggregation and settlement concentration. Because AnthroSim contains feedback and path dependence, those effects cannot be assumed to disappear merely because a run is long.

This contract makes founder residence an explicit scientific choice and defines what is required before a study may treat later spatial results as insensitive to that choice.

## 2. Relationship to the M2 founder contract

`docs/research/m2-founder-initialization-contract-v1.md` defines the complete day-0 founder population state: who the founders are, their households, ages, reproductive state, direct-parent state, condition and household residences.

This spatial contract does not create a second founder representation. It reuses that same `FounderPopulationDefinition` and adds the spatial research rule around its household residences.

In plain terms:

- the M2 founder contract defines **who exists at day 0 and what pre-run state is declared**;
- this contract defines **where those households begin on the modeled landscape and how that starting geography may be used in inference**.

The two contracts therefore form one population-initialization boundary without conflating demographic prehistory with spatial initial-condition uncertainty.

## 3. Supported spatial initialization modes

### 3.1 `synthetic_validation_v1`

The existing synthetic initializer remains available and unchanged.

For household residence it uses the frozen deterministic synthetic rule that draws locations over the available world cells from the bound population-realization RNG stream. This mode is appropriate for:

- software verification;
- deterministic regression tests;
- synthetic/null-model experiments;
- experiments whose question explicitly includes random founder placement as a modeled assumption.

Uniform random founder placement is **not** a neutral archaeological prior and is not an inferred settlement distribution. An evidence-grounded landscape does not upgrade this initializer into an evidence-grounded population merely because the two are executed together.

### 3.2 `declared_founder_state_v1`

A spatial run may instead use the existing `FounderPopulationDefinition`.

Each declared household residence is materialized exactly against the transformed authoritative `World`. Founder ages, reproductive sex, household membership, genealogy, condition and pre-run reproductive history are materialized by the same declared-founder contract used by ordinary core simulation.

This is a research-capable **initial-state transport mechanism**: it can faithfully execute a documented founder hypothesis. It does not claim that the hypothesis is true, at equilibrium, representative of the past or sufficiently evidenced merely because it is structurally valid.

## 4. Provenance and identity

Spatial founder initialization is not hidden behind the master seed.

The preserved experiment identifies the initial state through:

- `ExperimentConfig.population.initialization`;
- the complete `founderPopulation` when declared mode is used;
- the founder definition `schemaVersion`;
- `initializationId`;
- declared `ParameterProvenance`;
- genealogy-completeness status;
- exact household IDs and residence cells;
- exact person-level founder state;
- deterministic `contentDigest64` when serialized;
- the immutable experiment/run identity that contains the resulting `ExperimentConfig`.

For spatial ensemble and sweep orchestration, the declared founder definition is also part of the immutable spatial run settings used to build those exact experiment configurations.

This contract composes with `docs/research/spatial-realization-seed-separation-v1.md` from #212. Spatial runs preserve separate identities for:

- environment/world realization;
- stochastic population realization;
- dynamic process realization.

In `synthetic_validation_v1`, the population-realization seed is causal for the generated founder state, including residence.

In `declared_founder_state_v1`, the population-realization seed is still recorded as part of the spatial realization provenance but is **non-causal for founder materialization**. Changing that seed while holding the declared founder definition and world fixed must not change the initialized population digest.

## 5. Spatial execution and fail-closed validation

`SpatialLandscapeSimulation` applies the same founder-mode binding rules as the core simulation host.

It rejects:

- declared initialization without a founder definition;
- synthetic initialization carrying a founder definition;
- invalid founder counts, IDs, chronology, households or locations;
- a declared household location outside the reconstructed spatial world;
- declared genealogy marked `unspecified` when active permanent migration assigns non-zero weight to the direct-parent kin proxy.

Declared founder locations are validated against the **transformed authoritative world used by that run**, not merely against a source file before spatial transformation.

Checkpoint resume revalidates the founder definition against the reconstructed world. Completed spatial-run validation does the same. A preserved provenance label therefore cannot excuse a founder definition that is no longer valid for the world to which the run claims to be bound.

## 6. Spatial ensemble and sweep execution

Spatial `ensemble` and `sweep` commands accept `--founder-population <file>` together with the spatial landscape/mechanism inputs.

When supplied:

- the founder file is parsed as `FounderPopulationDefinition`;
- its exact person count becomes the configured initial population;
- the definition is preserved in the immutable spatial experiment settings;
- every generated run receives the same exact founder definition unless a separate experiment is deliberately defined;
- the resulting `ExperimentConfig` selects `declared_founder_state_v1` automatically.

A declared founder file conflicts with `--sweep-population` and `--sweep-household-size`. Those dimensions describe the synthetic founder generator; allowing them alongside an exact declared population would create sweep coordinates that appear scientifically varied while having no causal meaning.

Alternative plausible founder states can be run as separate immutable ensembles/sweeps and compared as structural initial-condition alternatives. A more general single-plan experiment language that treats arbitrary structural alternatives as first-class Cartesian dimensions remains coordinated with #205.

## 7. Initial-condition sensitivity

A real spatial study must decide whether founder geography is:

1. a substantive initial-state hypothesis whose uncertainty is carried into the result; or
2. a transient nuisance state that demonstrably ceases to control the target inference before the analysis window.

AnthroSim must not choose between these interpretations silently.

If more than one founder layout is plausible, the study should preserve those alternatives as distinct initialization identities and compare them while controlling the other uncertainty dimensions appropriately.

For example, a paired design may hold constant:

- landscape/environment realization;
- model configuration;
- process seed pairing;
- duration and observation schedule;

while varying only the declared founder layout. Conversely, a study can repeat each founder layout over the same set of process seeds to estimate interaction between initialization uncertainty and stochastic history.

The resulting variation is **initial-condition/structural uncertainty**, not ordinary process-seed variance.

## 8. Burn-in is a validation claim, not a magic duration

AnthroSim does not define a universal burn-in length.

A burn-in period is scientifically defensible only when the study demonstrates that the observables used for its final claim have become sufficiently insensitive to the plausible initial states under the active mechanisms.

A predeclared burn-in/convergence analysis should specify at least:

- the alternative founder states being compared and why they span a relevant uncertainty range;
- which environment realization(s) are held fixed or varied;
- how process seeds are paired or replicated;
- candidate burn-in checkpoints or time windows;
- the spatial observables used to judge convergence;
- the tolerance or decision criterion defined before inspecting the result;
- the later analysis window that will be treated as inferentially usable if the criterion is met.

Useful target observables may include, depending on the research question:

- cell-level occupancy and living person-days;
- largest-cell population share;
- spatial concentration/Herfindahl measures;
- migration frequency, direction, distance and origin-destination flows;
- cell resource pressure and condition-related outcomes;
- local persistence/extinction;
- M9 origin catchment, reachability, participation and visitor person-days when temporary mobility is active.

Convergence of one summary does not prove convergence of every causal or inferentially relevant quantity. The criterion must be tied to the actual claim the study intends to make.

## 9. What to do when convergence fails

Failure to converge is a scientific result, not an engine error.

If plausible founder states continue to produce materially different target outcomes at the proposed analysis boundary, the study must not discard that dependence by declaring a longer run to be "burned in" without further evidence.

Instead, founder spatial state remains a consequential uncertainty. The study should report or propagate it, narrow it using defensible evidence, change the research claim, or introduce a justified prehistory/equilibrium procedure whose own assumptions are then tested.

The controlled #213 regression intentionally demonstrates this principle. When migration, births, deaths and resource pressure are disabled, two different founder layouts remain different after two simulated years. There is no relaxation mechanism that could make them converge. The passage of model time alone therefore cannot erase initial-condition dependence.

## 10. Evidence and empirical interpretation

A declared founder residence with `empirical_direct`, `empirical_derived` or `evidence_informed` provenance is not automatically proven by that label.

For a real study, the researcher should preserve the evidence or reasoning used to derive residence assumptions, including where relevant:

- known or inferred occupation locations;
- chronological compatibility with the modeled start boundary;
- settlement-permission/exclusion assumptions;
- sampling or reconstruction procedure;
- spatial uncertainty and alternative placements;
- whether the same archaeological evidence is later used for validation, to avoid circularity.

Machine-verifiable closure between empirical provenance claims and evidence records remains part of the broader evidence-readiness work tracked by #181.

## 11. Verification evidence required by this contract

The implementation must retain regression evidence demonstrating at least:

- synthetic spatial founder initialization remains available and retains its established deterministic behavior when declared mode is unused;
- exact declared household residences are materialized in the transformed world;
- spatial observability reports those exact day-0 residence counts;
- changing only the stochastic population-realization seed does not change a declared founder population;
- invalid/out-of-world declared residences fail closed;
- checkpoint/resume and recorded-run validation revalidate founder definitions against the reconstructed world;
- the spatial ensemble/sweep CLI preserves the exact founder definition in immutable experiment identity;
- a controlled case exists in which alternative founder layouts demonstrably do not converge, preventing an undocumented assumption that burn-in always removes initialization effects.

These tests verify implementation and research-design semantics. They do not validate a particular archaeological settlement reconstruction.

## 12. What this repair does not solve

This contract deliberately does not introduce:

- an automatic archaeological settlement-inference algorithm;
- a landscape-weighted founder-placement heuristic presented as realistic by default;
- an equilibrium population generator;
- an automatic burn-in detector;
- a universal number of years to discard;
- proof that any declared founder state is historically correct;
- full evidence closure for empirical founder claims;
- the complete generic structural-sensitivity experiment language tracked by #205.

Those capabilities would require their own assumptions, evidence and validation. Hiding them inside #213 would replace one arbitrary starting rule with another.

## 13. Research-use rule

Before a spatial result is interpreted as a response to an evidence-grounded landscape, the study must be able to answer:

> **Why is the day-0 founder geography defensible, or what evidence shows that the reported conclusion no longer materially depends on the plausible alternatives?**

A study therefore needs one of two defensible paths:

1. use an explicit provenance-bearing declared founder state and carry plausible founder-state uncertainty into sensitivity/interpretation; or
2. use a predeclared burn-in/convergence analysis demonstrating that the target inference is insensitive to the relevant alternative founder states before the reported analysis window.

Uniform synthetic founder placement remains a legitimate null model, but it cannot silently substitute for either path in an evidence-grounded archaeological inference.
