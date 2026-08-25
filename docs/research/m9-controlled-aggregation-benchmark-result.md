# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `d0986a833e68a3682e831a2ed1b9ffea174f7a9d`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

## Current regression reference after M2 demographic-time repair

The M2 demographic-time repair changed authoritative population trajectories and advanced the global model identity to `anthrosim-model-semantics-v6`. The frozen M9.7 experiment was therefore rerun unchanged. The old exact values were not treated as calibration targets.

The reviewed semantics-v6 reference execution is GitHub Actions workflow run `32895255573`, generated from branch head:

`4480f062d9bee25b49f1ac4acda31a3c7a313e5c`

The pull-request merge-ref build for that execution was:

`add0152a9c6a14f263ae123eccbccf2f08c6bf74`

The complete uploaded artifact (`9581088637`) had SHA-256:

`6c277db5e9b6cb4971556aa8c4f8c462f8bc02c2e04e17820f4d80acb4d4df32`

The full aggregate had canonical SHA-256:

`e24e17c99d17ee9456b8eef0ad5ac577c9e2e99e2f521abc9817666d304f3215`

The current machine-readable reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`; the original first observation remains preserved in Git history.

### Semantics-v6 result

The capability conclusion survived the upstream M2 repair:

- all **8/8** paired seeds passed every predeclared paired criterion;
- paired resident person-days remained exactly equal between the continuous and intermittent arms;
- treatment total focal-region person-days differed from the paired control by about **2.8–3.6%**, still below the predeclared 5% ceiling;
- treatment peak visitors remained about **38.7–49.4%** of paired control mean resident focal population, still above the 25% floor;
- every treatment run again had exactly **270 days** with visitor presence;
- every continuous control again had zero temporary journeys and zero visitor person-days;
- treatment runs recorded approximately **988–1,188 completed journeys**;
- treatment origin catchments covered **29–30 cells**;
- travel burden remained positive in every treatment run and no household was classified unreachable;
- neither arm recorded permanent M4 migration or resource-scarcity death.

The aggregate medians under semantics v6 are:

- median total focal-person-day difference: **31 permille**;
- maximum paired total focal-person-day difference: **36 permille**;
- median intermittent peak-visitor share: **426 permille**;
- minimum intermittent peak-visitor share: **387 permille**.

The workflow again passed the independent non-statistical verification gates before the old exact reference was rejected:

- duplicate intermittent replay was byte-exact;
- the active annual checkpoint contained genuine in-progress temporary journeys;
- resumed authoritative state and temporary observability exactly matched uninterrupted execution.

Thus the changed exact totals are downstream consequences of the corrected demographic trajectory, not evidence that M9 replay/resume or residence-versus-presence semantics became unstable.

## Original first observation

The original first workflow run was `32785683492`, with first-result artifact `9541411806` (`sha256:a09a051c3d92d4755c1efd48e91758fb0fe522d39a06413926ba1a4af9b017f4`). Its classification was also `capability_distinguished`.

That first execution satisfied every predeclared paired-seed criterion across all eight seeds. The intermittent arm added a sharply bounded temporary aggregation signal while keeping aggregate focal-region use close to the continuous-residence control:

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

The original workflow also proved duplicate replay and active annual checkpoint/resume equivalence. Those exact first-observation values remain historical evaluation evidence, but they are no longer the current regression reference after the M2 model definition changed.

## What the current result establishes

This remains a capability result. Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve the difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

The semantics-v6 rerun adds a useful stronger statement about software/model robustness: the M9 capability distinction survived a consequential upstream correction to demographic timing even though exact population and presence totals changed.

That does **not** establish invariance to all plausible demographic models. It shows this one predeclared M9 capability classification remained intact under the repaired M2 implementation.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence, or any particular social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation remains a separate research task and would require evidence-grounded experiment design, uncertainty propagation, structural sensitivity, calibration/corroboration separation and appropriate domain review.

The rebaseline is verification/reference maintenance after a declared model-semantics change. It is not empirical calibration.