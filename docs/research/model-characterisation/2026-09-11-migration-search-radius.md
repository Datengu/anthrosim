# Exploratory model characterisation: migration search radius

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and sensitivity-test design  
**Model target:** `anthrosim-model-semantics-v37`  
**Executable source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Under otherwise fixed synthetic assumptions, how strongly does the distance over which households can search for a new residence affect long-run survival under resource pressure?

In plain language: does letting a household inspect progressively more nearby cells always help, or can a wider search become harmful because longer moves have costs?

This is a model-characterisation question, not an archaeological or anthropological claim.

## Design

The experiment reused the same v37 CI-built executable as the preceding migration/resource exploratory experiments.

Common configuration:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- seasonality scale 1000 permille;
- 30 matched seeds (`1..30`) per condition;
- two resource-productivity conditions: 150 and 250 permille;
- all other synthetic M3/M4 parameters left at the executable defaults.

Migration treatments:

- migration disabled;
- radius 1;
- radius 2;
- radius 3;
- radius 4;
- radius 6.

`migration-radius` is a Manhattan-distance local knowledge horizon. It controls which nearby cells a household may inspect as candidate permanent residences; it is not a real-world distance measurement.

Total runs: **360** (`2 productivity levels × 6 migration treatments × 30 seeds`). All runs completed.

## Results

### Final living population after 100 years

| Productivity | Migration disabled | Radius 1 | Radius 2 | Radius 3 | Radius 4 | Radius 6 |
|---:|---:|---:|---:|---:|---:|---:|
| 150 | 0.0 | 451.8 | 501.8 | **523.4** | 478.2 | 380.9 |
| 250 | 0.0 | 639.6 | 648.5 | **652.2** | 616.5 | 563.2 |

All 30/30 disabled-migration runs were extinct at both tested productivity levels. No radius-1 through radius-6 run became extinct.

The response was therefore **non-monotonic**: increasing radius from 1 toward 2–3 helped or approximately plateaued, but increasing it further to 4–6 reduced final population.

### Paired-seed contrasts

Because every treatment used the same 30 seeds, final-population differences can be compared seed-for-seed.

At productivity 150:

- radius 2 minus radius 1: mean `+50.0`; approximate 95% interval `+25.8 .. +74.2`;
- radius 3 minus radius 2: mean `+21.6`; approximate interval `+2.5 .. +40.7`;
- radius 4 minus radius 3: mean `-45.2`; approximate interval `-67.3 .. -23.1`;
- radius 6 minus radius 3: mean `-142.5`; approximate interval `-160.9 .. -124.2`, with radius 6 lower in **30/30** matched seeds.

At productivity 250:

- radius 2 minus radius 1: mean `+8.9`; approximate interval `-13.5 .. +31.3`;
- radius 3 minus radius 2: mean `+3.7`; approximate interval `-15.5 .. +23.0`;
- radius 4 minus radius 3: mean `-35.7`; approximate interval `-63.3 .. -8.1`;
- radius 6 minus radius 3: mean `-89.1`; approximate interval `-117.7 .. -60.5`, with radius 6 lower in 26/30 matched seeds.

These intervals are descriptive exploratory summaries, not promoted inferential claims.

## Why does a wider search eventually hurt?

The model exposes a clear trade-off.

### Productivity 150

| Radius | Mean move distance (cells) | Mean unmet food need | Mean cumulative travel-condition loss | Mean condition-mediated deaths | Final living population |
|---:|---:|---:|---:|---:|---:|
| 1 | 1.000 | 52,277 | 936,350 | 295.2 | 451.8 |
| 2 | 1.569 | 36,413 | 1,460,507 | 192.3 | 501.8 |
| 3 | 2.068 | 30,460 | 1,867,853 | 210.5 | **523.4** |
| 4 | 2.502 | 25,076 | 2,090,468 | 245.0 | 478.2 |
| 6 | 3.262 | **18,059** | **2,282,168** | **345.1** | 380.9 |

### Productivity 250

| Radius | Mean move distance (cells) | Mean unmet food need | Mean cumulative travel-condition loss | Mean condition-mediated deaths | Final living population |
|---:|---:|---:|---:|---:|---:|
| 1 | 1.000 | 18,733 | 615,296 | 77.6 | 639.6 |
| 2 | 1.555 | 11,152 | 842,351 | **43.8** | 648.5 |
| 3 | 2.036 | 9,568 | 1,054,732 | 56.0 | **652.2** |
| 4 | 2.455 | 8,110 | 1,203,379 | 81.1 | 616.5 |
| 6 | 3.163 | **6,632** | **1,402,207** | **151.4** | 563.2 |

A larger search horizon allows households to reach better-supported destinations and therefore reduces unmet food demand. However, selected moves become longer on average. M4 charges completed movement an explicit condition cost proportional to distance. That cumulative travel-condition loss rises strongly with radius. Because shared condition also contributes to condition-mediated mortality, the benefit of improved resource access eventually becomes outweighed by the cost of farther movement under these synthetic defaults.

The model therefore exhibits an interior trade-off rather than the simple rule "more mobility range is always better".

## Interpretation

Conditional model statement:

> Under the v37 synthetic resource and migration assumptions tested here, access to at least a small local migration neighbourhood was extremely important under resource pressure, but progressively widening that neighbourhood was not monotonically beneficial. Radius 2–3 produced the strongest long-run survival of the tested settings; larger radii reduced unmet resource demand further but increased travel-condition costs enough to reduce survival.

This is useful model knowledge because it shows that `migration.candidateRadiusCells` is not merely a harmless search-performance setting. It interacts materially with movement cost and condition-mediated mortality and can change the direction and magnitude of outcomes.

It also means a future empirical study must not interpret a migration-enabled result without examining plausible uncertainty in search/information horizon and movement cost. A site-specific conclusion could otherwise be driven by an arbitrary synthetic mobility radius.

## What this does not establish

This experiment does **not** show that real prehistoric households had an optimal migration radius, that wider geographic knowledge was harmful, or that travel mortality behaved this way. The cell scale, information horizon, utility terms, travel-condition cost and mortality mapping are synthetic placeholders.

The observed radius-2/3 "sweet spot" is a property of this model configuration and semantics, not a human behavioural estimate.

## Follow-up

The strongest next diagnostic is to separate the two coupled assumptions responsible for the trade-off:

1. hold search radius fixed while sweeping `travelConditionCostPerCell`; and/or
2. hold movement cost fixed while varying migration decision frequency or the minimum utility improvement required to move.

That would determine whether the radius optimum is primarily generated by the explicit distance cost, by repeated movement opportunities, or by their interaction.

## Reproducibility boundary

As with the preceding records, the conversational runtime directory is not a repository-authoritative finalized study bundle. This document records the exact design and aggregate results from the v37 executable. Any claim-driving use should be rerun from a checked-in experiment definition with authoritative retained outputs and the appropriate scientific gates.
