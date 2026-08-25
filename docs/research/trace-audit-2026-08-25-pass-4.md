# TRACE adversarial scientific audit — pass 4

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Execution note:** no new local numerical ensemble or full local test run was executed; the available runtime could not clone GitHub because outbound container DNS/network access was unavailable. Findings were checked against live repository source and documentation through the GitHub connection.  
**Overall result:** **NOT A CLEAN P1-CONVERGENCE PASS**

## Purpose

This fourth independent pass deliberately moved away from the earlier time-step, spatial-resolution, initialization, conservation and broad TRACE lenses. It targeted lifecycle extremes and scientific-output fidelity, especially places where preserved reports might describe a different quantity from the one the authoritative model actually executed.

The main questions were:

- do temporary-mobility summaries distinguish planned from actually realized travel?
- do spatial reports expose authoritative dynamic resource state rather than a similarly named baseline input?
- can detailed migration events distinguish nominal from realized bounded condition loss?
- do sweep aggregates preserve enough exposure information to compare cumulative outcomes across early-extinct and full-duration runs?
- do M9/M4 and M9/M3 cross-mechanism edge cases preserve their declared semantics?

## New findings

### P1 — #224: report actual M3 initial per-cell resource stock in spatial observability

`SpatialObservabilityReport` currently labels `World::Cell.food_stock` as per-cell `initialFoodStock`, while terminal stock is read from the authoritative `ResourceSystem` checkpoint.

That pair is not generally the same state variable at two times. `ResourceSystem::initialize()` applies the configured productivity scale and then caps stock by the configured capacity:

```text
scaled_initial = scale(world.food_stock, productivityScale)
capacity = baseProductivity × annualRegeneration × productivityScale × capacityYears
actual_initial_stock = min(scaled_initial, capacity)
```

Legitimate resource configurations therefore make actual M3 day-zero stock differ from the value reported by M8.5 as initial stock.

This can create false apparent depletion/recovery in spatial analysis. The fix is to report the real initialized `ResourceSystem` stock as `initialFoodStock`, retaining raw world stock only under a distinct baseline/input name if useful.

**Issue:** #224.

### P2 — #223: distinguish planned from realized M9 travel burden

Temporary observability adds the full planned outbound and return durations, round-trip route cost and derived route distance at departure. Active-at-end or household-extinction-terminated journeys therefore contribute future/unexecuted return legs to aggregate `travel burden` totals.

The detailed journey status and observed person-days preserve enough information to detect this downstream, so this is an output-semantics problem rather than a lifecycle execution failure.

The report should expose planned route burden separately from realized/observed travel within the report interval.

**Issue:** #223.

### P2 — #225: distinguish nominal from realized M4 travel-condition loss per migration

Every M4 migration event/trace stores a nominal per-person condition decrement derived from distance. Actual application is saturating: a mover with condition below the nominal cost loses only the condition they actually have.

The aggregate `travelConditionCostTotal` is correctly based on realized loss, but the event stream does not preserve realized loss by move. Because historical condition is not otherwise continuously available, multiplying event `peopleMoved` by nominal cost can overstate the realized physiological/condition effect.

**Issue:** #225.

### P2 — #226: preserve exposure duration and normalized rates for cumulative sweep outcomes

Scientifically valid completed runs can stop early through population extinction. Sweep analysis nevertheless averages cumulative births, deaths, scarcity deaths, unmet need, migration moves and migration distance, while the run-level derived table does not expose the manifest's exact `simulatedDays` value.

A smaller cumulative total can therefore mean lower process intensity, shorter survival/exposure, smaller population exposure, or all three. The existing totals remain useful, but comparative research needs exact exposure denominators and clearly named rates/intensities where scientifically defensible.

**Issue:** #226.

## Important non-findings

### M9 preventing M4 evaluation while away is deliberate

The M9 scientific contract explicitly states that a household with an active temporary journey is not eligible for M4 permanent-migration evaluation, and same-day temporary departure takes precedence. The implementation's skip is therefore declared model semantics, not an accidental scheduler defect.

It may still require structural sensitivity in a real study, but it was not re-filed as an implementation bug.

### Extinction during an active temporary journey is reconstructable

Core temporary state silently clears an active journey after the last living member dies, but downstream temporary observability explicitly reconstructs that history as `TerminatedNoLivingMembers` rather than fabricating a completion event.

The preserved event history is therefore sufficient for the current observability contract. The suspected missing-termination bug was rejected.

### M9 resource splitting does not inflate `householdPeriodsWithUnmetNeed`

M9 may split one household's period demand into home and visitor claims, but M3 recombines claim harvest into a household total before incrementing `householdPeriodsWithUnmetNeed`. A household contributes at most once per resource period to that counter. The suspected claim-counting bug was rejected.

### Incomplete migration digest is already tracked

The pass rediscovered that several serialized M4 counters/trace fields are omitted from the current component/state digest. Existing P1 #168 already explicitly covers this continuation-integrity problem, so no duplicate issue was filed.

## Pass result and convergence

This pass found:

- **1 new P1:** #224;
- **3 new P2s:** #223, #225, #226.

Because #224 is a new P1 scientific-output defect, this is **not** a clean P1-convergence pass.

The audit pattern continues to move outward from core mechanism equations toward scientific measurement and interpretation surfaces, but that trend is not sufficient to declare convergence. A research simulator can execute the right state transition and still mislead if a derived artifact gives a scientifically different quantity the same label.

## Recommended next independent lens

The next pass should avoid repeating output-name checks and instead target **causal intervention fidelity and population opportunity structure**, including:

- whether changing one structural assumption alters opportunity sets in hidden ways;
- fertility/parentage opportunity denominators and sex/household composition effects;
- whether treatment arms nominally differing in one mechanism also differ in the number/timing of decision opportunities;
- null/intervention tests where a configured mechanism is scientifically non-binding;
- emergent population patterns that can be generated by bookkeeping structure rather than the intended causal hypothesis.

A clean pass under a genuinely different lens is now more informative than continuing indefinitely to enumerate presentation conveniences.