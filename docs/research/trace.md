# AnthroSim TRACE evaluation dossier

**Framework:** TRACE (Grimm et al. 2014)  
**AnthroSim baseline:** v0.3.6 release line / current model semantics v35 (immutable v0.3.5 release baseline: v33; immutable v0.3.4 release baseline: v25; immutable v0.3.3 release baseline: v21)
**Status:** living model-evaluation dossier  
**Overall scientific status:** **NOT YET EMPIRICALLY RESEARCH-READY**

TRACE is used here as intended: not as a checklist that grants validity, but as a structure for accumulating inspectable evidence that AnthroSim was thoughtfully designed, correctly implemented, thoroughly tested, understood through analysis and used only within a justified domain of applicability.

A green software build, deterministic replay, an ODD description, completed scientific audits or a completed benchmark are not sufficient evidence of empirical scientific validity.

## Status vocabulary

- **Established** — current repository evidence substantially satisfies the item for the stated framework baseline.
- **Partial / in progress** — useful evidence exists, but important work remains.
- **Not established** — the required evidence does not yet exist for empirical research use.
- **Study-specific** — cannot be satisfied globally; must be demonstrated for each application/question.

## Current TRACE summary

| TRACE element | Current status | Main reason |
|---|---|---|
| 1. Problem formulation | Partial / study-specific | Framework purposes are explicit, but real research questions require their own frozen hypotheses, observables and applicability domain. |
| 2. Model description | Established for the living v35 framework line | ODD 2020, ODD+D and the detailed scientific specification describe current model semantics while preserving immutable v0.3.4/v25 and v0.3.3/v21 history. |
| 3. Data evaluation | Partial | Evidence/provenance machinery exists; question-specific evidence quality, uncertainty, representativeness and evidence-role separation remain study-specific. |
| 4. Conceptual model evaluation | Strong framework audit evidence; study-specific evaluation remains required | Scientific Audit v4 completed the fourth independent A–N audit generation against immutable v0.3.4/v25, demonstrated 15 findings, and all 15 were repaired and independently re-verified/dispositioned on the living line. Structural assumptions and empirical applicability remain study-specific. |
| 5. Implementation verification | Strong framework-level convergence; never proof of correctness | Four comprehensive audit generations have now been completed. Audit v4 closed 15/15 demonstrated findings after protected production repair and independent post-merge/current-state evidence, leaving no open Audit-v4 finding on the repaired v33 line. |
| 6. Model output verification | Not established empirically | Existing M7/M8/M9 and demographic-baseline exercises are capability/synthetic/model-analysis evidence, not validation against an empirical target population or archaeological pattern set. |
| 7. Model analysis | Substantially strengthened / study-specific | Scientific configuration exposure, stochastic precision gates, long-run diagnostics, structural/initialization sensitivity, provenance integrity and identifiability/equifinality support exist; adequacy for a particular claim remains study-specific. |
| 8. Model output corroboration | Not established | No held-out independent archaeological/anthropological corroboration has yet been completed for a real inferential study. |

This status must not be upgraded merely because documentation becomes more complete. The evidence described by each section must actually be generated and reviewed.

---

# 1. Problem formulation

## TRACE purpose

Specify the exact questions the model should answer, necessary outputs, intended users, domain of applicability and acceptable extrapolation.

## Current AnthroSim evidence

Repository-level purpose and research principles are documented in:

- [`odd.md`](odd.md), especially ODD element 1;
- [`../scientific-model.md`](../scientific-model.md);
- [`../research-principles.md`](../research-principles.md);
- [`../roadmap.md`](../roadmap.md).

The framework explicitly rejects scripted historical outcomes and distinguishes capability/regression verification from archaeological interpretation.

## Remaining requirement

Each inferential application must freeze a study protocol before confirmatory analysis, including:

- precise research question;
- system/site/time domain;
- competing hypotheses and null models;
- observables/patterns used to evaluate them;
- permitted and prohibited claims;
- spatial/temporal resolution and study boundary;
- stopping/exclusion rules;
- criteria that could reject each hypothesis;
- evidence reserved for independent corroboration.

**Gate:** a real application is not TRACE-complete without a question-specific problem formulation.

---

# 2. Model description

## TRACE purpose

Provide a detailed, understandable description of what the model is, how it works and why it was designed that way. TRACE recommends ODD for individual/agent-based models.

## Current AnthroSim evidence

