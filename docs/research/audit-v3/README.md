# AnthroSim scientific audit v3 — charter

Scientific audit v3 is the third independent/adversarial comprehensive scientific audit of AnthroSim. It is a convergence audit of the immutable `v0.3.3` release baseline, not a continuation of audit v2 and not an empirical validation of any archaeological reconstruction.

Protocol: `docs/research/scientific-audit-protocol.md`

Authoritative ledger: `docs/research/audit-v3/STATUS.md`

## Fixed initial target

Audit v3 starts from the immutable release identity:

- release tag: `v0.3.3`;
- exact tagged commit: `d3b6fc0b0750933b57252c9087513d156d88f218`;
- software version: `0.3.3`;
- executable model semantics: `anthrosim-model-semantics-v21`.

The tag is the fixed discovery target. Audit documentation and any later repairs may move protected `main`, but a finding attributed to the released baseline must be demonstrated against the tagged baseline or shown to be causally equivalent to it.

## Independence from audit v2

Audit v2 is historical evidence and a source of known failure modes. It is **not** completion evidence for audit v3.

For every Area A–N, audit v3 must independently inspect the authoritative implementation and current scientific documentation and must execute or construct fresh falsification-oriented evidence. Merely citing an audit-v2 result, previously green CI, a closed issue, or a preserved benchmark is insufficient to mark an Area complete.

Prior findings should be used as regression hypotheses: where practical, v3 should try to break the repaired contract from a different direction rather than simply rerun the exact original test.

## Convergence objective

The project-level objective is stronger than merely repairing whatever this audit finds: reach a point where a **fresh full-scale audit discovers no new P0 or P1 scientific defect**.

Therefore:

1. audit v3 uses the protocol severity definitions without downgrading findings to manufacture convergence;
2. every demonstrated P0/P1 is preserved, issued, repaired, and independently reverified before the affected repair line is considered scientifically closed;
3. if audit v3 discovers any new P0/P1, audit v3 may still be completed after repair/reverification, but it does **not** count as the desired clean convergence pass;
4. after any v3 P0/P1 repair line is stabilized and checkpointed, a later audit generation must again start from zero coverage to test whether the integrated system can complete A–N without discovering a new P0/P1;
5. a clean full audit is evidence of scientific-process convergence, never proof of correctness.

P2/P3 findings must still receive explicit dispositions. They must not be hidden or reclassified merely because they are inconvenient to the convergence goal.

## Required audit surface

Audit v3 covers the complete A–N surface defined by `docs/research/scientific-audit-protocol.md`:

- A — authoritative semantics and scheduler behaviour;
- B — demography, fertility, mortality, ageing, and population structure;
- C — households, kinship, social links, and lifecycle structure;
- D — resources, condition, subsistence, and depletion/recovery;
- E — spatial landscape, movement, migration, temporary mobility, and boundaries;
- F — aggregation and interaction mechanisms;
- G — initialization, burn-in, path dependence, and continuation state;
- H — stochasticity, RNG, ensembles, and Monte Carlo inference;
- I — sensitivity, uncertainty, convergence, and robustness;
- J — identifiability, equifinality, calibration, and discrimination;
- K — experiment orchestration, configuration, provenance, and reproducibility;
- L — observability, analysis outputs, and statistical summaries;
- M — documentation, TRACE/ODD/ODD+D, and claim consistency;
- N — cross-system integration.

## Additional v3 anti-confirmation rules

To make this pass genuinely independent rather than a mechanical repeat of v2:

- each Area must record at least one fresh adversarial question or construction not used as its sole completion evidence in audit v2;
- symmetry, limiting-case, relabelling, boundary, initialization, horizon, stochastic-precision, estimator-weighting, resume/provenance, and structural-counterfactual attacks should be reused across different mechanisms where they are scientifically relevant;
- apparently safe defaults must be challenged through the public configuration surface rather than assumed representative;
- preserved reference outputs are hypotheses to verify, not ground truth to trust automatically;
- documentation and analysis code are part of the scientific system and may generate P0/P1 findings when they can materially change or misstate a supported conclusion;
- locally correct subsystems must still be attacked in coupled configurations during Area N.

## Finding and repair workflow

For every demonstrated defect:

1. preserve exact baseline SHA/semantics and reproduction evidence;
2. search open and closed issues/PRs for overlap;
3. create one issue for the smallest underlying scientific defect;
4. assign severity from scientific consequence;
5. record it in `STATUS.md` before repair erases the failure mode;
6. repair on a dedicated branch/PR;
7. run local, neighbouring-system, and original/fresh adversarial verification;
8. update semantics/provenance/documentation identities where scientifically required;
9. require normal protected CI and applicable scientific/security gates;
10. record `fixed` separately from `independently reverified`.

## Completion rule

Audit v3 is complete only when the reusable protocol's comprehensive completion criteria are satisfied and the ledger has reconciled all Areas A–N, findings, issues, PRs, model identities, and final baseline state.

For the user's convergence goal, there is an additional outcome label:

- **P1-clean convergence pass:** the full fresh audit discovered no new P0 or P1 finding anywhere in A–N.
- **non-clean convergence pass:** one or more new P0/P1 findings were discovered, even if all were subsequently repaired and reverified.

A non-clean v3 audit is useful progress, but it means another fresh full audit is required before beginning the first empirical site study.

## Empirical boundary

Even a P1-clean audit does not validate AnthroSim for a specific archaeological site or any other prehistoric case. It supports confidence in the simulator as a scientific instrument under its declared assumptions. A later site-specific study still requires an explicit question, evidence roles, uncertainty/sensitivity design, identifiability analysis, Monte Carlo precision, model comparison, and held-out corroboration where feasible.

## Cross-session start instruction

A new audit agent should be able to continue with only:

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v3/README.md`, and `docs/research/audit-v3/STATUS.md`. Verify live `main`, immutable `v0.3.3`, open issues/PRs and overlapping work. Continue the next incomplete audit-v3 Area from first principles, using fresh adversarial/quantitative evidence and preserving any demonstrated defect before repair.
