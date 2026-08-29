# Household lifecycle structural sensitivity — dependency-aware v2 result

**Scientific status:** synthetic structural sensitivity; not empirical household validation.

The frozen issue #207 design was rerun after the #324 repair: eight paired seeds, 40 years per arm, founder population 120, with all non-household-lifecycle assumptions held fixed. All 16 arm-runs completed and none went extinct.

The fixed-founder arm is unchanged from the historical comparison. The alternative is now `deterministic_dependency_fission_v2` (maximum 8 living members; minimum independent age 18 years), which removes stable-PersonId cohort slicing and preferentially keeps dependents with living parents when feasible.

| Observable | Fixed founder | Dependency fission v2 | v2 change vs fixed |
| --- | ---: | ---: | ---: |
| Mean terminal living population | 102.125 | 100.375 | -1.7% |
| Mean terminal active households | 22.000 | 26.750 | +21.6% |
| Mean terminal largest household | 10.625 | 6.750 | -36.5% |
| Mean terminal multigenerational households | 16.625 | 20.250 | +21.8% |
| Mean terminal occupied residence cells | 21.625 | 24.500 | +13.3% |
| Total unmet resource need | 962 | 118 | -87.7% |
| Total M4 moves | 85 | 100 | +17.6% |
| Total people moved by M4 | 704 | 500 | -29.0% |
| Total M9 departures | 7336 | 8092 | +10.3% |
| Total M9 visitor person-days | 247348 | 248074 | +0.3% |
| Total M9 visitor household-days | 51317 | 56616 | +10.3% |
| Maximum peak simultaneous visitors | 130 | 126 | -3.1% |

## Comparison with the historical v1 fission treatment

The #324 repair does not erase the structural-sensitivity conclusion. Relative to the historical `deterministic_size_fission_v1` result, v2 changes terminal living population by -1.6%, active households by +2.9%, M4 moves by +3.1%, people moved by +0.2%, M9 departures by +0.6%, visitor person-days by +0.4%, visitor household-days by +0.6%, and maximum peak visitors by +5.0%.

The largest change is resource pressure: total unmet need falls from 449 under the historical PersonId-sliced fission treatment to 118 under dependency-aware v2 (-73.7%). This demonstrates that household **composition**, not merely household count, materially affects M3 sharing outcomes. The repaired treatment therefore strengthens the reason to carry household lifecycle/composition as structural uncertainty rather than treating the original v1 result as a neutral alternative.

M9 remains mixed: household-event frequency and household-days are sensitive, while aggregate visitor person-days are nearly invariant in this ensemble. M4 also remains structurally sensitive, particularly in the number of people grouped into moves.

These results do not establish that dependency-aware fission is historically correct. They establish that the earlier stable-ID composition rule was scientifically consequential and that the qualitative #207 conclusion—household lifecycle must remain an explicit structural uncertainty dimension—survives its removal.

Machine-readable aggregate: `research/household-lifecycle-sensitivity-v2/reference-result.json`.

Artifact provenance: workflow run `33272700305`, head `72beb5ce656ebc2b55dee20c48682161e55e1403`, artifact `9720595797`, artifact digest `sha256:11079ddefe586013faf3abf67ea809f49fa4698781a9edb55a2d3f3ff8c0c2f9`.
