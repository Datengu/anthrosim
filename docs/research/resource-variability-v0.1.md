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

## Current v13 reference and provenance

The current reference was deliberately regenerated after issue #188 corrected the M4 kin-residence proxy. The experiment definition was **not changed**.

Reviewed v13 execution:

- CI run: `33095180014`;
- branch head: `708ed24e5a046f0b660208581ab624bd6f9598dd`;
- pull-request merge-ref build embedded in the run: `100558f92beec25cd20a6f7f17ee0b0cf7ff5f3e`;
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

## Current v13 point results

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

Full unrounded point values are preserved in the machine-readable reference.

## Observations within this synthetic model

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

Historical snapshots are model-evaluation history, not targets that later corrected implementations must reproduce.

## Interpretation boundary

This experiment supports reproducible comparison of explicit synthetic assumptions, paired multi-seed execution, provenance-preserving factorial analysis, controlled mechanism attribution and detection of changes caused by upstream semantic repairs.

It does **not** support claims about real prehistoric population size, carrying capacity, calories/biomass, climate reconstruction, migration frequency/distance, settlement duration, physiology, kinship organization or any specific archaeological site. The v13 kin repair improves symmetry and scientific interpretability of the null model; it does not make the synthetic resource or migration model empirically validated.
