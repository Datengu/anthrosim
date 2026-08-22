# CI validation topology

AnthroSim separates validation by responsibility so cheap correctness failures are reported before expensive research and performance work consumes runner time.

## Main CI workflow

The `CI` workflow uses the following dependency graph:

1. **Quality and tests** — locked dependency metadata, formatting, Clippy, and the full Rust workspace test suite.
2. **Explorer and script validation** — JavaScript/Python syntax checks and explorer model tests.
3. Once both cheap gates pass, **Core benchmarks** and **Release build** run independently.
4. The release build is uploaded as a one-day workflow artifact and reused by four focused downstream gates:
   - **1000-run ensemble soak**;
   - **Performance and memory acceptance**;
   - **Canonical M7.6 reference experiment**;
   - **M5/M6 bundle integration**.

The downstream jobs restore the downloaded release binary's executable bit before use. The scientific checks themselves are unchanged from the original monolithic v0.1 CI workflow: the same benchmark commands, performance thresholds, reference results, bundle checks, and memory ceiling remain authoritative.

Splitting the jobs makes a failing category independently rerunnable and prevents a formatting, lint, unit-test, or script-validation failure from starting the expensive validation stages.

## Cross-platform determinism

`.github/workflows/cross-platform-determinism.yml` remains a separate focused workflow. It runs the compact authoritative golden fixture on Ubuntu, Windows, and macOS, then requires the normalized scientific outputs to be byte-identical. Keeping this gate separate makes platform-specific failures visible without replicating the full Linux research/performance suite on every operating system.

## Supply-chain rule

Every third-party GitHub Action is pinned to an immutable full commit SHA with a human-readable release comment. Updates follow the reviewed process in `CONTRIBUTING.md`; mutable tags and branches are not accepted as CI dependencies.
