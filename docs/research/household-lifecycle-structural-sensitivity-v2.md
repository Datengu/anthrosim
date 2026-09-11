# Household lifecycle structural sensitivity v2

## Status

This document defines the current synthetic household-lifecycle structural-sensitivity treatment introduced by audit-v2 issue #324. It supersedes `deterministic_size_fission_v1`, whose stable-PersonId slicing made packed record order an unintended cohort/generation rule.

This treatment is deliberately a **structural stress test**, not an ethnographic or archaeological model of household formation.

## Treatment identity

The active treatment is `deterministic_dependency_fission_v2`, represented by household-lifecycle schema version 2.

The treatment declares two scientific parameters:

- `maxLivingMembers`: target maximum living household size;
- `minimumIndependentAgeYears`: explicit age threshold used to decide which people can anchor an autonomous daughter household.

The current general-demography sensitivity arm uses a threshold of 18 years. This is a declared modelling assumption for the stress test, not a claim that age 18 is a cross-cultural or prehistoric household-independence threshold.

## Fission rule

At each eligible annual household-lifecycle boundary:

1. only households whose members are physically at their persistent residence are eligible;
2. if living membership does not exceed `maxLivingMembers`, nothing happens;
3. the minimum number of groups needed to meet the target size is calculated;
4. living members at or above `minimumIndependentAgeYears` are identified;
5. the number of daughter groups is limited by the number of independent-age members, so every created group can be seeded by at least one such member;
6. if fewer than two independent-age members exist, fission is deferred instead of manufacturing an autonomous child-only unit;
7. independent-age members are assigned deterministically by age, reproductive sex and an ID-independent relationship-role refinement over the living source-household parent/child graph plus living direct-parent context outside that household; a living external parent is distinguished only by persistent residence cell, matching the downstream M4 first-degree kin-location state, and PersonId is used only as a final tie-break within the same stabilized scientific context;
8. dependents are ordered by the same age/sex/relationship-role rule and then assigned deterministically, preferring groups containing their living parent(s) when those parents remain in the source household; remaining target capacity is used as the secondary allocation criterion;
9. the original household retains group 0 and each remaining group becomes a new household at the same persistent residence;
10. person identity, parent links and condition remain unchanged, and M3/M4/M9 operate on the resulting households normally.

Because parent relationships can span more than one independent-age group, parent-child co-residence is preferred for dependents but is not guaranteed for every parent link. Cross-household kin therefore remain possible and are visible to the existing kin/movement mechanisms.

## Target size is subordinate to dependency safety

`maxLivingMembers` is a structural target, not an absolute physical ceiling. If a household is oversized but has too few independent-age members to anchor the number of groups required by the target, the treatment creates fewer groups or defers fission. The model does not invent independent adults merely to satisfy a size cap.

This means a household can temporarily remain above the target. That is intentional and scientifically preferable to creating a child-only household solely because the target was exceeded.

## PersonId and relabelling contract

PersonId is not a household-composition variable. The treatment may use it only as a final deterministic tie-break among records that are otherwise equivalent under the declared age/sex/relationship rule.

Relabelling scientifically equivalent people while preserving their age, sex and relationship structure must preserve the unlabelled scientific composition of the resulting households. In particular, packed-storage order must not separate founders from model-born cohorts as it did under v1.

From model semantics v25, the relationship comparison is executable rather than documentary: living members of the source household are partitioned into age/sex role classes, then those classes are iteratively refined by female-parent state, male-parent state and the multiset of living in-household child roles until stable. Only records still in the same stabilized role class may fall through to PersonId. This keeps deterministic replay without allowing canonical record labels to choose between relationship-distinct anchors or dependents.

From model semantics v40, female- and male-parent refinement also preserves one explicit cross-household context for a represented living parent: that parent's persistent residence cell. This is the same causal context M4 exposes as a reciprocal first-degree kin-location anchor after household topology changes. The refinement does **not** use the external parent's PersonId, HouseholdId, packed-record position or global stochastic-coupling rank, and two otherwise equivalent external parents at the same persistent residence remain in the same relationship context. This is a relabelling-invariance rule, not an ethnographic preference for one direction or relative.

## Downstream integration

After fission, daughter households are first-class households for all household-level mechanisms:

- M3 resource sharing is household-local;
- M4 permanent migration evaluates each daughter household independently and records its HouseholdId in migration decision traces;
- M9 temporary mobility can select daughter households independently;
- checkpoint/resume must preserve the exact post-fission topology and subsequent decisions.

Issue #324 therefore changes authoritative causal state trajectories and advances `MODEL_SEMANTICS_ID` from `anthrosim-model-semantics-v20` to `anthrosim-model-semantics-v21`.

The old v1 reference remains historical evidence for the original #207 experiment. It must not be presented as the current treatment result without an explicit v2 rerun/rebaseline.

## Interpretation limits

This rule does **not** model marriage, inheritance, residence norms, fosterage, slavery, household headship, culturally defined adulthood, property division, or household economics. The independent-age threshold and parent-aware allocation are transparent safeguards against a demonstrated record-order artefact, not empirical validation of a particular social system.

Model semantics v40 changes authoritative daughter-household membership for the AV6-003/#699 external-kin equivalence case, so v39 checkpoints are not continuation-compatible with v40. Package version remains `0.3.6`, and immutable v0.3.6/v35 Audit-v6 discovery evidence is unchanged.
