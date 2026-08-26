# M3/M4 response-time contract v1

**Status:** normative executable contract for `anthrosim-model-semantics-v9`  
**Scope:** M3 condition response, M3 scarcity-mortality timing, independent M4 decision opportunities, and merged subannual scheduling  
**Scientific status:** implementation/model-contract specification; **not empirical validation**

## Purpose

The v8 resource-time repair made annual resource accounting coherent, but deliberately left one major timing defect open: `resources.periodsPerYear` still controlled several processes with different scientific meanings.

Before v9, increasing the M3 resource partition count also increased:

- the number of condition recovery/loss updates;
- the number of condition-mediated scarcity-mortality draws; and
- the number of permanent M4 relocation opportunities.

That made a numerical/resource-integration control into a hidden behavioural and physiological rate parameter. Issue #204 identified this as a TRACE model-structure defect.

v9 separates those meanings. It does **not** claim that all results must be identical under every resource partition. Resource settlement timing can still alter the causal trajectory when resources, condition, mobility or temporary presence vary through the year. The required invariant is narrower: changing M3 temporal partition must not by itself multiply an independently defined response or decision rate.

The v8 annual resource-accounting and seasonal-integration rules in [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md) remain normative unless explicitly superseded below.

## 1. Two independent subannual clocks

AnthroSim now has two independently configured fixed schedules:

### M3 resource-integration clock

`resources.periodsPerYear = P`

