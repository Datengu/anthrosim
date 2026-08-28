# Paired-seed semantics v1

## Status and scope

This document defines the scientific meaning of using the same AnthroSim process seed in two or more experiment arms under the current sequential named-stream RNG implementation.

It is an interpretation and study-design contract. It does **not** change authoritative simulation trajectories, RNG algorithms, stream labels, model semantics, benchmark references, or release identity.

## Contract

Under the current implementation, a shared process seed means:

1. each arm starts from the same process-seed value;
2. each named stochastic subsystem derives the same deterministic pseudo-random stream from that seed according to the repository's versioned RNG contract;
3. an identical configuration, code/model identity, and seed replays exactly;
4. once two arms differ in configuration or state, the sequence position consumed by a named stream may diverge because many draws are conditional on state and eligibility.

Therefore **paired process seeds are a replicate-blocking/coupling design, not an agent-level common-random-number guarantee**.

A study must not claim that two runs with the same process seed are "the same stochastic history except for the treatment" unless the relevant mechanism has an additional, explicitly verified coupling that preserves the scientific variates after divergence.

## Why streams can decorrelate

AnthroSim uses deterministic named streams, but draws within those streams are generally consumed sequentially and only when the current state reaches the relevant decision.

Examples include:

- demographic mortality draws only for currently living people;
- fertility draws only after sex, age, spacing, and local-male eligibility checks;
- parentage/newborn-sex draws only after successful fertility;
- M4 uncertainty draws only for households/candidates that reach the migration decision;
- M4 destination-choice draws only when positive-weight alternatives exist.

If one arm causes an earlier death, changes eligibility, changes a candidate set, or otherwise changes whether a draw occurs, later records can consume different positions from the same named stream. The process seed and named stream remain identical, but the later random variates are no longer aligned to the same person or event.

The regression `conditional_consumption_breaks_agent_level_paired_seed_alignment` locks this behavior down deliberately: an early agent consumes a mortality-stream variate in one arm but not the other, causing a later unrelated agent to receive a different stream position.

## Relation to separated realization identities

Issue #212 separated spatial stochastic realization roles. In explicit-split spatial mode:

- `runRealization.environmentSeed` selects the environment/world realization;
- `runRealization.populationSeed` selects stochastic founder/population initialization;
- `ExperimentConfig.seed` is the dynamic process seed.

This means a paired-treatment study can now state which uncertainty dimensions are held common independently of process-seed pairing.

For example, a fixed-environment, fixed-founder, paired-process design can use the same `environmentSeed`, `populationSeed`, and process seed in both arms. That guarantees common environment and founder realizations plus common named process-stream definitions. It still does **not** guarantee that the same later agent/event receives the same process variate after the treatment changes state-dependent stream consumption.

Spatial configurations that omit `runRealization` retain the historical joint-process-seed compatibility mode documented in `spatial-realization-seed-separation-v1.md`; studies needing clean counterfactual interpretation should prefer explicit split identities.

## Permitted interpretation of paired seeds

For current AnthroSim studies, paired process seeds may be used as a **replicate-blocking design**: arm A at process seed `s` and arm B at process seed `s` are intentionally compared as one pair because they share the same process-seed identity and whatever explicitly fixed environment/initialization identities the study declares.

This can be useful for variance reduction when paired outcomes remain positively correlated, but the degree of correlation is an empirical property of the experiment. It must not be assumed to be strong, stable, or agent-level.

A paired-seed contrast therefore means:

> difference between two deterministic runs generated from the same declared realization identities and process-seed identity under their respective model configurations.

It does **not** mean:

> outcome difference for an otherwise identical realized history in which every individual random shock has been held fixed.

## Analysis requirements

A confirmatory analysis using paired process seeds should:

- preserve the exact process-seed identity for every arm and pair;
- preserve and report environment and population-initialization realization identities where relevant;
- state explicitly that current pairing does not guarantee agent/event-level shock alignment after state divergence;
- report the paired estimand and uncertainty method when treating pairs as the analysis unit;
- inspect the empirical correlation of paired outcomes rather than assuming pairing automatically improves precision;
- compare against an unpaired or independently process-seeded sensitivity analysis when conclusions could depend on the coupling choice;
- avoid interpreting one seed-pair trajectory as a literal counterfactual history of the same agents.

For a paired difference, uncertainty should be estimated from the distribution of within-pair differences when pairing is part of the design. If pairing produces little or unstable correlation, an unpaired design may be equally or more efficient and can be easier to interpret. Paired and unpaired analyses target the same marginal arm comparison only when the study's estimand and sampling design otherwise match; the uncertainty estimator must match the actual design rather than being selected post hoc for a preferred result.

If paired and unpaired analyses materially disagree in uncertainty or conclusion, that dependence is part of the scientific result and should be reported.

## What pairing does and does not control

| Quantity | Same declared paired identity guarantees it is held common? |
| --- | --- |
| Process seed value | Yes |
| Deterministic replay within one unchanged arm | Yes |
| Named process RNG stream definitions | Yes, subject to the same code/model identity |
| Explicit environment realization | Yes, if the same `environmentSeed` is declared |
| Explicit stochastic founder realization | Yes, if the same `populationSeed` is declared |
| Stream position after treatment-induced state divergence | No |
| Random variate assigned to the same later person/event | No |
| Agent-level counterfactual shock history | No |

## When stronger common-random-number semantics are required

Some model-analysis questions may benefit from a stronger coupling in which a draw is keyed to stable scientific identities rather than to mutable sequential stream position. A future optional mode could derive variates from keys such as:

```text
(process seed, mechanism identity, person/household identity, boundary/event identity, draw role)
```

Such a mode would require its own versioned stochastic semantics, key-domain/collision rules, determinism tests, and explicit statement of which interventions preserve a key. It must not silently replace the existing sequential-stream contract.

A keyed mode is therefore **not required for ordinary reproducible ensemble comparisons or replicate blocking**. It becomes required only if a study's scientific claim depends on preserving agent/event-level common random numbers after treatment-induced state divergence.

Until such a mode exists and is validated for the mechanism in question, AnthroSim paired seeds must not be described as agent-level common random numbers.

## Verification expectations

Research or benchmark work that relies on pairing should include a controlled coupling diagnostic when the strength of the pairing matters scientifically:

1. run two otherwise similar arms with the same declared realization identities;
2. introduce an intervention that changes an early eligibility/state transition;
3. inspect a later unrelated stochastic outcome or trace;
4. demonstrate whether later variates remain aligned or decorrelate under the current implementation;
5. record that behavior as part of the study's coupling assumptions.

The core RNG regression for #214 provides the repository-level minimal example of the expected decorrelation under conditional sequential consumption.

## Relation to other research-readiness work

- **#212** is complete and supplies separate environmental/world, initialization, and dynamic-process stochastic identities for spatial runs.
- **#230** preserves the chosen pairing/coupling policy in a frozen confirmatory study protocol.
- **#231** governs replicate sufficiency and Monte Carlo precision; deterministic seed pairing does not establish that the number of replicates is adequate.

## Research gate

Before paired-seed differences are presented as strong counterfactual evidence, the study must state exactly what the pairing holds common and must not imply agent/event-level shock alignment that the current sequential RNG design does not provide.

Ordinary reproducible ensemble means and paired replicate-blocking designs remain valid uses of deterministic seeds without making that stronger counterfactual claim.
