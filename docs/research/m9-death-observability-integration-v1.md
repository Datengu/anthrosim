# M9 death-observability integration v1

## Purpose

This integration closes the remaining observability boundary in #218 after the per-death reconstruction introduced by `anthrosim-death-presence-report-v1`.

M9 mortality must be interpretable through two distinct spatial concepts:

- **persistent residence** — the authoritative residence cell recorded on the death event and used by residence-based demographic summaries;
- **physical-presence context at the mortality boundary** — `at_residence`, `outbound_transit`, `visiting`, or `return_transit`.

These concepts must not be silently collapsed into one location.

## Standard run-directory workflow

For a completed or preserved run directory containing authoritative `events.json` and `checkpoint.json`:

```text
python scripts/research-m9-death-observability.py derive --run-dir RUN_DIR
python scripts/research-m9-death-observability.py verify --run-dir RUN_DIR
```

The derive command writes two deterministic downstream artifacts beside the authoritative run files:

- `death-presence.json` — the per-death reconstruction from `anthrosim-death-presence-report-v1`;
- `m9-death-observability.json` — an integrated summary and cross-check surface.

Derivation is fail-closed: an existing differing output is not overwritten.

## Integrated summary

`m9-death-observability.json` reports:

- total deaths;
- deaths by physical-presence state;
- deaths by persistent residence cell;
- deaths by represented physical cell;
- transit deaths for which no physical cell is represented;
- resource-provisioning attribution at the mortality boundary.

The per-death companion artifact remains authoritative for individual death context. The integrated report is a derived summary for analysis and consistency checking.

## Spatial-observability reconciliation

When `spatial-observability.json` is present, the integration requires its residence-attributed death counts to match the persistent-residence counts reconstructed from authoritative death events exactly.

This establishes the intended relationship:

```text
spatial-observability death cells == persistent residence attribution
```

It does **not** reinterpret those cells as physical death locations.

Physical death-location interpretation must instead use `death-presence.json` / `m9-death-observability.json`.

## Temporary-observability reconciliation

When `temporary-observability.json` is present, its run-state digest and model-semantics identity must match the authoritative checkpoint used for death reconstruction.

This prevents a physical-presence report from being combined with temporary-mobility observability from a different run or semantics version.

## Transit boundary

M9 v1 deliberately has no authoritative within-route cell state. Therefore:

- `at_residence` deaths have the persistent residence as their represented physical cell;
- `visiting` deaths have the authoritative temporary destination as their represented physical cell;
- `outbound_transit` and `return_transit` deaths have no represented physical cell.

The integration reports transit deaths explicitly rather than inventing route coordinates.

## Resource-pressure interpretation

`resourceProvisioningAttribution` identifies whether the household was provisioned through persistent residence or the visitor destination at the mortality boundary. It is contextual information only.

It must not be interpreted as proof that local resource pressure caused a `condition_mediated` death. Mortality-cause interpretation remains governed by the authoritative mortality model and its own provenance.

## Research-use rule

For M9-enabled work that maps mortality spatially or relates mortality to local archaeological/resource conditions, preserve and verify these derived artifacts with the authoritative run directory.

Residence-only demographic analysis may continue to use residence-attributed death counts, but it must state that those counts are persistent-residence quantities rather than physical death-location observations.

This integration changes downstream observability only. It does not alter M1-M9 execution, RNG consumption, checkpoint state, mortality probabilities, temporary-mobility scheduling, or `MODEL_SEMANTICS_ID`.
