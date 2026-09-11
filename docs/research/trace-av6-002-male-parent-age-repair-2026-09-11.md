# TRACE — AV6-002 male-parent age time-reference repair — 2026-09-11

Issue: #694 / AV6-002 (P2).

## Problem demonstrated

Audit-v6 Area B preserved a controlled counterexample showing that `maleParentMinAgeYears` / `maleParentMaxAgeYearsExclusive` had two temporal meanings. Generated M2 genealogy evaluated the male at annual exposure start `t-365`, while declared founder genealogy evaluated the same parent-child relation at the child's birth day.

The discovery adversary covered both configured boundaries: a 69y200d interval-start male could father a child when already 70y200d at birth, while a 17y200d interval-start male was excluded despite being 18y200d at birth.

## Repair decision

The public male-parent age window now constrains completed male age at the authoritative child-birth boundary `t` in both generated M2 and declared founder genealogy.

This deliberately does not change female fertility-band or background-mortality schedule timing: those remain interval-start annual exposure schedules. It also does not change the pre-same-day-M4 residence rule used to reconstruct parentage locality.

## Verification target

Permanent regression `av6_002_male_parent_age_time_reference.rs` exercises both crossings and requires dynamic and declared genealogy to agree:

- 70y200d at child birth: rejected by both paths;
- 18y200d at child birth: accepted by both paths.

Demography observability replay uses the same birth-boundary male-age rule, so derived eligibility cannot silently disagree with authoritative execution.

## Compatibility

Because the repair may add/remove a birth or alter the eligible parent pool near a configured age threshold, authoritative future state can diverge from a v38 checkpoint. Living model semantics therefore advance to `anthrosim-model-semantics-v39`; package version remains `0.3.6` and immutable v0.3.6/v35 evidence is unchanged.

## Current demographic reference refresh

The semantics change was propagated through the current #304 confirmatory demographic reference rather than relabelling v38 output. Workflow run `34563009772` / job `103149422561` completed all 780 frozen-design runs and all three Monte Carlo precision gates. Artifact `10185085281` (SHA-256 `989a518b824d2b924152915138d140536b5e846a29e5213fa173746886a1e248`) contains the exact v39 `expected-result.json` used to refresh `research/general-demography-baseline-v1/confirmatory-result.json`. The recommendation remains `no_universal_demographic_baseline`; quantitative outputs changed and are therefore preserved as a true v39 rebaseline.
