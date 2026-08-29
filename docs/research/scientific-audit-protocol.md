# AnthroSim scientific audit protocol

Status: reusable living protocol for independent scientific audits of AnthroSim.

This document defines how a comprehensive scientific audit of AnthroSim must be performed, recorded, handed off between agents, and declared complete. It is intentionally stricter than ordinary code review. The purpose is not to demonstrate that AnthroSim is good; the purpose is to discover ways in which it could produce scientifically misleading, overconfident, irreproducible, non-identifiable, or implementation-dependent conclusions.

A scientific audit is therefore adversarial. Passing software tests, deterministic replay, reproducibility, documentation, or clean architecture are useful evidence, but none of them alone establish scientific validity.

## 1. Core audit principles

Every audit agent must follow these principles.

1. **Audit the live repository, not remembered state.** Before doing substantive work, fetch current `main`, current open issues, current open pull requests, active branches relevant to the area, and the authoritative model/documentation state. Do not rely on conversation handoff SHAs without verification.
2. **Separate model defects from model scope.** A deliberately minimal mechanism is not automatically a defect because it omits realism. A finding is scientifically important when the implementation, documentation, experiment machinery, or analysis permits a materially misleading inference relative to the model's stated scope.
3. **Prefer falsification over reassurance.** Construct tests that could fail the model, not merely tests expected to pass.
4. **Use quantitative evidence where possible.** Controlled experiments, invariance tests, limiting cases, replicate studies, perturbations, convergence analyses, analytical checks, and structural counterfactuals are preferred to impressionistic code review.
5. **Preserve findings before fixing them.** Establish the failure mode, evidence, affected scientific claim, and severity before implementation repair. Avoid silently repairing a defect and thereby erasing the audit trail.
6. **Do not audit your own fix as sufficient evidence.** A repaired P0/P1 finding requires independent re-verification against its acceptance criteria and, where relevant, integration with neighbouring mechanisms.
7. **One underlying scientific defect, one issue.** Do not fragment a single causal defect into many tickets, and do not combine unrelated defects merely because they occur in the same file.
8. **Search before creating issues.** Check open and recently closed issues/PRs for overlap, historical treatment, or deliberate design decisions.
9. **State uncertainty explicitly.** Distinguish demonstrated defects, plausible risks needing experiments, design limitations, and future model extensions.
10. **Do not upgrade empirical claims by implication.** AnthroSim's software/scientific-method quality and its empirical validity for a specific archaeological reconstruction are different questions.

## 2. Required audit startup procedure

At the start of every audit session, record in the audit status ledger:

- exact `main` commit SHA being audited;
- repository release/tag state relevant to that SHA;
- open issues and PRs that overlap the chosen audit area;
- model semantics identifier and any relevant scientific configuration/provenance version;
- audit area being entered;
- whether another agent is working on an overlapping area.

If `main` changes materially during an audit area, determine whether the evidence must be repeated on the new head. Do not mix numerical evidence from semantically different heads without labelling it.

## 3. Audit severity taxonomy

Use the following scientific severity scale.

### P0 — scientific integrity failure

A failure that can fundamentally invalidate trust in results or continuation state, including examples such as corrupted/reconstructed state being accepted as authoritative, uncontrolled nondeterminism where determinism is promised, provenance that can misidentify the executed scientific configuration, or experiment machinery that silently runs something different from what is reported.

### P1 — material scientific conclusion risk

A plausible scientific conclusion can be materially wrong, reversed, spuriously precise, or attributed to the wrong mechanism under supported use. Examples include order-dependent competing processes presented as order-invariant, hidden parameter omissions from sweeps, biased aggregate statistics, scientifically consequential initialization artefacts, or invalid stochastic inference.

### P2 — important scientific limitation or ambiguity

A weakness that materially restricts interpretation, robustness, uncertainty quantification, or transferability but is not normally expected to reverse a supported conclusion by itself. Examples include incomplete sensitivity coverage, weak diagnostics, poor observability, insufficient model-form comparison, or an undocumented dependence that is real but bounded.

### P3 — scientific robustness, clarity, or maintainability improvement

A useful improvement to documentation, diagnostics, ergonomics, methodology, or defensive testing where current scientific conclusions are not materially threatened.

Severity is about **scientific consequence**, not implementation difficulty.

## 4. Required audit surface

A comprehensive audit must cover all areas below. Areas may be split across agents, but no area is considered complete without explicit evidence in the status ledger.

