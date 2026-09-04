# Model-semantics v33 re-verification

Audit-v4 AV4-009 changes M4 spatial-candidate stochastic coupling while preserving the declared demographic controls. The first v33 abstention candidate was rejected because it collapsed all six confirmatory arms to extinction. The accepted equivalence-class sampler was therefore re-run independently before any reference update.

## #304 demographic baseline

- reviewed production head: `25c9a11dce8052fecfbb339114a4ba1c8da00b0c`
- issue #304 review: run `33884799723`, job `101061843958`, artifact `9941547662`, artifact SHA-256 `dc439b48bb7d2a048c3fe2365698d2403a67ac2a26e340c17bb6dc8a8901fa83`
- all 780 confirmatory runs completed
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`
- recommendation remains `no_universal_demographic_baseline`
- long-run environment and initialization dependence remain false; stochastic multi-regime context count remains 2

The numerical shifts are consistent with an M4 residence-coupling change rather than a demographic-mechanism rewrite. In particular, positive-growth/fission late growth remains about -1.07%/year while terminal population and extinction move from the v32 realization; positive-growth/fixed remains non-extinct. Historical v32 and earlier outputs remain evidence for their original semantics and are not relabelled.

After synchronization, exact-head run `33885763038`, job `101065007590`, on `c9b7a0f2d762323afa76b7d0f390f29930a77b0a` reran all 780 runs and passed the preserved scientific conclusion. Its artifact is `9941945506` with SHA-256 `d33cfb889d514087ddc9e70e8c67d6ca6abdb759494279cb79f02060c12718fc`.

## M8.6 and M9.7

M8.6 was reviewed on run `33884800100`, job `101062379280`, artifact `9941586776` (SHA-256 `21b2f9dacdbf85c5b036bae7ca90d158cad5257a67cca1dae66bc17e54e9f9ba`). All 32 runs completed with no degenerate arms. The benchmark remains `fragile_spatial_structure`; `terminalLargestCellSharePermille` remains fragile, while `terminalPopulationHerfindahlPerMillion` changes from fragile to not-distinctive under v33. M9.7 on the same workflow preserved its canonical scientific reference and exact replay/checkpoint-resume contracts.

After synchronization, exact-head applicable-gates run `33885763333` passed both scientific gates: M8.6 job `101065317830` accepted the v33 canonical reference and M9.7 job `101065317723` preserved its existing canonical reference and replay/resume contracts.

## M7.6 resource-variability reference

The first clean post-synchronization matrix exposed the remaining affected frozen surface rather than being treated as an automatic rebaseline. On exact head `c9b7a0f2d762323afa76b7d0f390f29930a77b0a`, central CI run `33885763026`, M7.6 job `101069178448`, produced artifact `9942318197` (`m7-6-resource-variability-derived`, SHA-256 `4f5abe584d9c30f5ba69e144d7c41ad3d5e9d4a9664953944685d43066166cd5`). All 144/144 runs completed and were scientifically eligible with no operational censoring.

The causal control split is diagnostic for AV4-009: all 9/9 migration-disabled point summaries are numerically unchanged from the v32 reference, while all 9/9 migration-enabled point summaries change. Migration-enabled terminal-population means move by approximately -2.9% to +4.1%; the source definition, paired seeds, resource factors, completion rules and endpoints are unchanged. The substantive synthetic result is preserved: all three low-productivity no-migration points are extinct in 8/8 seeds while their migration-enabled counterparts persist in 8/8, and migration-enabled arms retain higher terminal population with lower condition-mediated mortality and lower unmet resource need at every matched productivity/seasonality point. This is the expected scientific footprint of changing M4 migration candidate coupling rather than unexplained resource-process drift, so the M7.6 machine reference is deliberately rebound to this reviewed v33 artifact.

A new exact-head full protected/scientific matrix is required after these reference and living-documentation updates; the production PR remains unmergeable by protocol until that new head is green.
