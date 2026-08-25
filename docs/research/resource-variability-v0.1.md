# Resource variability experiment — v0.1

**Scientific status:** synthetic validation / exploratory model exercise  
**Empirical status:** unvalidated; no claim about a real prehistoric population  
**Canonical definition:** `experiments/v0.1-resource-variability.json`  
**Preserved derived reference:** `experiments/v0.1-resource-variability-reference.json`

## Question

The first documented M7 experiment asks:

> How do synthetic resource-productivity magnitude and seasonal variability affect population persistence, spatial dispersion and migration in AnthroSim?

This is a research-style question asked of the implemented synthetic model. It is **not** a claim that the tested values represent real carrying capacities, climate variability, migration distances or prehistoric demographic rates.

## Design

The canonical definition uses one otherwise-fixed 100-year setup:

- 5,000 synthetic founders;
- 64 × 64 synthetic world;
- target household size 5;
- persistent-person-record safety ceiling 150,000;
- annual resource need 100 abstract units per living person;
- migration information radius 3 cells when migration is enabled;
- eight fixed seeds: `71001` through `71008`.

It crosses three explicit factors:

- resource productivity scale: `250`, `500`, `1000` permille;
- resource seasonal-amplitude scale: `0`, `500`, `1000` permille;
- M4 migration: enabled or disabled.

This produces 18 parameter points × 8 paired seeds = **144 ordinary M7.2 runs**. The same seed set is reused at every parameter point so otherwise-equal comparisons share the same stochastic world/initialization seed identity.

`resourceSeasonalityScalePermille` scales the generated cell seasonal amplitude inside the M3 regeneration calculation. `0` removes that seasonal swing and `1000` retains the full synthetic baseline amplitude. It does not alter the generated world itself and does not represent a reconstructed climate series.

## Reproduction and provenance

Build and launch instructions are documented in `docs/experiments-v0.1.md`. The versioned-definition adapter calls the ordinary `anthrosim sweep` command; it does not implement a second simulation or analysis path.

A reproduction preserves this chain:

1. versioned source definition and its SHA-256;
2. model/package version, model-semantics identity and embedded source revision;
3. immutable sweep manifest with exact base settings, dimensions and point order;
4. one immutable M7.2 experiment manifest per point;
5. exact `ExperimentConfig` and seed for every child run;
6. M5 authoritative artifacts for every completed child;
7. derived run/point analysis tables whose rows retain their source run identities.

