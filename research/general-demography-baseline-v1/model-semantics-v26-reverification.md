# Model-semantics v26 reverification

Audit-v4 AV4-001 / issue #486 changes the causal same-seed assignment of annual fertility RNG draws by replacing incidental `PersonId` ordering with persisted scientific stochastic-coupling ranks. Because that changes deterministic continuation semantics, the current model advances from `anthrosim-model-semantics-v25` to `anthrosim-model-semantics-v26` and the frozen issue #304 demographic baseline must be scientifically re-executed rather than treated as analysis-method-compatible by assumption.

Issue-304 workflow run `33756260873`, job `100651295597`, re-executed the current confirmatory design under v26. Artifact `9893726528` (`issue-304-demographic-baseline-confirmatory`) records the exact result copied into the canonical `confirmatory-result.json`.

Reverification outcome:

- all **780/780** declared runs completed;
- all three predeclared Monte Carlo precision gates returned `sufficient_stop`;
- the recommendation remains **`no_universal_demographic_baseline`**;
- long-run analysis still rejects a universal stable regime (`researchGateStatus = failed`) and still detects multiple stable regimes;
- for every demographic schedule, dependency-aware fission still lowers terminal population and late realized growth and increases mate limitation relative to the fixed-founder control.

Representative quantitative movement from the preserved v25 reference to v26 is expected because fertility draws are now coupled to scientifically invariant person identities rather than arbitrary founder labels:

- negative-growth fission-minus-fixed mean terminal population: `-9.1385` -> `-10.1308` people;
- positive-growth fission-minus-fixed mean terminal population: `-84.9` -> `-89.1077` people;
- replacement fission-minus-fixed mean terminal population: `-38.4769` -> `-40.9` people;
- positive-growth fixed-founder mean late growth rate/year moves to approximately `-0.00025668` under v26;
- primary long-run classifications move from `drifting=738, insufficient_data=39, stable=3` to `drifting=743, insufficient_data=31, stable=6`, while `multipleStableRegimesDetected` remains `true`.

This is therefore a **causal v26 scientific-reference rebaseline**, not a silent rewrite of v25 evidence and not an analysis-method compatibility update. The immutable Audit-v4 discovery target `v0.3.4` / `anthrosim-model-semantics-v25` retains its original provenance. The v26 canonical result is copied byte-for-byte from the exact workflow artifact after validating its semantics ID, research execution identity, run count, recommendation, precision decisions, long-run conclusion and paired-effect directions.