- [`odd.md`](odd.md) — formal seven-element ODD 2020 description covering all 11 standard design concepts.
- [`odd-d.md`](odd-d.md) — explicit ODD+D human-decision supplement.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific semantics.
- [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md) — elapsed M3-interval competing mortality and year-end M2 fertility/parentage timing contract.
- [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) — explicit synthetic-versus-declared founder-state semantics, signed pre-run reproductive timing and scoped founder genealogy.
- module-specific research documents under `docs/research/` for resources, migration, spatial transformations, temporary mobility, travel/resource semantics and observability.

## Current assessment

**Established for description of the living current model semantics v35 line**, subject to normal documentation/code consistency review. The immutable `v0.3.4` release baseline remains v25 and immutable `v0.3.3` remains v21; living documentation does not retroactively redefine those releases.

Audit-v4 AV4-015 specifically demonstrated why this distinction matters: stale ODD/ODD+D mortality wording on frozen v0.3.4/v25 was repaired, and a permanent current-document consistency guard now checks the living mortality description. The post-v0.3.4 documentation-consistency audit additionally broadens current-state checks beyond ODD/ODD+D/scientific-model.

This status means the current framework model is described; it does not mean the described model is empirically valid.

---

# 3. Data evaluation

## TRACE purpose

Critically evaluate the quality, source, uncertainty, representativeness and applicability of data used directly, through calibration or to shape model structure/patterns.

## Current AnthroSim evidence

- [`evidence-provenance.md`](evidence-provenance.md);
- versioned `EvidenceCatalog` support in core configuration;
- source identity, original variable/units, transformation, simulation units, uncertainty, applicability and competing-estimate fields;
- parameter-to-evidence links and external-input provenance;
- M8 normalized landscape/preprocessing contracts;
- [`demography-v0.1.md`](demography-v0.1.md), which explicitly avoids presenting one comparative population as a universal prehistoric proxy;
- declared founder state with versioned initialization identity and coarse provenance, while [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) explicitly states that those fields do not themselves establish evidence closure.

## Required evidence roles

Every empirical datum/pattern used in an inferential study should be assigned one declared role:

1. **Model construction / structural evidence** — influenced mechanisms or model form.
2. **Parameterisation evidence** — constrains plausible values/ranges without fitting study outputs.
3. **Calibration evidence** — explicitly used to tune parameters/model variants to observed outputs.
4. **Model-output verification evidence** — tests whether a developed model reproduces required empirical patterns.
5. **Independent corroboration evidence** — held outside construction/calibration and used for a later independent challenge.

The same observation may not be represented as independent corroboration if it materially influenced model design or calibration.

## Remaining work

- make evidence-role separation explicit in study protocols and, where practical, machine-readable;
- assess measurement, sampling and taphonomic uncertainty rather than only source-file provenance;
- record why ethnographic/comparative analogues are applicable to the target question;
- propagate competing estimates and uncertainty into parameter/model ensembles;
- for declared founders, document how age, reproductive history, co-residence, condition and direct-parent completeness were derived and how alternative plausible initial states are represented.

**Current assessment:** Partial.

---

# 4. Conceptual model evaluation

## TRACE purpose

Critically evaluate simplifying assumptions, model structure and alternative conceptual formulations against empirical knowledge and basic principles before trusting implementation output.

## Current AnthroSim evidence

The detailed scientific model and ODD/ODD+D documents identify many null assumptions and missing mechanisms, including bounded local knowledge, minimal household structure, no general learning, limited kin semantics, synthetic resource physiology, atomic permanent relocation and generic temporary-mobility motives.

The repository's adversarial scientific audits provide issue-level evidence that conceptual/causal semantics are actively challenged rather than accepted because code runs reproducibly. Scientific Audit v4 is the latest comprehensive framework audit: it restarted A–N coverage from zero against immutable v0.3.4/v25, demonstrated 13 P1 and 2 P2 findings, and the subsequent remediation programme repaired and independently re-verified/dispositioned all 15 on the living line. Authoritative semantic repairs advanced the living identity through v26–v33 where required.

Audit-v4 repairs covered arbitrary label/order coupling in fertility, background mortality, M4 migration scheduling, newborn sex, parentage, condition-mediated mortality, M9 equal-cost destinations, scarce-resource remainder assignment and M4 spatial candidate choice, plus statistical/provenance/finalization/documentation defects. The current Audit-v4 closure record is [`audit-v4/STATUS.md`](audit-v4/STATUS.md).

