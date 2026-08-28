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
- paired run-length endpoints;
- alternative analysis-window starts and ends;
- #205 research-coordinate IDs that represent initialization alternatives;
- #205 research-coordinate IDs that represent environment/spatial-realization alternatives.

There are deliberately no universal convergence tolerances. A study must choose tolerances appropriate to its observable and claim before treating a result as equilibrium-like.

## Exact research execution binding

The analysis is not allowed to quietly operate on a convenient subset of a research execution. Before reading trajectories, the tool requires:

- `research-manifest.json` and `research-plan.json` to be identical;
- `analysis/runs.json` to identify the same `researchId`;
- the `analysis/runs.json` run-ID set to equal the complete immutable planned-run set exactly;
- every row's point ID, seed, coordinates, relative directory and resulting configuration to equal its immutable planned run;
- every completed child `manifest.json` experiment, state digest, model version, model-semantics ID and Git revision to agree with the immutable research execution.

A missing planned replicate, an extra row, or a changed child bundle fails closed instead of silently changing the replicate population being diagnosed.

## Trajectory classes

For each completed run, v1 reports one of four primary states:

- `stable`: all declared metrics satisfy the required trailing adjacent-window and within-window tolerances;
- `cyclic_stable`: the same requirement is met after comparing a declared repeated phase profile, and the stable profile has material within-cycle amplitude;
- `drifting`: enough trajectory is available, but at least one metric fails the declared trailing stability criterion;
- `insufficient_data`: the declared analysis interval is unavailable or does not contain enough complete windows to evaluate the criterion.

Only non-overlapping complete windows are used. They are anchored at the end of the selected interval so the latest regular annual state is represented; an incomplete leading fragment is not silently promoted into a full comparison window.

Adjacent-window differences are symmetric relative differences rounded upward to whole permille. The upward rounding prevents a small non-zero change from disappearing through integer truncation. For a non-cyclic metric, within-window drift compares the first and second halves of a window. For a declared cycle, the diagnostic compares same-phase profiles both between adjacent windows and between complete-cycle halves of each window.

These are diagnostics of the declared observables, not a proof that every latent model state is stationary.

## Annual cadence and early terminal snapshots

Core `MetricSeries` has `annual_boundary_plus_terminal` cadence. If a run terminates between year boundaries, its subannual terminal metric snapshot is valid provenance but is not a regularly spaced annual stationarity observation. V1 therefore uses whole-year-boundary snapshots for long-run windows and retains the authoritative terminal day/stop reason separately.

This prevents an early subannual terminal point from shifting a declared cycle phase or receiving the same statistical weight as a full annual interval.

An `equilibrium_like` gate can pass only when every run reaches ordinary `durationReached`. Population extinction or the person-record safety ceiling is scientifically meaningful long-run behavior, but it cannot be relabelled as a successfully observed equilibrium merely because the preceding annual points happened to look flat.

If a protocol declares an explicit primary analysis end beyond a run's realized terminal day, the primary assessment is `insufficient_data` with `declared_analysis_end_not_observed`. The tool never silently shortens a predeclared interval to fit what happened to be available.

## Cyclic and seasonal behavior

A repeated cycle is not automatically classified as failure to converge. If a metric declares `cyclePeriodSnapshots`, the diagnostic compares same-phase means between adjacent windows and reports `cyclic_stable` when the phase profile repeats within tolerance.

Cycle regime signatures are rotation-normalized when used to identify attractors, so the same unanchored repeating profile does not become two artificial regimes solely because a run ends on a different phase.

Current core metrics can diagnose annual-state and multi-year cycles from the annual artifact. They **cannot** establish within-year seasonal stationarity. A study making a subannual seasonal claim must use an observability artifact with suitable subannual cadence rather than inventing information between annual points.

## Treatment context versus multiple attractors

A #205 research experiment may deliberately vary ordinary treatments as well as initialization and environment. Different treatment levels are not evidence for multiple attractors of one model configuration.

The protocol therefore identifies which coordinate IDs represent initialization and environment. All remaining #205 coordinates form a `treatmentContext`. Stable-regime frequencies and path-dependence comparisons are evaluated **within** those treatment contexts.

For example, a low-resource treatment settling near population 100 and a high-resource treatment settling near population 200 are two treatment results, not automatically two attractors. By contrast, two different stable regimes within the same treatment context—arising across initialization, environment, or stochastic replicates—are reported as multiple stable regimes.

## Multiple regimes and path dependence

Every stable/cyclic-stable run receives a coarse `regimeSignature` from the declared metric bin widths. Binning exists to distinguish materially different stable regions; it is not parameter calibration.

The aggregate report preserves:

- stable-regime frequencies within every treatment context;
- treatment contexts in which more than one stable regime is observed;
- outcome frequencies by declared initialization within treatment context;
- outcome frequencies by declared environment within treatment context;
- contexts with more than one stable regime across stochastic seeds after treatment, initialization and environment are held fixed.

Initialization/environment dependence compares the **full normalized outcome distributions**, not just which regime labels appear. Thus two initialization states that both reach regimes A and B but in materially different proportions are still visibly dependent on initialization.

If multiple stable regimes occur within a treatment context, `multipleStableRegimesDetected` is true. The result must report regime frequencies/dependence rather than pooling different attractors into one mean and calling it a single equilibrium.

`singleRegimePooledLongRunAverageSupported` is deliberately stricter: it can be true only for a passing equilibrium-like diagnostic with one treatment context and no multiple stable regimes. The field is not permission to pool distinct experimental treatments.

This supports #212-style seed/environment decomposition: environment dimensions can be labelled separately from stochastic replicate seed, while initialization dimensions can be labelled independently. More detailed causal attribution remains study-specific.

