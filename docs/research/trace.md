# AnthroSim TRACE evaluation dossier

**Framework:** TRACE (Grimm et al. 2014)  
**AnthroSim baseline:** v0.3.3 / current model semantics v21 / scientific audit v2 complete (immutable v0.3.2 release baseline: v19)  
**Status:** living model-evaluation dossier  
**Overall scientific status:** **NOT YET EMPIRICALLY RESEARCH-READY**

TRACE is used here as intended: not as a checklist that grants validity, but as a structure for accumulating inspectable evidence that AnthroSim was thoughtfully designed, correctly implemented, thoroughly tested, understood through analysis and used only within a justified domain of applicability.

A green software build, deterministic replay, an ODD description or a completed benchmark is not sufficient evidence of scientific validity.

## Status vocabulary

- **Established** — current repository evidence substantially satisfies the item for the stated baseline.
- **Partial / in progress** — useful evidence exists, but important work remains.
- **Not established** — the required evidence does not yet exist for empirical research use.
- **Study-specific** — cannot be satisfied globally; must be demonstrated for each application/question.

## Current TRACE summary

| TRACE element | Current status | Main reason |
|---|---|---|
| 1. Problem formulation | Partial / study-specific | Framework purposes are explicit, but real research questions require their own frozen hypotheses, observables and applicability domain. |
| 2. Model description | Established for v0.3.3 / v21 baseline | Formal ODD 2020 description plus ODD+D supplement and detailed scientific specification describe current semantics while preserving immutable v0.3.2/v19 identity. |
| 3. Data evaluation | Partial | Evidence/provenance machinery exists; question-specific evidence quality, uncertainty and evidence-role separation remain to be completed. |
| 4. Conceptual model evaluation | In progress / audit convergence | Scientific audit v2 completed a second independent/adversarial A–N pass with all 15 AV2 findings repaired and no open P0–P3 finding; structural assumptions remain study-sensitive and a fresh independent audit of v0.3.3 remains required before empirical inference. |
| 5. Implementation verification | In progress / audit convergence | Two independent audit generations have now been completed and the current v21 line has no open P0–P3 scientific finding, but repeated audits are convergence evidence rather than proof of correctness; a fresh independent audit of v0.3.3 remains required before empirical inference. |
| 6. Model output verification | Not established empirically | Existing M8/M9 benchmarks are capability/synthetic verification, not validation against empirical target patterns. |
| 7. Model analysis | Substantially strengthened / study-specific | v0.3.1 adds complete scientific-configuration exposure, replicate-precision gates, long-run diagnostics, structural/initialization sensitivity and identifiability/equifinality analysis; empirical uncertainty propagation and adequacy remain study-specific. |
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

The framework explicitly rejects scripted historical outcomes and distinguishes capability validation from archaeological interpretation.

## Remaining requirement

No generic engine can globally satisfy problem formulation for every future study. Each inferential application must freeze a study protocol before confirmatory analysis, including:

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
- [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) — explicit synthetic-versus-declared founder-state semantics, signed pre-run reproductive timing and scoped founder genealogy.
- module-specific research documents under `docs/research/` for demography, migration, spatial transformations, temporary mobility, travel/resource semantics and observability.

## Current assessment

**Established for description of the v0.3.3 / v21 baseline**, subject to normal documentation/code consistency review. The immutable `v0.3.2` release remains the historical v19 baseline rather than being retroactively redefined by living documentation.

This status means the model is described; it does not mean the described model is empirically valid.

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
- [`demography-v0.1.md`](demography-v0.1.md) explicitly avoids presenting one comparative population as a universal prehistoric proxy;
- declared founder state now has a versioned initialization identity and coarse provenance label, while [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) explicitly states that those fields do not themselves establish evidence closure.

## Required evidence roles

Every empirical datum/pattern used in an inferential study should be assigned one declared role:

1. **Model construction / structural evidence** — influenced what mechanisms or model form were chosen.
2. **Parameterisation evidence** — constrains plausible parameter values/ranges without fitting to study outputs.
3. **Calibration evidence** — explicitly used to tune parameters/model variants to observed outputs.
4. **Model-output verification evidence** — used to test whether a developed model reproduces required empirical patterns.
5. **Independent corroboration evidence** — held outside model construction/calibration and used for a later, genuinely independent test.

