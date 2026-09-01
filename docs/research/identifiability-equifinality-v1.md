# Identifiability and equifinality analysis v2

AnthroSim treats calibration fit, simulation Monte Carlo precision, and scientific identification as different claims. A parameter set can reproduce a target while the evidence still fails to identify the underlying parameter, parameter combination, or structural mechanism. Likewise, a finite stochastic ensemble can have a point estimate near a target while remaining too imprecise to support a calibration decision.

## Research gate

Before a study claims that AnthroSim has quantitatively constrained a parameter or selected a historical mechanism, the declared calibration evidence must be analysed with `scripts/research-identifiability.py` over the study's preserved uncertainty/sensitivity design.

The gate is required for quantitative calibration, parameter inference and competing-hypothesis claims. It is not required for exploratory mechanism demonstrations that make no identification claim.

A failed gate is a scientific result, not an optimisation failure. The study must report the compatible parameter region or ensemble and must not collapse that region to a unique best-fit value unless the declared evidence identifies it.

## Schema-v2 inputs

The procedure takes two versioned JSON documents:

1. A **plan** declaring calibration targets, tolerances, held-out corroboration observables, claimed parameter IDs, whether a structural hypothesis is being claimed, and the maximum compatible normalized parameter-range width.
2. A **data table** containing every evaluated design point, its exact parameter coordinates, an explicit structural-model identifier where structural hypotheses are claimed, output summaries, and explicit `outputEvidence` for every calibration/corroboration output.

A `structure` identifier, when present, must be a non-empty JSON string. If `claim.structuralHypothesis=true`, every design point must carry such an explicit identifier; a missing identifier fails closed rather than being treated as an implicit structure. When no structural hypothesis is claimed, omission remains backward-compatible and maps only to the literal default identifier `"default"`. Non-string, empty, and whitespace-only structure values are invalid in either case. Structural equifinality and held-out discrimination use these exact validated string identities with no presentation-oriented coercion, so JSON values such as numeric `1` and string `"1"` can never collapse into one accepted structural identity.

`outputEvidence` must distinguish the two supported cases:

- `{"kind":"deterministic"}` means the output is genuinely deterministic for the declared analysis and therefore has no process-stochastic Monte Carlo sampling uncertainty.
- `{"kind":"monte_carlo","diagnosticId":"sha256:..."}` binds the output to an immutable embedded Monte Carlo diagnostic from `scripts/research-monte-carlo-sufficiency.py`. The identifier is the SHA-256 digest of the complete canonical diagnostic object and the analyzer verifies it before use.

For Monte Carlo evidence the analyzer also binds and verifies the replicate count, exact seed identities, diagnostic schema, uncertainty category, estimand point estimate, confidence interval, declared precision threshold, precision decision, and—in the quantile case—the finite-sample coverage result introduced by issue #334. Changing any bound diagnostic field without changing its content identity fails closed.

The point table is intended to be generated from the immutable #205 research experiment/sensitivity path. The identifiability analysis does not edit or rerun model configurations and therefore cannot hide a changed model behind an optimisation step.

## Simulation uncertainty is not empirical uncertainty

The Monte Carlo diagnostic represents **process-stochastic simulation uncertainty conditional on the declared model, parameterization, evidence treatment, estimand and seed design**. It does not estimate archaeological measurement error, dating uncertainty, parameter uncertainty, evidence ambiguity, or structural-model uncertainty.

Those empirical/evidence uncertainties must remain separately declared in the study. In this finite-design gate the calibration `target` and `tolerance` define the already-declared observational/calibration acceptance band; the simulation interval is then compared with that band rather than being silently substituted for it.

For a stochastic output, the upstream Monte Carlo gate must have returned `sufficient_stop`, the interval must exist, and its declared maximum half-width must be no larger than the calibration tolerance for which it is being used. A generic successful Monte Carlo calculation with a looser precision target is not sufficient evidence for a tighter identifiability task.

## Uncertainty-aware compatibility

For each target with calibration band `[target - tolerance, target + tolerance]`, a design point is classified from the full simulation uncertainty interval:

- **acceptable** only when the entire simulation interval lies inside the calibration band;
- **rejected** when the entire simulation interval lies outside the band without overlap;
- **unresolved** when the interval overlaps a calibration boundary or the required simulation precision has not been demonstrated.

A point is compatible if it is acceptable or unresolved. Parameter-width, profile, interaction-surface and structural diagnostics use this **compatible region**, not merely point estimates or only the definitely acceptable subset. Consequently simulation uncertainty can widen the region or leave it unresolved; it cannot spuriously narrow a parameter range by treating noisy estimates as exact.

The final research gate requires all three conditions:

1. no unresolved calibration points remain for the declared evidence;
2. the compatible parameter region identifies every claimed parameter to the predeclared resolution;
3. if a structural mechanism is claimed, only one compatible canonical structure identifier remains.

This gives the required precision trajectory: the same point estimates can remain non-identifying at low Monte Carlo precision and become identifying after a predeclared increase in independent replication makes their simulation intervals sufficiently narrow.

## Evidence-role firewall

Calibration targets and held-out corroboration observables must be disjoint. The analyzer fails closed if an observable is declared in both roles.

