# M9.7 controlled continuous-residence vs intermittent-aggregation benchmark v1

**Milestone:** M9.7  
**Status:** predeclared benchmark contract — criteria frozen before benchmark-result inspection  
**Scientific status:** synthetic validation only  
**Benchmark ID:** `m9_7_controlled_continuous_vs_intermittent_v1`

## Question

Can AnthroSim represent and machine-readably distinguish continuous focal-region residence from bounded repeated temporary aggregation when aggregate focal-region person-days are intentionally kept close?

This benchmark closes the M9 implementation programme by testing the capability defined in `docs/research/temporary-mobility-v1.md`. It does not test a named archaeological site and does not assign any social motive to aggregation.

## Controlled design

Both arms use the same eight paired seeds (`9701`–`9708`) and the same:

- 10 × 10 synthetic world;
- 2,000 synthetic founders in target households of five;
- 10-year observation duration;
- synthetic demographic mechanism;
- resource productivity scale of 1000 permille;
- seasonal-amplitude scale of zero;
- annual resource need of one abstract unit per living person;
- disabled M4 permanent migration;
- 70-cell synthetic focal region (`CellId` 1–70);
- M9.4 synthetic-validation travel model.

The deliberately low resource demand and disabled permanent migration isolate the M9 presence distinction. The benchmark must record any **condition-mediated death** or permanent migration as a failure of the intended controlled conditions rather than silently accepting a confounded comparison. Under model semantics v10, this death criterion is intentionally causal-neutral: it checks that the shared condition-mortality pathway stayed inactive, not that a death was or was not uniquely caused by food scarcity.

Founder household residences remain whatever the existing deterministic `SyntheticValidationV1` initializer produces. No benchmark-specific population-placement rule is added. The continuously resident focal population is therefore the deterministic set of households whose initialized persistent residence lies inside the predeclared 70-cell region. This set is fixed by the paired seed and is identical between arms.

## Arm A — continuous residence control

The control uses the same focal-region and travel-model contract, but its sole target-arrival trigger is day 4000, outside the 10-year (3650-day) observation window.

Therefore:

- households resident in the focal region contribute continuous resident person-days;
- no M9 temporary journey should start;
- visitor person-days and peak visitors should be zero.

Keeping an M9 definition attached to the control lets the same downstream temporary-observability report measure resident person-days under the exact same focal-region semantics as the treatment.

## Arm B — intermittent aggregation

Outside-resident households target common arrival days:

`350, 715, 1080, 1445, 1810, 2175, 2540, 2905, 3270`

Each visit lasts exactly 30 days. M9 target-arrival semantics allow households at different travel distances to depart on different days while sharing the same aggregation window.

There are nine non-overlapping 30-day windows, so the predeclared scheduled visitor-window total is 270 days over the 3650-day observation period.

The first window is `[350, 380)`. The annual checkpoint at day 365 therefore occurs while eligible outside-resident households are visiting. This gives the benchmark a deliberate active-journey checkpoint/resume case.

## Why aggregate focal use should be approximately matched

The focal region contains 70% of world cells. Synthetic household residence initialization is uniform over world cells, so the expected continuously resident share is approximately 70% and the expected outside-resident share approximately 30%.

The control's focal resident exposure is therefore approximately:

`0.70 × 3650 = 2555 person-days per average living-person equivalent`

The treatment adds approximately:

`0.30 × (9 × 30) = 81 visitor person-days per average living-person equivalent`

The added exposure is roughly 3.2% of the continuous resident exposure. The benchmark does **not** require an exact theoretical proportion; it measures the realized paired-seed person-days from authoritative event replay. Each paired treatment must remain within 5% of its control's total focal-region person-days.

This deliberately trades a very small aggregate-use difference for a large temporal-structure difference. During aggregation windows, approximately 30% of the population can be added temporarily to a resident base of approximately 70%, implying an expected visitor peak around 43% of the continuous resident mean. The predeclared minimum is much weaker: 25%.

## Authoritative and derived measurements

For each completed run, `anthrosim-temporary-observability` must regenerate `temporary-observability.json` from the preserved world, initial-population provenance and checkpoint/event history.

The benchmark analysis records at minimum:

- focal-region resident person-days;
- visitor person-days;
- total focal-region person-days = resident + visitor person-days;
- at-residence and transit person-days;
- peak visitors and mean visitors;
- days with any visitor presence, independently replayed from authoritative events and cross-checked against the M9 report;
- journey starts, arrivals, return departures and completions;
- origin-catchment cells;
- travel days, accumulated travel cost and route-edge distance;
- explicit not-started/unreachable outcomes;
- permanent M4 migration count separately;
- condition-mediated deaths separately.

Transit is never assigned to a synthetic occupied cell.

## Predeclared acceptance rule

The benchmark is classified **`capability_distinguished`** only if all of the following hold:

1. all eight paired control and treatment runs reach the configured 10-year duration;
2. each paired seed has identical authoritative world and initial-population artifacts across arms;
3. paired focal-region resident person-days are exactly equal;
4. the control has zero temporary journeys, zero visitor person-days and zero peak visitors;
5. the treatment has positive visitor person-days and at least one completed journey in every seed;
6. the treatment has exactly 270 days with at least one visitor in every seed;
7. treatment total focal-region person-days differ from paired control by no more than 50 permille (5%);
8. treatment peak visitors are at least 250 permille (25%) of the paired control's mean continuously resident focal population;
9. the treatment has a non-empty origin catchment and positive travel burden in every seed;
10. neither arm records a permanent M4 migration or condition-mediated death;
11. an identical-input duplicate execution reproduces the same authoritative state digest, events and temporary-observability report;
12. the day-365 checkpoint contains at least one active temporary journey;
13. resuming that active checkpoint to year 10 reproduces the uninterrupted run's final authoritative state digest, events and temporary-observability report.

If temporary-use structure is present but the 5% aggregate person-day bound fails, the benchmark is `near_match_failed`. If visitor structure or the 25% peak threshold is absent, it is `not_distinguished`. Missing/invalid runs or provenance reconciliation produce `degenerate`.

A negative or fragile result must be recorded as obtained. The schedule, region, thresholds or seeds must not be tuned after result inspection to force a pass. Any later benchmark revision requires a new benchmark ID/version and an explicit rationale.

## Interpretation boundary

Passing this benchmark demonstrates that AnthroSim can represent, preserve, replay and distinguish continuous residence from temporary aggregation under a controlled synthetic setup. It does **not** establish that either regime is a correct explanation for any real settlement, enclosure, landscape or archaeological deposit.
