# M4 reciprocal first-degree kin-location proxy v1

**Status:** normative post-M9 scientific-hardening contract  
**Model semantics:** `anthrosim-model-semantics-v13`  
**Scientific status:** synthetic / unvalidated

## Purpose

M4 has a deliberately narrow genealogical residence term so represented close-kin relationships can affect permanent-migration utility without introducing a general social-network or kinship model. This contract defines that term so reproductive-sex role, household inheritance and packed person-record order cannot become accidental one-way social rules.

## Authoritative rule

At each M4 decision boundary, M4 considers each represented living parent-child relation for which both people are alive.

- If parent and child belong to the **same household**, the relation creates no spatial kin anchor. M4 moves that household as one unit, so the relation would remain co-resident after either staying or relocating and does not distinguish residence alternatives.
- If parent and child belong to **different households**, the relation is represented reciprocally:
  - the child's household retains the parent's current persistent-residence cell;
  - the parent's household retains the child's current persistent-residence cell.
- `female_parent` and `male_parent` links use exactly the same rule.
- Every unique resulting cell is retained. There is no first-N truncation or encounter-order priority.

For any residence cell `c`:

```text
kinScore(c) = 250  if c is in the household's reciprocal cross-household first-degree-kin set
              0    otherwise
```

Multiple represented first-degree relatives at one cell do not stack. The configured `migration.kinWeight` multiplies this score in the ordinary M4 residence utility.

## Why the relation is reciprocal

Model-born children join the female parent's persistent household. Under the pre-v13 parent-only external-anchor rule, that storage/lifecycle convention made the female parent structurally unable to provide an external anchor while a male parent from another household could do so. An apparently gender-neutral direct-parent term therefore behaved predominantly as attraction toward external father locations.

Simply counting co-resident parents at their pre-move cell would remove the explicit household filter but create a different artefact: it would reward staying near a parent who is actually part of the moving household and would relocate with it. The v13 rule therefore treats the **cross-household genealogical edge**, rather than only the parent endpoint, as the spatial relation. A living cross-household parent-child tie connects both households to one another's persistent residence with equal strength, regardless of whether the represented parent is female or male.

This is a null-model symmetry rule. It does not assert matrilocality, patrilocality, descent ideology, household authority or a measured human preference.

## Same-household relatives

A same-household parent-child relation is genealogically real but spatially non-discriminating for M4 because all living household members relocate together. It therefore contributes no location-specific kin utility. AnthroSim does not currently model internal household cohesion, bargaining or fission as a separate migration term.

## Record-order invariance

The kin-location set contains every unique location produced by the reciprocal cross-household first-degree relations. It has no first-four or first-N selection rule. Reordering otherwise-equivalent person/birth records cannot cause a later kin location to disappear merely because another relationship happened to be encountered first.

The transient vector order used while collecting cells has no scientific meaning. M4 asks only whether an evaluated cell is present in the complete unique set.

## Scope and non-claims

This proxy is intentionally minimal. It includes only represented living **parent-child** relations and their reciprocal household-level spatial link. It does **not** represent:

- siblings, cousins, clans, lineages or bilateral kindreds generally;
- marriage, residence rules or descent systems;
- friendship, exchange, alliance or political obligation;
- culturally differentiated maternal versus paternal ties;
- kin-distance decay, relationship strength or household fission;
- empirical prehistoric mobility preferences.

The synthetic default `kinWeight` remains a mechanism-testing value, not a measured social coefficient. A study that interprets kin-sensitive migration must evidence-ground or structurally sensitivity-test an appropriate social model rather than treating this null proxy as anthropology.

## Verification invariants

The implementation must prove with controlled tests that:

- an otherwise-equivalent cross-household female-parent link and male-parent link create the same kin locations and utilities;
- every cross-household parent-child link is reciprocal between the two households;
- same-household parent-child links create no residence-specific anchor;
- more than four unique kin locations remain represented;
- changing only irrelevant person-record/child insertion order cannot change the represented kin-location set or its cell-wise utility;
- with all non-kin attraction/action terms neutralized, a reciprocal first-degree-kin cell increases residence utility by exactly the configured kin contribution.

These are model-verification claims only. They do not validate the proxy against archaeological or ethnographic evidence.
