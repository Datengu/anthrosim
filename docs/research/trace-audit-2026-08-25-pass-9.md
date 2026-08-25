# TRACE adversarial scientific audit — pass 9

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Lens:** symmetry, relabelling invariance and arbitrary-bookkeeping causality  
**Overall result:** **CLEAN WITH RESPECT TO NEW P1 DISCOVERY**

## Purpose

This pass tested whether scientifically equivalent states can produce systematically different causal outcomes solely because of arbitrary identifiers, record order, storage order, compass orientation or deterministic tie handling.

The guiding principle was:

> Stable bookkeeping may make execution reproducible, but bookkeeping identity must not become an undeclared scientific trait.

## Systematic symmetry paths reviewed

### M3 scarce-resource remainder allocation

The persistent lower-household/claim-order priority is already captured by **#182**. Repeated equal claims can privilege the same lower-ID household through indivisible-unit remainder assignment.

No distinct second M3 ID-priority mechanism was identified.

### M9 duration-weighted home/visitor rounding

The exact-tie home-side preference is already captured by **#194**. This is a semantic-side bias rather than a household-ID bias and remains a P2 structural/rounding gate.

### M9 equal-cost focal-region destinations

The lower-CellId destination tie rule is already **#190**. Because destination choice affects visitor presence and resource demand, that tie can become causal and must be resolved/sensitivity-tested.

### M4 kin proxy

Sex-role and person-record-order asymmetry in the bounded kin-location set is already **#188**. This is the major existing M4 relabelling failure.

### M4 stochastic candidate evaluation

Candidate uncertainty draws and household decisions use sequential named RNG streams. Reordering/equivalent treatment arms can therefore attach different latent draws to later candidates/households once states diverge.

This does not create a fixed marginal directional preference in ordinary ensemble behavior; it is the already-recorded paired/common-random-number interpretation problem in **#214**.

### M4 explanatory `best_candidate` tie

When candidate utilities tie exactly, the trace's explanatory `best_candidate` uses lower `CellId`. The actual selected destination, however, is drawn from eligible weighted alternatives rather than forced to that `best_candidate`.

Because this lower-ID tie affects explanatory metadata rather than the causal selected move, the audit did **not** promote it into a new scientific-behaviour issue. If future consumers start treating `best_candidate` as a causal output, its semantics should be revisited.

## Compass/orientation checks

Directional migration counters (north/east/south/west) are descriptive projections of authoritative origin/destination coordinates. They do not feed back into movement decisions.

Known physical-orientation/georeferencing risks remain under #185 and spatial scale/boundary issues #203/#211. This pass did not identify a new fixed north/west behavioral preference beyond already-known CellId/tie effects.

## Important non-findings

- M4 simultaneous planned moves avoid a first-mover household-ID state-update advantage at the same decision boundary.
- M4 weighted candidate selection does not deterministically choose the lower-ID `best_candidate`.
- No new stable person-ID mortality/fertility priority was identified beyond the already-known demographic opportunity/timing semantics.
- The major deterministic rounding/tie asymmetries are already represented by #182, #188, #190 and #194 rather than being undiscovered duplicates.

## TRACE interpretation

This pass primarily strengthens:

- **4 — conceptual model evaluation:** identify which tie/rounding rules are scientifically meaningful assumptions;
- **5 — implementation verification:** require relabelling/permutation invariance where identifiers should be scientifically irrelevant;
- **7 — model analysis:** sensitivity-test unavoidable discrete/tie semantics when they can affect inference.

## Pass result

**No new P1 scientific-behaviour issue was identified.**

All material systematic symmetry failures found by this lens map to existing issues. The remaining new candidates were either explanatory-only, marginally stochastic rather than directionally biased, or too weak to justify another research-readiness ticket.

This is a third consecutive independent clean P1-discovery pass after the last new-P1 pass.

## Discovery-convergence decision

Taken together with passes 7 and 8, the current audit sequence now has at least three genuinely different passes with no new P1 discovery:

1. confirmatory-study / stochastic-inference integrity;
2. cross-mechanism writer→reader causal graph;
3. symmetry / relabelling invariance.

That is sufficient to end the **audit-first discovery phase on the current code** and move to repair of the known P1 causal clusters.

It is **not** final TRACE verification. Existing P1s remain open. After those repairs, AnthroSim must repeat appropriate adversarial/metamorphic audits and again achieve clean post-fix passes before foundational scientific verification can be considered converged.