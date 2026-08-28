# Sweep exposure normalization v1

AnthroSim sweep analysis preserves valid early scientific stops such as `populationExtinct` rather than censoring them away. That means two scientifically eligible completed runs can have different realized durations even when they share the same configured horizon.

This document defines the derived-analysis contract for cumulative outcomes whose opportunity to accumulate depends on realized simulation time.

## Raw cumulative outcomes remain valid

The ordinary sweep run table preserves `simulatedDays` and `endDay` together with cumulative quantities such as births, deaths, condition-mediated deaths, unmet resource need, completed migration moves and migration distance. The ordinary point table continues to expose raw cumulative descriptive means. Those totals answer questions about the complete realized trajectory and must not be deleted merely because exposure differs.

A smaller cumulative value is not, by itself, evidence of lower process intensity. A run that becomes extinct after ten years has had less time to accrue later births, deaths, migration and resource deficit than a run that survives for one hundred years.

A machine-readable worked reversal is preserved in `docs/research/sweep-exposure-normalization-example-v1.json`.

## Exposure-aware derived analysis

`scripts/research-sweep-exposure.py` consumes the authoritative derived `analysis/runs.json` table and produces a provenance-bound exposure assessment. It reports, for every parameter point:

- counts of duration-reached, population-extinct and operationally censored runs;
- extinction frequency among scientifically eligible outcomes;
- mean, minimum and maximum realized `simulatedDays`;
- the existing cumulative outcomes as explicitly named cumulative means; and
- equal-replicate-weight means of per-run rates scaled to 365 simulated days.

The normalized quantities use exactly one denominator:

```text
rate_per_365_simulated_days = cumulative_value * 365 / simulated_days
```

A zero-day trajectory has no time-rate denominator, so its normalized rate is `null`, not zero.

## What these rates do not mean

The v1 exposure assessment does **not** claim person-time, household-time, migration-opportunity, fertility-opportunity, total-need, or another process-specific denominator. A per-365-simulated-day rate can distinguish a short intense trajectory from a long mild one, but changing population size can still change the amount of process opportunity within the same elapsed time.

Consequently:

- births/deaths per 365 simulated days are not per-capita demographic rates;
- migration moves per 365 simulated days are not moves per household-year or per migration evaluation;
- unmet need per 365 simulated days is not a fraction of total need; and
- condition-mediated deaths per 365 simulated days are not mortality hazards.

Future analyses should use more specific denominators when those exposures are preserved and scientifically appropriate. No universal denominator should be substituted merely for convenience.

## Extinction is an outcome, not censoring

`populationExtinct` remains a scientifically eligible outcome. Exposure normalization must therefore be reported jointly with extinction frequency. The normalized rate describes the intensity observed before the run stopped; it does not replace the terminal fact that the population became extinct.

`personRecordLimitReached`, by contrast, remains operational censoring and is excluded from scientific point summaries while remaining explicit in the source run table.

## Fixed-horizon estimands

A statistic declared as a value at a fixed horizon, for example a 100-year terminal spatial metric, is unavailable when the run stops before that horizon unless the study defines another explicit estimand for early-stop trajectories.

The M8.6 terrain benchmark therefore treats a paired primary metric as unavailable whenever either member of the pair is `terminalDegenerate`/did not reach the declared duration. The early-stop terminal state remains preserved as an outcome, but it cannot silently enter a statistic that claims to compare states at the full horizon.

This rule does not change the canonical M8.6 result because its protected reference runs all reach their declared duration.

## Analysis windows

Version 1 normalizes from simulation start to realized stop because that is the exposure preserved in the existing sweep table. When study-level analysis windows are implemented under #219, exposure-aware analyses should use the declared scientific window rather than automatically using day zero to stop.

## Scientific boundary

This is a derived-analysis change only. It does not change transition rules, mortality, migration, resource allocation, RNG streams, checkpoint state, run identity, protected scientific references or `MODEL_SEMANTICS_ID`.
