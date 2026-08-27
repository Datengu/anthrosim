# TRACE record: M4 proportional destination-choice repair (2026-08-27)

## Finding

Issue #195 identified a mismatch between the documented M4 stochastic destination-choice rule and executable weighting. Candidates are admitted only when their utility strictly exceeds origin utility plus `minimumUtilityImprovement`, so every eligible improvement is a positive integer. The executable rule nevertheless used `improvement + 1`, flattening relative preferences near the threshold.

## Repair

M4 now uses the eligible candidate's exact positive utility improvement as its stochastic weight. No pseudocount is added. Thus `[1, 2]` maps to `[1, 2]`, `[1, 10]` maps to `[1, 10]`, equal improvements remain equal, and common positive scaling preserves relative weights. Candidate eligibility, utility equations, uncertainty draws, candidate ordering, and the bounded integer draw algorithm are otherwise unchanged.

## Observability

`MigrationDecisionTrace` now preserves a stable candidate-order `eligibleCandidateWeights` table containing every eligible candidate's cell, utility and exact weight, in addition to `selectedWeight`, `totalMoveWeight`, and `choiceDraw`. For every retained move, a reviewer can reconstruct the complete categorical distribution as each candidate weight divided by the preserved total and can verify that the selected destination belongs to that table. The trace expansion remains bounded by the existing recorded-decision-trace cap and bounded M4 candidate radius.

## Compatibility

This changes authoritative M4 behavioural semantics and can alter migration destinations and downstream state for identical configuration and seed. `MODEL_SEMANTICS_ID` therefore advances from `anthrosim-model-semantics-v13` to `anthrosim-model-semantics-v14`. Historical artifacts remain bound to their original semantics identity.

## Acceptance evidence

The production weighting helper is exercised directly by unit coverage for exact `[1,2]`, `[1,10]`, equal-improvement, and common-scale invariance properties. Existing M4 migration, deterministic replay, checkpoint/resume, spatial, and protected benchmark gates remain required to detect unintended collateral effects.