The same observation may not be represented as independent corroboration if it materially influenced model design or calibration.

## Remaining work

- make evidence-role separation explicit in study protocols and, where practical, machine-readable;
- assess measurement/sampling/taphonomic uncertainty rather than only source-file provenance;
- record why an ethnographic/comparative analogue is applicable to the target question;
- propagate competing estimates and uncertainty into parameter/model ensembles rather than collapsing them prematurely to one value;
- for declared founder populations, document how ages, reproductive history, co-residence, condition and direct-parent completeness were derived and how alternative plausible initial states are represented.

**Current assessment:** Partial.

---

# 4. Conceptual model evaluation

## TRACE purpose

Critically evaluate simplifying assumptions, model structure and alternative conceptual formulations against empirical knowledge and basic principles before trusting implementation output.

## Current AnthroSim evidence

The detailed scientific model and ODD/ODD+D documents explicitly identify many null assumptions and missing mechanisms, including bounded local knowledge, minimal household structure, no general learning, limited kin semantics, synthetic resource physiology, atomic permanent relocation and generic temporary-mobility motives.

The repository's adversarial scientific audits have also produced concrete issue-level evidence that conceptual/causal semantics are being challenged rather than accepted because code runs reproducibly.

Scientific audit v2 provides the latest framework-level closure evidence: it independently covered Areas A–N, demonstrated 15 AV2 findings across the audited surface, required repair/reverification of those findings, and closed with no remaining P0–P3 scientific finding on the current v21 line. That result is evidence of stronger convergence, not a claim that the conceptual model is universally valid.

### High-priority conceptual/semantic debt and repair record

At the time this dossier was introduced, known P1 issues included scientific problems around:

- demographic age/timing semantics and birth spacing;
- founder/history initialization;
- parentage/migration boundary interaction;
- resource periodization/regeneration semantics;
- shared `condition` semantics and scarcity-mortality attribution;
- newborn condition initialization;
- resource-allocation fairness and M4 utility/stay semantics;
- M4/M9 demand and mobility integration;
- spatial georeferencing/physical movement interpretation;
- evidence closure for empirical provenance.

The dated TRACE repair records preserve the state and rationale of those findings at the time they were investigated. The live GitHub issue tracker and the repository-authoritative audit-v2 ledger are authoritative for current open/closed status. Founder/history initialization issue #192 has a dedicated repair/evaluation record in [`trace-m2-founder-initialization-repair-2026-08-25.md`](trace-m2-founder-initialization-repair-2026-08-25.md): the repair removes the requirement that research-facing founder reproductive/kin state begin implicitly at zero, while leaving empirical founder derivation, uncertainty and initialization sensitivity as separate scientific tasks for relevant studies.

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

**Current assessment:** In progress / audit convergence. The v0.3.3 / v21 baseline completed scientific audit v2 with no open P0–P3 finding at release preparation, but another genuinely fresh audit of immutable `v0.3.3` is required before beginning empirical inferential work.

---

# 5. Implementation verification

## TRACE purpose

Demonstrate that the software correctly implements the conceptual model and numerical rules.

## Existing strengths

AnthroSim already has unusually strong research-software verification infrastructure, including:

- deterministic named RNG streams;
- exact experiment/run identity and source provenance;
- invariant validation and event replay;
- checkpoint/resume equivalence;
- cross-platform golden tests;
- completed-run bundle validation;
- experiment/sweep retry reconciliation;
- run/archive integrity mechanisms;
- explicit operational stop reasons;
- preserved synthetic benchmark definitions/results;
- explicit declared-founder materialization and mode-mismatch checks, signed pre-run birth-spacing tests, founder-history checkpoint/resume equivalence and first-boundary direct-parent kin verification for the #192 repair line.

The detailed verification-target list is maintained in [`../scientific-model.md`](../scientific-model.md). The #192-specific evidence/rationale is recorded in [`trace-m2-founder-initialization-repair-2026-08-25.md`](trace-m2-founder-initialization-repair-2026-08-25.md).

## Verification work still required

