# TRACE scientific audit — adversarial pass 2 (2026-08-25)

**Status:** completed static scientific audit pass  
**Scientific status:** AnthroSim remains exploratory / not empirically research-ready  
**Scope:** current `main` source and research documentation after formal ODD 2020 / ODD+D / TRACE adoption  
**Execution note:** this pass was a static source/document audit. It did not execute new numerical ensembles locally and therefore does not claim empirical validation.

## Purpose

This pass deliberately used a different failure-mode lens from the first TRACE audit. It targeted scientific errors that can survive correct local equations and exact deterministic replay:

- finite-domain / boundary-condition dependence;
- environment-versus-process seed confounding;
- spatial initialization and burn-in/path dependence;
- paired-seed counterfactual semantics;
- observability needed for causal diagnosis and identifiability;
- resource-state initialization;
- parameter identifiability/equifinality;
- study analysis-window / long-run regime semantics;
- spatial/temporal support of reported observables.

Known issues were checked first to avoid re-filing existing defects under new wording.

## Result summary

The pass found **three new P1 research blockers** and a set of **P2 study-analysis / observability gates**.

### New P1 findings

- **#211 — finite study-area boundary effects**: an analyst-defined raster edge is currently a hard behavioral wall for M4 candidate discovery and M9 routing. Study extent can therefore change outcomes inside the target area even when the internal landscape is unchanged.
- **#212 — environment identity is confounded with stochastic replicate seed**: a spatial run still generates a synthetic world from the master seed before M8 overlays. M8 does not replace all causal environmental fields; notably seed-generated season amplitude/phase remain active in M3 when seasonality is enabled. Different replicate seeds can therefore mean different environments as well as different stochastic histories.
- **#213 — founder spatial initialization remains synthetic/uniform**: evidence-grounded spatial runs still use `SyntheticValidationV1` founder placement, drawing household residence uniformly over all cells. Initial settlement pattern can therefore become a strong unvalidated source of path dependence.

These findings mean the P1-convergence counter remains **0 clean passes**.

### New P2 research/readiness findings

- **#214 — paired-seed counterfactual semantics**: sequential, state-dependent RNG consumption means same seed does not preserve agent/event-level random shocks after two treatment arms diverge.
- **#215 — resource/condition temporal observability**: preserved artifacts do not currently retain enough time-resolved per-cell resource/condition history for strong causal diagnosis of different scarcity trajectories.
- **#216 — initial resource stock / resource burn-in**: the default M3 baseline begins cells at full stock capacity; research use needs explicit initial stock state or convergence/burn-in evidence.
- **#217 — parameter identifiability and equifinality**: research calibration must test whether observations actually constrain individual parameters/mechanisms rather than only ratios, correlated combinations or multiple structural explanations.
- **#218 — death physical-presence context under M9**: death events/spatial death counts are residence-attributed even when the household is visiting or in transit; physical-presence category is needed before spatial mortality interpretation.
- **#219 — study analysis windows / burn-in exclusion**: if startup transients are excluded, the analysis interval and burn-in rule must be predeclared/provenance-preserved rather than selected post hoc.
- **#220 — long-run regime semantics**: long simulation duration does not by itself establish equilibrium/stationarity; long-run claims require convergence/path-dependence diagnostics appropriate to the question.
- **#221 — spatial/temporal support of reported observables**: empirical comparison must declare the spatial and temporal aggregation scale at which model output and archaeological evidence are compared.

## Evidence behind the P1 findings

### 1. Finite-domain dependence (#211)

`World::neighbours4(...)` stops at the grid edge. M9 routing uses this finite graph. M4 candidate discovery also accepts only valid world coordinates. Therefore a crop edge acts as an absolute barrier even when it represents only the analyst's GIS extent.

A research study must distinguish:

- genuine physical/social barriers;
- simulation-domain extent;
- inner analysis/focal region;
- boundary buffers or open/external conditions.

A spatial-extent convergence test should embed the same inner landscape in increasingly large surrounding domains and verify whether inner outcomes stabilize.

### 2. Environment/process seed confounding (#212)

