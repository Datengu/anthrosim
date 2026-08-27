# Temporary mobility observability v1

Status: M9.6 implementation contract. The current serialized report schema is **2**.

This document defines the first downstream, machine-readable observability contract for M9 temporary mobility. It is generic and does not assign archaeological meanings such as refuge, market, ritual centre or seasonal fair.

The report is **derived** from preserved authoritative AnthroSim artifacts. It does not participate in simulation execution, random-number streams, checkpoint state or state digests, and it must be exactly regenerable from those artifacts.

Report schema 2 separates **planned** travel burden from burden that was actually **observed or realized inside the preserved observation interval**. Schema 1 travel-duration/cost/distance aggregate names represented the complete planned round trip as soon as a journey departed; those ambiguous names must not be interpreted as realized burden.

## 1. Separation from permanent residence and migration

M9 observability must preserve three concepts as distinct quantities:

1. **Persistent residence** — the household's authoritative home cell in `Population`.
2. **Physical temporary presence** — at residence, outbound transit, visiting the focal region or return transit.
3. **Permanent migration** — M4 `HouseholdMigration`, which changes persistent residence.

The M8 spatial observability report remains residence/permanent-migration based. M9 v1 therefore uses a separate temporary-mobility report rather than changing the meaning of existing M8 metrics.

## 2. Person-day accounting

For every living household member and every simulated day in the observation interval, the report records both persistent-residence continuity and exactly one physical-presence category.

### 2.1 Persistent residence person-days

Every living person-day is attributed to the household's persistent residence cell, including periods when the household is travelling or visiting. This quantity answers where the household remained resident, not where it was physically located.

Across the whole report:

`persistent_residence_person_days == total_living_person_days`

### 2.2 Physical-presence partition

Every living person-day belongs to exactly one of:

- `at_residence_person_days` — physically at the persistent residence;
- `visitor_person_days` — physically visiting the focal-region destination cell;
- `outbound_transit_person_days` — travelling from residence to the focal region;
- `return_transit_person_days` — travelling from the focal region back to residence.

Therefore:

`at_residence_person_days + visitor_person_days + outbound_transit_person_days + return_transit_person_days == total_living_person_days`

Transit person-days are intentionally **not assigned to arbitrary world cells**. M9 v1 has authoritative transit state but not authoritative daily en-route locations.

The resource system's explicit home-provisioning proxy for transit is a resource-accounting assumption and does not convert transit into physical residence occupancy for observability.

## 3. Event replay

The report replays authoritative events in sequence order. For each household it retains:

- current persistent residence;
- current living-member count;
- current physical-presence category;
- current active journey identity and planned timing;
- last accounted simulation day.

Before applying an event for a household, the replay accrues the half-open interval from that household's last accounted day to the event day under the previous state.

Birth and death events change the household's living-member count. Their event `cell` remains the residence-based demographic locality defined by M2 and must not be reinterpreted as temporary physical presence.

A permanent `HouseholdMigration` changes persistent residence and is invalid during an active temporary journey.

Temporary journey transition events change only physical presence. If the last living member of an away household dies, replay mirrors the authoritative reconciliation rule by terminating the active temporary journey and returning the empty household bookkeeping state to `AtResidence`; this is reported separately rather than fabricating a completion event.

At the report boundary all households are accrued to the checkpoint day and replayed residence, living counts and temporary presence are reconciled against the authoritative checkpoint.

## 4. Journey and trigger outcomes

The report preserves temporary mobility as a distinct event family and exposes at least:

- scheduled trigger evaluations represented by authoritative start/not-start outcomes;
- journeys started;
- arrivals;
- return departures;
- completed journeys;
- active journeys at the report boundary;
- journeys terminated because the household lost all living members;
- not-started counts by `TemporaryJourneyIneligibility`, including explicit unreachable and other non-participation reasons.

A journey row records its authoritative household, trigger, residence, destination, timing, travel-model identity and planned route metadata where available. The row also records derived visitor/transit person-days accumulated while that journey was active, elapsed outbound/return transit days, completed route-leg count and the split between planned, realized and still-unrealized travel burden.

## 5. Visitor presence

Visitor presence is physical presence at focal-region destination cells only. Resident households whose persistent residence is already inside the focal region are not visitors merely because the region contains their residence.

The report exposes:

- visitor person-days;
- visitor household-days;
- arrivals and return departures by destination cell;
- peak simultaneous visitors by destination cell and for the focal region as a whole;
- mean focal-region visitor presence over the observation interval using deterministic integer/fixed-point representation.

Peak and mean counts use living persons, not nominal founder household size. Births and deaths while away therefore affect subsequent presence intervals.

## 6. Planned versus observed/realized travel burden

Authoritative M9.4 travel metadata supplies a **plan** when a journey starts:

