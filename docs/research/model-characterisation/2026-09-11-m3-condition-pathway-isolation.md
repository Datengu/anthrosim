# Exploratory model characterisation: M3 coarse-timing condition-pathway isolation

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** mechanism isolation for residual M3 temporal-resolution sensitivity  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Is the residual coarse P1/P2 timing penalty seen after seasonality is removed mainly transmitted through condition loss/recovery and condition-mediated mortality, rather than through resource accounting itself?

Experiment 10 showed that removing seasonality removes the local P12 survivor bump but leaves a coarse-cadence signal: at P1, condition-mediated deaths remained about 41 higher than P4 on average and unmet need remained about 4,170 units higher. This follow-up isolates the condition pathway while leaving food stock, demand and regeneration accounting intact.

This is synthetic model characterisation. It does not validate a real mortality process or a real ecological accounting cadence.

## Design

Common setup matches Experiment 10:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity 750 permille;
- seasonality 0 permille;
- permanent migration disabled;
- matched seeds `7201..7230`;
- M3 `resources.periodsPerYear` = `1, 2, 4, 8, 12, 24`;
- exact historical v37 core.

Two v37 configuration arms were run:

1. **mortality-off** — retain the v37 condition response (`conditionRecoveryPerPeriod=25`, `maxConditionLossPerPeriod=200`) but set `maxConditionMortalityProbabilityPerMillion=0`;
2. **condition-neutral** — also set condition recovery and maximum condition loss to zero, so living condition remains at the founder value while condition-mediated mortality remains disabled.

Total new runs: **360** (2 arms × 6 cadences × 30 seeds). Experiment 10 supplies the matched canonical zero-seasonality comparison with condition-mediated mortality active.

## Execution and integrity

Temporary branch: `experiment/v37-m3-condition-pathway`  
Accepted workflow run: `34618897346`  
Accepted artifact: `anthrosim-v37-m3-condition-pathway-results`  
Artifact ID: `10271845889`  
Artifact SHA-256: `560358cddbe63038b2dfb72e1a9933a71782ad3d9b0996173985cffe5140b81c`

The accepted workflow checked out the exact v37 source commit and used the unchanged core simulation path. It hard-checked:

- exactly **360** result rows;
- **180** rows per arm;
- exactly 30 rows per cadence in each arm;
- productivity 750 and seasonality 0 in every row;
- migration disabled and zero migration moves in every row;
- condition mortality probability 0 in every row;
- mortality-off retained recovery 25 / max loss 200;
- condition-neutral used recovery 0 / max loss 0;
- zero condition-mediated deaths in all 360 runs;
- exact `anthrosim-model-semantics-v37` / source-commit provenance.

An earlier workflow run (`34618755691`) failed at Rust compilation because the temporary runner returned a borrowed string from its sorting key. No simulations executed in that failed attempt. The sort key was changed to a numeric arm order; no scientific configuration changed.

All accepted runs reached the full 100-year duration.

## Results

### Disabling condition-mediated mortality changes the demographic regime

Mean final population in Experiment 10 (condition mortality active) versus the mortality-off arm:

| M3 settlements/year | Exp. 10 canonical | Mortality off | Mean increase |
|---:|---:|---:|---:|
| 1 | 94.8 | 725.5 | +630.7 |
| 2 | 93.7 | 720.6 | +626.9 |
| 4 | 99.6 | 728.2 | +628.6 |
| 8 | 101.8 | 705.6 | +603.8 |
| 12 | 100.2 | 734.4 | +634.2 |
| 24 | 106.3 | 735.1 | +628.8 |

Every one of the 180 matched seed/cadence runs finishes with more people when condition-mediated mortality is disabled. This large level shift is expected from removing an active mortality mechanism and should not be interpreted as evidence that the mortality mechanism is empirically too strong; its parameters remain synthetic and uncalibrated.

More important for this experiment, the previous coarse survivor ordering does **not** become a stable alternative optimum. In the mortality-off arm, paired final-population differences versus P4 are:

| Comparison | Mean difference | Approx. paired 95% interval |
|---|---:|---:|
| P1 − P4 | −2.7 | −27.4 to +22.0 |
| P2 − P4 | −7.6 | −37.5 to +22.3 |
| P8 − P4 | −22.6 | −55.3 to +10.0 |
| P12 − P4 | +6.2 | −18.5 to +30.8 |
| P24 − P4 | +6.9 | −24.8 to +38.5 |

