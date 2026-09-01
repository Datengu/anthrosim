# Audit v3 Area L review — observability, analysis outputs and statistical summaries

Immutable discovery target: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / `anthrosim-model-semantics-v21`.

Discovery only. No finding is repaired.

## Surfaces inspected

Fresh v3 inspection covered:

- `scripts/research-general-demography-baseline.py` and the committed 780-run confirmatory result;
- `crates/anthrosim-cli/src/sweep_weighting.rs`;
- `scripts/research-survivor-conditioning.py`;
- `scripts/research-long-run-diagnostics.py`;
- current migration/temporary-mobility observability contracts for nominal vs realized burden;
- provenance binding of downstream analysis inputs.

Known cross-cutting limitations remain AV3-002/006/007/009/010/012/013/014 and are not repaired.

## Fresh paired-summary adversary — AV3-015

The frozen confirmatory general-demography definition uses 130 seeds and six arms = 3 demography schedules × 2 household-lifecycle arms, including current `deterministic_dependency_fission_v2`. Therefore the design contains:

- 3 expected paired-effect groups;
- 130 same-seed pairs per group;
- 390 paired contrasts in total.

`paired_household_effects()` instead searches each same-seed cell for historical `deterministic_size_fission_v1`. Current v2 rows never match, so the committed confirmatory result contains:

```json
"pairedHouseholdEffects": []
```

Observed representation is **0 groups / 0 of 390 paired contrasts**, despite the completed paired design. This is AV3-015/#429 (P2).

## Per-run versus pooled weighting

The frozen sweep weighting path now distinguishes estimands explicitly. Fresh hand-computable adversary:

- eligible run A: 1 completed move with resource score 1000;
- eligible run B: 9 completed moves with resource score 0.

Then:

- equal-run-weighted mean of run means = `(1000 + 0) / 2 = 500`;
- pooled per-completed-move mean = `(1000 + 0) / (1 + 9) = 100`.

`upgrade_point_rows()` exposes these as separate named fields and also exposes move count/support and move-occurrence fraction. Thus the factor-5 distinction in this adversary is not silently collapsed. No new defect demonstrated on this weighting surface.

Operationally censored `personRecordLimitReached` runs are excluded from scientific aggregates, while scientifically meaningful `durationReached` and `populationExtinct` outcomes remain eligible. Undefined denominator-based values stay null and the support counts are exposed.

## Survivor-conditioned estimands

The survivor-conditioning gate detects use of `meanLivingConditionPermille`, requires explicit survivor-conditioning/death-handling tokens, and requires a joint survival/population observable in comparisons.

Fresh direction adversary:

- control: survivor mean condition 600, final living population 100;
- treatment: survivor mean condition 800, final living population 20.

The condition direction is `higher` while living-population direction is `lower`, so the pair is explicitly classified as discordant and `survivorConditionIsPopulationTreatmentEffect = false`. No new defect demonstrated on this survivor-bias disclosure surface.

## Time aggregation, censoring and multimodality

Frozen long-run diagnostics:

- use annual-boundary observations and explicitly ignore subannual terminal snapshots as regular stationarity observations;
- require complete windows and report `insufficient_data` rather than fabricating stability;
- retain non-completed and early-terminated counts in the equilibrium gate;
- require declared run-length/start/end sensitivity coverage for equilibrium-like claims;
- expose stable-regime frequencies by treatment and stochastic multi-regime contexts rather than reducing every context to one long-run mean.

This provides explicit missingness/censoring and multimodality diagnostics. Known AV3-012 remains relevant because a separate identifiability/equifinality summary can still hide nuisance compensation; that is not duplicated here.

## Nominal versus realized quantities

Current frozen migration/temporary observability distinguishes planned/nominal burden from realized burden. M4 records nominal per-person travel condition cost separately from realized saturating total condition loss. Temporary mobility exposes planned round-trip travel versus observed/realized transit and unrealized planned burden. No fresh nominal/realized conflation was demonstrated on these surfaces.

## Uncertainty and downstream compatibility

AV3-006/#410 remains a P1 uncertainty failure: a same-seed 20-replicate contrast can report half-width 3.666756860283 when the covariance-aware half-width is 5.185577281736, falsely passing threshold 4.5. AV3-010/#418 and AV3-012/#421 also remain statistical-summary limitations.

Long-run analysis binds run rows back to immutable planned run IDs/configuration and compares child source fields to the research source. However AV3-014/#427 can make two source-distinct executables share the same null source identity, so this compatibility check inherits that P0 provenance limitation.

## Area-L disposition

Area L has fresh coverage of mechanism-diagnostic variables, nominal/realized quantities, denominators/missingness, time aggregation, per-run versus pooled weighting, operational censoring/extinction, survival bias, multimodality, uncertainty and downstream run compatibility.

New finding: AV3-015/#429 (P2), paired v2 household effects silently omitted from the committed general-demography summary. Known AV3-002/006/007/009/010/012/013/014 remain cross-cutting limitations. No repair was made.

Area L is complete with findings open. Next pending surface is Area M — documentation, TRACE/ODD/ODD+D and claim consistency.
