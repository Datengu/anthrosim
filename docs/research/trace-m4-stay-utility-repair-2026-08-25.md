# TRACE repair record — M4 stay utility semantics

**Date:** 2026-08-25  
**Issue:** #186  
**Model-semantics transition:** `anthrosim-model-semantics-v6` → `anthrosim-model-semantics-v7`  
**Scientific status:** implementation/conceptual repair; not empirical validation

## Problem

M4's origin comparator was intended to represent the action **remain at the current residence**, but it was evaluated through the same cell-utility function as relocation candidates. Even with distance and uncertainty set to zero, that path still subtracted:

- destination-terrain travel penalty from the origin cell's `movementCost`; and
- the base relocation-risk penalty.

The first term could make rough origins artificially unattractive even when no travel occurred. The second normally cancelled between stay and relocation, making a documented relocation-risk parameter behaviorally inert through much of its range and producing pathological saturation behaviour at high values.

Because the stay action is the counterfactual baseline for every M4 destination comparison, this was an authoritative scientific-behaviour defect rather than a trace-formatting issue.

## Conceptual repair

M4 now separates:

1. **residence-state terms** — resource support, water/security and the narrow direct-parent kin proxy; and
2. **relocation-only action costs** — travel/terrain penalty, candidate uncertainty and base/distance relocation risk.

The stay action evaluates only residence-state terms. Its travel, uncertainty and relocation-risk fields are explicitly zero.

Candidate relocation evaluates the same residence-state terms for the destination and then subtracts relocation-only action costs.

The existing terrain interpretation is retained for now: candidate travel penalty includes Manhattan distance plus the **destination** cell's movement-cost excess. This remains a synthetic proxy and is separate from the unresolved physical-distance/grid questions elsewhere in the audit backlog.

## Verification evidence added

A black-box two-cell acceptance fixture executes a real M4 migration boundary using a declared founder population and M8-style model-field overlays. It verifies:

- **base-risk directionality:** with otherwise equal states, a move that occurs at zero base relocation risk is blocked when the base risk is raised sufficiently;
- **stay action separation:** recorded origin utility has zero travel, uncertainty and relocation-risk penalties;
- **rough-origin null case:** changing only origin `movementCost` from the baseline to a strong M8-style rough value does not change migration summary/outcome through a stay travel penalty;
- **candidate terrain monotonicity:** increasing only candidate movement cost cannot make relocation more attractive and can remove move eligibility;
- **zero effective action-cost comparator:** with relocation risk zero and travel-cost weight zero, the move comparison reduces to the declared residence terms even though the raw candidate travel diagnostic remains observable.

The older M8 movement-cost directional fixture was also replaced because it asserted that a moving destination had a lower travel penalty than the stay origin. That assertion depended on the defect being repaired. The replacement stays on the currently supported synthetic spatial-initialization path, gives relocation an independent water-access advantage, and inspects a move into transformed non-baseline terrain. It requires the stay travel/uncertainty/risk fields to remain zero, the candidate travel penalty to equal the declared distance-plus-destination-terrain formula, and that penalty to reduce candidate utility through the configured travel weight.

During development, attempting to use the new `declared_founder_state_v1` path through `SpatialLandscapeSimulation` exposed a separate pre-existing integration gap: the spatial host still calls `Population::initialize(...)` rather than materializing the embedded founder definition. That finding is recorded on #213 and is not repaired as part of #186.

Existing deterministic M4, checkpoint, M8/M9 and cross-platform suites remain required. A green focused test alone is not sufficient to close the issue.

## Provenance consequence

The repair can change authoritative migration decisions, traces, downstream population/resource trajectories and spatial outcomes. Therefore `MODEL_SEMANTICS_ID` is incremented from v6 to v7.

Old v6 checkpoints are scientifically incompatible with continued execution under v7 rather than being silently resumed under the repaired comparator.

Canonical/reference outputs that change must be inspected and explained causally. They must not be silently rebaselined merely to restore green CI.

## M8 interpretation consequence

The strongest expected correction is for terrain experiments in which a household starts on a high-`movementCost` cell. Under v6, rough origin terrain could penalize the zero-distance stay action and thereby manufacture an incentive to leave. Under v7, terrain enters this part of M4 only as a relocation-candidate travel cost.

If M8.6 outputs change, the expected causal direction is therefore a reduction/removal of migration attributable specifically to the former rough-origin stay penalty. Any other material change requires separate investigation.

## Observed M8.6 consequence

