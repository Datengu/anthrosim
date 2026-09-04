# Model-semantics v33 re-verification

Audit-v4 AV4-009 changes M4 spatial-candidate stochastic coupling while preserving the declared demographic controls. The first v33 abstention candidate was rejected because it collapsed all six confirmatory arms to extinction. The accepted equivalence-class sampler was therefore re-run independently before any reference update.

- reviewed production head: `25c9a11dce8052fecfbb339114a4ba1c8da00b0c`
- issue #304 review: run `33884799723`, job `101061843958`, artifact `9941547662`, artifact SHA-256 `dc439b48bb7d2a048c3fe2365698d2403a67ac2a26e340c17bb6dc8a8901fa83`
- all 780 confirmatory runs completed
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`
- recommendation remains `no_universal_demographic_baseline`
- long-run environment and initialization dependence remain false; stochastic multi-regime context count remains 2

The numerical shifts are consistent with an M4 residence-coupling change rather than a demographic-mechanism rewrite. In particular, positive-growth/fission late growth remains about -1.07%/year while terminal population and extinction move from the v32 realization; positive-growth/fixed remains non-extinct. Historical v32 and earlier outputs remain evidence for their original semantics and are not relabelled.

M8.6 was also reviewed on run `33884800100`, job `101062379280`, artifact `9941586776` (SHA-256 `21b2f9dacdbf85c5b036bae7ca90d158cad5257a67cca1dae66bc17e54e9f9ba`). All 32 runs completed with no degenerate arms. The benchmark remains `fragile_spatial_structure`; `terminalLargestCellSharePermille` remains fragile, while `terminalPopulationHerfindahlPerMillion` changes from fragile to not-distinctive under v33. M9.7 on the same workflow preserved its canonical scientific reference and exact replay/checkpoint-resume contracts.
