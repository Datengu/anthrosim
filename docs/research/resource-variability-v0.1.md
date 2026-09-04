# Resource variability experiment — v0.1

**Scientific status:** synthetic validation / exploratory model exercise  
**Empirical status:** unvalidated; no claim about a real prehistoric population  
**Canonical definition:** `experiments/v0.1-resource-variability.json`  
**Preserved derived reference:** `experiments/v0.1-resource-variability-reference.json`

## Question and design

The experiment asks:

> How do synthetic resource-productivity magnitude and seasonal variability affect population persistence, spatial dispersion and migration in AnthroSim?

The canonical definition is unchanged: 100 years, 5,000 founders, 64 × 64 synthetic world, annual food need 100 abstract units/person, eight paired seeds (`71001`–`71008`), and a factorial cross of productivity `250/500/1000`, seasonality `0/500/1000`, and M4 migration enabled/disabled. This produces 18 points × 8 seeds = **144 runs**.

These are synthetic mechanism-testing settings, not reconstructed carrying capacities, climate variability or human energetics.

## Current v33 reference and provenance

Audit-v4 AV4-009 / #518 changes causal same-seed M4 candidate-choice coupling: candidate uncertainty and weighted-choice intervals are no longer assigned through arbitrary canonical `CellId` / container ordering. The accepted repair evaluates deterministic utility first, partitions scientifically equivalent candidates by `(total utility, movement distance)`, assigns uncertainty at equivalence-class level, preserves class aggregate proportional weight and samples exact-class members exchangeably. The frozen M7.6 reference was therefore rerun and reviewed rather than forced to reproduce v32 values.

Reviewed v33 execution before reference synchronization:

- CI run: `33885763026`;
- M7.6 job: `101069178448`;
- exact production head used by the archived run: `c9b7a0f2d762323afa76b7d0f390f29930a77b0a`;
- artifact: `9942318197` (`m7-6-resource-variability-derived`, `sha256:4f5abe584d9c30f5ba69e144d7c41ad3d5e9d4a9664953944685d43066166cd5`);
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v33`;
- sweep ID: `anthrosim-sweep-v2-b56e7e725c1ea70e`.

All **144/144** planned runs completed and were scientifically eligible, with no failed, incomplete, record-limit or otherwise operationally censored runs. The exact-head workflow failed only at its final equality assertion against the then-current frozen v32 point-results reference; the complete derived artifact was archived successfully and reviewed before the reference was changed.

The on/off control is exact and diagnostic for this M4-only repair: all **9/9 migration-disabled point summaries are numerically identical to v32**, while all **9/9 migration-enabled point summaries change**. Migration-enabled terminal-population means move by only about **-2.9% to +4.1%**. The source definition, 18-point factorial design, paired seeds, M3 resource settings, demographic settings, completion/censoring rules and declared endpoints are unchanged. This isolates the required rebaseline to trajectories in which permanent M4 migration can act, rather than unexplained resource or demographic drift.

The reviewed v33 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls.

## Current v33 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1672.125 | 343.875 | 278.625 | 36205.375 | 28995.125 | 2.048 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4891.25 | 548486.5 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1641.125 | 348 | 276.25 | 37098.25 | 28724.625 | 2.044 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4900.25 | 548966.25 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1659.125 | 348.5 | 289 | 39330 | 28787 | 2.047 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4909.625 | 551006.875 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1892 | 349.625 | 25.375 | 2428.125 | 7044.75 | 1.979 |
| 500 | 0 | off | 8 duration / 0 extinct | 18 | 14.375 | 3677.375 | 326601.875 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1826 | 342.75 | 24.5 | 2700.875 | 6872.375 | 1.984 |
| 500 | 500 | off | 8 duration / 0 extinct | 21.5 | 14.875 | 3705.75 | 331462.5 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1842.125 | 347.375 | 28.5 | 3330.375 | 6967.75 | 1.986 |
| 500 | 1000 | off | 8 duration / 0 extinct | 23.625 | 16.5 | 3693.625 | 335276.25 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1855.375 | 351.5 | 2.625 | 81.125 | 1124.625 | 1.889 |
| 1000 | 0 | off | 8 duration / 0 extinct | 628.75 | 224.375 | 1636.625 | 126130.25 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1870.625 | 350.75 | 2.375 | 39.875 | 1089.75 | 1.900 |
| 1000 | 500 | off | 8 duration / 0 extinct | 614.625 | 220 | 1596 | 125021.25 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1869 | 346.5 | 3.625 | 175.625 | 1120.625 | 1.924 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 628 | 221 | 1611.5 | 130377.875 | 0 | — |

Full unrounded point values are preserved in the machine-readable reference.

## Current v33 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while exact migration-enabled trajectories now reflect spatial-isomorphism-invariant M4 candidate stochastic coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison still does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v33 rebaseline preserves the experiment design and scientific question while recording the expected downstream consequences of correcting arbitrary spatial-candidate stochastic coupling. It does not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

## Historical v29 reference and provenance

Audit-v4 AV4-005 / #495 changes causal same-seed M2 parentage assignment: eligible residence-local males are no longer coupled to `demography/parentage` draws through arbitrary canonical `PersonId`/record order. Because genealogy can propagate into kin eligibility, household composition and migration histories, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v28 values.

Reviewed v29 execution:

- CI run: `33816687527`;
- M7.6 job: `100852554519`;
- exact production head: `f8c806a811d104c17f32864dd4abee2d123cfa2d`;
- artifact: `9917067903` (`m7-6-resource-variability-derived`, `sha256:202a464f058a80e7c1b788a5de9bd2f7ab30434a747524f0d7df66fcf7174f48`);
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v29`;
- sweep ID: `anthrosim-sweep-v2-ab941059524ac327`.

