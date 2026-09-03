# General scientific demographic baseline result — current confirmation

**Status: living/current TRACE-linked result.** This stable path now reports the authoritative current #304 confirmation. The superseded `deterministic_size_fission_v1` / 64-seed narrative is preserved separately in [`general-scientific-demographic-baseline-v1-historical.md`](general-scientific-demographic-baseline-v1-historical.md).

**Conclusion: no universal demographic baseline should be designated.** Future scientific studies must explicitly declare demography and household lifecycle. `replacement_control_v1` remains an intrinsic replacement control, not a promise of realized stationarity.

## Authoritative confirmation identity

The machine-readable sources of truth are:

- [`../../research/general-demography-baseline-v1/confirmatory-definition.json`](../../research/general-demography-baseline-v1/confirmatory-definition.json);
- [`../../research/general-demography-baseline-v1/confirmatory-result.json`](../../research/general-demography-baseline-v1/confirmatory-result.json).

Current confirmation:

- fixed household control: `fixed_founder_v1`;
- structural treatment: `deterministic_dependency_fission_v2`;
- fresh process seeds per arm: **130** (`3042001..3042130`);
- design: **3 demography schedules × 2 household-lifecycle arms × 130 seeds = 780 completed runs**;
- founder age ceiling: **60 years**;
- resource productivity: **1000 permille**;
- current model semantics: `anthrosim-model-semantics-v26`;
- current preserved research execution identity: `research-execution-v1-2187fac3eabf3d3c`.

The v26 causal semantics change in Audit-v4 AV4-001/#486 changes same-seed fertility-draw assignment while preserving the study's high-level conclusion. The canonical v26 result was reproduced by issue-304 workflow run `33756260873` (artifact `9893726528`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25 results remain immutable provenance rather than being rewritten as v26 evidence.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.470 [-2.764, -2.177] | 2.5 | 56.2% | 38.1% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.274 [-1.499, -1.048] | 12.6 | 16.9% | 23.3% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.080 [-1.255, -0.906] | 23.5 | 9.2% | 43.2% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.026 [-0.073, +0.022] | 112.6 | 0.0% | 12.2% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.834 [-2.082, -1.586] | 8.4 | 22.3% | 40.4% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.411 [-0.512, -0.311] | 49.3 | 1.5% | 16.6% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-10.1 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-89.1 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-40.9 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v26 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.026%/year under fixed-founder households to -1.080%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 12.2% to 43.2% and mean N240 falls from 112.6 to 23.5.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25 Audit-v4 evidence likewise remains attached to its original immutable semantics rather than being relabelled as v26 output.
