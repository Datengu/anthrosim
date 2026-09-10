# Confirmatory Monte Carlo frozen-seed and sample-value binding v2

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

Before invoking the general precision engine, this entry point requires the concatenated ordered `design.seedBatches` in the precision plan to equal `study/example/research-definition.json`'s exact ordered `seeds` list byte-for-value at the JSON value level. Any missing, extra, replaced, duplicated, or reordered seed fails closed.

Since AV6-010/#729, exact seed identity is necessary but not sufficient for AnthroSim-derived confirmatory samples. For a protocol observable whose `id` matches a submitted sample group, the confirmatory entry point inspects the observable's declared `source` before inference.

The first supported authoritative derivation is:

```text
research.analysis.runs.state
```

For that source, `study/example/research/analysis/runs.json` is authoritative. Its seed set must exactly equal the frozen confirmatory seeds, with no duplicates, and each submitted per-seed value must equal `1.0` exactly when the canonical run state is `completed` and `0.0` otherwise. Missing, duplicated, reordered, fabricated, or contradictory sample rows fail closed before the statistical engine can emit a confirmatory decision.

Protocol sources beginning `external.` or `user.` are explicitly treated as user-authored observations rather than AnthroSim-derived execution values. Their scientific provenance remains the responsibility of the declared external evidence workflow; they are not rewritten to match AnthroSim run state. A protocol source beginning `research.` for which no explicit authoritative derivation has been implemented fails closed instead of allowing a free-form sample to masquerade as an AnthroSim-derived quantity.

Legacy or synthetic protocols with no matching declared observable retain the existing generic sample behaviour. New production confirmatory studies should declare the observable and source explicitly so the domain gate can determine whether semantic binding is required.

The general precision engine then independently verifies that:

- the exact precision-plan identity is frozen into `StudyProtocol.ensemblePolicy.replicationPolicy`;
- the exact frozen study protocol matches `study-result-binding.json`;
- confirmatory work was bound before execution and remains eligible for a pre-result confirmatory claim;
- the analysed sample ends at an allowed predeclared cumulative batch boundary;
- sample seed identities and order match that declared boundary;
- the estimand-specific Monte Carlo precision rule and threshold are applied deterministically.

Together these checks create the intended lineage for an AnthroSim-derived confirmatory estimand:

```text
frozen StudyProtocol
  -> exact precision-plan identity
  -> frozen ResearchExperimentDefinition exact seed design
  -> finalized authoritative study output
  -> explicit observable source/derivation
  -> semantically validated per-seed estimand sample
  -> Monte Carlo precision diagnostic
  -> #232 executable analysis provenance
```

Byte-level analysis provenance remains complementary rather than substitutive. Declaring both an authoritative study artifact and an arbitrary sample file as provenance inputs proves that those exact bytes were available and replayed; it does not by itself prove that the sample values were derived from the authoritative artifact. The confirmatory domain gate therefore performs the semantic check before inference.

For a canonical reported result, the confirmatory entry point, the precision plan, the per-seed estimand sample, authoritative source artifacts, and emitted diagnostic should all be declared in the existing #232 analysis definition/provenance record. This addendum does not replace or duplicate that downstream provenance system.

`scripts/test-research-monte-carlo-confirmatory.py` preserves the frozen-seed regression. `scripts/test-research-monte-carlo-sample-binding.py` separately demonstrates acceptance of a truthful authoritative run-state sample, rejection of a contradictory value, preservation of explicit external observations, and fail-closed handling for unsupported AnthroSim-derived sources. The Rust integration wrapper executes both suites in normal repository CI.

This remains analysis-layer governance. It does not alter simulation RNG streams, mortality semantics, model state, Monte Carlo interval mathematics, sequential stopping behaviour, or `MODEL_SEMANTICS_ID`. AV6-009/#726 remains a separate statistical-validity finding.
