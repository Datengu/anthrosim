# Audit v3 Area I closure note — sensitivity, uncertainty, convergence, robustness

Immutable discovery target: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / `anthrosim-model-semantics-v21`.

This note closes the remaining Area I coverage after AV3-007/#413 and AV3-008/#415 were already demonstrated. It records the additional horizon, initialization, replicate and numerical/discretization checks required by the audit protocol. It is discovery evidence only; no finding is repaired.

## Research-space exposure and interaction coverage

The frozen `ResearchExperimentDefinition` starts from one complete authoritative `ExperimentConfig` and optional complete spatial configuration. Numeric/structural dimensions are typed JSON-pointer replacements rooted at `/experiment` or `/spatial`; only seed, schema-version fields and the spatial semantics identity are reserved. `ExperimentConfig.durationYears`, founder/initialization configuration, resource initialization and spatial mechanism fields are therefore part of the declared research-space surface rather than hidden source defaults.

Fresh Area-I adversaries nevertheless demonstrated two material failures in this machinery:

- AV3-007/#413: support-scale robustness can be certified with fabricated/nonexistent analysis identities.
- AV3-008/#415: a child-field dimension followed by an ancestor-object dimension can nominally record four factorial coordinates while realizing only two executable treatments. This directly attacks interaction/global-design adequacy.

AV3-011/#419 is cross-cutting: a claimed parameter held at one level can be falsely certified as identified.

## Horizon sensitivity

`durationYears` is an ordinary numeric field inside the complete authoritative experiment configuration and is not among the reserved research-dimension paths. Horizon can therefore be varied as an explicit scientific coordinate through the same typed expansion surface. The research-definition contract explicitly states that successful orchestration does not itself establish adequate horizon choice; horizon robustness remains a study-level obligation.

No additional generic horizon-specific implementation defect was demonstrated beyond the known treatment-overwrite/design-adequacy findings above.

## Initialization sensitivity

Fresh Area G evidence supplies a direct limiting-case initialization-sensitivity attack on the same immutable baseline: founder-condition arms 400 and 900 permille were evolved for five years with mortality, fertility, migration, resource need, condition loss and recovery disabled. They remained exactly 400 and 900, and independent year-2 checkpoint/resume reproduced uninterrupted complete `RecordedRun` output for both arms after excluding operational resume-lineage metadata. Thus elapsed model time did not silently erase the causal initialization contrast in this limiting case.

Known initialization limitations remain open as AV3-001/#387, AV3-002/#392 and AV3-003/#396 and are not repaired here.

## Replicate sensitivity

The research definition requires an ordered duplicate-free seed list and expands every scientific point across those seeds. Fresh Area H then attacked the precision consequence directly. AV3-006/#410 demonstrated that for a 20-seed two-arm comparison the implemented independent-groups half-width was 3.666756860283 while the covariance-aware half-width for the actually accepted same-seed design was 5.185577281736; at a predeclared threshold of 4.5 the gate falsely returned `sufficient_stop`.

Therefore replicate count is explicitly representable and provenance-bound, but replicate sufficiency is not trustworthy for the affected estimator until AV3-006 is repaired. This is recorded as an open cross-cutting limitation rather than treated as successful convergence.

## Numerical / spatial-resolution convergence

Frozen v21 explicitly declares its spatial semantics resolution-dependent rather than numerically convergent under raster refinement. Independent inspection of the frozen `spatial_resolution_dependence` tests gives hand-computable refinement effects for equal physical domains:

- halving cell size from 100 m to 50 m multiplies equal-area per-cell-total M3 resource stock by 4;
- the same 200 m M9 route changes from 2 grid edges / cost 2,000 / 1 outbound day to 4 edges / cost 4,000 / 2 outbound days;
- a fixed M4 radius of 3 cells changes physical horizon from 300 m to 150 m;
- a 50 m-separated M2 pair can share one 100 m cell but occupy different 50 m cells.

These are explicit model-scale semantics, not a hidden numerical approximation. A second halving to 25 m would analytically continue the equal-area cell-count scaling to 16 times the 100 m per-cell-total resource stock and four times the 100 m grid-edge route cost for the same 200 m distance if all other cell-space assumptions are held fixed. The correct scientific requirement is therefore declared resolution sensitivity, not convergence to a resolution-independent limit.

## Area-I disposition

Area I has now received fresh adversarial coverage of parameter/support sensitivity, interaction/global-design integrity, horizon exposure, initialization sensitivity, replicate sensitivity and spatial/numerical refinement semantics. Structural sensitivity is covered by the typed structural-dimension machinery and AV3-008/AV3-011 limitations; support-scale sensitivity is covered by AV3-007.

No new finding is created by this closure note. Existing Area-I/cross-cutting findings remain deliberately unrepaired:

- P2 AV3-005/#402;
- P2 AV3-007/#413;
- P1 AV3-008/#415;
- P1 AV3-011/#419;
- P1 AV3-006/#410 as the replicate-precision limitation.

Area I is complete with findings open. Next non-overlapping pending area is K — experiment orchestration, configuration, provenance and reproducibility.
