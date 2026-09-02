# Observable-support sensitivity reporting contract v1

## Purpose

Issue #221 requires more than predeclaring spatial and temporal support. Where evidence resolution is uncertain, the declared alternative binnings must actually be executed, and any substantive dependence of the inference on support scale must be preserved as part of the scientific result.

This contract is downstream of `observable-support-aggregation-v1.md` and does not alter authoritative simulation state.

## Required report

After deriving an `anthrosim-observable-support-assessment-v1`, a study produces an `anthrosim-observable-support-sensitivity-report-v1`.

For every support-plan observable, the report preserves:

- the primary analysis identity;
- one analysis identity for every predeclared alternative binning;
- a study-defined `inferenceClass` for the primary and each alternative analysis;
- whether the substantive inference changes across those declared support choices;
- an explicit dependence statement whenever the inference changes.

The report is bound to the exact support assessment identity and receives its own deterministic SHA-256 identity.

## Exact execution coverage

The validator compares the report against the frozen support plan. The alternative analysis set must equal the declared `alternativeBinnings` set exactly.

A report therefore fails closed if it:

- omits a declared alternative;
- invents an undeclared alternative;
- duplicates an alternative ID;
- reports a result for an unknown observable;
- omits a support-plan observable;
- binds to another support assessment.

This prevents a study from predeclaring scale sensitivity but later reporting only the most convenient aggregation.

## Material dependence

This v1 contract uses a deliberately narrow, machine-checkable definition: support-scale dependence is material when at least one declared alternative has a different study-defined `inferenceClass` from the primary analysis.

`inferenceClass` is not a universal AnthroSim significance threshold. It is the frozen study's substantive interpretation category, for example `supports_h1`, `does_not_distinguish`, or another protocol-appropriate classification. The scientific threshold or rule producing that class belongs in the study protocol/analysis design rather than this generic validator.

The report's `materialScaleDependence` flag must equal the result implied by those preserved classes. If it is true, `dependenceStatement` must be non-empty. A study therefore cannot silently mark a changed substantive conclusion as scale-insensitive.

Numeric effect sizes may vary without changing `inferenceClass`; those values remain the responsibility of the identity-bearing source analyses. The report points to those analyses through `analysisIdentity` rather than duplicating arbitrary result schemas.

## Tooling

A declaration can be normalized and identity-sealed with:

```text
python scripts/research-observable-support-results.py derive \
  --plan observable-support.json \
  --assessment observable-support-assessment.json \
  --declaration observable-support-sensitivity-declaration.json \
  --output observable-support-sensitivity-report.json
```

Existing reports can be checked with `validate` and exact deterministic reports can be checked with `verify`.

## Scientific boundary

This contract does not decide which archaeological polygon, phase, catchment, chronology, aggregation operation, or material-dependence threshold is historically correct. It enforces that the choices already declared by the study are executed and honestly reported.

It does not modify M1-M9 trajectories, RNG behavior, model semantics, checkpoints, spatial transformation semantics, or benchmark reference values.

## Issue #221 closure boundary

Together with the support-plan contract, this closes the generic machine-enforced portion of #221's acceptance criterion:

- empirical comparisons declare compatible observed/simulated support;
- uncertain support predeclares defensible alternatives;
- every declared alternative is represented by an identity-bearing analysis result;
- substantive inference changes across those alternatives are explicitly reported.

A real study still has to justify its chosen support definitions and inference classes from its evidence and protocol. Archaeological sampling/preservation remains the separate observation-model problem tracked by #209.

## Executed-analysis provenance binding (Audit-v3 AV3-007)

An `analysisIdentity` is not an opaque label. For sensitivity reporting it must resolve beneath `analysis/observable-support/` to a verified `anthrosim-analysis-provenance` schema-v2 record with an `analysis-provenance-v2-sha256-*` identity and `executionStatus: executed_by_wrapper`. The generic analysis-provenance verifier must succeed against the same finalized `study-result-binding.json`.

Each resolved analysis must consume, as an exact argv token and SHA-256-bound input, one `observable-support-binning-definition` generated from the supplied support plan and finalized support assessment. That definition fixes the observable, primary/alternative binning ID, observed/simulated support, and exact spatial/temporal aggregation rules. The analysis must also emit exactly one fingerprinted `observable-support-inference` output whose `inferenceClass` is the class reported by this sensitivity report. Fabricated identities, duplicate provenance identities, reused analysis identities, missing/tampered outputs, wrong study bindings, wrong binnings, and unconstrained inference labels therefore fail closed.

The report identity remains deterministic over the validated analysis provenance identities and reported support result. This layer reuses downstream analysis provenance v2 rather than defining a weaker parallel execution-receipt format.

