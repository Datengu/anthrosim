# M2 demographic-time contract v1

Status: authoritative scientific/model contract for the repaired annual M2 transition semantics.

This document defines what AnthroSim's current demographic probabilities and annual demographic boundary mean. It is a **verification contract**, not an empirical validation claim. The current M2 implementation remains a deliberately coarse annual discrete-time model; it must not be described as a continuous-time demographic hazard model.

The versioned downstream validation/diagnostic surface for these semantics is defined separately in [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md).

## 1. Purpose

The pre-repair M2 implementation allowed several separate-looking defects to arise from one ambiguity: the annual boundary simultaneously acted as an ageing boundary, a mortality draw, a fertility draw, a birth-spacing clock and a parentage-locality snapshot. The resulting behavior could skip the infant mortality interval, silently quantize a day-valued birth-spacing parameter, let a zero-duration same-day M4 relocation redefine the male parent pool, reset newborn condition to 1000, and let implementation order decide whether mortality erased fertility opportunity.

This contract makes those semantics explicit before empirical calibration. Founder demographic/kin prehistory is handled by the separate declared-founder contract rather than hidden inside annual transition code.

## 2. Annual interval

For an M2 boundary at day `t`, the demographic exposure interval is the half-open interval:

`[t - 365, t)`

M2 is currently evaluated only at completed annual boundaries, so `t` is a positive multiple of 365 days.

A person's age-specific mortality and female fertility schedule entries are selected using that person's age at **the start of the exposure interval**, `t - 365`, not age at `t`.

Consequences:

- a model-born child first exposed at the next annual boundary is evaluated in the age-0 `[0, 1)` mortality band rather than skipping directly to age 1;
- every schedule transition uses the age band that governed the elapsed interval rather than the band entered at its end;
- a person who crosses a band boundary during an annual interval retains the start-of-interval band's probability for that full discrete transition.

The last point is a deliberate discretization assumption. It must not be confused with piecewise continuous exposure within the year.

## 3. Mortality probability

For a person alive at the start of `[t - 365, t)`, let `q_a` be the configured `annual_probability_per_million` selected from the mortality schedule using age at `t - 365`.

The current model interprets `q_a` as the probability that the person undergoes the **annual demographic mortality transition at boundary `t`** after surviving to the start of the interval.

It is not an instantaneous hazard and no within-year death time is sampled. A successful draw records demographic death on day `t`.

## 4. Fertility probability and competing mortality

For a female record that existed at the start of the interval, let `p_a` be the configured fertility probability selected using her age at `t - 365`.

The current model interprets `p_a` as a **conditional annual live-birth opportunity at boundary `t` among females that survive the demographic mortality transition for that interval**, subject additionally to birth-spacing eligibility and an eligible local male parent pool.

Mortality is therefore a competing transition with explicit priority:

1. mortality is drawn for records present at interval start;
2. females and males that die at `t` are unavailable to fertility/parentage at `t`;
3. fertility is drawn only for surviving eligible females;
4. successful births are recorded at `t`.

For an otherwise eligible female with mortality probability `q` and conditional fertility probability `p`, ignoring spacing and male availability, the probability of a recorded birth is therefore:

`P(birth at t) = (1 - q) * p`

A female parent cannot both give birth and die from M2 demographic mortality on the same annual boundary under this v1 contract, and a male that dies at that boundary cannot be selected as the male parent. This is a declared coarse competing-transition assumption, not an empirical statement about real within-year ordering.

Any empirical schedule intended for this implementation must therefore be transformed or estimated consistently with this conditional-survival meaning. A directly observed unconditional annual live-birth probability must not be inserted without checking that the conditioning matches.

A later model may adopt subannual or continuous-time competing hazards, but that would be a new model-semantics identity rather than a silent implementation change.

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

`Death.cell` retains its existing event/state meaning: the person's persistent boundary-state residence when the death transition is recorded. M2 mortality is not currently spatially parameterized, so the model does not infer a separate spatial exposure location for the annual mortality probability. If mortality later becomes spatially varying, an explicit exposure-location field/state should be introduced rather than overloading `Death.cell` and breaking event/state reconciliation.

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

It distinguishes at least:

- mortality exposures and M2 demographic deaths by interval-start age band;
- surviving female records entering fertility;
- non-zero fertility-schedule eligibility;
- spacing eligibility using the explicit requested-to-executable rule;
- eligible-male availability under the pre-same-day-M4 parentage locality;
- fertility draws attempted;
- replayed stochastic draw successes/failures;
- successful births;
- person-record-limit blocking/truncation;
- model-period and declared-prehistory-to-first-birth interbirth intervals; and
- completed-fertility summaries with explicit censoring.

The derivation independently replays the `demography/fertility` RNG stream and reconciles reconstructed demographic history against final Population state. A mismatch is an analysis error rather than a silently estimated denominator.

These quantities remain separate so later calibration cannot compensate invisibly for male availability, founder history, mortality competition or spacing suppression by inflating the fertility schedule.

## 10. Verification tests required by this contract

The repair suite includes model-contract tests for:

- age-0 mortality exposure of a model-born child at its first later annual boundary;
- age-1 second-year risk at the following boundary;
- founder records immediately below/at an age-band boundary;
- equivalent fertility age-band boundaries;
- requested-to-executable birth-spacing normalization around 365-day boundaries;
- high-mortality/high-fertility cases proving the declared conditional-survival equation;
- same-day M4 relocation with origin-only and destination-only eligible males;
- the corresponding non-annual move after elapsed destination residence;
- exact newborn maternal-condition inheritance at high, medium and very-low condition;
- severe-scarcity M2→M3 regression proving no hidden newborn survival advantage;
- newborn contribution to household mean condition and the resulting M4 condition pressure;
- deterministic demographic-observability replay and final-state reconciliation; and
- end-to-end CLI derivation/checking from a normal run bundle.

Synthetic tests verify that software implements this declared model and that the repaired M2 condition state propagates coherently into M3 and M4. They do not empirically validate the model for any archaeological population.

## 11. Compatibility and interpretation

The transition-semantics repair changed authoritative demographic meaning relative to the v0.3.0 baseline and therefore advanced `MODEL_SEMANTICS_ID` when it was introduced. Subsequent authoritative repairs have advanced the repository identity further.

The newborn-condition acceptance work described here adds verification coverage and documentation only. It does **not** change authoritative simulation trajectories and therefore does not require a new model-semantics identity. Any canonical M7/M8/M9 output change caused by this acceptance-test work must be treated as a regression requiring investigation rather than rebaselining.

The repair programme is specifically intended to improve **verification** and interpretability. Empirical **validation** remains study-specific and future work. Exact Git provenance continues to identify the implementation used for every run.
