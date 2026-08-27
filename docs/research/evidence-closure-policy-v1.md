# Evidence closure policy v1

Status: proposed research-governance contract for issue #181.

This document defines the boundary between a configuration that is only schema-valid and one whose empirical or evidence-informed provenance claims are sufficiently supported to be described as evidence-closed for research use. It does not make any existing configuration empirically valid by itself and does not change simulation trajectories.

## Separate validation questions

AnthroSim must keep execution validity and evidence closure distinct.

Execution validity asks whether an experiment is structurally valid and executable under its model and schema contracts. Evidence closure asks whether empirical or evidence-informed provenance claims are supported by stable, inspectable evidence links and reproducible source identity.

Synthetic, hypothetical and unresolved experiments may remain executable. They must not become research-ready merely because ordinary configuration validation succeeds.

## Provenance meanings at the research boundary

- `synthetic_validation`: explicitly synthetic or null-model input. No empirical closure is claimed.
- `unresolved`: an explicit open assumption. It may be used for exploratory execution but blocks evidence-closed research status.
- `empirical_direct`: requires evidence directly supporting the claimed model value or definition.
- `empirical_derived`: requires evidence plus an explicit derivation from the source quantity or definition to the model quantity.
- `evidence_informed`: requires evidence showing that the model choice is constrained or informed by the declared source without representing it as a direct measurement of the model parameter.

The provenance label is a claim, not proof. Research-readiness validation must substantiate that claim from the evidence catalogue.

## Closure requirements

For each scientifically substantive subsystem or parameter carrying `empirical_direct`, `empirical_derived` or `evidence_informed` provenance, research-readiness validation must prove that:

1. an `EvidenceCatalog` is present;
2. the catalogue itself is valid;
3. appropriate evidence exists for the claimed assumption;
4. support resolves to the exact substantive parameter, stable collection member, or explicitly allowed containing schedule/object;
5. the evidence and model-side provenance meanings are compatible;
6. the source has a reproducible immutable or content-based identity appropriate to that source;
7. any required derivation is explicit and inspectable;
8. no required substantive assumption remains `unresolved`.

A catalogue containing only unrelated evidence does not satisfy closure.

## Scalars, schedules and collections

Scalar parameters should normally be supported by a stable parameter path resolving directly to that leaf.

Positional array indices are not stable scientific identities and must not be used as evidence addresses for schedule members.

A schedule-like collection may be closed in either of two explicit ways:

- whole-object closure, where the complete canonical schedule is one evidenced scientific object with a stable identity; or
- member closure, where members have stable semantic IDs independent of array order and evidence can bind to those IDs.

Research-readiness validation must fail closed when it cannot prove which schedule member or complete schedule an evidence record supports.

## External inputs

An evidence-grounded external input must not rely only on mutable prose, a mutable download location, or a human-readable label.

Research-ready closure requires a reproducible immutable or content-based identity, for example an exact content digest, a genuinely immutable dataset/version identifier, or another repository-approved equivalent identity.

Spatial layers, spatial transformations, founder-state inputs, focal-region evidence and other evidence-grounded external artifacts follow the same principle.

## Derived values

For `empirical_derived` claims, the derivation is part of the provenance claim. Closure requires the source quantity and units, derivation method, resulting simulation units or definition, applicability, and scientifically consequential uncertainty where relevant.

An empirical source does not automatically validate a derived parameter when the derivation is undocumented.

## Preserved readiness result

Evidence-closure validation should produce a versioned preserved result distinct from ordinary configuration validation. At minimum it must distinguish:

- `closed`: all required empirical/evidence-informed claims are supported and no blocking unresolved assumptions remain;
- `not_closed`: one or more explicit closure failures exist;
- `not_applicable_synthetic`: the run is explicitly synthetic/null-model and makes no research-ready empirical provenance claim.

Failure output should preserve machine-readable reasons including the affected parameter or object identity and the failure class, such as missing catalogue, unrelated evidence, missing reproducible source identity, unresolved assumption or unsupported collection-member addressing.

Downstream tooling must not infer `closed` from successful simulation execution alone.

## Required negative acceptance cases

Implementation of this policy must reject evidence-closed status for at least:

- an `empirical_direct` subsystem with `evidence = null`;
- an empirical/evidence-informed subsystem with an empty catalogue;
- a catalogue containing only unrelated evidence;
- evidence resolving to a different parameter than the claimed assumption;
- an `unresolved` substantive assumption in research-ready mode;
- schedule evidence depending on positional array indices;
- an external empirical input lacking the required reproducible source identity;
- an `empirical_derived` value without an inspectable derivation.

## Required positive acceptance cases

Implementation must include positive fixtures showing that intentionally evidence-grounded repository benchmarks or studies can satisfy the closure boundary when their evidence and source identities are complete.

Passing this gate means only that provenance claims are machine-closed under the declared policy. It does not establish that the evidence is historically correct, sufficient to identify the true model, or adequate for a particular archaeological inference.

## Implementation boundary

This policy remains separate from permissive exploratory execution. Implementing #181 should add an explicit research-readiness validation path rather than making unresolved or synthetic experiments impossible to run.

The eventual preserved provenance must keep these states machine-distinguishable: schema-valid, executable, evidence-closed, and empirically validated. They are different scientific claims.
