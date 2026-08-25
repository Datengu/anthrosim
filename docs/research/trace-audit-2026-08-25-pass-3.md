# TRACE scientific audit — adversarial pass 3 (2026-08-25)

**Status:** completed static scientific audit pass  
**Scientific status:** AnthroSim remains exploratory / not empirically research-ready  
**Scope:** current `main` population-level accounting, intervention semantics, symmetry/metamorphic behavior, scientific summaries and pattern-analysis readiness  
**Execution note:** static source/document audit only; no new numerical ensembles were executed locally.

## Purpose

This pass deliberately avoided re-auditing the time-resolution, physical spatial-scale and initialization failures emphasized in passes 1–2. It attacked:

- population-level conservation and emergence;
- authoritative-event/state reconciliation;
- causal ablation/intervention semantics;
- relabeling/symmetry failure modes;
- derived-summary semantics;
- false-positive pattern-matching risk.

## Result

The pass found **one new P1 scientific-output defect**:

- **#222 — undefined empty-set means are encoded as numeric zero.**

No additional new P1 causal-mechanism defect was found in the population-conservation, event-replay or basic intervention checks examined in this pass.

Because #222 is a new P1, this pass does **not** count as a clean P1-convergence pass.

## P1 finding: undefined means represented as zero (#222)

Two confirmed examples use an in-domain numeric zero to represent an empty denominator.

### Mean living condition after extinction

`Population::mean_living_condition_permille()` computes:

```rust
u16::try_from(total.checked_div(count).unwrap_or(0)).unwrap_or(PERMILLE_MAX)
```

When no people are alive, the mean condition of living people is undefined, but the run reports `0`.

The M7 sweep layer then averages that zero with ordinary surviving-run means. For example:

```text
surviving run: mean living condition 800
extinct run:   no living population
reported two-run mean: (800 + 0) / 2 = 400
```

That 400 is not a valid mean of living condition.

### Migration means with zero moves

`MigrationSystem::summary()` uses the same sentinel convention through:

```rust
fn mean_score(total: u64, count: u64) -> u16 {
    u16::try_from(total.checked_div(count).unwrap_or(0)).unwrap_or(PERMILLE_MAX)
}
```

When `moves_completed == 0`, mean origin/destination resource and water/security scores are undefined, but are emitted as zero — a value that can be misread as extremely poor resource/water conditions.

#222 was broadened to require an audit of all denominator-based outputs and nullable/typed semantics where no observations exist.

## Important non-findings

### Population accounting held up

`Population::validate(...)` explicitly reconciles:

```text
person records = initial population + births
living population = person records - deaths
```

and validates record limits, parent state, household/location consistency and occupancy indexing.

The run invariant layer independently reconciles event birth/death/migration totals with subsystem summaries.

No new population-conservation P1 was found.

### Resource conservation held up

The invariant layer checks:

```text
initial stock + regenerated stock = harvested stock + final stock
```

and resource internal accounting enforces the same conservation identity.

This does not resolve existing scientific-semantic issues in M3 (#180/#189/#199/#200/#204), but no additional stock-creation/loss accounting defect was identified here.

### Basic M4 ablation is isolated at the software level

When M4 is disabled, the migration process returns without consuming its decision logic, while demographic/resource RNG streams are separate named streams. The pass did not find a direct hidden execution of permanent migration under the disabled flag.

This does not solve the paired-seed common-random-number limitation in #214 or the wider question of how future study protocols define causal interventions.

### Event replay / residence observability accounting held up

M8 spatial observability accumulates half-open time intervals between authoritative birth/death/permanent-migration events and reconciles the replayed terminal population against authoritative person state.

No new same-day replay or person-day off-by-one P1 was identified in the inspected residence-based path.

### Existing symmetry findings remain the relevant ones

The pass did not identify a new independent ID/order bias beyond already tracked issues such as resource-allocation remainder bias, M4 kin/order bias, M9 destination/rounding tie behavior and state-dependent paired RNG alignment.

## TRACE interpretation

This pass primarily strengthens TRACE elements 5–7:

- **Implementation verification:** population/resource/event conservation has explicit executable checks and survived this audit lens.
- **Model-output verification:** summary statistics must distinguish a true zero from an undefined statistic.
- **Model analysis:** sweep aggregation can become scientifically misleading even when every underlying run is deterministic and internally valid.

## Convergence decision

**Not clean.** One new P1 was found.

The pattern is nevertheless informative:

- pass 1 found causal scale/timing/sensitivity infrastructure P1s;
- pass 2 found spatial boundary/environment/initialization P1s;
- pass 3's only new P1 came from derived scientific-output semantics while the core conservation machinery held.

That suggests the audit frontier is moving outward from local mechanism correctness toward experiment/inference semantics, but it is not yet sufficient evidence to stop auditing.

A subsequent independent pass should target areas not central to the first three passes, particularly:

- lifecycle/state transitions under extreme/extinction cases across M2–M4–M9;
- treatment/exposure verification for configured interventions;
- schema/provenance equivalence versus scientific equivalence;
- model-data comparison failure modes and predeclared multi-pattern acceptance logic.
