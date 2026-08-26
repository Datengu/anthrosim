# TRACE change record — M3 resource-time accounting repair

**Date:** 2026-08-26  
**Programme:** post-M9 scientific hardening / first M3 causal repair slice  
**Entering model semantics:** `anthrosim-model-semantics-v7`  
**Proposed model semantics:** `anthrosim-model-semantics-v8`  
**Scientific status:** implementation verification / conceptual-model repair; **not empirical validation**

## Purpose

This record documents the first coherent M3 repair slice after completion of the M2 hardening cluster. It addresses three linked findings:

- **#180** — annual resource quantities and M4 demand did not share one periodization contract;
- **#189** — seasonal regeneration sampled one endpoint per resource period, allowing phase and period resolution to alter annual potential yield unintentionally;
- **#199** — zero-demand intervals treated `0 / 0` as full supply and therefore created free condition recovery.

The normative executable specification is [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md). The evidence/provenance boundary remains [`resources-v0.1.md`](resources-v0.1.md).

This slice deliberately does not absorb the distinct remaining condition/timing/cause findings #204, #200, #208 or the downstream acceptance scope of #201.

## 1. Problem formulation

The verification question is:

> Can every quantity stated per model year be mapped deterministically to the exact elapsed resource intervals, can M3 and M4 use the same current-period demand, can seasonality change timing without silently changing the unconstrained annual total, and can a zero-demand interval remain physiologically neutral?

This is a model-contract question. It does not ask whether the annual quantities or seasonal curve are empirically correct for any archaeological population.

## 2. Model description

