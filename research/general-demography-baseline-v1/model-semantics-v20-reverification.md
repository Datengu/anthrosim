# Model-semantics v20 reverification

Audit-v2 issue #350 re-executed the frozen issue #304 384-run confirmatory design on branch head `96e50e9114e05543ae1fc6fd94a095867e54385a`, which contains live `anthrosim-model-semantics-v20` plus only this reverification trigger. All 384 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`.

The high-level recommendation remains **`no_universal_demographic_baseline`**, but quantitative evidence changed under the #326 M3 partial-supply fixed-point repair. Representative changes versus the pre-v20 reference are:

- positive-growth / fixed-founder mean terminal population: `108.25` -> `106.5`;
- positive-growth / fixed-founder mean late growth rate/year: `-1.865117e-05` -> `-4.466801e-04`;
- positive-growth fission-minus-fixed terminal-population effect: `-79.359375` -> `-77.859375`;
- long-run `multipleStableRegimesDetected`: `true` -> `false`;
- primary long-run classifications: `drifting=369, insufficient_data=10, stable=5` -> `drifting=374, insufficient_data=9, stable=1`.

This is a causal v20 scientific-reference rebaseline, not an analysis-method compatibility update. The canonical result is copied byte-for-byte in semantic content from the branch-specific workflow artifact `issue-304-demographic-baseline-confirmatory` produced by run `33264782340`; only the execution-specific `researchId` naturally identifies that new v20 reproduction.
