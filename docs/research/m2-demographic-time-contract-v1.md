# M2 demographic-time contract v1

Status: authoritative scientific/model contract for the current M2 demographic-time semantics.

This document defines what AnthroSim's current demographic probabilities and annual demographic boundary mean. It is a **verification contract**, not an empirical validation claim. Background mortality is parameterized annually but, since the v15 competing-risk repair, it is executed as elapsed subannual risk at M3 mortality boundaries. Fertility and parentage remain annual M2 transitions. The model must not be described as a continuous-time demographic hazard model.

The versioned downstream validation/diagnostic surface for these semantics is defined separately in [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md). Joint M2/M3 mortality execution is defined normatively in [`competing-mortality-risks-v1.md`](competing-mortality-risks-v1.md).

## 1. Purpose

The pre-repair M2 implementation allowed several separate-looking defects to arise from one ambiguity: the annual boundary simultaneously acted as an ageing boundary, a mortality draw, a fertility draw, a birth-spacing clock and a parentage-locality snapshot. The resulting behavior could skip the infant mortality interval, silently quantize a day-valued birth-spacing parameter, let a zero-duration same-day M4 relocation redefine the male parent pool, reset newborn condition to 1000, and let implementation order decide whether mortality erased fertility opportunity.

The first M2 repair made those annual semantics explicit. A later repair, introduced at model-semantics v15, removed background-mortality execution from the year-end M2 pass and resolved it instead as elapsed risk jointly with M3 condition-mediated mortality at each M3 mortality boundary. This contract describes that current combined state rather than the superseded annual-boundary mortality implementation.

Founder demographic/kin prehistory is handled by the separate declared-founder contract rather than hidden inside annual transition code.

From model semantics v22, both the synthetic and spatial authoritative hosts pass declared founder reproductive history into the same annual fertility-spacing finalizer, including after spatial checkpoint/resume. Immutable v0.3.3/v21 remains the historical baseline before that cross-host parity repair; its audit conclusions remain historical evidence rather than being rewritten by this repair.

## 2. Annual interval and schedule selection

For a model year ending at day `t`, the annual demographic exposure interval is the half-open interval:

`[t - 365, t)`

The annual M2 fertility/parentage stage is evaluated only after a completed model year, so `t` is a positive multiple of 365 days.

A person's age-specific M2 background-mortality schedule entry and a female's fertility schedule entry are selected using age at **the start of the model-year interval**, `t - 365`, not age at `t`.

Consequences:

- a model-born child first exposed during the next model year uses the age-0 `[0, 1)` mortality band rather than skipping directly to age 1;
- every annual schedule transition uses the age band that governed the model year rather than the band entered at its end;
- a person who crosses a band boundary during a model year retains the start-of-year band's configured annual probability for that model year's background-risk partitioning and annual fertility opportunity.

The final point is a deliberate discretization assumption. AnthroSim does not switch to a new M2 age band part-way through the same model year merely because M3 mortality is evaluated subannually.

## 3. Background mortality probability and execution

For a person alive at the start of `[t - 365, t)`, let `p_a` be the configured M2 `annual_probability_per_million` selected from the mortality schedule using age at `t - 365`.

`p_a` is the configured **annual cumulative background-mortality probability for that model year**. It is not an instantaneous hazard and it is no longer interpreted as a Bernoulli draw performed only at annual boundary `t`.

Current execution partitions that annual probability across the M3 mortality intervals within the year using the elapsed-time rule defined in [`competing-mortality-risks-v1.md`](competing-mortality-risks-v1.md). For elapsed time `x` days since the start of the model year:

```text
S(x) = 1 - p_a * x / 365
```

For one M3 mortality interval `[s,e)` measured from the start of that same model year:

```text
q_background(s,e) = 1 - S(e) / S(s)
```

The implementation evaluates this with exact integer-rational arithmetic. Multiplying conditional survival over a complete partition of the model year recovers exactly `1 - p_a`, so changing only the number of M3 resource periods does not multiply the configured annual background probability.

At each M3 mortality boundary, this elapsed M2 background risk is resolved jointly with the current M3 condition-mediated risk. No separate M2 background-mortality draw is performed again at year end.

This is a declared discrete elapsed-time approximation. It is **not** a fitted or continuous-time hazard model, and AnthroSim does not sample a biologically meaningful within-interval death time.

## 4. Fertility probability and competing mortality

For a female record that existed at the start of the model-year interval, let `f_a` be the configured annual fertility probability selected using her age at `t - 365`.

The current model interprets `f_a` as a **conditional annual live-birth opportunity at boundary `t` among females that remain alive through the model year's preceding mortality boundaries**, subject additionally to birth-spacing eligibility and an eligible local male parent pool.