For `P = periodsPerYear`, v8 defines resource period `i` as:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`.

For a fixed annual integer quantity `Q`, cumulative allocation after `t` elapsed model days is:

`C_Q(t) = floor(Q * t / 365)`.

The period allocation is therefore:

`C_Q(end) - C_Q(start)`.

This conserves the complete annual quantity exactly while respecting the scheduler's unequal integer-day period lengths. With four periods, the model intervals are 91, 91, 91 and 92 days; annual need `100` executes as `24, 25, 25, 26`.

M4 no longer derives an independent `ceil(annual / periods)` demand approximation. At a legitimate resource boundary it obtains the exact same per-person period share used by M3. A non-resource boundary fails closed rather than guessing a demand period.

Seasonal regeneration uses the existing synthetic triangular factor as a **within-year weighting curve**. The daily weights are integrated over each exact half-open interval and normalized by the complete-year weight, so seasonal phase changes timing but not unconstrained annual potential. Zero seasonal amplitude reduces exactly to the fixed elapsed-day allocation.

A household with executable need `0` in an interval receives no condition update from provisioning: zero demand is neutral rather than interpreted as full supply.

## 3. Data evaluation

No new empirical data are introduced. No parameter is calibrated or retuned.

The default resource configuration remains `synthetic_validation_v1`; its productivity, annual need, regeneration, seasonality, condition and mortality quantities remain abstract synthetic mechanism-testing values.

This repair changes how those already-declared annual quantities are executed in model time. It does not make them more empirically valid.

## 4. Conceptual-model evaluation

The repair removes three bookkeeping artefacts from the causal graph:

1. annual need no longer changes meaning because of unrelated integer division conventions;
2. M4 resource utility no longer evaluates a different demand quantity from the M3 process that just settled that interval;
3. seasonal phase/resolution no longer acts as an undeclared multiplier on unconstrained annual regeneration potential.

It also prevents zero-demand integer shares from becoming an accidental positive physiological intervention.

The remaining causal limitations are intentionally visible:

- **#204:** condition recovery/loss and scarcity-mortality draws remain per resource period/boundary, so changing `periodsPerYear` can still change annual physiological and mortality opportunity counts;
- **#200:** shared condition still mixes resource and M4 travel damage before a later broad `ResourceScarcity` death attribution;
- **#208:** coincident M3/M2 mortality remains sequential competing-risk scheduling;
- **#201:** newborn-condition repair still needs downstream M3/M4 interaction acceptance before closure.

## 5. Implementation verification

The implementation provides one shared resource-period allocation layer used by M3 and M4.

Predeclared verification includes:

- exact period boundaries and annual conservation for `P = 1, 3, 4, 5, 12, 365`;
- annual quantities including `0`, `1`, non-divisible values and larger values;
- the explicit four-period `100 -> 24,25,25,26` contract;
- boundary-day lookup matching period-index allocation and rejecting non-boundaries;
- zero-amplitude seasonal allocation matching fixed elapsed-day allocation exactly;
- integrated seasonal annual potential summing to the same annual quantity across tested phases and period resolutions;
- non-zero amplitude/phase changing within-year allocation timing;
- zero-demand intervals preserving reduced condition rather than healing it;
- existing positive-demand scarcity and resource-accounting directionality tests;
- full deterministic/checkpoint/reference workflow review under the new semantics identity.

The seasonal prefix table is deterministic derived state only. It is precomputed from model constants to avoid a 365-day per-cell inner loop and does not introduce fitted or hidden causal state.

## 6. Model-output verification

Because this is an authoritative semantics change, exact v7 synthetic output references are not assumed to remain valid.

The review rule is:

1. run the existing frozen references under v8;
2. inspect every mismatch;
3. determine whether it follows mechanistically from the declared resource-time change;
4. regenerate a reference only when the changed output is understood and expected; and
5. record the regeneration rather than tuning v8 behavior back toward v7 output.

Cross-platform equality and checkpoint/resume equivalence remain required within v8.

No empirical resource-output target or tolerance is introduced here, so empirical TRACE model-output validation remains unestablished.

## 7. Model analysis

The repair improves interpretability of future sensitivity work because `periodsPerYear` no longer simultaneously changes the annual demand total through one rounding rule and M4's resource expectation through another.

However, `periodsPerYear` is still not yet a clean numerical-resolution parameter because #204 remains: per-period condition and mortality/decision opportunities can still change outcomes. Therefore a period-resolution sensitivity experiment performed immediately after this slice would still mix numerical and substantive mechanisms.

Closing #180/#189/#199 must not be interpreted as closing temporal-resolution sensitivity for M3 as a whole.

## 8. Corroboration

None. No archaeological, palaeoecological, ethnographic or physiological corroboration is attempted or claimed.

## Issue-level closure interpretation

If PR #238 passes its final exact-head acceptance suite and the reference differences are reviewed, the evidence in this change is intended to close:

- **#180** — fixed annual quantities conserve under one scheduler-aligned contract and M4 uses the same period demand as M3;
- **#189** — seasonal regeneration is integrated over the actual interval and normalized to preserve unconstrained annual potential across phase/resolution;
- **#199** — zero-demand intervals are condition-neutral and regression tested.

The following remain open after this slice:

- **#204** — resource-period frequency still multiplies physiological/mortality/M4 opportunity clocks;
- **#200** — resource versus travel contributions to shared condition/death attribution;
- **#208** — coincident M3/M2 mortality attribution;
- **#201** — downstream newborn-condition/resource/migration interaction acceptance.

## Compatibility and reproducibility

The repair can change resource stock, unmet need, condition, scarcity death and M4 relocation trajectories. It is therefore **not** semantics-neutral.

`MODEL_SEMANTICS_ID` changes from v7 to v8. v7 checkpoints must not be resumed as if they were v8 trajectories. Package version remains unchanged in this development slice because package/release numbering is governed separately.

## Scientific boundary

A fully green v8 verification suite would establish that the simulator executes this resource-time contract consistently and reproducibly.

It would not establish that synthetic resource units represent calories, biomass or palaeoproductivity; that the triangular seasonality function is ecologically realistic; that condition is a valid physiological proxy; or that resulting simulated population/mobility patterns are empirically correct. Those remain subject to evidence provenance, uncertainty/sensitivity analysis, independent validation targets and corroboration under TRACE.