- chosen focal-region destination;
- accumulated minimum one-way travel cost where the M9.4 model is present;
- planned outbound and return travel duration;
- travel-model identity.

A departure does **not** make the future return leg realized. Report schema 2 therefore uses explicit planned names for the complete route plan and separate observed/realized quantities for what has occurred by the report boundary.

### 6.1 Duration

Duration can be observed while a leg is still in progress because the lifecycle preserves household transit state over time. Journey rows therefore expose:

- `plannedOutboundTravelDays`;
- `plannedReturnTravelDays`;
- `plannedRoundTripTravelDays`;
- `observedOutboundTransitDays`;
- `observedReturnTransitDays`;
- `observedTransitDays`;
- `unrealizedPlannedTransitDays`.

Top-level summary and origin-catchment rows expose the same planned/observed distinction. At each aggregation level:

`plannedRoundTripTravelDays == observedTransitDays + unrealizedPlannedTransitDays`

An active outbound journey can therefore have some observed outbound transit days while its entire return duration remains unrealized. An active visit has realized its full outbound duration but none of its planned return duration. An active return can have a partially observed return duration. A journey terminated by household extinction preserves elapsed transit duration and leaves the remainder of the plan in `unrealizedPlannedTransitDays`.

Transit **days** are journey-duration quantities. Transit **person-days** remain the separately reported population exposure quantity and can differ when household membership changes.

### 6.2 Route cost and route distance

M9 v1 does not persist a path, per-edge progress or an authoritative daily en-route cell. The observability report must therefore never invent fractional route progress or substitute Manhattan/straight-line displacement and call it realized route distance.

For an M9.4 program, downstream observability may deterministically recompute a **minimum edge-count among the authoritative minimum-cost routes to the authoritative destination**. Cost and destination remain the primary frozen routing criteria; edge count is used only as a downstream tie-break among routes that have the same authoritative cost and destination. The recomputed cost and destination must match the authoritative travel table or report derivation fails closed.

This edge count is labelled `route_distance_edges`. It is derived, not authoritative simulation state.

Because within-leg progress is unavailable, route cost and distance become realized only at authoritative leg endpoints:

- departure: `realizedRouteLegs = 0`;
- arrival at the focal destination: outbound leg becomes realized, `realizedRouteLegs = 1`;
- return completion at residence: return leg also becomes realized, `realizedRouteLegs = 2`.

Thus an active outbound journey has zero realized route cost/distance even if some transit days have elapsed. An active visit or active return has one complete realized route leg. A journey terminated by household extinction keeps any already completed leg realized, but an interrupted in-progress leg is not fractionally fabricated.

Journey rows expose planned round-trip, realized and unrealized-planned cost/distance quantities. Summary and origin-catchment rows expose corresponding totals. For journeys whose route metadata is available:

`plannedRoundTripTravelCostUnits == realizedTravelCostUnits + unrealizedPlannedTravelCostUnits`

`plannedRoundTripRouteDistanceEdges == realizedRouteDistanceEdges + unrealizedPlannedRouteDistanceEdges`

`plannedTravelCostUnavailableJourneys` and `plannedRouteDistanceUnavailableJourneys` identify starts for which those planned route quantities are unavailable. Legacy/lower-level temporary programs may therefore report route cost/distance as unavailable rather than inventing them.

## 7. Origin catchment

Origin-catchment rows are grouped by the household's persistent residence at trigger/departure time. They expose machine-readable totals such as:

- trigger outcomes;
- starts, arrivals and completions;
- unreachable and other not-started outcomes;
- people departing;
- visitor person-days;
- transit person-days;
- planned outbound, return and round-trip travel duration;
- observed outbound/return transit duration and still-unrealized planned duration;
- planned, realized and still-unrealized route cost/distance where available.

This is a model-derived catchment, not a claim that historical people actually travelled from those cells.

## 8. Provenance

Every report records sufficient source identity to reject cross-run substitution, including:

- report schema version and derived provenance;
- model version and model-semantics identity;
- Git source identity when available;
- experiment seed;
- checkpoint end day and state digest;
- authoritative world digest;
- temporary-mobility configuration identity when configured;
- resolved temporary-mobility program identity when present;
- focal-region identity;
- travel-model identity when present.

Bundle validation regenerates the report from preserved authoritative artifacts and requires exact equality.

## 9. Interpretation limits

M9 temporary-mobility observability measures consequences of the declared model and experiment assumptions. Planned burden describes the route/timing commitment encoded when a journey starts; realized/observed burden describes only the portion supported by preserved lifecycle history through the report boundary. Neither quantity by itself establishes why people travelled, whether aggregation was ritual/economic/defensive, whether a focal region corresponds to an archaeological site, or whether such activity would be archaeologically visible after preservation and detection processes.

Those questions require evidence-grounded experiment design and, where necessary, later archaeological observation models.
