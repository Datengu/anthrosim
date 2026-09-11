# Exploratory model characterisation: M3-only timing at moderate productivity

**Date:** 2026-09-11  
**Status:** exploratory complete / non-validating  
**Evidence role:** model understanding and temporal-resolution sensitivity control  
**Model target:** `anthrosim-model-semantics-v37`  
**Core source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`

## Question

> Does M3 resource-settlement frequency remain weakly influential when resources are moderate enough that no-migration populations survive for the full 100-year horizon?

Experiment 8 tested the same M3 timing dimension under severe scarcity (productivity 150 permille), where all 180 no-migration runs eventually became extinct. This follow-up raises productivity to 750 permille so that temporal-resolution sensitivity can be evaluated using long-run population and condition outcomes rather than universal extinction timing.

This remains synthetic model characterisation, not empirical calibration or evidence for a real ecological accounting cadence.

## Design

Common configuration:

- 100 simulated years;
- 2,000 founders;
- 64 × 64 synthetic world;
- target household size 5;
- annual food need 100 abstract units/person/year;
- productivity scale **750 permille**;
- seasonality scale 1000 permille;
- **permanent migration disabled**;
- 30 matched seeds (`7201..7230`) in every treatment;
- all other synthetic v37 defaults retained.

Only M3 `resources.periodsPerYear` varied:

- `1, 2, 4, 8, 12, 24` settlements/year.

Total runs: **180**.

## Execution and integrity

The experiment used temporary branch `experiment/v37-m3-only-moderate`. Workflow run `34615620104` checked out the exact v37 source commit above and called the unchanged core `Simulation::new(...).run()` path.

The accepted artifact contained:

- exactly **180** rows;
- exactly **30** rows for every requested M3 frequency;
- productivity fixed at **750 permille** in all rows;
- permanent migration disabled in **180/180** runs;
- **zero** permanent migration moves in every run;
- `anthrosim-model-semantics-v37` and source commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` in every run;
- **180/180** runs reached `durationReached` at 36,500 simulated days;
- **0/180** extinctions.

## Results

| M3 settlements/year | Mean final population | Median | Range | Mean unmet need | Mean condition-mediated deaths | Mean final living condition (permille) |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 94.8 | 94.5 | 60–147 | 69,694 | 869.7 | 994.9 |
| 2 | 96.0 | 92.5 | 36–152 | 66,920 | 848.7 | 995.9 |
| **4 (v37 default)** | **102.0** | **99.5** | **47–163** | **68,067** | **825.9** | **993.7** |
| 8 | 98.4 | 96.0 | 49–148 | 69,187 | 824.3 | 992.0 |
| 12 | 107.1 | 101.5 | 57–167 | 69,329 | 823.6 | 993.3 |
| 24 | 102.4 | 95.5 | 59–164 | 70,266 | 827.0 | 991.6 |

All treatments therefore remain in the same broad long-run regime: populations contract strongly from the 2,000-founder start but survive to year 100 in every replicate.

### Final population differences are modest and non-monotonic

Matched-seed differences relative to the v37 default `P=4` were:

| Comparison | Mean final-population difference | Approx. paired 95% interval | Alternative higher |
|---|---:|---:|---:|
| P1 − P4 | −7.1 | −13.6 to −0.7 | 10/30 (2 ties) |
| P2 − P4 | −6.0 | −12.3 to +0.3 | 9/30 (2 ties) |
| P8 − P4 | −3.5 | −12.2 to +5.1 | 14/30 |
| P12 − P4 | +5.1 | −2.6 to +12.9 | 16/30 (1 tie) |
| P24 − P4 | +0.4 | −9.9 to +10.8 | 16/30 |

These intervals are exploratory descriptive summaries, not confirmatory inference. There is no monotonic relationship with increasing M3 frequency, and most contrasts against P4 are small relative to seed-to-seed variation.

`P=12` has the highest mean final population in this particular block. Pairwise matched-seed comparisons give P12 about +12.3 people versus P1, +11.1 versus P2, +8.7 versus P8, and +4.7 versus P24. The first three differences are relatively consistent in this sample, but the pattern is not enough to call 12/year an intrinsic optimum: it is a single synthetic environment with strong seasonality and multiple exploratory comparisons.

### Coarse timing increases condition-mediated mortality

The clearest resolution-related signal is in condition-mediated deaths:

- `P=1 − P4`: **+43.8 deaths** on average, approximate paired 95% interval **+25.5 to +62.0**, P1 higher in 26/30 seeds;
- `P=2 − P4`: **+22.7 deaths** on average, approximate paired 95% interval **+1.6 to +43.9**, P2 higher in 17/30 seeds;
- P8, P12 and P24 are all within about ±2.4 mean condition-mediated deaths of P4.

This supports the same qualitative warning as the harsh-resource control: collapsing M3 into very coarse annual or semiannual settlements alters within-year condition dynamics. The effect is nevertheless much smaller than the catastrophic M3/M4 clock-mismatch effects observed when permanent migration is active.

Cumulative unmet need is also broadly similar across treatments (about 67k–70k). P24 is about +2,198 units above P4 on average in the paired comparison, while other contrasts cross zero; there is no simple monotonic scarcity response.

Final living condition remains very high and close across treatments (roughly 992–996 permille), so differences in year-100 survivor count are not accompanied by a large separation in condition among survivors.

## Interpretation

Conditional model statement:

> Under the tested moderate-resource v37 configuration with permanent migration disabled, M3 settlement frequency from 1 to 24/year does not change the qualitative long-run regime: all 180 runs survive the full 100 years, and final populations remain in a broadly similar range. Very coarse 1–2/year settlement produces somewhat more condition-mediated mortality, while frequencies from 4–24/year are substantially closer. The exact survivor-count ordering is non-monotonic, with 12/year highest in this exploratory block.

Plain-language summary:

> M3 is **reasonably but not perfectly resolution-stable** here. Very coarse timing matters, but changing M3 frequency does not remotely reproduce the huge clock sensitivity seen when M4 migration is coupled to it.

Combined with Experiment 8, this strengthens the interpretation that the severe timing pathology diagnosed earlier belongs primarily to M4's decision-frequency/planning-horizon coupling and its interaction with M3, rather than to an intrinsic requirement that M3 run at one exact frequency.

## Why P12 deserves a targeted follow-up

The moderate block does contain a reproducible-looking local feature: P12 finishes with more survivors than several lower-frequency treatments, even though P24 does not continue that improvement. Because M3 regeneration is seasonal and `seasonalityScalePermille=1000`, a plausible model-internal explanation is that temporal resolution interacts with the discretisation of the seasonal resource cycle.

That explanation is **not established by this experiment**. It motivates the next controlled question:

> **Does the apparent P12 survivor advantage disappear when resource seasonality is removed?**

Repeating this exact moderate M3-only grid with `seasonalityScalePermille=0` would isolate whether the non-monotonic survivor ordering is driven mainly by seasonal temporal discretisation. If the P12 feature collapses under zero seasonality while coarse P1/P2 condition-mortality effects also shrink, seasonality would explain much of the residual M3 timing sensitivity. If the pattern persists, another within-period mechanism requires diagnosis.

## Reproducibility boundary

The compact artifact is exploratory evidence, not a repository-authoritative research bundle or regression oracle. Any claim-driving use should be rerun from a checked-in, reviewed experiment definition with retained authoritative outputs and the appropriate scientific gates.
