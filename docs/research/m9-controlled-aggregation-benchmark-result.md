# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `d0986a833e68a3682e831a2ed1b9ffea174f7a9d`  
**First workflow run:** `32785683492`  
**First-result artifact:** `9541411806` (`sha256:a09a051c3d92d4755c1efd48e91758fb0fe522d39a06413926ba1a4af9b017f4`)  
**Classification:** `capability_distinguished`

## Result

The first execution of the frozen M9.7 benchmark satisfied every predeclared paired-seed criterion across all eight seeds.

The intermittent arm added a sharply bounded temporary aggregation signal while keeping aggregate focal-region use close to the continuous-residence control:

- all 8/8 paired seeds passed;
- paired focal-region resident person-days were exactly equal between arms;
- treatment total focal-region person-days differed from control by 2.9–3.6%, below the predeclared 5% ceiling;
- treatment peak visitors were 39.5–49.4% of the paired control mean resident focal population, above the predeclared 25% floor;
- every treatment run had exactly 270 days with visitor presence;
- every continuous control had zero temporary journeys and zero visitor person-days;
- treatment runs completed 990–1,188 temporary journeys;
- treatment origin catchments covered 29–30 cells;
- travel burden was positive in every treatment run and no household was classified unreachable;
- neither arm recorded permanent M4 migration or resource-scarcity death.

The independently replayed resident person-days, visitor person-days and peak visitors matched the M9.6 machine-readable observability report for every run.

## Determinism and resume acceptance

The workflow also exercised the non-statistical M9.7 acceptance gates on seed 9701:

- two identical intermittent executions produced byte-identical checkpoints, event logs and temporary-observability reports;
- the deliberately chosen first aggregation window `[350, 380)` left 132 households actively visiting at the annual day-365 checkpoint;
- resuming that active checkpoint to year 10 reproduced the uninterrupted run's final population, temporary-mobility state, resources, migration state, RNG positions, events, metrics, terminal state digest and temporary-observability report exactly.

## What this result establishes

This is a capability result. Under one controlled synthetic design, AnthroSim can now represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve the difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

That closes the specific M9 software question: temporary aggregation is no longer being represented as permanent relocation or as an unobservable bookkeeping distinction.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence, or any particular social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation remains a separate research task and would require evidence-grounded experiment design, uncertainty analysis and appropriate domain review.
