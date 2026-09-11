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

## Guarded v38 scientific-reference reconciliation

The first complete PR execution correctly exposed two reference consequences of the v38 semantics change. They were reconciled only after fresh execution and bounded-delta checks:

- fresh 780-run demographic confirmation run `34555842374` reproduced the normalized v37 demographic scientific result exactly under v38 after excluding execution/model-semantics identity and pre-existing guard-only precision metadata; the recommendation, all six arm summaries, paired effects and long-run classifications were unchanged;
- fresh M9.7 run `34555842586` preserved benchmark/definition identity, `capability_distinguished`, aggregate endpoints, both arm configuration identities, every frozen legacy per-seed metric and the independent M9.6 travel-burden reference; only authoritative terminal state digests changed under the new persisted M9 tie-policy/schema identity;
- fail-closed refresh run `34556757782` then required those bounded contracts to pass before updating the checked references and removed its temporary validation helpers after success.

## Guarded M7.6 applicability re-verification

The complete protected matrix on production candidate `138d489bcb63d8080858ac203e0d8897fb874f0b` reran the canonical M7.6 resource-variability factorial in central CI run `34557084821`. Its derived artifact `10183267124` (SHA-256 `2d9ae421993cf116fe6141e503f8a3a285b3d54f9d574ef683e43e3524a10a2e`; PR merge-ref build `f775c66651d3152210fc09582fae5752eb0767ea`) contained all 18 parameter points and all 144/144 declared simulations completed and scientifically eligible under v38.

Because AV6-006 changes only M9 temporary-destination tie coupling and M7.6 contains no temporary-mobility mechanism, a fail-closed provenance guard required every frozen v37 `pointResults` value to reproduce exactly before permitting any reference edit. Guard run `34558147431` passed that exact-equality test: 18/18 point summaries reproduced without numerical change, with the definition identity, experimental coordinates, completion/extinction classifications, eligibility rules and accounting guards unchanged. The M7.6 checked reference therefore advances only its model-semantics binding from v37 to v38 and records this provenance; no scientific endpoint was rebaselined.

These checks are evidence for the production candidate, not a substitute for exact-head protected validation. Production merge requires the complete final-head matrix, and #711 must remain open after merge until an independent evidence-only derivative re-runs the preserved #710 adversary against the merged production lineage.