### A. Authoritative semantics and scheduler behaviour

Audit:

- event ordering and simultaneous-process assumptions;
- competing risks and cause attribution;
- update frequency effects;
- hidden priority caused by iteration/container order;
- state read/write timing;
- boundary conditions between time steps;
- deterministic vs stochastic transition semantics;
- whether documentation matches executable semantics.

Adversarial questions:

- Would reordering equivalent agents/events change a scientific result?
- Does scheduler priority accidentally encode a causal assumption?
- Are events described as simultaneous actually sequential in a consequential way?
- Can a boundary transition double-count, skip, or overwrite state?

### B. Demography, fertility, mortality, ageing, and population structure

Audit:

- fertility eligibility and timing;
- mortality hazards and cause competition;
- age transitions;
- newborn initialization/inheritance;
- sex/role structure;
- mate limitation;
- population replacement behaviour;
- extinction/censoring handling;
- demographic stationarity claims;
- finite-population effects.

Required tests should include, where applicable, limiting cases, zero/near-zero hazards, competing hazards, fixed-population controls, long-run trajectories, and structural alternatives.

### C. Households, kinship, social links, and lifecycle structure

Audit:

- household formation/dissolution/fission;
- reciprocal relationship invariants;
- kinship graph consistency;
- lifecycle-induced demographic constraints;
- order dependence in relationship creation;
- household capacity/eligibility assumptions;
- structural effects on reproduction, movement, or aggregation.

Ask whether an apparently demographic effect is actually driven by household lifecycle or mate-network structure.

### D. Resources, condition, subsistence, and depletion/recovery

Audit:

- resource initialization;
- consumption and replenishment;
- condition gain/loss;
- resource-condition coupling;
- carrying-capacity-like emergent behaviour;
- depletion order effects;
- spatial extraction competition;
- burn-in dependence;
- realized vs nominal losses/costs;
- conservation/bounds where appropriate.

Check whether initial stock, update order, or replenishment cadence can dominate conclusions.

### E. Spatial landscape, movement, migration, temporary mobility, and boundaries

Audit:

- coordinate transforms and preprocessing;
- path/travel cost semantics;
- finite spatial boundaries;
- movement choice and tie-breaking;
- migration eligibility/triggers;
- temporary vs permanent movement distinctions;
- accessibility calculations;
- landscape loading determinism;
- edge effects;
- spatial observability and provenance.

Required adversarial cases should include symmetric landscapes, mirrored inputs, boundary-adjacent populations, zero-cost/equal-cost alternatives, unreachable cells, and transformed-landscape golden cases where relevant.

### F. Aggregation and interaction mechanisms

Audit:

- aggregation triggers;
- temporary concentration vs relocation;
- interaction opportunity accounting;
- crowding/resource consequences;
- recovery after aggregation;
- order/tie dependence;
- whether aggregate observables distinguish mechanisms.

Check whether the same output pattern can arise through structurally different mechanisms.

### G. Initialization, burn-in, path dependence, and continuation state

Audit:

- initial age/sex/household distributions;
- resource stocks;
- spatial placement;
- network initialization;
- warm-up/burn-in procedures;
- checkpoint continuation equivalence;
- transient vs stationary interpretation;
- path dependence under plausible alternative initial states.

No result should be called equilibrium/stationary merely because a finite run visually flattens.

### H. Stochasticity, RNG, ensembles, and Monte Carlo inference

Audit:

- seed control and provenance;
- stream independence and draw ordering;
- order-invariance where scientifically required;
- ensemble replicate sufficiency;
- Monte Carlo standard errors/confidence intervals;
- rare-event instability;
- censoring/extinction treatment;
- stopping rules;
- multiple comparisons where relevant;
- whether reproducibility is incorrectly treated as precision.

A stochastic conclusion must show that replicate count is adequate for the reported quantity, not merely that the same seeds reproduce.

### I. Sensitivity, uncertainty, convergence, and robustness

Audit:

- parameter exposure to sweeps;
- hidden fixed scientific configuration;
- local and global sensitivity;
- interaction effects;
- structural sensitivity;
- temporal horizon sensitivity;
- spatial resolution/domain sensitivity;
- initialization sensitivity;
- replicate sensitivity;
- numerical/discretization convergence where applicable.

Check whether a conclusion survives plausible changes to assumptions rather than only parameter changes within one structure.

### J. Identifiability, equifinality, calibration, and discrimination