All **144/144** planned runs completed and were scientifically eligible, with no failed, incomplete, record-limit or otherwise operationally censored runs. The exact-head workflow failed only at its final equality assertion against the then-current frozen v28 point-results reference; the complete derived artifact was archived successfully.

The on/off control is again diagnostic: all **9/9 migration-disabled point summaries are numerically identical to v28**, while all **9/9 migration-enabled point summaries change**. The source definition, 18-point factorial design, paired seeds, M3 resource settings, completion/censoring rules and declared endpoints are unchanged. This pattern is consistent with corrected parentage/genealogy coupling becoming consequential when residence and household histories can diverge through M4 migration, rather than unexplained resource-process drift.

The reviewed v29 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls.

## Historical v29 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1618.375 | 339.5 | 281.125 | 35961.5 | 28573 | 2.045 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4896.25 | 548877.5 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1602.125 | 343.375 | 269.125 | 35746.125 | 28574.875 | 2.046 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4891.875 | 551466.75 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1633.375 | 336.875 | 292.75 | 39449.625 | 28643.5 | 2.050 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4890 | 550569 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1851.625 | 347.125 | 27.625 | 2914.75 | 7031.5 | 1.985 |
| 500 | 0 | off | 8 duration / 0 extinct | 24.375 | 15.75 | 3680.25 | 328961.875 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1834.25 | 344.375 | 26.375 | 2560.75 | 6866 | 1.982 |
| 500 | 500 | off | 8 duration / 0 extinct | 18.75 | 13.875 | 3704.5 | 330508.125 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1822 | 350.25 | 29.625 | 3366.75 | 6900.5 | 1.992 |
| 500 | 1000 | off | 8 duration / 0 extinct | 19.625 | 14.875 | 3706.5 | 335893.25 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1868.25 | 340 | 3.375 | 64.5 | 1107 | 1.900 |
| 1000 | 0 | off | 8 duration / 0 extinct | 575.875 | 207.875 | 1623.375 | 124508.125 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1861.875 | 346.125 | 2.75 | 74.875 | 1142.75 | 1.901 |
| 1000 | 500 | off | 8 duration / 0 extinct | 590.625 | 211.25 | 1684.625 | 132043.5 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1918.75 | 350.25 | 3.625 | 206.375 | 1161.5 | 1.913 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 605.375 | 215.375 | 1647.625 | 134348.375 | 0 | — |

