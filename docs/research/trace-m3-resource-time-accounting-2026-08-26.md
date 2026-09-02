# TRACE change record — M3 resource-time accounting repair

**Date:** 2026-08-26  
**Programme:** post-M9 scientific hardening / first M3 causal repair slice  
**Entering model semantics:** `anthrosim-model-semantics-v7`  
**Proposed model semantics:** `anthrosim-model-semantics-v8`  
**Scientific status:** implementation verification / conceptual-model repair; **not empirical validation**

## Purpose

This record documents the first coherent M3 repair slice after completion of the M2 hardening cluster. It addresses:

- **#180** — annual resource quantities and M4 demand did not share one periodization contract;
- **#189** — seasonal regeneration sampled one endpoint per resource period, allowing phase/resolution to alter annual potential unintentionally;
- **#199** — zero-demand intervals treated `0 / 0` as full supply and created free condition recovery.

The normative executable specification is [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md). This slice deliberately does not absorb #204, #200, #208 or the downstream acceptance scope of #201.

## 1. Problem formulation

Verification question:

> Can annual quantities be mapped deterministically to exact elapsed resource intervals, can M3 and M4 use the same current-period demand, can seasonality redistribute production without silently changing unconstrained annual total, and can a zero-demand interval remain condition-neutral?

This is a model-contract question, not a claim that the annual quantities or seasonal curve are empirically correct.

## 2. Model description

For `P = periodsPerYear`, v8 defines period `i` as:

`[floor(i * 365 / P), floor((i + 1) * 365 / P))`.

For fixed annual integer quantity `Q`, cumulative allocation after `t` elapsed days is:

`C_Q(t) = floor(Q * t / 365)`

and the period receives `C_Q(end) - C_Q(start)`.

This conserves `Q` exactly across a complete year while respecting unequal integer-day periods. At four periods/year, intervals are 91, 91, 91 and 92 days; annual need `100` therefore executes as `24, 25, 25, 26`.

M4 obtains the same current-period demand share used by M3. A direct M4 invocation on a non-resource boundary fails closed instead of inventing a period demand.

Seasonal regeneration integrates the existing triangular synthetic weighting curve over every integer day of the exact interval and normalizes against its complete-year weight. Phase changes within-year timing, not unconstrained annual potential. Zero amplitude reduces exactly to fixed elapsed-day allocation.

When executable need is zero, provisioning causes no condition update. Zero demand is not interpreted as positive full-supply recovery.

## 3. Data evaluation

No new empirical data are introduced and no parameter is calibrated or retuned. `synthetic_validation_v1` remains synthetic; resource quantities and condition responses remain abstract mechanism-testing controls.

## 4. Conceptual-model evaluation

The repair removes four hidden bookkeeping effects:

1. repeated integer flooring no longer changes a fixed annual quantity merely because the year is partitioned differently;
2. M3 and M4 no longer reason about different current-period demand;
3. seasonal phase/resolution no longer acts as an undeclared annual-yield multiplier;
4. a zero-demand integer share no longer becomes an accidental positive physiological intervention.

Remaining limitations stay explicit:

- **#204:** condition recovery/loss, scarcity-mortality draws and M4 opportunities remain boundary-frequency dependent;
- **#200:** shared condition can still mix deprivation and M4 travel damage before broad scarcity attribution;
- **#208:** coincident M3/M2 mortality remains sequential competing-risk scheduling;
- **#201:** newborn-condition repair still needs downstream M3/M4 interaction acceptance.

## 5. Implementation verification

Verification added or strengthened includes:

- exact period boundaries for `P = 1, 3, 4, 5, 12, 365`;
- annual conservation for zero, one, non-divisible and larger integer quantities;
- explicit `100 -> 24,25,25,26` at four periods;
- M3 period-index demand == M4 boundary-day demand at every legitimate boundary;
- rejection of non-resource M4 boundary demand lookup;
- integrated seasonal annual potential invariant across tested phase/resolution combinations;
- phase changing within-year timing when amplitude is non-zero;
- zero amplitude exactly reproducing fixed allocation;
- zero-demand intervals preserving reduced condition;
- existing positive-demand resource/scarcity directionality;
- checkpoint/resume, cross-platform determinism and downstream regression suites.

### Acceptance-fixture correction discovered by v8

Six existing direct-M4 acceptance tests used `day = 1` as a shortcut even though day 1 is not a configured resource boundary. v8 correctly rejected them because M4's resource cue is now defined by the M3 interval settled on the same boundary. The tests were corrected to obtain the real first resource boundary from the authoritative scheduler helper. Their causal assertions were not weakened.

### Focused diff review correction

