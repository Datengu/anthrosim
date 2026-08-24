# Temporary mobility and aggregation semantics v1

**Milestone:** M9.0  
**Status:** implementation contract  
**Scientific status:** synthetic/null-model semantics; unvalidated for any real population  
**Motivating capability:** temporary, reversible household mobility to a declared focal region while preserving persistent residence

## 1. Purpose

M9 adds one missing model capability:

> A household can retain a persistent residence while its living members temporarily travel to a declared focal region, remain there for a bounded duration, and return, with travel, presence and resource consequences recorded separately from permanent migration.

This contract freezes the scientific/software meaning required before authoritative M9 state is implemented. It does not encode a named site, a known historical destination, or a preferred archaeological interpretation.

The initial M9 model is deliberately a **temporary-mobility null model**. A configured journey schedule is an experimental condition, not a claim that the household is travelling for trade, ritual, refuge, politics, warfare, livestock management or any other motive.

## 2. Non-negotiable distinctions

### Residence

A household has one authoritative **residence cell**. Residence represents its persistent home/base for the purposes of the M9 null model.

Residence changes only through an explicit permanent-residence process such as M4 household migration. Temporary travel must never silently change residence.

At initialization, every founder household's residence is its existing initialized household location.

### Physical presence

Physical presence answers where the household's living members are for occupancy and temporary-mobility purposes. M9 v1 keeps the household as the movement unit: all currently living members of a household share one temporary-mobility state.

The minimum presence states are:

- **at residence** — physically present at the residence cell;
- **outbound transit** — away from residence and not yet present at the focal-region destination cell;
- **visiting** — physically present at the selected destination cell inside the focal region;
- **return transit** — away from the destination and not yet back at residence.

Transit is a real authoritative mobility state but is **not** a claim that AnthroSim knows the exact en-route campsite or cell on every travel day. M9 v1 may compute an authoritative route/cost for travel semantics without promoting every route cell into an occupied settlement/camp.

### Permanent migration

`HouseholdMigration` continues to mean permanent relocation. It changes the household residence and, when the household is at home, its physical presence.

M9 must not implement a temporary visit by emitting a permanent migration to the destination followed by another migration home.

A household with an active temporary journey is not eligible for M4 permanent-migration evaluation. If a configured temporary departure and an M4 migration decision could occur on the same day, the temporary departure takes precedence and permanent migration is deferred until the household has returned.

### Temporary journey

A temporary journey is a bounded reversible lifecycle with stable identity. At most one temporary journey may be active for a household at a time in M9 v1.

A completed journey preserves the same residence that existed at departure unless a future model explicitly introduces a different scientifically documented interaction. M9 v1 does not permit residence changes while a temporary journey is active.

## 3. Focal-region semantics

A **focal region** is an immutable, identity-bearing set of one or more valid world cells supplied as experiment input. It says where temporary visitors may aggregate; it does not say why that place is important.

The core model-facing region contract should contain, at minimum:

- a stable region identifier;
- a sorted, duplicate-free set of member cells;
- deterministic content identity/digest;
- provenance linking when the region was derived from external evidence.

Synthetic validation may declare a region directly as a cell set.

Evidence-grounded spatial runs should normally derive the same core region contract from an externally prepared normalized landscape mask. A binary auxiliary layer (`0` outside, `1` inside, explicit nodata) is the preferred first binding because it is inspectable and does not turn AnthroSim into GIS software.

QGIS/GDAL or other mature external tooling remains responsible for drawing/editing polygons, reprojection, rasterization and other generic geospatial preparation. AnthroSim owns validation, identity and the scientific meaning of the consumed region binding.

M9 v1 requires one configured focal region per temporary-mobility mechanism instance. Support for interacting networks of multiple destinations is not required for M9 completion.

Households whose residence is already inside the focal region are treated as residents, not temporary visitors, for that region and are not assigned a degenerate visit to their own region.

## 4. Journey timing

All M9 authoritative times are integer simulation days.

For one journey:

- `departure_day` is the instant the household leaves residence;
- `outbound_travel_days` is a non-negative integer derived by the declared travel model;
- `arrival_day = departure_day + outbound_travel_days`;
- the visiting interval is `[arrival_day, return_departure_day)`;
- `return_departure_day = arrival_day + stay_duration_days`;
- `return_travel_days` is derived by the same declared travel model unless a later explicit asymmetric model says otherwise;
- `completion_day = return_departure_day + return_travel_days`;
- after completion the household is again physically present at its residence.

`stay_duration_days` must be at least one day for a M9 visit.

The interval convention is half-open so a five-day stay contributes exactly five visitor-days regardless of boundary coincidences.

M9 should use event-driven scheduling between existing resource/demographic boundaries rather than iterating every person through every simulation day. The scheduler may advance directly to the next scientifically meaningful boundary while preserving exact integer-day duration accounting.

### Generic triggers

M9 v1 triggers are exogenous experiment inputs. They must make the timing meaning explicit, for example whether a configured event day is a **departure day** or a **target arrival day**.

The controlled M9.7 aggregation benchmark should use a common target-arrival interpretation so households at different travel distances can depart at different times and aggregate during the same declared window.

A later refuge/threat model may legitimately use a common departure/alert day, but M9 must not encode that motive merely to complete temporary mobility.

## 5. Participation and eligibility

M9 v1 may support deterministic all-household participation and/or an explicit participation probability/share for experiments. Any stochastic participation must use its own named deterministic RNG stream and stable household-ID evaluation order.

At minimum, a household is ineligible to start a new journey when:

- it has no living members;
- it already has an active temporary journey;
- its residence is inside the target focal region;
- no valid destination/path can be resolved under the declared travel model;
- the calculated departure would occur before the start of the simulation or otherwise outside the configured event semantics.

Ineligibility must be observable as an explicit outcome/count rather than silently converted into participation.

M9 v1 does not add social imitation, prestige, kin obligation, coercion or utility-based motivation for participation. Those are separate hypotheses for later milestones if required.

## 6. Destination and travel-cost semantics

M9 needs authoritative travel time/cost because distance to a gathering place is part of the experimental mechanism. It does **not** need a general GIS routing product.

The M9.4 implementation must use the model-facing grid and existing `movement_cost` values to resolve a deterministic minimum-cost connection from a residence cell to the focal region.

The required semantic properties are:

- four-neighbour world topology, consistent with the authoritative grid;
- non-negative deterministic edge/traversal cost derived explicitly from adjacent model-facing movement-cost values;
- the destination is a cell inside the declared focal region;
- equal-cost alternatives have deterministic tie-breaking independent of hash/map iteration order;
- the same world, region and origin produce the same destination, accumulated cost and travel duration on every supported platform;
- unreachable origins remain explicitly unreachable;
- no hidden historical route, road, entrance or preferred destination is inserted by the engine.

For M9 v1, traversal cost should be symmetric unless an explicit later mechanism introduces directional travel. The exact fixed-point edge formula and executable synthetic travel-rate parameter belong in the M9.4 implementation/research documentation and must be frozen before that code is merged.

Travel duration is conceptually:

```text
travel_days = ceil(accumulated_travel_cost / configured_travel_capacity_per_day)
```

with integer/fixed-point arithmetic only. The parameter is a model assumption with provenance, not automatically an empirical walking-speed estimate.

The calculated accumulated travel cost may also impose an explicit condition cost, but M9 must not silently reuse M4's permanent-relocation cost if its meaning does not match. Any temporary-travel condition-cost function must be separately named and documented.

## 7. Resource accounting during temporary mobility

Current M3 resource processing charges a household's whole period need to one household location. That is unacceptable for short M9 visits because a visit entirely between resource boundaries would otherwise disappear, while a visit crossing a boundary could be charged as if it lasted the whole period.

M9 v1 therefore uses **duration-weighted period attribution**.

For each resource period, the household's ordinary period need remains conserved exactly. That need is partitioned according to the number of days spent in each relevant state during the period:

- days physically at residence are attributed to the residence cell;
- days visiting are attributed to the visitor destination cell;
- outbound/return transit days are attributed to the residence cell as a synthetic **home-provisioning proxy**.

The transit rule deliberately avoids inventing en-route foraging, stores, pack animals or intermediate camps. It means that, in the M9 v1 null model, travel provisions are assumed to originate from home. This is an explicit modelling assumption and a future sensitivity dimension, not a claim about prehistoric provisioning.

Integer rounding must conserve the exact household period need. Remainders must be resolved in a stable deterministic order.

When a household draws supply from more than one cell during a period, its overall resource-satisfaction fraction and condition update must reconcile exactly with the sum of those attributed needs and allocations. M9 must not create or destroy demand because a household travelled.

When temporary mobility is disabled, the resource model should remain behaviourally equivalent to the existing M3 single-location model.

## 8. Demography boundary

M9 temporary mobility is not a mating or residence-system model. The annual M2 fertility/parentage mechanism is too temporally coarse to treat presence on one arbitrary annual boundary as evidence of a temporary social/reproductive interaction.

Therefore, for M9 v1:

- M2 locality/parent-eligibility semantics remain **residence-based**, not visitor-presence-based;
- a temporary gathering does not by itself create new reproductive/social links;
- births join the parent's household and inherit that household's residence;
- physical-presence bookkeeping for a newborn must remain internally consistent if the household is temporarily away at the annual boundary;
- deaths must remove the person from the household's current physical-presence accounting even when residence differs from presence.

Any future model in which temporary gatherings alter mating, marriage, kinship or fertility is a separate scientific mechanism and must not emerge accidentally from scheduler timing.

## 9. Authoritative state and events

M9 implementation must make temporary mobility reconstructable without inferring it from permanent migration.

At minimum, authoritative state must preserve:

- household residence;
- current temporary-mobility state;
- active journey identity when present;
- focal-region identity;
- selected destination;
- departure/arrival/return/completion timing;
- travel duration/cost needed for deterministic continuation;
- any named RNG positions required by stochastic participation/choice.

The authoritative event vocabulary should distinguish at least these transitions (exact Rust names may differ while preserving meaning):

- temporary journey started / departed;
- focal region arrived;
- return journey started;
- temporary journey completed / residence re-entered.

Each transition must identify the household, journey, relevant region/residence/destination, people affected, and model quantities required for causal inspection.

Permanent `HouseholdMigration` remains a separate event kind.

## 10. Occupancy and observability

M9 extends the existing M8.5 event-replay architecture rather than creating a second analysis engine.

Machine-readable derived observables must distinguish at least:

- permanent residents of a focal region;
- temporary visitors;
- households/people in transit;
- visit count;
- arrival and return counts;
- visit duration distribution;
- peak and mean visitor presence;
- visitor person-days;
- resident person-days;
- total person-days with resident/visitor decomposition;
- journey distance/cost/time;
- origin catchment of participating households;
- eligible but non-participating/unreachable households where applicable;
- M4 permanent migration separately from M9 temporary movement.

Transit person-days must not be silently attributed to an arbitrary world cell for spatial occupancy. They may be reported separately while transit resource demand follows the home-provisioning rule above.

The read-only Explorer may visualise these outputs later, but visualisation is downstream and never authoritative.

## 11. Scheduling and same-day ordering

M9 must preserve the existing M3/M4/M2 order when temporary mobility is disabled.

When a temporary-mobility transition shares a day with an existing model boundary, the intended ordering is:

1. settle resource demand for the elapsed resource period using the duration ledger accumulated before that boundary;
2. complete temporary transitions due that day (arrivals, return completions);
3. start temporary transitions scheduled for that day (return departures, new outward departures) in stable order;
4. evaluate M4 permanent migration only for households physically at residence and without an active temporary journey;
5. at an annual boundary, run the existing M2 demographic process using the residence-based locality rule above;
6. record/validate the resulting authoritative state.

This ordering prevents a departure on a resource boundary from retroactively changing the preceding period and prevents M4 from permanently relocating a household after it has already departed temporarily on the same day.

