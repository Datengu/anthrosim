# Research analysis-window provenance v1

Status: normative study-analysis contract for GitHub issue #219.

AnthroSim research executions preserve the exact simulated configuration and execution artifacts. That is not the same thing as declaring which part of the trajectory a scientific study will analyse. This contract makes the distinction explicit without changing authoritative simulation state.

## Scientific purpose

Initialization choices can create transient behaviour that is scientifically irrelevant to a study's intended equilibrium or later-period question. Excluding that transient can be defensible, but choosing a burn-in after inspecting the trajectory creates researcher degrees of freedom and can hide initialization dependence.

The v1 analysis-window protocol therefore freezes a study-facing observation rule separately from the causal simulation configuration. A protocol identifies:

- the study (`studyId`);
- the first simulated day included in the primary analysis (`analysisStartDay`);
- an optional final included day (`analysisEndDayInclusive`), otherwise the run's realized terminal day;
- why that boundary was chosen (`selectionRule` plus a non-empty `rationale`);
- optional predeclared alternative windows for initialization/path-dependence sensitivity.

The allowed primary selection rules are:

- `predeclared_fixed_duration`;
- `convergence_diagnostic`;
- `externally_meaningful_historical_start`;
- `initial_state_in_scope`;
- `other_explicit`.

A convergence-based rule still requires the study to document the diagnostic in the rationale or associated study protocol. The v1 tool preserves the declaration; it does not infer convergence from a visually convenient part of the result.

## Interval semantics

A research definition has a configured maximum duration, but a completed run can terminate earlier, for example because the population becomes extinct. Those two boundaries must not be conflated.

For every planned run the output therefore preserves the configured maximum execution interval. For a completed run it additionally reads the authoritative `manifest.json` and uses its realized `endTime` as terminal day `T`:

- realized execution interval: `[0, T]`;
- burn-in/equilibration interval: `[0, analysisStartDay)`;
- primary analysis interval: `[analysisStartDay, analysisEndDayInclusive]`.

If no explicit analysis end is declared, the end defaults to that run's realized `T`. If an early-terminated completed run ends before the declared start or an explicit declared end, the tool fails closed rather than pretending that nonexistent days were observed or silently shortening a predeclared fixed interval. The run manifest's state digest must also agree with the immutable research execution state.

For runs that are not yet completed, interval validation can only use the configured maximum duration and the output labels that basis as planned rather than realized. Scientific analysis of completed runs is always bounded by realized execution.

This distinction matters because the #205 research definition can vary run duration as a scientific dimension and because stop reasons can vary across seeds. An `analysisStartDay` of zero means that the initial state is included in the analysis and the burn-in interval is empty.

## Binding to #205 research execution provenance

Use:

```text
python scripts/research-analysis-window.py <research-root> <protocol.json>
```

The tool requires the redundant immutable `research-manifest.json` and `research-plan.json` written by `anthrosim-research` to agree exactly. It also checks that `research-state.json` belongs to the same `researchId` and that every state's immutable run identity fields agree with the plan.

The validated protocol is canonicalized and assigned a SHA-256 content identity:

```text
analysis-window-protocol-v1-sha256-<digest>
```

Output is written beneath:

```text
analysis/studies/<protocolIdentity>/
```

with:

- `protocol.json`: the normalized exact protocol;
- `analysis-window-manifest.json`: the protocol identity, source research/definition identity, source revision, planned execution bounds, realized bounds for completed runs, stop reason, declared windows, and a resolved window for every run to which the protocol can validly apply.

Changing the analysis start, end, rule, rationale, study ID, or sensitivity-window declaration changes the protocol identity. It does not change the source simulation's run identity or state digest.

## Metric-snapshot boundary rule

Current core `metrics.json` snapshots contain a mixture of state quantities and cumulative **since-start** counters. Merely dropping pre-burn-in snapshots would therefore not make a cumulative field such as births, deaths, unmet need, harvested resources, or migration moves into an analysis-window total.

For every completed run, the analysis-window manifest records:

- the realized terminal snapshot day and verifies it equals the run manifest's realized `endTime`;
- the metric cadence;
- whether an exact metric snapshot exists at `analysisStartDay`;
- the preceding snapshot day when one exists;
- exactly which metric snapshot days fall inside the analysis interval.

The tool deliberately does **not** relabel since-start counters as post-burn-in totals. Interval totals must be derived by differencing against an exact boundary snapshot when the statistic permits it, or from the authoritative raw events/other appropriate source. If the chosen start day does not coincide with the preserved metric cadence, metric-based interval totals require a more appropriate derivation rather than silently using the previous annual snapshot.

This distinction prevents an apparently windowed result from still carrying initialization-period accumulation.

## Initialization/path-dependence sensitivity

Optional `sensitivityWindows` preserve plausible alternative analysis starts/ends alongside the primary choice. Each has a stable `id` and rationale. Duplicate IDs and duplicate intervals are rejected.

These variants exist so a study can ask whether its scientific conclusion depends on the burn-in choice. If materially different plausible windows change the conclusion, that dependence should be reported as initialization/path dependence. The intended workflow is not to inspect all windows and retrospectively select whichever produces the preferred result.

The tool records these windows but does not decide whether a conclusion changed; that belongs to the study's downstream analysis and protocol.

## Example

```json
{
  "schemaVersion": 1,
  "studyId": "example-settlement-study",
  "analysisWindow": {
    "analysisStartDay": 36500,
    "selectionRule": "predeclared_fixed_duration",
    "rationale": "Exclude the first 100 simulated years before evaluating the declared outcomes."
  },
  "sensitivityWindows": [
    {
      "id": "burn_in_50y",
      "analysisStartDay": 18250,
      "rationale": "Shorter plausible initialization interval."
    },
    {
      "id": "burn_in_200y",
      "analysisStartDay": 73000,
      "rationale": "Longer plausible initialization interval."
    }
  ]
}
```

## Model-semantics boundary

Analysis-window provenance is an observation/analysis rule only. It does not alter:

- `ExperimentConfig`;
- authoritative initial state;
- M2/M3/M4/M8/M9 transitions;
- random streams or draw order;
- checkpoints or continuation state;
- `MODEL_SEMANTICS_ID`.

Two studies may analyse different preserved intervals from the exact same deterministic run and therefore have different analysis-protocol identities while sharing the same simulation state digest.

## TRACE boundary

This contract closes one research-methods gap: a preserved study analysis can identify exactly which realized simulated interval was eligible and why the start boundary was chosen. It also provides a provenance-visible place to predeclare alternative burn-in windows and prevents early termination from being mistaken for unobserved planned duration.

It does not by itself prove that a selected burn-in is scientifically adequate, that a convergence diagnostic is valid, that the model has reached equilibrium, or that the result is empirically corroborated. Those remain study-specific TRACE obligations. Frozen confirmatory-study hypotheses, decision criteria, evidence roles and amendment history are addressed by the broader study-protocol work in #230.
