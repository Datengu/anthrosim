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
- current preserved research execution identity: `research-execution-v1-01b24567d4afddcc`.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.761 [-2.096, -1.426] | 11.3 | 21.5% | 24.5% |
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.725 [-3.027, -2.423] | 2.1 | 56.2% | 38.1% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.400 [-0.492, -0.309] | 46.8 | 0.0% | 17.2% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.599 [-1.807, -1.391] | 8.3 | 21.5% | 40.8% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.083 [-0.131, -0.036] | 107.0 | 0.0% | 12.2% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -0.993 [-1.118, -0.867] | 22.1 | 3.8% | 43.3% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-9.1 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-38.5 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-84.9 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.083%/year under fixed-founder households to -0.993%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 12.2% to 43.3% and mean N240 falls from 107.0 to 22.1.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result.
