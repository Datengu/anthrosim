# Property-style invariant testing

AnthroSim complements its hand-written deterministic and directional tests with a bounded generated property suite in `crates/anthrosim-core/tests/property_invariants.rs`.

## Why a bounded generator

The v0.1 authoritative model is deterministic and already carries strong cross-artifact invariant validation. For the pre-M8 hardening step, the property suite therefore uses an in-repository deterministic Cartesian generator rather than adding a random fuzzing dependency. This keeps `Cargo.lock` unchanged, CI runtime predictable and every failure directly reproducible.

The generated domain varies seed, world shape, founder population, target household size, resource magnitude, seasonal amplitude, migration enablement and migration radius. Every generated valid configuration is run through the ordinary `Simulation` path and the existing cross-system invariant validator.

Cases are sorted lexicographically from smaller to larger values. A failure message prints the complete generated tuple, and iteration stops at the first failing tuple. Within the declared generated domain, this provides a deterministic minimal reproducer without a separate stochastic shrinking engine.

## Properties covered

The suite currently checks that:

- generated valid population/resource/migration combinations complete only through an explicit supported stop reason and pass the full cross-artifact invariant validator;
- running the same generated configuration and seed twice produces identical authoritative manifest and checkpoint state;
- a representative generated subset preserves exact checkpoint/resume equivalence against uninterrupted execution;
- out-of-range resource and seasonality scales are rejected instead of being silently clamped or normalized.

The existing invariant validator already reconciles living people and households, references and genealogy, births/deaths, resource accounting, migration bounds/counters, event/metric provenance and checkpoint state. Running that validator over generated configuration families broadens state-space coverage without duplicating those assertions in a second implementation.

## Scope and evolution

This suite does not replace named behavioral/scientific tests. Directional claims still require explicit tests whose assumptions and expected causal direction are reviewable.

When M8 or later modules introduce materially larger input spaces, the generator should expand around stable new invariants. If deterministic bounded generation no longer gives sufficient coverage, a dedicated property-testing/fuzzing dependency can be introduced deliberately with a locked version and a reproducible shrinking/corpus policy.
