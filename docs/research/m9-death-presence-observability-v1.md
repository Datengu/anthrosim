# M9 death-time physical-presence observability v1

## Purpose

This contract addresses the research-observability gap tracked in #218 without changing authoritative mortality or temporary-mobility dynamics.

A death event preserves the deceased person's persistent residence cell. Under M9, persistent residence is not necessarily the person's physical-presence context at the mortality boundary. A household may instead be outbound in transit, visiting the temporary destination, or returning in transit.

For spatial mortality interpretation, those concepts must remain separate.

## Derivation boundary

`research-death-presence.py` deterministically replays the authoritative event log in exact sequence order and reconstructs the M9 presence phase that applies at each death event.

The report distinguishes:

- `at_residence`;
- `outbound_transit`;
- `visiting`;
- `return_transit`.

Persistent residence is always retained separately as `persistentResidenceCell`.

When the reconstructed phase is `visiting`, `physicalCell` is the authoritative temporary destination. When the phase is `at_residence`, `physicalCell` is the persistent residence. Transit deliberately has `physicalCell = null`; M9 v1 has no within-route cell-level physical state and the analysis must not invent one.

## Standard run-directory integration

For M9 research use, prefer the run-directory integration command:

```text
python scripts/research-m9-death-observability.py derive --run-dir RUN_DIR
python scripts/research-m9-death-observability.py verify --run-dir RUN_DIR
```

This preserves the per-death `death-presence.json` report beside the authoritative run artifacts and also writes `m9-death-observability.json`, which summarizes deaths by persistent residence, physical-presence state, represented physical cell, and provisioning context. Where ordinary spatial/temporary observability files are present, the integration cross-checks them against the same run identity and residence-attributed mortality counts.

The integrated contract is documented in `docs/research/m9-death-observability-integration-v1.md`.

## Same-day ordering

The derivation processes authoritative records by canonical event sequence, not merely by day.

This matters at shared boundaries. Under the current competing-mortality scheduler, mortality is resolved before M9/M4 work due later on the same boundary. A death before a same-day `temporaryJourneyArrived` therefore remains `outbound_transit`; a later death after that arrival event would be `visiting`.

## Resource-provisioning context

The report also exposes `resourceProvisioningAttribution`:

- `visitor_destination` while visiting;
- `persistent_residence` while at residence or in transit.

This matches the current M9 resource-attribution convention and is useful context for condition-mediated deaths. It is **not** a mortality-cause claim. In particular, authoritative `condition_mediated` mortality may reflect multiple explicit upstream causes, so a visitor/home provisioning label must not be interpreted as proof that local resource pressure caused the death.

## Fail-closed reconciliation

The derivation rejects event histories that cannot support an unambiguous reconstruction, including:

- non-canonical event sequence numbers;
- decreasing event days;
- non-authoritative records in the supplied authoritative log;
- a second departure while a household already has an active M9 journey;
- arrival/return/completion without an active journey;
- journey identity changing mid-journey;
- arrival destination conflicting with departure;
- permanent migration during an active M9 journey;
- death/departure/completion residence conflicting with already reconstructed persistent residence.

The output receives a deterministic SHA-256 content identity and `verify` requires exact re-derivation.

## Scientific interpretation boundary

This is derived observability, not a new mortality model and not a claim to know an exact historical death location.

The report supports statements such as:

- the deceased was persistently resident in cell X but the household was visiting cell Y at the mortality boundary;
- the deceased belonged to an outbound or return-transit household, for which no exact physical cell is represented;
- residence-attributed mortality counts should not automatically be mapped as physical death locations in M9-enabled studies.

It does not provide:

- within-route transit coordinates;
- a causal decomposition of condition-mediated mortality;
- archaeological visibility of death or burial;
- evidence that the deceased individual was historically present at a reconstructed place.

Those claims require separate model/evidence support.

## Research-use rule

Before an M9-enabled study interprets mortality spatially or links deaths to local archaeological/resource conditions, derive and preserve this report (or an equivalent provenance-bound representation) alongside the authoritative run artifacts. Residence-only mortality summaries remain valid when the study explicitly limits interpretation to persistent residence rather than physical death location.
