# Resource and condition period observability v1

Issue #215 adds retained M3 resource-period observations so a preserved run can diagnose trajectory shape without rerunning with undocumented instrumentation.

## Authoritative timing

One observation is recorded after every authoritative M3 resource boundary. The row is downstream observability: it does not feed resource allocation, condition response, mortality, migration, RNG choice, or scheduling. Rows are retained in the checkpoint's `ResourceSystem` and therefore survive exact checkpoint/resume. They are also copied into `spatial-observability.json`, whose source block binds them to the exact run state, model semantics, seed, landscape identity and spatial configuration.

The observation schema is `ResourcePeriodObservation` v1. The parent resource-system causal schema is intentionally unchanged: observation history is excluded from `ResourceSystem::digest64`, so causal state identity and `MODEL_SEMANTICS_ID` do not change. Complete continuation identity still binds the serialized observation history, because an exact resumed output must preserve it.

## Per-period quantities

Each period preserves the absolute start/end day, period sequence/index, total stock before regeneration, regeneration, stock after regeneration, need, supplied amount, unmet need and stock after harvest. Each cell preserves the same stock/need/supply accounting, plus demand decomposed into `homeNeed` and `visitorNeed` under M9 duration-weighted provisioning. Without temporary mobility all demand is home demand.

The identities reconcile exactly:

`need = supplied + unmet`

`stockAfterHarvest = stockBeforeRegeneration + regenerated - supplied`

and cell totals reconcile to the period totals.

## Household supply distribution

Rather than retain every household allocation, each period records the count of households with positive need in fixed supplied-fraction bins: 0-249, 250-499, 500-749, 750-999 and 1000 permille. This distinguishes broad mild shortfall from concentrated severe shortfall without creating a household event log.

## Condition distribution

Each period records compact living-condition distributions twice: immediately after the deterministic resource response and again after the competing M2/M3 mortality boundary. The summaries include living count, mean, min/max, p10/p25/median/p75/p90, and counts below 250/500/750 permille. Quantiles use the deterministic lower order statistic at `floor((n-1)*p)`.

Individual longitudinal condition trajectories are intentionally not retained in v1. They remain explicitly unavailable because the audit requirement can be met with compact distribution histories and retaining person-level time series would have materially larger storage/privacy-like analysis costs.

## Scarcity duration and intensity

Spatial observability schema v4 exposes the exact period rows plus a compact temporal summary: preserved period count, periods with unmet need, longest consecutive scarcity run, cumulative unmet need and maximum single-period unmet need. This makes chronic mild scarcity distinguishable from acute scarcity even when cumulative totals or terminal state are similar.

## Legacy resumes

New simulations mark resource-period history as complete from the start. Legacy checkpoints deserialize with an empty history and `historyCompleteFromStart = false`; subsequent observations are still retained, but spatial observability emits an explicit warning that the pre-checkpoint trajectory is unavailable. Missing history is never fabricated.

## Scientific boundary

These records are observability only. They do not change M3 allocation, condition response, M2/M3 competing mortality, M4/M9 behavior, random streams, scientific state digests, or `MODEL_SEMANTICS_ID`.
