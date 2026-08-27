# Exact research experiment definition contract v1

Status: normative infrastructure contract for GitHub issue #205.

This document defines AnthroSim's versioned research-facing mechanism for declaring sensitivity and uncertainty experiments without editing simulator source code. It supplements the legacy M7 ensemble/sweep interface; it does not reinterpret or replace the preserved `v0.1-resource-variability` experiment.

## Scientific purpose

The authoritative model configuration is `ExperimentConfig`. A research study must be able to preserve an exact base configuration and then declare which scientifically meaningful assumptions vary, which alternatives are permitted, and which random seeds are paired across those alternatives. The resulting point/run configurations must be reconstructible without knowing hidden source defaults.

The v1 research definition therefore contains:

- `schemaVersion`: research-definition schema version, currently `1`;
- `seeds`: an ordered, duplicate-free list of run seeds;
- `base.experiment`: one complete authoritative `ExperimentConfig`;
- optional `base.spatial`: an exact `LandscapeBundle` plus `SpatialMechanismConfig`;
- `dimensions`: an ordered list of varied scientific coordinates.

No synthetic model defaults are reconstructed by the research runner. The complete serialized `ExperimentConfig` in the definition is the base scientific state.

## Dimension contract

Each dimension has a stable study-facing `id`, a `kind`, an RFC 6901 JSON-pointer `path`, and an ordered list of `values`.

Paths are limited to the exact typed research configuration rooted at `/experiment` or `/spatial`. A path must already exist in the exact base configuration. Unknown paths fail before execution; there is no create-if-missing behavior and no ignored override behavior.

`/experiment/seed` is reserved because seeds are controlled by the ordered top-level `seeds` list. Any `schemaVersion` path is also reserved. Changing either through a scientific dimension would mix provenance/schema identity with scientific coordinates and is rejected.

The v1 path vocabulary is part of the versioned research-definition contract. Examples include:

- `/experiment/demography/minimumBirthSpacingDays`;
- `/experiment/resources/periodsPerYear`;
- `/experiment/resources/maxConditionMortalityProbabilityPerMillion`;
- `/experiment/migration/travelCostWeight`;
- `/spatial/mechanisms/transforms/0/direction` when an exact spatial configuration is present.

A dimension replacement is applied to the serialized authoritative configuration and the entire result is then deserialized back into the existing strongly typed AnthroSim configuration types. Invalid integer widths, enum spellings, object shapes, model alternatives, or other typed values cannot bypass this round trip. The fully resolved point is subsequently passed through the normal `Simulation` or `SpatialLandscapeSimulation` constructor before the experiment root is published.

## Numeric versus structural variation

`kind: "numeric"` is for scalar numeric parameter uncertainty. Its target and all proposed values must be JSON numbers.

`kind: "structural"` is for discrete non-numeric model/assumption alternatives, including booleans, model identifiers, enum alternatives, optional mechanism blocks, or whole typed sub-configurations. A structural dimension may not target an ordinary numeric field, and a numeric dimension may not target a structural field.

This classification is preserved in point/run analysis rows. Downstream analysis must therefore treat structural alternatives as distinct model structures/categories rather than pooling them as a scalar response axis.

## Deterministic expansion

Dimensions are expanded as a Cartesian product in declaration order. Values retain their listed order. Under schema v1 the final declared dimension varies fastest. A definition with no dimensions still produces one scientific point: the exact base configuration.

Each point preserves:

- its deterministic zero-based index;
- a deterministic point identity;
- every varied coordinate (`id`, `kind`, `path`, exact JSON value);
- the complete resulting `ResearchRunConfig`.

The ordered seed list is then paired with every point. Seed substitution changes only `ExperimentConfig.seed`; all other point configuration remains exact.

## Immutable identities and provenance

The research runner writes one immutable `research-manifest.json` before scientific execution. It contains the original definition, source-revision identity, all expanded points, all per-seed runs, and every exact resulting configuration.

Identity layers are deliberately separate:

- definition identity binds the complete versioned definition;
- point identity binds point index, coordinates, and complete resolved point configuration;
- run identity binds the point identity, complete seed-specific configuration, and source revision;
- execution/research identity binds the definition identity and source revision.

Changing a scientific value changes the appropriate identities. Re-running an unchanged definition against the same source revision yields the same plan and identities.

These identities are orchestration/provenance identities. They do not change AnthroSim's causal model semantics and do not require a `MODEL_SEMANTICS_ID` bump.

## Retry and crash recovery

Each run is written through the existing transactional run-directory machinery and validated as a normal AnthroSim bundle before publication. Mutable execution state is stored separately from the immutable research manifest.

`--retry` is fail-closed: the supplied definition and current source revision must reconstruct exactly the immutable recorded manifest. Existing completed bundles are revalidated against their exact planned `ExperimentConfig` and, for spatial runs, their exact landscape/mechanism configuration. Valid completed runs are retained; missing or failed runs are retried under the same immutable run identity. A retry may not silently substitute a new configuration.

This preserves the crash-recovery principle introduced for M7 orchestration by #172 while keeping the scientific plan immutable.

## Self-describing analysis surface

The runner derives machine-readable `analysis/points.json` and `analysis/runs.json` tables. Every row preserves all declared coordinates, including `kind` and `path`, plus the complete resulting configuration. Analysts therefore do not need source-code knowledge to discover which scientific assumptions differed between points.

These files expose the experimental design and execution status. They are not, by themselves, a statistical sensitivity-analysis result.

## Backward compatibility

The existing `anthrosim ensemble` and `anthrosim sweep` engineering/synthetic interfaces remain available under their existing contracts. `experiments/v0.1-resource-variability.json` is not migrated or silently reinterpreted by this change.

The research-definition v1 path is additive so validated historical experiment records keep their prior meaning.

## Model-semantics boundary

This infrastructure exposes existing configuration and spatial alternatives; it does not alter M2, M3, M4, M8 or M9 transition rules, defaults, RNG algorithms or draw ordering. For an identical effective `ExperimentConfig` and spatial configuration, the normal simulator is invoked without a sensitivity-specific model execution path.

`MODEL_SEMANTICS_ID` is therefore unchanged by #205.

## TRACE boundary

This contract satisfies an infrastructure prerequisite for reproducible sensitivity/uncertainty work: the study's scientific configuration space can now be declared, expanded, identified, preserved and retried without source edits.

It does **not** establish that a study has selected defensible uncertainty ranges, sampled the space adequately, completed global sensitivity analysis, quantified Monte Carlo error, resolved identifiability/equifinality, selected valid analysis windows, or performed empirical validation. Those remain study-specific TRACE obligations and/or separate backlog issues.
