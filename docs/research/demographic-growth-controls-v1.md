# Synthetic demographic growth controls v1

Status: normative model-analysis contract for GitHub issue #239.

## Purpose

AnthroSim's historical `synthetic_validation_v1` demographic schedule is an engineering/null-model baseline. It is not an empirical prehistoric reconstruction and it is not population-regulating. A direct run first highlighted that this background can decline strongly once mortality, spacing, local mate availability, founder state and other model structure are realized together.

Issue #239 does **not** change that historical default. Instead it adds three explicit synthetic controls so downstream studies can ask whether a conclusion depends on background demographic direction:

- `negative_growth_control_v1`;
- `replacement_control_v1`;
- `positive_growth_control_v1`.

All three retain `synthetic_validation` provenance. They are controls for model analysis, not claims about a real population.

## What changed after the original observation

The original #239 observation was made against older v7 semantics. Since then the causal repair cluster named by the issue has been completed: resource-time accounting, condition/mortality semantics, competing risks, newborn condition, independent M3/M4 clocks, full research configuration, explicit founder initialization and explicit initial resource stock are now represented by current main. The growth controls are therefore defined against the repaired model rather than used to compensate for those older defects.

Issue #220 remains intentionally downstream: it will generalize stationarity/path-dependence diagnostics beyond this targeted demographic-control benchmark.

## Derivation boundary

The controls share the current synthetic mortality schedule, birth-sex ratio, parent-age limits and requested 1,278-day birth spacing. Under annual M2 fertility evaluation that spacing allows a new birth on the fourth annual boundary after a prior birth, so the derivation tracks three skipped annual fertility boundaries after a successful birth.

A female-only age × birth-spacing transition model is used as the intrinsic demographic reference. Its states are `(age_years, skipped_boundaries_remaining)`. At each annual transition it applies:

1. the age-specific M2 mortality probability;
2. conditional fertility if the female survives and is spacing-eligible;
3. the female share of successful births (`1 - 0.512 = 0.488`);
4. mother aging and spacing-state transition.

Two complementary quantities are derived from this same declared schedule:

- the dominant annual growth factor `lambda` and `r = ln(lambda)`, which classify intrinsic annual growth direction;
- expected lifetime daughters per newborn female, an `R0`-like generation-replacement quantity that is centred on `1.0` for replacement.

The executable reproducer is `scripts/demographic-growth-control-analysis.py`, and the committed Rust regression independently recomputes the daughter-replacement quantity from the JSON schedules.

This derivation deliberately excludes:

- local-male availability;
- spatial fragmentation;
- resources and condition mortality;
- M4/M9 movement;
- the synthetic founder age distribution.

Those are not mistakes in the derivation. They are separate causal layers whose contribution should remain measurable rather than being silently absorbed into a fertility coefficient.

## Control definitions

### Negative growth

`negative_growth_control_v1` intentionally preserves the historical synthetic mortality/fertility schedule exactly, changing only `scheduleId` so its use as a control is explicit rather than treating the default as a neutral background.

Its intrinsic age/spacing transition has approximately:

```text
lambda ~= 0.99504/year
r = ln(lambda) ~= -0.00497/year
expected lifetime daughters/newborn female ~= 0.86245
```

The historical baseline is therefore intrinsically declining once its effective four-year spacing is represented, even before local-male losses and founder/spatial effects are added.

### Replacement

`replacement_control_v1` multiplies only the four non-zero synthetic fertility-band probabilities by `1.28`:

```text
18-24: 220,000 -> 281,600 per million/year
25-34: 250,000 -> 320,000
35-39: 180,000 -> 230,400
40-44:  80,000 -> 102,400
```

Mortality, spacing, birth-sex ratio and parent-age limits are unchanged.

The age/spacing transition gives approximately:

```text
lambda ~= 1.00002/year
r ~= +0.000025/year
expected lifetime daughters/newborn female ~= 1.00074
```

This is an **intrinsic replacement control**, not a guarantee that every realized simulation remains numerically constant. Spatial mate limitation, founder transients, stochasticity, mortality from other enabled mechanisms, and later structural changes may still produce growth or decline. That departure is diagnostically useful.

