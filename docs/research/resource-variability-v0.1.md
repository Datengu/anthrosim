# Resource variability experiment — v0.1

**Scientific status:** synthetic validation / exploratory model exercise  
**Empirical status:** unvalidated; no claim about a real prehistoric population  
**Canonical definition:** `experiments/v0.1-resource-variability.json`  
**Preserved derived reference:** `experiments/v0.1-resource-variability-reference.json`

## Question

The first documented M7 experiment asks:

> How do synthetic resource-productivity magnitude and seasonal variability affect population persistence, spatial dispersion and migration in AnthroSim v0.1?

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

`resourceSeasonalityScalePermille` scales the generated cell seasonal amplitude inside the M3 regeneration calculation. `0` removes that seasonal swing and `1000` retains the full synthetic v0.1 amplitude. It does not alter the generated world itself and does not represent a reconstructed climate series.

## Reproduction and provenance

Build and launch instructions are documented in `docs/experiments-v0.1.md`. The versioned-definition adapter calls the ordinary `anthrosim sweep` command; it does not implement a second simulation or analysis path.

A reproduction preserves this chain:

1. versioned source definition and its SHA-256;
2. model/package version and embedded source revision;
3. immutable sweep manifest with exact base settings, dimensions and point order;
4. one immutable M7.2 experiment manifest per point;
5. exact `ExperimentConfig` and seed for every child run;
6. M5 authoritative artifacts for every completed child;
7. derived run/point analysis tables whose rows retain their source run identities.

The preserved reference execution came from CI run `32576648716`, testing PR head `7250a82fef250364a329708aa5e0ed261ce5086a` through merge-test revision `a0d79378c270028f60fabbae9dea83103797460d`. Its source-definition SHA-256 is `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`, model version is `0.1.0`, and its immutable sweep ID is `anthrosim-sweep-v2-76d8eaed14d18830`.

The git identity participates in sweep identity, so a checkout at another revision is expected to have a different sweep ID even when the declared parameter definition is unchanged. The checked-in reference JSON is therefore explicitly a **derived reference snapshot**, not authoritative simulation state.

## Execution result

All **144/144 planned child runs completed with provenance-valid bundles**. A valid population-extinction stop is still a completed run; it is not an execution failure. Across the factorial grid, 120 runs reached the requested 100-year duration and 24 runs ended by population extinction. There were no failed, incomplete or person-record-limit runs.

The table below reports descriptive means over the eight completed seeds at each parameter point. Resource quantities remain abstract synthetic units. `Occupied cells` is the final number of cells containing living people and is used only as a spatial-dispersion proxy. `Move distance` is pooled migration distance per completed household move in grid-cell Manhattan distance.

| Productivity | Seasonality | Migration | Terminal outcomes | Final living | Occupied cells | Scarcity deaths | Unmet need | Migration moves | Move distance |
| ---: | ---: | :---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 250 | 0 | on | 8 duration / 0 extinct | 2811.6 | 410.9 | 722.1 | 65502.8 | 39900.4 | 2.161 |
| 250 | 0 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 5004.2 | 573707.0 | 0.0 | — |
| 250 | 500 | on | 8 duration / 0 extinct | 2801.5 | 414.6 | 725.5 | 65457.1 | 39724.1 | 2.162 |
| 250 | 500 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 5012.5 | 577221.6 | 0.0 | — |
| 250 | 1000 | on | 8 duration / 0 extinct | 2752.6 | 416.1 | 734.8 | 70005.9 | 39417.5 | 2.164 |
| 250 | 1000 | off | 0 duration / 8 extinct | 0.0 | 0.0 | 5011.9 | 577609.8 | 0.0 | — |
| 500 | 0 | on | 8 duration / 0 extinct | 3455.9 | 424.6 | 82.8 | 5312.4 | 10990.2 | 2.128 |
| 500 | 0 | off | 8 duration / 0 extinct | 15.2 | 10.5 | 3955.9 | 359262.5 | 0.0 | — |
| 500 | 500 | on | 8 duration / 0 extinct | 3534.2 | 423.5 | 86.2 | 6029.6 | 11116.5 | 2.134 |
| 500 | 500 | off | 8 duration / 0 extinct | 20.5 | 13.4 | 3963.1 | 365956.2 | 0.0 | — |
| 500 | 1000 | on | 8 duration / 0 extinct | 3522.2 | 412.2 | 85.2 | 6859.2 | 11006.8 | 2.139 |
| 500 | 1000 | off | 8 duration / 0 extinct | 18.6 | 13.1 | 3967.8 | 374048.8 | 0.0 | — |
| 1000 | 0 | on | 8 duration / 0 extinct | 3611.1 | 421.0 | 10.9 | 94.6 | 2264.0 | 2.080 |
| 1000 | 0 | off | 8 duration / 0 extinct | 755.8 | 241.4 | 2378.9 | 187841.8 | 0.0 | — |
| 1000 | 500 | on | 8 duration / 0 extinct | 3649.2 | 426.0 | 11.9 | 165.0 | 2260.5 | 2.084 |
| 1000 | 500 | off | 8 duration / 0 extinct | 710.2 | 234.5 | 2335.4 | 190257.8 | 0.0 | — |
| 1000 | 1000 | on | 8 duration / 0 extinct | 3687.4 | 414.5 | 12.2 | 461.4 | 2361.8 | 2.093 |
| 1000 | 1000 | off | 8 duration / 0 extinct | 749.6 | 236.1 | 2411.5 | 203643.1 | 0.0 | — |

