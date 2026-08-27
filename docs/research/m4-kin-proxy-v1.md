# M4 living-direct-parent kin proxy v1

**Status:** normative post-M9 scientific-hardening contract  
**Model semantics:** `anthrosim-model-semantics-v13`  
**Scientific status:** synthetic / unvalidated

## Purpose

M4 has a deliberately narrow genealogical residence term so a represented direct-parent relationship can affect permanent-migration utility without introducing a general social-network or kinship model. This contract defines that term so reproductive-sex role and packed person-record order cannot become accidental social rules.

## Authoritative rule

At each M4 decision boundary, for each household:

1. inspect every living household member;
2. inspect both represented direct-parent links (`female_parent` and `male_parent`);
3. ignore a missing/invalid parent link and a parent who is no longer living;
4. otherwise retain the parent's current persistent residence cell, **including when the parent belongs to the same household**;
5. deduplicate locations, but do not truncate the resulting set according to encounter order.

For any residence cell `c`:

```text
kinScore(c) = 250  if at least one retained living direct-parent location == c
              0    otherwise
```

Multiple parents at one cell do not stack. The configured `migration.kinWeight` multiplies this score in the ordinary M4 residence utility.

## Why co-resident parents count

Model-born children join the female parent's persistent household. Under the pre-v13 rule, M4 discarded every parent in the moving household and then retained at most the first four external parent locations. In normal model-generated families that made the female parent structurally unable to provide an external anchor while a male parent from another household could do so. The declared gender-neutral/direct-parent description therefore hid an effectively paternal spatial preference.

The v13 rule does not add a maternal, paternal, patrilocal, matrilocal or descent-system assumption. It removes the household-membership filter from the kin concept: if a represented living direct parent is at a cell, that cell is a direct-parent location whether the parent is co-resident or external.

A co-resident parent can consequently support the explicit **stay** utility at the current residence. An external parent can support a candidate residence. This is intentional symmetry, not an attempt to force movement toward kin.

## Record-order invariance

The kin-location set contains every unique represented living direct-parent cell. It has no first-four or first-N selection rule. Therefore reordering otherwise-equivalent person/birth records cannot cause a later parent location to disappear merely because another relationship happened to be encountered first.

The transient vector order used while collecting cells has no scientific meaning. M4 asks only whether an evaluated cell is present in the complete unique set.

## Scope and non-claims

This proxy is intentionally minimal. It does **not** represent:

- clans, lineages or bilateral kindreds;
- marriage, residence rules or descent systems;
- friendship, exchange, alliance or political obligation;
- culturally differentiated maternal versus paternal ties;
- kin-distance decay, relationship strength or household fission;
- empirical prehistoric mobility preferences.

The synthetic default `kinWeight` remains a mechanism-testing value, not a measured social coefficient. A study that interprets kin-sensitive migration must evidence-ground or structurally sensitivity-test an appropriate social model rather than treating this null proxy as anthropology.

## Verification invariants

The implementation must prove with controlled tests that:

- a living co-resident female parent and a living co-resident male parent are both valid parent-location anchors under otherwise equivalent declared state;
- external female and male direct parents are handled by the same collector rule;
- more than four unique parent locations remain represented;
- changing only irrelevant person-record/child insertion order cannot change the represented kin-location set or its cell-wise utility;
- with all non-kin attraction/action terms neutralized, adding a represented living direct parent at a candidate cell increases that candidate's utility by exactly the configured kin contribution.

These are model-verification claims only. They do not validate the proxy against archaeological or ethnographic evidence.
