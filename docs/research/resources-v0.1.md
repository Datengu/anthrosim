# v0.1 resource-model provenance and evidence boundary

**Status:** M3 synthetic validation baseline, executable timing refined by the [M3 resource-time contract v1](m3-resource-time-contract-v1.md), [M3/M4 response-time contract v1](m3-response-time-contract-v1.md), and [M3 condition-mediated mortality contract v1](m3-condition-mortality-contract-v1.md)  
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
- can persistent scarcity reduce condition and increase condition-mediated mortality pressure?
- do otherwise-equal richer and zero-productivity test environments differ in the expected direction?
- can M3 and M4 use one consistent elapsed-time allocation rule for annual demand while retaining independent clocks?
- can changing M3 settlement resolution avoid silently multiplying condition, condition-mediated mortality and M4 decision opportunity rates?
- can the system do this at the target population scale without global pairwise searches?

Passing those tests does not establish that the parameter values describe any real hunter-gatherer population.

## Current state variables and units

| Quantity | Current unit | Provenance | Interpretation boundary |
|---|---|---|---|
| Cell baseline productivity | 0..1000 synthetic index | M1 synthetic validation | Relative environmental input, not biomass/NPP/calories |
| Cell dynamic food stock | abstract integer units | M3 synthetic validation | Finite renewable stock, not kg or kcal |
| Person annual need | abstract units/person/year | M3 synthetic validation | Fixed annual demand scale, not human caloric requirement |
| Resource periods | M3 settlements/year | explicit model choice | Default 4; exact half-open integer-day integration/settlement intervals, not M4 opportunity frequency |
| Productivity scale | permille | experimental control | Multiplies synthetic productivity; not an empirical productivity estimate |
| Seasonality scale | permille | experimental control | Scales timing variation of the synthetic seasonal curve; v8+ treats seasonality as mean-preserving redistribution of annual regeneration |
| Stock capacity | years of baseline synthetic regeneration | synthetic rule | Numerical stock ceiling, not ecological carrying capacity |
| Condition | 0..1000 permille | shared synthetic mediator | General health/energetic model state, not a resource-only state, BMI, body-fat percentage, nutritional biomarker or clinical health score |
| Condition recovery/loss | permille/reference quarter | synthetic response rule | Legacy `...PerPeriod` wire names are retained, but v9+ interprets values against one of four canonical response quarters and rescales them by elapsed M3 interval |
| Condition-mortality maximum | probability per million/reference quarter at zero condition | synthetic response rule | General condition-mediated mortality mechanism; v9+ converts the reference probability to an exact survival-equivalent probability for the elapsed M3 interval |

## Executable time/accounting contract

The normative annual resource-allocation definition is [M3 resource-time contract v1](m3-resource-time-contract-v1.md). The independent response/opportunity timing semantics introduced by v9 are normative in [M3/M4 response-time contract v1](m3-response-time-contract-v1.md). The v10 cause semantics for the shared-condition hazard are normative in [M3 condition-mediated mortality contract v1](m3-condition-mortality-contract-v1.md). The v11 indivisible-unit competition rule is defined in the local-density section below.

