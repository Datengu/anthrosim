# v0.1 resource-model provenance and evidence boundary

**Status:** M3 synthetic validation baseline, executable timing refined by the [M3 resource-time contract v1](m3-resource-time-contract-v1.md)  
**Scientific status:** unvalidated  
**Runtime empirical dataset:** none

This document records what the first executable AnthroSim resource model means and, equally importantly, what it does **not** mean. It is an assumptions/provenance ledger for M3 rather than a literature calibration.

## Why the first M3 model is synthetic

The M3 milestone is intended to establish a causally inspectable resource mechanism and verify that resource scarcity can propagate into condition and survival. The current world itself is synthetic and does not have empirical biomass, caloric productivity, rainfall or palaeoclimate units. Assigning apparently realistic calorie values to that world would therefore create false precision rather than improve scientific validity.

The executable resource preset is consequently named `synthetic_validation_v1` and carries provenance `synthetic_validation`.

Its purpose is to answer engineering/model questions such as:

- does renewable stock regenerate deterministically from local conditions?
- do annual demand and unconstrained annual regeneration conserve exactly across the scheduler's resource periods?
- does seasonal phase redistribute regeneration through the year without silently changing the unconstrained annual total?
- does local demand compete for a finite cell stock?
- does household sharing reconcile exactly with acquisition and need?
- can persistent scarcity reduce condition and increase mortality pressure?
- do otherwise-equal richer and zero-productivity test environments differ in the expected direction?
- can M3 and M4 use one consistent definition of current-period demand?
- can the system do this at the target population scale without global pairwise searches?

Passing those tests does not establish that the parameter values describe any real hunter-gatherer population.

## Current state variables and units

| Quantity | Current unit | Provenance | Interpretation boundary |
|---|---|---|---|
| Cell baseline productivity | 0..1000 synthetic index | M1 synthetic validation | Relative environmental input, not biomass/NPP/calories |
| Cell dynamic food stock | abstract integer units | M3 synthetic validation | Finite renewable stock, not kg or kcal |
| Person annual need | abstract units/person/year | M3 synthetic validation | Fixed annual demand scale, not human caloric requirement |
| Resource periods | periods/year | explicit model choice | Default 4; exact half-open integer-day scheduler intervals, not assumed equal-duration quarters |
| Productivity scale | permille | experimental control | Multiplies synthetic productivity; not an empirical productivity estimate |
| Seasonality scale | permille | experimental control | Scales timing variation of the synthetic seasonal curve; v8 treats seasonality as mean-preserving redistribution of annual regeneration |
| Stock capacity | years of baseline synthetic regeneration | synthetic rule | Numerical stock ceiling, not ecological carrying capacity |
| Condition | 0..1000 permille | synthetic mediator | Not BMI, body-fat percentage, nutritional biomarker or clinical health score |
| Condition recovery/loss | permille/resource period | synthetic response rule | Not calibrated physiology; still period-frequency dependent pending #204 |
| Scarcity mortality maximum | probability per million/resource boundary | synthetic response rule | Additional mortality mechanism, not an empirical starvation schedule; opportunity frequency still depends on `periodsPerYear` pending #204 |

## Executable time/accounting contract

The normative executable definition is [M3 resource-time contract v1](m3-resource-time-contract-v1.md).

