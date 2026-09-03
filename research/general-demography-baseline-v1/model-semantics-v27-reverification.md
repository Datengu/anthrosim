# Model-semantics v27 reverification

Audit-v4 AV4-002 / issue #488 changes the causal same-seed assignment of background-demographic mortality RNG draws by replacing incidental `PersonId` record ordering with the persisted scientific stochastic-coupling rank introduced by AV4-001. Condition-mediated mortality remains on its pre-existing ordering for its separate AV4-006/#497 finding. Because AV4-002 changes deterministic continuation semantics, the current model advances from `anthrosim-model-semantics-v26` to `anthrosim-model-semantics-v27` and the frozen issue #304 demographic baseline must be scientifically re-executed rather than relabelled.

Issue-304 workflow run `33785448601`, job `100748979710`, re-executed the unchanged confirmatory design under v27. Artifact `9905391271` (`issue-304-demographic-baseline-confirmatory`, SHA-256 `9c8c2cc4f661377e2dca151147bb4c98d146a5668bc8d7c8de48c26ab381be96`) contains the exact workflow-generated `expected-result.json` copied into the canonical `confirmatory-result.json`.

Reverification outcome:

- all **780/780** declared runs completed;
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`;
- the recommendation remains **`no_universal_demographic_baseline`**;
- long-run analysis still rejects a universal stable regime (`researchGateStatus = failed`) and still detects multiple stable regimes;
- environment dependence and initialization dependence remain undetected in this design;
- for every demographic schedule, dependency-aware fission still lowers terminal population and late realized growth and increases mate limitation relative to the fixed-founder control.

Representative movement from the v26 reference to v27 is expected because mortality draws are now coupled to scientifically invariant person identities rather than arbitrary founder labels:

- negative-growth fission-minus-fixed mean terminal population: `-10.1308` -> `-11.0077` people;
- positive-growth fission-minus-fixed mean terminal population: `-89.1077` -> `-91.1231` people;
- replacement fission-minus-fixed mean terminal population: `-40.9` -> `-36.5923` people;
- positive-growth fixed-founder mean late growth rate/year moves from approximately `-0.00025668` to `-0.00038166`;
- primary long-run classifications move from `drifting=743, insufficient_data=31, stable=6` to `drifting=747, insufficient_data=25, stable=8`;
- stochastic multi-regime treatment contexts move from 2 to 1 while `multipleStableRegimesDetected` remains `true`.

This is therefore a **causal v27 scientific-reference rebaseline**, not a silent rewrite of v26 evidence. The immutable Audit-v4 discovery target `v0.3.4` / `anthrosim-model-semantics-v25` and the reviewed v26 AV4-001 reference retain their original provenance. The v27 canonical result is copied byte-for-byte from the exact workflow artifact after validating its semantics ID, run count, recommendation, precision decisions, long-run conclusion and paired-effect directions.
