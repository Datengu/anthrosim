# Monte Carlo sequential stopping contract — AV6-009

**Status:** normative addendum to `monte-carlo-sufficiency-v1.md` for Audit-v6 AV6-009/#726. Where the older v1 text permits `sufficient_stop` at an intermediate sequential boundary from an ordinary fixed-sample interval, this addendum supersedes that permission.

## Scientific defect

Audit-v6 AV6-009 demonstrated that repeatedly evaluating an ordinary Wilson probability interval and stopping when its observed half-width first crosses the declared threshold does not preserve the interval's declared confidence coverage. In the preserved exact Bernoulli adversary (`p=0.305`, nominal confidence 0.95, boundaries 30/100/300, `maxHalfWidth=0.1685`), each fixed boundary individually covered at least 95%, while the data-dependent stopped procedure covered only 0.918976766485.

The failure is a repeated-look/optional-stopping problem. Predeclaring seed batches prevents undeclared peeking, but it does not by itself make a fixed-sample confidence interval always-valid at a data-dependent stopping time.

## Repaired executable contract

AnthroSim does not currently claim an always-valid confidence sequence or alpha-spending construction for any supported Monte Carlo estimator family. Therefore:

1. A `fixed` design retains its estimator-specific fixed-sample precision semantics.
2. A `sequential` design may still be evaluated at every complete predeclared cumulative batch boundary for descriptive monitoring and provenance.
3. At every **intermediate** sequential boundary, an ordinary fixed-sample interval is descriptive only. `precision.sufficient` is forced to `false`, and interval width cannot authorize inferential `sufficient_stop`.
4. If another predeclared batch exists, the decision remains `insufficient_continue_with_declared_next_batch` even when the displayed interval half-width is already below `maxHalfWidth`.
5. Because early inferential stopping is disabled, every sequential path reaches the **predeclared terminal cumulative boundary**. At that terminal boundary the ordinary estimator-specific interval may again govern the precision decision under its existing fixed-sample validity assumptions.
6. The diagnostic exposes `precision.sequentialStoppingValidity`, including whether the current boundary is valid for inferential stopping, the observed and terminal replicate counts, and the executable contract string.
7. At an intermediate boundary, `precision.confidenceSemantics` explicitly states that the displayed fixed-sample interval is not valid for inferential early stopping.

This is intentionally conservative. A future reviewed implementation may restore genuine early stopping only by supplying a sequentially valid inferential method whose declared coverage matches the stopping rule, such as an appropriate confidence sequence or explicitly justified repeated-look procedure.

## Estimator-family scope

The exact discovery reproduction used the Wilson score interval for a Bernoulli probability. The causal defect, however, is not Wilson-specific: ordinary fixed-sample intervals generally do not become always-valid merely because the look schedule was predeclared. The terminal-only rule therefore applies to all currently supported estimator families:

- `probability`;
- `mean`;
- `difference_in_means`;
- `paired_mean_difference`;
- `quantile`.

Existing estimator-specific validity guards still apply at the terminal boundary. In particular, the mean-family normal approximation still requires its minimum replicate count and positive observed variance, and quantile inference still requires feasible exact order-statistic coverage.

## Scientific interpretation

An intermediate sequential diagnostic answers: "what would the current fixed-sample interval look like at this predeclared monitoring boundary?" It does **not** answer: "may this study stop now while retaining the displayed nominal confidence guarantee?"

A terminal sequential diagnostic is inferentially equivalent to evaluating the predeclared maximum sample, because no data-dependent early width rule is permitted to remove paths before that boundary.

This repair does not change simulator dynamics, RNG streams, model semantics, study execution identities, or the exact predeclared seed schedule. It changes only the analysis-layer permission to stop replication early on the basis of ordinary fixed-sample interval width.

## Regression requirements

CI must retain controls that:

- reproduce an intermediate Wilson interval whose numerical half-width satisfies the threshold but nevertheless cannot return `sufficient_stop`;
- repeat the check at more than one intermediate boundary;
- show that the terminal predeclared boundary can still make a fixed-sample precision decision;
- fail closed at intermediate looks for the other supported estimator families rather than assuming their fixed-sample intervals are always-valid;
- preserve all existing fixed-design interval controls, seed/provenance binding, estimator-specific validity guards, and terminal precision behavior.

For AV6-009 closure, the original #725 adversary must be independently rerun after the production repair is merged. The preserved adversary may require only the narrow version-drift adaptation allowed by `audit-reverification-version-drift.md` if its historical terminal assertion expected the demonstrated defect to remain present.