# Exploratory model characterisation: fixed migration planning horizon

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and structural-coupling diagnosis  
**Base model target:** `anthrosim-model-semantics-v37`  
**Base source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> If households always assess resource support over the same quarter-year planning horizon, does changing how often they are allowed to reconsider permanent relocation still matter?

The preceding experiments showed that the apparent optimum in `migration.decisionPeriodsPerYear` tracks the M3 resource clock. In ordinary v37 semantics, changing the M4 decision frequency also changes the demand interval used to score resource support. This experiment separates those concepts with an explicit counterfactual intervention.

This is a model-diagnostic intervention, not an alternative validated AnthroSim model and not an archaeological reconstruction.

## Design

The canonical comparison arm is the radius-3 subset of the previously recorded decision-frequency experiment, using the same 30 matched seeds (`5101..5130`). Common settings are:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity scale 150 permille;
- seasonality scale 1000 permille;
- M3 resource settlements/year `P = 4`;
- migration candidate radius = 3;
- travel-condition cost per cell = 10;
- migration decisions/year `D = 1, 2, 4, 8, 12, 24`;
- 30 matched seeds per treatment.

The counterfactual arm keeps all of those settings but modifies only M4 resource-support scoring: every migration evaluation uses the demand allocation for the **current canonical quarter-year planning horizon**, independent of how many M4 decision opportunities occur that year.

The quarter boundaries retain v37's ordinary 91/91/91/92-day annual partition. Therefore the `D = 4` counterfactual is deliberately identical to canonical v37 resource-support scoring; it acts as an internal intervention check.

Counterfactual runs: **180** (`6 decision frequencies × 30 seeds`). The canonical comparator contributes the matching 180 radius-3 runs already completed in the preceding experiment.

## Experimental identity and integrity

The counterfactual used temporary harness branch `experiment/v37-fixed-planning-horizon`. Workflow run `34608858908` checked out the exact v37 source commit above, then applied the planning-horizon intervention before compiling. To avoid falsely presenting modified semantics as authoritative v37, the experimental build identifies itself as:

`anthrosim-model-semantics-v37-fixed-quarter-horizon-experiment`

with experimental provenance string:

`9f0914e2772cd2f0c6243ab4be8b296f3ce48f30+fixed-quarter-horizon-intervention`

The completed counterfactual artifact contained exactly 180 rows: 30 for each requested `D`; all retained `P=4`, radius 3 and travel-condition cost 10.

### D=4 intervention identity control

At `D=4`, the phase-matched fixed-quarter intervention should reproduce canonical v37 exactly. It did.

For all **30/30 matched seeds**, canonical and counterfactual `D=4` runs were identical for every recorded comparison metric, including:

- final living population;
- completed moves and people moved;
- total migration distance;
- travel-condition loss;
- household evaluations and pressure classifications;
- cumulative unmet resource need;
- condition-mediated deaths.

This provides a strong internal check that differences at the other decision frequencies arise from separating planning horizon from opportunity frequency rather than from an unrelated change to the `D=4` baseline.

An earlier flat-25-units pilot was intentionally superseded and is not used below because it did not preserve the exact v37 quarter allocation at `D=4`. The recorded result uses the phase-matched intervention described above.

## Results

### Mean final living population after 100 years

| Decisions/year | Canonical v37: planning horizon tied to D | Fixed quarter-year planning horizon |
|---:|---:|---:|
| 1 | 187.6 | **1.6** |
| 2 | 392.0 | **54.1** |
| 4 | **505.4** | **505.4** |
| 8 | 153.5 | **566.0** |
| 12 | 89.0 | **566.7** |
| 24 | 46.1 | **570.3** |

The canonical hump-shaped response disappears once planning horizon is held fixed.

Under the fixed-quarter intervention:

- increasing decision opportunities from 1 → 2 → 4 → 8 greatly improves survival;
- the gain beyond 8 opportunities/year becomes very small;
- `D = 8`, `12` and `24` form a broad high-survival plateau rather than the catastrophic decline seen under canonical v37;
- `D = 1` produces 7/30 extinctions, while all other fixed-horizon treatments reach the requested duration.

### Paired counterfactual minus canonical contrasts

Because the same seeds were used, each counterfactual run can be paired directly with its canonical v37 counterpart.

| D | Canonical mean | Fixed-horizon mean | Mean difference | Approx. 95% paired interval | Fixed-horizon higher |
|---:|---:|---:|---:|---:|---:|
| 1 | 187.6 | 1.6 | −186.0 | −199.7 to −172.3 | 0/30 |
| 2 | 392.0 | 54.1 | −337.9 | −353.8 to −322.0 | 0/30 |
| 4 | 505.4 | 505.4 | 0.0 | exact identity | 0/30; 30 ties |
| 8 | 153.5 | 566.0 | +412.5 | +391.0 to +433.9 | 30/30 |
| 12 | 89.0 | 566.7 | +477.6 | +453.5 to +501.8 | 30/30 |
| 24 | 46.1 | 570.3 | +524.2 | +506.1 to +542.3 | 30/30 |

