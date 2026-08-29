# M3/M4 response-time contract v1

**Status:** normative executable timing contract introduced by `anthrosim-model-semantics-v9`; retained and amended through `anthrosim-model-semantics-v20`, with causal naming governed by [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md)  
**Scope:** M3 condition response, condition-mediated mortality timing, independent M4 decision opportunities, and merged subannual scheduling  
**Scientific status:** implementation/model-contract specification; **not empirical validation**

## Purpose

The v8 resource-time repair made annual resource accounting coherent, but deliberately left one major timing defect open: `resources.periodsPerYear` still controlled several processes with different scientific meanings.

Before v9, increasing the M3 resource partition count also increased:

- the number of condition recovery/loss updates;
- the number of condition-mediated mortality draws; and
- the number of permanent M4 relocation opportunities.

That made a numerical/resource-integration control into a hidden behavioural and physiological rate parameter. Issue #204 identified this as a TRACE model-structure defect.

v9 separates those meanings. It does **not** claim that all results must be identical under every resource partition. Resource settlement timing can still alter the causal trajectory when resources, condition, mobility or temporary presence vary through the year. The required invariant is narrower: changing M3 temporal partition must not by itself multiply an independently defined response or decision rate.

The v8 annual resource-accounting and seasonal-integration rules in [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md) remain normative unless explicitly superseded below. Under v10, the same timing equations remain in force, while #200 removes the false implication that this condition-dependent hazard is necessarily caused by resource scarcity. The immediate-cause semantics are normative in [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).

v20 additionally repairs the integer rounding of **partial-supply condition loss**. Before v20, each M3 boundary independently rounded any positive sub-unit loss upward. Finer resource partitioning could therefore manufacture large deterioration from the same elapsed supply deficit. v20 carries a person-specific fixed-point remainder so rounding is cumulative over actual partial-supply exposure rather than repeated at each settlement boundary.

## 1. Two independent subannual clocks

AnthroSim now has two independently configured fixed schedules:

### M3 resource-integration clock

`resources.periodsPerYear = P`

M3 interval `i` is:

`[ floor(i * 365 / P), floor((i + 1) * 365 / P) )`

relative to the model-year start.

This controls when elapsed resource regeneration, demand, condition response and condition-mediated survival are settled. It does not define the number of permanent-migration decisions.

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

The response quantities are reference-quarter coefficients attached to four canonical intervals. Under current input semantics, the relevant public resource fields are:

- `conditionRecoveryPerPeriod`
- `maxConditionLossPerPeriod`
- `maxConditionMortalityProbabilityPerMillion`

The first two retain historical `...PerPeriod` wire names, but their scientific meaning is not “whatever one configured M3 period happens to be.” The mortality field was explicitly renamed in v10 from the former v9 `maxScarcityMortalityProbabilityPerMillion` because the shared condition state can be changed by more than resource scarcity; the old scarcity-specific wire name is not accepted as a v10 alias.

The coefficients are interpreted against four canonical intervals:

| Reference interval | Model days | Duration |
|---:|---|---:|
| 0 | `[0,91)` | 91 days |
| 1 | `[91,182)` | 91 days |
| 2 | `[182,273)` | 91 days |
| 3 | `[273,365)` | 92 days |

Research reports must describe the reference-quarter meaning rather than implying these values scale with arbitrary `periodsPerYear`.

These coefficients remain synthetic validation assumptions unless separately evidence-grounded. The timing repair defines their temporal meaning; subsequent causal and rounding repairs do not make them physiologically realistic.

## 4. Condition recovery/loss uses elapsed interval response

For a reference-quarter condition quantity `R`, v9 constructs a linear cumulative response inside each canonical reference quarter.

For an arbitrary M3 interval `[a,b)`, the executable response is the sum of the cumulative differences over every overlapped reference quarter. At each complete reference-quarter boundary, the accumulated response is exactly `R`.

Consequences:

- `P = 4` reproduces the configured reference-quarter quantity exactly at each quarter;
- over a complete year, a continuously applicable response has a total budget of exactly `4R` regardless of whether `P = 1, 4, 12, 52, 365`, or another allowed partition;
- changing M3 partition therefore cannot multiply annual condition recovery/loss merely by creating more boundaries.

The resource-supply rule remains causal:

- zero modeled need is condition-neutral;
- positive full supply applies the elapsed recovery amount;
- positive partial supply applies the supply deficit to the elapsed maximum-loss amount.

### v20 fixed-point partial-supply rule

The authoritative exposed condition value remains the integer `condition`/`conditionPermille` scalar in `0..=1000`. v20 adds one causal per-person state variable, `conditionLossRemainderThousandths`, that stores an **unmaterialized M3 deterioration remainder** in `0..=999` thousandths of one condition point. It is checkpointed, validated, and included in deterministic population identity. It is not an extra public condition score and is not silently interpreted as an empirical physiological precision claim.

For one partial-supply M3 interval, let:

