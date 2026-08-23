# Source revision provenance

AnthroSim records the source revision used to build the simulator in the existing `gitCommit` provenance field carried by run manifests, checkpoints, experiment manifests and sweep manifests.

## Ordinary Git builds

No manual environment variable is required for a normal build from a Git checkout. The `anthrosim-core` and `anthrosim-cli` build scripts resolve `git rev-parse --verify HEAD` automatically and embed that revision at compile time.

For a clean tracked working tree, `gitCommit` is the exact Git commit SHA.

For a working tree with staged or unstaged changes to tracked files, `gitCommit` is recorded as:

```text
<commit-sha>-dirty-<working-tree-digest>
```

The digest is Git's object hash of a canonical binary/full-index diff from `HEAD` to the current tracked working tree. It therefore binds the dirty provenance value to the actual tracked modifications as well as the base commit. Two materially different tracked dirty trees based on the same `HEAD` must not share the same automatic source identity.

The build also emits a warning. The `-dirty-<working-tree-digest>` component is intentional: it prevents a locally modified binary from claiming to be source-identical either to the clean named commit or to a different dirty source tree based on that commit. This is required for immutable/retryable ensemble and sweep provenance, where completed child runs must not be retained across a source-code change that happens to remain uncommitted.

Untracked files are not themselves included in the dirty check. Any tracked source change that begins using a new untracked source file still makes the tracked tree dirty through that tracked reference. Research workflows should nevertheless keep the checkout clean and reviewable.

The build scripts register the tracked repository files and relevant Git metadata as Cargo rerun inputs so a normal rebuild refreshes the embedded revision/dirty identity when tracked source state changes.

If AnthroSim detects that the tracked tree is dirty but cannot derive its exact dirty-tree digest, it does not fall back to the ambiguous historical `<commit-sha>-dirty` identity. Instead it leaves `gitCommit` unavailable and emits a warning. Ordinary exploratory execution may continue, but research workflows that require exact provenance will reject the missing identity.

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
- `gitCommit` carries an automatic dirty-tree marker (`-dirty` or `-dirty-<working-tree-digest>`).

The legacy `-dirty` form remains recognized so older dirty builds cannot bypass this policy.

A versioned research sweep therefore requires either a clean Git build with automatic revision capture or a deliberate non-empty `ANTHROSIM_GIT_COMMIT` override supplied by a controlled build environment.

This policy applies only to the versioned research adapter. It does not prevent ordinary local runs from being used for development or exploratory work when exact source identity is unavailable.