Full unrounded point values are preserved in the machine-readable reference at the v29 commit.

## Historical v29 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while exact migration-enabled trajectories reflect both HouseholdId-invariant M4 scheduling and PersonId-invariant M2 parentage stochastic coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v29 rebaseline preserved the experiment design and scientific question while recording the expected downstream consequences of corrected parentage stochastic coupling. It did not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

## Historical v28 reference and provenance

Audit-v4 AV4-003 / #491 changes causal same-seed M4 migration draw assignment: shared sequential `migration/choice` and `migration/uncertainty` draws are no longer attached to households by arbitrary canonical `HouseholdId` iteration order. Because that correction can alter migration decisions and propagate through residence, resources, condition and later demographic state, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v27 values.

Reviewed v28 execution:

- CI run: `33800926422`;
- M7.6 job: `100803174912`;
- exact production head used by the archived run: `b8aa338edb73b6432bd87bac87f2dd58e6022c5d`;
- artifact: `9911428218` (`m7-6-resource-variability-derived`, `sha256:527aef0dbc5a8f47e263e9478107d53017ac727d4c02cb2c4c0d34fef5ff446d`);
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v28`;
- sweep ID: `anthrosim-sweep-v2-eb9be952b30f6e7d`.

All **144/144** planned runs completed and were scientifically eligible, with no failed, incomplete, record-limit or otherwise operationally censored runs. The workflow job failed only at the final equality assertion against the then-current frozen v27 point-results reference; the complete derived artifact was archived successfully.

The on/off control is exact and diagnostic for this M4-only repair: all **9/9 migration-disabled point summaries are numerically identical to v27**, while all **9/9 migration-enabled point summaries change**. The source definition, 18-point factorial design, paired seeds, M3 resource settings, demographic settings, completion/censoring rules and declared endpoints are unchanged. This isolates the required rebaseline to trajectories in which permanent M4 migration can act, rather than unexplained resource or demographic drift.

The reviewed v28 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls.

## Historical v28 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1616.25 | 341.5 | 288 | 36287.25 | 28641.75 | 2.049 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4896.25 | 548877.5 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1632 | 345.5 | 290 | 36696.25 | 28922.25 | 2.049 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4891.875 | 551466.75 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1647.125 | 341.5 | 283.125 | 38773 | 28473.375 | 2.049 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4890 | 550569 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1777.375 | 340.875 | 24.125 | 2363.875 | 6885.375 | 1.973 |
| 500 | 0 | off | 8 duration / 0 extinct | 24.375 | 15.75 | 3680.25 | 328961.875 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1807.375 | 344.5 | 25.875 | 2698.875 | 6980.25 | 1.976 |
| 500 | 500 | off | 8 duration / 0 extinct | 18.75 | 13.875 | 3704.5 | 330508.125 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1848.5 | 351 | 29 | 3240.25 | 6926.875 | 1.988 |
| 500 | 1000 | off | 8 duration / 0 extinct | 19.625 | 14.875 | 3706.5 | 335893.25 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1876 | 342.5 | 3 | 63.75 | 1125.25 | 1.892 |
| 1000 | 0 | off | 8 duration / 0 extinct | 575.875 | 207.875 | 1623.375 | 124508.125 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1829.25 | 340.375 | 3.375 | 63.875 | 1103.75 | 1.908 |
| 1000 | 500 | off | 8 duration / 0 extinct | 590.625 | 211.25 | 1684.625 | 132043.5 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1879.5 | 348.125 | 3.5 | 226.375 | 1123.125 | 1.928 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 605.375 | 215.375 | 1647.625 | 134348.375 | 0 | — |

Full unrounded point values are preserved in the historical machine-readable reference.

## Historical v28 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while the exact migration-enabled trajectories reflect HouseholdId-invariant M4 stochastic scheduling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v28 rebaseline preserved the experiment design and scientific question while recording the expected causal consequences of correcting arbitrary household-label migration draw assignment. It did not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

## Historical v27 reference and provenance

Audit-v4 AV4-002 / #488 changes the causal same-seed background-mortality coupling: background mortality draws are no longer assigned by arbitrary canonical `PersonId` record order. Because that correction can propagate through deaths, household composition, resources, fertility and migration, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v26 values.

Reviewed v27 execution:

- CI run: `33789214317`;
- M7.6 job: `100766028843`;
- exact production head used by the archived run: `0442c7d8a42402713195b24420f722a5d7226392`;
- artifact: `9907214330` (`m7-6-resource-variability-derived`, `sha256:c43a3a920e8c2ac440beb793c448fd39ecd81f1ca23e3ec934e318901f1eea82`);
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v27`;
- sweep ID: `anthrosim-sweep-v2-71270161787bc7ca`.

