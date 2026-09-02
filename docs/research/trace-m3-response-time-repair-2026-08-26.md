# TRACE change record — M3/M4 response-time repair

**Date:** 2026-08-26  
**Primary finding:** #204  
**Model semantics:** `anthrosim-model-semantics-v9`  
**Scientific status:** model-structure/implementation repair; **not empirical validation**

## 1. Finding

The post-M9 scientific audit identified that `resources.periodsPerYear` was doing more than choosing M3 numerical/resource integration resolution.

Before this repair, each configured resource boundary also created:

- another condition recovery/loss opportunity;
- another condition-mediated scarcity-mortality draw; and
- another permanent M4 relocation opportunity.

Consequently, increasing the M3 partition could change annual physiological, mortality and behavioural opportunity rates even when the corresponding scientific coefficients were otherwise held fixed.

This was recorded as #204 and treated as a research-readiness defect rather than accepted as an undocumented scheduling convention.

## 2. Conceptual decision

The repair distinguishes three meanings that were previously conflated:

1. **M3 integration/settlement resolution** — controlled by `resources.periodsPerYear`;
2. **M3 condition and scarcity response timescale** — interpreted against a fixed four-quarter reference clock and converted over actual elapsed M3 intervals; and
3. **M4 permanent-migration opportunity frequency** — independently controlled by `migration.decisionPeriodsPerYear`.

The synthetic M4 default remains four opportunities per model year, preserving the prior baseline opportunity count while making it explicit and independent.

The detailed normative equations and scheduler ordering are in [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md).

## 3. Executable changes

### Configuration

- `ExperimentConfig` schema changes from 8 to 9.
- `ResourceConfig` schema changes from 2 to 3.
- `MigrationConfig` schema changes from 1 to 2.
- `MigrationConfig` gains `decisionPeriodsPerYear`, valid in `1..=365`, with synthetic default 4.

### M3 condition response

The historical `conditionRecoveryPerPeriod` and `maxConditionLossPerPeriod` fields are retained as serialized names but receive an explicit v9 reference-quarter interpretation.

For arbitrary M3 intervals, deterministic cumulative allocation converts the reference-quarter response into the amount attributable to elapsed time. A complete continuously applicable year therefore has the same response budget under tested partitions rather than multiplying with boundary count.

### M3 condition-mediated scarcity survival

The condition-derived probability is interpreted as a reference-quarter probability. Exact integer-rational conditional survival is used to convert it to the actual M3 interval.

At fixed condition, the complete-year survival probability is invariant to the tested M3 partitions and equals the composition of four reference-quarter survivals. The actual stochastic comparison uses the rational probability; the existing event field reports a deterministic parts-per-million ceiling for observability.

### M4 opportunity and demand semantics

M4 no longer requires an M3 resource boundary.

Its decision index/day is derived from `migration.decisionPeriodsPerYear`, and its resource-support cue allocates annual demand over that M4 decision interval using the same cumulative annual-quantity rule used elsewhere.

The runtime reconciles declared decision index, decision period count and actual boundary day. A mismatch fails closed as an internal invariant violation.

### Scheduler

Both authoritative simulation hosts now merge the independent fixed schedules:

- `Simulation`
- `SpatialLandscapeSimulation`

When M3 and M4 coincide, the established M3 → M9 → M4 subannual ordering is preserved. Either process may otherwise occur alone.

This spatial-host change is material: leaving the copied older scheduler in `SpatialLandscapeSimulation` would have created contradictory scientific semantics between synthetic and evidence-grounded runs.

## 4. Verification added

The implementation includes controlled/metamorphic checks for:

- fixed M4 four-per-year opportunity count while M3 uses 1, 4, 12 or 365 periods per year;
- independent M4 decision-period configuration;
- annual reference-quarter condition-response budget under 1, 4, 12 and 365 M3 periods;
- controlled full-supply/full-deficit condition trajectories not gaining additional annual response from finer M3 partitioning;
- fixed-condition scarcity survival equivalence under 1, 4, 12 and 365 M3 periods;
- exact recovery of the configured reference-quarter mortality probability at `P = 4`;
- M4 decision index/day alignment;
- existing M4 first-boundary founder-kin and stay-utility acceptance tests under the independent decision clock;
- identical merged-clock ordering in the ordinary and evidence-grounded spatial simulation hosts; and
- ordinary checkpoint/resume and workspace invariants through the existing regression suite.

The focused model review also checked the exact-rational bounded draw used for scarcity mortality. Its accepted random-value set contains an integer multiple of the probability denominator, so the modulo mapping is unbiased; it can reject slightly more values than a minimal rejection sampler but does not approximate the requested rational probability.

This is software/model-contract verification, not evidence that the response coefficients or clocks are empirically correct.

## 5. Provenance / compatibility decision

Because authoritative timing and checkpoint-continuation meaning change, the global model semantics identity changes:

`anthrosim-model-semantics-v8` → `anthrosim-model-semantics-v9`

v8 checkpoints must therefore fail closed when loaded by a v9 build rather than being continued under silently different timing semantics.

The configuration schema versions also change, and the legacy condition-response wire field names are retained only as names: their v9 scientific meaning is the reference-quarter meaning declared in the response-time contract.

## 6. Frozen-reference review and rebaseline

Frozen references were not rewritten merely because v9 changed trajectories. Each failing reference was rerun unchanged, inspected, and rebaselined only after its difference was attributable to the declared timing repair.

### M7.6 resource-variability reference

The unchanged 18-point × 8-seed synthetic experiment completed all **144/144 runs** under v9.

Generation evidence:

- workflow run: `32923009965`;
- branch head: `641cb1c1907f7679ccff20574924cb36dd5f8188`;
- PR merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9590629317`;
- artifact SHA-256: `799ce09b74cab05814593ed87e74da585cd6aee9982712385fc513b3346968d8`;
- derived sweep identity: `anthrosim-sweep-v2-f467645573da673d`;
- source definition SHA-256 remained `3206a40dba8a29f0e916460277ceea8b1a46363dc97215767cf923c54b67e47e`.

The machine-readable reference records the unchanged definition and the changed v9 derived outputs. It is a synthetic regression snapshot, not calibration or empirical evidence.

The regenerated reference was then independently accepted by the canonical M7.6 CI job on branch head `b887bc5857cbd243007fc98695f2c81d21700c93` (CI run `32924358049`).

### M8.6 evidence-grounded spatial benchmark

The four unchanged arms × eight declared seeds completed all **32/32 runs** under v9 with no degenerate arm.

Generation evidence:

- workflow run: `32923009999`;
- branch head: `641cb1c1907f7679ccff20574924cb36dd5f8188`;
- PR merge-ref build: `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`;
- artifact: `9590576288`;
- artifact SHA-256: `909a4d1032c2f3da5a4c7f5c719008a70b6c04e268e9cdb2893f1cef7c04525d`;
- aggregate canonical SHA-256: `fb90ad3a8870038d7f7e1ec42b34ffb3d1564be9255fc8f068b80673c35bb8c2`.

The overall predeclared classification remains `fragile_spatial_structure`, with **no robust primary metric**. Under v9:

- `migrationTotalDistanceCells`: fragile;
- `cellTimeOccupiedPermille`: not distinctive;
- `terminalPopulationHerfindahlPerMillion`: not distinctive;
- `terminalLargestCellSharePermille`: fragile.

The migration-distance effect exceeds the strong-arm magnitude threshold but fails the sign/cross-arm consistency criteria. It is therefore retained as fragile rather than being promoted to a robust result. This is a scientifically meaningful downstream change after separating the M4 decision clock; the model was not tuned to preserve an older classification.

The regenerated M8.6 reference was independently accepted on branch head `b887bc5857cbd243007fc98695f2c81d21700c93` (workflow run `32924358052`).

### Other downstream gates

On the same `1570c211...` validation head:

- cross-platform determinism: pass;
- M9.7 controlled aggregation benchmark: pass;
- spatial mechanism determinism: pass;
- spatial observability: pass;
- source provenance: pass;
- landscape preprocessing/loading determinism: pass;
- resumed Explorer compatibility: pass;
- run-bundle packing: pass;
- core format/Clippy/workspace tests, benchmarks, release build, performance/memory acceptance, 1000-run soak and M5/M6 integration: pass.

The later provenance-ledger edits to `resources-v0.1.md` and `migration-v0.1.md` correct stale v8 wording only; they do not alter executable or frozen-reference content. The final PR head must nevertheless pass the same automated gates before merge.

## 7. Interpretation boundary

The repair removes a numerical/model-structure confound. It does not make `resources.periodsPerYear` scientifically irrelevant.

Legitimate scheduling sensitivity can remain because changing M3 settlement times changes when stock, condition and survival state become available to later processes. Capacity clipping, evolving condition, M9 presence overlap, extinction timing and M4 observation of current state can therefore still produce different trajectories across resource resolutions.

The removed artifact is specifically the automatic multiplication of independent response/decision opportunities by the M3 boundary count.

Likewise, `migration.decisionPeriodsPerYear = 4` remains a synthetic model assumption. Making that decision rate explicit and independent does not validate four opportunities per year for any real population.

## 8. Deliberately unresolved linked findings

This change does **not** close the following findings:

- **#200:** shared `condition` can still allow M4 travel damage to contribute to a later death labelled `ResourceScarcity`; causal attribution remains unresolved.
- **#208:** coincident M3 and M2 mortality still require an explicit competing-risk/cause-attribution contract.
- **#201:** newborn condition semantics were repaired earlier, but remaining downstream M3/M4 acceptance scope is tracked separately and is not claimed by #204.

No empirical calibration or archaeological case-study conclusion is introduced by this repair.

## 9. Focused review conclusion

The #204 review was restricted to the timing repair and its directly affected contracts/references rather than reopening a broad repository audit.

Reviewed areas include:

- configuration/schema and default clock semantics;
- elapsed-time condition allocation;
- exact-rational scarcity survival conversion and RNG comparison;
- M4 decision-period demand and day/index reconciliation;
- ordinary and spatial merged schedulers, including one-time same-day M9 processing;
- v9 model-semantics compatibility;
- controlled acceptance tests;
- M7/M8 reference provenance and classification changes;
- ODD, ODD+D, scientific-model, M3 and M4 provenance documentation.

No blocking contradiction or unexplained trajectory/reference change remained after that focused review. GitHub reported no unresolved inline review threads or submitted-review blockers on the draft PR during this review.

## 10. Merge condition and scientific claim boundary

The executable/reference validation point at `1570c211...` satisfies the substantive #204 acceptance surface. Because this TRACE record and the final M3/M4 provenance wording themselves create later documentation-only commits, the **actual PR head immediately before merge must still pass the required CI/determinism/reference workflow matrix**.

If that final exact-head matrix is green and no new review blocker appears, #204 can be considered implemented by the merged PR.

That closure means only that AnthroSim now states and tests the annual/elapsed-time meaning of these response processes independently of arbitrary M3 boundary count. It is **not** a claim that the chosen reference-quarter physiology/mortality semantics, the four-per-year M4 default, or any resulting human/archaeological trajectory is empirically validated.