# Monte Carlo numeric fidelity v1

**Status:** normative numeric-input addendum to `monte-carlo-sufficiency-v1.md`.  
**Scope:** research-facing numeric fidelity for `scripts/research-monte-carlo-sufficiency.py`; this does not change simulator causal semantics, RNG streams, checkpoints, or `MODEL_SEMANTICS_ID`.

## Contract

Monte Carlo replicate values have two deliberately distinct numeric contracts.

1. **JSON/Python integers are exact integer observables.** Python's arbitrary-precision integer value is authoritative. If converting an accepted integer to IEEE-754 binary64 would change its value, the gate preserves the integer and derives continuous-estimand moments using exact rational arithmetic before the final interval scale is converted for JSON diagnostic reporting.
2. **JSON non-integer numbers are binary64 observables after parsing.** Python's standard JSON parser does not preserve the original decimal token or its source precision. Finite floating values with absolute magnitude below `2^53` therefore retain the established binary64 analysis path. Floating values with absolute magnitude at or above `2^53` fail closed because the gate cannot prove whether scientifically material unit-scale information was already lost before analysis. Integer-valued observables in that range must be supplied as JSON integers; genuinely continuous observables should be rescaled to a scientifically interpretable range.

This distinction prevents a JSON integer such as `9007199254740993` from being silently collapsed to `9007199254740992.0`, while avoiding any claim that a decimal-looking JSON token carries precision that the parser no longer exposes.

## Continuous estimators

For `mean`, `difference_in_means`, and `paired_mean_difference`:

- the ordinary safe binary64 path is retained unchanged when every input is safely represented;
- if any accepted integer would not round-trip through binary64 exactly, all relevant estimator moments are derived over exact rational representations of the accepted values;
- paired differences are formed exactly before their sample variance is calculated;
- independent-arm covariance/seed contracts remain unchanged;
- the normal critical value and final diagnostic reporting remain binary64, but only **after** exact sample variation has been preserved in the mean/variance calculation;
- a finite diagnostic that cannot be represented at the final reporting scale fails closed and requires rescaling rather than returning a false precision decision.

When the exact-rational fallback is activated, the precision diagnostic records a `numericFidelity` object describing the accepted input representations, exact-rational moment arithmetic, and the point at which binary64 reporting resumes. Existing safe-range diagnostics retain their prior shape and numerical path.

The same fallback is used for value arithmetic in the quantile estimator when an unsafe exact integer is present; the existing exact finite-sample rank-coverage method is unchanged.

## Permanent adversaries

`scripts/test-research-monte-carlo-large-integer-fidelity.py`, executed from the Rust integration test matrix, permanently covers:

- `mean` on `[2^53, 2^53+1, 2^53, 2^53+1]`;
- `difference_in_means` with the same large-offset nonzero within-arm variance;
- `paired_mean_difference` where exact per-seed subtraction must retain `[0,1,0,1]`;
- fail-closed handling of a large floating input whose original decimal/source precision cannot be recovered;
- retention of the established ordinary binary64 path for safe floating inputs.

The AV4-010 post-merge evidence step separately reruns the exact original Audit-v4 adversary byte-for-byte against merged `main`.