The predeclared M8.6 terrain null-model benchmark was rerun under v7 before its preserved scientific reference was changed. GitHub Actions run `32905887259` (artifact `9584814771`) executed all four declared arms and eight declared seeds per arm without degenerate runs. Its aggregate canonical SHA-256 was `c4a101de4f2ad0c5044313cb86b0f8e57d0a9a203c2b937351bbce482ac27d3c`.

The overall benchmark class remains **`fragile_spatial_structure`**, so the repair does not create a stronger terrain result. The material classification change is narrower and scientifically important:

- `migrationTotalDistanceCells` changes from **robust** under the v6 reference to **not distinctive** under v7;
- in the strong terrain arm its median absolute relative effect is approximately **8.08%**, below the predeclared 10% robustness threshold;
- strong-arm paired signs are **5 positive / 2 negative / 1 zero**, below the required six same-sign non-zero pairs;
- `terminalLargestCellSharePermille` remains **fragile** rather than robust;
- no treatment arm becomes degenerate.

This observed change matches the defect's expected causal direction. Removing a rough-origin penalty from the stay counterfactual removes one mechanism that could manufacture extra terrain-associated movement. The corrected benchmark therefore weakens the previous terrain-distance claim rather than strengthening it: under the preserved M8.6 null model, the evidence-grounded terrain transform still produces fragile spatial structure, but **total migration distance is no longer a robust distinguishing metric**.

The M8.6 reference is updated only after this causal inspection. The new reference records the v7 model-semantics identity, the benchmark execution/artifact provenance, all predeclared paired-effect summaries, and every terminal state digest. Future changes must still pass the same scientific regression verifier; this repair does not relax that gate.

## Observed M7.6 consequence

The canonical synthetic resource-variability sweep was also rerun under v7 before its reference was changed. Umbrella CI run `32907506629`, merge-test revision `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`, produced artifact `9585478308` (ZIP SHA-256 `5e9765c169fea2a79c57e6bf588ea1268df30b649fa83c1432035c9ac86bdd31`). The source-definition SHA-256 remained exactly `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`; all 18 points × 8 seeds completed, with no failed, incomplete or person-record-limit runs.

M7.6 contains a particularly useful internal control because each productivity/seasonality setting is run once with migration enabled and once with migration disabled. The v6 → v7 comparison is unambiguous:

- **all nine migration-disabled points reproduce the v6 reference exactly in every compared point metric**;
- **all nine migration-enabled points change** under the repaired comparator;
- total migration distance decreases in all nine migration-enabled points, by roughly **4% to 12%** relative to the v6 reference;
- pooled mean distance per completed move decreases in all nine migration-enabled points, by roughly **5% to 8%**;
- migration-move counts do not move in one universal direction, so the most consistent correction is to the distance/attractiveness structure of selected moves rather than simply suppressing every migration decision;
- downstream population, occupancy and resource-stress quantities also change only when migration is enabled, as expected because altered relocation histories feed back into later resource exposure.

This migration-on/migration-off split is strong causal evidence that the M7.6 reference difference comes from #186 rather than an unrelated M2/M3 or orchestration change. It also reinforces the M8.6 finding: the former stay comparator was overstating the relative attractiveness of some relocations. The corrected v7 model still shows a large synthetic persistence difference between migration-enabled and migration-disabled arms, but the exact migration-distance outputs and downstream resource trajectories must be interpreted under the repaired comparator.

The checked-in M7.6 reference is therefore regenerated for v7 only after this controlled comparison. The versioned source experiment definition is unchanged; the reference remains a derived synthetic regression snapshot, not empirical validation or a calibration target.

## Scientific boundary

This repair establishes consistency between the implemented comparator and the documented meaning of travel/relocation costs. It does **not** establish that:

- the current utility weights are empirically correct;
- Manhattan distance is a valid physical travel model for a real study;
- destination-cell movement cost alone is a sufficient traversal-cost model;
- relocation risk is calibrated to any archaeological or ethnographic population;
- M4 is empirically validated.

Those questions remain subject to evidence grounding, structural sensitivity and other open P1 findings.

## TRACE classification

- **Conceptual model evaluation:** improved — the action/state distinction is explicit and internally coherent for this mechanism.
- **Implementation verification:** improved — direct counterfactual/metamorphic tests now cover the repaired semantics.
- **Model output verification:** unchanged — no new empirical validation evidence is created.
- **Model analysis:** improved in a narrow but useful respect — the M8.6 and controlled M7.6 reruns show that previously recorded migration-distance behaviour changes in the predicted direction when the stay comparator is repaired. Broader sensitivity/uncertainty analysis remains incomplete.
- **Output corroboration:** unchanged / not established.

Closing #186 therefore removes one blocking scientific-behaviour defect and corrects overstated synthetic migration-distance responses; it does not upgrade AnthroSim to empirically research-ready status.