These results are strong convergence evidence, not a claim that the conceptual model is universally valid or empirically validated.

## Required conceptual-evaluation methods

For each scientifically consequential submodel, record:

- rationale and evidence status;
- plausible alternative formulations;
- expected directional behaviour;
- limiting/null cases;
- known omitted processes;
- claims potentially affected by the omission;
- structural-sensitivity result comparing alternatives where the choice could affect inference.

**Gate:** unresolved conceptual P1 defects block claims that depend on the affected mechanism.

**Current assessment:** Framework-level conceptual audit evidence is strong for the repaired v33 line. There are no open Audit-v4 findings, but empirical inferential work still requires question-specific evidence, calibration/validation, uncertainty, applicability and corroboration gates.

---

# 5. Implementation verification

## TRACE purpose

Demonstrate that the software correctly implements the conceptual model and numerical rules.

## Existing strengths

AnthroSim includes:

- deterministic named RNG streams and scientific stochastic-coupling identities;
- exact experiment/run identity and source provenance;
- invariant validation and event replay;
- checkpoint/resume equivalence and source lineage;
- cross-platform golden tests;
- completed-run bundle validation;
- experiment/sweep retry reconciliation;
- run/archive integrity mechanisms;
- explicit operational stop reasons;
- preserved synthetic benchmark definitions/results;
- adversarial/metamorphic tests for arbitrary identity/order effects, provenance binding, numerical precision and analysis integrity.

The detailed verification-target list is maintained in [`../scientific-model.md`](../scientific-model.md).

## Verification methods that remain relevant

Implementation verification should continue to include where applicable:

- arbitrary ID/order permutation invariance;
- conservation/accounting invariants;
- parameter-direction monotonicity where required by the declared model;
- temporal-resolution convergence/invariance;
- spatial-resolution and boundary-condition invariance or explicit scale dependence;
- symmetric equivalent-state tests;
- null/zero-effect cases;
- shared-state causal attribution tests;
- initialization/start-time invariance or measured transient behaviour;
- competing-risk/hazard scheduling tests;
- equivalence of semantically identical experiment encodings.

## Audit convergence

Four comprehensive independent/adversarial audit generations have now been completed. Audit v4 demonstrated that an additional fresh audit could still find important defects after earlier convergence work, so audit count must not be treated as proof that no defect remains. What Audit-v4 closure establishes is narrower: every demonstrated Audit-v4 finding has a recorded production repair and required independent re-verification/disposition, and no Audit-v4 issue remains open.

**Current assessment:** Strong framework-level implementation convergence on the repaired current model semantics v35 line; never proof of correctness. Empirical inference remains gated by study-specific validation and corroboration.

---

# 6. Model output verification

## TRACE purpose

Test whether model outputs reproduce observations/patterns the model is expected to represent, using declared criteria.

## Current evidence

AnthroSim has controlled framework/capability exercises including:

- the M7 versioned synthetic resource-variability experiment;
- the M8 evidence-grounded terrain null-model benchmark;
- the M9 controlled residence-versus-intermittent-aggregation benchmark;
- the general demographic baseline/sensitivity study.

These demonstrate orchestration, mechanism distinction, reproducibility, regression sensitivity and inspectability. They do **not** establish empirical demographic, ecological, behavioural or archaeological validity.

## Required future output verification

A study-specific configuration should be tested against multiple predeclared patterns appropriate to the question, potentially including survivorship, fertility/birth spacing/growth, household lifecycle, resource/condition proxies, mobility, presence persistence, spatial patterning and other independently justified outputs.

Acceptance tolerances must be declared before interpreting success. Failure to reproduce a required pattern is a model result and must not be tuned away invisibly.

**Current assessment:** Not established empirically.

---

# 7. Model analysis

## TRACE purpose

Understand dependence on parameters, uncertainty, stochasticity, initial conditions, numerical resolution and structural choices.

## Existing capability

M7 supports deterministic ensembles and Cartesian sweeps with immutable provenance and explicit failed/incomplete run states. Subsequent hardening adds complete scientific-configuration exposure, stochastic-replicate precision/sufficiency checks, long-run drift/regime diagnostics, initialization and structural-sensitivity treatments, and fail-closed identifiability/equifinality analysis.

