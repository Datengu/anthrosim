# Confirmatory Monte Carlo frozen-seed binding v1

This addendum hardens the #231 Monte Carlo precision contract for frozen confirmatory studies.

A precision plan is not fully predeclared merely because it contains exact seed identities. Those identities must also be the exact ordered seeds in the `ResearchExperimentDefinition` frozen by #230 before execution. Otherwise a protocol could name one Monte Carlo design while the actual research execution used another.

Canonical confirmatory precision analysis therefore uses:

```text
python scripts/research-monte-carlo-confirmatory.py \
  precision-plan.json \
  estimand-samples.json \
  precision-diagnostic.json \
  --study-dir study/example
```

Before invoking the general precision engine, this entry point requires the concatenated ordered `design.seedBatches` in the precision plan to equal `study/example/research-definition.json`'s exact ordered `seeds` list byte-for-value at the JSON value level. Any missing, extra, replaced, or reordered seed fails closed.

The general precision engine then independently verifies that:

- the exact precision-plan identity is frozen into `StudyProtocol.ensemblePolicy.replicationPolicy`;
- the exact frozen study protocol matches `study-result-binding.json`;
- confirmatory work was bound before execution and remains eligible for a pre-result confirmatory claim;
- the analysed sample ends at an allowed predeclared cumulative batch boundary;
- sample seed identities and order match that declared boundary;
- the estimand-specific Monte Carlo precision rule and threshold are applied deterministically.

Together these checks create the intended lineage:

```text
frozen StudyProtocol
  -> exact precision-plan identity
  -> frozen ResearchExperimentDefinition exact seed design
  -> completed frozen study result
  -> exact per-seed estimand sample
  -> Monte Carlo precision diagnostic
  -> #232 executable analysis provenance
```

For a canonical reported result, the confirmatory entry point, the precision plan, the per-seed estimand sample, and the emitted diagnostic should all be declared in the existing #232 analysis definition/provenance record. This addendum does not replace or duplicate that downstream provenance system.

`scripts/test-research-monte-carlo-confirmatory.py` demonstrates both acceptance of an exact frozen seed match and rejection after one frozen research seed is replaced while the precision plan and analysed sample remain unchanged. A Rust integration wrapper executes that check in normal repository CI.

This remains analysis-layer governance. It does not alter simulation RNG streams, mortality semantics, model state, or `MODEL_SEMANTICS_ID`.
