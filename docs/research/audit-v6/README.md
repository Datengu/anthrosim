# AnthroSim scientific audit v6 — charter

**Status: active charter / discovery initialized.** Scientific Audit v6 is the sixth independent/adversarial comprehensive scientific audit of AnthroSim and the next convergence audit after completed Audit-v5 remediation and preservation of the `v0.3.6` release. It audits the immutable `v0.3.6` release baseline from zero coverage. It is not a continuation of Audit v5, does not inherit Audit-v5 completion evidence, and is not empirical validation of any real-world reconstruction.

Protocol: `docs/research/scientific-audit-protocol.md`

Post-discovery re-verification under repository/version evolution: `docs/research/audit-reverification-version-drift.md`

Authoritative ledger: `docs/research/audit-v6/STATUS.md`

## Fixed initial target

Audit v6 starts from the immutable release identity:

- release tag: `v0.3.6`;
- exact tagged commit: `7d5e47309556e458477cd7283230871363b2c89a`;
- software version: `0.3.6`;
- executable model semantics: `anthrosim-model-semantics-v35`.

The tag is the fixed discovery target. Audit documentation and later repairs may move protected `main`, but findings attributed to the released baseline must be demonstrated against the tagged baseline or source shown to be causally equivalent. Living/current documentation must continue to describe the current repository rather than being rewritten to impersonate the frozen target solely for historical audit compatibility.

## Why v6 exists

Audit v5 demonstrated 8 findings, including 4 P1 scientific defects. All eight findings were subsequently repaired/dispositioned and the P1 repairs independently re-verified, producing the v35 line frozen as `v0.3.6`. Because Audit v5 discovered new P1 defects, it was a **non-clean convergence pass** under the reusable protocol.

Audit v6 therefore exists to answer a narrower process question: has the fully integrated repaired framework now converged far enough that a genuinely fresh full A–N audit discovers no new P0/P1 scientific defect?

A clean v6 result would be evidence of scientific-process convergence. It would not prove correctness, calibrate the model, validate any archaeological interpretation, or remove the need for question-specific evidence and uncertainty analysis.

## Independence from prior audits

Audits v2, v3, v4 and v5 are historical evidence and regression-hypothesis sources. They are not completion evidence for Audit v6.

Every Area A–N must be independently inspected against the `v0.3.6` / v35 baseline with fresh falsification-oriented evidence. Prior findings may guide attacks and repaired adversaries may be rerun as controls, but merely citing a previous audit, closed issue, green CI run, preserved benchmark, release gate, regression test or prior re-verification cannot complete an Area.

Audit v6 must challenge the integrated post-Audit-v5 repaired line as a whole, including the cumulative v26–v35 semantics changes and especially the v34/v35 coupling/spatial-equivalence repairs, rather than assuming independently repaired scientific contracts compose correctly.

## Convergence objective

The desired outcome is a **P1-clean convergence pass**: a fresh full A–N audit that discovers no new P0 or P1 scientific defect.

- Severity follows the reusable protocol without downgrading.
- Every demonstrated finding is preserved before repair.
- Any new P0/P1 means Audit v6 is a non-clean convergence pass even if later repaired and independently reverified.
- P2/P3 findings still require explicit disposition.
- A clean audit is evidence of framework/process convergence, not proof of correctness or empirical validity.

If v6 is non-clean, discovery still completes across A–N before ordinary remediation begins. After remediation, the repaired line must be frozen as a new immutable release and another fresh audit generation is required before the empirical-readiness convergence gate can be considered satisfied.

## Required audit surface

Audit v6 covers the complete A–N surface in `docs/research/scientific-audit-protocol.md`, including the required cross-system integration pass.

Coverage starts at **0/14**. No Area is credited from Audit v5 or any earlier generation.

## Additional v6 anti-confirmation rules