Audit:

- whether multiple parameter/model combinations fit the same calibration outputs;
- acceptable regions vs false unique optima;
- parameter compensation;
- structural equifinality;
- held-out predictions;
- discriminating observables;
- calibration/validation leakage;
- profile and conditional sensitivity;
- whether uncertainty in inference is faithfully reported.

Never interpret optimizer convergence as scientific identifiability without separate evidence.

### K. Experiment orchestration, configuration, provenance, and reproducibility

Audit:

- complete scientific configuration capture;
- default resolution;
- sweep generation;
- resume/retry behaviour;
- crash recovery;
- duplicate/partial-run handling;
- source identity;
- package/model semantics identity;
- exact command/input recording;
- checkpoint integrity;
- artifact immutability;
- run-bundle completeness.

Ask whether two scientifically different runs can ever appear identical in metadata, or one run appear different only operationally while being scientifically identical.

### L. Observability, analysis outputs, and statistical summaries

Audit:

- whether outputs expose the variables required to diagnose mechanisms;
- nominal vs realized quantities;
- denominator definitions;
- censoring and missingness;
- time aggregation;
- per-agent vs per-run weighting;
- survival bias;
- summary statistics that can conceal multimodality;
- whether uncertainty accompanies estimates;
- whether downstream analysis can accidentally combine incompatible runs.

### M. Documentation, TRACE/ODD/ODD+D, and claim consistency

Audit:

- executable model vs scientific-model documentation;
- ODD/ODD+D consistency;
- TRACE claims vs available evidence;
- release/version statements;
- empirical vs synthetic claims;
- frozen benchmark interpretation;
- known limitations and deliberate null assumptions;
- stale statements after implementation changes.

Treat documentation drift as scientifically consequential when it changes how a result could reasonably be interpreted.

### N. Cross-system integration

After individual areas are audited, perform explicit integration passes. At minimum consider interactions among:

- demography × households;
- demography × resources;
- households × movement;
- movement × resources;
- aggregation × resources;
- initialization × demography;
- initialization × spatial placement;
- stochastic inference × censoring/extinction;
- sensitivity × hidden configuration;
- calibration × identifiability;
- checkpoint/resume × RNG;
- observability × scientific interpretation.

A subsystem can be locally correct while the coupled system is scientifically misleading.

## 5. Required evidence types

Use the strongest feasible evidence. A good audit area normally combines several of these:

- source inspection tied to authoritative semantics;
- unit/integration/property tests;
- exact invariance tests under permutations or symmetric transformations;
- analytical limiting cases;
- small hand-computable examples;
- deterministic golden cases;
- controlled structural counterfactuals;
- parameter perturbations;
- multi-seed experiments;
- Monte Carlo precision diagnostics;
- long-run/transient comparisons;
- temporal/spatial/refinement convergence checks;
- held-out prediction tests;
- alternative-initialization tests;
- comparison against documented claims.

"The code looks reasonable" is not sufficient evidence for completing an audit area.

## 6. Quantitative reporting standard

When an audit conclusion depends on numerical experiments, record enough information to reproduce and interpret it:

- exact commit SHA;
- model semantics ID;
- configuration or scenario identifiers;
- seed policy and seed list/range;
- number of replicates;
- run horizon;
- initialization details;
- relevant parameter values;
- estimator/statistic definition;
- uncertainty or Monte Carlo precision measure;
- effect size, not only significance;
- exclusions/censoring/failures;
- output/artifact paths or scripts;
- result interpretation and limitations.

If an effect is described as "large", "small", "stable", "stationary", "robust", or "negligible", support that wording numerically.

## 7. Issue creation standard

Before creating an issue:

1. Search open and closed issues and PRs for the mechanism and failure mode.
2. Reproduce or otherwise establish the problem on current `main`.
3. Identify the smallest underlying scientific defect.
4. Decide whether it is a defect, an explicit limitation, or merely a possible future extension.
5. Assign scientific severity using this protocol.

A scientific audit issue should contain:

- concise failure statement;
- scientific consequence;
- exact affected commit/semantics;
- reproduction or evidence;
- expected scientific contract;
- scope boundaries/non-goals;
- acceptance criteria;
- required tests/experiments;
- relevant documentation that must remain synchronized.

Do not prescribe a specific implementation unless the scientific contract requires it.

## 8. Repair and re-verification protocol

For P0/P1 findings:

