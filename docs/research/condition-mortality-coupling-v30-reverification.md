# Model-semantics v30 re-verification — AV4-006 condition-mortality coupling

This note records the deliberate scientific-reference review for Audit-v4 AV4-006 / #497. The production candidate reviewed here is `340e57ca7a864f6ae3b689f927ecda5e67db1a98` with `anthrosim-model-semantics-v30` and checkpoint schema 18. The repair changes same-seed assignment of condition-mediated mortality draws and simultaneous condition/background cause attribution, so downstream frozen results were rerun rather than relabelled.

## #304 confirmatory demographic baseline

Run `33820753145`, job `100862690900`, artifact `9918271667` (`sha256:76a943bae2a8a3bef13ee551f17edeb6d485897b03a52d8607ebe3deeb3bb634`) completed all **780/780** confirmatory arm-runs. All three predeclared Monte Carlo gates returned `sufficient_stop`; the recommendation remains **`no_universal_demographic_baseline`**.

| demography | household lifecycle | mean N240 | late growth | extinction | mate limitation |
| --- | --- | ---: | ---: | ---: | ---: |
| negative_growth_control_v1 | deterministic_dependency_fission_v2 | 2.02 | -2.492% | 57.692% | 37.839% |
| negative_growth_control_v1 | fixed_founder_v1 | 13.43 | -1.243% | 16.923% | 22.019% |
| positive_growth_control_v1 | deterministic_dependency_fission_v2 | 24.16 | -1.049% | 3.846% | 42.518% |
| positive_growth_control_v1 | fixed_founder_v1 | 104.36 | -0.071% | 0.000% | 12.966% |
| replacement_control_v1 | deterministic_dependency_fission_v2 | 6.42 | -1.915% | 29.231% | 40.858% |
| replacement_control_v1 | fixed_founder_v1 | 49.06 | -0.428% | 1.538% | 17.117% |

Paired fission-minus-fixed effects:

| demography | terminal N | late growth | mate limitation |
| --- | ---: | ---: | ---: |
| negative_growth_control_v1 | -11.42 | -1.265 pp/yr | 15.820 pp |
| positive_growth_control_v1 | -80.20 | -0.978 pp/yr | 29.552 pp |
| replacement_control_v1 | -42.64 | -1.509 pp/yr | 23.742 pp |

Long-run classification counts are `{'drifting': 745, 'insufficient_data': 27, 'stable': 8}`; stochastic multi-regime contexts = `2`; environment dependence = `False`; initialization dependence = `False`.

## M7.6 resource-variability reference

Review run `33821069979`, job `100863637973`, artifact `9918382546` (`sha256:3f27490d2c30098aab71ccc83d4f0ad2f822af708503694dae5b7f525cdd3920`) checked out the exact production candidate and completed **144/144** canonical runs. Exact point values change under corrected condition-mortality coupling, but the declared synthetic validation conclusion survives unchanged: each of the three 250-permille no-migration points goes extinct in 8/8 seeds while the matched migration-enabled point persists in 8/8, and migration-enabled runs have higher terminal population, lower condition-mediated mortality, and lower unmet resource need at **all 9/9** matched productivity/seasonality points.

## M8.6 terrain null-model benchmark

Run `33820753353`, job `100862718819`, artifact `9918254916` (`sha256:da9a934eb46176d7f8acc4e74f08a628ac72881c0756e7840a365420ea4ed292`) completed all four declared arms and all 32 runs. The exact trajectories and paired effects change, but the benchmark classification remains **`fragile_spatial_structure`** with no robust metrics. `terminalPopulationHerfindahlPerMillion` and `terminalLargestCellSharePermille` remain fragile; migration distance and occupied-cell time remain not distinctive.

Strong-arm median absolute relative effects are:

- migration total distance: `8.69%`;
- occupied-cell time: `1.40%`;
- terminal Herfindahl: `16.44%`;
- largest-cell share: `37.25%`.

## M9.7 controlled aggregation benchmark

The same applicable-gate run `33820753353`, job `100862718824`, artifact `9918256387` (`sha256:98df4dd2bb411cc7dee7652c87466045e2ea53cb895686863245277c4c0d7cfc`) passed **without any reference change**. Both ensembles, M9.6 observability, identical replay, active annual checkpoint/resume, the preserved M9.7 scientific reference, and tamper rejection all passed. The M9 machine reference is therefore intentionally left untouched.

## Rebaseline decision

Only #304, M7.6, and M8.6 are rebaselined, from their complete archived v30 outputs. M9.7 is not rebaselined because its existing scientific reference already matches v30 exactly. No threshold, gate, seed set, design, or scientific acceptance rule is weakened. The exact original Audit-v4 #496 adversary remains mandatory after production merge before #497 can close.