Full unrounded values for this reference execution are preserved in `experiments/v0.1-resource-variability-reference.json`; a fresh reproduction also emits run-level and point-level CSV/JSON directly from the immutable experiment outputs.

## Observations within this synthetic model

### 1. Mobility response dominates persistence in this design

At productivity `250`, every migration-disabled seed became extinct before year 100, while every otherwise-matched migration-enabled seed reached the full duration. At productivity `500`, migration-disabled runs technically reached year 100 but ended with only about 15–21 living people on average, compared with roughly 3,456–3,534 when migration was enabled. At productivity `1000`, the corresponding averages were roughly 710–756 without migration and 3,611–3,687 with migration.

This does **not** show that real human populations require this amount or form of migration. It shows that, under the current M3/M4 rules, allowing households under pressure to search a bounded local neighbourhood strongly changes their exposure to local resource scarcity.

### 2. Resource magnitude has a strong directional effect on resource stress

With migration enabled, moving from productivity `250` to `1000` reduced mean scarcity deaths from roughly 722–735 per point to roughly 11–12, and reduced mean unmet resource need from roughly 65,000–70,000 to roughly 95–461. Migration activity also fell sharply: about 39,400–39,900 completed moves at productivity `250`, about 11,000 at `500`, and about 2,260–2,362 at `1000`.

That direction is consistent with the implemented causal structure: better local resource support creates less scarcity pressure and therefore fewer reasons to relocate. It verifies useful model responsiveness; it does not establish an empirical productivity threshold.

### 3. Seasonal-amplitude effects are smaller and not consistently monotonic

At fixed productivity and migration state, changing seasonality produced much smaller shifts than changing productivity or enabling/disabling migration. The direction was not universal. For example, migration-enabled final living population declined across the three seasonality settings at productivity `250` (2811.6 → 2801.5 → 2752.6), was non-monotonic at `500` (3455.9 → 3534.2 → 3522.2), and increased at `1000` (3611.1 → 3649.2 → 3687.4).

The eight-seed descriptive design therefore does not support a general statement such as “more seasonal variability always reduces persistence.” A later scientific study would need broader uncertainty analysis and a better-grounded temporal resource model before making that kind of claim.

### 4. Migration changes the spatial-dispersion proxy as well as survival

Migration-enabled points finished with roughly 410–426 occupied cells. Migration-disabled productivity-`500` points finished with only roughly 10–13 occupied cells, while productivity-`1000` points finished with roughly 235–241. The productivity-`250` disabled cases were extinct and therefore occupied no cells.

This is evidence that the current mobility mechanism substantially changes spatial distribution inside the synthetic model. It is **not a formal fragmentation result**: occupied-cell count contains no information about connectedness, cluster topology, settlement identity, site duration or archaeological visibility, and it is confounded with the much larger surviving populations in migration-enabled runs.

### 5. Typical completed move length changes much less than move frequency

The pooled mean completed move remained near two cells: about 2.16 cells at productivity `250`, about 2.13 at `500`, and about 2.08–2.09 at `1000`. The major mobility response was therefore the **number of household moves**, not a dramatic change in typical move length. This is unsurprising given the fixed three-cell information radius and the current local utility/travel-cost rules.

## What v0.1 can support scientifically

As an engineering and synthetic-model baseline, v0.1 can now support:

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

## What v0.1 cannot support scientifically

The current model must **not** be used to claim:

- a real prehistoric population size, growth rate or extinction probability;
- real carrying capacity, caloric productivity or palaeoenvironmental reconstruction;
- real migration frequency, distance, route choice or settlement duration;
- a calibrated relationship between food supply and human physiology;
- a formal archaeological settlement-fragmentation measure;
- culturally realistic household, kinship, marriage, exchange or institutional behaviour;
- site-specific inference, including Bulstrode Camp or any other real archaeological site;
- that a synthetic parameter value is empirically plausible merely because the simulation behaves coherently;
- that eight fixed seeds constitute statistical inference or uncertainty quantification.

Important current simplifications include synthetic founder age/spatial initialization, abstract resource units, no explicit storage/spoilage/exchange, no direct resource-to-fertility mechanism, atomic household travel, bounded local information, a minimal kin proxy, no route memory, and no empirical landscape.

## Interpretation boundary

The strongest conclusion from this first experiment is methodological:

> AnthroSim v0.1 can execute a declared causal question as a reproducible multi-factor, multi-seed experiment and expose interpretable differences without scripting the requested outcome.

The observed differences are properties of `synthetic_validation_v1`. They are useful for testing whether the engine and current mechanisms respond coherently to changed assumptions. They are **not evidence about prehistory**.
