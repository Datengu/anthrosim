# Exploratory model characterisation: fixed-condition mortality-hazard partitioning

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** mechanism isolation for residual M3 temporal-resolution sensitivity  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> When condition is held fixed, is condition-mediated mortality itself stable when the same five-year exposure is partitioned into different numbers of M3 intervals?

Experiment 11 showed that disabling condition-mediated mortality removes the systematic coarse P1/P2 demographic penalty while leaving a coarse resource-accounting difference. That result did not distinguish whether the mortality penalty came from different condition trajectories or from the mortality hazard's own temporal discretisation.

This experiment removes that ambiguity by holding condition constant and making condition-mediated mortality the only possible cause of death.

This is synthetic model verification/characterisation. It does not calibrate or validate the mortality coefficient for any real population.

## Relevant v37 mechanism

For each living person, v37 converts condition deficit into a reference quarter-year mortality probability:

`reference probability = (1000 - condition) / 1000 × max condition-mortality probability`.

It then calls `reference_quarter_probability_for_interval(...)` for the current M3 interval. The implementation states that interval hazards are constructed from survival ratios so sub-interval survival factors telescope back to the same reference-quarter survival.

The v37 source also contains a unit test, `fixed_condition_mortality_survival_is_partition_invariant`, which checks exact composed-survival equality for reference probabilities 0, 200,000, 500,000 and 1,000,000 ppm across P=1, 4, 12 and 365.

The present experiment extends that check through the complete simulation path, adds P=2, 8 and 24, and tests lower reference hazards generated from realistic-in-model intermediate condition values.

## Design

Common configuration:

- exact historical v37 core;
- 5 simulated years;
- 2,000 explicitly declared founders;
- one synthetic cell and one founder household;
- all founders age 30 and male, solely to prevent any reproductive ambiguity;
- all fertility probabilities = 0;
- all background demographic mortality probabilities = 0;
- annual food need = 0;
- condition recovery = 0;
- maximum condition loss = 0;
- seasonality = 0;
- permanent migration disabled;
- maximum condition-mortality probability = 200,000 ppm per reference quarter at condition zero;
- M3 periods/year P = 1, 2, 4, 8, 12, 24;
- 30 matched seeds `8101..8130` in the primary grid.

Fixed founder conditions:

- 250 permille -> reference-quarter mortality probability 150,000 ppm;
- 500 permille -> 100,000 ppm;
- 750 permille -> 50,000 ppm;
- 900 permille -> 20,000 ppm.

Because resource need and condition response are both zero, every surviving person's condition remains exactly at the declared starting value. Because fertility and background mortality are zero, every death is condition-mediated. Therefore:

`final living population + condition-mediated deaths = 2000`

for every run.

Primary grid: **720 runs** (4 conditions × 6 cadences × 30 seeds).

## Primary execution and integrity

Temporary branch: `experiment/v37-fixed-condition-mortality-hazard`  
Workflow run: `34620230549`  
Harness commit: `a369b59970fbee9b48f98618f44c7154ee21f51d`  
Artifact: `anthrosim-v37-fixed-condition-mortality-hazard-results`  
Artifact ID: `10272192779`  
Artifact SHA-256: `557948d3042cefa8f76532241bfe516b303ac510e18de3951dbb19012e8559d1`

The workflow hard-checked:

- exactly 720 rows;
- 30 runs for every condition × cadence cell;
- declared P equals executed P;
- zero food need, zero condition recovery and zero condition loss;
- 200,000 ppm configured maximum condition mortality;
- migration disabled with zero moves;
- zero resource unmet need;
- final population + condition deaths = 2,000 in every run;
- surviving mean condition equals the declared fixed condition;
- every run reaches the full five-year duration;
- exact v37 semantics/source provenance.

## Primary results

Mean final living population after five years:

| Fixed condition | Theoretical quarter-composed expectation | P1 | P2 | P4 | P8 | P12 | P24 |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 250 | 77.52 | 77.87 | 78.77 | 79.10 | 74.47 | 75.13 | 77.67 |
| 500 | 243.15 | 241.67 | 239.80 | 248.10 | 244.07 | 242.87 | 244.23 |
| 750 | 716.97 | 712.90 | 721.73 | 714.47 | 720.83 | 711.83 | 718.73 |
| 900 | 1335.22 | 1339.43 | 1328.70 | 1334.77 | 1331.23 | 1331.40 | 1334.93 |

The theoretical expectation is `2000 × (1 - q)^20`, where `q` is the fixed reference-quarter death probability and five years contain 20 reference quarters.

