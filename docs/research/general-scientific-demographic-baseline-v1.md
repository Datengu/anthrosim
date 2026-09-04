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
- current model semantics: `anthrosim-model-semantics-v32`;
- current preserved research execution identity: `research-execution-v1-b9420f1e1bfc07e4`.

Audit-v4 AV4-008 / #514 changes same-seed scarce-resource largest-remainder coupling: equal-remainder claims are now ordered by a persistent person-derived scientific coupling key rather than canonical household/claim-vector position, while retaining the established period/cell fairness rotation. The unchanged confirmatory design was deliberately rerun under v32 in issue-304 workflow run `33853873343`, job `100962563001` (artifact `9929517739`, SHA-256 `165ec16492a582da75e31fd302f535a01e43fd195bd89c2d45a40e2f633c6280`). All **780/780** runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Exact arm/effect values changed in the positive-growth arms and the fixed-founder replacement-control arm, but the high-level recommendation remains `no_universal_demographic_baseline`. Historical v25–v31 results remain immutable evidence attached to the semantics that produced them rather than being relabelled as v32 output.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.492 [-2.786, -2.197] | 2.0 | 57.7% | 37.8% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.243 [-1.435, -1.051] | 13.4 | 16.9% | 22.0% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.071 [-1.212, -0.930] | 23.8 | 3.8% | 42.6% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.069 [-0.120, -0.018] | 104.0 | 0.0% | 13.0% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.915 [-2.133, -1.697] | 6.4 | 29.2% | 40.9% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.429 [-0.547, -0.311] | 48.9 | 1.5% | 17.1% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.4 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-80.2 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-42.5 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v32 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.069%/year under fixed-founder households to -1.071%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 13.0% to 42.6% and mean N240 falls from 104.0 to 23.8.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v32 long-run analysis still reports overwhelmingly drifting trajectories and 2 treatment contexts meeting the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25–v31 evidence likewise remains attached to its original semantics rather than being relabelled as v32 output.
