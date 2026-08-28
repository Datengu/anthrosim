# Monte Carlo precision and replicate sufficiency v1

**Status:** normative research-analysis contract for GitHub issue #231.  
**Scope:** process-stochastic Monte Carlo sampling uncertainty only. This layer does not change AnthroSim causal dynamics, simulation RNG streams, checkpoints, run identities, or `MODEL_SEMANTICS_ID`.

## Scientific question

Reproducibility answers whether the same simulation sample can be regenerated. It does **not** answer whether that sample is large enough for a stochastic scientific conclusion.

A confirmatory stochastic study must therefore declare, before inspecting the result, how replicate sufficiency will be judged. Eight perfectly reproducible seeds may be inadequate. Twenty or one hundred may also be inadequate for a noisy mean, a rare extinction probability, or a tail quantity.

The v1 gate is implemented by:

```text
scripts/research-monte-carlo-sufficiency.py
```

It consumes a frozen Monte Carlo precision plan plus a machine-readable sample of per-seed estimand values and emits a deterministic precision diagnostic.

## The uncertainty represented

Every plan must set:

```json
"uncertaintyCategory": "process_stochastic_monte_carlo"
```

The resulting interval answers:

> How much would this estimated stochastic summary vary because the declared simulation process was rerun with another independent Monte Carlo sample, conditional on this model, parameterisation, evidence treatment and estimand?

It does **not** quantify:

- parameter uncertainty;
- archaeological or evidence uncertainty;
- structural/model-form uncertainty.

Those uncertainties may be studied by other AnthroSim research layers, but a narrow Monte Carlo interval must never be presented as resolving them.

## Precision-plan schema v1

A plan declares:

- `planId` and content-derived `planIdentity`;
- `uncertaintyCategory`;
- one explicit `estimand`;
- a confidence level;
- a declared maximum acceptable interval half-width;
- fixed or sequential replication design;
- exact ordered seed batches;
- pairing semantics;
- a scientific rationale.

The identity is SHA-256 over canonical JSON with `planIdentity` blanked. Obtain it with:

```text
python scripts/research-monte-carlo-sufficiency.py identity precision-plan.json
```

After inserting that identity, validate the complete plan with:

```text
python scripts/research-monte-carlo-sufficiency.py validate-plan precision-plan.json
```

### Frozen StudyProtocol binding

StudyProtocol v1 already contains `ensemblePolicy.replicationPolicy`. #231 deliberately preserves schema v1 compatibility rather than invalidating archived study protocols.

A confirmatory protocol that uses this gate binds the exact precision plan by setting:

```text
ensemblePolicy.replicationPolicy =
  "monte-carlo-precision-plan-v1:<exact planIdentity>"
```

Because the replication-policy string is part of the frozen `StudyProtocol` content identity, changing the precision rule changes the protocol identity/revision. The precision plan itself can remain a separate machine-readable analysis artifact while its exact content identity is frozen before simulation execution.

When `diagnose --study-dir ...` is used, the gate verifies the exact frozen `study-protocol.json` against `study-result-binding.json`, requires the bound precision-plan identity above, and for confirmatory work requires `boundBeforeExecution=true` and `confirmatoryPreResultClaimEligible=true`.

This is the anti-post-hoc firewall: a different threshold, estimand, seed schedule or stopping plan cannot be retrofitted onto the old confirmatory protocol without producing a different plan/protocol identity.

## Fixed and sequential designs

### Fixed

A fixed design declares exactly one complete seed batch. The diagnostic may only be evaluated at that final sample. If the declared precision target is not met, the result is **insufficient** and the plan provides no scientific permission to add more seeds while retaining the original confirmatory stopping claim.

A fixed count should therefore be justified before execution by a suitable precision or power calculation for the actual estimand.

### Sequential

A sequential design declares two or more independent seed batches in advance. The gate may be evaluated only after a complete cumulative batch boundary. If precision is inadequate and another batch was predeclared, it returns:

```text
insufficient_continue_with_declared_next_batch
```

and records the exact next seed batch that may be added.

If the precision threshold is met, it returns:

```text
sufficient_stop
```

A sample ending inside a batch is rejected. A sample containing undeclared seeds is rejected. Reordering or replacing seeds is rejected. This makes repeated seed-by-seed peeking machine-visible and prevents the ordinary confirmatory workflow from simply continuing until a preferred result looks stable or significant.

The stopping criterion concerns interval precision, not whether a hypothesis has the desired sign or p-value.

## Estimand-specific methods

V1 does not impose one interval blindly on every output.

Supported estimands are:

| Estimand | V1 precision method |
| --- | --- |
| `mean` | sample-variance CLT standard error with a two-sided normal critical value |
| `difference_in_means` | independent two-sample CLT standard error |
| `paired_mean_difference` | CLT interval on exact per-seed paired differences |
| `probability` | Wilson score interval for a Bernoulli probability such as extinction/persistence |
| `quantile` | order-statistic rank interval using a binomial-rank normal approximation |

The diagnostic records the exact method name. The methods are analysis contracts, not universal guarantees. In particular, very small samples, extreme tails, heavy-tailed outputs or rare events may demand a more conservative/specialized predeclared method in a future schema rather than pretending a generic interval is adequate.

`paired_mean_difference` means paired **replicate-level seed contrasts** when scientifically justified. It does not claim per-agent common-random-number counterfactual coupling and does not alter simulator RNG semantics.

## Sample schema

The input sample is intentionally downstream of simulation execution. It contains one or two named groups and exact `(seed, value)` rows. The exact seed order must equal one declared cumulative seed-batch prefix.

For `probability`, values are boolean or `0/1`. Other v1 estimands use finite numeric values.

The emitted diagnostic preserves:

- precision-plan identity;
- study/protocol/result lineage when a frozen study root is supplied;
- estimand and confidence level;
- precision method;
- declared half-width threshold;
- exact seed identities;
- replicate count;
- completed batch boundary;
- point estimate and interval;
- realized half-width;
- sufficient/insufficient decision;
- exact next predeclared batch, if any;
- explicit statement of what uncertainty is and is not represented.

## Downstream analysis provenance (#232)

The precision diagnostic is a scientific analysis result and should be executed or captured through the existing analysis-lineage layer for canonical studies.

A confirmatory `anthrosim-analysis-definition` should declare:

- the extracted per-seed estimand sample and precision plan as analysis inputs;
- `scripts/research-monte-carlo-sufficiency.py` as an implementation artifact;
- the diagnostic JSON as an output artifact;
- the exact command argv, including `--study-dir`;
- the Python/runtime environment artifact required by #232.

`research-analysis-provenance.py run` then binds the executable precision calculation, its inputs, code, environment and exact output bytes to the frozen study result. #231 does not duplicate that lineage mechanism.

## Synthetic scientific verification

`scripts/test-research-monte-carlo-sufficiency.py` contains controlled demonstrations rather than only schema tests.

The continuous-mean demonstration begins with four predeclared independent seeds deliberately producing a noisy estimate. Its interval is too wide, so the gate rejects sufficiency and permits only the already-declared second batch. After that independent batch is added, the Monte Carlo interval narrows below the predeclared threshold and the gate stops.

The suite also verifies:

- Wilson precision for a persistence/extinction-style probability;
- exact deterministic reproduction from the same seed sample;
- a changed seed design changing plan identity and preserved seed provenance;
- rejection of an undeclared partial sequential batch;
- a fixed design that fails precision and has no post-hoc continuation escape;
- paired-seed mean contrasts;
- a quantile/tail estimand with an order-statistic method;
- confirmatory frozen-study binding and rejection of a post-result replacement precision plan.

A Rust integration-test wrapper executes the Python regression suite in the repository test matrix.

## Interpretation boundary

Passing this gate means only that the declared Monte Carlo sample has the predeclared numerical precision for the specified estimand under the specified stochastic experiment design.

It does not prove the model is archaeologically correct, parameter values are known, mechanisms are identifiable, exposures are equal, evidence is independent, or the structural model is adequate. Those are separate scientific questions and must remain separate in claims and provenance.
