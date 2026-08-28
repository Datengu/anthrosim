# Dynamic target-arrival scheduling semantics v1

## Purpose

This contract closes issue #197 for M9 `TargetArrivalDay` triggers.

A target-arrival trigger declares a desired arrival day. Its departure day is derived from the household's **current persistent residence** and the travel resolution for that residence. Because ordinary M4 migration may change persistent residence before the target day, a residence-dependent inability to depart at simulation start is not automatically permanent.

## Re-evaluation rule

For an unprocessed target-arrival trigger:

- if the current residence implies a future departure day, schedule evaluation on that departure day;
- if the current residence is unreachable, inside the focal region, or implies a departure that has already passed while the target day is still ahead, keep the trigger pending until the target boundary;
- if the current residence implies a departure before simulation day 0, likewise keep the trigger pending until the target boundary rather than consuming it on day 0;
- after any intervening M4 residence change, ordinary boundary scheduling recomputes the trigger from the new residence, so a newly feasible future departure can occur;
- no daily polling or new mutable scheduler state is introduced.

This is deterministic because re-evaluation uses only the authoritative current residence, immutable M9 program/travel table, current simulation day, and processed-trigger state.

## Final outcomes

Deferral is not automatic participation. At the eventual evaluation boundary:

- a still-reachable residence whose required departure is before day 0 produces `DepartureBeforeSimulationStart`;
- a non-negative required departure that is already earlier than the current day produces `DepartureWindowMissed`;
- an unreachable residence produces `Unreachable`;
- residence in the focal region produces `ResidenceInRegion`;
- a valid future departure created by an intervening residence change is evaluated on that newly calculated departure day and may start normally.

The existing temporary-mobility observability surface already reports `notStartedDepartureBeforeSimulationStart` and `notStartedDepartureWindowMissed` separately. A dynamically reconsidered trigger that becomes feasible is represented by its actual journey start rather than by a false pre-start rejection.

## Checkpoint/resume

No new scheduler field is required. A trigger remains reconsiderable precisely because it remains absent from the authoritative `processedTriggers` set. That state is already checkpointed and included in the temporary-mobility digest, so checkpoint/resume during the pre-trigger interval preserves reconsideration exactly.

## Scientific boundary

This change affects authoritative M9 journey outcomes when residence changes before a target-arrival event. It therefore advances `MODEL_SEMANTICS_ID` from v16 to v17.

It does not change:

- M4 migration decision semantics;
- M9 travel-cost or travel-duration calculation;
- focal-region membership;
- journey event schemas;
- temporary resource attribution;
- RNG consumption or stream definitions.

The change removes an unintended dependence on obsolete day-zero residence while preserving explicit fail-closed outcomes when no valid departure becomes possible.