All **144/144** planned runs completed and were scientifically eligible, with no operational censoring. The workflow job failed only after execution when the then-current CI assertion compared the generated v27 point summaries with the frozen v26 machine reference; that stale-reference assertion is not scientific evidence against the v27 execution.

The source definition is unchanged: 18 factorial points × 8 paired seeds. The reviewed v27 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls. Quantitative values move because corrected background-mortality coupling changes downstream trajectories.

## Historical v27 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1640.25 | 340.875 | 270.125 | 35091.25 | 28500.375 | 2.045 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4896.25 | 548877.5 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1653.25 | 345.25 | 287.75 | 36771.25 | 28660.5 | 2.048 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4891.875 | 551466.75 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1627 | 339.5 | 288.625 | 39066.625 | 28588.875 | 2.049 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4890 | 550569 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1816.5 | 342 | 24.25 | 2227 | 6868.25 | 1.981 |
| 500 | 0 | off | 8 duration / 0 extinct | 24.375 | 15.75 | 3680.25 | 328961.875 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1796.75 | 343.375 | 24.625 | 2537.75 | 6868.125 | 1.983 |
| 500 | 500 | off | 8 duration / 0 extinct | 18.75 | 13.875 | 3704.5 | 330508.125 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1810 | 344.875 | 32.625 | 3609.25 | 6889.875 | 1.988 |
| 500 | 1000 | off | 8 duration / 0 extinct | 19.625 | 14.875 | 3706.5 | 335893.25 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1911.75 | 341.25 | 2.75 | 56.625 | 1133.5 | 1.874 |
| 1000 | 0 | off | 8 duration / 0 extinct | 575.875 | 207.875 | 1623.375 | 124508.125 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1878.125 | 350.375 | 2.875 | 56 | 1125.875 | 1.904 |
| 1000 | 500 | off | 8 duration / 0 extinct | 590.625 | 211.25 | 1684.625 | 132043.5 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1847.5 | 344.75 | 4.5 | 213.75 | 1112.125 | 1.925 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 605.375 | 215.375 | 1647.625 | 134348.375 | 0 | — |

Full unrounded point values are preserved in the historical machine-readable reference.

## Historical v27 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while the exact trajectory-level values are those generated under the corrected v27 background-mortality coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v27 rebaseline preserved the experiment design and scientific question while recording the expected causal consequences of correcting arbitrary background-mortality draw assignment. It did not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

## Historical v26 reference and provenance

Audit-v4 AV4-001 / #486 changes the causal same-seed fertility coupling: annual fertility draws are no longer assigned by arbitrary canonical `PersonId` record order. Because that correction can propagate through births, household composition, resources and migration, the frozen M7.6 synthetic reference was rerun and reviewed rather than forced to reproduce v20 values.

Reviewed v26 execution:

- CI run: `33761813921`;
- M7.6 job: `100674851272`;
- exact production head used by the archived run: `5d73176f9c26a9701193cfa46db9f7d341af7d8a`;
- artifact: `9896479783` (`m7-6-resource-variability-derived`, `sha256:75445419e242e39c77804356216558490a58c62dd30c3c7df36c736f91cdbe82`);
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.4`;
- model semantics: `anthrosim-model-semantics-v26`;
- sweep ID: `anthrosim-sweep-v2-00d722b4c9bee97f`.

All **144/144** planned runs completed and were scientifically eligible. The workflow job failed only after execution when the then-current CI assertion still required the frozen v20 machine reference; that stale-reference assertion is not scientific evidence against the v26 execution.

The source definition is unchanged: 18 factorial points × 8 paired seeds. The reviewed v26 result preserves the substantive synthetic conclusions: every low-productivity (`250`) migration-disabled arm is extinct in 8/8 seeds while every matched migration-enabled arm reaches the requested duration; at every matched productivity/seasonality point migration-enabled runs retain higher terminal population, lower condition-mediated mortality and lower unmet resource need than migration-disabled controls. Quantitative values move because the corrected fertility coupling changes downstream trajectories.

## Historical v26 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1616.625 | 344.625 | 283.375 | 35793.375 | 28498.5 | 2.048 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4882.875 | 546282.25 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1637.5 | 351.375 | 261 | 34715.125 | 28509.625 | 2.045 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4897.375 | 551156.75 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1601 | 341.375 | 291.875 | 39558.25 | 28580 | 2.047 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4889.125 | 548478.875 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1833.125 | 345.75 | 22 | 1997.125 | 6877.625 | 1.978 |
| 500 | 0 | off | 8 duration / 0 extinct | 18.625 | 13.25 | 3693.75 | 328863.375 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1826.125 | 345.875 | 24.875 | 2272.125 | 6914.125 | 1.984 |
| 500 | 500 | off | 8 duration / 0 extinct | 24.5 | 17.25 | 3697.125 | 329621.75 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1840.125 | 351.5 | 26.125 | 3003 | 6872.875 | 1.987 |
| 500 | 1000 | off | 8 duration / 0 extinct | 25.5 | 18.375 | 3737.75 | 337138.875 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1852 | 344 | 2 | 17.875 | 1081.875 | 1.892 |
| 1000 | 0 | off | 8 duration / 0 extinct | 631.875 | 219.625 | 1617.125 | 124574.75 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1867 | 343.25 | 2.375 | 99 | 1102 | 1.901 |
| 1000 | 500 | off | 8 duration / 0 extinct | 618 | 218.5 | 1594.625 | 124008.125 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1879.5 | 347.875 | 4.25 | 272 | 1105.125 | 1.918 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 619.625 | 218.5 | 1588.125 | 128760.875 | 0 | — |

Full unrounded point values are preserved in the historical machine-readable reference.

## Historical v26 interpretation

### Productivity remains the strongest resource control

The broad productivity gradient remains: low productivity produces substantially greater condition-mediated mortality and unmet need than high productivity, while the exact trajectory-level values are those generated under the corrected v26 fertility coupling.

### Migration remains strongly associated with persistence in this synthetic design

At productivity `250`, all three migration-disabled seasonality points become extinct in all eight paired seeds, while all matched migration-enabled runs reach the requested duration. The persistence contrast also remains large at productivity `500` and `1000`.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the three-level comparison does not support a universal monotonic claim that greater seasonality always improves or worsens persistence.

### Interpretation boundary

This remains a synthetic mechanism-validation experiment, not calibration evidence. The v26 rebaseline preserved the experiment design and scientific question while recording the expected causal consequences of correcting arbitrary fertility-draw assignment. It did not support claims about real prehistoric population size, carrying capacity, climate, migration rates or any archaeological site.

## Historical v13 reference and provenance

The v13 reference was deliberately regenerated after issue #188 corrected the M4 kin-residence proxy. The experiment definition was **not changed**.

Reviewed v13 execution:

- CI run: `33095180014`;
- branch head: `708ed24e5a046f0b660208581ab624bd6f9598dd`;
- pull-request merge-ref build embedded in the run: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.0`;
- model semantics: `anthrosim-model-semantics-v13`;
- sweep ID: `anthrosim-sweep-v2-a362f82ab52bfae1`;
- preserved workflow artifact: `9656454468` (`sha256:4bc3b852cd56b3d6840c82bd4fcb9fdfb54db0211b25071ca02057ff2147fcdc`).