Implementation verification must extend beyond ordinary unit tests to scientific/metamorphic properties, including where applicable:

- arbitrary ID/order permutation invariance;
- conservation/accounting invariants;
- parameter-direction monotonicity where the declared model requires it;
- temporal-resolution convergence/invariance;
- spatial-resolution and boundary-condition invariance or explicitly documented scale dependence;
- symmetric equivalent-state tests;
- null/zero-effect cases;
- shared-state causal attribution tests;
- initialization/start-time invariance or measured transient behaviour;
- competing-risk/hazard scheduling tests;
- equivalence of semantically identical experiment encodings.

## P1 convergence rule

AnthroSim should not declare foundational scientific verification complete merely because the current known P1 backlog is empty. The corrected implementation should continue to undergo repeated independent/adversarial scientific audit passes.

A practical gate for moving from foundational verification to empirical calibration is:

> **At least two, preferably three, genuinely different deep scientific audit passes produce no new P1 scientific-behaviour defect, with all previously identified blocking P1s resolved and regression/metamorphic tests added where feasible.**

This is not proof that no defect exists; it is evidence of audit convergence.

**Current assessment:** In progress / audit convergence. Scientific audit v2 is the second independent audit generation and closed with all 15 AV2 findings repaired/reverified and no open P0–P3 scientific finding on the v0.3.3 / v21 baseline at release preparation. One further genuinely fresh audit of immutable `v0.3.3` remains required before empirical inferential work; a clean result would provide substantially stronger convergence evidence but still would not prove correctness.

---

# 6. Model output verification

## TRACE purpose

Test whether model outputs reproduce the observations/patterns the model is expected to represent, using declared quantitative/qualitative criteria.

## Current evidence

AnthroSim has completed controlled synthetic capability exercises:

- the M7 versioned synthetic resource-variability experiment;
- the M8 evidence-grounded terrain null-model benchmark;
- the M9 controlled residence-versus-intermittent-aggregation benchmark.

These demonstrate orchestration, mechanism distinction, reproducibility and inspectability. They do **not** establish empirical demographic, ecological, behavioural or archaeological validity.

## Required future output verification

A research-capable configuration should be tested against multiple predeclared patterns appropriate to the question, potentially including:

- survivorship/age-specific mortality;
- fertility/birth spacing/population growth;
- household-size/lifecycle distributions where households matter;
- condition/resource stress proxies with defensible meaning;
- mobility distance/frequency/duration;
- occupation/presence persistence;
- spatial distribution/clustering;
- other independently justified system-level patterns.

Acceptance tolerances must be declared before interpreting success. Failure to reproduce a required pattern is a model result and must not be tuned away invisibly.

**Current assessment:** Not established empirically.

---

# 7. Model analysis

## TRACE purpose

Understand how model behaviour depends on parameters, uncertainty, stochasticity, initial conditions, numerical resolution and structural choices.

## Existing capability

M7 supports deterministic ensembles and Cartesian sweeps with immutable provenance and explicit failed/incomplete run states. Derived analysis outputs retain contributing run identities. The v0.3.1 hardening line extends that foundation with complete scientific-configuration exposure to reproducible ensemble/sensitivity experiments, explicit stochastic-replicate precision/sufficiency checks, long-run drift/regime diagnostics, initialization and structural-sensitivity treatments, and fail-closed identifiability/equifinality analysis.

The #304 demographic study also provides a concrete structural-sensitivity result: no universal demographic baseline is justified because realized growth depends strongly on household lifecycle. Under the positive intrinsic schedule, fixed-founder households were approximately stationary while deterministic fission produced materially negative realized growth and much higher mate limitation. This is a model-form result, not empirical prehistoric calibration. See [`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md).

The #217 analysis gate distinguishes fit from identification. When declared evidence leaves multiple parameter combinations or structural hypotheses acceptable, AnthroSim reports the acceptable region/equifinality and preserves held-out discriminating predictions instead of selecting a false unique optimum. See [`identifiability-equifinality-v1.md`](identifiability-equifinality-v1.md).

## Required analysis programme

### Parameter sensitivity