The current preserved reference was regenerated after the M4 stay/relocation utility-semantics repair (#186). It came from CI run `32907506629`, testing branch head `6964699ce4c671895530e9e38990593e75a0b7c2` through merge-test revision `9dd1e447fa4ccab4d4e7be5f962b0236898b2b04`. Its source-definition SHA-256 is `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`, package version is `0.3.0`, model-semantics identity is `anthrosim-model-semantics-v7`, and its immutable sweep ID is `anthrosim-sweep-v2-244c857b10f09d36`.

The git/source/model identity participates in sweep identity, so a checkout at another authoritative model revision is expected to have a different sweep ID even when the declared parameter definition is unchanged. The checked-in reference JSON is therefore explicitly a **derived reference snapshot**, not authoritative simulation state.

## Execution result

All **144/144 planned child runs completed with provenance-valid bundles**. A valid population-extinction stop is still a completed run; it is not an execution failure. Across the factorial grid, 120 runs reached the requested 100-year duration and 24 runs ended by population extinction. There were no failed, incomplete or person-record-limit runs.

The table below reports descriptive means over the eight completed seeds at each parameter point. Resource quantities remain abstract synthetic units. `Occupied cells` is the final number of cells containing living people and is used only as a spatial-dispersion proxy. `Move distance` is pooled migration distance per completed household move in grid-cell Manhattan distance.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Scarcity deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 1562.5 | 339.1 | 289.5 | 34814.2 | 29596.1 | 2.045 |
| 250 | 0 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4947.4 | 560650.8 | 0.0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 1636.0 | 349.1 | 301.6 | 38186.6 | 29819.1 | 2.045 |
| 250 | 500 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4954.2 | 563425.0 | 0.0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 1607.5 | 348.1 | 319.0 | 43375.5 | 29428.5 | 2.050 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 4951.4 | 563495.9 | 0.0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 1817.8 | 347.5 | 24.9 | 1681.8 | 7193.0 | 1.977 |
| 500 | 0 | off | 8 duration / 0 extinct | 19.8 | 14.8 | 3764.0 | 338333.9 | 0.0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 1800.8 | 344.2 | 29.6 | 2764.1 | 7135.0 | 1.982 |
| 500 | 500 | off | 8 duration / 0 extinct | 19.9 | 14.8 | 3802.5 | 345930.6 | 0.0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 1832.1 | 353.8 | 34.6 | 5170.4 | 7200.6 | 1.987 |
| 500 | 1000 | off | 8 duration / 0 extinct | 19.1 | 15.6 | 3768.1 | 348839.8 | 0.0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 1818.2 | 343.8 | 3.2 | 64.1 | 1136.1 | 1.886 |
| 1000 | 0 | off | 8 duration / 0 extinct | 561.9 | 204.9 | 1665.9 | 128873.5 | 0.0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 1817.4 | 347.4 | 2.9 | 62.9 | 1103.5 | 1.899 |
| 1000 | 500 | off | 8 duration / 0 extinct | 546.9 | 200.8 | 1661.0 | 132517.1 | 0.0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 1848.1 | 344.5 | 5.8 | 620.5 | 1135.2 | 1.937 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 625.0 | 226.0 | 1634.5 | 136314.0 | 0.0 | — |

Full unrounded values for this reference execution are preserved in `experiments/v0.1-resource-variability-reference.json`; a fresh reproduction also emits run-level and point-level CSV/JSON directly from the immutable experiment outputs.

## v7 migration-semantics repair check

The v7 reference change is not an arbitrary numerical refresh. The experiment's migration-enabled/disabled factorial provides a built-in control for #186:

- all **nine migration-disabled points** reproduce the previous v6 reference exactly in every point metric compared by CI;
- all **nine migration-enabled points** change after repairing the stay comparator;
- total migration distance decreases in every migration-enabled point, by roughly 4–12%;
- pooled mean move distance decreases in every migration-enabled point, by roughly 5–8%;
- move counts change in mixed directions, indicating that the most consistent effect is on which relocations are attractive/how far selected moves extend rather than a blanket suppression of migration;
- downstream resource/population values change only in migration-enabled arms because altered residence histories change later resource exposure.

This controlled pattern is evidence that the reference change is caused by the M4 semantic repair rather than by an unrelated demographic, resource or orchestration change. It remains synthetic implementation/model-behaviour evidence, not empirical validation.

## Observations within this synthetic model

### 1. Mobility response dominates persistence in this design

At productivity `250`, every migration-disabled seed became extinct before year 100, while every otherwise-matched migration-enabled seed reached the full duration. At productivity `500`, migration-disabled runs reached year 100 with only about 19 living people on average, compared with roughly 1,801–1,832 when migration was enabled. At productivity `1000`, the corresponding averages were roughly 547–625 without migration and 1,817–1,848 with migration.

This does **not** show that real human populations require this amount or form of migration. It shows that, under the current M3/M4 rules, allowing households under pressure to search a bounded local neighbourhood strongly changes their exposure to local resource scarcity.

### 2. Resource magnitude has a strong directional effect on resource stress

With migration enabled, moving from productivity `250` to `1000` reduced mean scarcity deaths from roughly 290–319 per point to roughly 3–6, and reduced mean unmet resource need from roughly 34,800–43,400 to roughly 63–621. Migration activity also fell sharply: about 29,400–29,800 completed moves at productivity `250`, about 7,100–7,200 at `500`, and about 1,100–1,140 at `1000`.

That direction is consistent with the implemented causal structure: better local resource support creates less scarcity pressure and therefore fewer reasons to relocate. It verifies useful model responsiveness; it does not establish an empirical productivity threshold.

### 3. Seasonal-amplitude effects are smaller and not consistently monotonic

At fixed productivity and migration state, changing seasonality produced much smaller shifts than changing productivity or enabling/disabling migration. The direction was not universal. For example, migration-enabled final living population is non-monotonic at productivity `250` (1562.5 → 1636.0 → 1607.5), at `500` (1817.8 → 1800.8 → 1832.1), and nearly flat before rising at `1000` (1818.2 → 1817.4 → 1848.1).

The eight-seed descriptive design therefore does not support a general statement such as “more seasonal variability always reduces persistence.” A later scientific study would need broader uncertainty analysis and a better-grounded temporal resource model before making that kind of claim.

### 4. Migration changes the spatial-dispersion proxy as well as survival

Migration-enabled points finish with roughly 339–354 occupied cells. Migration-disabled productivity-`500` points finish with only roughly 15 occupied cells, while productivity-`1000` points finish with roughly 201–226. The productivity-`250` disabled cases are extinct and therefore occupy no cells.

This is evidence that the current mobility mechanism substantially changes spatial distribution inside the synthetic model. It is **not a formal fragmentation result**: occupied-cell count contains no information about connectedness, cluster topology, settlement identity, site duration or archaeological visibility, and it is confounded with the much larger surviving populations in migration-enabled runs.

### 5. Typical completed move length changes much less than move frequency

Under the repaired v7 comparator, pooled mean completed move distance remains near two cells: about 2.05 cells at productivity `250`, about 1.98–1.99 at `500`, and about 1.89–1.94 at `1000`. The major resource-driven mobility response is still the **number of household moves**, not a dramatic change in typical move length. This is unsurprising given the fixed three-cell information radius and current local utility/travel-cost rules.

The v7 repair nevertheless matters for this measure: all nine migration-enabled points have shorter mean moves than the previous v6 reference because the stay action is no longer made artificially worse by travel/relocation-only costs.

## What this experiment can support scientifically

As an engineering and synthetic-model baseline, the experiment can support:

- deterministic, repeatable comparisons between explicit model assumptions;
- paired multi-seed ensemble comparisons rather than single-run anecdotes;
- factorial sensitivity exercises over declared controls;
- exact provenance from experiment definition to child run artifacts;
- explicit failed/incomplete-run accounting rather than silent filtering;
- causal inspection through events, metrics, checkpoints and migration traces;
- external Python/R analysis of derived rectangular tables;
- long-duration invariant testing and a measured engineering performance envelope;
- generation of model-behaviour hypotheses and questions that can later be confronted with evidence.

These are important prerequisites for research use. They are not empirical validation by themselves.

## What this experiment cannot support scientifically

The current model must **not** be used to claim:

- a real prehistoric population size, growth rate or extinction probability;
- real carrying capacity, caloric productivity or palaeoenvironmental reconstruction;
- real migration frequency, distance, route choice or settlement duration;
- a calibrated relationship between food supply and human physiology;
- a formal archaeological settlement-fragmentation measure;
- culturally realistic household, kinship, marriage, exchange or institutional behaviour;
- site-specific inference, including any real archaeological site;
- that a synthetic parameter value is empirically plausible merely because the simulation behaves coherently;
- that eight fixed seeds constitute statistical inference or uncertainty quantification.

Important current simplifications include synthetic founder age/spatial initialization, abstract resource units, no explicit storage/spoilage/exchange, no direct resource-to-fertility mechanism, atomic household travel, bounded local information, a minimal kin proxy, no route memory, and no empirical landscape.

## Interpretation boundary

The strongest conclusion from this experiment is methodological:

> AnthroSim can execute a declared causal question as a reproducible multi-factor, multi-seed experiment and expose interpretable differences without scripting the requested outcome.

The observed differences are properties of `synthetic_validation_v1` under the referenced model semantics. They are useful for testing whether the engine and current mechanisms respond coherently to changed assumptions. They are **not evidence about prehistory**.
