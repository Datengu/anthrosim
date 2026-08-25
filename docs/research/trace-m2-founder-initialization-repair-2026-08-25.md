# TRACE record: M2 founder initialization repair

**Date:** 2026-08-25  
**Issue:** #192  
**Implementation PR:** #234  
**Scope:** founder reproductive/genealogical initialization only  
**Status:** implementation/conceptual verification evidence; not empirical validation

## 1. Defect being repaired

The pre-repair synthetic founder population initialized age, reproductive sex, households and residence, but every founder implicitly entered day 0 without pre-simulation reproductive history or direct-parent genealogy.

That zero-history state could create scientifically consequential start-date artifacts:

- birth spacing could treat founders as if they had never recently given birth;
- the M4 direct-parent kin proxy could be empty at the first migration boundaries regardless of the intended represented population;
- early outputs could therefore depend on the arbitrary simulation epoch rather than only on declared model assumptions.

This was classified as a P1 scientific-behaviour defect because it affects causal state read by M2 and M4 rather than merely reporting or user interface behaviour.

## 2. Conceptual repair

The repair does not attempt to disguise the synthetic initializer as realistic. Instead it separates two meanings:

- `synthetic_validation_v1` remains a frozen deterministic engine-validation/null-model founder generator;
- `declared_founder_state_v1` accepts exact versioned founder state including signed pre-run reproductive timing and explicit living direct-parent links.

The complete declared state is part of immutable `ExperimentConfig` identity.

The declared path is intentionally not called "stable", "empirical" or "research-ready". It can carry an empirically derived population, a schedule-generated population or a synthetic test population, but the scientific justification belongs to the process that produced the declaration.

Normative semantics: [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md).

## 3. TRACE element 2 — model description

The model-description change is explicit:

- founder birth chronology is signed relative to the run epoch;
- optional pre-run last-birth timing is initial-condition provenance, not a simulated birth event;
- optional founder direct-parent links are materialized into authoritative Population state;
- genealogy completeness is declared separately from the individual links;
- synthetic-only age/sex/household generator knobs do not affect declared state.

ODD/ODD+D must be updated so the previous statement that founder prehistory is necessarily absent is removed.

## 4. TRACE element 4 — conceptual model evaluation

### Alternatives considered

**A. Silently synthesize missing founder histories.** Rejected. This would replace an obvious zero-history artifact with an unobservable invented prehistory whose assumptions could be mistaken for evidence.

**B. Generate fictitious pre-run children so last-birth state has a record.** Rejected. AnthroSim would then assert people/events that were not supplied and contaminate runtime birth accounting.

**C. Add only pre-run birth timing but leave founder genealogy empty.** Rejected as incomplete because M4 reads direct-parent state causally.

**D. Require complete historical genealogy.** Rejected as unnecessarily strong for the current model, which only consumes living direct-parent locations as a narrow kin proxy.

**Selected design:** exact declared initial state with signed reproductive timing plus a scoped genealogy-completeness assertion, and fail-closed behaviour where the active model would otherwise interpret unknown kin state as absence.

### Key conceptual boundary

`complete_living_direct_parents` means complete only for the current M4 direct-parent proxy among people represented in the founder population. It does not assert complete ancestry or anthropological kinship knowledge.

## 5. TRACE element 5 — implementation verification

Verification added by PR #234 includes:

- founder-definition structural/chronological validation;
- exact declared-state materialization tests;
- tests that synthetic-only knobs cannot alter declared state;
- explicit no-fallback mode mismatch tests;
- first-boundary M2 birth-spacing tests using recent versus distant signed pre-run births;
- full-lifecycle tests showing pre-run history affects year-one fertility without creating runtime pre-run birth records;
- checkpoint/resume equivalence with founder history embedded in experiment identity;
- fail-closed tests for active M4 kin weighting with unspecified founder genealogy;
- a focused first-M4-boundary acceptance fixture in which kin is the only positive destination utility and the declared child household moves toward its declared living parent.

The ordinary repository-wide CI, determinism, bundle, spatial and preserved M8/M9 benchmark gates remain applicable because the existing synthetic path is intentionally unchanged unless the new mode is selected.

## 6. TRACE element 3 — data evaluation remains open

This repair creates a place to represent source-derived founder state, but it does not establish evidence closure for any real founder declaration.

For empirical use, the study must still evaluate:

- how founder ages and reproductive-sex state were inferred or sampled;
- how households/co-residence were derived;
- how initial locations and condition were derived;
- whether pre-run last-birth information is observed, model-derived or uncertain;
- whether living direct-parent completeness is genuinely defensible;
- how uncertainty and competing initial states are represented.

A `ParameterProvenance` enum value is metadata, not independent evidence that these questions have been answered.

## 7. TRACE element 7 — model analysis remains required

Removing a hidden zero-history assumption does not remove initialization sensitivity.

Question-specific analysis must still vary plausible founder states or use a documented generation/burn-in procedure and test whether conclusions depend on:

- age structure;
- sex structure;
- household composition;
- residence distribution;
- initial condition;
- reproductive-history distribution;
- available direct-parent links;
- the chosen simulation epoch/burn-in length where relevant.

If conclusions change materially across plausible initial states, that dependence is a scientific result rather than a reason to select the most convenient initialization.

## 8. Validation and corroboration status

No empirical demographic or archaeological validation is created by this repair.

The existing M8/M9 benchmark results remain capability/synthetic verification. No real archaeological application, calibration or held-out corroboration is added here.

## 9. Effect on the P1 backlog

If the implementation and regression gates pass, #192 can be closed as a repaired foundational causal defect because AnthroSim will no longer require relevant founder reproductive/kin state to be implicitly zero.

Closing #192 must **not** be interpreted as:

- founder initialization being empirically validated;
- a stable-population generator being complete;
- evidence closure being complete;
- initialization sensitivity being complete;
- AnthroSim being ready for a specific archaeological inference.

Those remain separate scientific-readiness tasks.

## 10. Post-repair review gate

Before #234 is merged:

1. all latest-head CI and determinism gates must pass;
2. the first-boundary kin fixture must execute successfully;
3. ODD/ODD+D and the living TRACE dossier must no longer describe #192's zero-history state as unavoidable current behaviour;
4. ordinary user-facing execution must have a documented path for supplying declared founder state, or the PR must explicitly state and track that interface limitation rather than implying the mode is usable externally;
5. the final diff should be reviewed specifically for silent fallback, invented history, experiment-identity drift and synthetic benchmark churn.

After merge, #192 should be treated as repaired only within the exact semantics documented in the normative founder-initialization contract.
