# Audit v5 Area A — background-mortality cadence adversary

Target: immutable `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / `anthrosim-model-semantics-v33`.

This is fresh Audit-v5 Area-A evidence targeting **update-frequency effects** in the authoritative M3/M2 scheduler. It is separate from AV5-001 and is not merely a replay of an Audit-v4 regression.

## Hypothesis

Background demographic mortality is specified as an annual age-indexed probability but is executed over elapsed M3 intervals. Changing `resources.periodsPerYear` changes the number and timing of mortality boundaries. Under the declared elapsed-risk contract, that operational partition must not materially change the one-year mortality distribution when all resource/condition effects are neutralized.

## Controlled experiment

For each M3 cadence `1`, `4`, `12`, and `365` periods/year:

- run 8,192 one-year replicates using seeds `1..=8192`;
- one synthetic founder in a one-cell world;
- annual background mortality fixed to `500,000 / 1,000,000` in every age band;
- fertility disabled;
- annual resource need zero;
- condition-mediated/scarcity mortality disabled;
- permanent migration disabled.

The endpoint is whether the sole founder is dead at the end of the run.

The test reports exact death counts and empirical annual-risk estimates for every cadence. Predeclared guards are deliberately conservative relative to binomial Monte Carlo precision:

- each arm must remain between 47% and 53% mortality;
- maximum minus minimum cadence mortality must be at most 4 percentage points.

At `n=8192`, the standard error at true `p=0.5` is approximately 0.00552, so the individual-arm ±3 percentage-point band is more than five standard errors. The spread guard is intended to detect a material cadence-induced shift without converting routine Monte Carlo noise into an audit finding.

## Interpretation

A passing result falsifies this particular update-frequency hypothesis for the tested annual risk/configuration; it does not prove all scheduler cadences or age-band crossings are invariant and does not complete Area A by itself.

A failing scientific guard requires investigation of the exact counts, age/schedule boundary semantics, and interval-risk composition before classification. Harness/build failures are not scientific evidence.
