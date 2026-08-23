# ADR 0004: Separate model-semantics compatibility from source revision

## Status

Accepted.

## Context

A deterministic checkpoint can only be resumed scientifically if the executable continuing it implements compatible authoritative model semantics. AnthroSim persists both the Cargo package version (`modelVersion`) and the exact source revision (`gitCommit`), but neither value expresses that compatibility contract precisely.

The package version is a software-release identifier. The Git commit is exact provenance. Requiring an identical Git commit for resume would be too strict because documentation, diagnostics, tooling, CI, or other source-neutral changes can leave simulation meaning unchanged. Conversely, allowing any source revision with the same package version can be too weak if authoritative model behaviour changes before the package version changes.

A further provenance problem arises when a compatible checkpoint is deliberately resumed under a different source revision. The final artifact then contains authoritative history produced by more than one executable. Recording only the final executable's `gitCommit` loses the revision that produced the pre-resume state and the exact boundary between the two segments.

## Decision

AnthroSim defines a dedicated compatibility identifier, `MODEL_SEMANTICS_ID`, currently `anthrosim-model-semantics-v1`.

Checkpoint and run-manifest artifacts persist this value as `modelSemanticsId`. `Simulation::from_checkpoint(...)` compares the stored checkpoint value with the executable's current `MODEL_SEMANTICS_ID` and rejects the checkpoint when they differ.

The three identities have deliberately different meanings:

- `modelSemanticsId` answers **may this executable continue this authoritative model state without changing its scientific meaning?** It is the scientific resume compatibility key.
- `gitCommit` answers **which exact source revision produced this artifact or execution segment?** It remains provenance and is not, by itself, a resume gate.
- `modelVersion` answers **which packaged AnthroSim software version produced this artifact?** It remains software/version provenance and the existing compatibility guard; it is not a substitute for the semantics identity.

A change must bump `MODEL_SEMANTICS_ID` when it alters authoritative simulation meaning in a way that makes continuation of an existing checkpoint scientifically incompatible. Examples include changes to demographic transition rules, resource accounting semantics, migration decisions, event ordering that affects state evolution, or RNG consumption that changes authoritative trajectories.

A bump is not required solely for source-neutral changes such as documentation, CI configuration, explorer presentation, diagnostics that do not affect authoritative state, or refactoring proven to preserve the same model semantics and deterministic execution contract.

When uncertainty exists, prefer bumping the semantics identity and treating old checkpoints as incompatible rather than silently resuming across an unverified semantic boundary.

### Resume source lineage

Checkpoint schema v5 and run-manifest schema v10 add a versioned `resumeLineage` object. Fresh uninterrupted runs carry an empty lineage. Every successful `Simulation::from_checkpoint(...)` appends one deterministic boundary containing:

- the source checkpoint's `modelVersion`, `modelSemanticsId`, and `gitCommit`;
- the continuing executable's corresponding identity;
- the exact checkpoint boundary day and completed year;
- the source checkpoint's authoritative `stateDigest64`.

The lineage is append-only across successive resumes. The next boundary's source identity must equal the previous boundary's continuation identity, boundary times cannot move backwards, and the final continuation identity must reconcile with the containing checkpoint/run manifest.

This lineage is provenance, not an additional compatibility gate. A source-neutral Git revision change is therefore allowed when the existing model-version and model-semantics compatibility rules allow the resume, but the completed artifact records both revisions and where the continuation occurred.

Semantic run validation reconciles the lineage carried by `manifest.json` and `checkpoint.json`. Historical boundary state digests are preserved as provenance; they are not recomputed from the final state because the earlier checkpoint state is no longer present in a completed bundle.

Checkpoint schema v4 is accepted as a one-step migration input because it predates `resumeLineage`. Such a checkpoint must have no lineage boundaries; on resume AnthroSim records that v4 checkpoint itself as the first source boundary and emits schema v5 thereafter. Completed run validation continues to require current schemas.

## Consequences

Exact Git provenance is preserved even when one logical run spans multiple compatible source revisions. A completed resumed run can now distinguish which source created the checkpoint state, which source continued it, and at what authoritative state boundary that transition happened.

Uninterrupted and resumed executions remain expected to reach identical authoritative simulation state when their model semantics are identical. Their provenance artifacts are intentionally no longer byte/equality-identical, because the resumed artifact truthfully records that a resume occurred.

Changing `MODEL_SEMANTICS_ID` remains a deliberate scientific/provenance action and should be reviewed alongside the model change that requires it. Exact Git revision remains deliberately separate from that compatibility decision.
