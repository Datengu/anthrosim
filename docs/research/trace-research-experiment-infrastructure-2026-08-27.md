# TRACE change record: exact research experiment infrastructure

Date: 2026-08-27  
Issue: #205 — P1 TRACE audit: expose the full scientific configuration to reproducible ensemble and sensitivity experiments

## Problem identified by the audit

AnthroSim already had strong deterministic M7 run bundles, manifests and retry semantics, but its public sweep-definition surface represented only a small fixed subset of the scientifically meaningful configuration. The execution manifest could preserve a complete `ExperimentConfig` after it had been constructed, while the study definition itself still depended on hidden synthetic defaults for many M2/M3/M4 assumptions and had no general way to declare M8/M9 alternatives.

That gap was a TRACE reproducibility problem: a study could appear to define its uncertainty space while scientifically consequential assumptions remained frozen in source code rather than represented in the preserved experiment definition.

## Repair

#205 adds a separate, versioned **research-definition path** around the existing authoritative `ExperimentConfig` rather than extending the legacy M7 sweep with many more dedicated flags.

The research definition preserves:

- one complete exact base `ExperimentConfig`;
- optional exact spatial model-semantics identity, landscape and mechanism configuration;
- an ordered seed set;
- ordered sensitivity/uncertainty dimensions with stable IDs, typed classification (`numeric` or `structural`), exact configuration paths and permitted values.

Expansion creates complete resulting typed configurations before any scientific run directory is published. Unknown paths, malformed paths, type/classification mismatches, invalid typed enum/model alternatives, incompatible spatial semantics and invalid resulting simulator configurations fail closed. Cartesian expansion is deterministic and is rejected above 100,000 scientific points rather than allocating an unbounded experiment plan.

The normal `Simulation` / `SpatialLandscapeSimulation` constructors remain the validation and execution boundary. No sensitivity-specific model implementation was introduced.

The normative contract is documented in [`research-experiment-definition-v1.md`](research-experiment-definition-v1.md).

## Reproducibility and recovery interpretation

The runner publishes a redundant immutable `research-plan.json` / `research-manifest.json` pair before scientific execution. Each records the complete definition, source revision, deterministic expanded point plan, exact point coordinates and all exact seed-specific resulting configurations. Definition, point, run and execution identities bind those records. The plan is published first so a crash between the two atomic root-metadata writes can reconstruct the missing manifest before any child scientific run begins.

Mutable run lifecycle state is separate from the immutable plan. Missing or malformed mutable state can be reconstructed from the immutable plan and provenance-valid child bundles. During `--retry`, a missing or malformed immutable root copy is reconstructed only when the other copy exactly corroborates the requested definition/source; a valid conflicting copy fails closed. Existing child bundles are retained only after semantic bundle validation plus exact planned `ExperimentConfig`, source revision and, when spatial, spatial-semantics/landscape/mechanism reconciliation. Missing or failed children are recreated from the same immutable run identity. End-to-end retry coverage deletes a completed child and verifies recreation with the same configuration, run identity and deterministic state digest.

This extends the crash-recoverable orchestration principle of #172 to the new research-definition surface rather than weakening it.

Derived point/run analysis records include every varied coordinate and its numeric/structural classification plus the full resulting configuration. Analysts therefore do not need to infer hidden differences from source defaults or from fixed-column sweep code.

## Structural-variation interpretation

Numeric dimensions are scalar parameter axes. Structural dimensions are categorical/non-numeric executable alternatives and remain visibly classified as structural in provenance and analysis outputs so downstream tooling cannot accidentally treat them as ordinary scalar coordinates.

Free-form provenance labels such as a `modelId` or `scheduleId` do not, by themselves, create a causal structural alternative. A study claiming a structural comparison must vary an executable typed field or configuration block (and may also vary its accompanying provenance label where appropriate). Typed enum alternatives and spatial mechanism alternatives are covered by the v1 validation/tests.

## Backward-compatibility boundary

The existing `anthrosim ensemble` and `anthrosim sweep` paths remain under their prior engineering/synthetic contracts. The preserved `experiments/v0.1-resource-variability.json` experiment is not migrated, rewritten or reinterpreted by #205.

No existing canonical simulation is intentionally reparameterized by this repair.

## Model-semantics assessment

`MODEL_SEMANTICS_ID` is **unchanged**.

Reason: #205 changes research configuration declaration, expansion, orchestration identity and provenance. It does not change M2/M3/M4/M8/M9 causal transition rules, parameter defaults, RNG algorithms or draw ordering. The same effective authoritative configuration continues through the normal simulator execution path.

Any unexplained numerical change in protected M7.6/M8.6/M9.7 references is therefore a regression signal, not a reason to rebaseline a scientific reference.

## Verification scope for #205

Acceptance coverage demonstrates:

- demographic timing variation;
- M3 timing variation (`periodsPerYear`);
- condition-mediated mortality-response variation;
- an M4 utility/travel parameter;
- a spatial/M8 structural alternative;
- simultaneous dimensions and deterministic Cartesian order;
- stable identities for the same definition and identity changes when scientific values change;
- exact paired seed substitution;
- fail-closed unknown field, wrong type, invalid typed enum, reserved identity path, incompatible spatial semantics and excessive Cartesian expansion handling before scientific execution;
- exact full resulting configuration in immutable point/run provenance;
- self-describing analysis rows with all varied coordinates;
- explicit structural alternatives that cannot be silently treated as numeric axes;
- retry/recovery using the same exact point/run configurations, source identity and deterministic result;
- recovery of one missing/malformed immutable root copy and of missing/malformed mutable execution state.

Repository protection gates remain authoritative for trajectory/reference stability and cross-platform determinism. #205 does not authorize rebaselining a scientific reference merely because orchestration schemas or identities changed.

## What this closes — and what it does not

Closing #205 means AnthroSim has infrastructure capable of **declaring and reproducing the scientifically relevant sensitivity/uncertainty space without source edits**.

It does **not** mean that global sensitivity analysis has been completed. It does not establish defensible uncertainty ranges for a particular archaeological study, adequate Monte Carlo sample sizes, global design coverage, parameter identifiability, equifinality resolution, valid analysis windows, empirical calibration/validation or independent corroboration.

Those remain separate TRACE research requirements and must be addressed by study-specific protocols and the relevant later issues. The correct post-#205 statement is therefore: **the reproducible experiment-definition prerequisite exists; the scientific sensitivity programme remains to be designed and executed for each study.**
