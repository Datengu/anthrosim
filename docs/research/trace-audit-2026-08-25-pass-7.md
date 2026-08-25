# TRACE adversarial scientific audit — pass 7

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Lens:** confirmatory-study integrity, stochastic precision and reproducible downstream inference  
**Overall result:** **CLEAN WITH RESPECT TO NEW P1 DISCOVERY**

## Purpose

This pass deliberately moved outside the model's causal equations and asked whether a scientifically correct/reproducible simulation could still support an invalid or irreproducible inferential claim because the research procedure around it was under-specified.

The pass examined:

- whether a confirmatory study protocol can be frozen before result inspection;
- whether treatment/manipulation realization is distinguishable from merely configuring a mechanism;
- whether stochastic replicate counts can be justified by Monte Carlo precision rather than convention;
- whether exact downstream analysis code/configuration can be traced from preserved run data to the reported result;
- whether the underlying named-RNG architecture itself showed a new scientific defect.

## New findings

### P2 — #230: bind a frozen study protocol to confirmatory experiments and results

AnthroSim's versioned M7 experiment definition already preserves the question, base settings, parameter dimensions and seeds, while `research-standards.md` and TRACE now require a fuller study-specific protocol for inferential research.

The missing bridge is a generic versioned protocol identity that freezes, before confirmatory analysis, the hypotheses/null models, primary and secondary observables, decision criteria, evidence roles, analysis windows, stop/exclusion rules, sensitivity/equifinality plan and intended analysis method.

The M9.7 benchmark demonstrates the stronger pattern manually: its acceptance contract was committed before result inspection and the result records that predeclared contract. #230 generalizes that pattern for future research studies.

### P2 — #231: require Monte Carlo precision / replicate-sufficiency evidence

A fixed and perfectly reproducible seed set does not establish that stochastic sampling error is small enough for the intended conclusion.

Research studies need a predeclared replicate-count or convergence rule appropriate to the estimand and should quantify Monte Carlo uncertainty separately from parameter/evidence/structural uncertainty.

### P2 — #232: preserve executable downstream-analysis provenance

AnthroSim strongly provenance-binds simulation execution, but sophisticated statistical analysis is intentionally downstream in Python/R or other research tooling.

For a canonical inferential result, the exact analysis implementation/configuration/environment and output artifacts therefore need their own provenance lineage. Reproducing the raw simulation is not automatically the same as reproducing the scientific inference derived from it.

## Important non-findings

### Named RNG architecture held up under this lens

`RngFactory` deterministically derives stable named ChaCha streams from the experiment seed, and checkpoint continuation preserves stream positions. This pass found no new defect in marginal random-number generation or deterministic replay.

The previously identified paired-seed interpretation issue remains #214: state-dependent sequential consumption means two treatment arms do not retain agent/event-level common random shocks after their states diverge.

### Out-of-window M9 triggers are not automatically invalid

The M9.7 control deliberately places a trigger outside the observation window to preserve the same focal-region/travel definition while producing zero temporary journeys. That is a legitimate declared control design.

The research requirement is therefore not to reject all out-of-window triggers, but to predeclare expected manipulation realization and verify it in the study protocol/result when positive treatment exposure is required.

### Research archive integrity remains a strength

The SHA-256 research-integrity layer can freeze an assembled publication/research archive exactly. #232 is narrower: it asks that the execution lineage among input data, analysis code/configuration and reported result also be preserved.

## TRACE interpretation

This pass primarily strengthens TRACE elements:

- **1 — problem formulation:** hypotheses, outcomes and decision criteria must be frozen for confirmatory inference;
- **3 — data evaluation:** evidence roles belong in the study protocol;
- **7 — model analysis:** stochastic precision and planned sensitivity/equifinality analysis need explicit rules;
- **8 — corroboration:** held-out evidence and downstream inference must remain provenance-distinguishable.

## Pass result

This pass found:

- **0 new P1 scientific-behaviour defects**;
- **3 new P2 research-integrity/readiness gates:** #230, #231 and #232.

It is therefore a **clean P1-discovery pass**.

This does not mean AnthroSim is research-ready. The existing P1 backlog remains unresolved, and the stronger post-repair TRACE verification criterion still requires renewed adversarial auditing after those fixes.