All are broad and include zero. Thus the systematic coarse **condition-death** difference from Experiment 10 no longer translates into an ordered demographic penalty once the condition-mediated mortality channel is removed.

### Neutralising condition itself adds no further causal effect here

The mortality-off and condition-neutral arms are exactly identical for **all 180 matched seed/cadence pairs** in:

- final living population;
- cumulative unmet need;
- condition-mediated death count (zero);
- simulated duration and stop reason;
- migration counts (zero).

Their only intended difference is the condition state itself: the condition-neutral arm remains at mean living condition 1000, whereas mortality-off retains v37 scarcity-driven condition loss/recovery and finishes with mean living condition around 206–228 across the tested cadences.

Conditional interpretation:

> With migration disabled and condition-mediated mortality disabled, changing condition loss/recovery no longer changes population or resource accounting in this experiment. The condition state is therefore not an independent demographic pathway in this configuration; its relevant demographic transmission is through condition-mediated mortality.

### Resource-accounting cadence sensitivity remains

Removing the mortality pathway does **not** make M3 resource accounting cadence-invariant. In the mortality-off arm mean cumulative unmet need is:

| M3 settlements/year | Mean unmet need |
|---:|---:|
| 1 | 3,026,156 |
| 2 | 2,880,717 |
| 4 | 2,881,971 |
| 8 | 2,807,747 |
| 12 | 2,833,025 |
| 24 | 2,788,979 |

P1 − P4 is **+144,186** unmet units on average, with an approximate paired 95% interval of **+29,236 to +259,136**. The absolute unmet-need totals are much larger than Experiment 10 because removing condition-mediated mortality leaves hundreds more people alive and therefore creates much greater cumulative food demand.

The important point is qualitative: even without seasonality, migration or condition-mediated mortality, the one-settlement/year treatment still produces more cumulative unmet need than P4 in this block. A residual coarse resource-accounting/path-dependence effect therefore remains.

### Why final population is not seed-for-seed identical across P

Mortality-off does not imply that all mortality is disabled. v37 evaluates background demographic mortality on the same elapsed intervals used by the M3 resource boundary, converts the age-specific annual probability to the current interval, and draws from the background-mortality RNG at those boundaries.

Changing `resources.periodsPerYear` therefore changes the partition and sequence of background-mortality draws even when condition mortality is zero. The annual risk is interval-scaled, so the scientifically relevant expectation is distributional stability rather than exact seed-for-seed identity across different temporal partitions. Consistent with that, the final-population P-versus-P4 contrasts above are noisy and all span zero.

This also means matched seeds across different P values are not perfect common-random-number coupling for background mortality after the clocks diverge.

## Interpretation

Plain-language summary:

> **The coarse P1/P2 demographic penalty requires the condition-mediated mortality pathway, but the underlying coarse resource-accounting difference does not disappear when that pathway is removed.**

The two condition-isolation arms further show that condition loss/recovery has no separate demographic effect in this migration-off experiment once condition mortality is disabled. It changes condition state, but not population or resource accounting.

The timing diagnosis is therefore now more specific:

1. seasonality explains the local P12 bump from Experiment 9;
2. very coarse M3 cadence still changes resource scarcity/accounting even without seasonality;
3. condition-mediated mortality converts scarcity/condition differences into extra deaths;
4. background demographic mortality is itself partitioned on M3 boundaries, creating expected stochastic path differences across cadence treatments even when condition mortality is zero.

This remains much smaller and qualitatively different from the severe M3/M4 clock-coupling pathology diagnosed earlier.

## What this does not establish

This experiment does **not** yet tell us whether the coarse condition-mortality penalty originates primarily from:

- coarse condition loss/recovery trajectories; or
- the interval partitioning of the condition-mortality hazard itself.

Turning mortality off removes both the demographic consequence and the ability to distinguish those two sources.

## Follow-up

The strongest next question is:

> **When condition is held at a controlled fixed level, is condition-mediated mortality itself invariant to M3 interval partitioning?**

A small controlled hazard experiment can hold people at the same condition while varying only the number/length of mortality intervals. If cumulative mortality remains stable across P, the remaining coarse penalty comes mainly from condition trajectories/resource response. If mortality changes systematically with P even at fixed condition, the mortality-hazard discretisation itself needs targeted repair or stronger verification.

## Reproducibility boundary

These compact outputs are exploratory evidence, not a repository-authoritative study bundle or regression oracle. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
