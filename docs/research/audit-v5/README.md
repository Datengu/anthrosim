# AnthroSim scientific audit v5 — charter

Scientific Audit v5 is the fifth independent/adversarial comprehensive scientific audit of AnthroSim and the next convergence audit after completion of Audit v4 remediation and preservation of the `v0.3.5` release. It audits the immutable `v0.3.5` release baseline from zero coverage. It is not a continuation of Audit v4 and is not empirical validation of any archaeological reconstruction.

Protocol: `docs/research/scientific-audit-protocol.md`

Post-discovery re-verification under repository/version evolution: `docs/research/audit-reverification-version-drift.md`

Authoritative ledger: `docs/research/audit-v5/STATUS.md`

## Fixed initial target

Audit v5 starts from the immutable release identity:

- release tag: `v0.3.5`;
- exact tagged commit: `e7667af52d48a1ffbae2bf7713a2388e65994b42`;
- software version: `0.3.5`;
- executable model semantics: `anthrosim-model-semantics-v33`.

The tag is the fixed discovery target. Audit documentation and later repairs may move protected `main`, but findings attributed to the released baseline must be demonstrated against the tagged baseline or source shown to be causally equivalent. Living/current documentation must continue to describe the current repository rather than being rewritten to impersonate the frozen target solely for historical audit compatibility.

## Independence from prior audits

Audits v2, v3 and v4 are historical evidence and regression-hypothesis sources. They are not completion evidence for Audit v5.

Every Area A–N must be independently inspected against the `v0.3.5` / v33 baseline with fresh falsification-oriented evidence. Prior findings may guide attacks, but merely citing a previous audit, closed issue, green CI run, preserved benchmark, release gate or regression test cannot complete an Area.

Audit v5 must specifically challenge the integrated Audit-v4 repair line, including model-semantics changes v26–v33 and their interactions, rather than assuming independently repaired contracts compose correctly.

## Convergence objective

The desired outcome is a **P1-clean convergence pass**: a fresh full A–N audit that discovers no new P0 or P1 scientific defect.

- Severity follows the reusable protocol without downgrading.
- Every demonstrated finding is preserved before repair.
- Any new P0/P1 means Audit v5 is a non-clean convergence pass even if later repaired and independently reverified.
- P2/P3 findings still require explicit disposition.
- A clean audit is evidence of scientific-process convergence, not proof of correctness or empirical validity.

## Required audit surface

Audit v5 covers the complete A–N surface in `docs/research/scientific-audit-protocol.md`, including the required cross-system integration pass.

## Additional v5 anti-confirmation rules

- Start every Area at zero coverage.
- Require at least one genuinely fresh adversarial construction per Area rather than replaying only an Audit-v4 adversary.
- Treat Audit-v4 repairs and their new shared coupling/provenance machinery as hypotheses to attack from neighbouring mechanisms and coupled configurations.
- Re-run earlier adversaries when scientifically useful, but do not count them as the sole fresh evidence for an Area.
- Prefer exact symmetry, relabelling, limiting-case, boundary, initialization, horizon, stochastic-precision, estimator-weighting, resume/provenance, structural-counterfactual and cross-mechanism composition tests where scientifically relevant.
- Challenge public configuration surfaces, defaults and unsupported/edge combinations rather than assuming representative settings.
- Treat preserved references, benchmarks, release checks and existing regression suites as claims to challenge, not ground truth.
- Audit analysis, documentation, checkpointing, provenance and experiment orchestration as part of the scientific system.
- Explicitly test whether repairs that remove one arbitrary identity/order dependence introduce another shared coupling, equivalence-class or tie-handling dependence elsewhere.
- Do not repair production behaviour during discovery until the full A–N discovery pass is complete, unless a repository-integrity emergency makes continued discovery impossible; document any exception explicitly.

## Finding workflow

For every demonstrated defect:

1. preserve exact `v0.3.5` SHA / v33 semantics and reproduction evidence;
2. search open and closed issues/PRs for overlap;
3. create one issue for the smallest underlying scientific defect;
4. assign protocol severity from scientific consequence;
5. assign the next sequential Audit-v5 identifier (`AV5-001`, `AV5-002`, ...);
6. record it in `STATUS.md` before repair;
7. continue discovery against immutable `v0.3.5`;
8. after discovery completes, repair findings on dedicated branches/PRs and independently reverify them under the reusable protocol plus the version-drift addendum when needed.

## Discovery versus remediation

Audit v5 discovery and Audit v5 remediation are distinct phases.

- Discovery remains anchored to immutable `v0.3.5` / v33 even if audit-documentation commits advance `main`.
- Findings are preserved before any production repair.
- Full A–N discovery completes before ordinary production remediation begins.
- After discovery, each P0/P1 repair requires exact-head production validation and independent post-merge re-verification of the original adversarial scientific contract.
- Historical discovery evidence is never rewritten to make a later repaired repository appear as though the original defect was absent.

## Empirical boundary

Audit v5 assesses AnthroSim as a scientific simulation instrument under its declared assumptions. It does not establish empirical validity for a particular place, period, population or archaeological reconstruction.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v5/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v5/STATUS.md`. Verify live `main`, immutable `v0.3.5`, open issues/PRs and overlapping work. Continue the next incomplete Audit-v5 Area from first principles using fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, and update the repository-authoritative ledger before handoff.