For `P = periodsPerYear`, resource period `i` is the half-open interval:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`.

A fixed annual integer quantity `Q` is allocated by cumulative elapsed days:

`C_Q(t) = floor(Q * t / 365)`

and the period share is `C_Q(end) - C_Q(start)`.

This means the default four-period schedule has durations `91, 91, 91, 92` days. An annual need of `100` therefore executes as `24, 25, 25, 26` units per person across the four periods. This is an integer conservation rule tied to actual model time, not an empirical claim about seasonal human consumption.

M4 uses the same current-period demand allocation when evaluating local resource support. It no longer substitutes an independent `ceil(annual / periods)` approximation.

## Implemented causal assumptions

### Renewable resource production

Each cell has a baseline productivity inherited from the synthetic M1 world. M3 converts that relative index into a fixed annual regeneration baseline using explicit integer multipliers, productivity scale and environmental stress. Synthetic seasonality then redistributes that annual potential through the model year.

The existing triangular seasonal curve is integrated over every model day in each actual resource interval and normalized by its complete-year integral. Therefore, when stock capacity is not binding, changing seasonal phase changes the **timing** of potential regeneration but not the annual potential total. Changing resource-period resolution likewise changes temporal aggregation rather than creating or destroying annual potential by repeated integer division.

Finite stock capacity remains a separate causal constraint: potential regeneration can be clipped when a cell is already near capacity, so realized annual regeneration need not equal unconstrained annual potential.

This asserts only the qualitative mechanism:

> other things equal, more productive/less stressed cells can renew more resource than less productive/more stressed cells, while seasonal phase can redistribute when that potential becomes available.

The functional form and magnitudes are not currently evidence-grounded ecological equations.

### Local density competition

Living people create resource need through their household. Household demand is summed within the provisioning cells associated with the household during the current M3 period. If available stock cannot meet total cell demand, competing claims receive proportional shares according to need.

M9 temporary presence can split one household's period demand between residence and visitor destination according to the existing duration-weighted provisioning contract. That separate rounding contract remains subject to its own open audit findings where applicable.

This asserts a transparent finite-resource competition mechanism without modelling specific foraging technology, search time, patch choice, territorial exclusion, prey depletion, plant phenology, cooperative hunting, exchange or conflict.

### Household pooling and sharing

Acquired resource is pooled at household level. Every living member receives the same supply fraction during a positive-demand period. Harvest is consumed immediately.

This is a deliberately minimal sharing rule. It is not a claim that real forager households distribute food equally. Age/sex/status asymmetry, donor-recipient networks, inter-household sharing, storage, spoilage, waste and trade are absent.

### Condition response

For a **positive-demand** interval, full household supply permits bounded condition recovery. Incomplete supply produces condition loss proportional to the deficit. Condition is bounded 0..1000.

A **zero-demand** interval is explicitly condition-neutral. It does not interpret `0 / 0` as full supply and therefore cannot create free recovery merely because integer annual demand allocates zero units to that interval.

Condition is a causal mediator chosen so resource scarcity does not directly rewrite baseline demographic schedules. The numerical response is synthetic and not currently mapped to measured human physiology.

Important unresolved timing boundary: `conditionRecoveryPerPeriod` and `maxConditionLossPerPeriod` are still per-period quantities. Changing `periodsPerYear` therefore changes the number of physiological update opportunities in a year. That is #204 and is not claimed to be solved by the v8 annual resource-accounting repair.

### Mortality response

At each resource boundary, an individual's additional scarcity-mortality probability increases with condition deficit. This draw uses its own deterministic named random stream and is additive in scheduling to the annual M2 baseline mortality process.

This is an explicit hypothesis-bearing link:

> sustained nutritional/resource shortfall can worsen condition and thereby increase mortality pressure.

The current linear probability function and maximum probability are placeholders. They must not be interpreted as an estimate of real starvation mortality.

Two related limitations remain explicit after the resource-time repair:

- scarcity-mortality opportunity count is still tied to `periodsPerYear` pending #204;
- the shared condition state can include M4 travel costs, so a later death can be recorded under the broad `ResourceScarcity` cause even when travel contributed to the deficit (#200).

Competing-risk attribution when an M3 and M2 mortality boundary coincide is separately tracked by #208.

### Fertility

M3 does **not** modify fertility as a function of resources or condition. Food availability changing fertility may be a plausible future hypothesis, but it is deliberately excluded until a specific mechanism/evidence basis is documented. This avoids baking a contested or population-specific relationship into the model merely because it creates convenient population regulation.

## Current default synthetic parameters

`synthetic_validation_v1` currently uses:

| Parameter | Default |
|---|---:|
| Resource periods per year | 4 |
| Annual need per person | 100 abstract units |
| Annual regeneration units per productivity point | 1 |
| Productivity scale | 1000 permille |
| Seasonality scale | 1000 permille |
| Cell stock capacity | 10 synthetic baseline-regeneration years |
| Condition recovery per fully supplied positive-demand period | 25 permille |
| Maximum condition loss per positive-demand period | 200 permille |
| Maximum scarcity-mortality probability | 200,000 per million per resource boundary |

These values were selected to exercise the system across surplus/scarcity regimes and are **not empirical estimates**.

## Verification claims M3 may make

Once the relevant CI and acceptance tests pass, it is legitimate to say that the implementation verifies properties such as:

- same configuration/seed yields the same resource-demographic trajectory within the declared determinism boundary;
- cell resource accounting reconciles across initialization, regeneration and harvest;
- fixed annual integer quantities conserve exactly across the scheduler's resource periods;
- zero-amplitude seasonal allocation reduces to the fixed elapsed-day allocation;
- unconstrained seasonal annual potential is invariant to phase and tested period resolutions while within-year timing can differ;
- M3 and M4 resolve the same current-period demand;
- positive-demand period demand reconciles to consumption plus unmet need;
- zero-demand intervals do not create condition recovery;
- zero sustained resources can lower condition and survival under a configured severe synthetic shock;
- an otherwise-equal positive-resource test case can support better survival/condition than a zero-productivity case; and
- resource processing remains local/data-oriented rather than global pairwise interaction.

It is **not** legitimate to infer from those tests that a specific prehistoric population would have experienced the simulated mortality, carrying capacity or resource requirements.

## Evidence required before an empirical resource preset

A future research-capable resource configuration should investigate and document, as appropriate to the research question:

1. human energetic requirements by age, sex, reproductive state, activity and climate;
2. ethnographic/archaeological evidence on food acquisition, sharing and storage where suitable;
3. resource-return distributions and their seasonal/environmental variability;
4. ecological productivity or palaeoenvironmental reconstruction with explicit spatial/temporal units;
5. how resource availability maps to nutritional/body condition at the model's time resolution;
6. how condition maps to mortality and, if later justified, fertility;
7. disease and infection as competing or interacting mortality mechanisms;
8. uncertainty ranges and population/context dependence rather than one universal forager parameter set;
9. calibration targets that are separate from validation targets;
10. sensitivity and uncertainty analysis showing whether conclusions depend on arbitrary energetic assumptions.

Sources, transformations, units, licences and uncertainty must be recorded alongside any future empirical preset.

## M3 interpretation rule

Results produced by `synthetic_validation_v1` should be described in conditional model language:

> Under the stated synthetic resource, timing and response assumptions, changing X caused Y within the model.

They should not be promoted to statements such as:

> Hunter-gatherers require X resources, or scarcity caused Y mortality in prehistory.

That distinction remains mandatory until the relevant configuration has empirical provenance and validation claims appropriate to the question being investigated.