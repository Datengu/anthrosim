# M4 travel-condition loss observability v1

## Status

This document defines the research-facing meaning of M4 travel-condition cost fields introduced for issue #225. It clarifies observability only: it does not change the condition decrement applied by migration, destination choice, migration timing, or RNG consumption.

## Scientific distinction

A completed M4 relocation has two different condition quantities that must not be conflated.

### Nominal per-person decrement

`nominalTravelConditionCostPerPerson` is the configured decrement M4 requests for every living mover after the selected move distance is known:

```text
min(travelConditionCostPerCell × distanceCells, 1000)
```

It is an intended/applied decrement before the lower bound of the condition scale is considered. It is therefore not necessarily the amount a person actually loses.

### Realized household loss

`realizedTravelConditionLossTotal` is the exact sum of condition actually lost by the household's living movers after condition saturates at zero. For each mover with pre-move condition `c` and nominal cost `n`, the realized loss is:

```text
min(c, n)
```

and the per-move household total is the sum across living movers.

For example, if the nominal decrement is 100 and two movers begin at conditions 1000 and 50, the nominal request is 200 condition units in total but the realized loss is 150.

## Authoritative accounting

M4 still applies all selected household relocations simultaneously through the authoritative packed `Population` relocation pass. The per-move realized loss is calculated from the same pre-move living conditions and the same nominal decrement, then checked against the aggregate loss returned by that authoritative population mutation.

The invariant suite additionally requires the sum of `realizedTravelConditionLossTotal` over every authoritative `householdMigration` event to equal `MigrationSummary.travelConditionCostTotal` exactly.

Consequently:

- `nominalTravelConditionCostPerPerson` answers how much M4 attempted to deduct from each mover;
- `realizedTravelConditionLossTotal` answers how much condition that particular household move actually removed;
- `travelConditionCostTotal` is the all-move aggregate of realized loss, not nominal requested loss.

A recorded realized loss may never exceed `peopleMoved × nominalTravelConditionCostPerPerson`. Saturation can make it smaller.

## Preserved artifacts

Current artifacts expose both quantities explicitly under their established wire-casing conventions:

- authoritative `householdMigration` event JSON uses `nominal_travel_condition_cost_per_person` and `realized_travel_condition_loss_total`;
- retained `MigrationDecisionTrace` JSON uses `nominalTravelConditionCostPerPerson` and `realizedTravelConditionLossTotal`.

The historical ambiguous event field `travel_condition_cost_per_person` and trace field `travelConditionCostPerPerson` are not emitted by the current contract.

The schema changes are observability/persistence changes:

- `EventLog` schema v3;
- `MigrationSummary` schema v3;
- `MigrationCheckpointState` schema v2;
- migration artifact schema v2;
- `SimulationCheckpoint` schema v12, because it embeds the changed migration/event history contract;
- `RunManifest` schema v16, because it embeds the changed migration summary and artifact-schema identities.

`MODEL_SEMANTICS_ID` is unchanged because the executable migration decision and condition-transition rules are unchanged. Exact serialized histories and continuation identities legitimately change because the preserved scientific record is richer.

## Interpretation boundary

These fields do not provide a complete historical per-person condition trajectory. They preserve the exact realized condition effect attributable to each completed M4 move without inventing intermediate person-state history. Broader temporal resource/condition observability remains a separate concern under issue #215.

The current condition cost is still a synthetic model quantity. Distinguishing nominal from realized loss makes the implemented causal effect inspectable; it does not empirically validate the coefficient as an energetic, physiological, injury, or mortality model.

## Verification requirements

The implementation must cover at least:

- no saturation: every mover has condition at least as large as the nominal decrement;
- partial saturation: some movers have condition below the nominal decrement;
- full bounded depletion: the decrement reduces all available condition to zero where applicable;
- exact conservation between per-move realized losses and the aggregate M4 realized-loss total;
- deterministic replay and checkpoint/resume under the versioned artifact contract.

A consumer must never reconstruct realized condition loss by multiplying `peopleMoved × nominalTravelConditionCostPerPerson` unless it has independently established that no mover saturated at zero.
