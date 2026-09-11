# Exploratory model characterisation: M3-only resource timing

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and timing-sensitivity control  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> If permanent migration is completely disabled, how much does M3 resource-settlement frequency alone change the harsh-resource population trajectory?

The preceding clock experiments found very large interactions between the M3 resource clock and M4 permanent-migration clock. This control removes M4 permanent migration entirely so that any remaining timing sensitivity belongs to M3/resource-demography dynamics rather than migration-clock coupling.

This is model characterisation, not evidence for a real ecological accounting cadence or prehistoric behaviour.

## Design

Common configuration:

- 100 simulated years requested;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity scale 150 permille (15% of the synthetic baseline scale);
- seasonality scale 1000 permille;
- **permanent migration disabled**;
- 30 matched seeds (`7101..7130`) in every treatment;
- all other synthetic defaults retained.

Only M3 `resources.periodsPerYear` varied:

- `1, 2, 4, 8, 12, 24` settlements/year.

Total runs: **180**.

## Why this is a useful control

Under v37, changing M3 settlement frequency does not simply multiply annual demand or annual regeneration. The resource mechanism allocates annual food need and seasonal regeneration across configured intervals and rescales condition recovery/loss and mortality probability by elapsed interval duration. M3 frequency is therefore intended to behave largely as a temporal-resolution/cadence choice around the same annual quantities.

A large survival change in this control would consequently indicate meaningful within-year path dependence or discretisation sensitivity rather than a trivial multiplication of annual food demand.

## Execution and integrity

The experiment used a temporary compact runner on `experiment/v37-m3-only-timing`. The workflow checked out the exact v37 source commit above and called the unchanged core `Simulation::new(...).run()` path.

The accepted enriched run (`34610680047`) retained compact per-run outcomes including extinction timing. Integrity checks and artifact inspection found:

- exactly **180** rows;
- exactly **30** runs for each of the six requested M3 frequencies;
- every row used the requested `resources.periodsPerYear` value;
- **180/180** rows had permanent migration disabled;
- **180/180** rows completed **zero** permanent migration moves;
- **180/180** rows reported `anthrosim-model-semantics-v37` and source commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`;
- all 180 populations eventually reached `populationExtinct` before the requested 100-year horizon.

The first accepted compact grid was subsequently rerun only to add `statistics.simulatedDays` / resource-period observability. Every previously recorded overlapping outcome (final population, unmet need, condition-mediated deaths, migration counts, stop reason and provenance) reproduced exactly for all 180 seed/configuration pairs.

Two earlier workflow attempts are not scientific failures: their model runs completed, but post-run shell/validation logic rejected the job before a usable artifact was accepted. The final run corrected that harness validation without changing the experiment configuration.

## Results

Because every treatment eventually became extinct, final population at year 100 is uninformative (`0` for all 180 runs). The primary comparison is therefore **time to extinction**.

| M3 settlements/year | Mean extinction time (years) | Median (years) | Range (years) | Mean unmet need | Mean condition-mediated deaths |
|---:|---:|---:|---:|---:|---:|
| 1 | 40.90 | 38.50 | 11.00–81.00 | 263,762 | 2,008.1 |
| 2 | 39.85 | 40.25 | 11.50–79.00 | 257,663 | 2,001.9 |
| **4 (v37 default)** | **38.19** | **41.87** | **11.50–76.25** | **255,642** | **1,991.4** |
| 8 | 39.47 | 42.31 | 10.37–76.75 | 256,398 | 1,984.5 |
| 12 | 39.01 | 41.29 | 11.33–80.67 | 255,447 | 1,985.7 |
| 24 | 41.50 | 44.10 | 10.37–84.46 | 256,314 | 1,985.2 |

### Extinction timing is noisy rather than monotonic

Using matched seeds, mean extinction-time differences relative to the default `P=4` were:

| Comparison | Mean difference in extinction time | Approx. paired 95% interval | Alternative lasted longer |
|---|---:|---:|---:|
| P1 − P4 | +2.71 years | −6.87 to +12.29 | 15/30 (1 tie) |
| P2 − P4 | +1.66 years | −7.10 to +10.42 | 18/30 |
| P8 − P4 | +1.28 years | −8.00 to +10.56 | 17/30 |
| P12 − P4 | +0.82 years | −9.28 to +10.91 | 14/30 |
| P24 − P4 | +3.31 years | −6.47 to +13.09 | 19/30 |

These are descriptive exploratory intervals, not confirmatory inference. All are broad and include zero. The identity of the longest-lasting treatment also varied across seeds rather than concentrating on one M3 frequency.

There is therefore no evidence in this block for a simple rule such as “more frequent M3 settlement always helps” or “the default four/year is intrinsically best.”

### The coarsest clock shows somewhat greater scarcity burden

Although extinction time is not robustly ordered, `P=1` accumulated more unmet resource need and condition-mediated mortality than the default `P=4` across matched seeds:

- unmet need: P1 − P4 = **+8,120** units on average, approximate paired 95% interval **+6,183 to +10,057**, P1 higher in 29/30 seeds;
- condition-mediated deaths: P1 − P4 = **+16.7** on average, approximate paired 95% interval **+11.4 to +22.0**, P1 higher in 26/30 seeds.

These are cumulative lifetime quantities and should not be interpreted as calibrated rates. Treatment lifetimes differ, births occur before extinction, and the population histories are highly stochastic. They nevertheless suggest that collapsing the resource system to one large annual settlement changes within-year scarcity/condition dynamics somewhat even though the eventual extinction outcome remains the same.

Frequencies `P=4, 8, 12, 24` are much closer in cumulative unmet need (about 255k–256k) and condition-mediated deaths (about 1,985–1,991) than `P=1` is to the default.

## Interpretation

Conditional model statement:

> Under the tested harsh-resource v37 configuration with permanent migration disabled, changing M3 resource-settlement frequency from 1 to 24/year does not produce a stable monotonic or sharply peaked survival response. All 180 populations eventually go extinct, and matched-seed extinction-time differences among the tested M3 frequencies are small relative to stochastic variation. A very coarse one-settlement/year clock does, however, produce somewhat greater cumulative scarcity and condition-mediated mortality than the default four/year.

Plain-language summary:

> M3's clock matters somewhat, especially when made extremely coarse, but **M3 alone does not show anything like the huge timing sensitivity seen when M3 and M4 migration clocks were coupled**.

That strengthens the interpretation that the dramatic clock-alignment effects found previously arise primarily from interaction with M4's decision/planning-horizon semantics rather than from a standalone M3 preference for matching frequencies.

## What this does not establish

This experiment does **not** establish that M3 temporal resolution is generally irrelevant. The tested environment is deliberately severe and every no-migration population ultimately collapses. A less severe environment in which populations remain viable could expose differences that terminal extinction obscures.

It also does not validate any particular settlement frequency against empirical ecology or archaeology. The synthetic resource parameters, initial conditions and condition/mortality mappings remain uncalibrated.

## Follow-up

The strongest next question is:

> **Does M3 settlement frequency remain this weakly influential in a less severe resource environment where no-migration populations survive for the full 100 years?**

Repeating the M3-only control at a moderate productivity level would replace “time until universal extinction” with long-run population size and condition outcomes. If M3 frequencies remain broadly similar there, we gain stronger evidence that M3's elapsed-time scaling is reasonably resolution-stable. If they diverge materially, temporal discretisation deserves targeted diagnosis before empirical use.

## Reproducibility boundary

The compact output is exploratory evidence, not a repository-authoritative research bundle or regression oracle. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
