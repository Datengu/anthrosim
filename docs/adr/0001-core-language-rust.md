# ADR 0001: Rust for the simulation core

**Status:** Accepted  
**Date:** 2026-08-21

## Context

AnthroSim may eventually need to represent millions of persistent agents, execute long batch experiments, tightly control memory layout, parallelise deterministic workloads, and interoperate with native/GPU tooling. The first implementation choice should not require an early rewrite merely to escape runtime overhead or memory unpredictability.

## Decision

Use stable Rust (Rust 2024 edition) for the authoritative simulation core and headless CLI. Pin the project toolchain for reproducible builds. Produce simple versioned outputs that can be consumed by Python/R or other research tooling rather than putting scientific analysis in the hot simulation loop.

## Why

- predictable native performance and explicit memory layout;
- memory/thread safety without a garbage collector;
- strong type system for IDs, units, invariants, and state transitions;
- good support for data-oriented programming and controlled parallelism;
- mature C ABI / C++ interoperability options and a future path to GPU/native kernels;
- Cargo provides integrated builds, tests, docs, benchmarks, and dependency locking.

## Alternatives considered

### C++

Maximum ecosystem/HPC reach and direct CUDA integration, but substantially greater memory-safety burden and implementation risk for a project expected to evolve rapidly with AI-assisted contributions.

### C#/.NET

Excellent developer productivity and increasingly strong performance, but garbage-collection/runtime considerations are less attractive for the engine's most ambitious memory-density and low-level execution goals. C# remains viable for tooling or future UI components if useful.

### Python

Excellent scientific ecosystem and will likely be used for analysis. It is not selected for the authoritative hot loop because population-scale per-agent simulation would quickly depend on native extensions/vectorised kernels anyway.

## Consequences

- contributors need a Rust toolchain;
- FFI/Python bindings may be added later, but are not required for v0.1;
- unsafe Rust is not banned, but requires demonstrated need, narrow scope, safety documentation, and benchmarks;
- compiler/toolchain upgrades are deliberate changes, not automatic drift.
