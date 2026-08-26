# M3 resource-time contract v1

**Status:** normative executable contract for `anthrosim-model-semantics-v8`  
**Scope:** resource-period boundaries, annual quantity allocation, seasonal regeneration, M3/M4 demand alignment, and zero-demand condition handling  
**Scientific status:** implementation/model-contract specification; **not empirical validation**

## Purpose

M3 previously mixed several incompatible meanings of a “resource period”:

- M3 divided annual need using integer quotient/remainder by period count;
- M4 independently estimated period need with `ceil(annual / periods)`;
- regeneration sampled one seasonal endpoint and then divided the sampled annual value by period count; and
- a zero-demand interval implicitly treated `0 / 0` as fully supplied and therefore improved condition.

Those rules made annual totals and downstream behavior depend on numerical scheduling details in ways that were not part of the intended resource hypothesis. v8 defines one explicit time/accounting contract shared across M3 and M4.

This contract addresses #180, #189 and #199. It does **not** resolve #204, #200, #208 or the remaining downstream acceptance scope of #201.

## 1. Authoritative resource-period boundaries

A model year contains exactly 365 integer model days. For `P = periodsPerYear`, resource period `i`, where `0 <= i < P`, is the half-open interval:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`

relative to the start of that model year.

The period end is the scheduler boundary at which M3 processes that interval. These are the same boundaries used by M9 resource-presence accounting.

For `P = 4`, the intervals are therefore:

| Period | Half-open day interval | Duration |
|---:|---|---:|
| 0 | `[0, 91)` | 91 days |
| 1 | `[91, 182)` | 91 days |
| 2 | `[182, 273)` | 91 days |
| 3 | `[273, 365)` | 92 days |

The model must not pretend these are four identical-duration quarters when allocating quantities defined per year.

## 2. Fixed annual integer quantities

For a fixed annual integer quantity `Q`, define cumulative allocated quantity at elapsed day `t` as:

`C_Q(t) = floor(Q * t / 365)`

The quantity assigned to resource period `[a,b)` is:

`Q_period = C_Q(b) - C_Q(a)`

This rule has three required properties:

1. all period allocations are non-negative integers;
2. allocations respect the scheduler's actual elapsed-day boundaries; and
3. the complete model year sums exactly to `Q`.

Examples for four periods:

| Annual quantity | Executable period allocation |
|---:|---|
| 0 | `0, 0, 0, 0` |
| 1 | `0, 0, 0, 1` |
| 4 | `0, 1, 1, 2` |
| 100 | `24, 25, 25, 26` |

This is intentionally different from assigning an integer remainder to the first `N` periods. The timing is determined by elapsed model days, not arbitrary period ordinal.

## 3. M3 demand and M4 resource-support demand

`annualNeedUnitsPerPerson` is a fixed annual quantity and uses the allocation rule above.

At a resource boundary, M3 and M4 must use the **same current-period per-person need**. M4 is not allowed to derive a separate `ceil(annual / P)` approximation.

This does not mean M4's resource utility is empirically validated. It means the decision model and the resource-consumption model no longer disagree about the amount of demand associated with the same simulated interval.

A resource boundary that cannot be reconciled to the configured resource-period schedule is an internal model error rather than an invitation to guess a period demand.

## 4. Seasonal regeneration is mean-preserving redistribution

The configured annual regeneration quantity is first formed from:

- cell baseline productivity;
- `annualRegenerationUnitsPerProductivity`;
- `productivityScalePermille`; and
- environmental-stress scaling.

The existing synthetic triangular seasonal curve then determines **when within the year** that fixed annual quantity becomes available. It must not change the unconstrained annual total merely because the phase or `periodsPerYear` changes.

For each cell, the seasonal factor is evaluated on every integer model day of the 365-day year. For a period `[a,b)`, let `W(a,b)` be the sum of those daily factors over the interval and `W_year` the sum over the complete year.

Define cumulative seasonal allocation:

`S_Q(t) = floor(Q * W(0,t) / W_year)`

and period regeneration potential:

`Q_period = S_Q(b) - S_Q(a)`

Consequences:

- changing seasonal **phase** changes within-year timing but not the unconstrained annual total;
- changing resource-period resolution changes temporal aggregation but not the unconstrained annual total;
- zero seasonal amplitude reduces exactly to the fixed elapsed-day allocation in section 2;
- finite cell stock capacity may still prevent potential regeneration from being realized, because capacity is a separate causal constraint.

This is a numerical integration/accounting repair, not a claim that the triangular seasonal curve is ecologically realistic.

## 5. Integer determinism and performance

The implementation remains integer-only. No floating-point integration is introduced.

The phase-zero seasonal cumulative curves are deterministically precomputed by amplitude and circularly shifted for cell phase. This makes period integration an O(1) prefix-sum query per cell/period rather than adding a 365-day inner loop to every resource step.

The precomputed table is derived entirely from model constants and carries no hidden fitted state.

## 6. Zero-demand condition semantics

When a household's executable need for a resource interval is zero, that interval is **condition-neutral**:

`condition_after = condition_before`

Zero need is not evidence of full provisioning and therefore does not trigger `conditionRecoveryPerPeriod`.

For positive need:

- full supply retains the configured recovery rule;
- partial supply retains the configured proportional deficit/loss rule.

This fixes the prior accidental `0 / 0 -> fully supplied` behavior.

Important limitation: condition recovery/loss parameters remain specified **per resource period**, and scarcity mortality remains evaluated at resource boundaries. Therefore changing `periodsPerYear` can still change annual physiological and mortality opportunity counts. That is the distinct #204 timing defect and remains open after this contract.

## 7. Mortality and condition-cause boundary

This slice does not redesign scarcity mortality. After the M3 condition update, the existing resource-boundary mortality draw still reads the person's current shared condition state.

Accordingly, #200 remains open: condition damage created by M4 travel can still influence a later death recorded under the broad `ResourceScarcity` cause. Cause decomposition belongs to the next condition/mortality slice.

Likewise, this contract does not resolve coincident M3/M2 competing-risk attribution (#208).

## 8. Model-semantics compatibility

The changes in this contract alter authoritative trajectories. In particular, they can change:

- within-year demand timing;
- regeneration timing;
- stock and unmet-need trajectories;
- condition trajectories;
- scarcity deaths; and
- M4 resource scores and therefore relocation choices.

`MODEL_SEMANTICS_ID` therefore changes from `anthrosim-model-semantics-v7` to `anthrosim-model-semantics-v8`.

Package versioning remains separate and is not bumped merely for this development slice.

Checkpoints created under v7 are scientifically incompatible with v8 continuation and must fail closed under the existing model-semantics compatibility boundary.

## 9. Required verification

The v8 implementation must verify at minimum:

- exact annual conservation for fixed quantities under `P = 1, 3, 4, 5, 12, 365`;
- very small and non-divisible quantities, including annual quantity `1` with four periods;
- M3 and M4 resolving the same current-period need at every legitimate resource boundary;
- zero-amplitude seasonal regeneration reducing exactly to fixed elapsed-day allocation;
- annual seasonal potential invariant to phase and tested resource-period resolutions when capacity is not binding;
- phase changing within-year timing when amplitude is non-zero;
- zero-demand intervals leaving reduced condition unchanged;
- positive fully supplied and undersupplied intervals retaining their declared condition directions;
- deterministic checkpoint/resume and cross-platform behavior under v8; and
- explicit review of frozen M7/M8/M9 references whose authoritative outputs may change because the semantics changed.

A frozen reference may be updated only after its difference is explained by this declared v8 contract. Reference failure is evidence to inspect, not an automatic instruction to rebaseline.

## 10. Verification versus empirical validation

Passing this contract can establish that AnthroSim executes its declared annual resource accounting consistently.

It cannot establish that:

- `100` abstract annual need units correspond to real human energy requirements;
- synthetic cell productivity represents palaeoenvironmental yield;
- the triangular seasonal curve represents any real resource ecology;
- condition recovery/loss is physiologically realistic;
- scarcity mortality probabilities are empirically defensible; or
- a resulting population trajectory is archaeologically valid.

Those require explicit evidence provenance, calibration/validation separation, uncertainty and sensitivity analysis, and study-specific corroboration under TRACE.