The general demographic study provides a concrete structural-sensitivity result: no universal demographic baseline is justified because realized growth depends strongly on household lifecycle and mate opportunity. This is a model-form result, not empirical prehistoric calibration. See [`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md).

Identifiability/equifinality analysis is intended to report compatible regions and discriminating predictions rather than select a false unique optimum. Audit-v4 further hardened this path against fabricated downstream parameter certification and stale/fabricated provenance bindings.

## Required analysis programme for real inference

- local and global parameter sensitivity;
- propagated evidence uncertainty;
- separation of aleatory and epistemic uncertainty;
- structural sensitivity for consequential assumptions;
- identifiability/equifinality analysis;
- temporal convergence;
- spatial resolution/extent/boundary sensitivity;
- initialization/burn-in sensitivity;
- counterfactual/RNG-design robustness.

**Research-ready criterion:** a reported conclusion should state which uncertainties, parameters and model structures it is robust to and which it is sensitive to.

**Current assessment:** Substantially strengthened at framework level, but still study-specific for strong inference.

---

# 8. Model output corroboration

## TRACE purpose

Compare predictions/patterns against information sufficiently independent from model construction, calibration and earlier verification to provide a genuine external challenge.

## Current AnthroSim status

No real archaeological/anthropological study has yet completed this gate for the current model baseline.

## Required corroboration design

Strong options include:

- held-out archaeological patterns not used to construct/calibrate the model;
- spatial/temporal subsets reserved before fitting;
- independent datasets or methods measuring another consequence of the same hypothesized process;
- successful prediction of a discriminating observation later checked externally;
- independent reimplementation/replication;
- comparison against established models where relevant;
- domain-specialist review of mechanisms, evidence transformations and permissible interpretations.

For archaeology, corroboration must respect the observation process. Simulated activity/presence is not necessarily the quantity archaeologists observe after deposition, preservation, disturbance, survey and recovery. Where this matters, an explicit observation/taphonomic/sampling model or justified comparison procedure is required.

**Current assessment:** Not established.

---

# TRACE research-readiness gates

AnthroSim should distinguish two claims:

## Research-grade software infrastructure

A statement about reproducibility, provenance, deterministic execution, inspectability, integrity and tooling.

## Research-ready scientific configuration for a question

A much stronger and question-specific statement requiring demonstrated fitness for purpose.

A configuration should not be called **research-ready for inferential use** until, at minimum:

1. its ODD/ODD+D description matches the implemented semantics;
2. blocking conceptual/implementation issues for the relevant causal pathways are resolved;
3. foundational scientific auditing and relevant verification evidence support the implemented pathways;
4. required temporal/spatial/initialization convergence tests pass or scale dependence is incorporated explicitly;
5. materially consequential assumptions can be sensitivity/structural-sensitivity tested;
6. evidence provenance, uncertainty and role are explicit;
7. required empirical output patterns are tested against predeclared criteria;
8. global sensitivity/uncertainty/equifinality analysis supports the reported conclusion;
9. independent corroboration is attempted using held-out evidence/predictions where possible;
10. the application receives relevant archaeological/anthropological/domain review before strong historical claims.

Passing these gates for one study does not certify all future uses of AnthroSim.

---

# TRACE evidence-maintenance procedure

For each significant scientific change:

1. update the detailed scientific specification;
2. update ODD and ODD+D where model/decision semantics changed;
3. update other living current-facing architecture/status documentation affected by the change;
4. add verification tests and link the relevant issue/PR/benchmark evidence;
5. record whether previous validation/sensitivity evidence remains applicable or is invalidated;
6. rerun affected evaluation experiments under a new immutable model/source identity where required;
7. update this TRACE dossier's status only when the evidence exists.

For each real study, create a study-specific TRACE appendix/dossier that references this framework-level dossier but records its own problem formulation, evidence roles, calibration, sensitivity, validation and corroboration.

The current living-document consistency audit is recorded in [`post-v0.3.4-documentation-consistency-audit.md`](post-v0.3.4-documentation-consistency-audit.md). Historical release/audit/evidence records remain frozen to the identities they actually evaluated rather than being mechanically rewritten to v33.

---

## Reference

Grimm, V. et al. (2014). *Towards better modelling and decision support: Documenting model development, testing, and analysis using TRACE.* Ecological Modelling 280:129–139. DOI: `10.1016/j.ecolmodel.2014.01.018`.