These intervals are descriptive exploratory summaries, not confirmatory inference.

The sign reversal around `D=4` is exactly what the structural-coupling hypothesis predicts. Canonical v37 gives low-frequency treatments a longer resource-support horizon and high-frequency treatments a shorter one. Fixing the horizon to one quarter removes both effects.

### Within the fixed-horizon arm

Mean mechanism-facing results are:

| D | Final population | Completed moves | Unmet need | Condition-mediated deaths |
|---:|---:|---:|---:|---:|
| 1 | 1.6 | 1,898 | 147,693 | 1,563 |
| 2 | 54.1 | 5,790 | 135,391 | 990 |
| 4 | 505.4 | 15,641 | 31,087 | 206 |
| 8 | 566.0 | 16,769 | 20,941 | 133 |
| 12 | 566.7 | 16,761 | 20,052 | 129 |
| 24 | 570.3 | 16,771 | 19,962 | 130 |

With a fixed quarter-year support horizon, more frequent reconsideration increases useful completed movement until roughly `D=8`, after which completed moves and survival are essentially saturated.

Matched-seed exploratory comparisons inside the fixed-horizon arm show:

- `D=8` exceeds `D=4` by 60.6 survivors on average, with approximate 95% interval +41.3 to +79.9; `D=8` is higher in 25/30 seeds;
- `D=12` versus `D=8` differs by only +0.7 on average, interval −26.0 to +27.4;
- `D=24` versus `D=8` differs by only +4.3 on average, interval −12.0 to +20.6.

The robust pattern is therefore not that 24 decisions/year is best. It is that the strong penalty for high `D` disappears, and benefits saturate around the 8/year region under this particular synthetic setup.

## Interpretation

Conditional model statement:

> In the tested harsh-resource v37-derived counterfactual, the catastrophic decline at high migration-decision frequencies is primarily produced by tying M4 resource-support planning horizon to decision frequency. When the planning horizon is held at the canonical quarter-year scale, increasing decision opportunities from 4 to 8 improves survival and further increases to 12 or 24 have little additional effect rather than causing collapse.

The experiment also shows that decision opportunity frequency has an **independent effect**. Holding the planning horizon fixed does not make all frequencies equivalent: one or two opportunities/year perform very poorly because households have too few chances to react between quarterly M3 resource settlements, while 4–8 opportunities greatly improve adaptive relocation.

A plain-language summary is:

> The previous "too many chances to move is harmful" result was mostly a planning-horizon artefact of the bundled v37 parameter. Once households look the same distance ahead, extra chances to reconsider moving are helpful up to a point and then mostly show diminishing returns.

## Why this matters for model design

This is stronger evidence that `migration.decisionPeriodsPerYear` currently combines two scientifically distinct assumptions:

1. **opportunity frequency** — how often a household may reconsider permanent residence;
2. **planning horizon** — how much future food demand is used when judging whether a location is adequately supported.

Those assumptions can have opposite effects, and bundling them can create a large artificial preference for matching the M3 and M4 clocks.

Before empirical/site-specific use, AnthroSim should consider whether the resource-support planning horizon deserves an explicit independent parameter or otherwise a clearly justified formulation. This experiment does not by itself choose the correct replacement semantics.

## What this does not establish

This experiment does **not** establish that real prehistoric households reconsidered residence eight times/year, that a quarter-year planning horizon is realistic, or that more frequent decisions are always beneficial. The fixed horizon is a mechanism-isolation device, not an empirical estimate.

The intervention also leaves all other v37 migration assumptions unchanged, including pressure thresholds, utility weights, relocation costs and candidate search radius. Different assumptions may shift the plateau or alter the response.

## Follow-up

Two especially useful next diagnostics now separate naturally:

1. **M3-only timing control:** disable permanent migration and vary `resources.periodsPerYear` to measure how much resource-settlement frequency alone changes the harsh-resource trajectory.
2. **Planning-horizon sensitivity:** with opportunity frequency held fixed, vary the independent resource-support planning horizon to see how strongly survival and movement depend on short versus long look-ahead.

The M3-only control is the cleaner next experiment because it establishes the baseline timing sensitivity before introducing another counterfactual migration parameter.

## Reproducibility boundary

The counterfactual build deliberately changes one model rule and therefore must not be cited as authoritative `anthrosim-model-semantics-v37`. It is an exploratory intervention derived from the exact v37 source. Any future model change based on this diagnosis requires its own reviewed semantics decision, tests, provenance identity and scientific rationale.
