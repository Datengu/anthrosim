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

## Current v8 reference and provenance

The current reference was regenerated only after the M3 resource-time repair was reviewed. The experiment definition was **not changed**.

Reference execution:

- CI run: `32917412267`;
- branch head: `7e13d5ee82db0c65d5ac52e4e5501c812fc968b0`;
- merge-test revision embedded in the run: `bdee1f2831d8c18a9798acc5756cc10d21df1d04`;
- definition SHA-256: `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`;
- model version: `0.3.0`;
- model semantics: `anthrosim-model-semantics-v8`;
- sweep ID: `anthrosim-sweep-v2-9eccc6efd971714f`;
- preserved workflow artifact: `9588771705` (`sha256:cb543afc3fa2abd3e945eaab8fb559cce0980294978bd4a693c03b5f9d92a072`).

All **144/144** planned runs completed with provenance-valid outputs. There were no failed, incomplete or record-limit runs. Across the 18 points, 120 runs reached the requested 100 years and 24 valid completed runs ended by population extinction.

## Why the v8 reference changed

This rebaseline is not an arbitrary numerical refresh.

The frozen experiment uses `annualFoodNeed = 100` and the default four resource periods. Under v7, annual need was effectively reasoned about as equal 25-unit periods in important paths. Under v8 the actual 365-day scheduler intervals are 91, 91, 91 and 92 days, so the canonical fixed annual allocation is **24, 25, 25, 26**. M3 and M4 now use that same current-period quantity.

Seasonal regeneration also changed from endpoint sampling to integration over each actual period, normalized to preserve the complete-year unconstrained annual potential. Therefore both migration-enabled **and migration-disabled** arms can legitimately change. This differs from the earlier v7 M4-only repair, where migration-disabled arms served as an unchanged control.

The v8 rerun preserves the broad qualitative structure of the experiment rather than reproducing the old numbers mechanically:

- productivity `250` with migration disabled still becomes extinct in every paired seed;
- enabling migration still strongly improves persistence under the current synthetic M3/M4 rules;
- increasing productivity still sharply reduces scarcity deaths and unmet resource need;
- seasonal-amplitude effects remain smaller and non-monotonic compared with the productivity/migration effects;
- no change was made to the source experiment definition to obtain those outcomes.

## Current v8 point results

The table reports descriptive means over the eight completed seeds per point. `Move distance` is pooled Manhattan grid-cell distance per completed household move.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Scarcity deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1629.5 | 345.4 | 294.4 | 37595.9 | 29540.6 | 2.047 |
| 250 | 0 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4946.0 | 558940.6 | 0.0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1610.6 | 341.4 | 281.1 | 36245.9 | 29364.2 | 2.047 |
| 250 | 500 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4953.6 | 557657.5 | 0.0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1637.0 | 341.9 | 295.8 | 40060.5 | 29379.4 | 2.048 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4949.4 | 562073.1 | 0.0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1818.9 | 347.6 | 25.5 | 2149.6 | 7076.5 | 1.983 |
| 500 | 0 | off | 8 duration / 0 extinct | 20.4 | 15.6 | 3760.1 | 336518.1 | 0.0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1859.9 | 349.2 | 28.8 | 2575.8 | 7243.1 | 1.982 |
| 500 | 500 | off | 8 duration / 0 extinct | 21.4 | 15.8 | 3734.6 | 338666.2 | 0.0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1762.8 | 343.5 | 29.8 | 3058.1 | 7008.0 | 1.985 |
| 500 | 1000 | off | 8 duration / 0 extinct | 17.9 | 13.2 | 3756.4 | 343288.0 | 0.0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1855.1 | 344.2 | 3.4 | 75.4 | 1123.8 | 1.885 |
| 1000 | 0 | off | 8 duration / 0 extinct | 590.5 | 210.1 | 1639.5 | 127245.4 | 0.0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1802.9 | 342.4 | 3.1 | 59.2 | 1119.4 | 1.902 |
| 1000 | 500 | off | 8 duration / 0 extinct | 599.6 | 212.2 | 1619.2 | 127397.5 | 0.0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1804.6 | 342.5 | 3.6 | 174.1 | 1110.1 | 1.918 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 552.2 | 205.0 | 1647.6 | 133862.5 | 0.0 | — |

Full unrounded point values are preserved in the machine-readable reference.

## Observations within this synthetic model

### Productivity remains the strongest resource control

With migration enabled, low productivity produces hundreds of scarcity deaths and tens of thousands of unmet resource units, while productivity `1000` reduces those values to single-digit scarcity deaths and comparatively tiny unmet need. Migration activity likewise falls from roughly 29,000 moves at productivity `250` to roughly 1,100 at `1000`.

This verifies a directional response of the implemented model; it does not identify an empirical productivity threshold.

### Migration remains strongly associated with persistence

At productivity `250`, every migration-disabled run becomes extinct while every paired migration-enabled run reaches the requested duration. At productivity `500`, migration-disabled terminal populations remain extremely small compared with migration-enabled runs. At productivity `1000`, the difference remains large.

This is a property of the current synthetic M3/M4 coupling, not evidence that real populations require this modeled migration behavior.

### Seasonality remains non-monotonic

Once endpoint aliasing is removed, changing seasonal amplitude still changes within-year resource timing and therefore downstream trajectories. Those effects remain non-monotonic across the small eight-seed design. The experiment therefore does not support a general statement that increasing seasonality always improves or worsens persistence.

### Typical move length remains locally bounded

Pooled mean completed move distance remains near two grid cells, while move count changes much more strongly with resource pressure. The fixed three-cell information radius remains an important structural constraint on this result.

## Historical reference context

Earlier exact references are preserved in Git history. The v7 reference followed the M4 stay/relocation comparator repair and had the useful control property that all migration-disabled points remained identical to v6. That control no longer applies to v8 because M3 itself changed.

Historical snapshots are retained as model-evaluation history, not targets that later corrected implementations must reproduce.

## Interpretation boundary

This experiment supports reproducible comparison of explicit synthetic assumptions, paired multi-seed execution, provenance-preserving factorial analysis and detection of changes caused by upstream semantic repairs.

It does **not** support claims about real prehistoric population size, carrying capacity, calories/biomass, climate reconstruction, migration frequency/distance, settlement duration, physiology or any specific archaeological site. The v8 resource-time repair improves internal meaning and comparability; it does not make the synthetic resource model empirically validated.