The important pattern is the absence of a coherent cadence ordering. P1 is not systematically worse, high P is not systematically better or worse, and means fluctuate around the same condition-specific expectation.

Two of 20 exploratory P-versus-P4 paired contrasts were marginally non-zero in this 30-seed block: condition 250 P8−P4 and condition 500 P1−P4. Because different P values consume different numbers/sequences of mortality RNG draws, equal seed labels across partitions are not exact common-random-number coupling. Those isolated contrasts were therefore treated as a reason for independent confirmation rather than as evidence of a cadence effect.

## Independent confirmation

A larger independent block targeted the potentially sensitive comparisons:

- fixed conditions 250 and 500;
- P=1, 4 and 8;
- 200 new seeds `8201..8400`;
- **1,200 additional runs**.

Temporary branch: `experiment/v37-fixed-condition-mortality-hazard-confirm`  
Workflow run: `34620522994`  
Harness commit: `a7b63fd53302584b812ea4d1353a471ea7d625d1`  
Artifact: `anthrosim-v37-fixed-condition-mortality-hazard-confirm-results`  
Artifact ID: `10272590822`  
Artifact SHA-256: `5ffa7778e62ad0770c8da4b2c3328540829ff3f10ff4aea077b9b41d8170d2b9`

All 1,200 confirmation rows passed the same relevant configuration/provenance and accounting checks.

Confirmation means and ordinary across-seed approximate 95% intervals:

| Condition | P | Mean survivors | Approx. 95% interval | Theoretical expectation |
|---:|---:|---:|---:|---:|
| 250 | 1 | 78.32 | 77.01–79.63 | 77.52 |
| 250 | 4 | 78.46 | 77.12–79.80 | 77.52 |
| 250 | 8 | 77.12 | 75.95–78.29 | 77.52 |
| 500 | 1 | 243.76 | 241.72–245.80 | 243.15 |
| 500 | 4 | 244.28 | 242.39–246.18 | 243.15 |
| 500 | 8 | 244.12 | 241.88–246.37 | 243.15 |

Every confirmation interval contains the exact quarter-composed expectation.

The independent paired contrasts that motivated confirmation are also no longer persuasive:

- condition 250, P8−P4 = −1.34 survivors, approximate paired 95% interval −3.05 to +0.37;
- condition 500, P1−P4 = −0.53, approximate paired 95% interval −3.28 to +2.23.

## Interpretation

Plain-language result:

> **With condition fixed, v37 condition-mediated mortality is effectively stable to the tested M3 interval partitioning.**

The full simulation agrees with the intended survival-ratio construction and the existing exact unit-level partition-invariance test. The small treatment-to-treatment fluctuations behave like stochastic sampling rather than a systematic coarse- or fine-cadence mortality bias.

This sharply narrows the diagnosis from Experiments 10 and 11:

1. the residual P1/P2 demographic penalty is **not** principally caused by the condition-mortality hazard rescaling itself;
2. condition-mediated mortality is instead transmitting differences created upstream;
3. the remaining mechanism to isolate is the **resource scarcity / condition trajectory produced by coarse M3 settlement**, including stock, regeneration and demand timing.

## Methodological note on matched seeds

Using the same seed label across different P values does not make mortality draws person-for-person identical. Different temporal partitions consume the condition-mortality RNG stream differently. Paired contrasts are therefore useful diagnostics but are not pure common-random-number estimates once the partitions diverge.

For this fixed-hazard question, comparison to the analytically implied survival expectation and independent treatment ensembles is the stronger test.

## What this does not establish

This result does not validate the synthetic 200,000-ppm maximum mortality coefficient, the linear relationship between condition deficit and reference mortality, or any real prehistoric mortality process.

It establishes only that, under the tested v37 mechanism, temporal partitioning does not introduce a meaningful additional mortality bias when condition itself is held fixed.

## Follow-up

The strongest next question is now:

> **With population held fixed and all condition/mortality effects disabled, how much resource scarcity is created purely by M3 stock/regeneration/demand timing?**

A fixed-population resource-accounting experiment should remove births, all mortality, migration and condition response while varying only M3 settlement cadence. If P1 still accumulates substantially more unmet need, the remaining coarse effect is directly in resource stock/regeneration/demand timing rather than downstream physiology.

## Reproducibility boundary

These outputs remain exploratory model-characterisation evidence rather than a canonical study bundle or empirical validation result. Claim-driving use requires reviewed experiment definitions, retained authoritative outputs and the appropriate scientific gates.
