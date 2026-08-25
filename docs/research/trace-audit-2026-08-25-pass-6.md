# TRACE adversarial scientific audit — pass 6

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Lens:** limiting cases, null interventions, parameter directionality and metamorphic/equivalent-state semantics  
**Overall result:** **CLEAN WITH RESPECT TO NEW P1 DISCOVERY**

## Purpose

This pass deliberately avoided the demographic-opportunity and output-label findings from the preceding audits. It tested the model through limiting/null cases and metamorphic reasoning:

- what happens when a weight or effect is zero?
- do higher/lower parameters move intermediate quantities in the declared direction?
- do zero-denominator outputs become explicitly undefined rather than invented values?
- do equivalent/canonical encodings resolve to equivalent model meaning?
- do derived observables clearly distinguish reconstruction from authoritative state?

The objective was not to prove validity but to search for sign errors, hidden residual effects and false equivalence.

## Areas challenged

### M3 resources / condition

Reviewed:

- productivity-scale and capacity directionality;
- annual regeneration, environmental-stress and seasonality scaling;
- zero/maximum condition-loss and recovery semantics;
- scarcity mortality probability direction;
- resource accounting conservation;
- resource-period boundary uniqueness up to the validated maximum of 365 periods/year.

Known issues such as #180, #189, #199, #200 and #204 remain blockers, but this pass did not identify a distinct new P1 beyond them.

### M4 migration

Reviewed:

- zeroed utility weights;
- separation of travel-cost weight, relocation risk and realized condition cost;
- bounded candidate generation;
- utility sign/direction;
- origin-versus-candidate semantics;
- simultaneous-move snapshot behavior.

Existing issues #186/#195/#214/#225 cover the relevant known problems. No new P1 emerged.

### M8 spatial transformations

Reviewed:

- direct/inverse linear mapping monotonicity;
- target-domain validation;
- nodata policy constraints;
- canonical provenance identity;
- transformed-world validation against core invariants.

The transformation arithmetic held under this static limiting-case audit. Existing scale/provenance issues remain #181/#185/#203/#211/#212/#224.

### M9 temporary mobility / travel

Reviewed:

- focal-region member canonicalization;
- travel capacity monotonicity;
- traversability threshold behavior;
- minimum-cost destination semantics;
- route-distance reconstruction;
- zero observation duration;
- temporary resource half-open duration accounting.

A suspected route-distance problem was rejected. Execution does not persist an authoritative path, and the observability contract explicitly defines `route_distance_edges` as a **derived minimum edge count among routes with the authoritative minimum cost and destination**. The derivation verifies cost/destination against the frozen travel table and fails closed if they differ.

Zero-duration mean visitor presence correctly becomes `None`, providing a useful contrast with the empty-set-zero defect already tracked by #222.

## Important non-findings

- Focal-region cell input order is canonicalized before identity/behavior.
- M8 direct/inverse transformations are monotone under their declared direction.
- M9 route-distance derivation does not claim an executed route that the model never stored.
- M9 temporary-resource presence intervals reconcile exactly and reject zero-length settlement periods.
- Resource scheduling validation prevents more than 365 periods/year, so integer day scheduling does not create duplicate subannual boundaries through an excessive period count.
- No additional zero-denominator sentinel equivalent to #222 was identified in the inspected M8/M9 ratio outputs; those paths use optional/undefined values.

## Pass result

**No new P1 scientific-behaviour issue was identified in this pass.**

No new issue was filed from the pass because the remaining candidates were either:

- already covered by existing issues;
- explicitly declared model semantics;
- correctly represented as derived rather than authoritative quantities; or
- too weak to justify a research-readiness ticket.

This is the **first clean P1-discovery pass** in the current audit sequence.

## What this does and does not mean

This is positive evidence that the obvious limiting-case/sign/metamorphic defect surface is shrinking. It is **not** evidence that AnthroSim is research-ready:

- many previously discovered P1s remain unresolved;
- no local numerical ensemble was executed as part of this static pass;
- final convergence should be reassessed after the P1 repairs because fixes can introduce or reveal new interactions.

The practical convergence counter is therefore:

```text
clean independent audit passes on current code: 1
required before confidence in discovery convergence: at least 2, preferably 3
known P1 backlog resolved: no
```

A second clean pass should use a different lens again rather than repeating these limiting cases.