All **144/144** planned runs completed with provenance-valid, scientifically eligible outputs. There were no failed, incomplete, record-limit or otherwise operationally censored runs. Across the 18 points, 120 runs reached the requested 100 years and 24 valid completed runs ended by population extinction.

The immediate pre-#188 control is the final successful M7.6 artifact from PR #272:

- CI run: `33091589580`;
- branch head: `7dcfa3550716d8b5e25148ff3f47b9784b4905bb`;
- artifact: `9654873821`;
- artifact SHA-256: `1dff62be5ebeacc6c7bad9d93dc54384eb18dd09da053c75ac5f74144c33bde4`.

## Why the v13 reference changed

Issue #188 changes only M4 permanent-migration kin semantics. A living parent-child relationship that crosses household boundaries is now represented reciprocally: each household can receive the other household's current residence as a kin anchor, irrespective of whether the represented parent is female or male. Same-household relatives add no residence-specific anchor because the household moves together, and the old first-four/order-sensitive truncation is removed.

The M7.6 rerun was compared against the immediate v12 control before accepting a new reference.

The control pattern is unusually strong:

- exactly **72/144** run-level scientific outcomes change;
- those are **all 72 migration-enabled runs**: eight seeds in each of the nine migration-enabled parameter points;
- **all 72 migration-disabled runs are scientifically identical** to v12;
- all nine migration-enabled point aggregates change;
- all nine migration-disabled point aggregates remain unchanged;
- experiment definition, paired seeds, M3 resource settings, demographic settings, completion/censoring rules and package version are unchanged.

That on/off control isolates the changed reference values to M4 migration semantics. If the rebaseline had arisen from the M3 resource model or a demographic rule, the migration-disabled half of the factorial design would not remain exactly stable.

This does not imply that every downstream difference is a direct kin effect of the same magnitude. Once an early migration decision changes, later resource access, condition, births, deaths and migration history can diverge. The reference preserves that causal propagation rather than tuning the model to recover v12 aggregates.

## Historical v13 point results

The table reports descriptive means over the eight scientifically eligible seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move. `Condition deaths` are deaths through the model's condition-mediated mortality path; they are not uniquely attributable to resource scarcity.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Condition deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1615.625 | 334.625 | 312.375 | 39539.625 | 29338.25 | 2.048 |
| 250 | 0 | off | 0 duration / 8 extinct | 0 | 0 | 4948.125 | 558182.875 | 0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1603.625 | 338.375 | 303 | 39595.125 | 29519 | 2.048 |
| 250 | 500 | off | 0 duration / 8 extinct | 0 | 0 | 4937.375 | 557335.25 | 0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1620 | 351.625 | 293 | 40934.625 | 29375.75 | 2.048 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0 | 0 | 4944.5 | 559035.625 | 0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1791 | 347.75 | 25.25 | 2369.625 | 7094.5 | 1.977 |
| 500 | 0 | off | 8 duration / 0 extinct | 20.5 | 15.375 | 3753.375 | 333178 | 0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1781.75 | 339.875 | 28.625 | 2548.125 | 7082.75 | 1.984 |
| 500 | 500 | off | 8 duration / 0 extinct | 19.75 | 14.25 | 3764.375 | 337179.25 | 0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1766.125 | 343.75 | 33.5 | 3741.875 | 7018 | 1.987 |
| 500 | 1000 | off | 8 duration / 0 extinct | 16.5 | 11.375 | 3757.375 | 340643.875 | 0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1811.875 | 342.625 | 2.875 | 45.25 | 1127.75 | 1.882 |
| 1000 | 0 | off | 8 duration / 0 extinct | 579.75 | 212 | 1629.75 | 125288.625 | 0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1825.75 | 342.625 | 3.375 | 101.625 | 1136.75 | 1.899 |
| 1000 | 500 | off | 8 duration / 0 extinct | 565 | 206.875 | 1676.375 | 132356.625 | 0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1791.25 | 344.5 | 3.375 | 269.375 | 1140 | 1.92 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 570.625 | 206.25 | 1687.75 | 137699.375 | 0 | — |

