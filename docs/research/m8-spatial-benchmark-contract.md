# M8 spatial research and benchmark contract

## Status

This document defines the M8.0 scientific contract for evidence-grounded spatial experiments. It is intentionally case-study-neutral. It does not identify a particular locality, archaeological site, unpublished dataset or private research question.

M8.0 changes no executable model semantics. It defines the requirements that M8.1-M8.6 must satisfy before evidence-grounded spatial results can be interpreted beyond the existing synthetic-validation baseline.

## Purpose

M8 introduces evidence-grounded spatial environments so AnthroSim can ask a narrower and more useful class of questions:

> What spatial patterns do the model's declared demographic, resource and mobility mechanisms generate when constrained by a reproducible real-world-derived environment, and which patterns do they fail to generate?

The first M8 benchmark is a **null-model benchmark**. It is not a reconstruction attempt and has no acceptance requirement that a known historical pattern must emerge.

Its role is to establish what the mechanisms already represented by AnthroSim can and cannot explain before additional social or cultural mechanisms are introduced.

## Scientific question class

An M8 spatial benchmark must be expressible without embedding a desired historical outcome in the model. Suitable question classes include:

- whether declared environmental heterogeneity changes population persistence or fragmentation;
- whether terrain-related movement costs alter household relocation patterns;
- whether water/resource opportunity changes the distribution or persistence of occupied cells;
- whether spatial constraints change the frequency, distance or directionality of migration;
- whether combinations of environmental assumptions generate stable, unstable or multimodal spatial outcomes across ensembles;
- whether a simple environmental-demographic-mobility null model fails systematically in ways that motivate a specific later mechanism.

M8 does not require a public benchmark to name the real case study that motivated these question classes.

## Null-model rule

The benchmark must not encode a known settlement, route, boundary, destination, archaeological feature or desired concentration as a target in authoritative simulation rules.

A spatial input may contain environmental information supported by declared evidence. A downstream analysis may compare simulated outcomes with external observations. Those are different operations.

The core causal direction is:

```text
source evidence
    -> documented spatial transformations
    -> normalized AnthroSim landscape
    -> authoritative model mechanisms
    -> simulated state/events
    -> derived spatial observables
    -> downstream comparison/interpretation
```

The arrow must not reverse. A known historical outcome must not be used to alter the same simulation in order to make that outcome appear, except in an explicitly labelled calibration experiment whose validation data remain separate.

## Required spatial/environmental inputs

M8.0 does not prescribe the final M8.1 serialization format, but an evidence-grounded benchmark must be able to identify at minimum:

- normalized landscape schema version;
- spatial extent and dimensions;
- cell resolution or equivalent cell geometry;
- coordinate reference metadata sufficient to interpret location;
- explicit nodata/missing-data semantics;
- each model-facing spatial layer and its units/value domain;
- source evidence records for externally derived layers;
- transformations used to convert source observations into model-facing values;
- uncertainty or alternative reconstructions where scientifically relevant;
- a stable content identity for the normalized input used by the run.

Synthetic spatial fixtures remain valid for verification and directional tests, but must remain distinguishable from evidence-grounded inputs.

## Initial model-facing layer classes

M8 should begin with the minimum layer classes needed to exercise mechanisms already present in the engine rather than introducing unrelated human behaviours.

Expected early classes are:

1. **Terrain / traversal opportunity** — a normalized quantity from which an explicit movement-cost transformation can be derived.
2. **Water accessibility** — a declared spatial quantity that can replace or constrain the existing synthetic water/security proxy where justified.
3. **Resource opportunity** — a declared spatial quantity from which the dynamic resource system's baseline productivity or capacity can be initialized or scaled.

The source value and the model-facing value are not automatically the same thing. Every scientific transformation between them must be explicit, inspectable and sensitivity-testable where uncertainty matters.

## Model assumptions and competing alternatives

Every M8 benchmark must state which assumptions are held constant and which are varied.

At minimum, the benchmark definition should identify:

- the demographic preset/model identity;
- the resource preset/model identity;
- the migration preset/model identity;
- the spatial transformation rules used by those mechanisms;
- which assumptions remain synthetic or unresolved;
- which parameters are evidence-grounded, evidence-informed or empirical-derived;
- which plausible alternative transformations or parameter ranges are included in sensitivity analysis.

The presence of a real spatial dataset must never cause synthetic demographic or behavioural assumptions to be described as empirical.

## Required observables

M8.5/M8.6 must make spatial outcomes measurable through machine-readable derived outputs rather than relying on visual inspection of a map.

The first benchmark should support, where applicable:

- population persistence and terminal population;
- occupied-cell count and occupancy fraction;
- spatial distribution of living population;
- persistence/duration of occupancy by cell or spatial region;
- concentration/dispersion summaries that are explicitly defined;
- migration count and distance distribution;
- origin/destination or movement-flow summaries at an appropriate spatial aggregation;
- resource stress/scarcity summaries in spatial context;
- the run/seed/configuration identities contributing to every aggregate result.

Any new concentration, clustering or persistence metric must be documented as a derived metric, not authoritative state.

## Ensemble and sensitivity requirement

A single visually interesting run is not an M8 scientific result.

The first evidence-grounded benchmark must execute through the existing M7 ensemble/sweep machinery and should vary enough uncertainty to distinguish a robust effect from a fragile artefact.

Sensitivity dimensions should include, as justified by the benchmark:

- random seed;
- alternative evidence-grounded spatial reconstructions or transformations;
- resource scaling/translation assumptions;
- terrain-to-movement-cost assumptions;
- water-access transformations;
- existing demographic/resource/migration parameters whose uncertainty could dominate the spatial result.

The benchmark definition must state which uncertainty dimensions were omitted and why.

