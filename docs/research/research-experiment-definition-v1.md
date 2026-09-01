# Exact research experiment definition contract v1

Status: normative infrastructure contract for GitHub issue #205, with structural-treatment validity enforced by #336 and non-overlapping treatment paths enforced by #415.

This document defines AnthroSim's versioned research-facing mechanism for declaring sensitivity and uncertainty experiments without editing simulator source code. It supplements the legacy M7 ensemble/sweep interface; it does not reinterpret or replace the preserved `v0.1-resource-variability` experiment.

## Scientific purpose

The authoritative model configuration is `ExperimentConfig`. A research study must be able to preserve an exact base configuration and then declare which scientifically meaningful assumptions vary, which alternatives are permitted, and which random seeds are paired across those alternatives. The resulting point/run configurations must be reconstructible without knowing hidden source defaults.

The v1 research definition therefore contains:

- `schemaVersion`: research-definition schema version, currently `1`;
- `seeds`: an ordered, duplicate-free list of run seeds;
- `base.experiment`: one complete authoritative `ExperimentConfig`;
- optional `base.spatial`: the exact `SPATIAL_MODEL_SEMANTICS_ID`, `LandscapeBundle`, and `SpatialMechanismConfig`;
- `dimensions`: an ordered list of varied scientific coordinates.

No synthetic model defaults are reconstructed by the research runner. The complete serialized `ExperimentConfig` in the definition is the base scientific state.

## Dimension contract

Each dimension has a stable study-facing `id`, a `kind`, an RFC 6901 JSON-pointer `path`, and an ordered list of `values`.

Paths are limited to the exact typed research configuration rooted at `/experiment` or `/spatial`. A path must already exist in the exact base configuration. Unknown paths fail before execution; there is no create-if-missing behavior and no ignored override behavior.

Dimension paths must also be pairwise non-overlapping by JSON-pointer ancestry. Exact duplicate paths are rejected, and a path may not be a strict ancestor or descendant of another dimension path. For example, `/experiment/resources` and `/experiment/resources/annualNeedUnitsPerPerson` cannot coexist in one definition. Schema v1 intentionally defines no parent/child composition or declaration-order overwrite rule: permitting such a pair could erase a declared treatment while retaining its coordinate metadata. Sibling paths remain valid. Ancestry is evaluated on decoded RFC 6901 path segments rather than raw string prefixes.

`/experiment/seed` is reserved because seeds are controlled by the ordered top-level `seeds` list. Any `schemaVersion` path is also reserved. `/spatial/spatialModelSemanticsId` is likewise reserved: it identifies executable build semantics and cannot be pretended into existence by a within-build dimension. Changing any of these through a scientific dimension would mix provenance/schema identity with scientific coordinates and is rejected.

The v1 path vocabulary is part of the versioned research-definition contract. Examples include:

- `/experiment/demography/minimumBirthSpacingDays`;
- `/experiment/resources/periodsPerYear`;
- `/experiment/resources/maxConditionMortalityProbabilityPerMillion`;
- `/experiment/migration/travelCostWeight`;
- `/spatial/mechanisms/transforms/0/direction` when an exact spatial configuration is present.

A dimension replacement is applied to the serialized authoritative configuration and the entire result is then deserialized back into the existing strongly typed AnthroSim configuration types. Invalid integer widths, enum spellings, object shapes, model alternatives, or other typed values cannot bypass this round trip. The fully resolved point is subsequently passed through the normal `Simulation` or `SpatialLandscapeSimulation` constructor before the experiment root is published.

## Numeric versus structural variation

`kind: "numeric"` is for scalar numeric parameter uncertainty. Its target and all proposed values must be JSON numbers.

`kind: "structural"` is for discrete non-numeric executable model/assumption alternatives, including booleans, typed enum alternatives, optional mechanism blocks, or whole typed sub-configurations. A structural dimension may not target an ordinary numeric field, and a numeric dimension may not target a structural field.

A value is not a structural treatment merely because its JSON type is non-numeric. Before a research definition can expand, AnthroSim projects every proposed structural level onto executable configuration and removes fields that carry provenance, evidence linkage, realization identity, or descriptive parameterization identity but do not enter the transition rules. Declared structural levels must then remain distinct under that executable projection. A one-level structural override is allowed only when its executable projection differs from the base configuration.

The current non-treatment metadata projection includes:

- parameter `provenance` fields;
- demography and temporary-mobility `scheduleId` labels;
- descriptive `modelId` labels for M3 resources, M4 migration, M9 travel and spatial-mechanism parameterizations;
- evidence catalogues and `evidenceId` / `evidenceInputId` links;
- spatial `runRealization`, which controls which stochastic environment/founder draw is used rather than which structural mechanism is being tested;
- M9 focal-region identity/source provenance (`regionId` and `source`) when authoritative membership is otherwise unchanged.

This classification is deliberately path-aware. `HouseholdLifecycleConfig.modelId`, for example, is retained in the executable projection because it is an actual implemented model selector. Likewise booleans, enums, typed mechanism blocks, spatial transform directions and source bindings remain executable structure.

Free-form provenance labels such as `modelId` or `scheduleId` may still accompany a real whole-object structural alternative. A whole typed sub-configuration remains representable when its executable fields differ after metadata is removed. What is rejected is a nominal structural contrast whose proposed levels collapse to the same executable configuration.