The causal ordering is therefore expressed in terms of survival state, not an obsolete year-end mortality function-call priority:

1. throughout `[t - 365, t)`, elapsed M2 background mortality and M3 condition-mediated mortality are resolved jointly at M3 mortality boundaries;
2. deaths recorded during those boundaries remove records from subsequent state;
3. after all subannual mortality/resource boundaries for the model year have completed, the annual M2 stage evaluates fertility and parentage for the surviving eligible records;
4. successful births are recorded at `t`.

A female that dies during the model year cannot subsequently give birth at the year-end M2 stage, and a male that dies during the model year cannot be selected as the year-end male parent. There is no additional background-mortality draw at `t` that competes with fertility on that same annual function call.

When condition-mediated mortality is absent, complete-year survival under the partitioned M2 background process remains exactly `1 - p_a`. For an otherwise eligible female with configured annual background probability `p_a` and conditional fertility probability `f_a`, ignoring spacing and male availability, the unconditional probability of surviving the model year and then recording a birth therefore remains:

`P(birth at t) = (1 - p_a) * f_a`

When condition-mediated mortality is active, the corresponding birth probability is additionally conditioned on surviving that explicit cause-specific process. It must not be reduced to the two-term expression above unless the M3 condition-mediated risk for the year is zero.

Any empirical schedule intended for this implementation must be transformed or estimated consistently with these conditioning rules. In particular, an all-cause empirical mortality schedule must not be inserted unchanged as M2 background mortality while explicit M3 condition-mediated mortality is also active; doing so would double count causes already represented elsewhere.

## 5. Birth spacing

`minimum_birth_spacing_days` remains a requested lower-bound duration in days, but births occur only on annual M2 boundaries in this model.

The executable spacing is therefore the smallest whole number of 365-day annual boundaries that is at least the requested duration:

`effective_spacing_days = ceil(requested_spacing_days / 365) * 365`

with zero mapping to zero.

Examples:

| requested days | executable days |
| ---: | ---: |
| 0 | 0 |
| 365 | 365 |
| 366 | 730 |
| 730 | 730 |
| 731 | 1095 |
| 1278 | 1460 |
| 1460 | 1460 |

The engine exposes this normalization as an authoritative derived function and tests it explicitly. The versioned M2 observability report also writes both the requested and executable values into ordinary run-facing analysis output. The raw requested value must not be described as if births can resume on that exact subannual day.

A future research mode requiring exact day-scale interbirth intervals needs a genuinely subannual/event-time M2 redesign rather than a different comparison operator at the annual boundary.

## 6. Parentage locality versus same-day M4 relocation

M2 parentage is based on **persistent residence immediately before any M4 relocation recorded on the same annual boundary day**. A destination entered at day `t` contributes zero elapsed exposure to `[t - 365, t)` and therefore cannot redefine the male parent pool for the fertility transition accrued over that interval.

The implementation reconstructs the pre-M4 residence snapshot from authoritative same-day `HouseholdMigration` events. M9 temporary visitor/transit presence remains excluded from M2 parentage; parentage uses persistent residence, not temporary physical co-presence.

Birth state itself is recorded at the mother's current persistent household residence at `t`. Therefore, when an M4 move occurs on the same boundary, parentage exposure can refer to the pre-move residence while the newborn immediately inherits the already-updated household residence. This is intentional: parentage locality represents exposure over the elapsed interval, whereas the newborn's stored residence is boundary state.

`Death.cell` retains its existing event/state meaning: the person's persistent residence when the mortality transition is resolved. Current M2 background mortality is partitioned over M3 intervals and can therefore be recorded before the annual boundary. M2 background mortality is not spatially parameterized, so the model does not infer a separate spatial exposure location for that risk. If mortality later becomes spatially varying, an explicit exposure-location field/state should be introduced rather than overloading `Death.cell` and breaking event/state reconciliation.

## 7. Newborn condition

M2 does not create newborns at perfect condition by fiat.

A newborn inherits the female parent's condition at the birth boundary. This is a minimal transparent null proxy that preserves the M3 condition signal already present in the household without adding an uncalibrated neonatal physiology model.

This rule does **not** make fertility itself condition-dependent. It only defines the newborn's initial downstream M3/M4 condition state.

Cross-system acceptance tests verify the rule at female-parent conditions 900, 500 and 100 permille. They also follow a very-low-condition birth into a deterministic fully unsupplied M3 period: the female parent and newborn begin with the same condition, receive the same condition loss, and undergo the same forced condition-mediated mortality outcome rather than the newborn retaining a hidden perfect-condition survival advantage. A separate test verifies that maternal inheritance contributes to the authoritative household mean condition used by M4 and therefore preserves the corresponding condition-pressure signal rather than suppressing it through a synthetic 1000-permille reset.

