# Exploratory model characterisation: M3-only moderate timing with seasonality removed

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and temporal-resolution sensitivity diagnosis  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Does the apparent P12 survivor advantage from the moderate M3-only timing experiment disappear when resource seasonality is removed?

Experiment 9 found broadly similar year-100 outcomes across M3 settlement frequencies at productivity 750 permille, but P12 had the highest mean survivor count. Because that block retained full synthetic seasonality, this follow-up repeats the exact same grid and matched seeds with only `seasonalityScalePermille` changed from 1000 to 0.

This is synthetic model characterisation, not empirical calibration or evidence that any real ecological system should use a particular settlement cadence.

## Design

Identical to Experiment 9 except for seasonality:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity scale 750 permille;
- **seasonality scale 0 permille**;
- permanent migration disabled;
- 30 matched seeds (`7201..7230`) in every treatment;
- M3 `resources.periodsPerYear` = `1, 2, 4, 8, 12, 24`;
- all other synthetic v37 defaults retained.

Total runs: **180**.

## Execution and integrity

Temporary branch: `experiment/v37-m3-only-moderate-no-seasonality`  
Workflow run: `34618042316`  
Accepted artifact: `anthrosim-v37-m3-only-moderate-no-seasonality-results`  
Artifact ID: `10270729195`  
Artifact SHA-256: `ff8faf975950989306c7d3ebd2767780677222cdde459a62b6edfb0358db8cce`

The workflow checked out the exact v37 source commit above and called the unchanged core `Simulation::new(...).run()` path. Integrity checks required:

- exactly 180 rows;
- exactly 30 rows for each requested M3 frequency;
- productivity 750 permille in every row;
- seasonality **0 permille** in every row;
- permanent migration disabled in every row;
- zero permanent migration moves in every row;
- exact `anthrosim-model-semantics-v37` provenance and exact core commit in every row.

All **180/180** runs reached the full 100-year duration.

## Results

### Zero-seasonality M3 grid

| M3 settlements/year | Mean final population | Median | Range | Mean unmet need | Mean condition-mediated deaths | Mean final living condition (permille) |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 94.8 | 94.5 | 60–147 | 69,694 | 869.7 | 994.9 |
| 2 | 93.7 | 91.0 | 48–148 | 67,155 | 848.7 | 991.4 |
| **4 (v37 default)** | **99.6** | **94.5** | **65–186** | **65,523** | **828.6** | **996.8** |
| 8 | 101.8 | 100.5 | 56–163 | 64,971 | 816.6 | 992.3 |
| 12 | 100.2 | 97.0 | 39–169 | 65,880 | 828.9 | 996.3 |
| 24 | 106.3 | 98.0 | 52–175 | 66,475 | 814.0 | 993.8 |

There is still no monotonic relationship between M3 settlement frequency and final population. All six treatments remain in the same broad long-run demographic regime.

### The P12 survivor bump disappears

Experiment 9, with full seasonality, had mean final populations:

- P1: 94.8
- P2: 96.0
- P4: 102.0
- P8: 98.4
- **P12: 107.1**
- P24: 102.4

With seasonality removed, P12 falls to **100.2** and is no longer locally dominant. P24 has the highest mean at 106.3, but the matched-seed contrast against P4 remains broad and includes zero.

Within the zero-seasonality block, P12 versus nearby/reference treatments is:

| Comparison | Mean final-population difference | Approx. paired 95% interval | P12 higher |
|---|---:|---:|---:|
| P12 − P1 | +5.3 | −2.7 to +13.4 | 16/30 (1 tie) |
| P12 − P2 | +6.5 | −1.6 to +14.5 | 18/30 |
| P12 − P4 | +0.5 | −8.1 to +9.1 | 15/30 |
| P12 − P8 | −1.7 | −9.1 to +5.8 | 11/30 |
| P12 − P24 | −6.1 | −15.2 to +2.9 | 13/30 |

