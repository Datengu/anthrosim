# TRACE-structured scientific audit — 2026-08-25

**Audited baseline:** `main` at/after v0.3.0 ODD/ODD+D/TRACE adoption  
**Audit type:** repository-level TRACE scientific/readiness audit  
**Execution status:** static source/document/issue inspection; this audit did not execute new numerical ensembles locally  
**Overall result:** **NOT YET EMPIRICALLY RESEARCH-READY**  
**New findings:** 3 P1 research blockers, 4 P2 research/readiness gates

This audit applies the eight TRACE elements to the current AnthroSim repository rather than treating scientific review as an undirected search for code defects. It deliberately distinguishes:

- implementation defects;
- ambiguous or scale-dependent model semantics;
- documented simplifying assumptions that require structural sensitivity rather than immediate replacement;
- research-workflow limitations;
- empirical validation/corroboration that cannot be completed until a question-specific study exists.

The audit also deduplicated findings against the existing scientific-audit backlog. Existing issues such as #179, #180, #181, #182, #183, #185, #186, #188, #189, #191, #192, #193, #196, #200 and #201 remain active blockers where applicable and were not re-filed under TRACE terminology.

---

## Executive assessment

AnthroSim's **research engineering** remains considerably stronger than its current empirical/model-validation status. The repository has unusually explicit deterministic provenance, model semantics, run identity, checkpoints, event history, evidence binding, ODD/ODD+D description and a now-formal TRACE dossier. Those strengths survived this audit.

The principal new concern is **representation dependence**: some variables that look like numerical or preprocessing choices currently alter the causal system itself. In particular, raster resolution and resource-period count can change resource capacity, physical movement scales, physiological response frequency, mortality opportunities and migration opportunities. Those effects can produce highly reproducible but scientifically representation-dependent outcomes.

The second major concern is **analysis reach**. M7 can reproduce a narrow parameter sweep very well, but most scientifically consequential `ExperimentConfig` assumptions are frozen behind synthetic defaults in the ensemble/sweep interface. That prevents TRACE-compliant global sensitivity from being a first-class reproducible workflow.

No evidence was found in this audit that invalidates the repository's existing claims of deterministic/software capability. The problem is narrower: the current scientific baseline has not yet earned the right to support scale-independent, empirically calibrated archaeological inference.

---

# TRACE 1 — Problem formulation

**Status:** 🟡 Partial / study-specific

### What holds up

The repository-level purpose is well constrained. ODD/ODD+D and `scientific-model.md` correctly describe AnthroSim as an exploratory causal/research framework rather than a reconstruction of a particular past society. The standards explicitly prohibit treating capability benchmarks as archaeological validation.

The study-specific standard now requires research question, competing hypotheses/null models, permitted interpretations, observables, evidence roles, uncertainty, ensemble/stopping rules, sensitivity plan, analysis method and discriminating predictions to be frozen before confirmatory inference.

### Remaining gap

No real archaeological study protocol exists yet. This is expected, not an implementation defect. A real case-study experiment must not begin as confirmatory research until that protocol is frozen.

### Audit judgement

No new issue required. The repository-level formulation is adequate; TRACE 1 remains incomplete by design until a question-specific study is declared.

---

# TRACE 2 — Model description

**Status:** 🟢 Strong for the current baseline

### What holds up

The formal ODD 2020 description, ODD+D supplement and detailed `scientific-model.md` collectively expose:

- entities/state;
- spatial and temporal scales;
- scheduling;
- initialization;
- stochasticity;
- interaction;
- adaptation/decision rules;
- omitted learning/social institutions;
- M1-M9 submodels;
- synthetic versus evidence-grounded status.

ODD+D is particularly valuable because it prevents M4's utility rule from being mistaken for a validated psychological model and prevents configured M9 attendance from being mistaken for an empirical theory of aggregation motive.

### Remaining risk

Any fixes to the P1 backlog that change authoritative causal meaning must update ODD/ODD+D/scientific-model and the model-semantics identity where scientifically incompatible.

### Audit judgement

No new TRACE-2 blocker found. Documentation quality is currently one of the strongest parts of the project.

---

# TRACE 3 — Data evaluation

