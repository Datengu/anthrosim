# Long-run regime diagnostics v1

**Status:** normative study-analysis contract for GitHub issue #220.  
**Scope:** convergence, approximate stationarity, cyclic-regime and path-dependence diagnostics; this does not change AnthroSim model dynamics.

A long AnthroSim run is not automatically an equilibrium run. A late-time average can still describe a trajectory that is drifting, approaching extinction, cycling, or occupying one of several history-dependent regimes. Long-run interpretation therefore requires an explicit study-level diagnostic rather than a duration threshold.

## Scientific contract

`research-long-run-diagnostics.py` binds a versioned diagnostic protocol to an immutable `anthrosim-research` execution and reads the preserved annual `metrics.json` trajectory for every completed replicate.

The protocol declares:

- whether the study is making an `equilibrium_like` claim or is intentionally `explicitly_transient`;
- the primary analysis start/end;
- a fixed number of metric snapshots per comparison window;
- the number of consecutive trailing stable-window comparisons required;
- one or more study-relevant trajectory metrics;
- study-specific tolerances for adjacent-window change and within-window drift;
- a regime bin width used only to describe distinct stable outcome regions;
- an optional known cycle period in metric snapshots;
- optional paired run-length endpoints;
- optional alternative analysis-window starts;
- #205 research-coordinate IDs that represent initialization alternatives;
- #205 research-coordinate IDs that represent environment/spatial-realization alternatives.

There are deliberately no universal convergence tolerances. A study must choose tolerances appropriate to its observable and claim before treating a result as equilibrium-like.

## Trajectory classes

For each completed run, v1 reports one of four primary states:

- `stable`: all declared metrics satisfy the required trailing adjacent-window and within-window tolerances;
- `cyclic_stable`: the same requirement is met after comparing a declared repeated phase profile, and the stable profile has material within-cycle amplitude;
- `drifting`: enough trajectory is available, but at least one metric fails the declared trailing stability criterion;
- `insufficient_data`: the declared analysis interval does not contain enough complete windows to evaluate the criterion.

Only non-overlapping complete windows are used. They are anchored at the realized end of the selected interval so the terminal observed state is always represented; an incomplete leading fragment is not silently promoted into a full comparison window.

Adjacent-window differences are symmetric relative differences rounded upward to whole permille. The upward rounding prevents a small non-zero change from disappearing through integer truncation. Within-window drift compares the first and second halves of a window. For a declared cycle, the halves are aligned to whole cycles and adjacent windows are compared phase by phase.

These are diagnostics of the declared observables, not a proof that every latent model state is stationary.

## Cyclic and seasonal behavior

A repeated cycle is not automatically classified as failure to converge. If a metric declares `cyclePeriodSnapshots`, the diagnostic compares same-phase means between adjacent windows and reports `cyclic_stable` when the phase profile repeats within tolerance.

Current core `MetricSeries` has annual-boundary cadence plus a possible terminal snapshot. It can therefore diagnose annual-state and multi-year cycles from that artifact. It **cannot** establish within-year seasonal stationarity from annual snapshots. A study making a subannual seasonal claim must use an observability artifact with suitable subannual cadence rather than inventing information between annual points.

## Multiple regimes and path dependence

Every stable/cyclic-stable run receives a coarse `regimeSignature` from the declared metric bin widths. Binning exists to distinguish materially different stable regions; it is not parameter calibration.

The aggregate report preserves:

- frequency of each stable regime across replicates;
- regime frequencies by declared initialization coordinates;
- regime frequencies by declared environment coordinates;
- contexts with more than one stable regime across stochastic seeds;
- whether initialization groups have different observed regime supports;
- whether environment groups have different observed regime supports.

If more than one stable regime is observed, `multipleStableRegimesDetected` is true and `singleRegimePooledLongRunAverageSupported` is false. A study must report regime frequencies/dependence rather than pooling different attractors into one mean and calling it a single equilibrium.

This directly supports #212-style seed/environment decomposition: environment dimensions can be labelled separately from stochastic replicate seed, while initialization dimensions can be labelled independently. More detailed causal attribution remains study-specific.

## Run-length sensitivity

`runLengthSensitivityEndDays` re-evaluates the exact preserved trajectory at earlier predeclared endpoints. Because an AnthroSim deterministic run has the same causal prefix when extended, this is a paired test of whether the apparent long-run classification depends on having simulated longer.

An endpoint beyond a run's realized terminal day is recorded as unavailable rather than fabricated. If any available run-length assessment differs from that run's primary status/regime signature, `runLengthSensitivityDetected` is true.

## Analysis-window sensitivity and #219

`analysisStartSensitivityDays` re-evaluates plausible alternative starts against the same realized run. If an alternative start changes the status/regime signature, `analysisWindowSensitivityDetected` is true.