Implementation tests must cover same-day boundary cases explicitly.

## 12. Determinism, checkpoint and provenance rules

M9 is part of the authoritative scientific model and must satisfy the same deterministic contract as M1-M8.

Required properties include:

- all stochasticity uses named deterministic streams;
- iteration/tie-breaking order is stable and documented;
- active journeys survive checkpoint/resume exactly;
- a resumed run reproduces uninterrupted authoritative state/events/derived observability;
- region configuration, schedule, participation, travel and resource-attribution assumptions are part of immutable experiment identity/provenance;
- schema changes are explicit and fail closed when incompatible;
- implementation that changes authoritative model meaning must explicitly review and normally update `MODEL_SEMANTICS_ID` rather than hiding the change behind an unchanged semantics identity.

M9.0 itself is documentation only and does not change `MODEL_SEMANTICS_ID` or the package version.

During M9 implementation, the package remains on the current released line until the explicit v0.3.0 release-preparation phase.

## 13. Disabled-mode compatibility

Temporary mobility is opt-in.

An experiment that does not enable/configure the M9 mechanism must not acquire temporary journeys, temporary events or temporary resource attribution.

Implementation should preserve existing M1-M8 behaviour as closely as possible for disabled M9 configurations. Existing reference benchmarks must not be casually rebaselined: any unavoidable authoritative digest/schema change requires an explicit scientific/compatibility rationale and review.

## 14. M9.7 controlled benchmark contract

M9 closes with a controlled aggregation benchmark rather than with infrastructure alone.

The benchmark must include at least two regimes on the same controlled world/region and paired seeds:

1. **continuous residence** — a declared population is resident in the focal region continuously;
2. **intermittent aggregation** — households remain resident outside the focal region but visit it for bounded, repeated intervals and return.

At least one pair of regimes must be configured so their total focal-region person-days are approximately matched while their temporal structure is materially different.

The benchmark must demonstrate that AnthroSim can distinguish those regimes in machine-readable output through, at minimum:

- resident versus visitor person-days;
- peak presence;
- occupied/visited duration;
- visit/return counts;
- origin catchment and travel burden for the intermittent regime.

Acceptance also requires:

- exact deterministic replay for identical inputs;
- paired-seed ensemble execution through ordinary M7 machinery;
- checkpoint/resume equivalence, including at least one case where an annual checkpoint occurs while a journey is active;
- run-bundle/provenance validation for the new configuration/state/events;
- no use of a named historical site or archaeological interpretation in the public benchmark;
- a predeclared result/acceptance statement written before inspecting the benchmark outcome.

Passing M9.7 establishes that the software can represent and measure the two mobility regimes. It does **not** establish that either regime explains any real archaeological site.

## 15. Explicit M9 non-goals

M9 v1 does not add:

- a named archaeological site or case-study rule;
- a claim about why households aggregate;
- trade/exchange economics;
- ritual, feasting, politics or religion;
- warfare, attackers or combat;
- livestock/herd movement;
- storage or detailed carried-provision inventories;
- en-route camps as settlements;
- social-network or mating effects of gatherings;
- settlement formation/persistence as an institution;
- archaeological preservation/detection/observation modelling;
- a general GIS/routing/cartography application;
- empirical calibration of travel rate, visit duration or participation for a real prehistoric population.

Those remain possible later hypotheses only when a real experiment demonstrates that the missing capability is necessary.

## 16. M9.0 acceptance

M9.0 is complete when this contract is merged and later M9 implementation can answer the following without inventing hidden semantics in code:

- What is residence?
- What is physical presence?
- How is temporary travel distinct from permanent migration?
- What makes a focal region authoritative and reproducible?
- When does a journey start, arrive, stay, return and end?
- How is travel time/cost represented without rebuilding GIS?
- How is short-duration resource demand accounted for exactly?
- How do temporary journeys interact with M4 migration and M2 demography?
- Which state/events must survive checkpoint/resume?
- Which outputs must distinguish continuous residence from intermittent aggregation?
- What controlled benchmark must pass before M9 can be declared complete?
