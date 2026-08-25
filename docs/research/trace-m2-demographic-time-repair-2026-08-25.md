# TRACE change record — M2 demographic-time repair

**Date:** 2026-08-25  
**Programme:** post-M9 scientific hardening / first P1 causal-repair cluster  
**Baseline entering repair:** v0.3.0, completed M9, pre-repair audit convergence recorded through TRACE pass 9  
**Scientific status:** implementation-verification repair in progress; **not empirical validation**

## Purpose

This change record applies TRACE maintenance rules to the first coherent repair cluster after the audit-first P1 discovery phase converged. It records why the M2 model changed, which verification evidence is being generated, which previous synthetic references are invalidated, and which M2 gates remain open.

The normative repaired transition semantics are defined in [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md). This record is evaluation evidence, not a second competing model specification.

## 1. Problem formulation

The repair question is implementation/conceptual verification:

> Does the executable M2 annual demographic process implement one explicit, reproducible and internally coherent time contract for age-specific mortality, fertility, birth spacing, parentage locality and newborn condition without allowing scheduler accidents to define scientific meaning?

This is not a question about a particular archaeological population and does not establish fitness for a specific archaeological site or any other empirical study.

## 2. Model-description change

The repaired annual contract declares:

- an annual demographic transition at positive multiples of 365 days;
- exposure interval `[t-365,t)` for a boundary at day `t`;
- age-band lookup from age at the **start** of that interval;
- mortality first, followed by fertility among surviving eligible females;
- fertility probability interpreted conditionally on surviving the M2 mortality transition, plus spacing and eligible-male filters;
- requested day-valued birth spacing normalized to the smallest executable annual-boundary duration at least as large as the request;
- same-day M4 destinations excluded from the preceding interval's parentage-locality exposure;
- newborn persistent residence taken from the mother's current boundary state;
- newborn initial condition inherited from the female parent's boundary condition;
- `Death.cell` retained as boundary-state persistent residence rather than overloaded as an unmodelled physical/exposure death location.

ODD and ODD+D are synchronized with these scheduling and interaction semantics. `MODEL_SEMANTICS_ID` advances to `anthrosim-model-semantics-v6`.

## 3. Data evaluation

No empirical demographic data were retuned in this repair. The existing `synthetic_validation_v1` schedule remains synthetic/evidence-informed rather than calibrated to a real population.

The semantic change means an empirical schedule used in future must be checked for compatibility with the model's probability conditioning. In particular, an unconditional observed annual live-birth probability must not be inserted as if it were automatically equivalent to the current conditional-survival fertility probability.

Issue #192 remains open because founders still lack explicit pre-simulation reproductive/genealogical history. No invented history is being added merely to restore previous trajectories.

## 4. Conceptual model evaluation

The following formerly separate P1 findings are treated as one causal-time redesign:

- #179 — mortality/fertility age interval;
- #191 — executable birth-spacing timing;
- #193 — same-instant M4 relocation versus M2 parentage locality;
- #201 — newborn condition initialization;
- #227 — mortality/fertility competing opportunity structure.

Issue #192 is part of the same M2 programme but requires persisted initialization/genealogy semantics and therefore remains a separate implementation slice. Issue #228 is the corresponding opportunity/denominator observability work and should be built against the repaired contract.

The repair deliberately does **not** claim subannual or continuous-time demographic hazards. If later research requires within-year birth/death ordering, exact day-scale spacing or continuous competing risks, that is a new structural model requiring a new semantics identity and structural-sensitivity comparison.

## 5. Implementation verification

Scientific/model-contract tests added in the first slice include:

- half-open age-band lookup at exact boundaries;
- a model-born child's first later annual mortality transition using the age-0 interval rather than skipping to age 1;
- requested/effective birth-spacing normalization around annual boundaries;
- the limiting 100% mortality + 100% fertility case, proving the declared mortality-priority/conditional-survival contract;
- newborn condition inheritance;
- same-day M4 relocation not redefining the eligible male-parent pool.

Existing deterministic replay, checkpoint, event/state invariants and long-run soak tests remain active. During implementation, an existing death-event/state invariant rejected an attempted reinterpretation of `Death.cell` as pre-M4 exposure location. The design was corrected instead of weakening the invariant.

This is verification evidence that the software implements the declared model. Synthetic tests do not validate prehistoric demography.

## 6. Model-output verification and frozen-reference impact

Changing M2 semantics necessarily changes downstream synthetic trajectories. Exact M7.6, M8.6 and M9.7 reference results produced under earlier model semantics are therefore **invalidated as current regression baselines** but retained in repository history as evidence of the old model.

The correct response is not to tune M2 until old numbers return. The required process is:

1. run the unchanged predeclared synthetic experiment under the repaired semantics;
2. confirm execution/invariants/determinism remain valid;
3. inspect whether the qualitative capability/classification survives or changes;
4. record the new source/model-semantics identity;
5. deliberately replace the exact synthetic reference only after review.

Current evidence during PR #233:

- the M7.6 144-run resource-variability experiment completed all runs under semantics v6 and produced a new deterministic reference candidate;
- the M8.6 terrain null-model benchmark completed all four paired ensembles and retained overall class `fragile_spatial_structure`, while `migrationTotalDistanceCells` moved from fragile to robust under the new demographic trajectory;
- the M9.7 controlled aggregation benchmark retained class `capability_distinguished`, passed all 8 paired seeds, exact duplicate replay and active-checkpoint/resume equivalence, while exact aggregate values changed;
- benchmark workflows are being hardened to upload generated evidence even when the preserved-reference comparison fails, so a mismatch does not discard the result needed for review.

These are synthetic capability/regression results only.

## 7. Model analysis

This repair does not satisfy the broader sensitivity programme. In particular, the following remain required before strong inferential use:

- founder/start-state sensitivity (#192, #219);
- demographic opportunity denominators (#228);
- full scientific sensitivity surface (#205);
- temporal-resolution/structural sensitivity if annual M2 is consequential to a claim;
- identifiability/equifinality (#217);
- long-run regime/path-dependence analysis (#220).

The M7.5 engineering performance acceptance is also being separated from demographic persistence: a valid synthetic population extinction is model behavior, not a performance defect. The performance gate instead requires a substantial minimum amount of simulated work plus throughput/wall-time/RSS limits.

## 8. Corroboration

None. No independent empirical archaeological/anthropological corroboration is claimed or attempted by this repair.

## Remaining M2 gates

The first transition-semantics PR must not be treated as completion of the whole M2 programme. Remaining work is:

1. #192 — provenance-bearing founder reproductive/genealogical prehistory or an explicitly justified alternative initialization contract;
2. #191/#228 — run-facing requested/effective spacing provenance plus mortality/fertility opportunity and denominator diagnostics;
3. rerun the relevant scientific regression/metamorphic suite over the combined repaired M2 programme;
4. after the known P1 repair backlog is completed, rerun at least two, preferably three, genuinely different adversarial scientific audits against the corrected model before foundational P1 discovery is considered converged again.

No post-fix convergence claim is made by this document.