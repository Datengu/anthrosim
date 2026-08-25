# CI validation topology

AnthroSim separates validation by responsibility so cheap correctness failures are reported before expensive research and performance work consumes runner time. Workflow files are kept independent where a failure represents a distinct research-engineering contract; this document describes those responsibilities rather than duplicating their implementation steps.

## Main CI workflow

The `CI` workflow (`.github/workflows/ci.yml`) owns the general Rust/application quality and scale gates:

1. **Quality and tests** — locked dependency metadata, formatting, Clippy, and the full Rust workspace test suite.
2. **Explorer and script validation** — JavaScript/Python syntax checks and explorer model tests.
3. Once both cheap gates pass, **Core benchmarks** and **Release build** run independently.
4. The release build is reused by the focused downstream gates for the 1000-run ensemble soak, performance/memory acceptance, the canonical M7.6 reference experiment, and M5/M6 bundle integration.

The dependency graph prevents formatting, lint, unit-test, or script failures from spending time on the expensive acceptance stages while keeping the release binary identical across downstream gates.

## Independent reproducibility and research-artifact workflows

The following workflow families remain separate from the main CI graph because each protects a distinct contract:

- **Cross-platform determinism** (`cross-platform-determinism.yml`) — requires normalized authoritative outputs from both the compact core golden fixture and a compact enabled-M9 temporary-mobility fixture to agree byte-for-byte across Ubuntu, Windows and macOS. The M9 sentinel exercises departure, arrival, visiting, return, completion, temporary observability, and checkpoint/resume through an active journey; it complements rather than duplicates the larger Ubuntu M9.7 scientific benchmark.
- **Landscape preprocessing** (`landscape-preprocessing.yml`) — checks deterministic normalization/preprocessing of declared landscape inputs.
- **Landscape loading determinism** (`landscape-loading.yml`) — verifies normalized landscape loading and landscape-bound execution remain deterministic and correctly bound.
- **Spatial mechanism determinism** (`spatial-mechanisms.yml`) — validates versioned landscape-to-model transformations, mechanism identity and transformed spatial execution.
- **Spatial observability** (`spatial-observability.yml`) — regenerates/validates downstream spatial observability, including resumed spatial histories and tamper rejection.
- **Resumed Explorer compatibility** (`resumed-explorer.yml`) — proves new-directory resumed core and transformed-spatial bundles contain true day-zero founders, retain resume-boundary provenance, reconcile through M6, and are served read-only.
- **Run bundle pack** (`run-bundle-pack.yml`) — validates semantic pack acceptance/rejection and deterministic canonical ZIP output for shareable completed run bundles.
- **Source provenance** (`source-provenance.yml`) — checks clean/dirty/override source identities and the exact-binary provenance preflight used by versioned research sweeps.
- **Dependency advisory audit** (`dependency-audit.yml`) — runs pinned `cargo-audit` 0.22.2 against `Cargo.lock` for dependency-changing pull requests/pushes, every day, and on demand so newly disclosed RustSec vulnerabilities are surfaced even without a source-code change.
- **M8 benchmark data** (`m8-benchmark-data.yml`) — validates the committed benchmark input/data provenance contract independently of simulation output.
- **M8.6 evidence-grounded spatial benchmark** (`m8-spatial-benchmark.yml`) — executes and checks the declared evidence-grounded reference benchmark/reproduction path.
- **M9.7 controlled aggregation benchmark** (`m9-aggregation-benchmark.yml`) — reruns the frozen paired continuous-residence/intermittent-aggregation benchmark, verifies the preserved scientific reference, independently replays focal-region occupancy from authoritative events, proves exact duplicate replay, checks active-journey annual checkpoint/resume equivalence, and rejects deliberate reference tampering.

These workflows complement rather than replace semantic validation inside the binaries. A green workflow shows that the corresponding contract passed in CI; artifact readers and research commands still validate the artifacts they consume.

When a new independent workflow becomes part of the research-integrity or artifact contract, update this topology in the same change so the documented gate set does not drift from `.github/workflows/` again.

## Protected `main` gate set

Workflow existence and branch-protection enforcement are separate contracts. The exact status contexts that are intended to be required on `main`, the administrator-bypass policy and the deliberately non-required/path-filtered jobs are recorded in [`required-status-checks.md`](required-status-checks.md).

Any change that adds, removes or renames an independent correctness, determinism, provenance, artifact-integrity or research-reproducibility job must review both this topology and the protected-main contract. The workspace test `required_status_checks_contract` checks that the documented required names still correspond to the current workflow job names and matrix operating systems; the live GitHub branch rule must additionally be verified after administrative changes.

## Supply-chain rule

Every third-party GitHub Action is pinned to an immutable full commit SHA with a human-readable release comment. Updates follow the reviewed process in `CONTRIBUTING.md`; mutable tags and branches are not accepted as CI dependencies.

Rust dependency advisory scanning follows the same reproducibility principle: `cargo-audit` is installed at an exact reviewed version with `cargo install --locked`, while the advisory database itself is intentionally refreshed at execution time. A bare audit treats applicable vulnerability advisories as failures; informational/unmaintained/unsound/notice and yanked-crate warnings are surfaced for maintainer assessment rather than automatically forcing dependency churn. Any remediation still goes through the ordinary deterministic scientific regression suite.
