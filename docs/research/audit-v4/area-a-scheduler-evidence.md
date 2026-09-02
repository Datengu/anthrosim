# Audit v4 Area A — scheduler/collision adversary

Target: immutable `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / `anthrosim-model-semantics-v25`.

This is fresh Audit-v4 discovery evidence, not completion evidence inherited from Audit v2/v3.

## Adversarial hypothesis

The two authoritative hosts could drift in same-day scientific process ordering, or the merged M3/M4 fixed clocks could skip/duplicate a boundary when integer-partition schedules collide. Either failure could make a supported conclusion depend on host choice or scheduler bookkeeping.

## Attack

`area-a-scheduler-adversary.py`:

1. exhaustively enumerates all `365 × 365 = 133,225` supported M3/M4 period-count pairs;
2. reconstructs the fixed boundary sets exactly with integer arithmetic;
3. asserts merged dispatch is the ordered set union, processes each configured resource/migration boundary exactly once, and reports exact collision counts;
4. asserts day 365 is a real resource+migration collision for every enabled pair, which is also the annual M2 boundary;
5. independently source-inspects both `Simulation` and `SpatialLandscapeSimulation` and requires the same scientific marker order: pre-day temporary transitions → M3 resource processing → resource-period completion → M4 migration → annual M2 demography.

The dedicated workflow runs the checker on the evidence PR. The PR is audit-only and is not intended to merge into production merely to preserve a passing result.

## Interpretation rule

A pass falsifies this specific scheduler-drift/collision hypothesis on v0.3.4. It does not by itself complete Area A; scheduler semantics, simultaneous competing processes, tie-breaking, and neighbouring v25 repairs still require separate adversarial inspection.