M3 interval `i` is:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`

relative to the model-year start.

This controls when elapsed resource regeneration, demand, condition response and condition-mediated scarcity survival are settled. It does not define the number of permanent-migration decisions.

### M4 permanent-migration decision clock

`migration.decisionPeriodsPerYear = D`

M4 decision interval `j` uses the same deterministic fixed-boundary construction:

`[ floor(j * 365 / D), floor((j + 1) * 365 / D) )`

The decision occurs at that interval's end. The synthetic validation default is `D = 4`.

Changing `P` while holding `D` fixed therefore changes M3 resolution but not the configured number of M4 opportunities. Changing `D` is an intentional behavioural-model intervention and may change movement outcomes.

Both `P` and `D` are constrained to `1..=365`.

## 2. M4 resource-support demand belongs to the M4 interval

Under v8, M3 and M4 shared one resource-period boundary, so M4 used the M3 current-period demand share. Once the clocks are independent, that interpretation is no longer coherent on non-coincident days.

v9 therefore defines M4's resource-support demand from the annual per-person need allocated over the **M4 decision interval**:

`demand_j = C_Q(b_j) - C_Q(a_j)`

where:

`C_Q(t) = floor(Q * t / 365)`

and `Q = annualNeedUnitsPerPerson`.

This remains the same cumulative elapsed-day annual-allocation rule used by M3; only the interval belongs to the M4 decision clock rather than the M3 integration clock.

The runtime reconciles both the declared M4 decision index and the actual decision day. A context whose day does not align with `decisionPeriodsPerYear`, or whose index/day imply different annual demand shares, fails as an internal model error rather than silently guessing.

This alignment is numerical/model consistency, not empirical validation of the M4 resource-utility equation.

## 3. Canonical reference-quarter response coefficients

The existing public resource fields are retained to avoid an unnecessary input-format rename in this hardening slice:

- `conditionRecoveryPerPeriod`
- `maxConditionLossPerPeriod`
- `maxScarcityMortalityProbabilityPerMillion`

Under v9 their scientific interpretation is no longer “whatever one configured M3 period happens to be.” They are **reference-quarter coefficients** attached to four canonical intervals:

| Reference interval | Model days | Duration |
|---:|---|---:|
| 0 | `[0,91)` | 91 days |
| 1 | `[91,182)` | 91 days |
| 2 | `[182,273)` | 91 days |
| 3 | `[273,365)` | 92 days |

The names remain historical API names; the normative v9 meaning is defined here. Research reports must describe the reference-quarter meaning rather than implying these values scale with arbitrary `periodsPerYear`.

These coefficients remain synthetic validation assumptions unless separately evidence-grounded. v9 repairs their temporal meaning; it does not make them physiologically realistic.

## 4. Condition recovery/loss uses elapsed interval response

For a reference-quarter condition quantity `R`, v9 constructs a linear cumulative response inside each canonical reference quarter.

For an arbitrary M3 interval `[a,b)`, the executable response is the sum of the cumulative differences over every overlapped reference quarter. At each complete reference-quarter boundary, the accumulated response is exactly `R`.

Consequences:

- `P = 4` reproduces the configured reference-quarter quantity exactly at each quarter;
- over a complete year, a continuously applicable response has a total budget of exactly `4R` regardless of whether `P = 1, 4, 12, 365`, or another allowed partition;
- changing M3 partition therefore cannot multiply annual condition recovery/loss merely by creating more boundaries.

The resource-supply rule remains causal:

- zero modeled need is condition-neutral;
- positive full supply applies the elapsed recovery amount;
- positive partial supply applies the existing supply-deficit proportion to the elapsed maximum-loss amount.

Because supply state can differ between partitions and condition is bounded to `0..1000`, realised trajectories need not be identical even though the response budget is no longer frequency-multiplied.

## 5. Scarcity mortality uses survival-equivalent interval conversion

At each M3 settlement, the person's current condition still determines a **reference-quarter** condition-mediated probability `q`:

`q = condition_deficit_fraction × maxScarcityMortalityProbabilityPerMillion`

where `q` is represented relative to one canonical reference quarter.

For an arbitrary sub-interval inside a reference quarter, v9 treats cumulative incidence as linear in elapsed days within that reference quarter and derives the conditional survival probability for the interval exactly as an integer rational number.

If one reference quarter has length `L`, interval-local offsets are `x0..x1`, probability scale is `M = 1,000,000`, and the reference-quarter probability is `q`, then conditional survival over that sub-interval is:

`S(x0,x1) = (M*L - q*x1) / (M*L - q*x0)`

For an M3 interval crossing reference-quarter boundaries, the overlapping conditional-survival fractions are multiplied. The executable death probability is `1 - S_interval` and the stochastic comparison uses that rational value directly rather than first rounding to parts-per-million.

This construction has the required partition property at fixed condition:

- `P = 4` reproduces `q` exactly for each canonical reference quarter;
- the complete-year survival is `(1-q)^4` for `P = 1, 4, 12, 365`, or any other valid partition of the same year;
- changing only M3 partition therefore does not multiply the fixed-condition mortality hazard.

The authoritative death event retains a `probabilityPerMillion` field for observability. For a non-quarter interval this field stores the deterministic ceiling of the exact rational interval probability; the random draw itself uses the exact rational probability.

### Important causal limitation

This section repairs **timing**, not cause attribution. The hazard still reads the shared `condition` scalar. Issue #200 remains open because M4 travel can reduce that scalar and a later death may still be labelled `ResourceScarcity` even if the deficit was not resource-derived.

Likewise, issue #208 remains open: when M3 and annual M2 mortality coincide, cause-specific competing-risk attribution still requires a separate explicit contract.

## 6. Merged fixed-boundary scheduler

The authoritative core host and the evidence-grounded spatial host both merge the independent M3 and M4 schedules within each model year.

For the next due fixed day:

1. process any M9 temporary boundaries strictly before that day;
2. if M3 is due, settle the elapsed M3 interval, including resource accounting, condition response and M3 condition-mediated mortality;
3. process due M9 temporary transition/start semantics for that day;
4. if M4 is due, evaluate eligible permanent relocation using the M4 decision interval;
5. after the year's subannual schedules complete, run the M2 annual demographic boundary.

When M3 and M4 share a day, this preserves the established M3 → M9 → M4 ordering. When only one clock is due, that process can occur without inventing a boundary for the other.

The main `Simulation` and `SpatialLandscapeSimulation` must implement the same timing contract. A landscape-bound run must not silently revert to the old “M4 at every M3 boundary” behaviour.

## 7. What temporal partition should and should not change

### Required invariants when only M3 `P` changes

Holding all scientific inputs and `D` fixed:

- fixed annual M3 demand remains exactly conserved;
- unconstrained annual regeneration potential remains governed by the v8 mean-preserving integration contract;
- a continuously applicable condition response has the same complete-year response budget;
- fixed-condition scarcity survival has the same complete-year probability;
- the configured number of M4 decision opportunities remains exactly `D` per complete year;
- M4 decision boundaries and their own annual-demand allocation are determined by `D`, not `P`.

### Effects that may legitimately remain partition-sensitive

v9 does not assert trajectory identity across resource resolutions. Partition can still affect outcomes through declared causal sequencing, including:

- when stock becomes available or is consumed;
- cell-capacity clipping of regeneration;
- when condition changes relative to other processes;
- a condition-dependent hazard when condition itself evolves within the year;
- the timing of M9 residence/visitor/transit demand allocation;
- M4 observing different current stock/condition state at its fixed decision day;
- extinction occurring before a later scheduled process.

Those are model-resolution/scheduling sensitivities to measure. They are distinct from the removed artifact where simply adding numerical M3 boundaries automatically added more M4 decisions or repeated a nominally unchanged physiological probability.

## 8. Configuration and compatibility

v9 changes the versioned input contract:

- `ExperimentConfig` schema: `8 -> 9`
- `ResourceConfig` schema: `2 -> 3`
- `MigrationConfig` schema: `1 -> 2`
- `MigrationConfig` gains `decisionPeriodsPerYear`

The synthetic validation migration default sets `decisionPeriodsPerYear = 4`, preserving the old baseline opportunity count while making that count explicit and independently configurable.

The authoritative model-semantics identity changes:

`anthrosim-model-semantics-v8 -> anthrosim-model-semantics-v9`

A v8 checkpoint cannot be scientifically continued under v9 and must fail closed at the existing model-semantics compatibility boundary.

Package versioning remains separate from this development semantics change.

## 9. Required verification

The v9 implementation must verify at minimum:

- resource partitions `P = 1, 4, 12, 365` with fixed `D = 4` produce exactly four M4 decision boundaries per complete year;
- changing `D` changes M4 opportunity count independently of `P`;
- reference-quarter condition response sums to the same annual budget under `P = 1, 4, 12, 365`;
- controlled fully supplied and fully unsupplied one-person cases do not gain extra annual condition response merely from finer M3 partitioning;
- fixed-condition scarcity survival composes identically under `P = 1, 4, 12, 365`, including `q = 0` and `q = 1,000,000` edge cases;
- `P = 4` reproduces each configured reference-quarter probability exactly;
- M4 rejects an index/day mismatch in its declared decision schedule;
- both synthetic and spatial-landscape simulation hosts use the same independent-clock scheduler;
- checkpoint/resume remains exact under v9;
- cross-platform deterministic execution remains within the declared determinism contract; and
- every frozen M7/M8/M9 reference affected by v9 is reviewed before rebaseline.

Reference outputs may be updated only when their differences are explained by the v9 contract. A failing frozen reference is evidence to investigate, not automatic permission to tune or overwrite it.

## 10. What this repair does not validate

Passing this contract can show that the simulator no longer confounds one numerical resource-partition control with several independent scientific rates.

It cannot establish that:

- the four-quarter reference response clock is a realistic human physiological timescale;
- the condition-loss or recovery coefficients are empirically defensible;
- the scarcity-mortality coefficient represents real mortality;
- four M4 opportunities per year is a realistic mobility decision rate;
- the M4 decision utility represents human cognition;
- the shared condition state has correct causal attribution (#200);
- coincident M2/M3 causes are correctly allocated (#208); or
- any resulting trajectory reconstructs a real archaeological population.

Those remain evidence, calibration, validation and model-structure questions under TRACE.