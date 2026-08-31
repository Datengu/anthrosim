# Pull-request CI path classification

AnthroSim is introducing conservative path-aware CI under issue #317. This document defines the first, **advisory** classification layer. It does not yet suppress any globally required branch-protection check.

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

The classifier and its aggregator workflow are self-protecting: changing either forces `full` and forces all currently conditional M8.6, M9.7 and RustSec gates. This prevents the classification policy from weakening its own review surface.

## Fail-safe rules

The classifier rejects an empty changed-file set, absolute paths, parent traversal and empty path entries. Pull-request classification retrieves the complete rename-aware changed-file set from the GitHub API and fails when the retrieved count disagrees with pull-request metadata.

Renames are classified using both the new and previous path. A rename from a sensitive path therefore cannot evade a gate merely because its destination looks harmless.

Unknown paths are never guessed into a cheaper class. They fall back to `full`.

## Current enforcement boundary

The three risk classes are **advisory groundwork only** at this stage. They are surfaced and tested so later #317 work can condition heavy jobs on a reviewed, stable classification without simultaneously inventing classification semantics and changing branch-protection behavior.

Until a later reviewed change explicitly wires required workflows to these classes, all existing globally required checks continue to run exactly as before. Gate-specific M8.6, M9.7 and RustSec applicability remains enforced independently by the existing protected `Applicable scientific/security gates` context.

Any future optimization must preserve the exact required context names in `docs/required-status-checks.md`, must make skipped heavy work resolve through an explicit successful/N/A disposition rather than absent checks, and must retain `full` as the fallback for mixed or ambiguous changes.
