# Cross-platform determinism boundary (v0.1)

AnthroSim treats deterministic execution as part of its research contract. In addition to same-seed repeatability on one host, CI now exercises one compact authoritative fixture on Linux, Windows, and macOS and requires the normalized scientific output to be byte-identical across all three runners.

## What is compared

`crates/anthrosim-core/tests/cross_platform_golden.rs` runs the same fixed experiment twice on each host and exports a normalized `CrossPlatformGoldenRun` containing:

- the completed run manifest and its scientific summaries/statistics;
- the final deterministic state digest and component digests;
- the complete authoritative event log;
- the complete metric series;
- the final checkpoint, including population, resource, migration, RNG, event, metric, and deterministic-state data.

The fixture deliberately clears `gitCommit` before serialization. Exact Git revision is source provenance rather than simulated state, and the matrix already executes one common revision. No paths, timestamps from the host clock, runner names, or other operating-system metadata are included in the comparison.

The CI matrix uploads the normalized JSON from `ubuntu-latest`, `windows-latest`, and `macos-latest`. A final comparison job requires all three files to be byte-identical. The common output produced by that revision is the cross-platform golden identity.

## What this proves

A passing matrix is evidence that the tested authoritative v0.1 execution path does not currently depend on operating-system-specific integer behaviour, iteration order, serialization, filesystem representation, or RNG behaviour for this representative fixture.

It is intentionally a small smoke test, not a claim that every possible configuration has been exhaustively compared across platforms. The existing deterministic tests, invariant/soak tests, performance acceptance, and canonical experiment reproduction remain responsible for their separate validation boundaries.

## Golden identity versus model evolution

The cross-platform job compares platforms against each other for the same source revision; it is not a permanent historical-output lock. A deliberate authoritative model-semantics change may legitimately change the fixture output, but all supported platforms must still agree on the new output. Historical scientific compatibility is handled separately by schema/model-semantics provenance and versioned reference experiments.

If a platform differs, the failure should be treated as a reproducibility defect until the difference is either removed or explicitly documented as outside the supported determinism boundary.
