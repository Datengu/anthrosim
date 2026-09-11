# Exploratory model characterisation: fixed-population M3 resource accounting

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** mechanism isolation for residual M3 temporal-resolution sensitivity  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> With population held completely fixed and every condition/mortality/migration pathway disabled, does changing M3 settlement cadence still change cumulative unmet food need?

Experiments 10–12 progressively removed seasonality, condition-mediated mortality and mortality-hazard partitioning as explanations for the residual coarse-cadence signal. Experiment 12 showed that the fixed-condition mortality hazard itself is partition-stable. This experiment therefore removes demographic and physiological feedback entirely and tests the food-accounting system directly.

This is synthetic model characterisation. It does not validate a real subsistence system, storage technology or ecological renewal process.

## Design

Common setup:

- exact historical v37 core;
- 100 simulated years;
- 64 × 64 synthetic world;
- 2,000 synthetic founders;
- target founder household size 5;
- historical fixed-founder household lifecycle (the optional lifecycle field is omitted, corresponding to `fixed_founder_v1`);
- annual food need = 100 abstract units/person/year;
- productivity scale = 750 permille;
- seasonality scale = 0;
- all demographic mortality probabilities = 0;
- all fertility probabilities = 0;
- condition recovery = 0;
- maximum condition loss = 0;
- maximum condition-mediated mortality probability = 0;
- permanent migration disabled;
- no temporary-mobility program;
- M3 periods/year P = 1, 2, 4, 8, 12, 24;
- 30 matched seeds `7501..7530`.

Total: **180 runs**.

Because births and all mortality are disabled, every accepted run must retain exactly 2,000 living people and exactly 2,000 person records for the entire century. Condition remains fixed at 1000 permille. No permanent or temporary movement is active. Consequently, differences in resource outcomes cannot be attributed to population size, mortality, fertility, migration, condition response or household fission.

## Execution and integrity

Temporary branch: `experiment/v37-fixed-population-resource-accounting`  
Workflow run: `34621469165`  
Harness commit: `5464c26f904d954837d5bb0e3653a2e29946e52c`  
Artifact: `anthrosim-v37-fixed-population-resource-accounting-results`  
Artifact ID: `10273180445`  
Artifact SHA-256: `65798eae7e54089a81c6909116afb3558c0c98cef905ddc322ed7597d02ba66a`

The workflow hard-checked:

- exactly 180 rows and 30 rows per cadence;
- executed P equals declared P;
- productivity 750 and seasonality 0;
- annual need 100;
- zero fertility and zero background mortality;
- zero condition recovery/loss and zero condition-mediated mortality probability;
- migration disabled and zero moves;
- no temporary mobility configuration;
- historical fixed-founder household lifecycle represented by the omitted optional lifecycle field;
- exactly 2,000 living people and 2,000 person records at the end of every run;
- mean living condition exactly 1000;
- full 36,500-day duration reached;
- exactly `100 × P` resource periods processed;
- exact `anthrosim-model-semantics-v37` / source-commit provenance.

## Results

Mean century totals:

| M3 settlements/year | Unmet need | Realised regeneration | Harvested / consumed | Final food stock |
|---:|---:|---:|---:|---:|
| 1 | 2,239,809 | 16,535,294 | 17,760,191 | 16,916,820 |
| 2 | 2,193,072 | 16,618,850 | 17,806,928 | 16,953,639 |
| 4 | 2,170,336 | 16,660,481 | 17,829,664 | 16,972,534 |
| 8 | 2,158,984 | 16,681,597 | 17,841,016 | 16,982,298 |
| 12 | 2,155,304 | 16,688,441 | 17,844,696 | 16,985,461 |
| 24 | 2,151,630 | 16,695,276 | 17,848,370 | 16,988,623 |

Initial food stock is identical across cadence treatments within each seed and averages 18,141,717 units across this seed block.

### Total demand is exactly cadence-invariant

For **every one of the 180 runs**:

`harvested food + unmet need = 20,000,000 units`.

That is exactly the century demand implied by 2,000 fixed people × 100 units/person/year × 100 years.

Therefore the coarse-cadence difference is not produced by changing the total amount of food requested. The annual-demand partitioning preserves the century-total demand exactly.

### P1 has systematically greater unmet need

Matched differences versus the default P4:

| Comparison | Mean unmet-need difference | Approx. paired 95% interval | Seed direction |
|---|---:|---:|---:|
| P1 − P4 | +69,473 | +63,471 to +75,475 | 30/30 higher |
| P2 − P4 | +22,736 | +20,740 to +24,731 | 30/30 higher |
| P8 − P4 | −11,352 | −12,356 to −10,349 | 30/30 lower |
| P12 − P4 | −15,033 | −16,362 to −13,703 | 30/30 lower |
| P24 − P4 | −18,706 | −20,362 to −17,050 | 30/30 lower |

P1 also exceeds P24 by about **88,179 unmet units** on average, approximate paired 95% interval **+80,522 to +95,835**, again in all 30 matched seeds.

Unlike earlier demographic outcomes, this is an orderly and highly consistent cadence response under fixed population.

### The difference tracks realised regeneration

Compared with P4, mean realised regeneration changes by:

| Comparison | Mean regeneration difference | Approx. paired 95% interval | Seed direction |
|---|---:|---:|---:|
| P1 − P4 | −125,187 | −127,405 to −122,970 | 30/30 lower |
| P2 − P4 | −41,631 | −42,393 to −40,868 | 30/30 lower |
| P8 − P4 | +21,116 | +20,712 to +21,520 | 30/30 higher |
| P12 − P4 | +27,960 | +27,435 to +28,485 | 30/30 higher |
| P24 − P4 | +34,795 | +34,151 to +35,439 | 30/30 higher |

P1 also finishes with about **55,715 fewer food-stock units than P4** on average. Accounting reconciles exactly:

`initial stock + realised regeneration − harvested = final stock`.

The P1−P4 regeneration shortfall is therefore partitioned between less harvested food (and hence more unmet demand) and lower residual stock.

## Mechanistic interpretation

Plain-language result:

> **Yes. Even with population, households, condition and all demographic pathways held fixed, coarse M3 settlement itself creates more unmet food need.**

The v37 resource period performs regeneration before demand/harvest, and realised regeneration is capped by the cell's remaining storage capacity. This creates a strong mechanistic candidate for the cadence effect:

- at coarse cadence, a larger regeneration quantity is attempted before the corresponding large harvest has freed storage space;
- potential regeneration that encounters a full or nearly full cell can be clipped by capacity;
- at finer cadence, earlier harvests repeatedly free space before later regeneration events;
- more of the annual regeneration potential can therefore be realised.

The observed results are exactly consistent with that mechanism: coarse treatments realise less regeneration, harvest less food, accumulate more unmet need and finish with somewhat less stock.

However, this experiment does **not by itself prove that storage-capacity clipping is the only cause**. Demand/harvest partitioning and local stock path dependence occur in the same M3 period operation. A direct capacity intervention is still required before assigning the entire effect to storage clipping.

## Relationship to earlier experiments

The timing diagnosis can now be decomposed more cleanly:

1. the severe M3/M4 pathology is primarily the M4 planning-horizon / decision-cadence coupling;
2. seasonality explains the transient P12 survivor bump in the M3-only demographic experiment;
3. the condition-mortality hazard itself is stable when condition is fixed;
4. condition-mediated mortality transmits upstream scarcity differences into demographic differences;
5. **a genuine M3 resource-accounting cadence effect remains even with population fully fixed**.

This means the coarse P1/P2 signal is not merely an artefact of mortality or stochastic demographic scheduling. It originates upstream in the resource-state trajectory.

## What this does not establish

This experiment does not establish that one particular M3 cadence is scientifically or archaeologically correct. P remains a numerical/model-time representation choice under synthetic assumptions.

It also does not establish that finer cadence should always be preferred. The magnitude and even practical importance of the effect can depend on storage capacity, initial stock, regeneration rate, demand, productivity and spatial concentration.

## Follow-up

The strongest next question is:

> **Does the cadence effect disappear when storage capacity can no longer clip regeneration?**

The clean follow-up should repeat this fixed-population grid while making storage capacity deliberately non-binding (or equivalently reducing initial stock / regeneration relative to capacity while preserving the same demand regime). If the P1–P24 unmet-need gradient largely disappears, storage-capacity clipping is the dominant mechanism. If a substantial gradient remains, demand/harvest timing or another stock-path effect must also contribute.

## Reproducibility boundary

These outputs are exploratory model-characterisation evidence, not an empirical validation result or canonical regression oracle. Claim-driving use requires a reviewed experiment definition, retained authoritative outputs and the relevant scientific gates.