Because these checks run during `ResearchExperimentDefinition::validate()`, a metadata-only structural design or overlapping parent/child treatment design fails before point expansion, immutable research-plan publication, run execution or `analysis/points.json` / `analysis/runs.json` generation. The analysis surface therefore cannot report metadata relabelling or an overwritten coordinate as evidence that multiple treatments were tested.

This classification is preserved in point/run analysis rows. Downstream analysis must therefore treat accepted structural alternatives as distinct executable model structures/categories rather than pooling them as a scalar response axis.

## Deterministic expansion

Dimensions are expanded as a Cartesian product in declaration order. Values retain their listed order. Under schema v1 the final declared dimension varies fastest. This ordering controls deterministic point enumeration only; because dimension paths are pairwise non-overlapping, declaration order cannot change which treatments survive in the final executable configuration. A definition with no dimensions still produces one scientific point: the exact base configuration. Expansion fails closed above 100,000 scientific points rather than allocating an unbounded orchestration plan.

Each point preserves:

- its deterministic zero-based index;
- a deterministic point identity;
- every varied coordinate (`id`, `kind`, `path`, exact JSON value);
- the complete resulting `ResearchRunConfig`.

The ordered seed list is then paired with every point. Seed substitution changes only `ExperimentConfig.seed`; all other point configuration remains exact.

## Immutable identities and provenance

The research runner writes a redundant immutable `research-plan.json` / `research-manifest.json` pair before scientific execution. Each contains the original definition, source-revision identity, all expanded points, all per-seed runs, and every exact resulting configuration. The plan is published first so a crash between the two atomic metadata writes can reconstruct the manifest before any scientific child run is attempted.

Identity layers are deliberately separate:

- definition identity binds the complete versioned definition;
- point identity binds point index, coordinates, and complete resolved point configuration;
- run identity binds the point identity, complete seed-specific configuration, and source revision;
- execution/research identity binds the definition identity and source revision.

Changing a scientific value changes the appropriate identities. Re-running an unchanged definition against the same source revision yields the same plan and identities.

These identities are orchestration/provenance identities. They do not change AnthroSim's causal model semantics and do not require a `MODEL_SEMANTICS_ID` bump.

## Retry and crash recovery

Each run is written through the existing transactional run-directory machinery and validated as a normal AnthroSim bundle before publication. Mutable execution state is stored separately from the immutable research manifest.

`--retry` is fail-closed: the supplied definition and current source revision must reconstruct exactly at least one valid immutable root copy, and any valid immutable copy that disagrees causes failure. A missing or malformed counterpart is deterministically reconstructed. Mutable `research-state.json` may also be reconstructed from the immutable plan and child bundles if a crash leaves it missing or malformed. Existing completed bundles are revalidated against their exact planned `ExperimentConfig`, exact source revision and, for spatial runs, their exact spatial semantics/landscape/mechanism configuration. Valid completed runs are retained; missing or failed runs are retried under the same immutable run identity and must reproduce the same deterministic result. A retry may not silently substitute a new configuration.

This preserves the crash-recovery principle introduced for M7 orchestration by #172 while keeping the scientific plan immutable.

## Self-describing analysis surface

The runner derives machine-readable `analysis/points.json` and `analysis/runs.json` tables. Every row preserves all declared coordinates, including `kind` and `path`, plus the complete resulting configuration. Analysts therefore do not need source-code knowledge to discover which scientific assumptions differed between points.

These files expose the experimental design and execution status. They are not, by themselves, a statistical sensitivity-analysis result.

## Backward compatibility

The existing `anthrosim ensemble` and `anthrosim sweep` engineering/synthetic interfaces remain available under their existing contracts. `experiments/v0.1-resource-variability.json` is not migrated or silently reinterpreted by this change.

The research-definition v1 path is additive so validated historical experiment records keep their prior meaning. #336 tightened validation for definitions that labelled non-causal metadata variation as `kind: "structural"`; #415 additionally rejects definitions whose dimension paths overlap by ancestry. Such overlapping definitions had declaration-order-dependent treatment overwrite semantics and did not represent an unambiguous factorial design under the pre-existing requirement that every recorded coordinate correspond to an executable treatment.

## Model-semantics boundary

This infrastructure exposes existing configuration and spatial alternatives; it does not alter M2, M3, M4, M8 or M9 transition rules, defaults, RNG algorithms or draw ordering. For an identical effective `ExperimentConfig` and spatial configuration, the normal simulator is invoked without a sensitivity-specific model execution path.

The #336 and #415 validators change research-design admissibility, not simulation transitions, so `MODEL_SEMANTICS_ID` is unchanged.

## TRACE boundary

This contract satisfies an infrastructure prerequisite for reproducible sensitivity/uncertainty work: the study's scientific configuration space can now be declared, expanded, identified, preserved and retried without source edits. A coordinate recorded as `kind: "structural"` is machine-checked to represent a distinct executable model assumption rather than a provenance relabel, and every accepted dimension path is independent of the others by JSON-pointer ancestry so one declared treatment cannot erase another by subtree replacement.

It does **not** establish that a study has selected defensible uncertainty ranges, sampled the space adequately, completed global sensitivity analysis, quantified Monte Carlo error, resolved identifiability/equifinality, selected valid analysis windows, or performed empirical validation. Those remain study-specific TRACE obligations and/or separate backlog issues.
