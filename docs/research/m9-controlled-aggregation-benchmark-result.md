# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `d0986a833e68a3682e831a2ed1b9ffea174f7a9d`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

## Current regression reference — model semantics v10

The M3 condition-mortality causality repair advanced the authoritative model identity to `anthrosim-model-semantics-v10` and replaced scarcity-attributed mortality output names with general condition-mediated terminology. The frozen M9.7 experiment was rerun unchanged. Its numerical scientific result was reviewed before the reference was rebaselined; old exact identities were not treated as calibration targets.

Reviewed execution:

- workflow run: `32930245559`;
- branch head: `a329f68f3278a600dece4193b9a6179d4e981180`;
- pull-request merge-ref build: `4a224061e4f4387430a33215518503b064810a1f`;
- artifact: `9593016422`;
- artifact SHA-256: `561f828adec030fba9879b9a354f285b4b10b9f0431b16591134e539d866bb08`;
- aggregate canonical SHA-256: `4a7e4a95edbb01f0ab7371d313bd24e5f989cf2371a022ed6dd62fe4426f8a07`.

The current machine-readable reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`.

### v10 result

The predeclared capability result is numerically unchanged:

- all **8/8** paired seeds pass every paired criterion;
- paired resident person-days remain exactly equal between continuous and intermittent arms;
- every continuous control has zero temporary journeys and zero visitor person-days;
- every intermittent treatment has exactly **270 days** with visitor presence;
- intermittent treatments complete roughly **988–1,188 journeys**;
- origin catchments cover **29–30 cells**;
- travel burden remains positive in every treatment and no household is classified unreachable;
- neither arm records permanent M4 migration or **condition-mediated death**.

The aggregate values are exactly unchanged from the preserved v8 capability result:

- median total focal-person-day difference: **31 permille**;
- maximum paired total focal-person-day difference: **36 permille**;
- median intermittent peak-visitor share: **426 permille**;
- minimum intermittent peak-visitor share: **387 permille**.

Every one of the eight paired seed outcomes and every predeclared pass criterion is unchanged numerically. The v10 benchmark/reference schema is version 2 because `conditionMortalityDeaths` and `noConditionMortalityDeaths` replace the former scarcity-attributed field names. Experiment identities, authoritative state digests and aggregate identity changed as expected under the new global model semantics and wire contracts; those identity changes were reviewed rather than interpreted as scientific effects.

The workflow also passes the non-statistical replay gates before reference comparison:

- duplicate intermittent replay is exact;
- the active annual checkpoint contains genuine in-progress temporary journeys;
- resumed authoritative state and temporary observability exactly match uninterrupted execution.

This is an informative null sensitivity result for issue #200. The benchmark disables permanent migration, uses deliberately low resource demand and records zero condition-mediated deaths in both arms, so the repaired cause attribution does not alter its temporary-presence distinction. Importantly, the v10 criterion now states only what the executable evidence supports: the general shared-condition mortality pathway remained inactive. It no longer claims that a zero death count is specifically a zero resource-scarcity death count.

## Historical references

The first M9.7 observation, the later v6 demographic-time rebaseline and the v8 resource-time rebaseline are preserved in Git history. The v6 M2 repair changed exact population/presence totals while retaining `capability_distinguished`. The v8 rerun reproduced the paired scientific metrics exactly despite changed upstream resource semantics. The v10 rerun again preserves the same classification, aggregate statistics and paired outcomes while correcting mortality cause terminology and identity/schema material.

These historical references are evaluation evidence, not calibration targets.

## What the current result establishes

Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve that difference through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ordinary ensemble machinery.

The v10 result additionally verifies that this particular capability distinction is unchanged by the M3 condition-mortality cause repair. That is a regression/capability statement only; it is not empirical validation.

That does **not** establish invariance to all resource, demographic, condition or temporary-mobility alternatives. It establishes only the behavior of this frozen capability benchmark under current semantics.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence or any social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation would require question-specific evidence, uncertainty propagation, structural sensitivity, calibration/corroboration separation and domain review.

The v10 reference update is reproducibility maintenance after a declared causal/model-semantics change. It is not empirical calibration.
