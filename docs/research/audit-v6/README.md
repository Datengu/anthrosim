# AnthroSim scientific audit v6 — charter

**Status: discovery complete / non-clean convergence pass / remediation next.** Scientific Audit v6 is the sixth independent/adversarial comprehensive scientific audit of AnthroSim and the convergence audit after completed Audit-v5 remediation and preservation of the `v0.3.6` release. It audited the immutable `v0.3.6` release baseline from zero coverage. It is not empirical validation of any real-world reconstruction.

Protocol: `docs/research/scientific-audit-protocol.md`

Post-discovery re-verification under repository/version evolution: `docs/research/audit-reverification-version-drift.md`

Authoritative ledger: `docs/research/audit-v6/STATUS.md`

## Fixed target

Audit v6 is anchored to the immutable release identity:

- release tag: `v0.3.6`;
- exact tagged commit: `7d5e47309556e458477cd7283230871363b2c89a`;
- software version: `0.3.6`;
- executable model semantics: `anthrosim-model-semantics-v35`.

The tag is the fixed discovery target. Audit documentation and later repairs may move protected `main`, but findings attributed to the released baseline remain demonstrated against the tagged baseline or causally equivalent source. Historical discovery evidence must not be rewritten to make later repaired code appear as though the original defect was absent.

## Why v6 exists

Audit v5 demonstrated 8 findings, including 4 P1 scientific defects. All eight findings were subsequently repaired/dispositioned and the P1 repairs independently re-verified, producing the v35 line frozen as `v0.3.6`. Because Audit v5 discovered new P1 defects, it was a **non-clean convergence pass** under the reusable protocol.

Audit v6 therefore asked a narrower process question: had the fully integrated repaired framework converged far enough that a genuinely fresh full A–N audit discovered no new P0/P1 scientific defect?

The answer is **no**. Audit-v6 discovery completed all 14 Areas and demonstrated 14 findings: 7 P1 and 7 P2. No P0/P3 finding was assigned. Therefore v6 is itself a **non-clean convergence pass** and cannot satisfy the P1-clean convergence objective even after remediation.

This result is evidence about the audit/convergence process, not empirical validity. It does not calibrate the model, validate an archaeological interpretation or remove the need for question-specific evidence and uncertainty analysis.

## Discovery completion state

Audit-v6 A–N discovery is complete against immutable `v0.3.6` / v35.

Authoritative findings:

- P1: AV6-001/#687, AV6-004/#707, AV6-006/#711, AV6-009/#726, AV6-010/#729, AV6-012/#737, AV6-013/#741;
- P2: AV6-002/#694, AV6-003/#699, AV6-005/#708, AV6-007/#718, AV6-008/#721, AV6-011/#733, AV6-014/#745.

Area N supplied a fresh final integration attack that quantified AV6-006 propagation into temporary aggregation and M3 resource demand without opening a duplicate root issue. See `docs/research/audit-v6/area-n-2026-09-09.md`.

The ordinary discovery/remediation barrier is now lifted only because A–N discovery is complete. The next phase is controlled remediation/disposition of all 14 findings.

## Independence from prior audits

Audits v2, v3, v4 and v5 are historical evidence and regression-hypothesis sources. They were not completion evidence for Audit v6.

Every Area A–N was independently inspected against the `v0.3.6` / v35 baseline with fresh falsification-oriented evidence. Prior findings guided attacks and repaired adversaries sometimes served as controls, but no Area was completed merely by citing a previous audit, closed issue, green CI run, preserved benchmark, release gate, regression test or prior re-verification.

Audit v6 challenged the integrated post-Audit-v5 repaired line as a whole, including cumulative v26–v35 semantics changes and especially the v34/v35 coupling/spatial-equivalence repairs.

## Convergence objective and outcome

The desired outcome was a **P1-clean convergence pass**: a fresh full A–N audit discovering no new P0 or P1 scientific defect.

- Severity follows the reusable protocol without downgrading.
- Every demonstrated finding was preserved before repair.
- Any new P0/P1 makes Audit v6 non-clean even if later repaired and independently reverified.
- P2 findings still require explicit disposition.
- A clean audit would have been evidence of framework/process convergence, not proof of correctness or empirical validity.

Because v6 is non-clean, the repaired line must eventually be frozen as a new immutable release and another fresh audit generation is required before the empirical-readiness framework-convergence gate can be considered satisfied.

## Required audit surface

Audit v6 covered the complete A–N surface in `docs/research/scientific-audit-protocol.md`, including the required cross-system integration pass.

Coverage began at **0/14** and is now **14/14 complete**. No Area was credited from Audit v5 or an earlier generation.

## Anti-confirmation rules retained for evidence interpretation