These tests verify causal consistency of the declared null rule. They do not establish that maternal condition inheritance is an empirically accurate neonatal physiology model.

A future empirically grounded neonatal-condition model may replace this proxy only with explicit evidence, parameters, uncertainty and a new compatible scientific contract.

## 8. Founder initialization

Founder history is not inferred implicitly by the annual M2 transition.

`SyntheticValidationV1` deliberately remains a labelled zero-history synthetic initializer. Research-facing runs can instead use `declared_founder_state_v1`, defined by [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md), to carry provenance-bearing founder ages, reproductive state, signed pre-run last-birth timing and explicit living direct-parent links into authoritative Population/experiment state.

M2 consults declared pre-run `lastBirthDay` only until a real model-period birth supersedes it. Declared direct-parent links are immediately available to the M4 kin proxy. The declaration is content-bound for persistence/reproducibility rather than reconstructed from hidden pseudo-events.

This removes the previous requirement to pretend every founder has no reproductive or kin history. It does **not** make an arbitrary founder declaration empirically plausible; the supplied initialization still requires study-specific evidence, uncertainty and sensitivity analysis.

## 9. Opportunity observability

The versioned derived report defined in [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md) reconstructs the M2 opportunity funnel from authoritative day-zero Population, EventLog history, immutable experiment configuration and final checkpoint Population.

Under the current competing-risk semantics it distinguishes at least:

- M3-aligned M2 background-mortality risk-interval exposures and resulting `demographic_mortality` deaths by interval-start age band;
- configured mortality-risk intervals per year and the declared order-invariant competing-risk process;
- surviving female records entering the year-end fertility stage;
- non-zero fertility-schedule eligibility;
- spacing eligibility using the explicit requested-to-executable rule;
- eligible-male availability under the pre-same-day-M4 parentage locality;
- fertility draws attempted;
- replayed stochastic draw successes/failures;
- successful births;
- person-record-limit blocking/truncation;
- model-period and declared-prehistory-to-first-birth interbirth intervals; and
- completed-fertility summaries with explicit censoring.

The derivation independently replays the relevant deterministic RNG streams and reconciles reconstructed demographic history against final Population state. A mismatch is an analysis error rather than a silently estimated denominator.

These quantities remain separate so later calibration cannot compensate invisibly for male availability, founder history, mortality competition or spacing suppression by inflating the fertility schedule.

## 10. Verification tests required by this contract

The repair suite includes model-contract tests for:

- age-0 background-mortality exposure of a model-born child during its first later model year;
- age-1 second-year risk during the following model year;
- founder records immediately below/at an age-band boundary;
- equivalent fertility age-band boundaries;
- exact preservation of annual M2 background survival when partitioned over 1, 4, 12 or 365 M3 mortality intervals;
- zero/certain cause-specific mortality edges and order-invariant joint M2/M3 competing-risk behavior;
- proof that year-end M2 fertility/parentage does not redraw background mortality;
- requested-to-executable birth-spacing normalization around 365-day boundaries;
- survival-conditioned fertility cases consistent with the current mortality process;
- same-day M4 relocation with origin-only and destination-only eligible males;
- the corresponding non-annual move after elapsed destination residence;
- exact newborn maternal-condition inheritance at high, medium and very-low condition;
- severe-scarcity M2→M3 regression proving no hidden newborn survival advantage;
- newborn contribution to household mean condition and the resulting M4 condition pressure;
- deterministic demography-observability replay and final-state reconciliation; and
- end-to-end CLI derivation/checking from a normal run bundle.

Synthetic tests verify that software implements this declared model and that the repaired M2 condition state propagates coherently into M3 and M4. They do not empirically validate the model for any archaeological population.

## 11. Compatibility and interpretation

The original M2 transition-semantics repair changed authoritative demographic meaning relative to the v0.3.0 baseline and therefore advanced `MODEL_SEMANTICS_ID` when it was introduced. The later v15 competing-risk repair changed authoritative M2 mortality execution from a year-end annual draw to elapsed M3-boundary background risk resolved jointly with condition-mediated mortality. Subsequent authoritative repairs have advanced the repository identity further.

Historical descriptions of the pre-v15 annual-boundary mortality implementation remain useful only when explicitly labelled as superseded behavior. Current-facing interpretation must follow this document together with the normative competing-risk contract.

The repair programme is specifically intended to improve **verification** and interpretability. Empirical **validation** remains study-specific and future work. Exact Git provenance continues to identify the implementation used for every run.