Full unrounded point values are preserved in the historical machine-readable reference preserved in Git history.

## Historical v13 observations within this synthetic model

### Productivity remains the strongest resource control

With migration enabled, productivity `250` produces hundreds of condition-mediated deaths and roughly forty thousand units of unmet need, while productivity `1000` reduces those values to single-digit condition deaths and much smaller unmet need. Migration activity likewise falls from roughly 29,000 moves at productivity `250` to roughly 1,100 at `1000`.

This verifies a directional response of the implemented synthetic model; it does not identify an empirical productivity threshold.

### Migration remains strongly associated with persistence

At productivity `250`, every migration-disabled run becomes extinct while every paired migration-enabled run reaches the requested duration. At productivity `500`, migration-disabled terminal populations remain extremely small compared with migration-enabled runs. At productivity `1000`, the difference remains large.

The #188 rebaseline shows that the *details* of this migration-enabled outcome are conditional on the exact M4 kin representation. The qualitative on/off contrast is not a historical claim about real populations.

### Seasonality remains non-monotonic

Changing seasonal amplitude changes within-year resource timing and downstream trajectories, but the effects remain non-monotonic across this small eight-seed design. The experiment does not support a general statement that greater seasonality always improves or worsens persistence.

### Typical move length remains locally bounded

Pooled mean completed move distance remains near two grid cells while move count changes much more strongly with resource pressure. The fixed three-cell information radius remains an important structural constraint on this result.

## Historical reference context

Earlier exact references remain preserved in Git history.

- v8 followed the M3 resource-time accounting repair and changed both migration-enabled and migration-disabled arms because M3 itself changed.
- v11 followed the scarce-resource indivisible-unit apportionment repair.
- later v11-era reference updates corrected derived/nullability semantics without changing authoritative model trajectories.
- v12 is the immediate pre-#188 control used here; its M7.6 artifact passed the then-current frozen reference.
- v13 is different in a diagnostically M4-specific way: migration-enabled runs change while migration-disabled controls remain stable.
- v20 followed issue #326's fixed-point resource-condition response repair and is preserved in Git history as the immediate pre-v26 scientific reference.
- v26 records the expected downstream consequences of correcting arbitrary founder-person fertility draw assignment in AV4-001/#486.
- v27 records the expected downstream consequences of correcting arbitrary founder-person background-mortality draw assignment in AV4-002/#488.
- v28 again shows a diagnostically M4-specific pattern: every migration-disabled point remains exactly v27 while every migration-enabled point changes after AV4-003/#491 corrects arbitrary HouseholdId migration draw assignment.
- v29 records the expected downstream consequences of correcting parentage stochastic coupling in AV4-005/#495.
- v30/v31/v32 re-verification detail is preserved in dedicated contemporaneous research notes and Git history.
- v33 again shows the M4-specific control split: all migration-disabled point summaries remain exactly v32 while all migration-enabled summaries change after AV4-009/#518 removes canonical spatial-candidate coupling.

Historical snapshots are model-evaluation history, not targets that later corrected implementations must reproduce.

## Historical interpretation boundary

The historical v13 experiment supports reproducible comparison of explicit synthetic assumptions, paired multi-seed execution, provenance-preserving factorial analysis, controlled mechanism attribution and detection of changes caused by upstream semantic repairs.

It does **not** support claims about real prehistoric population size, carrying capacity, calories/biomass, climate reconstruction, migration frequency/distance, settlement duration, physiology, kinship organization or any specific archaeological site. The v13 kin repair improved symmetry and scientific interpretability of the null model; it did not make the synthetic resource or migration model empirically validated.

## v30 condition-mortality-coupling re-verification

Audit-v4 AV4-006 / #497 reran the canonical 144-run M7.6 design on the exact production candidate. Exact point summaries change under corrected condition-mortality coupling, while all pre-existing qualitative synthetic-validation contrasts remain intact. Exact v30 results and provenance are recorded in [`condition-mortality-coupling-v30-reverification.md`](condition-mortality-coupling-v30-reverification.md).
