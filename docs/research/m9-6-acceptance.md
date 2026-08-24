# M9.6 acceptance record

Status: implementation-complete candidate. Merge and umbrella-issue closure require the repository's full protected validation suite to pass on the exact head containing this record.

M9.6 makes the M9 temporary-mobility mechanism inspectable, resumable and usable through ordinary AnthroSim experiment/artifact workflows without changing the mechanism's authoritative meaning beyond `anthrosim-model-semantics-v5`.

## Acceptance checklist

### Authoritative state and resume

- Temporary transition events are separately versioned and remain distinct from permanent `HouseholdMigration`.
- Active journey, focal-region/program identity, resource ledger and existing RNG/state continuity are checkpointed and participate in invariant/state-digest validation.
- Core uninterrupted and active-journey-resumed execution reconcile at the authoritative scientific-state boundary.
- Transformed-spatial uninterrupted and active-journey-resumed execution reconcile at the same boundary.

### Ordinary experiment machinery

- `TemporaryMobilityConfig` is a world-independent experiment definition containing focal region, exogenous schedule and travel model.
- Each run derives its resolved M9.4 travel program from that run's own authoritative world; a seed-specific resolved table is never reused across different worlds.
- Run, Ensemble, Sweep and the standalone transformed-landscape runner accept the same versioned temporary-mobility definition.
- Ensemble and sweep identity/retry semantics include the definition through the existing immutable experiment settings.
- Multi-seed acceptance re-derives each persisted resolved program from that run's stored world.

### Machine-readable observability

`TemporaryMobilityObservabilityReport` is downstream/derived and keeps the following distinct:

- persistent residence;
- physical presence at residence;
- temporary visitor presence;
- outbound transit;
- return transit;
- permanent M4 migration.

The report exposes starts and explicit non-start reasons, arrivals, return departures, completions, active/terminated journeys, visit-duration distributions, visitor household/person-days, peak and mean visitor presence, travel duration/cost, derived route edge distance when it reconciles with authoritative M9.4 routing, and origin catchment.

Required accounting identities are fail-closed:

- persistent-residence person-days equal total living person-days;
- outbound plus return transit person-days equal total transit person-days;
- at-residence plus visitor plus transit person-days equal total living person-days.

Transit is never assigned an arbitrary world cell because M9 v1 does not contain authoritative per-day en-route locations.

### Deterministic regeneration and artifacts

- Repeated report derivation from the same authoritative artifacts is exactly equal.
- Uninterrupted and checkpoint-resumed executions regenerate exactly equal reports even though resume-lineage provenance itself may differ.
- `anthrosim-temporary-observability run` derives or verifies one report.
- `anthrosim-temporary-observability tree` processes nested experiment/sweep run trees.
- Completed bundle validation accepts the optional derived report only when provenance matches and deterministic regeneration is exactly equal; deliberate report tampering is rejected.
- A paused-run regression with only `resume-start-population.json` validates that artifact, reconstructs the day-zero founder population from immutable experiment identity plus the authoritative world, derives the report and verifies it exactly.
- The existing deterministic bundle packer includes `temporary-observability.json` through the ordinary optional-artifact path rather than a separate M9 archive format.

### Read-only Explorer

After machine-readable semantics were established, the Explorer gained optional read-only support for `temporary-observability.json`:

- it validates report provenance and exact person-day accounting for display;
- it surfaces a compact M9 summary and the authoritative temporary-event family;
- it keeps the existing residence map semantics unchanged;
- it does not invent transit locations or make the Explorer authoritative simulation state.

## Scientific scope

M9.6 establishes observability and experiment integration for the declared M9 v1 temporary-mobility model. It does not establish a historical reason for aggregation, a named-site interpretation, empirical calibration, or an archaeological preservation/detection model.

M9 remains in progress after M9.6. M9.7 must exercise the completed capability in the controlled continuous-residence versus intermittent-aggregation benchmark before milestone-level acceptance and the subsequent v0.3.0 audit/release process.