**Status:** 🟡 Good foundation; research closure incomplete

### What holds up

The evidence system records source identity, original/simulation units, declared transformations, uncertainty text, applicability, competing estimates, parameter links and external-input links. Evidence-derived spatial layers remain conceptually separate from model-facing transformations and outputs.

### Existing blocker

#181 already captures the most serious data-provenance problem: an empirical/evidence-informed provenance label does not yet guarantee complete machine-verifiable evidence closure for the scientifically substantive configuration.

### New finding

**#206 — P2: make calibration, validation, and held-out corroboration evidence roles explicit.**

A valid source can currently be linked without machine-readable information about how it was used. TRACE requires a firewall between model construction, parameterisation, calibration, output verification and independent corroboration. Without that role record, a future study could accidentally use one pattern to tune the model and later call reproduction of that same pattern independent validation.

### Audit judgement

Data provenance is structurally promising but not yet sufficient for empirical validation claims.

---

# TRACE 4 — Conceptual model evaluation

**Status:** 🟠 In progress / scientifically consequential assumptions remain unresolved

This element asks whether the model structure itself is defensible for the intended use, not merely whether code matches documentation.

### New P1 — temporal semantics

**#204 — P1: separate temporal resolution from per-boundary physiology, mortality and migration opportunity.**

`resources.periodsPerYear` currently changes more than resource integration. Every resource boundary also triggers condition response, one scarcity-mortality opportunity and one M4 permanent-migration opportunity.

Consequently, holding nominal coefficients fixed while changing `periodsPerYear` changes annual causal rates. For example, a 20% same-state mortality probability applied once annually is 20% annual exposure; applied four times is approximately 59%; applied twelve times is approximately 93%. Default condition recovery of 25 per boundary similarly implies 25, 100 or 300 recovery points/year at 1, 4 or 12 periods before saturation. M4 receives the same multiplication of decision opportunities.

#180/#189 remain necessary but do not solve this response-frequency problem.

### New P2 — household structural sensitivity

**#207 — P2: quantify structural sensitivity to fixed founder-defined household lifecycles.**

Founder households are created once, births inherit the female parent's household, and the household set has no production fission/dissolution/formation process. Because households are simultaneously the M3 resource-sharing unit, M4 relocation unit and M9 participation unit, a long run can turn an initial synthetic household into a large multi-generational descendant group while retaining atomic pooling/movement semantics.

This is a documented simplification, not an accidental bug. TRACE nevertheless requires testing whether target conclusions survive at least one defensible alternative household-lifecycle abstraction before household-mediated long-run inference is treated as robust.

### New P2 — competing risks

**#208 — P2: define competing-risk semantics for coincident M3 and M2 mortality hazards.**

On the annual boundary M3 condition/scarcity mortality executes before M2 annual mortality. A person removed by M3 never receives the M2 draw that day. Total survival can remain mathematically coherent under independent sequential hazards, but cause-specific attribution depends on ordering. Since AnthroSim reports scarcity deaths as a scientific output, the cause model must be explicit rather than an incidental function-call priority.

### Existing conceptual blockers

The prior audit backlog still includes major conceptual/causal defects such as #179, #180, #186, #188, #189, #191, #192, #193, #196, #200 and #201.

### Audit judgement

TRACE 4 is not near completion yet. The recurring risk is not arbitrary realism; it is hidden coupling between otherwise simple submodels.

---

# TRACE 5 — Implementation verification

**Status:** 🔴 Blocking despite strong software verification

### What holds up

The repository has strong deterministic/software verification infrastructure:

- exact population/resource accounting;
- stable IDs and genealogy checks;
- authoritative event history;
- checkpoint/resume validation;
- state digests;
- cross-platform golden runs;
- immutable experiment identity;
- retry/reconciliation rules;
- spatial and M9 lifecycle invariants.

These are real strengths. They answer whether the executable state is reproducible and internally consistent.

### Why TRACE 5 is still red

Scientific verification also requires metamorphic/invariance tests: equivalent scientific scenarios should not diverge because of record order, arbitrary IDs, timestep partition, raster resolution or other representation choices unless those choices are explicitly part of the model.

