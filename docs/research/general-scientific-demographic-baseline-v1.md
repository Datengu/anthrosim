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
- current model semantics: `anthrosim-model-semantics-v28`;
- current preserved research execution identity: `research-execution-v1-93503391ed6cd598`.

The v28 causal semantics change in Audit-v4 AV4-003/#491 changes same-seed M4 migration draw assignment by removing arbitrary HouseholdId decision ordering while preserving the study's high-level conclusion. The canonical v28 result was reproduced by issue-304 workflow run `33797427793`, job `100788374930` (artifact `9909929317`, SHA-256 `f2c1929fd558eab3d6a003f2d1ff447f42c81b0585f81f315cca8b1f9430470c`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25, v26 and v27 results remain immutable provenance rather than being rewritten as v28 evidence.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.555 [-2.898, -2.213] | 2.1 | 56.9% | 37.8% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.402 [-1.659, -1.144] | 13.7 | 19.2% | 22.4% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.148 [-1.333, -0.963] | 22.1 | 6.9% | 42.4% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.107 [-0.180, -0.033] | 106.1 | 0.8% | 12.5% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.781 [-2.035, -1.528] | 8.3 | 26.9% | 40.6% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.609 [-0.786, -0.432] | 45.6 | 3.8% | 17.2% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.6 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-84.0 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-37.2 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The paired v28 effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.107%/year under fixed-founder households to -1.148%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 12.5% to 42.4% and mean N240 falls from 106.1 to 22.1.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The v28 long-run analysis still detects multiple stable regimes, with primary classifications `drifting=744`, `insufficient_data=32`, `stable=4`; 1 treatment context meets the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Historical v25, v26 and v27 evidence likewise remains attached to its original semantics rather than being relabelled as v28 output.