- Start every audit Area at zero coverage.
- Require at least one genuinely fresh adversarial construction per Area rather than replaying only an earlier-audit adversary.
- Treat earlier repairs, replacement mechanisms and interactions as hypotheses to attack, not established truth.
- Explicitly test whether a repair that removed one arbitrary identity/order dependence introduced another hidden canonicalization, ranking, representative-selection, tie-breaking or equivalence-class dependence.
- Challenge composition across independently repaired mechanisms. Local correctness does not establish coupled correctness.
- Re-run earlier adversaries when scientifically useful as positive regression controls, but do not count them as the sole fresh evidence for an Area.
- Prefer exact symmetry, relabelling, insertion/removal locality, limiting-case, boundary, initialization, horizon, stochastic-precision, estimator-weighting, resume/provenance, structural-counterfactual and cross-mechanism composition tests where scientifically relevant.
- Challenge public configuration surfaces, defaults, unsupported/edge combinations and representation changes rather than assuming representative settings.
- Treat preserved references, benchmark classifications, release checks and existing regression suites as claims to challenge, not ground truth.
- Audit analysis, documentation, checkpointing, provenance and experiment orchestration as part of the scientific system.
- Search both historical and current issues/PRs before creating a finding so one underlying scientific defect receives one authoritative issue.
- Keep audit work case-study-neutral. Do not introduce site-specific evidence, calibration, desired outcomes or empirical assumptions into the framework audit.

## Finding workflow

For every demonstrated defect:

1. preserve exact `v0.3.6` SHA / v35 semantics and reproduction evidence;
2. search open and closed issues/PRs for overlap;
3. create one issue for the smallest underlying scientific defect;
4. assign protocol severity from scientific consequence;
5. assign the next sequential Audit-v6 identifier;
6. record it in `STATUS.md` before repair;
7. preserve immutable discovery evidence;
8. after discovery, repair findings on dedicated branches/PRs and independently reverify P0/P1 findings under the reusable protocol plus the version-drift addendum when needed.

Evidence-only adversarial branches/PRs must not silently mutate production semantics. Their purpose is to preserve reproducible audit evidence.

## Discovery versus remediation

Audit v6 discovery and remediation are distinct phases.

- Discovery remained anchored to immutable `v0.3.6` / v35 even as audit-only documentation commits advanced `main`.
- Findings were preserved before production repair.
- Full A–N discovery is now complete, so ordinary remediation may begin only after the authoritative completion ledger reaches protected `main`.
- Each P0/P1 repair requires exact-head production validation and independent post-merge re-verification of the original adversarial scientific contract.
- P2 findings require explicit disposition and regression evidence appropriate to their scientific consequence.
- Historical discovery evidence is never rewritten to make a later repaired repository appear as though the original defect was absent.
- Evidence-only re-verification remains separate from production repair and should not be merged merely to make a repaired branch appear to contain the adversarial evidence.

## Post-discovery remediation path

The remediation phase must:

1. reconstruct live `main`, all 14 open Audit-v6 issues and overlapping repair work before selecting a finding;
2. prioritize by severity and dependency rather than raw issue number;
3. implement one scientifically coherent repair per dedicated production PR unless a shared root change requires explicitly coordinated findings;
4. preserve existing neighbouring scientific contracts and run the complete applicable protected/scientific matrix;
5. independently re-run every P0/P1 adversarial contract after the production repair is merged;
6. keep issue state and `STATUS.md` explicit about `fixed` versus `reverified`;
7. freeze a new repaired immutable release only after all findings have been dispositioned and required re-verification is complete;
8. require another fresh comprehensive audit generation before treating the framework-convergence gate as satisfied.

## Audit target versus live repository evolution

The immutable release tag controls attribution of discovery findings. Remediation happens on the live repository and must follow `audit-reverification-version-drift.md` whenever later changes could affect the original acceptance contract.

Do not mix numerical evidence from semantically different heads without explicit labels. Preserve exact source identity for every repair and re-verification experiment.

## Parallel-agent ownership

Parallel remediation is permitted only when findings and shared scientific machinery are clearly separated in `STATUS.md`.

Before beginning work, verify:

- live `main` and the immutable v6 tag;
- open Audit-v6 issues and PRs;
- active repair/reverification branches and overlapping ownership;
- dependencies among findings.

Shared scheduler, RNG/coupling, configuration/provenance, experiment-analysis and checkpoint machinery often crosses several findings. Prefer sequential work or explicit ownership boundaries where parallel repair would make evidence attribution ambiguous.

## Empirical boundary

Audit v6 assesses AnthroSim as a scientific simulation instrument under its declared assumptions. It does not establish empirical validity for a particular place, period, population, dataset or reconstruction.

Because v6 is non-clean, the framework-convergence gate is not satisfied. Even after eventual convergence, any empirical study would still require its own question formulation, evidence roles, parameterization/calibration choices, uncertainty and sensitivity analysis, identifiability/equifinality assessment, held-out corroboration where possible and relevant domain review.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and `docs/research/audit-v6/area-n-2026-09-09.md`. Verify live `main`, all open Audit-v6 finding issues/PRs and overlapping work. Audit-v6 A–N discovery is complete against immutable `v0.3.6`/v35; do not redo discovery. Begin controlled remediation by severity/dependency, preserve original discovery evidence, and independently reverify every P0/P1 repair before closure.