### Positive growth

`positive_growth_control_v1` multiplies the same non-zero fertility bands by `1.60`:

```text
18-24: 220,000 -> 352,000 per million/year
25-34: 250,000 -> 400,000
35-39: 180,000 -> 288,000
40-44:  80,000 -> 128,000
```

The intrinsic age/spacing transition gives approximately:

```text
lambda ~= 1.00417/year
r ~= +0.00416/year
expected lifetime daughters/newborn female ~= 1.13076
```

This creates a deliberately modest positive-growth control rather than an explosive high-fertility stress case.

## Replicated realization test

`crates/anthrosim-core/tests/demographic_growth_controls.rs` evaluates all three committed controls through the ordinary `Simulation` path, not through a separate demographic calculator.

The benchmark uses:

- eight fixed stochastic seeds;
- 600 synthetic founders;
- one cell, so local-male availability is not deliberately suppressed by spatial fragmentation;
- M4 disabled;
- condition-mediated mortality disabled so the comparison isolates M2 demographic growth;
- 160 simulated years;
- late-window log growth from years 80 to 160, excluding the strongest founder-age transient without redefining authoritative simulation state.

Predeclared acceptance bands are:

```text
negative:    mean late annual log growth < -0.002
replacement: absolute mean late annual log growth < 0.0025
positive:    mean late annual log growth > +0.002
```

The generation-replacement regression additionally requires:

```text
negative:    expected lifetime daughters < 0.90
replacement: |expected lifetime daughters - 1.0| < 0.005
positive:    expected lifetime daughters > 1.10
```

These are classification tolerances for synthetic controls, not empirical confidence intervals. The fixed-seed regression proves directional model behavior; future inferential use remains subject to the repository's Monte Carlo precision contract.

A separate regression runs the same replacement schedule in a concentrated one-cell versus dispersed 32×32 founder world and requires more realized births in the concentrated case. This explicitly preserves local-male availability as a structural fertility suppressor instead of tuning it out of the replacement schedule.

## Interpreting founder and non-demographic effects

The control definition separates rather than hides these effects:

- **Founder initialization:** the intrinsic calculation has no founder-age distribution; the simulation benchmark uses the ordinary synthetic founder distribution but measures years 80–160, after the strongest direct founder-age transient. A study that needs a stronger convergence claim must use the analysis-window/initialization machinery and, ultimately, #220 diagnostics.
- **Local male availability:** the concentrated-versus-dispersed regression deliberately demonstrates that the same replacement schedule can realize fewer births when locality suppresses eligible male opportunities.
- **Non-demographic mortality:** the control benchmark sets condition-mediated mortality to zero and disables M4, so its classification is not contaminated by resource/travel condition mortality. Downstream studies may re-enable those pathways as independent causal dimensions rather than retuning fertility to offset them.

This is the intended decomposition boundary: the demographic schedule has a known intrinsic replacement tendency, while departures caused by initialization, locality or other mechanisms remain visible as departures.

## Interpretation

The central lesson from #239 is therefore not "AnthroSim fertility was wrong and has now been increased." The result is:

1. the historical synthetic baseline is a **declining demographic control**, not a neutral background;
2. a mathematically derived intrinsic replacement schedule is now available without changing the historical default;
3. a modest positive-growth schedule provides the opposite control;
4. realized growth can still depart from the intrinsic schedule because locality, initialization and other mechanisms are real model structure;
5. downstream resource, migration, landscape and aggregation results can now be sensitivity-tested against demographic direction rather than inheriting one hidden declining background.

Population stability remains an experimental condition. AnthroSim does not regulate population toward its initial size, and none of these schedules is labelled prehistoric, ancestral, calibrated or realistic.

## Model-semantics boundary

The three controls are versioned configuration artifacts consumed by the existing M2 engine. They do not alter the default `ExperimentConfig`, M2 equations, RNG streams, scheduler ordering, checkpoint schema, or `MODEL_SEMANTICS_ID`. A run changes only when a study explicitly selects a different committed control configuration.
