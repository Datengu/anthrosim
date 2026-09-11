# Exploratory model characterisation: migration travel-condition cost

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and sensitivity-test design  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Is the apparent radius-2/3 migration "sweet spot" mainly produced by AnthroSim's assumed physical cost of travelling?

In plain language: the preceding search-radius experiment found that letting households search somewhat farther helped, but letting them search much farther eventually hurt survival. This experiment asks whether that pattern remains when the physical condition penalty charged for each cell travelled is changed.

This is a model-characterisation question. It does not estimate real prehistoric travel costs or an optimal real-world migration distance.

## Design

Common configuration:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity scale 150 permille (15% of the synthetic baseline scale);
- seasonality scale 1000 permille;
- permanent migration enabled;
- 30 matched seeds (`4101..4130`) in every treatment;
- all other synthetic M3/M4 parameters left at their v37 defaults.

Two migration assumptions were crossed:

- candidate search radius: `1, 2, 3, 4, 6` cells;
- `travelConditionCostPerCell`: `0, 2, 5, 10, 20, 40`.

The v37 synthetic default for `travelConditionCostPerCell` is `10`. A cost of `0` is deliberately artificial and is used as a mechanism-isolation test: it asks what the model does when completed movement itself causes no physical-condition decrement.

Total runs: **900** (`6 costs × 5 radii × 30 matched seeds`).

## Execution and integrity checks

The ordinary CLI does not expose `travelConditionCostPerCell`, and the first full-bundle attempt was operationally unsuitable for a 900-run exploratory grid because the retained histories exhausted the local 32 GB filesystem. Those disk failures were not simulation outcomes and were discarded.

The completed experiment therefore used a temporary compact runner created only on the isolated `experiment/v37-travel-cost-runner` branch. The workflow checked out the exact v37 core source commit above, constructed ordinary `ExperimentConfig` values directly, called the unchanged core `Simulation::new(...).run()` path, and retained only the compact per-run measurements needed for this question. The workflow source commit that executed the compact grid was `3f446f9c4e64bdc1b89d0c9e86fa2a753450c940`.

Integrity checks on the resulting 900 rows found:

- exactly 150 runs at each tested travel cost;
- exactly 180 runs at each tested radius;
- **900/900** rows reported `anthrosim-model-semantics-v37`;
- **900/900** rows reported core source commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`;
- **0/900** requested/configured travel-cost mismatches;
- **0/900** requested/configured radius mismatches;
- **900/900** runs stopped because the requested duration was reached;
- no run was treated as extinct, censored, or failed because of an operational limit in the completed compact grid.

The compact output is exploratory evidence, not a repository-authoritative research bundle.

## Results

### Mean final living population after 100 years

| Travel-condition cost per cell | Radius 1 | Radius 2 | Radius 3 | Radius 4 | Radius 6 | Highest tested mean |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 496.9 | 599.2 | 624.1 | 630.0 | **662.7** | radius 6 |
| 2 | 482.5 | 591.9 | 609.7 | 613.6 | **631.1** | radius 6 |
| 5 | 477.2 | 556.1 | **588.6** | 586.0 | 577.9 | radius 3 |
| 10 (v37 default) | 449.7 | **526.7** | 511.5 | 481.8 | 383.4 | radius 2 |
| 20 | **383.0** | 381.1 | 297.0 | 222.2 | 124.8 | radius 1 |
| 40 | **219.8** | 123.3 | 65.1 | 30.4 | 11.4 | radius 1 |

The important result is not the identity of a single winning radius. It is the **movement of the response curve as travel cost changes**.

- At cost `0`, mean survival increases across every tested increase in search radius; radius 6 exceeds radius 1 by 165.8 people on average and is higher in **30/30** matched seeds.
- At cost `2`, the same broad pattern remains; radius 6 exceeds radius 1 by 148.6 on average and is higher in **30/30** seeds.
- At cost `5`, the curve flattens around radius 3–6. Radius 3 has the highest mean, but radius 6 is only 10.8 lower on average and that contrast is not stable seed-to-seed.
- At the default cost `10`, the curve has clearly turned over. Radius 6 is 66.3 below radius 1 and 128.1 below radius 3 on average; radius 6 is lower than radius 3 in **29/30** matched seeds.
- At cost `20`, the best tested mean has shifted to radius 1, and radius 6 is lower than radius 1 in **30/30** seeds.
- At cost `40`, increasing radius is strongly harmful across this grid; radius 6 is lower than radius 1 in **30/30** seeds.

Approximate exploratory paired 95% intervals for the radius-6 minus radius-1 final-population contrast are:

| Cost | Mean radius 6 − radius 1 | Approx. 95% interval | Radius 6 higher |
|---:|---:|---:|---:|
| 0 | +165.8 | +138.7 to +192.9 | 30/30 |
| 2 | +148.6 | +122.8 to +174.4 | 30/30 |
| 5 | +100.7 | +78.2 to +123.2 | 28/30 |
| 10 | −66.3 | −87.7 to −45.0 | 3/30 |
| 20 | −258.2 | −278.4 to −238.0 | 0/30 |
| 40 | −208.3 | −221.1 to −195.6 | 0/30 |

These intervals are descriptive exploratory summaries, not confirmatory inference.

### Mechanism check

Representative means show why the direction flips:

| Cost | Radius | Final population | Mean move distance | Cumulative travel-condition loss | Condition-mediated deaths |
|---:|---:|---:|---:|---:|---:|
| 0 | 1 | 496.9 | 1.00 | 0 | 247.9 |
| 0 | 6 | **662.7** | 3.30 | 0 | **79.8** |
| 10 | 1 | **449.7** | 1.00 | 947,026 | 309.0 |
| 10 | 6 | 383.4 | 3.27 | 2,342,863 | 354.7 |
| 20 | 1 | **383.0** | 1.00 | 1,705,106 | 380.7 |
| 20 | 6 | 124.8 | 3.22 | 2,360,494 | 806.9 |
| 40 | 1 | **219.8** | 1.00 | 2,358,919 | 639.2 |
| 40 | 6 | 11.4 | 3.16 | 1,806,604 | 1,284.1 |

With no movement-condition penalty, wider search allows households to reach better-supported destinations without paying a physical-condition penalty for the extra distance, and survival improves. As the per-cell condition cost rises, the longer journeys enabled by a wide search horizon become increasingly damaging. Eventually the distance penalty overwhelms the resource-access advantage and the best-performing tested radius moves inward.

At the most severe costs, cumulative travel-condition loss can fall again at wide radii even while condition-mediated deaths rise. That is not evidence of improvement: populations collapse earlier, leaving fewer people and moves over which additional travel loss can accumulate.

The same caution applies to cumulative unmet food need. A nearly collapsed population can generate less total unmet demand simply because far fewer people remain alive. Cumulative unmet need should therefore not be interpreted as a standalone welfare measure across these high-mortality treatments.

## Interpretation

Conditional model statement:

> Under the tested v37 synthetic assumptions, the apparent intermediate search-radius optimum is strongly generated by the interaction between search horizon and the configured physical condition cost of completed movement. When that condition cost is removed or made very small, wider search remains beneficial across the tested radii; as the cost rises, the survival-maximising region moves progressively toward shorter-range search.

This means the preceding phrase "radius 2–3 is the sweet spot" was too specific if read as a general property of AnthroSim. A better statement is:

> AnthroSim v37 contains a search-benefit versus movement-cost trade-off whose apparent optimum depends materially on the assumed travel-condition cost.

The fresh seed block also shows why radius 2 versus radius 3 should not be overinterpreted. In the preceding radius experiment, radius 3 had the higher mean at the default cost. In this independent 30-seed block, radius 2 has the higher mean (`526.7` versus `511.5`), while the paired radius-3 minus radius-2 difference is only `−15.2` with an exploratory interval spanning zero. The robust feature is the **broad intermediate region and the deterioration at large radius under the default cost**, not an exact optimum of radius 2 or 3.

## Why this matters for future archaeological use

`travelConditionCostPerCell` is currently a synthetic modelling assumption, not an empirically calibrated prehistoric travel penalty. This experiment shows that it can change not only the magnitude of a result but even the direction of the effect of wider migration knowledge.

A future site-specific result involving permanent migration therefore must not hold search radius and travel-condition cost at one arbitrary default and present the resulting population outcome as robust. Plausible uncertainty in both assumptions needs to be propagated through the experiment.

## What this does not establish

This experiment does **not** show that real prehistoric movement had one of these costs, that real people suffered a linear physical penalty per unit distance, or that any real society had an optimal migration radius. Cost `0` and the high-cost treatments are mechanism probes, not competing historical reconstructions.

## Follow-up

The next useful diagnostic is to test another part of the migration mechanism independently. Two strong candidates are:

1. migration decision frequency — how often households get an opportunity to reconsider residence; or
2. the minimum utility improvement required before a household moves.

Either would test whether the movement-cost/search interaction remains dominant when the model changes **how readily and how often** households relocate.

## Reproducibility boundary

The completed compact grid is stronger operational evidence than the aborted full-bundle attempt because it contains the full 900-run design and explicit per-run configuration checks, but it remains an exploratory instrument rather than a canonical study package. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
