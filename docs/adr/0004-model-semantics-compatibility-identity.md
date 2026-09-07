# ADR 0004: Separate model-semantics compatibility from source revision

## Status

Accepted.

## Context

A deterministic checkpoint can only be resumed scientifically if the executable continuing it implements compatible authoritative model semantics. AnthroSim persists both the Cargo package version (`modelVersion`) and the exact source revision (`gitCommit`), but neither value expresses that compatibility contract precisely.

The package version is a software-release identifier. The Git commit is exact provenance. Requiring an identical Git commit for resume would be too strict because documentation, diagnostics, tooling, CI, or other source-neutral changes can leave simulation meaning unchanged. Conversely, allowing any source revision with the same package version can be too weak if authoritative model behaviour changes before the package version changes.

A further provenance problem arises when a compatible checkpoint is deliberately resumed under a different source revision. The final artifact then contains authoritative history produced by more than one executable. Recording only the final executable's `gitCommit` loses the revision that produced the pre-resume state and the exact boundary between the two segments.

## Decision

AnthroSim defines a dedicated compatibility identifier, `MODEL_SEMANTICS_ID`. The value when this ADR was introduced was `anthrosim-model-semantics-v1`; it is **not** a permanently fixed value. The authoritative current value is defined in `crates/anthrosim-core/src/provenance.rs` and must advance when authoritative simulation meaning changes incompatibly. At the post-v0.3.4 documentation-audit snapshot, that executable value is `anthrosim-model-semantics-v33`.

Checkpoint and run-manifest artifacts persist this value as `modelSemanticsId`. `Simulation::from_checkpoint(...)` compares the stored checkpoint value with the executable's current `MODEL_SEMANTICS_ID` and rejects the checkpoint when they differ.

The three identities have deliberately different meanings:

- `modelSemanticsId` answers **may this executable continue this authoritative model state without changing its scientific meaning?** It is the scientific resume compatibility key.
- `gitCommit` answers **which exact source revision produced this artifact or execution segment?** It remains provenance and is not, by itself, a resume gate.
- `modelVersion` answers **which packaged AnthroSim software version produced this artifact?** It remains software/version provenance and a compatibility/provenance field; it is not a substitute for the semantics identity.

A change must bump `MODEL_SEMANTICS_ID` when it alters authoritative simulation meaning in a way that makes continuation of an existing checkpoint scientifically incompatible. Examples include changes to demographic transition rules, resource accounting semantics, migration decisions, event ordering that affects state evolution, or RNG consumption/assignment that changes authoritative trajectories.

A bump is not required solely for source-neutral changes such as documentation, CI configuration, explorer presentation, diagnostics that do not affect authoritative state, or refactoring proven to preserve the same model semantics and deterministic execution contract.

When uncertainty exists, prefer bumping the semantics identity and treating old checkpoints as incompatible rather than silently resuming across an unverified semantic boundary.

### Resume source lineage

Resume lineage is append-only provenance across compatible source-revision changes. Fresh uninterrupted runs carry an empty lineage. Every successful compatible resume appends a deterministic boundary containing the source and continuation model/source identities, the exact checkpoint boundary time, and the source checkpoint's preserved continuation/state identity.

The next boundary's source identity must equal the previous boundary's continuation identity, boundary times cannot move backwards, and the final continuation identity must reconcile with the containing checkpoint/run manifest. Current schema constants are defined by the implementation; historical schema numbers mentioned in the Git history of this ADR document the migration state that originally introduced the lineage mechanism and should not be treated as permanently current values.

This lineage is provenance, not an additional compatibility gate. A source-neutral Git revision change is therefore allowed when the existing model-version and model-semantics compatibility rules allow the resume, but the completed artifact records both revisions and where the continuation occurred.

Semantic run validation reconciles the lineage carried by `manifest.json` and `checkpoint.json`. Historical boundary identities are preserved as provenance rather than recomputed from final state that no longer contains the earlier checkpoint state.

## Consequences

Exact Git provenance is preserved even when one logical run spans multiple compatible source revisions. A completed resumed run can distinguish which source created checkpoint state, which source continued it, and at what authoritative state boundary that transition occurred.

Uninterrupted and resumed executions remain expected to reach identical authoritative simulation state when their model semantics are identical. Their provenance artifacts are intentionally not necessarily byte/equality-identical, because the resumed artifact truthfully records that a resume occurred.

Changing `MODEL_SEMANTICS_ID` remains a deliberate scientific/provenance action and should be reviewed alongside the model change that requires it. Exact Git revision remains deliberately separate from that compatibility decision.
