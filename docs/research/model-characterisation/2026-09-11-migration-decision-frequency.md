# Exploratory model characterisation: migration decision frequency

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and sensitivity-test design  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> How much does it matter how often households are allowed to reconsider permanent relocation?

In plain language: the previous experiments showed that permanent migration is a powerful synthetic mechanism, and that search distance interacts strongly with movement cost. This experiment asks whether simply giving households more or fewer opportunities to decide whether to move changes long-run survival under harsh resource pressure.

This is a model-characterisation question. It does not estimate how often real prehistoric households reconsidered residence.

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
- v37 default travel-condition cost per cell = `10`;
- 30 matched seeds (`5101..5130`) in every treatment;
- all other synthetic M3/M4 parameters left at their v37 defaults.

Migration decision opportunities were varied across:

- `1, 2, 4, 8, 12, 24` decisions per model year.

The v37 synthetic default is `4` decisions/year. To avoid making the result depend on the noisy radius-2-versus-radius-3 ordering seen in the previous sensitivity experiment, every frequency was tested at both candidate radii `2` and `3`.

Total runs: **360** (`6 frequencies × 2 radii × 30 matched seeds`).

The resource integration clock remained at its synthetic default of `4` M3 settlements/year.

## Execution and integrity checks

The ordinary CLI does not expose migration decision frequency, so the experiment used a temporary compact runner on the isolated `experiment/v37-migration-decision-frequency` branch. The workflow checked out the exact v37 core source commit above, constructed ordinary `ExperimentConfig` values directly, called the unchanged core `Simulation::new(...).run()` path, retained only compact per-run measurements, and required an exact 360-row result count before uploading the artifact.

Integrity checks found:

- exactly **360** rows;
- exactly 60 rows at each tested decision frequency;
- exactly 180 rows at each tested radius;
- **360/360** rows reported `anthrosim-model-semantics-v37`;
- **360/360** rows reported core source commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`;
- **0/360** requested/configured decision-frequency mismatches;
- **0/360** requested/configured radius mismatches;
- **360/360** runs retained the default travel-condition cost `10`;
- **360/360** runs stopped because the requested duration was reached.

The compact output is exploratory evidence, not a repository-authoritative research bundle.

## Results

### Mean final living population after 100 years

| Migration decisions/year | Radius 2 | Radius 3 |
|---:|---:|---:|
| 1 | 171.3 | 187.6 |
| 2 | 383.0 | 392.0 |
| **4 (v37 default)** | **506.2** | **505.4** |
| 8 | 160.4 | 153.5 |
| 12 | 88.3 | 89.0 |
| 24 | 49.2 | 46.1 |

The response is strongly **hump-shaped**, not monotonic. In this configuration, increasing decision frequency from 1 → 2 → 4 improves survival sharply, but increasing it beyond 4 reverses the effect and produces progressively worse outcomes.

The broad pattern is insensitive to the radius-2/radius-3 ambiguity. At radius 2, the default four-opportunity treatment exceeds 1, 8, 12 and 24 opportunities in **30/30 matched seeds**, and exceeds 2 opportunities in 29/30. At radius 3, it exceeds every alternative frequency in **30/30 matched seeds**.

Approximate exploratory paired 95% intervals for the default `4/year` minus each alternative final-population contrast are:

| Radius | Comparison | Mean difference | Approx. 95% interval | Default higher |
|---:|---|---:|---:|---:|
| 2 | 4 − 1 | +335.0 | +315.8 to +354.1 | 30/30 |
| 2 | 4 − 2 | +123.2 | +101.4 to +145.0 | 29/30 |
| 2 | 4 − 8 | +345.8 | +328.6 to +363.0 | 30/30 |
| 2 | 4 − 12 | +417.9 | +399.7 to +436.1 | 30/30 |
| 2 | 4 − 24 | +457.0 | +439.8 to +474.2 | 30/30 |
| 3 | 4 − 1 | +317.8 | +300.6 to +335.1 | 30/30 |
| 3 | 4 − 2 | +113.4 | +92.5 to +134.2 | 30/30 |
| 3 | 4 − 8 | +351.9 | +333.4 to +370.3 | 30/30 |
| 3 | 4 − 12 | +416.4 | +399.3 to +433.5 | 30/30 |
| 3 | 4 − 24 | +459.3 | +442.3 to +476.3 | 30/30 |

These intervals are descriptive exploratory summaries, not confirmatory inference.

### Mechanism-facing summaries

| Decisions/year | Moves, radius 2 | Moves, radius 3 | Unmet need, radius 2 | Unmet need, radius 3 | Condition-mediated deaths, radius 2 | Condition-mediated deaths, radius 3 |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 9,401 | 9,566 | 88,583 | 80,292 | 724.9 | 683.2 |
| 2 | 13,939 | 13,769 | 61,514 | 52,204 | 357.4 | 334.7 |
| **4** | **16,561** | **15,641** | **37,176** | **31,087** | **189.8** | **206.1** |
| 8 | 9,641 | 9,023 | 128,312 | 118,382 | 697.4 | 701.9 |
| 12 | 7,519 | 7,166 | 145,923 | 139,085 | 875.2 | 888.0 |
| 24 | 5,833 | 5,577 | 157,493 | 150,396 | 1,034.9 | 1,031.9 |

More decision opportunities therefore do **not** translate into more completed moves. Moves increase from 1 to 4 opportunities/year and then collapse as decision frequency rises further. At the same time, cumulative unmet resource need and condition-mediated mortality rise sharply in the 8–24 treatments.

This is not because households are mechanically forced to move every time a decision boundary occurs. A boundary is only an opportunity to evaluate relocation.

## Important structural coupling: this is not a pure frequency knob

The result cannot be interpreted as "quarterly decision-making is intrinsically optimal" because v37 deliberately couples the M4 decision clock to the resource-demand horizon used inside migration utility.

The normative v37 timing contract defines M4 resource-support demand from the annual need allocated over the **M4 decision interval**. In simplified terms:

- `1` decision/year asks whether current stock supports roughly a full year's demand;
- `2` decisions/year asks about roughly half a year;
- `4` decisions/year asks about roughly a quarter-year;
- `24` decisions/year asks about roughly the next 15 days.

The migration pressure calculation then uses that interval-specific resource score. Increasing `decisionPeriodsPerYear` therefore does two things at once:

1. gives households more opportunities to reconsider residence; and
2. shortens the future food-demand horizon against which the current cell is judged.

The M3 resource-integration clock remained fixed at 4 settlements/year in this experiment. At the default `D = 4`, the two clocks have the same broad quarter-year cadence. At `D > 4`, M4 repeatedly evaluates much shorter demand horizons between the ordinary M3 resource settlements.

That helps explain the observed reversal. At high decision frequencies, many more household evaluations occur, but a much smaller fraction are classified as under migration pressure, and a smaller fraction of pressured evaluations find a destination that clears the configured improvement threshold. The model therefore executes fewer total relocations despite offering more decision opportunities. Resource shortfall later accumulates strongly at M3 settlement, and survival deteriorates.

Approximate aggregate funnel behaviour across the two tested radii is:

| Decisions/year | Household evaluations under pressure | Completed moves / pressured evaluations |
|---:|---:|---:|
| 1 | ~42% | ~89% |
| 2 | ~27% | ~90% |
| 4 | ~15% | ~90% |
| 8 | ~6% | ~72% |
| 12 | ~6% | ~45% |
| 24 | ~6% | ~17% |

The apparent optimum at `4/year` is therefore best treated as an **interaction between behavioural opportunity rate, M4 planning horizon, and the fixed M3 resource clock**, not as an isolated estimate of an ideal behavioural frequency.

## Interpretation

Conditional model statement:

> Under the tested v37 synthetic configuration with four M3 resource settlements/year, permanent-migration outcomes are highly sensitive to `migration.decisionPeriodsPerYear`. Survival peaks near the default four opportunities/year and deteriorates sharply at both lower and higher tested frequencies. However, the parameter changes both decision opportunity frequency and the M4 resource-support demand horizon, so the observed peak does not identify a pure behavioural optimum.

A safer plain-language summary is:

> AnthroSim currently cares a great deal about how the migration decision clock is configured, and part of that sensitivity comes from the clock also changing how far ahead households effectively look when judging local food support.

## Why this matters for future archaeological use

A future site-specific model must not silently use `decisionPeriodsPerYear = 4` and interpret the resulting population trajectory as robust. Four opportunities/year is a synthetic default, not an empirically calibrated prehistoric behavioural rate.

More importantly, uncertainty about decision timing cannot currently be described as only uncertainty about "how often people considered moving," because the same parameter also controls the resource-demand interval used in the migration decision. That structural coupling needs to be understood and, where relevant, included in sensitivity analysis.

## What this does not establish

This experiment does **not** show that prehistoric households reconsidered residence quarterly, that quarterly mobility was historically optimal, or that real people used a fixed planning horizon tied mechanically to decision frequency.

It also does not establish that the current D-to-demand-horizon coupling is scientifically preferable to alternative migration-decision formulations. It only characterises the documented v37 semantics.

## Follow-up

The strongest next diagnostic is now:

> **Does the apparent best migration-decision frequency follow the M3 resource-settlement frequency?**

That experiment should vary `resources.periodsPerYear` and `migration.decisionPeriodsPerYear` together and independently. If the survival peak systematically tracks `D ≈ P`, that would indicate that clock alignment is a major driver of the current result. If the optimum remains near four decisions/year even when the M3 clock changes, the behavioural frequency itself is doing more of the work.

This follow-up is more informative than immediately testing another arbitrary migration parameter because it directly tests the structural explanation exposed here.

## Reproducibility boundary

The completed compact grid contains the full 360-run design and explicit per-run configuration/provenance checks, but it remains an exploratory instrument rather than a canonical study package. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