The unresolved P1 backlog already contains examples of such failures. This audit found one further major spatial example.

### New P1 — spatial representation dependence

**#203 — P1: make evidence-grounded behavior invariant to raster resolution or explicitly scale-normalized.**

M8 records physical grid cell sizes but `SpatialLandscapeSimulation` transforms landscape layers into a core `World` whose behavioral resource/migration/travel mechanisms do not receive physical cell area/length.

For the same uniform 1 km² physical landscape:

- 100 m cells create 100 resource-bearing cells;
- 50 m cells create 400 resource-bearing cells.

If both cells receive the same transformed `baseProductivity`, the finer raster can create roughly four times as many independently stocked/productive cells. Meanwhile M4's radius and travel condition cost are in cell steps, and M9 duration is based on accumulated edge costs with no physical edge length. Finer resolution can therefore multiply resource capacity while shrinking M4's physical horizon and increasing M4/M9 physical travel cost/duration.

This is distinct from #185, which addresses ambiguous georeferencing/orientation and rectangular-cell movement semantics. #203 applies even to perfectly aligned square cells.

### P1 convergence

This audit discovered new P1s, therefore it **does not count as a no-new-P1 convergence pass**. The TRACE convergence counter remains effectively at zero until the current P1 cluster is fixed and genuinely different audits stop discovering new high-impact causal errors.

### Audit judgement

Software reproducibility is strong; scientific implementation verification is not yet converged.

---

# TRACE 6 — Model output verification

**Status:** 🔴 Empirical output verification not established

### What holds up

M8 and M9 preserved benchmarks are useful capability/regression exercises and are correctly labelled synthetic/null or methodological where appropriate. The project does not currently overclaim those exercises as validation of prehistory.

### Remaining gap

There is not yet a validated empirical configuration with predeclared tolerances showing that the model reproduces the real-world demographic/resource/mobility patterns it claims to represent.

### New P2 — archaeological observation model

**#209 — P2: require an explicit archaeological observation model for empirical pattern comparison.**

Residence, physical presence, person-days and simulated activity are model variables; archaeological evidence is a filtered product of behavior, deposition, preservation, disturbance, sampling, recovery and recording. A research study must therefore define how a simulated variable maps to an archaeological observable rather than equating high simulated activity directly with high material density or treating non-observation as behavioral absence.

### Audit judgement

TRACE 6 cannot turn green through more code auditing alone. It requires question-specific empirical validation after the foundational P1 blockers are resolved.

---

# TRACE 7 — Model analysis

**Status:** 🔴 Blocking

This is the largest remaining research-capacity gap after implementation verification.

### New P1 — insufficient sensitivity surface

**#205 — P1: expose the full scientific configuration to reproducible ensemble and sensitivity experiments.**

M7's immutable ensemble/sweep workflow is strong, but the first-class sweep dimensions cover only a narrow subset: founder population, household size, resource productivity/seasonality scale, annual food need, migration enabled state and migration radius (plus shared spatial/M9 definitions).

The normal experiment builder reconstructs most of the scientific model from frozen synthetic defaults. Consequently global sensitivity cannot first-class vary mortality/fertility schedules, birth spacing, resource-period frequency, regeneration, stock capacity, condition response/mortality, most M4 thresholds/weights/risk terms and other consequential assumptions without source edits or separate custom orchestration.

A research-ready framework needs a versioned typed route for varying the full study-relevant `ExperimentConfig` while retaining the same immutable provenance/retry semantics.

### Other analysis requirements

Even after #205, TRACE 7 still requires actual analyses, not merely capability:

- global sensitivity and parameter interactions;
- evidence-derived uncertainty propagation;
- structural sensitivity;
- equifinality/identifiability;
- temporal convergence (#204);
- spatial convergence/boundary analysis (#185/#203);
- initialization/burn-in sensitivity (#192 and follow-up experiments);
- counterfactual robustness;
- operational-censoring treatment (#183).

The current named sequential RNG streams provide deterministic reproducibility, but this audit does not treat paired-seed treatment arms as automatically maintaining common-random-number pairing after the trajectories consume different numbers of stochastic draws. That is an analysis-design consideration, not a newly filed implementation defect at this stage.

### Audit judgement

AnthroSim can run reproducible ensembles, but it cannot yet claim TRACE-compliant global model analysis over its full scientific uncertainty space.

---

# TRACE 8 — Model output corroboration

**Status:** 🔴 Not established

There is no genuinely independent held-out archaeological/anthropological corroboration exercise yet, and no external domain review establishing a configuration's fitness for a historical claim.

That is appropriate at the current stage. Independent corroboration should happen only after:

- known scientific P1s are resolved;
- audit convergence improves;
- calibration/validation evidence roles are frozen (#206);
- the relevant observation model exists (#209);
- sensitivity/uncertainty and equifinality analyses are complete;
- a study protocol declares what would count as support, rejection or ambiguity.

### Audit judgement

No implementation fix can mark TRACE 8 complete. It is an empirical/external-review stage.

---

# New issues created by this audit

| Issue | Severity | TRACE elements | Finding |
|---|---|---|---|
| #203 | P1 | 4, 5, 7 | Raster resolution changes resources and physical mobility semantics because geometry is dropped from behavioral execution. |
| #204 | P1 | 4, 5, 7 | `periodsPerYear` multiplies physiological, mortality and M4 decision opportunities, coupling temporal partition to causal rates. |
| #205 | P1 | 7 | M7 cannot first-class sweep most scientifically consequential configuration parameters. |
| #206 | P2 | 3, 6, 8 | Evidence provenance lacks machine-readable construction/calibration/verification/corroboration use roles. |
| #207 | P2 | 4, 7 | Fixed founder-defined households require structural-sensitivity testing for long-run household-mediated inference. |
| #208 | P2 | 4, 5, 7 | Coincident M3/M2 mortality cause attribution depends on sequential competing-risk semantics. |
| #209 | P2 | 6, 8 | Archaeological comparisons need an explicit observation-model/taphonomy/sampling boundary. |

---

# Findings deliberately not filed as new defects

- Existing P1/P2 issues were not duplicated merely because TRACE gives them another category.
- Lack of a real case-study protocol is expected before a real study; the standards already require one.
- Lack of empirical output verification/corroboration is a status, not a software defect.
- Absence of rich culture, politics, marriage, learning, storage, warfare or institutions is not automatically a bug; such mechanisms become necessary only when the research question or structural-sensitivity result requires them.
- Sequential named RNG streams guarantee reproducibility, not permanent common-random-number identity between diverged counterfactual arms. This should be handled explicitly in study analysis if paired stochastic control is claimed.
- Synthetic M8/M9 benchmark success remains capability evidence, not historical validation; the repository already states this correctly.

---

# Recommended next order

The audit suggests fixing by causal cluster rather than by issue number alone:

1. **Temporal/demographic/condition semantics:** #179, #180, #189, #191, #192, #200, #201, #204, #208.
2. **Spatial physical semantics:** #185, #187, #196, #203 plus relevant M9 travel/focal-region interactions.
3. **M4 decision semantics:** #182, #186, #188, #195 and related resource-demand integration.
4. **Research experiment surface:** #183, #184, #205.
5. **Evidence/readiness governance:** #181, #206.
6. **Structural sensitivity:** #207 and other question-specific model alternatives.
7. **Empirical archaeology bridge:** #209, followed by calibration/output verification and finally held-out corroboration.

After the blocking P1 clusters are repaired, rerun the same audit families and then perform genuinely different adversarial passes. TRACE foundational convergence should not be claimed until at least two, preferably three, independent deep passes find no new P1 scientific error.

---

## Conclusion

This TRACE audit **did find additional P1 issues**, so the earlier suspicion that AnthroSim had not yet reached P1 convergence was correct. The new findings are not evidence that the entire architecture is unsound. They cluster around scientific scale semantics and the research-analysis interface, while deterministic execution/provenance and the standards documentation remain strong.

The current priority is therefore not to add more archaeological complexity. It is to make time, space, hazard attribution and sensitivity design scientifically explicit enough that an eventual real-site experiment cannot obtain a convincing result merely because of timestep, raster resolution, frozen defaults or circular evidence use.
