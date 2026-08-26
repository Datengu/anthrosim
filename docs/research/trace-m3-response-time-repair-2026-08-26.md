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
- existing M4 first-boundary founder-kin and stay-utility acceptance tests under the independent decision clock; and
- ordinary checkpoint/resume and workspace invariants through the existing regression suite.

On the pre-documentation code head `8e184e0295260471cfddce6d457e8897f06a3a47`, core CI confirmed:

- `cargo fmt --check`: pass;
- Clippy with warnings denied: pass; and
- full workspace tests: pass.

This is software/model-contract verification, not evidence that the response coefficients or clocks are empirically correct.

## 5. Provenance / compatibility decision

Because authoritative timing and checkpoint-continuation meaning change, the global model semantics identity changes:

`anthrosim-model-semantics-v8` → `anthrosim-model-semantics-v9`

v8 checkpoints must therefore fail closed when loaded by a v9 build rather than being continued under silently different timing semantics.

Frozen M7/M8/M9 references are not automatically rewritten. Each changed reference must be inspected and only rebaselined when its difference is attributable to the declared v9 repair rather than a new unexplained regression.

## 6. Interpretation boundary

The repair removes a numerical/model-structure confound. It does not make `resources.periodsPerYear` scientifically irrelevant.

Legitimate scheduling sensitivity can remain because changing M3 settlement times changes when stock, condition and survival state become available to later processes. Capacity clipping, evolving condition, M9 presence overlap, extinction timing and M4 observation of current state can therefore still produce different trajectories across resource resolutions.

The removed artifact is specifically the automatic multiplication of independent response/decision opportunities by the M3 boundary count.

## 7. Deliberately unresolved linked findings

This change does **not** close the following findings:

- **#200:** shared `condition` can still allow M4 travel damage to contribute to a later death labelled `ResourceScarcity`; causal attribution remains unresolved.
- **#208:** coincident M3 and M2 mortality still require an explicit competing-risk/cause-attribution contract.
- **#201:** newborn condition semantics were repaired earlier, but remaining downstream M3/M4 acceptance scope is tracked separately and is not claimed by #204.

No empirical calibration or archaeological case-study conclusion is introduced by this repair.

## 8. Remaining merge evidence

Before #204 can be marked completed, the final PR head must still demonstrate:

- complete required CI/determinism/reference workflow results;
- explicit review and justified v9 updates for any frozen references that changed;
- synchronized scientific-model, ODD and ODD+D documentation;
- focused PR diff/review with no unresolved blocker; and
- exact-head merge evidence.

This record should be updated if the final verification phase discovers a material change to the declared v9 contract.