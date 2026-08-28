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

## Protected M7.6 rebaseline review

The protected v14 M7.6 source was CI run `33121918130`, head `fa8604452c267b918d7c542e96c8ec7f8d5152a9`, test-merge commit `d1df97e9e47dd1e252e683fc7cbc05ceedeec644`, sweep `anthrosim-sweep-v2-2435a2eee06d7132`. The reviewed v15 source was CI run `33129843888`, head `70b123afd685168b8dd5a4a38e6bebd380cbab6b`, test-merge commit `e86cef007694bdfae6377248b27f737b8fd7693e`, sweep `anthrosim-sweep-v2-f26b183e8d3eefbb`.

Both sources used the same definition identity `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`, the same 18-point factorial design and paired seeds. All 144 runs completed and remained scientifically eligible in both versions; no run changed stop reason or censoring category.

Across all 144 paired runs, v14 -> v15 mean changes were: final living population `967.2639 -> 990.5972` (+23.3333), births `4533.5556 -> 4577.1250` (+43.5694), total deaths `8566.2917 -> 8586.5278` (+20.2361), condition-mediated deaths `1781.2361 -> 1755.3264` (-25.9097), unmet resource need `177967.5000 -> 174407.9514` (-3559.5486), migration moves `6230.7153 -> 6073.8264` (-156.8889), and migration distance `12653.2083 -> 12327.0069` (-326.2014).

The migration-disabled half of the design is the key causal control. Migration remained exactly zero in all 72 control runs, while final population increased by 15.2222 per run on average, condition-mediated deaths fell by 50.1528, unmet need fell by 6628.4861, and inferred non-condition/background deaths increased by 74.9028. This is consistent with the intended mortality repair: background deaths may occur earlier inside the year, reducing later resource exposure and fertility eligibility, while dual-trigger deaths no longer inherit M3 cause priority. The resulting population/resource feedback then changes later births and deaths even though the configured complete-year background survival probability is preserved for a fixed annual schedule.

With migration enabled, the changed mortality timing and survivor set propagate downstream into M4: mean moves fell by 313.7778 and total migration distance by 652.4028 per run. Because the migration-disabled controls remain at exactly zero moves and the M4 choice kernel is unchanged, this is downstream trajectory movement rather than evidence of a direct migration regression.

All 144 deterministic state digests changed, which is expected because interval-aligned background mortality consumes the existing mortality RNG stream at different boundaries than v14. No new RNG stream or checkpoint field was introduced, and the repository's deterministic/golden gates remained green at the reviewed source head. No unexplained scientific-reference movement was found.

Accordingly, only the protected synthetic M7.6 reference snapshot is rebaselined to the reviewed v15 results, and the CI semantics guard is advanced from v14 to v15. The rebaseline is regression evidence for the new model semantics, not empirical validation.

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
