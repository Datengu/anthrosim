# Source revision provenance

AnthroSim records the source revision used to build the simulator in the existing `gitCommit` provenance field carried by run manifests, checkpoints, experiment manifests and sweep manifests.

## Ordinary Git builds

No manual environment variable is required for a normal build from a Git checkout. The `anthrosim-core` and `anthrosim-cli` build scripts resolve `git rev-parse --verify HEAD` automatically and embed that revision at compile time.

For a clean tracked working tree, `gitCommit` is the exact Git commit SHA.

For a working tree with staged or unstaged changes to tracked files, `gitCommit` is recorded as:

```text
<commit-sha>-dirty
```

The build also emits a warning. The suffix is intentional: it prevents a locally modified binary from claiming to be byte-for-byte/source-identical to the clean named commit while preserving the base commit that the modifications were made against.

Untracked files are not themselves included in the dirty check. Any tracked source change that begins using a new untracked source file still makes the tracked tree dirty through that tracked reference. Research workflows should nevertheless keep the checkout clean and reviewable.

The build scripts register the tracked repository files and relevant Git metadata as Cargo rerun inputs so a normal rebuild refreshes the embedded revision/dirty marker when tracked source state changes.

## Explicit override

`ANTHROSIM_GIT_COMMIT` remains an explicit build-time override for controlled environments:

```text
ANTHROSIM_GIT_COMMIT=<revision> cargo build --locked --workspace --release
```

When supplied, the non-empty override is trusted verbatim and automatic Git detection is bypassed. CI/release workflows may therefore pin a revision supplied by the build system even when Git metadata is not the desired authority.

## Outside a Git checkout

If no explicit override is supplied and Git metadata cannot be resolved, AnthroSim does not fabricate a source revision. The build emits a warning and run provenance records:

```json
"gitCommit": null
```

This is permitted for ordinary exploratory execution, but it is not sufficient for the versioned research/reference workflow.

## Research/reference policy

`scripts/run-versioned-sweep.py` requires reproducible source identity. It fails after validating the generated immutable sweep manifest when:

- `gitCommit` is absent/null; or
- `gitCommit` ends in `-dirty`.

A versioned research sweep therefore requires either a clean Git build with automatic revision capture or a deliberate non-empty `ANTHROSIM_GIT_COMMIT` override supplied by a controlled build environment.

This policy applies only to the versioned research adapter. It does not prevent ordinary local runs from being used for development or exploratory work when exact source identity is unavailable.
