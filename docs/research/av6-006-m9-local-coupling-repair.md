# AV6-006 / #711 — M9 reachable-component tie-coupling repair

Status: production repair candidate under Scientific Audit v6 remediation.

## Defect being repaired

Audit-v6 discovery PR #710 showed that the v35 M9 equal-cost destination policy could change the selected tied destination when one explicitly impassable, unreachable cell was appended outside an otherwise unchanged local travel problem. The physical local alternatives, route costs, household coupling key, trigger index and tie seed were unchanged; only irrelevant authoritative-world padding changed.

The root cause was the whole-world reflection-canonical frame used by the v35 spatial-equivalence policy. Although that frame repaired row-major reflection bias, it still allowed non-causal global extent/state outside the origin's reachable travel component to orient the local tie.

## Living repair contract

Model semantics v38 keeps the existing keyed ambiguity draw but changes the spatial equivalence frame used to order tied alternatives:

- compute authoritative M9.4 minimum-cost reachability first;
- for each reachable origin, isolate its reachable traversable connected component;
- normalize that component to its own bounding box;
- canonicalize only that local component over identity, horizontal, vertical and both-axis reflections;
- include component presence, focal-region membership, movement cost and only explicitly declared transformed spatial-host fields in the frame signature;
- use reflected local origin/destination coordinates plus minimum route-edge distance to form destination coupling classes;
- exclude absolute world coordinates, whole-world dimensions/digests, extent ordinals and global row-major `CellId` from scientific tie classification.

The destination policy is `m9/equal-cost-destination-local-household-spatial-equivalence-v5`. Because persisted destination coupling classes change meaning, the derived M9 travel-table schema advances from v5 to v6. Package version remains 0.3.6.

## Preserved causal sensitivity

The repair is local, not extent-invariant. An extent change remains scientifically causal when it changes traversability, reachability, the origin's connected component, route cost, equal-minimum candidate membership, or declared model-facing state inside the reachable component. Existing `spatial_boundary_dependence.rs` coverage proves this distinction: a hard wall makes a fixed M9 route unreachable in the tight extent, while a real route buffer makes it reachable and stable only after adequate enlargement.

## Pre-PR focused validation

Before full protected validation, the repaired implementation passed:

- the exact #710 impassable-padding locality construction across tie seeds 0..128;
- the Audit-v5 horizontal and vertical reflection-isomorphism sweeps across 256 seeds;
- exact-symmetry exchangeability and deterministic replay controls in that suite;
- Audit-v4 household-label invariance.

These focused checks do not replace the normal protected/scientific matrix. Production merge requires the complete exact-head matrix, and #711 must remain open after merge until an independent evidence-only derivative re-runs the preserved #710 adversary against the merged production lineage.