## Predeclared interpretation outcomes

Before the first benchmark is run, its documentation should recognise at least four legitimate outcome classes:

### 1. Robust spatial structure

A spatial pattern appears consistently across seeds and remains materially present across plausible parameter/input alternatives.

This supports a statement only about the declared model under the tested evidence-grounded constraints. It does not by itself establish that the same mechanism caused a real archaeological pattern.

### 2. Fragile spatial structure

A pattern appears only under narrow parameter choices, transformations or particular reconstructions.

The sensitivity is part of the result and should prevent strong interpretation.

### 3. No distinctive spatial structure

The null model does not generate a stable or discriminating pattern under the tested assumptions.

This is informative. It can show that environmental/demographic/mobility mechanisms alone are insufficient for the intended comparison, or that the available spatial constraints are not discriminating.

### 4. Failure/extinction/degenerate behaviour

Many or all runs terminate, collapse, saturate or otherwise enter a regime that prevents the intended comparison.

This remains a scientific result if the behaviour is reproducible and traceable. It may identify an implausible transformation, incompatible parameter combination or a missing mechanism. It must not be hidden by tuning toward a desired outcome.

## Comparison with external observations

M8.6 may produce outputs suitable for later comparison with archaeological or anthropological observations, but the authoritative M8 simulation should not require those observations as target labels.

Where external observations are used downstream, the comparison must distinguish:

1. model inputs used to constrain the simulation;
2. calibration information, if any;
3. observations used for evaluation;
4. observations withheld from calibration where possible;
5. unavailable or uncertain evidence.

Using the same observation to construct the model and then claiming its reproduction as independent validation is not permitted without explicit qualification.

## Evidence and provenance requirements

The existing `EvidenceCatalog` is the required provenance bridge for evidence-grounded parameters and external inputs.

M8 implementations must preserve, where applicable:

- source citation and persistent identifier;
- dataset version and licence;
- spatial/temporal coverage;
- original variable and units;
- transformation into simulation units;
- uncertainty representation;
- applicability statement;
- competing estimates;
- external input format/spatial reference/content identity.

Traceability does not establish correctness. A source can be fully traceable and still be unsuitable for a particular model assumption.

## Reproducibility requirements

An M8 evidence-grounded run must be reproducible from preserved inputs and configuration under AnthroSim's declared determinism boundary.

M8.1-M8.3 therefore must ensure that:

- the normalized landscape contributes to exact experiment identity;
- loading the same landscape does not depend on filesystem iteration order or GIS library behaviour at simulation runtime;
- checkpoint/resume refers to the same immutable landscape identity;
- missing, modified or incompatible spatial inputs fail explicitly rather than silently falling back to synthetic geography;
- synthetic M1 experiments remain reproducible and supported.

Raw GIS source files do not need to be simulation-runtime dependencies if a normalized, provenance-tracked AnthroSim landscape bundle is preserved.

## Validation ladder

M8 should progress through the following validation ladder.

### Level A — Contract/serialization verification

Synthetic fixtures verify schema, nodata, units, extent, digest and deterministic round-trip behaviour.

### Level B — Directional mechanism tests

Constructed landscapes with known gradients/barriers verify expected directional effects, for example that an explicitly higher traversal cost cannot become cheaper because of loading or indexing errors.

These tests validate implementation/model wiring, not archaeology.

### Level C — Cross-run invariants

Evidence-grounded runs participate in checkpoint/resume, artifact reconciliation, generated invariants and cross-platform determinism where applicable.

### Level D — Evidence-grounded null-model benchmark

A versioned ensemble/sweep uses a real-world-derived normalized landscape and declared uncertainty while retaining explicit interpretation limits.

### Level E — Later case-study validation

Comparison with archaeological observations, calibration/validation separation, archaeological observation models and domain review belong to later scientific work unless a narrowly defined M8 comparison can be justified without overclaiming.

M8 completion requires Level D, not Level E.

## Privacy and publication boundary

The public AnthroSim core should remain reusable and case-study-neutral.

A private or unpublished research project may use the M8 contracts without requiring its locality, source files, archaeological targets or hypotheses to be committed to the public repository.

Public core documentation and tests should use generic descriptions, synthetic fixtures, or deliberately publishable open examples. A specific case-study package may be published separately when there is a scientific reason and appropriate data/licence permissions.

This boundary is about publication scope, not reproducibility. Any actual scientific run still needs enough preserved private/public provenance within its research archive to be reproducible by authorized collaborators.

## M8.0 acceptance criteria

M8.0 is complete when:

- [x] the first M8 question class is defined as an evidence-grounded spatial null-model benchmark rather than a reconstruction target;
- [x] required spatial input/provenance categories are declared;
- [x] required machine-readable spatial observables are declared;
- [x] ensemble/sensitivity requirements and legitimate negative outcomes are declared before implementation;
- [x] the calibration/evaluation leakage boundary is explicit;
- [x] the validation ladder distinguishes implementation verification from empirical/case-study validation;
- [x] public core documentation remains independent of a named locality or unpublished research question;
- [x] M8.1-M8.6 can be scoped as implementation issues from this contract.

## Relationship to M8.1-M8.6

- **M8.1** defines the versioned normalized landscape bundle required by this contract.
- **M8.2** defines reproducible external preprocessing into that bundle.
- **M8.3** loads the bundle deterministically into authoritative experiment state.
- **M8.4** connects declared spatial layers to existing resource/mobility mechanisms through explicit transformations.
- **M8.5** records and exposes the spatial observables required here without making visualisation authoritative.
- **M8.6** executes the first evidence-grounded null-model ensemble/sensitivity benchmark and documents its interpretation limits.

The results of M8.6, rather than a preselected feature list, should determine which scientific limitation deserves the next milestone.