1. preserve the original evidence;
2. create/confirm an issue;
3. implement the repair on a dedicated branch/PR;
4. verify local mechanism acceptance criteria;
5. run neighbouring subsystem tests;
6. repeat the original adversarial experiment;
7. check documentation/provenance/model-semantics implications;
8. require normal CI and applicable scientific/security gates;
9. record closure evidence in the audit ledger.

A closed issue is not automatically an audited-and-verified finding. The ledger must distinguish `fixed` from `reverified`.

## 9. Cross-chat and cross-agent handoff protocol

The repository, not conversation memory, is the authoritative audit state.

Every audit session must update the active audit status ledger before handoff. The handoff entry must include:

- exact main SHA examined;
- audit area/sub-area completed;
- files/mechanisms inspected;
- experiments/tests run;
- quantitative results and artifact/script locations;
- findings and severity;
- issue/PR numbers created or relevant;
- unresolved hypotheses;
- areas explicitly not examined;
- whether results must be repeated because main changed;
- recommended next audit area.

A new agent should be able to begin from the repository with only this instruction:

> Read `docs/research/scientific-audit-protocol.md` and the current audit status ledger. Verify live `main`, issues, PRs, and overlapping agent work, then continue the next incomplete audit area according to the protocol.

Conversation summaries may help, but they are never authoritative over repository state.

## 10. Parallel-agent rules

Parallel auditing is allowed when ownership surfaces are clearly separated.

Before working in parallel, each agent must:

- identify its audit area in the ledger;
- check open PRs and active branches for overlap;
- avoid mutating shared scientific semantics while another agent is auditing them unless coordinated;
- record dependencies on findings from other areas;
- avoid duplicate issue creation.

If two areas share central experiment/config/provenance machinery, prefer sequential work or explicit ownership boundaries.

## 11. Anti-bias checklist for every area

Before marking an area complete, explicitly ask:

- What result would surprise me?
- What assumption is being held fixed without appearing in the plotted result?
- Could scheduler/order/tie-breaking create this pattern?
- Could initialization create this pattern?
- Could finite run horizon create this pattern?
- Could finite spatial extent create this pattern?
- Could censoring/extinction create this pattern?
- Could too few stochastic replicates create this pattern?
- Could a different parameter set or model structure create the same pattern?
- Is the analysis weighting runs/agents/time correctly?
- Is the mechanism observable enough to support the claimed explanation?
- Does the documentation state a stronger claim than the evidence permits?
- Would the conclusion survive a reasonable structural counterfactual?

## 12. Completion criteria for an audit area

An area may be marked complete only when:

- authoritative implementation and documentation have been inspected;
- at least one adversarial/falsification-oriented test has been considered and, where feasible, executed;
- relevant quantitative evidence is recorded;
- interactions with neighbouring systems have been considered;
- all findings have dispositions;
- any P0/P1 findings are either explicitly open or independently reverified as fixed;
- unresolved uncertainties are documented rather than silently ignored.

"No issue found" is a valid result only if the evidence supporting that conclusion is recorded.

## 13. Completion criteria for a comprehensive audit

A comprehensive audit is complete only when:

1. every required audit surface in section 4 has a completed ledger entry;
2. cross-system integration passes have been completed;
3. all findings have a disposition;
4. no P0/P1 finding is silently deferred;
5. closed P0/P1 findings have independent re-verification evidence;
6. the final audited `main` SHA is recorded;
7. the final issue/PR state is reconciled with the ledger;
8. scientific documentation is checked for convergence after repairs;
9. a final audit synthesis states what was tested, what remains uncertain, and what empirical claims are still unsupported.

## 14. Audit convergence interpretation

Repeated audits are evidence about process maturity, not proof of correctness.

A useful convergence signal is that a fresh independent audit of the integrated system finds no new P0/P1 defects and progressively fewer/narrower P2 findings. Conversely, another large tranche of new P0/P1 findings means the framework has not yet converged scientifically, even if the issue tracker had previously been empty.

Do not redefine severity downward merely to claim convergence.

## 15. Empirical-readiness boundary

This protocol evaluates whether AnthroSim behaves coherently, transparently, reproducibly, and scientifically defensibly **under its stated assumptions**. It does not by itself validate those assumptions for any specific past society, archaeological site, or empirical reconstruction.

A clean audit therefore supports confidence in AnthroSim as a scientific instrument, but empirical application still requires question-specific evidence, calibration choices, uncertainty treatment, model comparison, and external domain scrutiny.
