# M2 parentage relatedness scope v1

Status: living scientific contract.

## Scope

AnthroSim M2 parentage is intentionally a **residence-local eligible-male null model**. The current authoritative parentage mechanism does not model marriage, pair bonding, incest avoidance, inbreeding avoidance, kinship-based mate choice, or any other relatedness constraint on male-parent eligibility.

A male candidate is eligible according to the executable M2 parentage rules for living state, reproductive sex, configured male-parent age range, and the applicable demographic residence exposure. Persisted genealogy is **not** consulted to exclude a candidate from parentage selection.

This means that even when founder genealogy is declared complete for living direct parents, a first-degree biological relative can be selected as the male parent of a later child if he otherwise satisfies the M2 eligibility rules. In particular, a woman's declared biological father can become the male parent of her child when he is the only eligible residence-local male. This is deliberate null-model scope, not evidence that such a pairing is historically, socially, or biologically realistic.

## Interpretation boundary

Generated M2 parentage must therefore not be interpreted as a realistic mating network, marriage system, incest-avoidance system, or empirically defensible kinship structure without an explicit structural alternative that adds those semantics.

Research conclusions may use M2 parentage as the model's minimal local reproductive-link mechanism. They may not treat the resulting genealogy, consanguinity rate, mate-relatedness distribution, or downstream kin-driven patterns as empirically realistic merely because the simulation records genealogy authoritatively.

Any study whose conclusion materially depends on mate relatedness, marriage rules, close-kin exclusion, inbreeding avoidance, or realistic kin-network structure requires a separately declared structural model or sensitivity analysis. Absence of a genealogy link must not be treated as positive evidence that two people are unrelated when founder genealogy is incomplete or unspecified.

## Downstream consequence

Although relatedness is outside M2 mate eligibility, generated parentage remains authoritative genealogy and can affect later mechanisms that consume kin structure. Therefore downstream effects caused by this null parentage rule are model-structural consequences, not independently validated human-demographic predictions.

## Executable regression

`crates/anthrosim-core/tests/m2_parentage_relatedness_scope.rs` permanently fixes this scope contract. It constructs complete living-direct-parent genealogy in which the only fertility-eligible female is the adult daughter and her own living father is the only residence-local eligible male. The expected authoritative birth records that father as the child's male parent.

Changing that regression requires an explicit scientific model-scope decision, updated living documentation, compatibility/semantics assessment, and appropriate audit treatment; it must not drift silently as an incidental implementation change.

## Audit-v5 disposition

This document is the explicit post-discovery scope disposition for AV5-002 / issue #617. Audit-v5 demonstrated that first-degree parentage is executable and previously insufficiently disclosed. The chosen remediation is to preserve the minimal local eligible-male null model while making its relatedness limitation explicit and executable, rather than silently adding genealogy-aware mate exclusion without a separately justified scientific model.