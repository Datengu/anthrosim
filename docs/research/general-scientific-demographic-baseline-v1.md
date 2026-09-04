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
- current model semantics: `anthrosim-model-semantics-v30`;
- current preserved research execution identity: `research-execution-v1-3c51577035a3770f`.

Audit-v4 AV4-006 / #497 changes same-seed condition-mediated mortality coupling: M3 condition-mortality draws and simultaneous condition/background cause attribution are now assigned in persistent person stochastic-coupling-rank order rather than arbitrary canonical `PersonId`/record order. The unchanged confirmatory design was deliberately rerun under v30 in issue-304 workflow run `33820753145`, job `100862690900` (artifact `9918271667`, SHA-256 `76a943bae2a8a3bef13ee551f17edeb6d485897b03a52d8607ebe3deeb3bb634`). All 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`; the high-level recommendation remained unchanged. Exact v30 re-verification provenance for #304, M7.6, M8.6 and M9.7 is recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md). Historical v25–v29 results remain immutable evidence attached to the semantics that produced them rather than being relabelled as v30 output.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.492 [-2.786, -2.197] | 2.0 | 57.7% | 37.8% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.243 [-1.435, -1.051] | 13.4 | 16.9% | 22.0% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.049 [-1.186, -0.912] | 24.2 | 3.8% | 42.5% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.071 [-0.122, -0.020] | 104.4 | 0.0% | 13.0% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.915 [-2.133, -1.697] | 6.4 | 29.2% | 40.9% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.428 [-0.546, -0.310] | 49.1 | 1.5% | 17.1% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.4 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-80.2 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-42.6 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v30 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.071%/year under fixed-founder households to -1.049%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 13.0% to 42.5% and mean N240 falls from 104.4 to 24.2.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v30 long-run analysis detects primary classifications `drifting=745`, `insufficient_data=27`, `stable=8`; 2 treatment contexts meet the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25–v29 evidence likewise remains attached to its original semantics rather than being relabelled as v30 output.
