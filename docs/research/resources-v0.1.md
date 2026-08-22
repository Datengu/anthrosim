# v0.1 resource-model provenance and evidence boundary

**Status:** M3 synthetic validation baseline  
**Scientific status:** unvalidated  
**Runtime empirical dataset:** none

This document records what the first executable AnthroSim resource model means and, equally importantly, what it does **not** mean. It is an assumptions/provenance ledger for M3 rather than a literature calibration.

## Why the first M3 model is synthetic

The M3 milestone is intended to establish a causally inspectable resource mechanism and verify that resource scarcity can propagate into condition and survival. The current world itself is synthetic and does not have empirical biomass, caloric productivity, rainfall or palaeoclimate units. Assigning apparently realistic calorie values to that world would therefore create false precision rather than improve scientific validity.

The executable resource preset is consequently named `synthetic_validation_v1` and carries provenance `synthetic_validation`.

Its purpose is to answer engineering/model questions such as:

- does renewable stock regenerate deterministically from local conditions?
- does local demand compete for a finite cell stock?
- does household sharing reconcile exactly with acquisition and need?
- can persistent scarcity reduce condition and increase mortality pressure?
- do otherwise-equal richer and zero-productivity test environments differ in the expected direction?
- can the system do this at the target population scale without global pairwise searches?

Passing those tests does not establish that the parameter values describe any real hunter-gatherer population.

## Current state variables and units

| Quantity | Current unit | Provenance | Interpretation boundary |
|---|---|---|---|
| Cell baseline productivity | 0..1000 synthetic index | M1 synthetic validation | Relative environmental input, not biomass/NPP/calories |
| Cell dynamic food stock | abstract integer units | M3 synthetic validation | Finite renewable stock, not kg or kcal |
| Person annual need | abstract units/person/year | M3 synthetic validation | Demand scale, not human caloric requirement |
| Resource periods | periods/year | explicit model choice | Default 4; scheduling approximation |
| Productivity scale | permille | experimental control | Multiplies synthetic productivity; not an empirical productivity estimate |
| Stock capacity | years of baseline synthetic regeneration | synthetic rule | Numerical stock ceiling, not ecological carrying capacity |
| Condition | 0..1000 permille | synthetic mediator | Not BMI, body-fat percentage, nutritional biomarker or clinical health score |
| Condition recovery/loss | permille/period | synthetic response rule | Not calibrated physiology |
| Scarcity mortality maximum | probability per million/period | synthetic response rule | Additional mortality mechanism, not an empirical starvation schedule |

## Implemented causal assumptions

### Renewable resource production

Each cell has a baseline productivity inherited from the synthetic M1 world. M3 converts that relative index into dynamic resource regeneration using explicit integer multipliers, synthetic seasonality and environmental stress. Regeneration is capped by a finite cell stock capacity.

This asserts only the qualitative mechanism:

> other things equal, more productive/less stressed cells can renew more resource than less productive/more stressed cells.

The functional form and magnitudes are not currently evidence-grounded ecological equations.

### Local density competition

Living people create resource need through their household. Household demand is summed within the cell the household currently occupies. If available stock cannot meet total cell demand, co-located households receive proportional shares according to need.

This asserts a transparent finite-resource competition mechanism without modelling specific foraging technology, search time, patch choice, territorial exclusion, prey depletion, plant phenology, cooperative hunting, exchange or conflict.

### Household pooling and sharing

Acquired resource is pooled at household level. Every living member receives the same supply fraction during a period. Harvest is consumed immediately.

This is a deliberately minimal sharing rule. It is not a claim that real forager households distribute food equally. Age/sex/status asymmetry, donor-recipient networks, inter-household sharing, storage, spoilage, waste and trade are absent.

### Condition response

Full household supply permits bounded condition recovery. Incomplete supply produces condition loss proportional to the deficit. Condition is bounded 0..1000.

This is a causal mediator chosen so resource scarcity does not directly rewrite baseline demographic schedules. The numerical response is synthetic and not currently mapped to measured human physiology.

### Mortality response

At each resource boundary, an individual's additional scarcity-mortality probability increases with condition deficit. This draw uses its own deterministic named random stream and is additive in scheduling to the annual M2 baseline mortality process.

This is an explicit hypothesis-bearing link:

> sustained nutritional/resource shortfall can worsen condition and thereby increase mortality pressure.

The current linear probability function and maximum probability are placeholders. They must not be interpreted as an estimate of real starvation mortality.

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
| Cell stock capacity | 10 synthetic baseline-regeneration years |
| Condition recovery per fully supplied period | 25 permille |
| Maximum condition loss per period | 200 permille |
| Maximum scarcity-mortality probability | 200,000 per million per period |

These values were selected to exercise the system across surplus/scarcity regimes and are **not empirical estimates**.

## Verification claims M3 may make

Once CI and the M3 milestone acceptance tests pass, it is legitimate to say that the implementation verifies properties such as:

- same configuration/seed yields the same resource-demographic trajectory within the declared determinism boundary;
- cell resource accounting reconciles globally across initialization, regeneration and harvest;
- period demand reconciles to consumption plus unmet need;
- zero sustained resources can lower condition and survival under a configured severe synthetic shock;
- an otherwise-equal positive-resource test case can support better survival/condition than a zero-productivity case;
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

> Under the stated synthetic resource and response assumptions, changing X caused Y within the model.

They should not be promoted to statements such as:

> Hunter-gatherers require X resources, or scarcity caused Y mortality in prehistory.

That distinction remains mandatory until the relevant configuration has an empirical provenance and validation claim appropriate to the question being investigated.