- local one-at-a-time tests for debugging/directionality;
- global sensitivity over jointly plausible parameter ranges;
- interaction effects rather than only main effects;
- sufficient sweepability/external orchestration for every scientifically consequential parameter.

### Uncertainty analysis

- propagate evidence ranges/distributions;
- distinguish aleatory stochastic variation from epistemic parameter/model uncertainty;
- report uncertainty in outputs and conclusions, not only mean trajectories.

### Structural sensitivity

Compare plausible alternative submodels for consequential assumptions rather than treating one convenient equation as fixed truth.

### Identifiability and equifinality

Determine whether different parameter/model combinations produce observationally indistinguishable outcomes. Report non-identifiability as a scientific result and identify evidence that could discriminate alternatives.

### Temporal convergence

Test whether changing timestep/resource-period resolution while preserving the intended continuous/annual scenario changes the scientific conclusion. If it does, timestep must be treated as a model assumption rather than a numerical convenience.

### Spatial convergence and boundaries

Represent equivalent physical landscapes at different grid resolutions/extents where feasible. Test whether results depend on cell size, raster extent, edge/corner position, discretized distance or total cell count for reasons unrelated to the target hypothesis.

### Initialization / burn-in

Vary founder age/household/location/resource/start-season conditions or use justified burn-in/initialization procedures. For declared founder populations, include reproductive-history and available direct-parent/genealogy alternatives where they can affect the active mechanisms. Distinguish persistent dynamics from day-zero artifacts.

### Counterfactual robustness

Check whether a conclusion survives reasonable paired/unpaired stochastic designs and whether RNG-stream divergence complicates causal comparisons.

## Research-ready criterion

A reported conclusion should state which uncertainties/parameters/model structures it is robust to and which it is sensitive to.

**Current assessment:** Substantially strengthened at framework level, but still study-specific for strong inference. A real study must demonstrate adequate uncertainty propagation, design coverage, stochastic precision, structural sensitivity and identification for its own claim.

---

# 8. Model output corroboration

## TRACE purpose

Compare predictions or patterns against information sufficiently independent from model construction, calibration and earlier verification to provide a genuine external challenge.

## Current AnthroSim status

No real archaeological/anthropological study has yet completed this gate for the current model baseline.

## Required corroboration design

Strong options include:

- held-out archaeological patterns not used to construct/calibrate the model;
- spatial/temporal subsets reserved before model fitting;
- independent datasets or methods measuring a different consequence of the same hypothesized process;
- successful prediction of a discriminating observation later checked against external evidence;
- independent reimplementation/replication by another researcher/team;
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
2. blocking P1 conceptual/implementation issues for the relevant causal pathways are resolved;
3. foundational scientific audits have approached P1 convergence;
4. required temporal/spatial/initialization convergence tests have passed or their scale dependence is explicitly incorporated in inference;
5. all materially consequential assumptions can be sensitivity/structural-sensitivity tested;
6. evidence provenance, uncertainty and role (construction/calibration/corroboration) are explicit;
7. required empirical output patterns are reproduced within predeclared criteria or failures are transparently reported;
8. global sensitivity/uncertainty/equifinality analysis supports the reported conclusion;
9. independent corroboration is attempted using held-out evidence/predictions where possible;
10. the application receives relevant archaeological/anthropological/domain review before strong historical claims.

Passing these gates for one study does not certify all future uses of AnthroSim.

---

# TRACE evidence-maintenance procedure

For each significant scientific change:

1. update the detailed scientific specification;
2. update ODD and ODD+D where model/decision semantics changed;
3. add verification tests and link the relevant issue/PR/benchmark evidence;
4. record whether previous validation/sensitivity evidence remains applicable or is invalidated by the model revision;
5. rerun affected evaluation experiments under a new immutable model/source identity;
6. update this TRACE dossier's status only when the evidence exists.

For each real study, create a study-specific TRACE appendix/dossier that references this framework-level dossier but records its own problem formulation, evidence roles, calibration, sensitivity, validation and corroboration.

---

## Reference

Grimm, V. et al. (2014). *Towards better modelling and decision support: Documenting model development, testing, and analysis using TRACE.* Ecological Modelling 280:129–139. DOI: `10.1016/j.ecolmodel.2014.01.018`.