That is very different from the seasonal block, where P12 exceeded P1 by +12.3, P2 by +11.1 and P8 by +8.7 on average.

Using the exact same seeds, removing seasonality changed P12 final population by **−6.9 people on average**, approximate paired 95% interval **−13.3 to −0.6**, with the zero-seasonality run lower in 22/30 seeds. Other per-frequency seasonality contrasts were less consistent in survivor count.

Conditional interpretation:

> The local P12 survivor advantage observed in Experiment 9 is not stable when the seasonal resource swing is removed. It is therefore best interpreted as a seasonality × temporal-resolution interaction in this synthetic configuration, not evidence that 12 M3 settlements/year is intrinsically superior.

### Coarse P1/P2 condition-mortality sensitivity remains

Removing seasonality does **not** remove the main coarse-cadence warning.

Relative to P4 in the zero-seasonality block:

| Comparison | Mean condition-death difference | Approx. paired 95% interval | Alternative higher |
|---|---:|---:|---:|
| P1 − P4 | **+41.1** | **+21.1 to +61.1** | 23/30 |
| P2 − P4 | +20.1 | −1.7 to +41.9 | 19/30 |
| P8 − P4 | −12.0 | −32.4 to +8.4 | 13/30 |
| P12 − P4 | +0.3 | −23.7 to +24.4 | 18/30 |
| P24 − P4 | −14.5 | −29.7 to +0.7 | 8/30 |

Experiment 9's seasonal P1 − P4 condition-death difference was +43.8. The zero-seasonality result is therefore almost the same magnitude.

P1 also has +4,170 unmet-need units relative to P4 on average in the zero-seasonality block, approximate paired 95% interval +2,335 to +6,005.

So the coarse annual settlement effect cannot be explained away as seasonal discretisation. Some other within-period nonlinearity—condition loss/recovery, mortality timing, stock/demand timing, or their interaction—still makes very coarse M3 resolution behave differently.

### P1 is exactly unchanged by the seasonality intervention

For P1, the seasonal and zero-seasonality runs are identical across all recorded outcomes for all 30 matched seeds. This is an observed model property of this configuration and is consistent with the seasonal shape having no opportunity to alter within-year timing when there is only one annual resource settlement. It should still be treated as a model-internal observation rather than an empirical claim.

## Interpretation

Plain-language summary:

> **Seasonality explains the apparent monthly/P12 bump, but it does not explain the coarse P1/P2 timing penalty.**

The combined M3-only experiments now suggest two separable sources of timing sensitivity:

1. a **seasonality × temporal-resolution interaction** that can change the relative ordering of intermediate/high M3 frequencies; and
2. a more general **coarse-interval effect** that raises scarcity/condition mortality at very low settlement frequencies even when seasonality is absent.

This is still far smaller than the catastrophic timing sensitivity previously observed when M4 migration's decision frequency and planning horizon were coupled to M3.

## What this does not establish

This experiment does not show that P24 is optimal merely because it has the highest zero-seasonality mean. The P24 − P4 survivor contrast is +6.7 people with an approximate paired 95% interval of −2.4 to +15.7, and the ordering remains noisy across seeds.

It also does not identify the exact mechanism behind the remaining P1/P2 coarse-timing penalty. That requires a more targeted mechanism-isolation experiment.

## Follow-up

The strongest next question is:

> **Which M3 sub-mechanism creates the coarse P1/P2 penalty when seasonality is absent?**

A clean next step is to separate condition-mediated mortality/condition dynamics from pure resource accounting. For example, hold the same zero-seasonality moderate resource setup and compare cadence sensitivity with condition-mediated mortality disabled or neutralised. If the P1/P2 penalty largely disappears, the remaining resolution sensitivity is principally a condition/mortality discretisation effect; if it persists, stock/demand/regeneration timing deserves the next isolation test.

## Reproducibility boundary

The compact artifact is exploratory evidence, not a repository-authoritative research bundle or regression oracle. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
