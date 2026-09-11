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
- current model semantics: `anthrosim-model-semantics-v38`;
- current preserved research execution identity: `research-execution-v1-1f388962905c7f84`.

Audit-v4 AV4-009 / #518 changed M4 weighted migration candidate stochastic coupling: physically equivalent candidate classes receive uncertainty and weighted choice independently of arbitrary canonical `CellId`/container order, while the declared demographic controls remained unchanged. The accepted equivalence-class sampler was reviewed before reference synchronization on production head `25c9a11dce8052fecfbb339114a4ba1c8da00b0c` in issue-304 workflow run `33884799723`, job `101061843958` (artifact `9941547662`, SHA-256 `dc439b48bb7d2a048c3fe2365698d2403a67ac2a26e340c17bb6dc8a8901fa83`). The synchronized v33 result was then rerun on exact head `c9b7a0f2d762323afa76b7d0f390f29930a77b0a` in run `33885763038`, job `101065007590` (artifact `9941945506`, SHA-256 `d33cfb889d514087ddc9e70e8c67d6ca6abdb759494279cb79f02060c12718fc`). Those older results remain immutable provenance for the semantics that produced them rather than being relabelled as current output.

Audit-v6 AV6-004 / #707 changes M3 scarce-resource equal-remainder allocation so canonical resource-cell index is no longer a causal tie phase. Because that authoritative allocation change can propagate into condition and downstream demographic trajectories, the living model semantics advanced to v37 and the 780-run demographic confirmation was rerun rather than metadata-relabelled. Fresh v37 issue-304 run `34544291570` produced the current result; guarded verifier run `34550319686` confirmed that the numerical delta was confined to the negative-growth and replacement `fixed_founder_v1` controls and their derived paired household effects, while the positive-growth fixed-founder arm, all dependency-fission arms, the preserved recommendation and the remaining normalized scientific content reproduced. The recommendation remains `no_universal_demographic_baseline`.

The numerical table below is synchronized from the current confirmatory result. CI values are the result's 95% Monte Carlo confidence intervals; full-precision machine values remain authoritative.

| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |
|---|---|---:|---:|---:|---:|
| `negative_growth_control_v1` | `deterministic_dependency_fission_v2` | -2.844 [-3.233, -2.455] | 2.3 | 54.6% | 37.7% |
| `negative_growth_control_v1` | `fixed_founder_v1` | -1.236 [-1.444, -1.027] | 13.7 | 16.9% | 23.0% |
| `positive_growth_control_v1` | `deterministic_dependency_fission_v2` | -1.073 [-1.207, -0.938] | 19.8 | 7.7% | 43.2% |
| `positive_growth_control_v1` | `fixed_founder_v1` | -0.090 [-0.137, -0.042] | 107.7 | 0.0% | 12.0% |
| `replacement_control_v1` | `deterministic_dependency_fission_v2` | -1.854 [-2.094, -1.613] | 7.6 | 26.9% | 40.6% |
| `replacement_control_v1` | `fixed_founder_v1` | -0.478 [-0.590, -0.365] | 44.6 | 2.3% | 18.4% |

## Same-seed household-lifecycle effects

The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.

- `negative_growth_control_v1`: fission-minus-fixed mean N240 = **-11.4 people** across 130 same-seed pairs.
- `positive_growth_control_v1`: fission-minus-fixed mean N240 = **-87.9 people** across 130 same-seed pairs.
- `replacement_control_v1`: fission-minus-fixed mean N240 = **-37.0 people** across 130 same-seed pairs.

These paired effects are bound to the household-lifecycle contrast declared in the confirmatory definition. Missing, duplicated or unknown lifecycle arms are a fail-closed analysis error rather than being silently summarized as an empty paired result.

The current v38 paired effects retain the established directions in all three schedules: dependency-aware fission lowers terminal population and late realized growth while increasing mate limitation relative to fixed-founder households.

## Interpretation of the demographic drag

The experiments separate **intrinsic demographic tendency** from **realized population growth**. Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware household-fission treatment is enabled: mean late realized growth changes from -0.090%/year under fixed-founder households to -1.073%/year under `deterministic_dependency_fission_v2`, while mate limitation rises from 12.0% to 43.2% and mean N240 falls from 107.7 to 19.8.

This does not by itself prove that dependency-aware household fission is scientifically wrong. It establishes that household/mating structure is a major structural treatment on realized demography and therefore cannot be hidden behind a universal demographic default. Other contributing mechanisms may include founder age structure, stochastic sex composition, spatial separation, migration, kin/mate eligibility, spacing and compounding small-population effects. AnthroSim deliberately contains no target-population feedback that would increase fertility or reduce mortality merely because population is falling.

The current v38 long-run analysis still reports overwhelmingly drifting trajectories (741 drifting, 32 insufficient-data and 7 stable runs) and 2 treatment contexts meeting the stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline hidden in the current synthetic controls.

A future focused model-interrogation study should decompose missed reproductive opportunities by gate (age/sex composition, household membership, local male availability, kin restrictions, spacing and geographic separation) before deciding whether any structural rule should change. The purpose of that follow-up is diagnosis, not tuning the simulator toward a desired flat population curve.

## Scientific scope

The result demonstrates **model-form dependence, not prehistoric calibration**. The demography schedules and the dependency-aware household lifecycle are synthetic/structural controls, not empirically calibrated prehistoric presets. The broad result is therefore a study-design constraint: future confirmatory studies must name and justify both demographic and household structure, propagate relevant uncertainty, and test structural alternatives when the conclusion depends on them.

The historical 64-seed/v1 result remains available only as provenance in the explicitly superseded historical page linked above; it must not be used as the current #304 quantitative result. Earlier synchronized outputs likewise remain attached to their original model semantics rather than being relabelled as v37 output.


### Audit-v6 AV6-006 v38 applicability re-verification

AV6-006/#711 changes only M9 equal-cost temporary-destination spatial coupling. Fresh 780-run confirmatory execution `34555842374` under model semantics v38 reproduced the complete normalized demographic scientific result exactly after excluding only execution/model-semantics identity and pre-existing guard-only precision metadata. All six arm summaries, paired household effects, long-run classifications, Monte Carlo precision decisions and the `no_universal_demographic_baseline` recommendation therefore remain unchanged. The checked result is rebound to the fresh v38 research execution rather than being numerically rebaselined.
