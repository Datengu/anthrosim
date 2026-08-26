# Paired-seed semantics v1

## Status and scope

This document defines the scientific meaning of using the same AnthroSim master seed in two or more experiment arms under the current sequential named-stream RNG implementation.

It is an interpretation and study-design contract. It does **not** change authoritative simulation trajectories, RNG algorithms, stream labels, model semantics, benchmark references, or release identity.

## Contract

Under the current implementation, a shared master seed means:

1. each arm starts from the same master seed value;
2. each named stochastic subsystem derives its deterministic pseudo-random stream from that master seed according to the repository's versioned RNG contract;
3. an identical configuration, code/model identity, and seed replays exactly;
4. once two arms differ in configuration or state, the sequence position consumed by a named stream may diverge because many draws are conditional on state and eligibility.

Therefore **paired seeds are not an agent-level common-random-number guarantee**.

A study must not claim that two runs with the same seed are "the same stochastic history except for the treatment" unless the mechanism being studied has an additional, explicitly verified coupling that preserves the relevant variates after divergence.

## Why streams can decorrelate

AnthroSim uses deterministic named streams, but draws within those streams are generally consumed sequentially and only when the current state reaches the relevant decision.

Examples include:

- demographic mortality draws only for currently living people;
- fertility draws only after sex, age, spacing, and local-male eligibility checks;
- parentage/newborn-sex draws only after successful fertility;
- M4 uncertainty draws only for households/candidates that reach the migration decision;
- M4 destination-choice draws only when positive-weight alternatives exist.

If one arm causes an earlier death, changes eligibility, changes a candidate set, or otherwise changes whether a draw occurs, later records can consume different positions from the same named stream. The master seed remains identical, but the later random variates are no longer aligned to the same person/event.

## Permitted interpretation of paired seeds

For current AnthroSim studies, paired seeds may be used as a **replicate-blocking design**: arm A at seed `s` and arm B at seed `s` are intentionally compared as one pair because they share the same master-seed identity and whatever stochastic/environmental components remain common before or despite divergence.

This can be useful for variance reduction when the arms remain positively correlated, but the degree of correlation is an empirical property of the experiment. It must not be assumed to be strong, stable, or agent-level.

A paired-seed contrast therefore means:

> difference between two deterministic runs generated from the same master-seed identity under their respective model configurations.

It does **not** mean:

> outcome difference for an otherwise identical realized history in which every individual random shock has been held fixed.

## Analysis requirements

A confirmatory analysis using paired seeds should:

- preserve the exact seed identity for every arm and pair;
- state explicitly that current pairing does not guarantee agent/event-level shock alignment after state divergence;
- report the paired estimand and uncertainty method when treating pairs as the analysis unit;
- inspect the empirical correlation of paired outcomes rather than assuming pairing automatically improves precision;
- compare against an unpaired or independently seeded sensitivity analysis when conclusions could depend on the coupling choice;
- avoid interpreting one seed-pair trajectory as a literal counterfactual history of the same agents;
- distinguish process-replicate pairing from environment/initialization pairing once those uncertainty dimensions are separated under #212.

If paired and unpaired analyses materially disagree in uncertainty or conclusion, that dependence is part of the scientific result and should be reported rather than choosing whichever analysis is more favourable.

## What pairing does and does not control

The current contract is best summarized as follows:

| Quantity | Same master seed guarantees it is held common? |
| --- | --- |
| Master seed value | Yes |
| Deterministic replay within one unchanged arm | Yes |
| Named RNG stream definitions | Yes, subject to the same code/model identity |
| Stream position after treatment-induced state divergence | No |
| Random variate assigned to the same later person/event | No |
| Synthetic environment/initialization realization | Currently may be coupled to the master seed; see #212 |
| Agent-level counterfactual shock history | No |

The environment/initialization row is deliberately qualified. Issue #212 tracks separation of environment, initialization, and dynamic-process stochastic identities so a future study can state more precisely which uncertainty dimensions are fixed or varied.

## When stronger common-random-number semantics are required

Some model-analysis questions may benefit from a stronger coupling in which a draw is keyed to stable scientific identities rather than to mutable sequential stream position. A future optional mode could derive variates from keys such as:

```text
(master seed, mechanism identity, person/household identity, boundary/event identity, draw role)
```

Such a mode would require its own versioned stochastic semantics, collision/domain rules, determinism tests, and explicit statement of which interventions preserve a key. It must not silently replace the existing sequential-stream contract.

Until such a mode exists and is validated for the mechanism in question, AnthroSim paired seeds must not be described as agent-level common random numbers.

## Verification expectations

Research or benchmark work that relies on pairing should include a controlled diagnostic where useful:

1. run two otherwise similar arms with the same seed set;
2. introduce an intervention that changes an early eligibility/state transition;
3. inspect a later unrelated stochastic outcome or trace;
4. demonstrate whether later variates remain aligned or decorrelate under the current implementation;
5. record that behavior as part of the study's coupling assumptions.

This diagnostic is especially important before using pairing as a strong variance-reduction or counterfactual argument.

## Relation to other research-readiness work

- **#212** separates environmental/world, initialization, and dynamic-process stochastic identities.
- **#214** tracks the broader paired-seed/counterfactual semantics hardening; this document resolves the interpretation and analysis-guidance portion but does not by itself add keyed RNG mode or mechanism-level coupling tests.
- **#230** should preserve the chosen pairing/coupling policy in a frozen confirmatory study protocol.
- **#231** governs replicate sufficiency and Monte Carlo precision; deterministic seed pairing does not establish that the number of replicates is adequate.

## Research gate

Before paired-seed differences are presented as strong counterfactual evidence, the study must state exactly what the pairing holds common and must not imply agent/event-level shock alignment that the current sequential RNG design does not provide.

Ordinary reproducible ensemble means remain valid uses of deterministic seeds without making that stronger counterfactual claim.