- `D = 1000 - suppliedPermille` be the supply deficit in permille;
- `L` be the elapsed maximum whole-condition loss budget for that interval after the reference-quarter time conversion; and
- `r` be the incoming carried remainder in thousandths.

The exact integer numerator is:

`N = D * L + r`

and the update is:

`wholeLoss = floor(N / 1000)`

`nextRemainder = N mod 1000`

The whole loss is subtracted from authoritative integer condition. The remainder is carried to the person's next M3 response boundary. This makes successive partial-supply updates associative with respect to the integer fixed-point numerator: splitting the same elapsed loss budget into more M3 boundaries cannot repeatedly round the same fractional deterioration upward.

For the audit stress case `maxConditionLossPerPeriod = 100`, a full year has elapsed maximum-loss budget `400`. Holding the supply deficit constant across the year gives the following exact end states for **every** tested partition `P = 1, 4, 12, 52, 365`:

| Supply deficit | Exact accumulated deterioration | Whole condition loss | Carried remainder |
|---:|---:|---:|---:|
| 1‰ | 0.4 | 0 | 400/1000 |
| 10‰ | 4.0 | 4 | 0 |
| 100‰ | 40.0 | 40 | 0 |
| 500‰ | 200.0 | 200 | 0 |
| 1000‰ | 400.0 | 400 | 0 |

This explicitly replaces the pre-v20 per-boundary ceiling behaviour, under which the same 1‰ annual deficit could materialize whole losses of `1`, `4`, `12`, `52`, or `365` solely because `P` changed.

The remainder follows **actual exposure**, not calendar phase. For example, a 1‰ deficit confined to the fourth canonical quarter with `R = 100` contributes exactly `100/1000` of a condition point whether that quarter is settled once or subdivided into many M3 intervals.

Boundary rules are:

- zero modeled need changes neither integer condition nor the carried remainder;
- full supply applies the elapsed whole recovery budget; an existing partial-loss remainder is retained unless condition saturates at the upper bound `1000`;
- partial supply updates whole loss plus carried remainder using the fixed-point equation above;
- zero supply therefore still applies the full elapsed maximum whole-condition loss; any already-carried sub-unit remainder remains carried unless the lower bound is reached;
- if any deterioration or other whole-condition loss saturates condition at `0`, the M3 loss remainder is cleared because no sub-unit deterioration below the lower bound can be represented;
- exact external/test assignments to integer condition clear the M3 loss remainder; newborns begin with zero remainder after inheriting the mother's authoritative integer condition;
- M4 whole-point travel loss preserves the M3 remainder unless it saturates condition at `0`.

Mortality and M4 pressure continue to consume the authoritative integer condition scalar. A non-zero remainder is latent M3 deterioration that has not yet crossed a whole-condition boundary; it changes those downstream mechanisms only when subsequent M3 exposure materializes another whole point. This finite quantization is deliberate and explicit. It is preferable to introducing settlement-frequency-dependent upward bias, but it is not evidence that real human condition changes in thousandth-point increments.

Because supply state can differ between partitions and condition is bounded to `0..1000`, realised trajectories need not be identical even though the response budget and partial-supply rounding are no longer frequency-multiplied for an otherwise identical exposure history.

## 5. Condition-mediated mortality uses survival-equivalent interval conversion

At each M3 settlement, the person's current condition determines a **reference-quarter** condition-mediated probability `q`:

`q = condition_deficit_fraction × maxConditionMortalityProbabilityPerMillion`

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

### Causal attribution boundary

This section defines **timing**, while [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md) defines cause interpretation.

Under v10 and later, the hazard reads the shared `condition` scalar and authoritative deaths serialize the immediate cause as `condition_mediated`. M3 resource balance and M4 permanent-travel cost can both alter that shared state. Therefore the death cause identifies the immediate model mediator, not the upstream process that produced the condition deficit. A `condition_mediated` death must not be re-labelled as resource scarcity merely because the hazard is evaluated during M3 settlement.

Resource shortage remains separately observable through resource accounting such as unmet need. Travel burden remains separately observable through M4 movement/travel-condition outputs. The current model does not maintain a validated causal ledger that allocates a later condition-mediated death fractionally or exclusively back to those upstream contributors.

## 6. Merged fixed-boundary scheduler

The authoritative core host and the evidence-grounded spatial host both merge the independent M3 and M4 schedules within each model year.

For the next due fixed day:

1. process any M9 temporary boundaries strictly before that day;
2. if M3 is due, settle the elapsed M3 interval, including resource accounting, condition response and condition-mediated mortality;
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
- subdivision of an otherwise identical partial-supply exposure preserves the same fixed-point whole-loss-plus-remainder result;
- fixed-condition condition-mediated survival has the same complete-year probability;
- the configured number of M4 decision opportunities remains exactly `D` per complete year;
- M4 decision boundaries and their own annual-demand allocation are determined by `D`, not `P`.

### Effects that may legitimately remain partition-sensitive

v20 does not assert trajectory identity across resource resolutions. Partition can still affect outcomes through declared causal sequencing, including:

- when stock becomes available or is consumed;
- cell-capacity clipping of regeneration;
- when condition changes relative to other processes;
- a condition-dependent hazard when condition itself evolves within the year;
- the timing of M9 residence/visitor/transit demand allocation;
- M4 observing different current stock/condition state at its fixed decision day;
- extinction occurring before a later scheduled process.

Those are model-resolution/scheduling sensitivities to measure. They are distinct from the removed artifacts where adding numerical M3 boundaries automatically added more M4 decisions, repeated a nominally unchanged physiological probability, or repeatedly rounded a sub-unit partial-supply loss upward.

## 8. Configuration and compatibility

The v9 timing repair originally changed the versioned input contract:

- `ExperimentConfig` schema: `8 -> 9`
- `ResourceConfig` schema: `2 -> 3`
- `MigrationConfig` schema: `1 -> 2`
- `MigrationConfig` gained `decisionPeriodsPerYear`

The synthetic validation migration default sets `decisionPeriodsPerYear = 4`, preserving the old baseline opportunity count while making that count explicit and independently configurable.

The v9 authoritative model-semantics identity was:

`anthrosim-model-semantics-v8 -> anthrosim-model-semantics-v9`

v10 subsequently changed the causal/serialization boundary for the shared-condition mortality mechanism. In particular, `ExperimentConfig` advanced to schema 10, `ResourceConfig` advanced to schema 4, the mortality parameter became `maxConditionMortalityProbabilityPerMillion`, and the authoritative cause became `condition_mediated`.

v20 changes deterministic continuation state without changing the experiment input schema. `Population` state schema advances from `3` to `4` because the per-person M3 condition-loss remainder is causal state required for exact continuation. The authoritative model-semantics identity advances:

`anthrosim-model-semantics-v19 -> anthrosim-model-semantics-v20`

A v19 checkpoint cannot be silently continued as v20: it lacks the fractional M3 loss state needed to determine future whole-condition transitions. Package versioning remains separate from these development semantics changes.

## 9. Required verification

The timing implementation must verify at minimum:

- resource partitions `P = 1, 4, 12, 365` with fixed `D = 4` produce exactly four M4 decision boundaries per complete year;
- changing `D` changes M4 opportunity count independently of `P`;
- reference-quarter condition response sums to the same annual budget under `P = 1, 4, 12, 365`;
- controlled fully supplied and fully unsupplied one-person cases do not gain extra annual condition response merely from finer M3 partitioning;
- partial-supply fixed-point response is identical in whole-loss-plus-remainder state under `P = 1, 4, 12, 52, 365` for at least deficits `1, 10, 100, 500, 1000‰`;
- a sub-year partial-supply exposure retains the same fixed-point result when subdivided at different boundary counts, demonstrating that the remainder follows exposure rather than calendar phase;
- zero supply retains the full elapsed maximum-loss endpoint and full supply retains the elapsed recovery endpoint;
- equivalent controlled partial-supply histories feed the same authoritative integer condition into condition-mediated mortality and M4 pressure;
- fractional M3 loss state is validated, serialized/checkpointed, included in deterministic population identity, initialized safely for founders/newborns, and handled explicitly at condition bounds;
- fixed-condition condition-mediated survival composes identically under `P = 1, 4, 12, 365`, including `q = 0` and `q = 1,000,000` edge cases;
- `P = 4` reproduces each configured reference-quarter probability exactly;
- M4 rejects an index/day mismatch in its declared decision schedule;
- both synthetic and spatial-landscape simulation hosts use the same independent-clock scheduler;
- checkpoint/resume remains exact under the declared current semantics;
- cross-platform deterministic execution remains within the declared determinism contract; and
- frozen M7/M8/M9 references affected by a semantics/schema change are causally reviewed before rebaseline.

The v10 cause-attribution repair additionally requires controlled resource-only, travel-only/full-resource, mixed-pathway, and migration-enabled/disabled tests demonstrating that the mortality output is condition-mediated rather than automatically resource-attributed.

Reference outputs may be updated only when their differences are explained by the declared contract. A failing frozen reference is evidence to investigate, not automatic permission to tune or overwrite it.

## 10. What this contract does not validate

Passing this contract can show that the simulator no longer confounds one numerical resource-partition control with several independent scientific rates, that partial-supply rounding no longer manufactures extra deterioration merely from numerical subdivision, and that the current shared-condition hazard is not mislabeled as resource-specific.

It cannot establish that:

- the four-quarter reference response clock is a realistic human physiological timescale;
- thousandth-point fixed-point bookkeeping corresponds to measurable human physiology;
- the condition-loss or recovery coefficients are empirically defensible;
- the condition-mediated mortality coefficient represents real human mortality;
- four M4 opportunities per year is a realistic mobility decision rate;
- the M4 decision utility represents human cognition;
- the shared condition state can identify which upstream mechanism caused a later death; or
- any resulting trajectory reconstructs a real archaeological population.

Those remain evidence, calibration, validation and model-structure questions under TRACE.
