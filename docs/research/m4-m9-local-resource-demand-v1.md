# M4/M9 local resource-demand contract v1

**Status:** authoritative synthetic/null-model model semantics  
**Applies to:** M4 permanent migration with optional M9 temporary mobility  
**Scientific status:** internally consistent accounting/decision semantics; not empirically calibrated  
**Related contracts:** `migration-v0.1.md`, `m9-duration-aware-resource-semantics-v1.md`

## Purpose

M3, M4 and M9 must not use contradictory meanings of who creates local resource demand.

M9 already distinguishes persistent residence from temporary presence: visiting days consume resources at the visitor destination, while outbound and return transit remain provisioned from the persistent home cell. Before this contract, M4 still estimated local resource support from persistent-residence population only. A temporarily crowded cell could therefore consume visitor-attributed resources under M3 while M4 evaluated the same cell as if those visitors did not contribute to local demand.

This contract defines the M4 resource-support denominator when M9 is active.

## Declared M4 quantity

M4 `resource_score_permille` is a **current-boundary local provisioning-support proxy**:

```text
resource score = min(1000, current dynamic food stock * 1000
                           / (M4 interval need per person * boundary provisioning population))
```

For a hypothetical destination, the moving household's living members are added to that destination's boundary provisioning population before the score is calculated.

The annual food need is allocated to the M4 decision interval by the existing fixed annual-quantity rule. Issue #180 separately governs that temporal arithmetic. This contract governs which people are attributed to which cell in the denominator.

## Boundary provisioning population

At an M4 decision boundary, each living person's demand location follows the household's authoritative M9 presence state:

| M9 household presence | M4 demand location |
|---|---|
| M9 disabled / no temporary state | persistent residence |
| `AtResidence` | persistent residence |
| `Visiting` | temporary visitor destination |
| `OutboundTransit` | persistent residence |
| `ReturnTransit` | persistent residence |

Transit is not assigned to an arbitrary route cell. The home attribution deliberately matches M9.5's home-provisioning assumption.

This demand snapshot is separate from permanent residence occupancy. Temporary visitors do **not** become residents, do not change the persistent population location, and do not alter M4's occupied-cell accounting merely by visiting.

## Relation to M3 duration-aware accounting

M3 and M4 use the same **spatial attribution rule**, but they answer different temporal questions.

M3 settles the resource period using the authoritative M9 duration ledger. It partitions a household's completed period need according to the actual number of days spent at home/provisioned from home versus visiting. That is an elapsed-period accounting quantity.

M4 instead needs a bounded decision cue at one decision instant. It therefore uses the current authoritative presence state as a boundary snapshot and combines it with the M4 decision interval's per-person need. It does **not** predict how many days a current visit will continue, and it does not reconstruct the preceding duration ledger.

Accordingly, the M4 score must be described as a **current-boundary provisioning-support proxy**, not as an exact forecast of next-period M3 demand.

## Same-day ordering

The existing scheduler order remains authoritative. On a day shared by M3/M9/M4:

1. elapsed M3 resource settlement occurs for the preceding interval where due;
2. M9 transitions and trigger evaluations due on that day are processed;
3. M4 evaluates permanent migration;
4. annual M2 processing follows where applicable.

Therefore M4 observes the post-M9 presence state for that day.

Consequences include:

- a household that arrives at a visitor destination on the M4 boundary contributes to destination demand immediately;
- a household whose visit ends and enters return transit before M4 is again provisioned from home;
- a newly outbound household remains home-provisioned while in transit;
- households away from residence remain ineligible to make their own permanent-migration decision under the existing M4/M9 eligibility rule.

The same-day rule is deterministic and does not depend on household record ordering.

## Candidate and stay comparison

All M4 decisions at one boundary continue to use one shared pre-permanent-move snapshot.

For staying, the origin resource score uses the current boundary provisioning population at the residence cell, including the deciding household because it is at residence when eligible to decide.

For a candidate destination, M4 uses the current boundary provisioning population already attributed to that candidate cell and then adds the deciding household's living members as the hypothetical permanent-move demand.

Temporary visitors at that candidate are therefore part of the competition signal. They can lower the candidate's resource score and total utility; they cannot be silently omitted merely because their persistent residence lies elsewhere.

## Disabled-M9 compatibility

When M9 is disabled, every living person is attributed to persistent residence. The boundary provisioning population is therefore exactly equal to the legacy persistent-residence population used by M4.

This repair introduces no alternative disabled-mode resource semantics and no new stochastic draw.

## Permanent occupancy remains distinct

M4 retains a separate persistent-residence population snapshot for permanent relocation accounting. That snapshot continues to drive the migration-attributable occupied-cell delta before and after simultaneous permanent moves.

The visitor-aware boundary demand snapshot is used only for the M4 resource-support cue. This separation prevents a temporary visitor from being misclassified as a permanent resident or from changing settlement occupancy statistics.

## Scientific interpretation

This change closes an internal causal mismatch, but it does not validate the resource-support heuristic empirically.

A lower M4 resource score under temporary crowding means only:

> Under the declared synthetic provisioning rule, the current stock supports less M4 interval demand when more people are presently provisioned from that cell.

It does not establish that real households estimated carrying capacity from instantaneous head counts, that visitors shared resources equally with residents, or that temporary aggregation necessarily discouraged permanent settlement.

The snapshot formulation is itself a future sensitivity dimension. A later evidence-informed model could instead use expected visit duration, anticipated next-period demand, household-specific provisioning knowledge, storage, exchange, or other decision information, but such a change would require explicit new semantics and provenance.

## Verification

Regression coverage establishes that:

- disabled M9 produces the same M4 demand snapshot as persistent residence;
- identical persistent residences with different temporary visitor presence produce different destination demand only according to the declared M9 presence state;
- a zero-day outbound journey that arrives on the M4 boundary contributes visitor demand at the destination before M4 evaluation;
- outbound transit remains home-provisioned rather than being assigned to a route cell;
- with non-resource utility terms neutralized, adding visitors to a candidate increases its demand denominator and reduces both its resource score and its total relocation utility;
- permanent-residence occupancy remains unchanged by the temporary visitor attribution.

The full repository regression suite, deterministic platform checks, M8.6 terrain baseline and M9.7 controlled aggregation baseline remain required before this contract is accepted into `main`.

## Provenance boundary

This repair can change authoritative M4 choices when M9 is enabled, even if all persisted state and RNG positions are otherwise identical. It therefore advances `MODEL_SEMANTICS_ID` from `anthrosim-model-semantics-v11` to `anthrosim-model-semantics-v12`.

A checkpoint produced under v11 must not be represented as scientifically continuous execution under v12. Existing resume compatibility checks use the model-semantics identity to fail closed across that boundary.
