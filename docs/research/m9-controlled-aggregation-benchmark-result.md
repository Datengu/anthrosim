# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `d0986a833e68a3682e831a2ed1b9ffea174f7a9d`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

## Current regression reference — model semantics v8

The v8 M3 resource-time repair changed authoritative resource execution and therefore required the frozen M9.7 experiment to be rerun unchanged. The old exact state identities were not treated as calibration targets.

Reviewed execution:

- workflow run: `32917412358`;
- branch head: `7e13d5ee82db0c65d5ac52e4e5501c812fc968b0`;
- pull-request merge-ref build: `bdee1f2831d8c18a9798acc5756cc10d21df1d04`;
- artifact: `9588720942`;
- artifact SHA-256: `1d0616edcd8c36c3c3c214ddd2efa0fa6d8f0133d14e65b128c3bb9544b86696`;
- aggregate canonical SHA-256: `30a9bc5e19c47f90290a3aab204ef18ab5b9754b0233d086f92e47aad678ba76`.

The current machine-readable reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`.

### v8 result

The predeclared capability result is unchanged:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment again has exactly **270 days** with visitor presence;
- intermittent treatments complete roughly **988–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or resource-scarcity death.

The aggregate values are exactly unchanged from the previous preserved capability result:

- median total focal-person-day difference: **31 permille**;
- maximum paired total focal-person-day difference: **36 permille**;
- median intermittent peak-visitor share: **426 permille**;
- minimum intermittent peak-visitor share: **387 permille**.

The workflow also passes the non-statistical replay gates before reference comparison:

- duplicate intermittent replay is exact;
- the active annual checkpoint contains genuine in-progress temporary journeys;
- resumed authoritative state and temporary observability exactly match uninterrupted execution.

This is an informative negative sensitivity result for this particular upstream repair. Although v8 changes state/provenance identities, its corrected resource clock does **not** change the predeclared M9 capability distinction in this fixture. The benchmark disables permanent migration, records zero resource-scarcity deaths in both arms, uses zero seasonality scaling, and both arms share the same corrected resource semantics; therefore the paired temporary-presence result remains identical.

## Historical references

The first M9.7 observation and the later v6 demographic-time rebaseline are preserved in Git history. The v6 M2 repair changed exact population/presence totals while retaining `capability_distinguished`. The v8 rerun is stronger in a different way: the paired scientific metrics themselves reproduce exactly even though experiment and terminal-state identities change with the global model-semantics identity.

These historical references are evaluation evidence, not calibration targets.

## What the current result establishes

Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve that difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

The v8 result additionally verifies that this particular capability distinction survives the corrected M3 resource-time implementation without numerical drift in its predeclared paired metrics.

That does **not** establish invariance to all resource, demographic or temporary-mobility alternatives. It establishes only the behavior of this frozen capability benchmark under current semantics.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence or any social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation would require question-specific evidence, uncertainty propagation, structural sensitivity, calibration/corroboration separation and domain review.

The v8 reference update is reproducibility maintenance after a declared model-semantics change. It is not empirical calibration.
