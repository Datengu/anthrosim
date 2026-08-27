# Competing mortality risks v1

**Status:** normative executable contract for `anthrosim-model-semantics-v15`  
**Scope:** M2 background mortality + M3 condition-mediated mortality  
**Scientific status:** model-structure specification; **not empirical calibration or archaeological validation**

## Purpose

Before v15, AnthroSim applied M3 condition-mediated mortality before the annual M2 mortality pass. At the final resource boundary of a model year, a person killed by M3 was removed before M2 could draw its background mortality probability. Total survival could still resemble a sequential product, but the recorded cause depended on scheduler order: condition-mediated mortality received first claim simply because its function ran first.

Issue #208 requires an explicit competing-risk interpretation rather than function-call priority.

## 1. Cause-specific processes

v15 treats the two existing mechanisms as separate cause-specific risks:

- **M2 background mortality** is the configured age-specific annual probability. The age band is selected from age at the start of the model year, preserving the #179 interval-start contract.
- **M3 condition-mediated mortality** is the existing condition-dependent reference-quarter probability, preserving the #204 elapsed-time conversion and #200 causal-neutral `condition_mediated` meaning.

Neither configured schedule is claimed to be empirically correct merely because the two are now combined coherently.

## 2. Elapsed-time interpretation of M2 mortality

The M2 annual probability `p` is cumulative incidence over one 365-day model year. It is not interpreted as a biologically instantaneous event that occurs only because the annual M2 function happens to execute on day 365.

For an elapsed interval `[s,e)` within the model year, v15 uses linear cumulative survival:

```text
S(t) = 1 - p * t / 365
q_background(s,e) = 1 - S(e) / S(s)
```

The implementation evaluates this as an exact integer rational. Multiplying conditional survival across any complete partition of the model year recovers exactly `1 - p`. Therefore changing only `resources.periodsPerYear` does not multiply the annual background mortality probability by adding more mortality boundaries.

This is a declared discrete elapsed-time approximation. It is **not** a continuous-time exponential hazard model.

## 3. Joint competing-risk rule

At every M3 mortality boundary, let:

- `q_c` = the exact condition-mediated interval probability;
- `q_b` = the exact elapsed M2 background interval probability.

Each cause receives an independent latent Bernoulli trigger on its existing deterministic named RNG stream:

- condition: historical private stream `resources/scarcity_mortality`;
- background: `demography/mortality`.

The private historical condition stream name does not change the scientific `condition_mediated` cause semantics.

The outcome is:

```text
no condition trigger + no background trigger -> survive
condition only                           -> condition_mediated death
background only                          -> demographic_mortality death
both triggers                            -> one death, cause allocated symmetrically
```

Consequently all-cause interval survival is exactly:

```text
(1 - q_c) * (1 - q_b)
```

and does not depend on which cause is evaluated first.

## 4. Dual-trigger cause attribution

If both latent cause triggers occur in the same interval, AnthroSim records exactly one authoritative death. Attribution is stochastic in proportion to the two cause-specific interval risks:

```text
P(condition cause | dual trigger)  proportional to q_c
P(background cause | dual trigger) proportional to q_b
```

The implementation uses the per-million representations of the two exact interval risks as integer weights. The tie draw combines one draw from each named cause stream using XOR before unbiased bounded selection. XOR is commutative, so exchanging the two cause labels/streams exchanges attribution but cannot introduce a left/right or first-called preference.

This is an explicit **discrete independent-trigger competing-risk model**. It should not be described as a fitted continuous-time cause-specific hazard model.

## 5. Scheduling and M4 residence

For each M3 interval end, the authoritative order is:

1. resource settlement and condition update;
2. joint condition/background competing mortality;
3. M9 transitions/start decisions due that day;
4. M4 permanent-migration decision if due.

Therefore a death resolved at a boundary shared with M4 occurs before that move opportunity. `Death.cell` remains the person's persistent residence immediately before same-day M4. M4 is not allowed to move the person first and thereby redefine the residence attributed to mortality over the preceding elapsed interval.

After all subannual boundaries in the year have completed, the annual M2 stage performs fertility and parentage for survivors. It does **not** draw background mortality again.

## 6. Event observability

The `Death` event wire shape is unchanged. Under v15:

- `cause = demographic_mortality` means the background cause won the competing-risk resolution;
- `cause = condition_mediated` means the condition cause won;
- `probabilityPerMillion` is the selected cause's conditional probability for that elapsed interval;
- it is **not** the joint all-cause probability.

The demography observability report advances to schema v2. Its mortality exposure counts are M3-aligned background-risk interval exposures rather than one synthetic exposure per completed annual boundary. The report records the configured number of mortality risk intervals per year and states that the mortality process is order-invariant competing risk. Fertility opportunity reconstruction remains annual.

## 7. Double-counting warning for empirical schedules

A future empirical mortality schedule may already represent **all-cause** mortality. Such a schedule must not be inserted unchanged as M2 background mortality while also retaining explicit condition, disease, conflict, travel or other mortality causes. Doing so would double count the explicit causes already present inside the empirical all-cause schedule.

Before empirical calibration, each mortality input must therefore declare whether it is:

- all-cause;
- cause-specific;
- residual/background after removing explicit causes; or
- another explicitly justified quantity.

This is a research gate, not an optional documentation detail.

## 8. Validation requirements

v15 must retain automated evidence that:

1. partitioning an annual M2 probability over 1, 4, 12 or 365 M3 intervals preserves complete-year background survival exactly;
2. zero/certain single-cause edges behave exactly;
3. exchanging cause labels and their RNG streams exchanges attribution without changing survival or creating call-order priority;
4. controlled two-cause frequencies match the declared independent-trigger union and dual-trigger risk weighting;
5. a certain background death is resolved before a coincident M4 opportunity and occurs only once;
6. condition-only causal naming remains `condition_mediated`;
7. demography observability reconciles interval background deaths and interval-specific recorded probabilities.

## 9. Interpretation boundary

Valid language is:

> Under the declared v15 competing-risk assumptions, background and condition-mediated mortality were resolved as independent cause-specific interval risks without scheduler-priority attribution.

Invalid language is:

> AnthroSim now knows the real fraction of deaths attributable to nutrition versus natural mortality.

The repair makes the synthetic causal structure internally coherent; it does not validate prehistoric mortality schedules or causes.