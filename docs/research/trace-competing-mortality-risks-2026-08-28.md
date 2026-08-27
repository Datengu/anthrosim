# TRACE record — competing mortality risks

**Date:** 2026-08-28  
**Issue:** #208  
**Target semantics:** `anthrosim-model-semantics-v15`

## Audit finding

The post-v0.3.0 TRACE audit found that coincident M3 condition-mediated and M2 demographic mortality were represented by sequential function calls. M3 ran first, so a same-boundary M3 death prevented the M2 draw and automatically received condition-mediated cause attribution. This made cause-specific mortality sensitive to scheduler implementation order.

## Scientific decision

AnthroSim v15 uses a discrete cause-specific competing-risk contract:

- M2 mortality remains an age-at-year-start annual background risk parameter;
- that annual risk is converted to exact conditional risks over elapsed M3 intervals;
- the existing M3 condition risk keeps the v9/v10 exact reference-quarter conversion;
- both causes receive independent latent triggers;
- all-cause survival is the product of the two cause-specific survivals;
- a dual trigger produces one death and a symmetric risk-weighted cause allocation;
- the year-end M2 stage no longer redraws mortality and retains fertility/parentage only.

This preserves #179's age-interval correction, #204's elapsed-time condition-mortality semantics, and #200's general `condition_mediated` causal naming.

## Determinism/provenance decision

No new RNG stream is introduced. The two existing streams remain continuation state:

- `demography/mortality` for background latent triggers;
- historical private `resources/scarcity_mortality` for condition latent triggers.

When both triggers fire, the cause-allocation entropy is formed symmetrically from both streams, so there is no first-stream/call-order priority. Because the authoritative execution meaning changes, `MODEL_SEMANTICS_ID` advances from v14 to v15. The checkpoint wire shape does not change because no new continuation state is added.

## Observability decision

Background deaths may now occur at M3 interval ends rather than only annual boundaries. Demography observability therefore advances to schema v2 and reconstructs mortality exposures from the exact resource-period schedule and persistent birth/death histories. Fertility replay remains annual.

`Death.probabilityPerMillion` keeps its wire shape but under v15 means the selected cause's interval-specific conditional probability. It is not the combined all-cause probability.

## Same-day M4 interpretation

Mortality is resolved before M9/M4 on a shared boundary. A person cannot permanently migrate first and thereby change the persistent residence attributed to a mortality outcome representing the preceding elapsed interval.

## Empirical boundary

This repair is model-structural, not calibration. Any future empirical mortality schedule must declare whether it is all-cause or cause-specific. An all-cause schedule cannot be added unchanged to explicit condition/disease/conflict/travel causes without double counting.

## Required verification

Before merge:

- focused mortality/math tests must pass;
- M2 and condition-mortality acceptance tests must pass;
- core and spatial hosts must remain deterministic and checkpoint-resumable;
- demography observability must reconcile interval deaths;
- full workspace format/Clippy/tests must pass;
- protected scientific references affected by the v15 trajectory change must be reviewed explicitly rather than silently updated;
- exact final PR head must pass the repository protection matrix.

Any reference rebaseline is synthetic regression evidence only and must not be described as empirical validation.