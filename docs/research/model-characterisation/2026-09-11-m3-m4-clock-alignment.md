# Exploratory model characterisation: M3/M4 clock alignment

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and structural sensitivity diagnosis  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Does the apparent best permanent-migration decision frequency follow the M3 resource-settlement frequency?

The previous experiment found a strong apparent optimum near four migration decisions/year while M3 resource settlement was also fixed at four periods/year. That result was structurally ambiguous because `migration.decisionPeriodsPerYear` controls both how often households reconsider relocation and the M4 demand interval used when scoring resource support.

This experiment therefore varies the M3 and M4 clocks independently. The diagnostic question is whether the best-performing migration clock stays near four decisions/year, or moves with the M3 resource clock.

This is model characterisation, not an estimate of real prehistoric decision frequency, planning horizon, or resource-accounting cadence.

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
- candidate radius = 3 cells (v37 default);
- travel-condition cost per cell = 10 (v37 default);
- 30 matched seeds (`6101..6130`) in every treatment;
- all other synthetic M3/M4 parameters left at their v37 defaults.

Two clocks were crossed independently:

- M3 resource settlements/year `P`: `1, 2, 4, 8, 12`;
- M4 migration decision opportunities/year `D`: `1, 2, 4, 8, 12`.

Total runs: **750** (`5 resource clocks × 5 migration clocks × 30 matched seeds`).

## Execution and integrity checks

The experiment used a temporary compact runner on the isolated `experiment/v37-clock-alignment` branch. The workflow was added by experimental commit `c73ccd58c37c75f6f217114da73b64d76d410088`, but it explicitly checked out and executed the unchanged v37 core source at `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`.

The runner constructed ordinary `ExperimentConfig` values, called the unchanged core `Simulation::new(...).run()` path, retained only compact per-run measurements, and required exactly 750 rows before uploading the artifact.

Integrity checks found:

- exactly **750** rows;
- exactly **30** runs for every one of the 25 `(P,D)` cells;
- **750/750** rows reported `anthrosim-model-semantics-v37`;
- **750/750** rows reported core source commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`;
- **0/750** requested/configured M3-period mismatches;
- **0/750** requested/configured M4-decision-period mismatches;
- all runs retained radius `3` and travel-condition cost `10`;
- runs ended either at the requested 100-year duration or by population extinction; extinction occurred only in the most severely misaligned `P=1` treatments described below.

The compact output is exploratory evidence, not a repository-authoritative research bundle.

## Results

### Mean final living population after 100 years

Rows are M3 resource settlements/year (`P`). Columns are M4 permanent-migration decisions/year (`D`). Bold cells are clock-aligned (`P = D`).

| P \ D | 1 | 2 | 4 | 8 | 12 |
|---:|---:|---:|---:|---:|---:|
| **1** | **131.3** | 8.3 | 0.8 | 0.3 | 0.2 |
| **2** | 184.3 | **370.6** | 52.9 | 11.6 | 6.6 |
| **4** | 198.3 | 414.9 | **527.9** | 162.5 | 89.4 |
| **8** | 204.7 | 408.7 | 506.2 | **555.7** | 420.1 |
| **12** | 201.5 | 400.5 | 502.8 | 482.5 | **542.0** |

The highest tested mean in **every row** lies on the alignment diagonal:

- `P=1` -> best `D=1`;
- `P=2` -> best `D=2`;
- `P=4` -> best `D=4`;
- `P=8` -> best `D=8`;
- `P=12` -> best `D=12`.

The previous apparent four-decisions/year optimum therefore does **not** remain fixed when the M3 resource clock changes.

### Aligned versus best misaligned treatment

For each M3 clock, the aligned cell was compared with the highest-mean misaligned alternative using matched seeds.

| M3 periods/year | Aligned D | Aligned mean final population | Best misaligned D | Best misaligned mean | Mean aligned advantage | Approx. 95% paired interval | Aligned higher |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 131.3 | 2 | 8.3 | +123.0 | +112.6 to +133.3 | 30/30 |
| 2 | 2 | 370.6 | 1 | 184.3 | +186.3 | +169.0 to +203.7 | 30/30 |
| 4 | 4 | 527.9 | 2 | 414.9 | +113.0 | +93.3 to +132.7 | 29/30 (1 tie) |
| 8 | 8 | 555.7 | 4 | 506.2 | +49.5 | +29.8 to +69.2 | 25/30 |
| 12 | 12 | 542.0 | 4 | 502.8 | +39.2 | +17.6 to +60.8 | 23/30 |

These intervals are descriptive exploratory summaries, not confirmatory inference.

### Extinction behaviour

The strongest failures occur when M4 runs substantially faster than a very coarse M3 clock.

At `P=1`:

- `D=1`: 0/30 extinct;
- `D=2`: 0/30 extinct, but mean final population falls to 8.3;
- `D=4`: 13/30 extinct;
- `D=8`: 24/30 extinct;
- `D=12`: 24/30 extinct.

No other tested `(P,D)` cell produced extinction within the 100-year horizon.

### Mechanism-facing summaries

Mean completed migration moves:

| P \ D | 1 | 2 | 4 | 8 | 12 |
|---:|---:|---:|---:|---:|---:|
| 1 | 8,669 | 3,116 | 1,758 | 1,344 | 1,237 |
| 2 | 9,416 | 13,293 | 5,621 | 3,337 | 2,782 |
| 4 | 9,579 | 13,665 | 15,540 | 9,030 | 7,069 |
| 8 | 9,562 | 13,312 | 14,975 | 15,933 | 13,646 |
| 12 | 9,437 | 13,182 | 14,854 | 14,781 | 15,810 |

Mean cumulative unmet resource need:

| P \ D | 1 | 2 | 4 | 8 | 12 |
|---:|---:|---:|---:|---:|---:|
| 1 | 68,608 | 113,036 | 133,410 | 142,844 | 146,564 |
| 2 | 77,237 | 46,248 | 119,821 | 134,599 | 136,685 |
| 4 | 77,820 | 49,713 | 28,953 | 116,918 | 135,619 |
| 8 | 75,971 | 47,723 | 27,017 | 14,830 | 64,323 |
| 12 | 73,472 | 47,442 | 26,016 | 37,892 | 10,519 |

Mean condition-mediated deaths:

| P \ D | 1 | 2 | 4 | 8 | 12 |
|---:|---:|---:|---:|---:|---:|
| 1 | 822 | 1,343 | 1,605 | 1,690 | 1,717 |
| 2 | 706 | 387 | 1,009 | 1,278 | 1,351 |
| 4 | 657 | 320 | 196 | 674 | 852 |
| 8 | 662 | 337 | 215 | 172 | 334 |
| 12 | 648 | 347 | 226 | 260 | 180 |

The same diagonal structure appears in these mechanism-facing quantities: aligned clocks generally support more useful completed migration, much lower unmet need, and lower condition-mediated mortality than nearby severe mismatches.

## Interpretation

Conditional model statement:

> Under the tested v37 synthetic configuration, the migration-decision frequency that maximises long-run survival tracks the M3 resource-settlement frequency across every tested clock pair. The previous apparent optimum at four migration decisions/year is therefore primarily a clock-coupled result under `P=4`, not a fixed behavioural optimum of four decisions/year.

A safer plain-language summary is:

> AnthroSim v37 strongly prefers its resource and permanent-migration clocks to operate at similar cadences in this harsh-resource scenario.

This is stronger evidence for structural clock sensitivity than the preceding one-dimensional decision-frequency sweep because the location of the optimum moves from 1 to 2 to 4 to 8 to 12 as `P` moves through those same values.

## Why the coupling is plausible inside v37

The normative M3/M4 timing contract deliberately separates the two clocks, but M4 resource-support scoring allocates annual food need over the **M4 decision interval**. At the same time, the resource system's authoritative stock state is settled on the M3 clock.

Consequently, when `D > P`, M4 evaluates increasingly short demand horizons more often than M3 settles the resource state. In the tested harsh-resource world this can make current cells appear adequately supported or make candidate improvements too small to clear the movement threshold. The completed-move rate then falls even though more decision opportunities exist, while later M3 settlements expose large resource shortfalls.

When `D < P`, M4 uses a longer demand horizon and tends to identify pressure more aggressively; this is generally less catastrophic than the `D > P` direction in the tested grid, but it still underperforms the aligned treatment.

This interpretation is model-mechanistic, not anthropological evidence. It also does not prove that equality of the clocks is scientifically desirable; it shows that equality currently has substantial causal leverage on outcomes.

## Scientific concern exposed

`resources.periodsPerYear` is partly a model-resolution/settlement choice, while `migration.decisionPeriodsPerYear` is presented as an independently configurable behavioural-model choice. A large survival preference for `P ≈ D` means future archaeological conclusions could depend heavily on a numerical/structural timing relationship unless that dependence is explicitly controlled.

The effect should therefore be treated as a **high-priority sensitivity and model-design concern**, not as evidence that real human migration decisions naturally occurred at the same frequency as ecological/resource accounting intervals.

The current behaviour is documented by the v37 timing contract and is not automatically a software bug. However, before empirical/site-specific interpretation it should be decided whether M4 resource-support assessment is intended to depend this strongly on M3 settlement cadence, and whether the planning horizon should remain mechanically tied to the M4 decision interval.

## What this does not establish

This experiment does **not** establish:

- a real-world optimal decision frequency;
- a real-world optimal resource-observation cadence;
- that prehistoric households synchronised relocation decisions with ecological cycles;
- that `P=D` is intrinsically scientifically correct;
- that the observed sensitivity is entirely caused by one implementation detail rather than the combined documented timing semantics.

## Follow-up

The strongest next diagnostic is to separate the two concepts currently bundled into the migration clock:

> **Can migration decision opportunity frequency be varied while holding the resource-support planning horizon fixed?**

That requires an experiment-only intervention or a deliberate model change because v37 derives M4 demand horizon from the decision interval. If holding the planning horizon fixed removes most of the clock-alignment effect, the present sensitivity is primarily a planning-horizon coupling. If a strong frequency effect remains, repeated behavioural opportunities themselves have independent causal importance.

A second useful control is to repeat selected clock pairs with migration disabled. That would measure how much changing the M3 clock alone alters the harsh-resource trajectory before M4 interactions are added.

## Reproducibility boundary

The completed compact grid contains the full 750-run design and explicit per-run configuration/provenance checks, but it remains an exploratory instrument rather than a canonical study package. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
