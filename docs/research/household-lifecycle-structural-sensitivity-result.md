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

## Interpretation

The declared lifecycle contrast is **material for at least one predeclared household/resource/mobility observable** in this synthetic ensemble. Household lifecycle must therefore remain an explicit structural uncertainty dimension for claims that depend on household sharing, M4 permanent migration, or M9 participation. This does not establish which lifecycle is historically correct.

The fixed-founder arm's household ages are exactly the run duration by construction. Its size and generation-span distributions can be regenerated from each checkpoint with `anthrosim-household-observability`; the alternative removes that permanent founder-topology assumption and creates younger household records at annual fission boundaries.

The machine-readable aggregate used for this page is `research/household-lifecycle-sensitivity-v1/reference-result.json`.