## Run-length sensitivity

`runLengthSensitivityEndDays` re-evaluates the exact preserved trajectory at predeclared earlier endpoints. Because an AnthroSim deterministic run has the same causal prefix when extended, this is a paired test of whether the apparent long-run classification depends on having simulated longer.

An endpoint beyond a run's realized terminal day is recorded as unavailable rather than fabricated. An unavailable declared sensitivity check prevents the equilibrium-like gate from passing.

## Analysis-window sensitivity and #219

`analysisStartSensitivityDays` and `analysisEndSensitivityDays` re-evaluate plausible alternative windows against the same realized trajectory. If an alternative changes status or regime signature, the corresponding sensitivity flag is true.

The primary `analysisStartDay`/optional end should be the interval frozen for the study by the #219 analysis-window contract / #230 study protocol. Alternative windows are sensitivity analyses, not permission to inspect several burn-ins and retrospectively choose the preferred result.

Cumulative since-start metric counters remain inappropriate as direct window outcomes unless their scientific meaning is explicitly derived by differencing or raw-event analysis. Recommended v1 diagnostic metrics are state-like quantities such as living population, living occupied cells, mean condition when defined, or resource stock. The diagnostic rejects missing/non-integer metric pointer values rather than silently imputing them.

## Equilibrium-like research gate

For `claimMode: equilibrium_like`, `researchGateStatus` is `passed` only when:

1. the analysis run set exactly matches the immutable planned run set;
2. every planned run completed;
3. every completed run reached ordinary `durationReached` rather than extinction or an operational safety stop;
4. every trajectory is `stable` or `cyclic_stable` under the primary criterion;
5. at least one run-length sensitivity endpoint is declared;
6. at least one alternative analysis start is declared;
7. at least one alternative analysis end is declared;
8. every declared sensitivity assessment is actually observable from every relevant run;
9. none of those declared run-length/window variants changes a run's status or regime signature.

The gate intentionally does not require initialization/environment coordinate IDs to be non-empty, because a particular study design may not vary those dimensions. The output records whether those groupings were declared so a claim cannot imply that untested initialization/environment robustness was demonstrated.

Multiple stable regimes do **not** by themselves fail the equilibrium-like gate. They change the permitted interpretation: a regime-specific/path-dependent long-run result may be supported, while a single pooled long-run average is not.

For `claimMode: explicitly_transient`, `researchGateStatus` is `not_required`. Drifting/non-stationary behavior can be the intended historical result and must be reported as such rather than forced through an equilibrium criterion.

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
  "runLengthSensitivityEndDays": [73000, 109500, 146000],
  "analysisStartSensitivityDays": [18250, 54750],
  "analysisEndSensitivityDays": [109500, 127750],
  "initializationCoordinateIds": ["founder_state"],
  "environmentCoordinateIds": ["spatial_realization_seed"],
  "rationale": "Study-specific diagnostic thresholds and sensitivity intervals fixed before interpreting the long-run comparison."
}
```

Metric pointer names are evaluated against each preserved `MetricSnapshot`; they are not a free-form expression language.

The normalized protocol receives a SHA-256 content identity. By default the assessment is written beneath:

```text
analysis/studies/<protocolIdentity>/long-run-diagnostics.json
```

Changing a tolerance, metric, cycle period, window, grouping coordinate or claim mode changes the protocol identity. None of these changes alter simulation identity.

## Controlled synthetic benchmarks

The regression suite includes deliberately known cases:

- replicated constant trajectories recognized as one stable regime;
- a repeated two-phase trajectory preserved as `cyclic_stable` when its period is declared;
- a monotonic trajectory rejected as `drifting` for equilibrium-like interpretation;
- two stable outcome regimes tied to different initialization labels, reported rather than pooled;
- initialization groups with the same attractor support but different attractor frequencies, which must still be detected as dependent;
- two stable regimes under the same treatment/initialization/environment context but different stochastic seeds;
- different ordinary treatments that settle at different levels but must **not** be mislabelled as multiple attractors;
- an apparently stable late trajectory whose classification changes under declared shorter-duration/alternative-window checks;
- a stable trajectory with missing required sensitivity coverage, which cannot pass the equilibrium gate;
- an early-terminated run that cannot masquerade as equilibrium;
- a predeclared analysis end beyond the realized run, which remains unobserved rather than being shortened;
- an explicitly transient study where non-stationarity is reported but stationarity is not required;
- missing/tampered immutable planned runs and provenance metadata, which must be rejected.

These tests validate the analysis logic. They are not evidence that AnthroSim's prehistoric applications are stationary or that the example tolerances are scientifically appropriate.

## Relationship to initialization and environment audits

The initialization coordinate mechanism gives studies a machine-readable way to incorporate alternatives produced by #192, #213 and #216 rather than assuming one founder/resource initial state has been forgotten after a burn-in. Environment coordinates likewise attach #212-style realization variation to the long-run regime report instead of mixing it invisibly with ordinary stochastic replicates.

The tool cannot infer which #205 dimension represents those concepts. The study must declare the relevant coordinate IDs; the output records whether it did so.

## Relationship to #304

#304 can now compare candidate demographic baselines only after applying a declared long-run diagnostic under the relevant #207 household structures. `replacement_control_v1` should not be promoted merely because its intrinsic schedule is replacement-centred; realized trajectories must first be classified for stability, extinction/path dependence, initialization/environment dependence and structural sensitivity.

## Model-semantics boundary

This work changes only analysis and interpretation. It does not alter `ExperimentConfig`, initialization, M2/M3/M4/M8/M9 transitions, RNG streams/draw order, checkpoints, metric generation, or `MODEL_SEMANTICS_ID`.
