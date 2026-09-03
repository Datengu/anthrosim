# Model-semantics v28 reverification

Audit-v4 AV4-003 / issue #491 changes the causal same-seed assignment of M4 migration RNG draws by replacing arbitrary `HouseholdId` decision order with a schedule derived from persistent person stochastic-coupling identities. Within-household candidate enumeration remains unchanged for AV4-009/#518, and all households still observe one shared pre-move snapshot before simultaneous move application. Because AV4-003 changes deterministic continuation semantics, the current model advances from `anthrosim-model-semantics-v27` to `anthrosim-model-semantics-v28` and the frozen issue #304 demographic baseline must be scientifically re-executed rather than relabelled.

Issue-304 workflow run `33797427793`, job `100788374930`, re-executed the unchanged confirmatory design under v28. Artifact `9909929317` (`issue-304-demographic-baseline-confirmatory`, SHA-256 `f2c1929fd558eab3d6a003f2d1ff447f42c81b0585f81f315cca8b1f9430470c`) contains the exact workflow-generated `expected-result.json` copied byte-for-byte into the canonical `confirmatory-result.json`.

Reverification outcome:

- all **780/780** declared runs completed;
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`;
- the recommendation remains **`no_universal_demographic_baseline`**;
- long-run analysis still rejects a universal stable regime (`researchGateStatus = failed`) and still detects multiple stable regimes;
- environment dependence and initialization dependence remain undetected in this design;
- for every demographic schedule, dependency-aware fission still lowers terminal population and late realized growth and increases mate limitation relative to the fixed-founder control.

Representative v28 values are expected to differ from v27 because permanent-migration draws are now coupled to scientifically invariant household composition rather than arbitrary canonical household labels:

- negative-growth fission-minus-fixed mean terminal population: `-11.5846` people;
- positive-growth fission-minus-fixed mean terminal population: `-83.9769` people;
- replacement fission-minus-fixed mean terminal population: `-37.2231` people;
- positive-growth fixed-founder mean late growth rate/year: `-0.00106635`;
- primary long-run classifications: `drifting=744, insufficient_data=32, stable=4`;
- stochastic multi-regime treatment contexts: `1` while `multipleStableRegimesDetected` remains `true`.

This is therefore a **causal v28 scientific-reference rebaseline**, not a silent rewrite of v27 evidence. The immutable Audit-v4 discovery target `v0.3.4` / `anthrosim-model-semantics-v25` and reviewed v26/v27 references retain their original provenance. The v28 canonical result is copied byte-for-byte from the exact workflow artifact after validating its semantics ID, run count, recommendation, precision decisions, long-run conclusion and paired-effect directions.
