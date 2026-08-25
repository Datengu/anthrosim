# TRACE adversarial scientific audit — pass 5

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Lens:** causal opportunity structure, demographic interval semantics and reproductive opportunity observability  
**Overall result:** **NOT A CLEAN P1-CONVERGENCE PASS**

## Purpose

This pass asked whether AnthroSim's emergent population outcomes can change because the model silently changes who receives an opportunity to reproduce or interact, rather than because the nominal scientific parameter being studied changed.

It focused especially on M2's annual demographic opportunity structure and its interaction with persistent residence, household composition and spatial representation.

## New findings

### P1 — #227: define within-year mortality/fertility event semantics

M2 processes annual mortality first and offers fertility only to surviving females; eligible male parents likewise must survive the mortality pass.

If mortality and fertility probabilities both represent opportunities/hazards over the same elapsed year, this makes death erase any possible birth/parentage event during that entire interval. Ignoring other filters, a configured fertility probability `p` becomes an effective boundary birth probability `(1-q) × p` when annual mortality probability is `q`.

That may be defensible only if fertility is explicitly defined as conditional on survival through the interval and empirical rates are transformed accordingly. Current research documentation does not establish that interpretation.

This is distinct from #179's wrong age interval, #191's spacing quantization, #193's same-instant migration/locality effect and #208's M3-vs-M2 competing mortality attribution. Together they strongly indicate that M2 needs one coherent interval/event-time redesign rather than isolated patches.

**Issue:** #227.

### P2 — #228: expose M2 fertility opportunity denominators/rejection pathways

A realized birth requires passing multiple filters: survival, reproductive sex, age schedule, spacing, same-cell eligible male availability, the fertility draw and the operational record ceiling.

Successful births are preserved, but failed opportunities are not exposed in the default analysis surface. Low realized fertility can therefore reflect the nominal fertility schedule, local male scarcity/spatial fragmentation, spacing, mortality timing or stochastic failure, and a preserved run cannot diagnose that directly from total births.

A versioned demographic-validation report should expose opportunity denominators and the demographic outputs already promised by `demography-v0.1.md`.

**Issue:** #228.

## Existing issue extended

### #203 also affects M2 reproductive locality

M2 requires an eligible male to share the female parent's exact persistent-residence cell. Raster/cell partition therefore defines a reproductive-contact boundary as well as resource and mobility scale.

A comment was added to #203 requiring its spatial-resolution work to include reproductive-contact semantics or explicitly decouple mating/contact neighbourhood from raw cell identity. This was not filed as a duplicate P1.

## Important non-findings

### Same-cell parent eligibility is documented

The model specification explicitly states that M2 parent selection is a minimal same-persistent-cell rule and excludes visitor co-presence, marriage, pair bonds, mate preference, incest avoidance and related institutions. The audit therefore did not label the rule itself an implementation bug.

The research problem is whether conclusions depend on that structural rule and whether the resulting opportunity suppression is observable/sensitivity-tested.

### Fixed household topology remains owned by #207

The audit reconfirmed that founder household size/composition can influence local male availability and long-run opportunity structure, but the broader frozen household lifecycle is already tracked by #207. No duplicate household-lifecycle issue was created.

## Pass result

This pass found:

- **1 new P1:** #227;
- **1 new P2:** #228;
- one material extension to existing P1 #203.

Because #227 is new, this pass is not a clean P1-convergence pass.

## Architectural implication

The demographic blocker cluster should now be treated as one redesign programme:

- #179 age/risk interval;
- #191 executable birth spacing;
- #192 founder demographic history;
- #193 migration/parentage boundary locality;
- #201 newborn condition initialization;
- #227 within-year mortality/fertility competition;
- #228 opportunity/validation observability.

Fixing these independently risks creating a demographic process whose individual patches are locally correct but whose combined interval semantics remain incoherent.