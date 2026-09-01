# Audit v3 Area K review — experiment orchestration, configuration, provenance and reproducibility

Immutable discovery target: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / `anthrosim-model-semantics-v21`.

This is discovery evidence only. No finding is repaired.

## Surfaces inspected

Fresh v3 inspection covered the frozen research definition/orchestrator and source-provenance paths:

- `crates/anthrosim-core/src/research_experiment.rs`;
- `crates/anthrosim-core/src/provenance.rs`;
- `crates/anthrosim-cli/src/bin/anthrosim-research.rs`;
- `build/git_provenance.rs`;
- `docs/research/research-experiment-definition-v1.md`;
- `docs/source-provenance.md`.

Known cross-cutting findings AV3-002/#392, AV3-007/#413, AV3-008/#415 and AV3-013/#423 remain open and unrepaired.

## Complete configuration capture and defaults

The research definition embeds one complete authoritative `ExperimentConfig` and optional complete spatial configuration. Point expansion stores both the declared coordinates and the complete resulting `ResearchRunConfig`; each per-seed planned run again stores the complete seed-specific config. Invalid resolved configurations are passed through the normal authoritative simulation constructor before an experiment root is published.

This avoids a second hidden research-specific set of simulation defaults. However AV3-008 remains a P1 design-integrity exception: overlapping dimensions can preserve coordinate metadata after one treatment has been overwritten in the executable config.

## Immutable planning and retry

The runner publishes redundant `research-plan.json` and `research-manifest.json` before child execution. On retry:

- a non-empty root must have at least one exact valid immutable copy;
- any valid immutable copy that differs from the expected current definition/source fails closed;
- a missing/malformed counterpart is reconstructed only from an exact corroborating copy/current expected manifest;
- mutable `research-state.json` can be reconstructed from immutable plan plus child bundles;
- existing child bundles are semantically revalidated before being retained;
- a missing child is recreated under the same planned `runId`, configuration and source identity.

Frozen unit tests cover exact retry without reexecution, missing-run recreation with identical digest, redundant immutable-root recovery, malformed mutable-state recovery and invalid-definition failure before root creation. These are useful positive controls but not sole v3 completion evidence.

## Child bundle/source validation

`validate_completed_run()` revalidates the bundle file set and requires core manifest/checkpoint source fields, exact experiment config and digest to match the immutable planned run. Spatial runs additionally require exact landscape/mechanism objects and wrapper checkpoint/manifest consistency. Non-spatial planned runs reject unexpected spatial artifacts.

AV3-002 remains a fail-closed replay limitation for declared-founder + M9 histories; it does not make the generic retry source comparison permissive.

## Fresh source-identity adversary

The new Area-K attack asks whether exact/versioned research can retain a result from a source-distinct executable under the same immutable identity. The answer is yes when exact Git identity is absent.

`SourceRevisionIdentity` represents source as `(modelVersion, modelSemanticsId, gitCommit: Option<String>)`. The build helper explicitly permits `gitCommit=null` when Git metadata cannot be derived. `anthrosim-research` does not reject this missing identity before creating its immutable plan or `runId`s.

Therefore two source-distinct 0.3.3/v21 builds with no Git identity collapse to the same source object. With the same definition they produce the same `researchId` and `runId`s, and B's `--retry` can accept and retain A's completed bundle because `None == None` satisfies the source comparisons. Full derivation is preserved in `area-k-null-source-identity-adversary-2026-09-01.md`.

This is AV3-014, severity P0, because immutable research provenance can positively misidentify which executable source generated authoritative accepted output.

## Other K dispositions

- Exact command-line spelling is not itself part of scientific identity; the definition, source identity, resolved configs and execution state encode the scientifically relevant content. No separate defect demonstrated from command spelling.
- Operational attempt count is deliberately mutable execution metadata and remains separate from immutable scientific run identity. No defect demonstrated.
- The explicit `ANTHROSIM_GIT_COMMIT` override is trusted by design for controlled build environments. This review does not infer a second finding merely because a privileged build system could lie; AV3-014 is limited to the ordinary missing-identity state being accepted as exact research identity.
- Analysis-identity/evidence-identity provenance gaps are already represented by AV3-007 and AV3-013; no duplicate issue is created.

## Area-K disposition

Area K has fresh v3 evidence across complete configuration capture, dimension expansion, immutable plan publication, retry/crash recovery, child bundle validation, source identity and run/research identities. One new P0 finding, AV3-014, is demonstrated and left unrepaired. Known AV3-002/007/008/013 remain cross-cutting limitations.

Area K is complete with findings open. Next pending surface is Area L — observability, analysis outputs and statistical summaries.
