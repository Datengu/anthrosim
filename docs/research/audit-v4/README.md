# AnthroSim scientific audit v4 — charter

Scientific audit v4 is the fourth independent/adversarial comprehensive scientific audit of AnthroSim and the next convergence audit after Audit v3 remediation. It audits the immutable `v0.3.4` release baseline from zero coverage. It is not a continuation of Audit v3 and is not empirical validation of any archaeological reconstruction.

Protocol: `docs/research/scientific-audit-protocol.md`

Authoritative ledger: `docs/research/audit-v4/STATUS.md`

## Fixed initial target

Audit v4 starts from the immutable release identity:

- release tag: `v0.3.4`;
- exact tagged commit: `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09`;
- software version: `0.3.4`;
- executable model semantics: `anthrosim-model-semantics-v25`.

The tag is the fixed discovery target. Audit documentation and later repairs may move protected `main`, but findings attributed to the released baseline must be demonstrated against the tagged baseline or source shown to be causally equivalent.

## Independence from prior audits

Audit v2 and Audit v3 are historical evidence and regression-hypothesis sources. They are not completion evidence for Audit v4.

Every Area A–N must be independently inspected against the v0.3.4/v25 baseline with fresh falsification-oriented evidence. Prior findings may guide attacks, but merely citing a previous audit, closed issue, green CI run, or preserved benchmark cannot complete an Area.

Audit v4 must specifically challenge the integrated repair line created after Audit v3 rather than assuming repaired contracts compose correctly.

## Convergence objective

The desired outcome is a **P1-clean convergence pass**: a fresh full A–N audit that discovers no new P0 or P1 scientific defect.

- Severity follows the reusable protocol without downgrading.
- Every demonstrated finding is preserved before repair.
- Any new P0/P1 means Audit v4 is a non-clean convergence pass even if later repaired and independently reverified.
- P2/P3 findings still require explicit disposition.
- A clean audit is evidence of scientific-process convergence, not proof of correctness or empirical validity.

## Required audit surface

Audit v4 covers the complete A–N surface in `docs/research/scientific-audit-protocol.md`, including the required cross-system integration pass.

## Additional v4 anti-confirmation rules

- Start every Area at zero coverage.
- Require at least one fresh adversarial construction per Area rather than relying solely on a prior-audit regression.
- Treat Audit-v3 repairs as hypotheses to attack from neighbouring mechanisms and coupled configurations.
- Prefer exact symmetry, relabelling, limiting-case, boundary, initialization, horizon, stochastic-precision, estimator-weighting, resume/provenance, and structural-counterfactual tests where scientifically relevant.
- Challenge public configuration surfaces and defaults rather than assuming representative settings.
- Treat preserved references and benchmarks as claims to test, not ground truth.
- Audit analysis/documentation/provenance machinery as part of the scientific system.
- Do not repair production behaviour during discovery until the full A–N discovery pass is complete, unless a repository integrity emergency makes continued discovery impossible; document any exception explicitly.

## Finding workflow

For every demonstrated defect:

1. preserve exact v0.3.4 SHA/v25 semantics and reproduction evidence;
2. search open and closed issues/PRs for overlap;
3. create one issue for the smallest underlying scientific defect;
4. assign protocol severity from scientific consequence;
5. record it in `STATUS.md` before repair;
6. continue discovery against immutable v0.3.4;
7. after discovery completes, repair findings on dedicated branches/PRs and independently reverify them.

## Empirical boundary

Audit v4 assesses AnthroSim as a scientific simulation instrument under its declared assumptions. It does not establish empirical validity for a particular place, period, population, or archaeological reconstruction.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v4/README.md`, and `docs/research/audit-v4/STATUS.md`. Verify live `main`, immutable `v0.3.4`, open issues/PRs and overlapping work. Continue the next incomplete Audit-v4 Area from first principles using fresh adversarial/quantitative evidence, and preserve demonstrated defects before repair.
