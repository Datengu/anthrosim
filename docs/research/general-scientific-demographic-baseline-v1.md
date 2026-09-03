# General scientific demographic baseline result — current confirmation

**Status: living/current TRACE-linked result.** This stable path reports the authoritative current #304 confirmation. The superseded `deterministic_size_fission_v1` / 64-seed narrative is preserved separately in [`general-scientific-demographic-baseline-v1-historical.md`](general-scientific-demographic-baseline-v1-historical.md).

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
- current model semantics: `anthrosim-model-semantics-v27`;
- current preserved research execution identity: `research-execution-v1-2a74837d17e12d37`.

The v27 causal semantics change in Audit-v4 AV4-002/#488 changes same-seed background-mortality draw assignment while preserving the study's high-level conclusion. The canonical v27 result was reproduced by issue-304 workflow run `33785448601`, job `100748979710` (artifact `9905391271`, SHA-256 `9c8c2cc4f661377e2dca151147bb4c98d146a5668bc8d7c8de48c26ab381be96`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25 and v26 results remain immutable provenance rather than being rewritten as v27 evidence.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.553 [-2.844, -2.262] | 2.4 | 53.8% | 38.1% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.219 [-1.395, -1.042] | 13.5 | 18.5% | 23.1% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.062 [-1.207, -0.917] | 20.8 | 3.8% | 43.3% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.038 [-0.095, +0.019] | 111.9 | 0.8% | 11.8% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.839 [-2.053, -1.625] | 7.4 | 24.6% | 41.0% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.480 [-0.588, -0.373] | 44.0 | 2.3% | 17.9% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.0 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-91.1 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-36.6 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v27 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.038%/year under fixed-founder households to -1.062%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 11.8% to 43.3% and mean N240 falls from 111.9 to 20.8.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v27 long-run analysis still detects multiple stable regimes, with primary classifications `drifting=747`, `insufficient_data=25`, `stable=8`; one treatment context meets the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25 and v26 evidence likewise remains attached to its original semantics rather than being relabelled as v27 output.
