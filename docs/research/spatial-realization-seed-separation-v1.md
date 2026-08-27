# Spatial realization seed separation v1

Status: authoritative scientific contract for spatial-run stochastic realization semantics introduced for issue #212.

## Purpose

A spatial AnthroSim ensemble must distinguish uncertainty in the environment from randomness in the people and processes operating within that environment. Before this contract, `ExperimentConfig.seed` also seeded synthetic `World` generation. Changing the nominal replicate seed could therefore change residual environmental fields that were not replaced by the M8 landscape transform, as well as changing demographic, resource-mortality, and migration draws.

That coupling made a common research statement — “repeat the same evidence-grounded scenario under different stochastic histories” — potentially false.

Spatial transform semantics v2 separates three realization roles:

1. **Environment realization seed** — chooses the synthetic baseline world from which any non-overlaid environmental fields are inherited.
2. **Population-initialization realization seed** — chooses stochastic synthetic founder initialization when that initialization mode is used.
3. **Process seed** — `ExperimentConfig.seed`; chooses stochastic dynamic histories after initialization, including demographic, resource-mortality, and migration RNG streams.

These are provenance-bearing scientific identities, not interchangeable technical seed aliases.

## Input and resolution

`SpatialMechanismConfig` may contain an optional `runRealization` object:

```json
{
  "schemaVersion": 1,
  "environmentSeed": 7001,
  "populationSeed": 8001
}
```

The process seed remains the ordinary experiment `seed`.

If `runRealization` is omitted, AnthroSim uses **joint-process-seed mode** for backwards-compatible spatial behavior:

```text
environmentSeed = populationSeed = processSeed = ExperimentConfig.seed
```

If `runRealization` is present, AnthroSim uses **explicit-split mode**:

```text
environmentSeed = runRealization.environmentSeed
populationSeed  = runRealization.populationSeed
processSeed     = ExperimentConfig.seed
```

The resolved values and mode are stored in the spatial mechanism binding. Checkpoint reconstruction and completed-run validation re-use and verify that binding rather than silently substituting the current process seed for the environment.

## Transformation identity versus realization identity

`SpatialMechanismConfig::identity()` describes the scientific landscape-to-model transformation only. It deliberately excludes `runRealization`.

This means two runs may share the same transformation identity while using different environmental or founder realizations. Their complete run provenance remains distinct because the serialized spatial configuration, resolved realization binding, and transformed-world digest retain those choices.

This separation is intentional: “how evidence was transformed into model-facing fields” and “which stochastic residual world was sampled” answer different scientific questions.

## Residual synthetic environmental fields

A normalized landscape does not currently replace every field in `World`. Spatial provenance therefore lists both:

- `residualSyntheticFields`: fields still inherited from synthetic world generation after the declared M8 transforms; and
- `seedVaryingResidualFields`: the subset whose initial values actually vary with `environmentSeed`.

For the current world generator:

- `elevation` is synthetic and environment-seed-varying;
- `seasonAmplitude` is synthetic and environment-seed-varying;
- `seasonPhaseDays` is synthetic but **not** seed-varying: it follows the deterministic row/hemisphere rule;
- `environmentalStress` is a synthetic initial constant of zero;
- `movementCost`, `waterAccess`, `baseProductivity`, and derived initial food stock remain synthetic only when their corresponding M8 transform does not replace them.

This corrects a precision error in the audit wording: the scientific confounding was real, but `seasonPhaseDays` itself was not generated from the seed.

The provenance list is a visibility statement, not a claim that every residual field is active in every experiment. For example, seasonal fields have no resource effect when the configured seasonality scale is zero.

## Research experiment modes

### Fixed environment + varying process histories

Use one fixed `environmentSeed` and one fixed `populationSeed`, then vary `ExperimentConfig.seed` across replicates.

This is the default interpretation for estimating stochastic process variation conditional on one environmental and founder realization. In this mode, changing only the process seed must leave the authoritative `World` digest and stochastic founder initialization unchanged while dynamic event histories may diverge.

A spatial CLI ensemble naturally supports this mode because it shares one spatial mechanism definition (and therefore one explicit `runRealization`) across its run seeds while each run receives a different process seed.

### Varying environment

Change `environmentSeed` while holding the process and population-initialization choices fixed where the design requires that comparison. The transformed-world digest must then identify the resulting environmental realization exactly.

The current experiment layer represents different environment realizations as different immutable spatial experiment definitions. A future expanded experiment/sensitivity layer under #205 may provide a higher-level Cartesian design over environment, initialization, and process seed dimensions; #212 does not collapse those dimensions back into one seed merely for convenience.

### Varying initialization

Change `populationSeed` while holding the environment and process choices fixed when the study is specifically estimating uncertainty from stochastic founder initialization. Declared founder-state initialization does not acquire stochastic uncertainty merely because a population seed exists in provenance; the seed is only causal when the selected initialization path consumes it.

### Joint uncertainty

Vary two or all three dimensions deliberately and record each resolved identity. Joint ensembles are scientifically interpretable only when analysis preserves the grouping needed to separate within-environment/process variation from between-environment or between-initialization variation.

## Variance-decomposition guidance

Do not report one undifferentiated “seed variance” when more than one realization dimension varies.

For a fixed-environment process ensemble, variation among process replicates estimates stochastic outcome variation conditional on that fixed environmental and initialization realization.

For a design with multiple environment realizations, preserve the environment identity in analysis and distinguish at least:

- **within-environment variation**: variation among process replicates sharing the same environment realization;
- **between-environment variation**: changes associated with different environment realizations; and
- where population initialization also varies, the corresponding within/between initialization component or an explicitly declared joint component.

This contract supplies the immutable identities required for that decomposition. Richer automated factorial/sensitivity analysis belongs to the expanded experiment layer tracked separately by #205.

## Compatibility boundary

Spatial transformation/realization semantics advance to:

```text
anthrosim-spatial-transform-semantics-v2
```

`SpatialMechanismBinding` advances to schema v2 because it now carries resolved environment provenance and seed roles.

The ordinary non-spatial core simulation retains its existing model-semantics identity and seed behavior. Spatial configurations that omit `runRealization` retain the historical joint-seed trajectory; the spatial semantics identifier changes because the host now defines and verifies the new realization/provenance contract and explicit split mode.

## Required validation properties

The implementation and regression suite must preserve these properties:

- changing only the process seed in explicit-split mode leaves the complete authoritative `World` digest unchanged;
- with the same stochastic founder seed and same world, initial synthetic population state is unchanged while dynamic histories may diverge;
- changing the environment seed can change the authoritative world without changing the transformation identity;
- resolved realization provenance is present in spatial manifest/checkpoint bindings;
- checkpoint and completed-run validation reconstruct the world from the bound environment realization and fail closed on provenance/world mismatch;
- omitting `runRealization` preserves the established joint-seed spatial behavior.