A full-file edit used for the semantics bump temporarily included an unrelated rewrite of resume-lineage validation. A focused PR diff review caught this before merge. `provenance.rs` was restored to the current-main implementation with only the intended `MODEL_SEMANTICS_ID` change from v7 to v8. No unrelated resume semantics are part of this repair.

The seasonal prefix table is deterministic derived state only; it avoids a 365-day per-cell inner loop and adds no fitted causal state.

## 6. Model-output verification and frozen-reference review

All reference changes were generated from one tested branch head:

`f4fd6aa1d35fa6b313199c958861b6195e833005`

with merge-ref build identity:

`pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`.

No model parameter or experiment definition was changed to recover old outputs.

### M7.6 resource variability

- workflow run: `32917412267`;
- artifact: `9588771705`;
- artifact SHA-256: `cb543afc3fa2abd3e945eaab8fb559cce0980294978bd4a693c03b5f9d92a072`;
- all **144/144** runs completed;
- source-definition SHA-256 remained `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`.

Both migration-enabled and migration-disabled arms change under v8 because M3 itself changed. The qualitative experiment structure remains: productivity-250 migration-disabled runs all become extinct; productivity magnitude strongly changes resource stress; migration remains strongly associated with persistence in the synthetic design; seasonal effects remain smaller/non-monotonic. The reference was deliberately regenerated from the observed v8 table.

### M8.6 terrain null model

- workflow run: `32917412247`;
- artifact: `9588696469`;
- artifact SHA-256: `7beb866c91f36be7c26b2195e2b07a5910e0cb563d2da4ea690522d908255f8b`;
- aggregate canonical SHA-256: `61f7965f875ba212778f6911261334c39cb9a340bd4717317441526fc80be811`.

All 32 runs completed and no arm was degenerate. The overall predeclared classification remains `fragile_spatial_structure`. Under v8 there are no robust primary metrics; `terminalLargestCellSharePermille` remains fragile and the other three primary metrics are not distinctive. Exact paired effects change materially, which is expected because corrected M3 timing changes resource exposure and M4 decisions throughout the trajectory.

### M9.7 controlled aggregation

- workflow run: `32917412358`;
- artifact: `9588720942`;
- artifact SHA-256: `1d0616edcd8c36c3c3c214ddd2efa0fa6d8f0133d14e65b128c3bb9544b86696`;
- aggregate canonical SHA-256: `30a9bc5e19c47f90290a3aab204ef18ab5b9754b0233d086f92e47aad678ba76`.

All **8/8** paired criteria still pass. The headline paired capability metrics are exactly unchanged: median focal-person-day difference `31` permille, maximum `36`; median intermittent peak visitor share `426` permille, minimum `387`. Duplicate replay and active checkpoint/resume are exact. State/provenance identities change with v8, but the predeclared capability distinction does not.

This combination is useful verification evidence: v8 changes downstream results where the resource clock is causal, while a controlled M9 capability comparison can remain numerically invariant when both arms share the corrected clock and the tested result is not resource-limited.

## 7. Model analysis

The repair makes future resource sensitivity easier to interpret because periodization no longer mixes several incompatible annual-quantity conventions. However, `periodsPerYear` is still not a pure numerical-resolution parameter because #204 remains. A period-resolution study performed now would still mix temporal opportunity frequency with numerical resolution.

Closing #180/#189/#199 therefore does not close M3 temporal-resolution sensitivity as a whole.

## 8. Corroboration

None. No archaeological, palaeoecological, ethnographic or physiological corroboration is attempted or claimed.

## Issue-level closure interpretation

After the final exact-head suite reproduces the checked-in v8 references, this change is intended to close:

- **#180** — annual quantities conserve under one scheduler-aligned contract and M4 uses M3's same current-period demand;
- **#189** — seasonal regeneration is integrated over actual periods and preserves unconstrained annual potential across phase/resolution;
- **#199** — zero-demand intervals are explicitly condition-neutral.

Still open:

- **#204** — resource-period frequency and physiological/mortality/M4 opportunity clocks;
- **#200** — resource versus travel contributions to condition/death attribution;
- **#208** — coincident M3/M2 mortality attribution;
- **#201** — downstream newborn-condition/resource/migration acceptance.

## Compatibility and scientific boundary

This repair intentionally changes resource stock, unmet need, condition, scarcity-death and M4 trajectories. `MODEL_SEMANTICS_ID` therefore changes from v7 to v8; v7 checkpoints must not be resumed as if they were v8 trajectories.

A green v8 suite verifies consistent execution of this contract. It does **not** establish that resource units are calories/biomass/palaeoproductivity, that the triangular seasonal function is ecologically realistic, that condition is a valid human physiological proxy, or that simulated archaeological outcomes are empirically correct.