The primary `analysisStartDay`/optional end should be the interval frozen for the study by the #219 analysis-window contract / #230 study protocol. The alternative starts are sensitivity analyses, not permission to inspect several burn-ins and retrospectively choose the preferred result.

Cumulative since-start metric counters remain inappropriate as direct window outcomes unless their scientific meaning is explicitly derived by differencing or raw-event analysis. The recommended v1 diagnostic metrics are state-like quantities such as living population, living occupied cells, mean condition when defined, or terminal resource stock. The diagnostic script rejects non-integer/missing metric pointer values rather than silently imputing them.

## Research gate

For `claimMode: equilibrium_like`, `researchGateStatus` is `passed` only when:

1. every planned analysis run represented in the research execution completed;
2. every completed trajectory is `stable` or `cyclic_stable` under the primary criterion;
3. no declared run-length endpoint changes a run's status/regime signature;
4. no declared alternative analysis start changes a run's status/regime signature.

Multiple stable regimes do **not** by themselves fail the gate. They change the permitted interpretation: a regime-specific/path-dependent long-run result may be supported, while a single pooled long-run average is not.

For `claimMode: explicitly_transient`, `researchGateStatus` is `not_required`. Drifting/non-stationary behavior can be the intended historical result and must be reported as such rather than forced through an equilibrium criterion.

## Immutable research binding

The tool fails closed unless:

- `research-manifest.json` and `research-plan.json` are identical;
- `analysis/runs.json` names the same `researchId`;
- each completed run uses a safe relative path;
- the run's `manifest.json` experiment equals the immutable resulting configuration;
- the state digest equals the research execution record;
- model version, model-semantics ID and Git source revision equal the immutable research source.

The normalized protocol receives a SHA-256 identity and the result is written by default to:

```text
analysis/studies/<protocolIdentity>/long-run-diagnostics.json
```

Changing a tolerance, metric, cycle period, window, grouping coordinate or claim mode changes the protocol identity. None of these changes alter simulation identity or `MODEL_SEMANTICS_ID`.

## Example

```json
{
  "schemaVersion": 1,
  "studyId": "candidate-long-run-study",
  "claimMode": "equilibrium_like",
  "analysisStartDay": 36500,
  "windowSnapshots": 20,
  "requiredConsecutiveStableWindows": 2,
  "metrics": [
    {
      "id": "living_population",
      "sourcePointer": "/population/livingPopulation",
      "maxAdjacentWindowMeanShiftPermille": 50,
      "maxWithinWindowDriftPermille": 50,
      "regimeBinWidth": 10
    },
    {
      "id": "occupied_cells",
      "sourcePointer": "/population/livingOccupiedCellCount",
      "maxAdjacentWindowMeanShiftPermille": 50,
      "maxWithinWindowDriftPermille": 50,
      "regimeBinWidth": 2
    }
  ],
  "runLengthSensitivityEndDays": [36500, 73000, 109500],
  "analysisStartSensitivityDays": [18250, 73000],
  "initializationCoordinateIds": ["founder_state"],
  "environmentCoordinateIds": ["spatial_realization_seed"],
  "rationale": "Study-specific diagnostic thresholds fixed before interpreting the long-run comparison."
}
```

Metric pointer names are evaluated against each preserved `MetricSnapshot`; they are not a free-form expression language.

## Controlled synthetic benchmarks

The regression suite includes deliberately known cases:

- replicated constant trajectories that must be recognized as one stable regime;
- a repeated two-phase trajectory that must be preserved as `cyclic_stable` when its period is declared;
- a monotonic trajectory that must fail an equilibrium-like claim as `drifting`;
- two stable outcome regimes tied to different initialization labels, which must be reported separately rather than pooled;
- two stable regimes under the same initialization/environment context but different stochastic seeds, demonstrating stochastic multi-regime reporting;
- an apparently stable late trajectory whose classification changes under declared shorter-duration/alternative-window checks;
- an explicitly transient study where non-stationarity is reported but stationarity is not required;
- provenance tampering that must be rejected.

These tests validate the analysis logic. They are not evidence that AnthroSim's prehistoric applications are stationary or that the example tolerances are scientifically appropriate.

## Relationship to #304

#304 can now compare candidate demographic baselines only after applying a declared long-run diagnostic under the relevant #207 household structures. `replacement_control_v1` should not be promoted merely because its intrinsic schedule is replacement-centred; realized trajectories must first be classified for stability, extinction/path dependence and structural sensitivity.

## Model-semantics boundary

This work changes only analysis and interpretation. It does not alter `ExperimentConfig`, initialization, M2/M3/M4/M8/M9 transitions, RNG streams/draw order, checkpoints, metric generation, or `MODEL_SEMANTICS_ID`.
