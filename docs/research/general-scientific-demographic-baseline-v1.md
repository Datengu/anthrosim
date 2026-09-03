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
- current model semantics: `anthrosim-model-semantics-v29`;
- current preserved research execution identity: `research-execution-v1-e66b1372b97e7faf`.

The v29 causal semantics change in Audit-v4 AV4-005/#495 removes arbitrary canonical male-person ordering from M2 parentage RNG assignment while preserving the study's high-level conclusion. The unchanged confirmatory design was reviewed under v29 in issue-304 workflow run `33813558679`, job `100840609676` (artifact `9915805924`, SHA-256 `607dbdf2e86db582fe7b519c1bf9ea1ad8d69ba02ffc282f934c4f5d4240d45c`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25, v26, v27 and v28 results remain immutable provenance rather than being relabelled as v29 evidence.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.519 [-2.808, -2.231] | 1.9 | 58.5% | 37.7% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.231 [-1.430, -1.033] | 13.8 | 16.9% | 22.1% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.001 [-1.139, -0.863] | 25.3 | 3.8% | 42.8% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.087 [-0.141, -0.034] | 101.7 | 0.0% | 13.0% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.924 [-2.149, -1.698] | 6.9 | 29.2% | 40.6% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.441 [-0.554, -0.328] | 48.2 | 0.8% | 17.2% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.9 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-76.3 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-41.2 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v29 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.087%/year under fixed-founder households to -1.001%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 13.0% to 42.8% and mean N240 falls from 101.7 to 25.3.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v29 long-run analysis still detects multiple stable regimes, with primary classifications `drifting=747`, `insufficient_data=26`, `stable=7`; 2 treatment contexts meet the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25, v26 and v27 evidence likewise remains attached to its original semantics rather than being relabelled as v29 output.
