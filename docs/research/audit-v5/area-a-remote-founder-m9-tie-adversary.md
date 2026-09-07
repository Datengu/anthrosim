# Audit v5 Area A — isolated-founder / M9 tie-coupling adversary

Target: immutable `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / `anthrosim-model-semantics-v33`.

This is fresh Audit-v5 discovery evidence. It is not a replay of an Audit-v4 finding.

## Adversarial hypothesis

The v26–v33 repair line introduced persistent person stochastic-coupling ranks and later reused the minimum living-person rank as the M9 equal-cost destination tie key. Initial ranks are global ordinals over the complete represented founder population, while M9 hashes the numeric rank value.

A possible composition failure is therefore that adding a scientifically non-interacting founder elsewhere in the world renumbers an otherwise unchanged focal household and changes its equal-cost M9 destination. That would make a local movement decision depend on unrelated global population composition rather than the focal household, its origin, the travel field, trigger and declared tie seed.

## Controlled attack

The Rust adversary constructs two day-zero populations on the same 5x5 world:

- baseline: one unchanged focal founder in household 1 at cell 13;
- augmented: the exact same focal founder plus one older founder in appended household 2 at cell 1.

Cell 1 is enclosed by movement costs above the declared M9 traversability ceiling, so the added household is unreachable and cannot participate in the focal journey. The focal founder keeps the same PersonId, HouseholdId, age, sex, residence, condition and genealogy.

The focal origin at cell 13 has two exact equal-cost destinations, cells 8 and 18. The test verifies the added unreachable founder changes the focal serialized stochastic-coupling rank from 1 to 2, then sweeps 1,024 destination-tie seeds through the authoritative `TemporaryTravelTable::resolution_for_coupling_key` path.

## Interpretation rule

The scientific null is locality: adding an unreachable, otherwise non-interacting founder must not alter the unchanged focal household's equal-cost destination solely through a global coupling-rank renumbering.

If any seed diverges, the evidence demonstrates a new composition dependence between global founder canonicalization and M9 tie resolution. Before classifying it as an Audit-v5 finding, search prior issues/PRs for overlap and distinguish this from Audit-v4 AV4-007, which concerned arbitrary `HouseholdId` labels rather than global population-composition renumbering.

If all seeds agree, this particular composition hypothesis is falsified but Area A remains incomplete until broader scheduler/order/simultaneity surfaces are examined.
