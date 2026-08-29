# Identifiability and equifinality analysis v1

AnthroSim treats calibration fit and scientific identification as different claims. A parameter set can reproduce a target while the evidence still fails to identify the underlying parameter, parameter combination, or structural mechanism.

## Research gate

Before a study claims that AnthroSim has quantitatively constrained a parameter or selected a historical mechanism, the declared calibration evidence must be analysed with `scripts/research-identifiability.py` over the study's preserved uncertainty/sensitivity design.

The gate is required for quantitative calibration, parameter inference and competing-hypothesis claims. It is not required for exploratory mechanism demonstrations that make no identification claim.

A failed gate is a scientific result, not an optimisation failure. The study must report the acceptable parameter region or ensemble and must not collapse that region to a unique best-fit value unless the declared evidence identifies it.

## Inputs

The procedure takes two versioned JSON documents:

1. A **plan** declaring calibration targets, tolerances, held-out corroboration observables, claimed parameter IDs, whether a structural hypothesis is being claimed, and the maximum acceptable normalized parameter-range width.
2. A **data table** containing every evaluated design point, its exact parameter coordinates, an explicit structural-model identifier, and the relevant output summaries.

The point table is intended to be generated from the immutable #205 research experiment/sensitivity path. The identifiability analysis does not edit or rerun model configurations and therefore cannot hide a changed model behind an optimisation step.

## Evidence-role firewall

Calibration targets and held-out corroboration observables must be disjoint. The analyzer fails closed if an observable is declared in both roles.

Held-out corroboration may be used to derive a **discriminating prediction** between currently equifinal structural hypotheses, but it is not silently consumed to make the calibration gate pass. If investigators later choose to use that observation for calibration, that is a new declared analysis with a new plan/provenance record, consistent with #206.

## Practical parameter identifiability

For each parameter, the analyzer reports the range represented by all evaluated points and the range represented by the final acceptable region. Numeric parameters are considered practically identified only when the acceptable range is no wider than the plan's predeclared fraction of the explored range. Categorical parameters are identified only when a single value remains acceptable.

This is deliberately a transparent finite-design diagnostic, not a claim that a grid has reconstructed a continuous posterior. A real study remains responsible for choosing a scientifically adequate design density, parameter ranges and stochastic precision.

## Equifinality and interactions

The result preserves:

- all acceptable point IDs and the fraction of the evaluated design they occupy;
- per-parameter profile/conditional acceptance counts;
- pairwise parameter acceptance surfaces;
- the set of acceptable structural-model identifiers;
- staged diagnostics showing what each additional calibration pattern contributes;
- held-out observables that would discriminate between structural hypotheses still compatible with the calibration evidence.

When multiple parameter combinations or structures remain compatible with the claim, `equifinality.present` is true and the reporting policy is `report_acceptable_region_not_unique_optimum`.

The pairwise surfaces are diagnostic summaries, not a substitute for adequate global sensitivity sampling. Strong ridges or broad surfaces indicate combinations that the current evidence constrains more strongly than their individual components.

## Synthetic benchmark

`research/identifiability-benchmark-v1/` contains an intentionally non-identifiable benchmark inspired by the ratio structures highlighted in issue #217.

Two synthetic parameters, `opportunity_scale` and `need_scale`, each take values 1–4. The first calibration pattern is their ratio with target 1.0. It accepts all four diagonal combinations:

- (1, 1)
- (2, 2)
- (3, 3)
- (4, 4)

Thus a perfect match to the ratio alone cannot identify either absolute scale. Reporting one diagonal point as the calibrated answer would be false precision.

The independent second pattern is the sum, with target 4.0. Combining both declared patterns leaves only (2, 2). The benchmark therefore demonstrates the acceptance criterion from #217 directly: one observable constrains a parameter combination, while an independent second pattern materially increases identification.

The companion self-test also verifies that removing the independent second target makes the research gate fail and reports equifinality instead of selecting a best-fit point.

## Structural hypotheses

Each design point carries a `structure` identifier. If the study claims a structural mechanism, the calibration gate passes only when one structural identifier remains in the acceptable region. Multiple acceptable structures are reported as structural equifinality.

If held-out corroboration observables differ between those structures by more than the predeclared discrimination tolerance, the analyzer records them as discriminating predictions. That tells the investigator what additional observation could separate the hypotheses without pretending the existing evidence already does so.

## Interpretation limits

Passing this gate means only that the **declared evidence over the declared explored uncertainty space** identifies the declared quantity to the predeclared resolution. It does not prove that the explored range is empirically complete, that the model is structurally correct, that observations are error-free, or that the archaeological interpretation is unique outside the tested hypothesis set.

Conversely, failing the gate is not a defect in AnthroSim. It means the evidence does not support the requested inference at the stated resolution. That distinction is central to using equifinality as information rather than hiding it behind a single optimum.
