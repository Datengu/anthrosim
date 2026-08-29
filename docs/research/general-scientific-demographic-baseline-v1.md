# General scientific demographic baseline result v1

**Conclusion: no universal demographic baseline should be designated.** Future scientific studies must explicitly declare demography. `replacement_control_v1` remains an intrinsic replacement control, not a promise of realized stationarity.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.549 [-1.875, -1.223] | 11.4 | 26.6% | 24.8% |
| `negative_growth_control_v1` | `deterministic_size_fission_v1` | -2.463 [-2.815, -2.111] | 2.6 | 43.8% | 35.6% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.370 [-0.518, -0.221] | 50.3 | 1.6% | 16.7% |
| `replacement_control_v1` | `deterministic_size_fission_v1` | -1.743 [-2.058, -1.427] | 8.2 | 23.4% | 38.8% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.002 [-0.062, +0.058] | 108.2 | 0.0% | 11.3% |
| `positive_growth_control_v1` | `deterministic_size_fission_v1` | -0.994 [-1.198, -0.789] | 28.9 | 4.7% | 38.7% |

For the positive schedule, deterministic fission changes mean year-240 population by -79.4 people (95% MC CI -87.9 to -70.9) relative to fixed-founder households.

This 64-seed confirmation uses new paired process seeds, founder age ceiling 60 years and resource productivity 1000 permille. The separate 288-run exploratory factorial tested founder ages 40/60 and productivity 500/750/1000. The #220 equilibrium gate and #231 precision diagnostics are preserved in the machine-readable result.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. The #239 positive control is only mildly positive in isolation (about +0.42%/year), so structural losses of similar or larger magnitude can erase that margin. The strongest observed association is with mate limitation: under the positive schedule it rises from 11.3% with fixed-founder households to 38.7% under deterministic fission, while late realized growth moves from approximately stationary (-0.002%/year) to strongly negative (-0.994%/year).

This does not by itself prove that deterministic household fission is scientifically wrong. It establishes that household/mating structure is a major treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any of the structural rules should be changed. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates model-form dependence, not prehistoric calibration: deterministic fission is a structural sensitivity treatment, and none of these schedules is empirically calibrated. Historical `synthetic_validation_v1` and all three #239 controls remain unchanged; no target-population feedback is introduced.

The recommendation is therefore a **study-design constraint**, not a new calibrated demographic preset: future confirmatory studies must name and justify the demographic and household structure they use.
