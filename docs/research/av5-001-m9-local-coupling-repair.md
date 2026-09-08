# AV5-001 / #606 M9 local coupling repair

## Scope

This note records the production remediation contract for Audit-v5 AV5-001 / issue #606.

Immutable discovery baseline remains:

- release `v0.3.5`;
- tag commit `e7667af52d48a1ffbae2bf7713a2388e65994b42`;
- software version `0.3.5`;
- model semantics `anthrosim-model-semantics-v33`.

The repair applies to the later living development line only.

## Root cause

The v31 M9 equal-cost destination policy removed canonical `HouseholdId` from the causal tie key but reused the minimum persistent person `stochastic_coupling_rank` among living household members. Those ranks are globally ordinal. Inserting an unrelated founder can therefore renumber an unchanged focal household even when the added household is unreachable through M9, causing the hashed equal-cost destination to change through a nonlocal bookkeeping pathway.

## Repair contract

Model semantics v34 leaves the shared persistent stochastic-coupling ranks unchanged for M2, M3, M4 and scarce-resource mechanisms. M9 alone derives a local ambiguity-coupling value using `m9/household-local-demographic-equivalence-v1`.

For the current M9 null ambiguity policy, the key hashes the sorted multiset of each living household member's:

- exact `birth_day`;
- reproductive sex.

It excludes canonical IDs, packed-record order, global coupling ordinals, residence, mutable condition and unrelated population records. Exact demographic duplicate households may share a key rather than receiving arbitrary globally unique labels.

The destination policy becomes `m9/equal-cost-destination-local-household-coupling-v3`. It retains the authoritative tie seed, origin representation and trigger index alongside the local household key.

## Compatibility assessment

This repair intentionally changes authoritative future M9 outcomes for some v33 states and therefore advances `MODEL_SEMANTICS_ID` to `anthrosim-model-semantics-v34`.

No checkpoint, event, travel-table or program schema changes are required:

- checkpoints and run artifacts already bind model-semantics identity;
- v33-to-v34 continuation therefore fails the existing scientific compatibility boundary;
- travel-table/program identity already binds the destination-policy identifier;
- tied departure events already record `destinationTieCouplingKey` for replay/observability.

The repair does **not** change traversability, edge costs, minimum-cost search, equal-minimum candidate construction, travel duration, M4 migration, mortality, resource allocation, or sequential RNG streams.

## Permanent regression

`crates/anthrosim-core/tests/m9_remote_founder_tie_locality.rs` preserves the discovery construction while exercising authoritative spatial execution:

- unchanged focal founder at `CellId(13)`;
- optional older founder appended at isolated/unreachable `CellId(1)`;
- focal tied destinations `CellId(8)` and `CellId(18)`;
- the focal persistent global rank is positively checked to renumber `1 -> 2` between arms;
- process/tie seeds `0..=1023` are swept;
- the emitted focal M9 coupling key and destination must remain identical in both arms.

The existing AV4-007 household-label regression remains required and was run before the production source commit.

## Deliberate boundary with AV5-004 / #629

AV5-001 is a population-composition locality defect. AV5-004/#629 is the remaining spatial-reflection-equivariance defect in equal-cost destination resolution. This repair does not attempt to close #629 and does not claim full spatial symmetry. The later #629 repair must preserve the local household-coupling contract introduced here.

## Acceptance and closure

Production acceptance requires the normal protected CI and applicable scientific/security gates on the exact PR head, including replay/checkpoint, cross-platform, M9, resource and spatial integration surfaces.

Because #606 is P1, production merge is not final issue closure. Independent post-merge adversarial re-verification must rerun the isolated-founder construction against exact merged `main`; the evidence-only branch/PR must remain unmerged, and #606 closes only after that evidence is recorded.