Spatial execution reconstructs the world by first generating the synthetic `World` from `config.seed`, then replacing only configured M8 target fields. Current M8 targets are movement cost, water access and base productivity.

Seed-generated `season_amplitude` and `season_phase_days` remain in the world and are consumed by M3 regeneration when seasonality is enabled. Thus an ensemble over seeds can mix:

- process stochasticity;
- population initialization variation;
- residual synthetic environmental variation.

Research experiments need explicit uncertainty dimensions rather than one master seed silently meaning all three.

### 3. Spatial initialization (#213)

The only executable population initialization mode remains `SyntheticValidationV1`. Founder household locations are sampled uniformly from world cells and are not conditioned on the transformed landscape or on evidence-derived starting settlement state.

Because M2 locality, M3 resource competition, M4 migration and M9 catchment all depend on residence, the initialization can affect later outcomes through path dependence. A real-landscape result therefore needs either an explicit starting spatial state or demonstrated burn-in/initial-condition convergence.

## Important non-findings / clarifications

### Named RNG streams remain a strength

The pass did **not** find evidence that the marginal RNG distributions are biased or that deterministic replay is broken. #214 is an interpretation/design issue: current streams are not agent/event-keyed common random numbers after treatment divergence.

### Equifinality is not itself a model defect

#217 is intentionally a research-analysis gate, not a claim that AnthroSim should have one uniquely identifiable parameter set. Discovering that multiple hypotheses/parameter regions fit the evidence may itself be a scientifically valuable result.

### Residence-based M2 under M9 is documented

M9 explicitly declares residence-based parentage/locality semantics rather than visitor-presence mating semantics. This pass therefore did not file that as an implementation error. The narrower #218 concerns preserving physical-presence context for mortality interpretation.

### Event/observability architecture remains strong overall

Demographic, migration and temporary-mobility events are strongly provenance-bound and replayable. #215 is specific to the comparatively sparse time-resolved observability of M3 resource/condition history.

## TRACE interpretation

This pass primarily adds evidence to TRACE elements:

- **4 — conceptual model evaluation:** finite boundaries, initial-state semantics, paired counterfactual meaning;
- **5 — implementation verification:** whether an analyst-controlled representation choice alters causal execution;
- **6 — model-output verification:** whether resource/mortality trajectories can be observed at the level needed for validation;
- **7 — model analysis:** initialization sensitivity, extent convergence, identifiability/equifinality, long-run regime analysis;
- **8 — corroboration preparation:** matching model/evidence support scales and avoiding residence/physical-presence conflation.

## Recommended repair clustering after this pass

### A. Time / demographic / condition semantics

#179, #180, #189, #191, #192, #199, #200, #201, #204, #208.

### B. Physical spatial semantics

#185, #187, #196, #203, **#211**, **#212**, **#213**.

### C. M4 decision semantics

#182, #186, #188, #195.

### D. Research experiment and analysis surface

#183, #184, #205, #214, #215, #217, #219, #220, #221.

### E. Evidence / validation governance

#181, #206, #209.

### F. Structural sensitivity and initialization

#207, #216 and the initialization parts of #192/#213.

### G. M9-specific interpretive/observability gates

#197, #218 plus the relevant parts of #196/#209.

## Convergence decision

This pass **cannot count as a clean convergence pass** because it found three P1s.

However, the severity pattern changed during the audit: once the three P1 representation/initialization problems were identified, later findings were progressively study-procedure/observability P2s rather than new fundamental causal implementation errors.

That is weak evidence that the remaining undiscovered P1 surface may be shrinking, but it is not enough to declare convergence.

The next genuinely independent audit should therefore avoid revisiting the same time/space/initialization mechanisms and instead attack another high-level failure family, such as:

- population-level conservation/emergence and demographic regime behavior;
- structural intervention semantics / causal ablation;
- symmetry/metamorphic invariance under relabeling and equivalent representations;
- multi-pattern validation logic and false-positive pattern matching.

Foundational scientific audit should still stop only after the post-fix model achieves the TRACE convergence rule: resolve blocking P1s, then obtain at least two (preferably three) genuinely different serious passes with no new P1 findings.
