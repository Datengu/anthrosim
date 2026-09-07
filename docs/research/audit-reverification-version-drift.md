# Audit re-verification under repository evolution

Status: normative remediation addendum for scientific-audit post-discovery re-verification.

This addendum resolves a class of audit-harness failures that can occur when a finding is discovered against an immutable historical target but is repaired only after the living repository has legitimately advanced beyond that target's software, model-semantics, schema, compiler, formatting, or documentation identity.

The historical discovery target remains immutable. Living AnthroSim documentation and identifiers must describe the current repaired repository and must never be regressed merely to satisfy an obsolete discovery-harness positive control.

## Required evidence layers

Post-discovery re-verification separates two evidence layers:

1. **Historical adversary preservation.** The original discovery adversary, workflow and recorded results remain preserved byte-for-byte in their original evidence PR/commit history. They establish exactly what failed on the frozen audit target.
2. **Current-state re-verification.** The repaired finding must be challenged against the exact merged `main` being certified. Re-verification should rerun the original adversary unchanged whenever its harness remains forward-compatible. When it does not, a narrowly adapted derivative may be used under the rules below.

## When adaptation is permitted

A derivative adversary is permitted only when the unchanged historical adversary cannot reach its substantive scientific assertion because a positive control or harness assumption is intrinsically tied to the frozen discovery target and is no longer truthful for current `main`.

Examples include assertions that a historical model-semantics identifier is still the *current* identifier, an obsolete schema/compiler/formatting assumption, or a discovery-era terminal guard whose sole purpose was to prove defect presence on the frozen target.

Adaptation is not permitted merely because a substantive scientific assertion still fails after repair.

## Adaptation constraints

A current-state derivative must:

- start from the exact merged production `main` being certified;
- preserve the original adversary in repository/Git history unchanged;
- enumerate every changed harness assertion in the evidence PR and finding issue;
- change only the minimum frozen-target positive controls or non-scientific harness logic required to reach the substantive adversarial assertion;
- preserve the original scientific failure oracle, stale-claim list, invariance condition, quantitative threshold, or equivalent substantive test semantics without weakening them;
- positively verify both the immutable historical baseline identity and the current living repository identity where version drift is the reason for adaptation;
- run normal exact-head CI plus any dedicated scientific/security workflow applicable to the finding;
- be closed unmerged after classification, unless the derivative is separately promoted as a permanent regression through an ordinary production PR;
- record exact production merge SHA, derivative evidence head, workflow run/job/result, and the complete adaptation list on the original finding.

If the adapted derivative passes, the finding may be classified as independently reverified only when the production repair is present on merged `main`, the normal protected/scientific matrix is acceptable, and no substantive scientific assertion was weakened.

If the derivative still fails at the substantive oracle, the finding remains open.

## Frozen baseline versus living current state

Audit ledgers must keep these concepts distinct:

- **Frozen audit target:** the immutable release/SHA/model-semantics combination against which discovery evidence was produced.
- **Current remediation line:** the living repaired repository state being certified after fixes.

Historical identifiers may remain in audit records, release notes and explicitly historical sections. Current-facing ODD, ODD+D, scientific-model, provenance and other living surfaces must identify and describe the current repository state rather than impersonating the frozen audit target.

## Audit-v4 AV4-015 application

For AV4-015/#549, discovery PR #548 correctly used `current model semantics v25` as a positive control on immutable `v0.3.4`/v25. After Audit-v4 remediation advanced living `main` to `anthrosim-model-semantics-v33`, that exact positive control became intrinsically obsolete and prevents the historical script from reaching its unchanged three-phrase mortality-drift oracle.

The required current-state derivative therefore may replace only the three `current model semantics v25` positive controls/diagnostic labels with checks that:

- v25 remains explicitly recorded as the immutable Audit-v4/v0.3.4 baseline;
- living ODD, ODD+D and scientific-model identify current semantics v33;
- the original three stale mortality phrases remain exactly the scientific failure oracle;
- the existing positive controls for elapsed-M3 competing mortality and year-end fertility/parentage-only semantics remain in force.

No living production document should be changed back to calling v25 current.