For `P = periodsPerYear`, M3 resource period `i` is the half-open interval:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`.

A fixed annual integer quantity `Q` is allocated by cumulative elapsed days:

`C_Q(t) = floor(Q * t / 365)`

and the period share is `C_Q(end) - C_Q(start)`.

This means the default four-period M3 schedule has durations `91, 91, 91, 92` days. An annual need of `100` therefore executes as `24, 25, 25, 26` units per person across those four M3 periods. This is an integer conservation rule tied to actual model time, not an empirical claim about seasonal human consumption.

M4 uses the same cumulative annual-quantity rule for its resource-support cue, but applies it to the independently configured M4 decision interval. When M3 and M4 both use the default four-per-year clocks their demand shares coincide; when their frequencies differ, each process uses the annual share corresponding to its own elapsed interval rather than pretending the clocks are the same.

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

Because food stock is represented by indivisible integer units, v11 makes the rounding rule part of the explicit model semantics. For each cell, every claim first receives

`floor(cell_target × claim_need / cell_demand)`.

Any remaining cell units are then assigned by the **largest-remainder method**: claims with larger fractional remainders receive an indivisible unit before claims with smaller remainders. This preserves the proportional target as closely as the integer representation permits and reconciles exactly to the cell's available allocation target.

Exact fractional ties do not use stable household or claim order as a permanent priority. Tied claims form a deterministic ring whose starting position rotates with the persisted M3 resource-period sequence and cell index. No additional RNG stream is consumed. For an unchanged repeated `n`-way exact tie, the priority therefore cycles across the tied claims rather than granting the same low-ID claim the rounding unit every period. Stable claim order supplies only the reproducible ring ordering; it does not provide a persistent first-claim advantage.

This rotating tie rule is a **numerical apportionment null rule**, not a claim about social priority, status, territorial access, sharing norms or historical food-distribution institutions. If such mechanisms are scientifically required, they need their own explicit model and evidence rather than being inferred from bookkeeping identity.

M9 temporary presence can split one household's period demand between residence and visitor destination according to the existing duration-weighted provisioning contract. That home-versus-visitor split is upstream of cell competition and is deliberately unchanged by v11; its separate exact-tie semantics remain tracked independently by #194.

This asserts a transparent finite-resource competition mechanism without modelling specific foraging technology, search time, patch choice, territorial exclusion, prey depletion, plant phenology, cooperative hunting, exchange or conflict.

### Household pooling and sharing

Acquired resource is pooled at household level. Every living member receives the same supply fraction during a positive-demand M3 interval. Harvest is consumed immediately.

This is a deliberately minimal sharing rule. It is not a claim that real forager households distribute food equally. Age/sex/status asymmetry, donor-recipient networks, inter-household sharing, storage, spoilage, waste and trade are absent.

### Condition response

For a **positive-demand** interval, full household supply permits bounded condition recovery. Incomplete supply produces condition loss proportional to the deficit. Condition is bounded 0..1000.

A **zero-demand** interval is explicitly condition-neutral. It does not interpret `0 / 0` as full supply and therefore cannot create free recovery merely because integer annual demand allocates zero units to that interval.

Condition is a shared causal mediator. M3 resource balance can change it, but it is not resource-specific: M4 permanent-travel cost also changes the same state, and initialization/newborn rules supply condition values. The numerical state is synthetic and not currently mapped to measured human physiology.

Under v9+, the historical `conditionRecoveryPerPeriod` and `maxConditionLossPerPeriod` serialized fields are **reference-quarter response quantities**. The model has four canonical response quarters `[0,91)`, `[91,182)`, `[182,273)`, `[273,365)`. For each actual M3 interval, the executable response budget is the deterministic cumulative share attributable to the elapsed overlap with those reference quarters. Holding supply status otherwise fixed, a full model year therefore receives the same configured response budget under tested M3 partitions of 1, 4, 12 and 365 rather than multiplying the response by the number of M3 boundaries.

This repair removes the former numerical opportunity-count artifact. It does not require complete trajectory invariance when resource settlement is changed: stock timing, capacity clipping, changing supply fractions, M9 presence, condition state and extinction can legitimately evolve differently when state is observed/settled at different times.

### Condition-mediated mortality response

Condition-mediated mortality is evaluated when M3 settles an elapsed resource interval. Under v10 the run-facing `maxConditionMortalityProbabilityPerMillion` means the conditional probability at zero condition over one canonical reference quarter, with the actual condition deficit scaling that reference probability.

For an arbitrary M3 interval, the v9 timing contract converts the reference-quarter probability to an exact rational conditional probability from the elapsed interval's survival ratio. At fixed condition, composing all M3 intervals over a complete year therefore gives the same survival probability for tested partitions 1, 4, 12 and 365, equal to four reference-quarter survivals. At the default four-period clock the executable interval probability is exactly the configured reference-quarter probability after condition scaling.

The actual stochastic draw uses the exact rational probability. The authoritative death event's parts-per-million field is an observability representation of that interval probability and uses a deterministic ceiling; it is not the quantity used to decide the draw.

The causal statement supported by the mechanism is:

> lower shared condition increases the configured condition-mediated mortality pressure.

Resource scarcity can contribute to that pathway by lowering condition. M4 travel can also contribute by lowering condition. Because the state does not retain a decomposition by upstream source, v10 death events serialize `cause = condition_mediated` and the resource summary reports `conditionMortalityDeaths`; neither may be interpreted as an event-level count of food-scarcity deaths.

The current condition-deficit function and maximum probability are placeholders. They must not be interpreted as estimates of real starvation, travel, frailty or general mortality.

One major related limitation remains after #200:

- competing-risk attribution when an M3 condition-mediated and M2 demographic mortality boundary coincide remains separately tracked by #208.

### Fertility

M3 does **not** modify fertility as a function of resources or condition. Food availability changing fertility may be a plausible future hypothesis, but it is deliberately excluded until a specific mechanism/evidence basis is documented. This avoids baking a contested or population-specific relationship into the model merely because it creates convenient population regulation.

## Current default synthetic parameters

`synthetic_validation_v1` currently uses:

| Parameter | Default |
|---|---:|
| M3 resource settlements per year | 4 |
| Annual need per person | 100 abstract units |
| Annual regeneration units per productivity point | 1 |
| Productivity scale | 1000 permille |
| Seasonality scale | 1000 permille |
| Cell stock capacity | 10 synthetic baseline-regeneration years |
| Condition recovery per fully supplied reference quarter | 25 permille |
| Maximum condition loss per reference quarter | 200 permille |
| Maximum condition-mediated mortality probability at zero condition per reference quarter | 200,000 per million |

M4's synthetic default is separately four permanent-migration decision periods per year; that value belongs to `MigrationConfig`, not to M3 resource resolution.

These values were selected to exercise the system across surplus/scarcity regimes and are **not empirical estimates**.

## Verification claims M3 may make

Once the relevant CI and acceptance tests pass, it is legitimate to say that the implementation verifies properties such as:

- same configuration/seed yields the same resource-demographic trajectory within the declared determinism boundary;
- cell resource accounting reconciles across initialization, regeneration and harvest;
- fixed annual integer quantities conserve exactly across the scheduler's resource periods;
- zero-amplitude seasonal allocation reduces to the fixed elapsed-day allocation;
- unconstrained seasonal annual potential is invariant to phase and tested period resolutions while within-year timing can differ;
- M3 and M4 use the same cumulative elapsed-time rule for annual demand on their respective clocks;
- scarce cell allocation follows largest fractional remainders and repeated exact ties rotate rather than permanently favoring the first stable claim;
- positive-demand period demand reconciles to consumption plus unmet need;
- zero-demand intervals do not create condition recovery;
- controlled full-supply and full-deficit condition-response budgets do not multiply when M3 is partitioned into 1, 4, 12 or 365 intervals;
- fixed-condition mortality survival is equivalent under tested M3 partitions 1, 4, 12 and 365;
- changing M3 resource resolution does not itself change the configured M4 opportunity count;
- zero sustained resources can lower condition and survival under a configured severe synthetic shock;
- an otherwise-equal positive-resource test case can support better survival/condition than a zero-productivity case;
- travel-created low condition with full positive food supply is not falsely serialized as a resource-scarcity death; and
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

For a controlled resource intervention, a valid causal statement can be:

> Under otherwise fixed model assumptions, reducing resource support lowered condition and increased condition-mediated mortality.

They should not be promoted to statements such as:

> Hunter-gatherers require X resources, or the model observed N prehistoric starvation deaths.

That distinction remains mandatory until the relevant configuration has empirical provenance and validation claims appropriate to the question being investigated.