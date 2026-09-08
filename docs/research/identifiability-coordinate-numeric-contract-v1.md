# Identifiability parameter-coordinate numeric contract v1

Status: authoritative scientific contract for research-facing parameter coordinates used by `scripts/research-identifiability.py`.

## Purpose

Parameter coordinates are scientific design identities. An identifiability result must not silently merge two accepted coordinates merely because a downstream numeric representation cannot distinguish them.

This contract distinguishes exact JSON integer coordinates from approximate floating coordinates and defines when AnthroSim must fail closed.

## Exact JSON integers

A parameter dimension whose evaluated coordinates are all JSON integers is treated as an **exact integer coordinate domain**.

For such a dimension:

- coordinate identity is the exact integer value, including values above `2^53`;
- explored-level counts use exact integer identity;
- `fullRange` and `compatibleRange` retain exact integer endpoints;
- range subtraction is exact integer arithmetic;
- `normalizedCompatibleWidth` is formed from the exact compatible-span/full-span rational ratio and only then serialized to the JSON numeric result field;
- the numeric parameter diagnostic reports `coordinateRepresentation: "exact_json_integer"`.

This is required for supported wide-integer configuration coordinates such as `PopulationConfig.maxPersonRecords` (`u64`).

## Approximate floating coordinates

If a numeric parameter dimension contains floating coordinates, the dimension is interpreted as an **approximate binary64 coordinate domain**.

For such a dimension:

- finite floating values keep the established binary64 analysis semantics;
- safely representable integer coordinates may participate in the same approximate domain;
- the numeric parameter diagnostic reports `coordinateRepresentation: "binary64_approximate"`.

A floating coordinate is therefore not retroactively claimed to be an exact decimal/rational value merely because its JSON spelling contains decimal digits.

## Fail-closed mixed domains

A dimension that mixes floating coordinates with an exact integer whose magnitude is outside binary64's consecutive-integer domain (`abs(integer) > 2^53`) is rejected.

AnthroSim must not silently coerce that exact integer into binary64 because doing so can collapse distinct executed-design coordinates and create a false parameter-identification result.

The machine-readable result-level `parameterCoordinateSemantics` object records:

```json
{
  "integerCoordinates": "exact_json_integer",
  "floatingCoordinates": "binary64_approximate",
  "unsafeMixedIntegerFloatCoordinates": "fail_closed"
}
```

## Coordinate identity across diagnostics

The same coordinate identity contract applies to research-facing interpretation of:

- parameter diagnostics and identification decisions;
- staged parameter diagnostics;
- one-dimensional profiles;
- pairwise interaction surfaces;
- compatible-parameter-combination/equifinality summaries;
- nuisance-parameter compensation diagnostics.

Profiles, pairwise surfaces and equifinality keys use canonical JSON coordinate identity, so exact integer coordinates remain distinct there. Numeric range arithmetic must preserve the same distinction rather than re-collapsing those values through binary64.

## Preserved scientific controls

The implementation must preserve all of the following:

- `[0, 1, 2]` with the first two levels compatible has normalized compatible width `0.5` and is not identified at threshold `0.25`;
- `[2^53, 2^53+1, 2^53+2]` has the same exact width `0.5` and must not collapse its first two levels;
- equivalent tests near the maximum `u64` domain retain exact integer identity;
- ordinary floating coordinates remain supported under explicit binary64-approximate semantics;
- fewer than two genuinely explored levels remain non-identifying (AV3-011/#419);
- parameter coordinates remain bound to the immutable executed design before they may authorize a research claim (AV4-011/#535).

## Compatibility and semantics

This repair changes analysis/inference correctness, not simulation execution. It does not alter causal simulator behavior, stochastic trajectories, checkpoint schemas, or `anthrosim-model-semantics-v33`.

The result schema remains version 2 with additive machine-readable coordinate-semantics fields. Consumers that make scientific claims from parameter diagnostics should honor `coordinateRepresentation` and `parameterCoordinateSemantics` rather than assuming all numeric coordinates are binary64 values.
