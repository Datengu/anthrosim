# ADR 0004: Separate model-semantics compatibility from source revision

## Status

Accepted.

## Context

A deterministic checkpoint can only be resumed scientifically if the executable continuing it implements compatible authoritative model semantics. AnthroSim previously persisted both the Cargo package version (`modelVersion`) and the exact source revision (`gitCommit`), but neither value expresses that contract precisely.

The package version is a software-release identifier. The Git commit is exact provenance. Requiring an identical Git commit for resume would be too strict because documentation, diagnostics, tooling, CI, or other source-neutral changes can leave simulation meaning unchanged. Conversely, allowing any source revision with the same package version can be too weak if authoritative model behaviour changes before the package version changes.

## Decision

AnthroSim defines a dedicated compatibility identifier, `MODEL_SEMANTICS_ID`, currently `anthrosim-model-semantics-v1`.

Checkpoint and run-manifest artifacts persist this value as `modelSemanticsId`. `Simulation::from_checkpoint(...)` compares the stored checkpoint value with the executable's current `MODEL_SEMANTICS_ID` and rejects the checkpoint when they differ.

The three identities have deliberately different meanings:

- `modelSemanticsId` answers **may this executable continue this authoritative model state without changing its scientific meaning?** It is the resume compatibility key.
- `gitCommit` answers **which exact source revision produced this artifact?** It remains provenance and is not, by itself, a resume gate.
- `modelVersion` answers **which packaged AnthroSim software version produced this artifact?** It remains software/version provenance and the existing compatibility guard; it is not a substitute for the semantics identity.

A change must bump `MODEL_SEMANTICS_ID` when it alters authoritative simulation meaning in a way that makes continuation of an existing checkpoint scientifically incompatible. Examples include changes to demographic transition rules, resource accounting semantics, migration decisions, event ordering that affects state evolution, or RNG consumption that changes authoritative trajectories.

A bump is not required solely for source-neutral changes such as documentation, CI configuration, explorer presentation, diagnostics that do not affect authoritative state, or refactoring proven to preserve the same model semantics and deterministic execution contract.

When uncertainty exists, prefer bumping the semantics identity and treating old checkpoints as incompatible rather than silently resuming across an unverified semantic boundary.

## Consequences

Checkpoint schema v4 and run-manifest schema v9 carry the explicit compatibility field. Older checkpoint schemas remain rejected through normal schema-version validation rather than being guessed compatible.

Exact Git provenance is preserved, so two artifacts can be distinguished by source revision even when they share a model-semantics identity. This permits source-neutral development between checkpoint creation and resume while making scientifically incompatible model changes fail explicitly.

Changing `MODEL_SEMANTICS_ID` is therefore a deliberate scientific/provenance action and should be reviewed alongside the model change that requires it.
