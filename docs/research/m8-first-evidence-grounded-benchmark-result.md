# M8.6 first evidence-grounded spatial null-model result

## Status

This document records the first completed M8 Level-D evidence-grounded spatial null-model benchmark defined in `m8-first-evidence-grounded-benchmark.md`.

The benchmark remained case-study-neutral and did not target a known settlement, route, archaeological feature, boundary or desired spatial pattern. The only evidence-grounded environmental input was the declared open terrain source. Water/resource geography, founder placement, demography and other existing baseline mechanisms remained synthetic-validation assumptions.

The reference machine-readable result is preserved in `examples/m8-first-evidence-grounded-benchmark/reference-result.json`.

## Reference execution

The corrected reference execution was GitHub Actions workflow run `32655513479` from branch head:

`5f35540e2dbf01b07a661b04a9d5a42c1c6dcc09`

The pull-request merge-ref build recorded by AnthroSim was:

`47735f9861b9ffa586a9f5ea4475cc8e1748af33`

The complete uploaded research artifact had SHA-256:

`372f1b5381b4c50aa09a8e94736ef7d491e80e4c6e4dfea2cb387dcc9a911e20`

The full derived aggregate from that execution had canonical SHA-256:

`5d9108ef8302fb9774dc3556b5f5f2abd81324d749dbeffb00b789a140a96b2e`

The source terrain content digest was:

`sha256:cf790a87057dfdf126f9b314d9ec71d407afdedbe71c79e2b14c903d024ce0b9`

The normalized landscape identity was:

`landscape-v1-da7cb2c3d74d497a`

All four arms used `anthrosim-model-semantics-v1` and `anthrosim-spatial-transform-semantics-v1`. Every contributing run is additionally identified by seed, arm experiment identity, spatial configuration identity and terminal state digest in `reference-result.json`.

## Execution outcome

All **32 runs** completed the configured 100 simulated years:

- flat: 8/8 duration reached;
- weak terrain cost: 8/8 duration reached;
- moderate terrain cost: 8/8 duration reached;
- strong terrain cost: 8/8 duration reached.

No arm met the predeclared degeneracy criterion.

Terminal populations remained non-zero in every run. Across individual runs they ranged from 545 to 1,045 living people at year 100. Resource-scarcity deaths remained a small minority of deaths in this benchmark, so the comparison was not dominated by widespread scarcity collapse.

## Predeclared classification

The benchmark result is:

> **fragile spatial structure**

No primary metric met the predeclared robust-effect criteria.

| Primary metric | Result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |
| --- | --- | ---: | ---: |
| total migration distance | fragile | 10.30% | 3 / 5 / 0 |
| cell-time occupied | not distinctive | 1.08% | 6 / 2 / 0 |
| terminal population Herfindahl | not distinctive | 7.38% | 3 / 5 / 0 |
| terminal largest-cell share | fragile | 13.40% | 4 / 4 / 0 |

The two metrics exceeding the 10% materiality threshold did **not** show stable direction across seeds. Strong terrain cost sometimes increased and sometimes decreased total migration distance. The largest-cell population share was split evenly between positive and negative paired changes.

Occupancy duration moved in a more consistent positive direction under the strong treatment, but the median absolute effect was only 1.08%, below the predeclared materiality threshold. The terminal Herfindahl concentration metric likewise remained below that threshold and had inconsistent direction.

## Interpretation

The tested terrain-to-movement-cost mapping can materially alter individual trajectories, but this benchmark does **not** support a robust claim that stronger terrain constraints systematically increase or decrease migration distance or spatial concentration under the existing model.

The most defensible interpretation is therefore narrower:

- real-world-derived terrain is now capable of propagating through the deterministic model and measurably affecting simulated outcomes;
- those effects can be large enough to matter within individual runs;
- for the tested terrain patch, transformation range and M1-M4 assumptions, stochastic/initial-condition variation across seeds is large enough that the direction of the main terrain effects is not stable;
- environmental terrain constraints alone do not produce a strong, seed-robust spatial signature in the primary concentration/occupancy observables used here.

This is useful evidence about the model. It argues against treating one visually plausible terrain-bound run as explanatory evidence and demonstrates why ensembles and paired sensitivity are necessary.

## What this does not establish

This benchmark does not establish that:

- the simulated population represents a particular historical population;
- the selected terrain patch represents an ancient landscape state;
- the terrain-contrast proxy is a calibrated human travel-cost function;
- water, land use, vegetation, soils or resource geography are historically realistic;
- the synthetic demographic or migration rules are empirically valid for a particular society;
- a similar-looking simulated spatial pattern explains any archaeological pattern;
- terrain had no historical effect merely because this null model lacks a robust directional effect.

The benchmark is Level D of the M8 validation ladder: evidence-grounded environmental constraints with reproducible ensemble sensitivity. It is not Level E case-study validation.

## Analysis correction

The first execution of the new aggregation script incorrectly classified every run as degenerate because the postprocessor compared the serialized camel-case stop reason `durationReached` against the Rust-style string `duration_reached`.

Inspection of the authoritative run manifests showed that all 32 simulations had in fact reached the full configured duration. The postprocessor was corrected without changing seeds, model parameters, landscape, evidence catalogue or terrain transformations, and the complete four-arm benchmark was rerun.

The corrected reference execution above is the canonical result. The initial false degeneracy classification must not be interpreted as a model outcome.

## Reproduction

A third party can reproduce the input package and benchmark as follows:

1. regenerate the public input package using `scripts/prepare-m8-benchmark-landscape.py` with the pinned source SHA-256;
2. require byte-identical equality with `examples/m8-first-evidence-grounded-benchmark/` input files;
3. execute four ordinary `anthrosim ensemble` commands with seeds 8601-8608 and the predeclared common settings, changing only the corresponding terrain mechanism file;
4. supply `evidence.json` so source evidence is part of immutable experiment identity;
5. run `anthrosim-spatial-observability tree` over each arm;
6. run `scripts/aggregate-m8-spatial-benchmark.py` over the four experiment roots.

The dedicated M8.6 workflows automate these steps. Local filesystem locations are runtime locators only and are not scientific identities.

## M8 scientific conclusion

M8 has now demonstrated the complete generic path:

```text
open external spatial evidence
    -> pinned/reproducible preprocessing
    -> normalized immutable landscape
    -> explicit sensitivity-testable model transformation
    -> deterministic spatial ensemble
    -> machine-readable spatial observability
    -> predeclared aggregate interpretation
```

The first result is deliberately modest: terrain matters enough to perturb outcomes, but the tested null model does not produce a robust directional spatial effect across seeds.

That is a better scientific starting point than tuning the benchmark until a preferred pattern appears. Follow-on work should be driven by a specific research question and by which missing mechanisms or evidence dimensions could discriminate competing explanations, rather than by adding generic complexity for its own sake.
