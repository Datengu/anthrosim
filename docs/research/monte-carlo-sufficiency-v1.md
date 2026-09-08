# Monte Carlo precision and replicate sufficiency v1

**Status:** normative research-analysis contract for GitHub issue #231, with the Audit-v5 AV5-005/#640 guarded normal-approximation repair.  
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

If the precision threshold is met **and the selected estimator's executable validity contract is satisfied**, it returns:

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
| `mean` | guarded sample-variance CLT standard error with a two-sided normal critical value |
| `difference_in_means` | guarded independent two-sample CLT standard error |
| `paired_mean_difference` | guarded CLT interval on exact per-seed paired differences |
| `probability` | Wilson score interval for a Bernoulli probability such as extinction/persistence |
| `quantile` | exact finite-sample binomial/order-statistic rank interval; infeasible confidence/quantile/sample-size combinations fail closed |

### Guarded normal-approximation contract for mean-family estimands

Audit-v5 AV5-005/#640 demonstrated that the former machine gate could stop at two identical observations. For a process with `X=0` with probability `0.9` and `X=10` with probability `0.1`, the first two values are `[0,0]` with probability `0.81`; the old sample-variance normal interval was then `[0,0]` and the gate called it a nominal 95% `sufficient_stop`, even though the true mean is `1`. A prose warning was therefore insufficient.

The authoritative executable now applies a fail-closed validity guard to `mean`, `difference_in_means`, and `paired_mean_difference`:

- every contributing Monte Carlo group must contain at least **30** replicates at the evaluated predeclared batch boundary;
- the observed variance used by the mean-family estimator must be positive;
- for `difference_in_means`, both independent groups must have positive observed variance;
- for `paired_mean_difference`, the exact per-seed paired differences must have positive observed variance;
- if either condition fails, the normal interval may still be emitted as a descriptive diagnostic, but `precision.sufficient` is forced to `false` and the run cannot return `sufficient_stop`;
- a sequential design may then continue only to its already predeclared next batch; a fixed design remains insufficient with no post-hoc seed extension.

The 30-replicate floor is a **defensive operational asymptotic guard, not a finite-sample coverage theorem**. It prevents the machine from treating very small samples as though the central-limit approximation were already justified, but it does not make arbitrary heavy-tailed, rare-event, multimodal, or highly skewed outputs normal. Likewise, observed non-zero variance is necessary for this gate but is not proof that the variance estimate is stable. Studies for which those assumptions are doubtful must predeclare a more suitable reviewed method or transform/estimand rather than interpreting this gate as distribution-free coverage.

For those three estimands, `confidenceLevel` therefore means a **nominal asymptotic normal-approximation confidence level**, not an exact finite-sample distribution-free guarantee. The diagnostic makes that boundary machine-visible with:

- `precision.normalApproximationValidity.validForStopping`;
- `minimumReplicatesPerGroup` and `observedReplicatesPerGroup`;
- `requiresPositiveObservedVariance`;
- explicit failure `reasons`;
- `precision.confidenceSemantics`.

This guard intentionally does not alter the precision-plan schema or identity format. The plan still freezes the estimand, confidence level, half-width threshold, design and seed batches before result inspection; the executable analysis implementation and its source provenance determine whether the selected normal approximation is valid for stopping at a particular declared boundary.

For `quantile`, diagnostic schema v2 replaces the former normal rank approximation with an exact finite-sample order-statistic coverage contract. Under a continuous population, if `K ~ Binomial(n, p)` is the number of observations below the true `p`-quantile, an emitted 0-based rank interval `[l, u]` is accepted only when `P(l + 1 <= K <= u)` is at least the declared confidence level. Rank selection depends only on `n`, `p`, and the confidence level, never on observed sample values. If no sample-only order-statistic interval can attain the declared coverage, interval bounds and half-width are null, `coverageFeasible=false`, and the gate cannot return `sufficient_stop`. At 95% confidence the minimum feasible replicate counts for `p=0.50, 0.90, 0.95, 0.99` are respectively 6, 29, 59, and 299. This contract assumes a continuous distribution for the true quantile; discrete/tied-output quantiles require separately reviewed interpretation.

