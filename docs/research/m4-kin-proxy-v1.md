# M4 kin proxy — reciprocal first-degree residence null model

## Status

This document defines the normative M4 kin-residence proxy introduced for issue #188. It is a deliberately minimal **null-model** representation of first-degree kin proximity. It is not an empirical reconstruction of matrilocality, patrilocality, descent, marriage residence, household authority, inheritance, fosterage or wider social-network structure.

## Scientific problem repaired

Before model semantics v13, M4 collected parent locations one way from child records. Model-born children inherit the female parent's household. M4 then discarded parent locations belonging to the moving household, making the female-parent side structurally unavailable as an external kin anchor while the male-parent side could remain external. The supposedly neutral kin term therefore behaved predominantly as a paternal/father-location signal.

The old representation also retained only the first four unique external parent locations encountered during the packed person scan. Equivalent kin graphs could therefore produce different M4 inputs when person/birth insertion order changed.

Both behaviours were implementation artefacts rather than declared social assumptions.

## Normative v13 rule

For every represented living parent-child relationship:

1. identify the current persistent household of the child and of the living parent;
2. if both belong to the same household, create **no** spatial kin anchor because M4 moves that household as one unit;
3. if they belong to different households, add the parent's household residence as a kin anchor for the child's household **and** add the child's household residence as a kin anchor for the parent's household;
4. apply exactly the same rule to `female_parent` and `male_parent`;
5. retain every unique resulting kin cell; there is no fixed first-four truncation and encounter order has no scientific meaning.

The evaluated kin score remains deliberately narrow and binary:

- `kinScorePermille = 250` when the candidate cell is one of the household's reciprocal cross-household first-degree kin anchors;
- `kinScorePermille = 0` otherwise;
- multiple represented relatives at the same candidate cell do not stack.

This means the current proxy represents **presence of at least one living cross-household parent-child connection at a cell**, not kin count, relatedness coefficient, lineage membership or network centrality.

## Why co-resident relatives do not create a stay bonus

An earlier candidate repair simply retained same-household parent locations. Scientific regression review rejected that formulation: because those relatives move with the household, rewarding their pre-move cell creates an unintended household-inertia term rather than spatial kin proximity.

Under the normative v13 rule, co-resident first-degree kin remain socially represented by household membership but do not independently bias the household toward its current cell.

## Symmetry and invariance requirements

The implementation must preserve the following properties:

- swapping otherwise-equivalent female-parent and male-parent roles does not change represented kin anchors;
- every cross-household living parent-child relationship is reciprocal between the two households;
- permuting packed person records or birth insertion order without changing the represented kin graph does not change the kin-location set or kin utility;
- more than four qualifying locations remain represented;
- duplicate relationships pointing to one cell do not multiply the binary score;
- same-household parent-child relationships do not create a residence-specific preference.

These are scientific semantics, not merely test conveniences.

## Controlled utility interpretation

When resource, water, travel, uncertainty and relocation-risk contributions are neutralized, evaluating the child household at the parent's cell and the parent household at the child's cell must produce the same declared `250` kin advantage.

The scalar `250` remains a synthetic/null-model weight. It has not been calibrated to archaeological or ethnographic evidence and must not be interpreted as a measured historical strength of kin preference.

## Provenance and downstream consequences

This repair changes authoritative M4 residence utility and therefore advances `MODEL_SEMANTICS_ID` from `anthrosim-model-semantics-v12` to `anthrosim-model-semantics-v13`.

The change can legitimately alter later resource access, condition, demographic events and spatial structure through changed permanent-migration decisions. Frozen downstream scientific references must therefore be causally reviewed rather than assumed to remain numerically identical.

The reviewed M8.6 rebaseline localizes the first divergence in every one of its 32 runs to the newly represented reciprocal kin term. The reviewed M7.6 rebaseline provides an independent factorial control: all 72 migration-enabled runs change while all 72 migration-disabled runs remain scientifically identical to the immediate v12 control. Together these checks support attribution of downstream changes to M4 rather than an unrelated resource or demographic modification.

## Interpretation boundary

The v13 proxy fixes symmetry and record-order defects in the null model. It does **not** establish that real households treated maternal and paternal kin identically, that first-degree kin were the dominant social ties, that households migrated as indivisible units, or that `250` is historically realistic.

Future case-study work should treat alternative kinship/residence systems and kin weights as explicit evidence-backed hypotheses or sensitivity dimensions rather than silently inferring them from this null rule.
