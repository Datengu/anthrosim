# Observable support and aggregation contract v1

## Purpose

Empirical model-data comparison is meaningful only when the simulated and observed quantities are compared at compatible spatial and temporal support. AnthroSim's raw cell and day resolution is an execution representation, not automatically the resolution of archaeological evidence.

This contract implements the analysis-side requirement from issue #221 without changing simulation trajectories or forcing one universal aggregation scheme.

## Separation of concerns

Three different scale questions must remain distinct:

1. **Numerical raster resolution** describes the physical size represented by a simulation cell and is governed by the spatial-resolution contract (#203).
2. **Finite simulation extent/boundaries** describe the modeled domain and boundary dependence (#211).
3. **Reporting/observation support** describes the spatial area and time interval over which an empirical quantity and its simulated analogue are aggregated. This document governs that third question.

The future archaeological observation model (#209) may formalize additional sampling and preservation processes. It must not erase the support declaration required here.

## Companion support plan

A study that will make an empirical model-data comparison creates an `anthrosim-observable-support-plan-v1` JSON object before interpreting results. Each comparison observable must have exactly one plan entry containing:

- observed spatial support and its source identity;
- observed temporal support and its source identity;
- the simulated quantity's native spatial and temporal support;
- the exact spatial aggregation rule applied to simulation output;
- the exact temporal aggregation rule applied to simulation output;
- missing-data and weighting rules;
- whether evidence resolution is fixed or uncertain;
- predeclared defensible alternative binnings when resolution is uncertain;
- a rule requiring aggregation-scale dependence to be reported when substantive inference changes.

An empirical support object identifies its `kind`, `unit`, human-readable but unambiguous `definition`, and evidence/source identity. Aggregation rules identify the source artifact/variable, operation, grouping, weighting, and missing-data behavior.

## Binding to the frozen StudyProtocol

The plan receives a deterministic SHA-256 content identity. Every StudyProtocol observable covered by the plan binds that exact identity in its existing `interpretation` field using:

`observable-support-plan-v1:<planIdentity>`

This avoids silently changing the frozen StudyProtocol schema while still making the support definition part of the protocol's immutable scientific identity. Editing a polygon definition, phase definition, aggregation operation, weighting rule, missing-data rule, or alternative binning changes the plan identity and invalidates the frozen protocol binding.

For empirical-comparison validation, every observable referenced by a comparison must have an empirical support-plan entry. Raw synthetic mechanism tests do not need to invoke this empirical research gate.

## Source-run identity and derived provenance

`research-observable-support.py derive` can bind the assessment to a preserved `study-result-binding.json`. When supplied, the assessment records the exact `studyExecutionId`, `researchId`, and protocol identity. The assessment itself has a deterministic identity and can be supplied to the broader analysis-provenance tooling from #232 as an input artifact.

This means a derived empirical comparison can preserve both:

- the exact aggregation definitions; and
- the exact source study execution/research root from which the aggregated quantities were derived.

A study-result binding whose protocol identity does not match the supplied frozen protocol is rejected.

## Resolution uncertainty and sensitivity

If empirical spatial or temporal resolution is uncertain, `resolutionUncertainty` must be `uncertain` and at least one alternative binning/aggregation must be declared. A fixed-resolution declaration must not carry alternatives.

The plan also requires a `dependenceReportingRule`. Its scientific purpose is explicit: if the substantive conclusion changes across defensible declared aggregation choices, that dependence is part of the result rather than a nuisance to hide.

This contract predeclares and preserves those sensitivity choices. It does not manufacture a claim that sensitivity has been run; the downstream analysis must execute the declared alternatives and preserve their results/provenance.

## Fail-closed behavior

The validator rejects, among other cases:

- an empirical observable with missing observed spatial support;
- an empirical observable with missing observed temporal support;
- unknown or misspelled plan fields;
- empty aggregation source/operation/grouping/weighting/missing-data definitions;
- uncertain support with no alternative binning;
- fixed support with undeclared alternatives;
- support entries referring to unknown StudyProtocol observables;
- an observable whose protocol does not bind the exact plan identity;
- empirical comparisons whose observables are absent from the support plan;
- a study-result binding from another protocol;
- a previously derived assessment that does not exactly re-derive.

## Tooling

Validate the frozen relationship:

```text
python scripts/research-observable-support.py validate \
  --plan observable-support.json \
  --protocol study-protocol.json
```

Derive a provenance-bearing assessment after the study execution is preserved:

```text
python scripts/research-observable-support.py derive \
  --plan observable-support.json \
  --protocol study-protocol.json \
  --study-result-binding study-result-binding.json \
  --output observable-support-assessment.json
```

`verify` deterministically re-derives and byte-semantically compares an existing assessment.

## Scientific boundary

This is research-analysis governance. It does not modify M1-M9 mechanisms, model semantics, spatial transformation semantics, RNG behavior, checkpoints, or benchmark reference values. It does not choose a historically correct polygon, phase, catchment, or temporal bin for a real study; those remain evidence-bearing study decisions.