`difference_in_means` means genuinely independent Monte Carlo arms with separately predeclared, disjoint seed schedules. Reusing a seed identity across those arms is not treated as evidence of independence and fails closed. `paired_mean_difference` means paired **replicate-level seed contrasts** on the same exact seed identities when scientifically justified. It does not claim per-agent common-random-number counterfactual coupling and does not alter simulator RNG semantics.

## Sample schema

The input sample is intentionally downstream of simulation execution. It contains one or two named groups and exact `(seed, value)` rows. For one-group estimands and `paired_mean_difference`, the exact seed order must equal the shared `design.seedBatches` cumulative prefix; paired groups must use the same exact identities/order. For `difference_in_means`, the plan instead carries exactly two `design.groupSeedBatches` schedules. Each arm must match its own predeclared cumulative boundary, both arms must be evaluated at the same boundary, and the two seed sets must be disjoint. A shared or overlapping seed layout is rejected before the independent variance estimator can run.

For `probability`, values are boolean or `0/1`. Other v1 estimands use finite numeric values.

The emitted diagnostic remains schema v2 and retains precision-plan schema/identity v1. For `difference_in_means`, v2 diagnostics additionally record `pairingSemantics=independent` and the exact per-group seed identities so the independent-arm contract is machine-visible. Existing diagnostic shapes remain backward-readable; guarded mean-family diagnostics add the explicit normal-approximation validity fields described above. This analysis-layer repair does not change `MODEL_SEMANTICS_ID`.

The emitted diagnostic preserves:

- precision-plan identity;
- study/protocol/result lineage when a frozen study root is supplied;
- estimand and confidence level;
- precision method and, for mean-family methods, executable approximation validity;
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

The continuous-mean demonstration now begins below the normal-approximation floor, where the gate is required to continue rather than stop. After the already predeclared independent batch raises the sample above the floor and leaves positive observed variance, the Monte Carlo interval narrows below the predeclared threshold and the gate may stop.

The suite also verifies:

- the exact AV5-005 two-observation `[0,0]` adversary fails closed despite a computed zero-width interval and can only request its already predeclared next batch;
- zero observed variance cannot establish deterministic mean or paired-difference behaviour for stopping purposes;
- guarded independent and paired mean-family examples above the sample floor;
- Wilson precision for a persistence/extinction-style probability;
- exact deterministic reproduction from the same seed sample;
- a changed seed design changing plan identity and preserved seed provenance;
- rejection of an undeclared partial sequential batch;
- a fixed design that fails precision and has no post-hoc continuation escape;
- genuinely independent two-arm mean contrasts with disjoint provenance-bound seed schedules, including rejection of overlapping/same-seed layouts;
- paired-seed mean contrasts with covariance-sensitive adversaries;
- central and tail quantile estimands with exact binomial coverage assertions, including fail-closed under-supported samples at `p = 0.5, 0.9, 0.95, 0.99`;
- confirmatory frozen-study binding and rejection of a post-result replacement precision plan.

A Rust integration-test wrapper executes the Python regression suite in the repository test matrix.

## Existing confirmatory-result review after AV5-005

The checked-in `research/general-demography-baseline-v1/confirmatory-result.json` records 130 replicates for its mean diagnostic, a non-zero interval half-width, and `normal_clt_mean_se`. It is therefore above the new hard replicate floor and is not an instance of the demonstrated small-n/zero-variance stopping failure. That historical result remains bound to the exact analysis implementation and evidence chain under which it was produced; this review does not retroactively convert its normal approximation into an exact finite-sample guarantee.

Any other canonical confirmatory mean-family diagnostic should be interpreted against the executable guard in the source revision that produced it. A historical artifact that stopped below the floor or from zero observed variance must not be cited as satisfying the repaired current sufficiency contract without rerunning the diagnostic under the current implementation.

## Interpretation boundary

Passing this gate means only that the declared Monte Carlo sample has the predeclared numerical precision for the specified estimand under the specified stochastic experiment design **and that the executable estimator-specific stopping validity checks pass**.

For the normal-CLT mean family, passing remains an asymptotic approximation claim rather than an exact finite-sample distribution-free coverage guarantee. It does not prove the model is archaeologically correct, parameter values are known, mechanisms are identifiable, exposures are equal, evidence is independent, or the structural model is adequate. Those are separate scientific questions and must remain separate in claims and provenance.
