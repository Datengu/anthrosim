# ADR 0002: Determinism is a first-class contract

**Status:** Accepted  
**Date:** 2026-08-21

## Context

AnthroSim is intended for controlled experiments and causal inspection. A model that changes unrelated history because a developer inserted one random draw into another subsystem is difficult to debug and weak for reproducible research.

## Decision

All simulation randomness derives from an explicit master seed and named deterministic streams. The initial implementation uses a portable deterministic ChaCha stream generator. Ambient OS/thread randomness is forbidden in authoritative simulation paths.

System ordering is explicit. Parallel execution may be introduced only when its determinism boundary is documented and tested.

## Consequences

- experiment manifests record master seed and model/schema version;
- tests compare deterministic outputs/state digests;
- named streams isolate subsystems where practical;
- changing the RNG scheme is a model/reproducibility change and requires a versioned migration decision.