- Start every Area at zero coverage.
- Require at least one genuinely fresh adversarial construction per Area rather than replaying only an earlier-audit adversary.
- Treat all Audit-v5 repairs, their replacement mechanisms and their interactions as hypotheses to attack, not as established truth.
- In particular, probe whether locality, relabelling invariance, spatial equivariance, equivalence-class handling, exact parameter-coordinate arithmetic, inference guards, survivor-window alignment, replay seed identity and documented null-model boundaries fail under neighbouring mechanisms or changed but scientifically equivalent representations.
- Explicitly test whether a repair that removed one arbitrary identity/order dependence introduced another hidden canonicalization, ranking, representative-selection, tie-breaking or equivalence-class dependence.
- Challenge composition across independently repaired mechanisms. Local correctness does not establish coupled correctness.
- Re-run earlier adversaries when scientifically useful as positive regression controls, but do not count them as the sole fresh evidence for an Area.
- Prefer exact symmetry, relabelling, insertion/removal locality, limiting-case, boundary, initialization, horizon, stochastic-precision, estimator-weighting, resume/provenance, structural-counterfactual and cross-mechanism composition tests where scientifically relevant.
- Challenge public configuration surfaces, defaults, unsupported/edge combinations and representation changes rather than assuming representative settings.
- Treat preserved references, benchmark classifications, release checks and existing regression suites as claims to challenge, not ground truth.
- Audit analysis, documentation, checkpointing, provenance and experiment orchestration as part of the scientific system.
- Search both historical and current issues/PRs before creating a finding so that one underlying scientific defect receives one authoritative issue.
- Do not repair production behaviour during discovery until the full A–N discovery pass is complete, unless a repository-integrity emergency makes continued discovery impossible; document any exception explicitly.
- Keep the audit case-study-neutral. Do not introduce site-specific evidence, calibration, desired outcomes or empirical assumptions into the framework audit.

## Finding workflow

For every demonstrated defect:

1. preserve exact `v0.3.6` SHA / v35 semantics and reproduction evidence;
2. search open and closed issues/PRs for overlap;
3. create one issue for the smallest underlying scientific defect;
4. assign protocol severity from scientific consequence;
5. assign the next sequential Audit-v6 identifier (`AV6-001`, `AV6-002`, ...);
6. record it in `STATUS.md` before repair;
7. continue discovery against immutable `v0.3.6`;
8. after discovery completes, repair findings on dedicated branches/PRs and independently reverify P0/P1 findings under the reusable protocol plus the version-drift addendum when needed.

Evidence-only adversarial branches/PRs must not silently mutate production semantics. Their purpose is to preserve reproducible audit evidence.

## Discovery versus remediation

Audit v6 discovery and Audit v6 remediation are distinct phases.

- Discovery remains anchored to immutable `v0.3.6` / v35 even if audit-documentation commits advance `main`.
- Findings are preserved before any production repair.
- Full A–N discovery completes before ordinary production remediation begins.
- After discovery, each P0/P1 repair requires exact-head production validation and independent post-merge re-verification of the original adversarial scientific contract.
- Historical discovery evidence is never rewritten to make a later repaired repository appear as though the original defect was absent.
- Evidence-only re-verification remains separate from production repair and should not be merged merely to make the repaired branch appear to contain the adversarial evidence.

## Audit target versus live repository evolution

The immutable release tag controls attribution of discovery findings. At the start of every audit session, agents must still reconstruct live repository state as required by the reusable protocol.

If protected `main` advances during v6 discovery:

- continue to attribute v6 discovery to immutable `v0.3.6` / v35;
- determine whether the changed live source is causally relevant to the Area being audited;
- do not mix numerical evidence from semantically different heads without explicit labels;
- preserve exact source identity for every experiment;
- record whether any evidence must later be repeated on a repaired/current head.

## Parallel-agent ownership

Parallel work is permitted only when Areas and shared scientific machinery are clearly separated in `STATUS.md`.

Before beginning work, an agent must verify:

- live `main` and the immutable v6 tag;
- open issues and PRs;
- active audit branches and overlapping ownership;
- the Area/sub-area it is claiming in the ledger.

Shared scheduler, RNG/coupling, configuration/provenance, experiment-analysis and checkpoint machinery often crosses several Areas. Prefer sequential auditing or explicit ownership boundaries when parallel work would make evidence attribution ambiguous.

## Empirical boundary

Audit v6 assesses AnthroSim as a scientific simulation instrument under its declared assumptions. It does not establish empirical validity for a particular place, period, population, dataset or reconstruction.

Even a P1-clean v6 result would satisfy only the repository's framework-convergence gate. Any later empirical study would still require its own question formulation, evidence roles, parameterization/calibration choices, uncertainty and sensitivity analysis, identifiability/equifinality assessment, held-out corroboration where possible and relevant domain review.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.