Held-out corroboration may be used to derive a **discriminating prediction** between currently compatible structural hypotheses, but it is not silently consumed to make the calibration gate pass. If investigators later choose to use that observation for calibration, that is a new declared analysis with a new plan/provenance record, consistent with #206.

For stochastic held-out outputs the analyzer uses a conservative envelope of their bound pointwise Monte Carlo intervals. A prediction is labelled discriminating only when the bound simulation precision is adequate and those interval envelopes remain separated by more than the predeclared corroboration discrimination tolerance. This envelope is a decision safeguard, not a newly invented estimator of between-point uncertainty.

## Practical parameter identifiability

For each parameter, the analyzer reports the range represented by all evaluated points and the range represented by the final compatible region. A claimed parameter must have at least two distinct evaluated levels before the finite design can support an identification claim. A parameter held at one level is **fixed by design**, not identified by evidence; its diagnostic reports `identified=false`, reason `insufficient_explored_variation`, and an explored-level count of one. For a fixed numeric parameter the normalized compatible width is undefined and therefore reported as `null`, rather than being coerced to zero.

Once genuine explored variation exists, numeric parameters are considered practically identified only when the compatible range is no wider than the plan's predeclared fraction of the explored range. Categorical parameters likewise require at least two explored alternatives and are identified only when one of those alternatives remains compatible.

Profiles and pairwise surfaces continue to report the actually evaluated levels and compatibility counts. They therefore expose whether a coordinate was varied independently of whether the final identification gate passes. A StudyProtocol or downstream finalization step must consume the fail-closed research gate rather than reinterpret a fixed-by-design parameter as quantitatively constrained.

This is deliberately a transparent finite-design diagnostic, not a claim that a grid has reconstructed a continuous posterior. A real study remains responsible for choosing a scientifically adequate design density, parameter ranges, empirical uncertainty treatment and stochastic precision.

## Equifinality and interactions

The result preserves:

- definitely acceptable, unresolved and full compatible point IDs;
- the exact per-target uncertainty-aware classification for every point;
- the immutable Monte Carlo diagnostic identities actually used by the calibration decision;
- per-parameter profile/conditional compatibility counts;
- pairwise parameter compatibility surfaces;
- the set of compatible canonical structural-model identifiers;
- staged diagnostics showing what each additional calibration pattern contributes;
- held-out observables that would discriminate between structural hypotheses still compatible with the calibration evidence.

When multiple parameter combinations or structures remain compatible with the claim, `equifinality.present` is true and the reporting policy is `report_compatible_region_not_unique_optimum`.

The pairwise surfaces are diagnostic summaries, not a substitute for adequate global sensitivity sampling. Strong ridges or broad surfaces indicate combinations that the current evidence constrains more strongly than their individual components.

## Synthetic benchmark

`research/identifiability-benchmark-v1/` remains the deterministic identifiability benchmark inspired by the ratio structures highlighted in issue #217. Schema v2 now marks every synthetic output explicitly as deterministic rather than allowing the analyzer to infer that scientific status from a bare number.

Two synthetic parameters, `opportunity_scale` and `need_scale`, each take values 1–4. The first calibration pattern is their ratio with target 1.0. It accepts all four diagonal combinations (1,1), (2,2), (3,3), and (4,4), so that pattern alone cannot identify either absolute scale. The independent second pattern is the sum with target 4.0. Combining both declared patterns leaves only (2,2).

The analyzer self-test adds the stochastic adversarial acceptance case from issue #338. Two parameter points have fixed estimates 0.00 and 0.10 against target 0.00 ± 0.05. With four-replicate, ±0.20 Monte Carlo intervals the design remains unresolved and non-identifying. With the **same point estimates** but adequately bound ±0.01 intervals, the first point is wholly inside the band, the second wholly outside, and the claimed parameter becomes identified. Tampering with the bound replicate provenance also fails closed.

The self-test also preserves the AV3-009 boundary: numeric JSON `1` and string JSON `"1"` are not two spellings of one structure. The numeric identifier is rejected as noncanonical, a structural claim with no explicit identifier is rejected, and two distinct valid string identifiers remain two compatible structures and force the structural gate to fail.

## Structural hypotheses

Each design point in a structural claim carries a canonical non-empty string `structure` identifier. The analyzer compares those exact identifiers; it never stringifies arbitrary JSON values. If the study claims a structural mechanism, the calibration gate passes only when one structural identifier remains in the compatible region. Multiple compatible structures are reported as structural equifinality.

If held-out corroboration observables differ between those structures by more than the predeclared discrimination tolerance after simulation uncertainty is accounted for, the analyzer records them as discriminating predictions. The held-out grouping uses the same canonical structure identifier semantics as the structural-equifinality gate. That tells the investigator what additional observation could separate the hypotheses without pretending the existing evidence already does so.

## Interpretation limits

Passing this gate means only that the **declared evidence over the declared explored uncertainty space**, with the declared simulation Monte Carlo precision, identifies the declared quantity to the predeclared resolution. It does not prove that the explored range is empirically complete, that the model is structurally correct, that observations are error-free, or that the archaeological interpretation is unique outside the tested hypothesis set.

Conversely, failing the gate is not a defect in AnthroSim. It means the evidence and precision available do not support the requested inference at the stated resolution. That distinction is central to using equifinality and stochastic uncertainty as information rather than hiding them behind a single optimum.
