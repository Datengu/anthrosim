# Pull-request CI path classification

AnthroSim is introducing conservative path-aware CI under issue #317. The classifier is now enforced by the central CI, cross-platform determinism, spatial-mechanism, landscape-loading, landscape-preprocessing, run-bundle-pack and resumed-Explorer-compatibility workflows while the remaining required workflows continue to run their existing full behavior.

## Why this exists

Audit-ledger and documentation-only pull requests should not eventually need the same simulation, release and deterministic-golden workload as executable scientific changes. Any optimization must nevertheless fail safe: a mixed or unknown path set must continue to receive the full validation surface, and required GitHub status contexts must still resolve rather than disappearing behind workflow-level path filters.

The versioned classifier is `scripts/classify-applicable-pr-gates.py`. The always-present `Applicable scientific/security gates` workflow validates the classifier on every pull request and records the resulting class in its job summary.

## Classes

### `audit_status_only`

Every changed path is under:

- `docs/research/audit-v2/**`

This class is intentionally narrow. It covers the audit-v2 ledger, handoff state and dedicated audit evidence stored below that directory.

### `scientific_documentation_only`

Every changed path is within the scientific-documentation surface:

- `docs/scientific-model.md`
- `docs/research-integrity.md`
- `docs/research-principles.md`
- `docs/research/**`

An audit-v2-only change is classified more specifically as `audit_status_only`; a mixture of audit-v2 files and other scientific documentation is `scientific_documentation_only`.

This class does **not** mean all scientific gates are irrelevant. Existing gate-specific rules remain authoritative. For example, an M9 benchmark contract/result document can still make the M9.7 gate applicable even while the overall PR is documentation-only.

### `full`

Any mixed, executable, workflow, script, configuration, benchmark/reference, unknown or otherwise unclassified path set receives `full`.

The classifier and its aggregator workflow are self-protecting: changing either forces `full` and forces all currently conditional M8.6, M9.7 and RustSec gates. This prevents the classification policy from weakening its own review surface. Other workflow files are unclassified paths, so a PR changing them also falls back to `full`.

## Fail-safe rules

The classifier rejects an empty changed-file set, absolute paths, parent traversal and empty path entries. Pull-request classification retrieves the complete rename-aware changed-file set from the GitHub API and fails when the retrieved count disagrees with pull-request metadata.

Renames are classified using both the new and previous path. A rename from a sensitive path therefore cannot evade a gate merely because its destination looks harmless.

Unknown paths are never guessed into a cheaper class. They fall back to `full`.

## Current enforcement boundary

`CI`, `Cross-platform determinism`, `Spatial mechanism determinism`, `Landscape loading determinism`, `Landscape preprocessing`, `Run bundle pack` and `Resumed Explorer compatibility` are the required workflows currently wired to the reviewed risk classes. On pull requests classified `audit_status_only` or `scientific_documentation_only`, their protected execution contexts still run and resolve successfully, but irrelevant executable work reports an explicit documentation-only N/A disposition instead of running simulation, release, preprocessing, bundle, resume-Explorer or deterministic-golden fixtures.

The central `CI` workflow keeps `Quality and tests` present for documentation-only changes but replaces the full Rust workspace build/test surface with lightweight documentation consistency checks. Those checks verify the living model-semantics documentation against executable provenance and validate the basic audit-v2 ledger/document structure. The other protected central-CI contexts resolve explicitly as N/A for documentation-only changes. Non-required core benchmarks, performance/memory acceptance and the 1000-run ensemble soak do not run for those classes.

The affected protected contexts are:

- `Quality and tests`
- `Explorer and script validation`
- `Release build`
- `M5/M6 bundle integration`
- `Canonical M7.6 reference experiment`
- `Golden run (ubuntu-latest)`
- `Golden run (windows-latest)`
- `Golden run (macos-latest)`
- `Compare cross-platform golden runs`
- `Spatial mechanism golden (ubuntu-latest)`
- `Spatial mechanism golden (windows-latest)`
- `Spatial mechanism golden (macos-latest)`
- `Spatial M7 sweep integration`
- `Compare transformed landscape golden runs`
- `Landscape golden run (ubuntu-latest)`
- `Landscape golden run (windows-latest)`
- `Landscape golden run (macos-latest)`
- `Compare landscape golden runs`
- `M8.2 preprocessing validation`
- `Deterministic completed-run ZIP`
- `New-directory resume Explorer compatibility`

For `full` changes, for every push to protected `main`, and therefore for any PR that changes executable/scientific machinery, an unknown path, or any optimized workflow itself, the existing executable validation remains unchanged. Each optimized workflow validates the classifier before deciding which disposition applies; incomplete GitHub changed-file retrieval fails classification instead of degrading to the cheaper path.

All other globally required checks continue to run exactly as before until they receive equivalent reviewed N/A handling. Gate-specific M8.6, M9.7 and RustSec applicability remains enforced independently by the existing protected `Applicable scientific/security gates` context.

Future #317 optimizations must preserve the exact required context names in `docs/required-status-checks.md`, make skipped heavy work resolve through an explicit successful/N/A disposition rather than absent checks, and retain `full` as the fallback for mixed or ambiguous changes.
