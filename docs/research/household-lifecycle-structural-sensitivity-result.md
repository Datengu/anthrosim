# Household lifecycle structural sensitivity — first result

**Scientific status:** synthetic structural sensitivity; not empirical household validation.

Eight paired seeds were run for 40 years. The arms differ only in household lifecycle: `fixed_founder_v1` versus `deterministic_size_fission_v1` with a maximum of 8 living members per eligible household after an annual boundary.

| Observable | Fixed founder | Size fission |
| --- | ---: | ---: |
| Completed runs | 8/8 | 8/8 |
| Extinct runs | 0/8 | 0/8 |
| Mean terminal living population | 102.12 | 102.00 |
| Mean terminal active households | 22.00 | 26.00 |
| Mean terminal largest household | 10.62 | 7.25 |
| Mean terminal multi-generational households | 16.62 | 19.00 |
| Mean terminal occupied residence cells | 21.62 | 24.12 |
| Total unmet resource need | 962 | 449 |
| Total M4 moves | 85 | 97 |
| Mean people per M4 move | 8.282 | 5.144 |
| Total M9 departures | 7336 | 8046 |
| Total M9 visitor person-days | 247348 | 246965 |
| Total M9 visitor household-days | 51317 | 56280 |
| Maximum peak simultaneous visitors | 130 | 120 |

## Pooled terminal household distributions

Counts below pool active terminal households across all eight paired seeds.

- **Living members per household — fixed:** 1: 30, 2: 16, 3: 27, 4: 23, 5: 23, 6: 20, 7: 9, 8: 8, 9: 5, 10: 4, 12: 8, 13: 3
- **Living members per household — fission:** 1: 27, 2: 27, 3: 35, 4: 36, 5: 37, 6: 24, 7: 19, 8: 3
- **Household age (days) — fixed:** 14600d: 176
- **Household age (days) — fission:** 365d: 1, 1825d: 3, 2190d: 2, 2555d: 1, 3650d: 2, 5110d: 3, 5475d: 1, 6205d: 1, 6935d: 4, 7665d: 1, 8030d: 1, 8395d: 1, 9125d: 2, 9490d: 2, 9855d: 2, 10220d: 1, 10950d: 2, 11315d: 2, 12045d: 2, 12410d: 2, 12775d: 1, 14600d: 171
- **Living genealogical generations — fixed:** 1: 43, 2: 57, 3: 71, 4: 5
- **Living genealogical generations — fission:** 1: 56, 2: 79, 3: 71, 4: 2

## Interpretation

The declared lifecycle contrast is **material for at least one predeclared household/resource/mobility observable** in this synthetic ensemble. Household lifecycle must therefore remain an explicit structural uncertainty dimension for claims that depend on household sharing, M4 permanent migration, or M9 participation/aggregation. This does not establish which lifecycle is historically correct.

M9 is specifically mixed rather than uniformly sensitive: fission changes household-level participation (departures +9.7% and visitor household-days +9.7%), while visitor person-days are nearly unchanged (-0.2%) and the maximum peak is lower (-7.7%). Thus claims about household-event frequency/grouping are structurally sensitive here, whereas this exact aggregate person-exposure measure is comparatively robust in the tested ensemble.

The fixed-founder arm's active household ages are exactly the 40-year run duration by construction. The size-fission arm instead contains multiple household ages because annual creation boundaries are now preserved authoritatively and replayable. M9 visitor person-days, household-days and peak visitors are derived through the ordinary temporary-mobility observability replay rather than counted by a special analysis path.

The machine-readable aggregate used for this page is `research/household-lifecycle-sensitivity-v1/reference-result.json`.
