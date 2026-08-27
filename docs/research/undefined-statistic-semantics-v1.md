# Undefined denominator-based statistic semantics v1

## Status and scope

This note defines how AnthroSim represents scientific summary statistics whose denominator can be empty. It records the repair made for issue #222 and applies to persisted run summaries, metric snapshots, sweep-derived analysis and Explorer presentation.

The change is an observability/statistical-semantics correction. It does not alter agent behaviour, migration choice, resource allocation, demography, RNG streams or authoritative simulation state.

## Core rule

An in-domain numeric value must never be used as a sentinel for an empty observation set.

- `0` means a quantity was defined and its measured value was zero.
- `null` means the quantity was mathematically undefined because there were no observations in its denominator.
- Operational censoring remains a separate lifecycle/provenance concept and must not be inferred from `null`.

This distinction matters because zero is scientifically meaningful on the affected permille scales.

## Living-condition mean

`meanLivingConditionPermille` is the mean condition of people who are living at the reporting boundary.

- If `livingPopulation > 0`, the field is defined and is a numeric value in the model's condition scale. A genuine mean of zero remains `0`.
- If `livingPopulation == 0`, the field is `null`. An extinct population has no living-condition observations, so the mean is undefined rather than zero.

The same nullable meaning propagates through population summaries, resource summaries, terminal run manifests and metric snapshots. `ResourceSummary` repeats this population-level diagnostic for resource-facing observability, so it follows the same nullability rule rather than reintroducing a zero sentinel.

## Migration-quality means

The following run-level migration summaries are conditional on completed moves:

- `meanOriginResourceScorePermille`;
- `meanDestinationResourceScorePermille`;
- `meanOriginWaterSecurityScorePermille`;
- `meanDestinationWaterSecurityScorePermille`.

If `movesCompleted == 0`, all four fields are `null`. If one or more moves occurred, they are numeric, including a legitimate observed value of `0`.

The migration checkpoint state intentionally continues to persist the raw cumulative totals and `movesCompleted`. Those totals/counts are authoritative continuation state and do not require a nullable sentinel; the nullable means are derived from them only when the denominator is non-zero.

## Sweep analysis

Scientifically eligible runs remain eligible when extinction occurs or when no migration occurs. Undefined run-level quantities are not converted to zero before aggregation.

For living condition, point analysis reports:

- `livingConditionDefinedRunsScientificallyEligibleOnly`: the number of scientifically eligible runs contributing a defined living-condition mean;
- `meanLivingConditionPermilleScientificallyEligibleOnly`: the mean across those defined run-level means only.

Extinction frequency remains explicit through `populationExtinctRuns`, so excluding an undefined extinct-run condition mean does not hide the extinction outcome.

For migration, point analysis separates occurrence from conditional quality:

- `migrationMoveObservedRunsScientificallyEligibleOnly`: eligible runs with at least one completed move;
- `migrationMoveOccurrenceFractionScientificallyEligibleOnly`: fraction of eligible runs in which at least one move occurred;
- `meanMigrationOriginResourceScorePermilleMoveObservedOnly`;
- `meanMigrationDestinationResourceScorePermilleMoveObservedOnly`;
- `meanMigrationOriginWaterSecurityScorePermilleMoveObservedOnly`;
- `meanMigrationDestinationWaterSecurityScorePermilleMoveObservedOnly`.

The four quality means therefore describe runs in which the measured phenomenon occurred. They must not be interpreted as unconditional averages over zero-move runs. Movement occurrence/frequency should be inspected alongside them.

CSV output represents undefined optional values as empty fields; JSON represents them as `null`.

## Explorer presentation

The Explorer renders nullable scientific means as an em dash (`—`) rather than coercing them to `0`. A numeric zero remains visibly `0`.

This prevents extinction or zero-move states from appearing as extreme low-condition or low-quality observations.

## Schema changes

The nullable contract is explicitly versioned:

- `RunManifest`: schema v13;
- `MetricSeries` / `MetricSnapshot`: schema v3;
- `ResourceSummary`: schema v3;
- `MigrationSummary`: schema v2;
- sweep run/summary derived-analysis schema: v5;
- sweep point-analysis schema: v6.

The authoritative `ResourceSystem` and migration checkpoint-state schemas remain unchanged because they store model state/raw totals and counts rather than the affected derived means.

## Audit note

The #222 audit also checked other denominator-based observability paths. Existing spatial-observability ratios already return optional values for zero denominators, and temporary-mobility mean visitors already uses optional/null semantics when observation duration is zero. Those paths therefore did not require conversion from a numeric sentinel.

## Interpretation rule

Any downstream statistical workflow must retain the distinction between:

1. a defined zero measurement;
2. an undefined statistic caused by an empty denominator;
3. a run excluded because of operational censoring or incomplete lifecycle state.

Pooling these cases can create false parameter effects, especially when extinction or movement occurrence changes across experimental conditions.
