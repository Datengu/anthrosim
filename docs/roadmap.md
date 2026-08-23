# AnthroSim roadmap

## Purpose

AnthroSim's roadmap is driven by research capability rather than feature count. The project should add new mechanisms or infrastructure when they are needed to ask a clearer question, distinguish competing explanations, expose uncertainty, or make an experiment more reproducible and inspectable.

The long-term direction remains the one described in `docs/vision.md`: build enough defensible lower-level rules that history-like structure can emerge without scripting historical outcomes.

This roadmap is intentionally case-study-neutral. Public core documentation should describe reusable scientific capabilities and validation boundaries without depending on a particular site, locality, unpublished dataset, or private research question. Specific case studies can be defined separately when there is a deliberate reason to publish them.

## Completed baseline: v0.1 / M1-M7

v0.1 established the software and experiment-engine baseline:

- deterministic synthetic spatial environments;
- persistent people, households, demography and genealogy;
- renewable resources, condition and scarcity response;
- bounded, interpretable household migration;
- authoritative events, derived metrics and resumable checkpoints;
- a read-only explorer separated from authoritative simulation state;
- deterministic ensembles and parameter sweeps;
- immutable experiment provenance and explicit retry/failure semantics;
- long-run invariant, performance, memory and cross-platform determinism gates;
- machine-readable evidence provenance for future evidence-grounded parameters and external inputs.

This baseline can answer questions of the form "what does this declared synthetic model do when assumption X changes?" It is not yet an empirically validated model of a real past population or landscape.

## Development rule after v0.1

Post-v0.1 milestones should normally follow this sequence:

1. **State a research question or methodological target.** Define what distinction the experiment is intended to examine.
2. **Define the simplest relevant hypotheses or null model.** Do not add mechanisms merely because they are historically plausible.
3. **Define observable outputs before implementation.** State what patterns, distributions or contrasts would be compared and what would count as an informative failure.
4. **Identify the minimum missing capability.** Add only the model or data boundary required to run the experiment defensibly.
5. **Record assumptions and evidence.** Parameters and external inputs must retain provenance, transformations, units and uncertainty where scientifically relevant.
6. **Run ensembles and sensitivity analysis.** Prefer distributions and controlled comparisons to interpretation of a single artificial history.
7. **Treat negative results as information.** Failure of a simple model can identify which assumptions or mechanisms deserve investigation next.
8. **Let results shape the next milestone.** Later milestones should not become a fixed list of increasingly elaborate human behaviours detached from research need.

This keeps AnthroSim from becoming either a feature-accumulation project or a historical reconstruction engine whose desired outcome is embedded in its rules.

## M8 — Evidence-grounded spatial experiments

**Status:** completed. M8.0-M8.6 establish the generic Level-D evidence-grounded spatial experiment path; they do not establish case-study or archaeological validation.

### Goal

Allow AnthroSim to run controlled, reproducible experiments on evidence-grounded spatial environments while preserving the existing separation between authoritative simulation semantics and external GIS/scientific tooling.

M8 makes it possible to ask whether the mechanisms already represented by the model can generate informative spatial patterns under declared real-world environmental constraints, without scripting known settlements, destinations, routes or historical outcomes.

The first real-landscape exercise is treated as a **null-model benchmark**, not as a reconstruction claim. Its value is to establish what the existing demographic, resource and mobility mechanisms can and cannot explain before more complex social mechanisms are introduced.

### Architectural boundary

AnthroSim should not become a GIS application.

Raw elevation, LiDAR, hydrology, land-cover, palaeoenvironmental or other geospatial source data should normally be prepared with mature external tooling such as QGIS/GDAL and converted into a documented, versioned AnthroSim landscape bundle.

AnthroSim owns:

- the normalized landscape input contract used by authoritative runs;
- validation of that contract;
- deterministic mapping from declared spatial inputs to simulation state;
- the scientific meaning of model-facing layers and transformations;
- experiment provenance and content identity;
- causal observability of how spatial inputs affected simulated decisions and outcomes.

External tooling should continue to own generic GIS editing, reprojection, raster/vector processing and exploratory cartography.

### Implemented M8 slices

#### M8.0 — Spatial research/benchmark contract

The generic benchmark specification defines:

- the class of question being tested;
- the null or competing model assumptions;
- the spatial/environmental inputs required;
- the outputs that will be compared;
- sensitivity dimensions and uncertainty to preserve;
- explicit interpretation limits.

The benchmark contract remains usable without naming a particular archaeological site or requiring private/unpublished case-study information in the public core repository.

#### M8.1 — Versioned landscape input contract

The normalized spatial bundle records explicit:

- schema version;
- dimensions and cell resolution;
- coordinate reference metadata and extent;
- nodata/missing-data semantics;
- layer names, units and value domains;
- source/evidence references;
- content identity.

The contract supports synthetic fixtures as well as externally prepared real-world-derived landscapes.

#### M8.2 — Reproducible external preprocessing workflow

Documented lightweight tooling converts externally prepared GIS/scientific data into the normalized AnthroSim landscape contract while making transformations explicit rather than hiding them inside the simulation engine.

#### M8.3 — Deterministic landscape loading

Experiments can bind an external normalized landscape while preserving deterministic replay, invariant validation, checkpoint/resume semantics, exact experiment identity and the separation between immutable environmental inputs and dynamic simulation state.

Synthetic world generation remains available for engine tests and controlled experiments.

#### M8.4 — Evidence-grounded spatial mechanisms

Declared spatial layers connect to existing model mechanisms through explicit, inspectable and identity-bearing transformations for movement cost, water accessibility and resource opportunity. Source values and model-facing values remain distinguishable, and nodata behavior is explicit.

No transformation is treated as empirically valid merely because its input data are real. Source evidence, modelling assumptions, units, uncertainty and sensitivity ranges remain separate concerns.

#### M8.5 — Spatial observability and explorer support

Machine-readable spatial observability records provenance, occupancy, migration flows and spatial concentration independently of the read-only explorer. The explorer can display normalized and transformed layers without becoming authoritative simulation state.

Visualisation remains downstream from authoritative state. A visually realistic map does not substitute for provenance or imply historical validation.

#### M8.6 — First evidence-grounded spatial benchmark

The first Level-D benchmark runs four terrain-to-movement-cost alternatives across eight paired seeds through ordinary M7 ensemble machinery on one pinned, open, provenance-tracked terrain input.

All 32 runs reached the configured 100-year duration. The predeclared aggregate classification is **fragile spatial structure**: total migration distance and terminal largest-cell share showed material paired effects under the strong terrain mapping, but effect direction was not stable across seeds; cell-time occupancy and terminal Herfindahl concentration were not distinctive under the predeclared threshold.

This is a result about the declared terrain-only null model, not a reconstruction or validation of a historical population. See `docs/research/m8-first-evidence-grounded-benchmark-result.md` and the machine-readable `examples/m8-first-evidence-grounded-benchmark/reference-result.json`.

### M8 non-goals

M8 does not by itself:

- reconstruct a particular historical community or event;
- establish that a spatially grounded simulation is archaeologically valid;
- script known settlements, routes, boundaries or destinations;
- add culture, trade, warfare, institutions, religion, language or other mechanisms solely for completeness;
- replace QGIS/GDAL or general statistical tooling;
- turn explorer visualisation into authoritative simulation input;
- collapse uncertainty in environmental reconstruction into a single supposedly true landscape.

## Direction after M8

No fixed M9 feature list is declared yet.

The first evidence-grounded spatial benchmark shows that real-world-derived terrain can materially perturb individual simulated histories while the direction of the tested migration/concentration effects remains seed-sensitive. That result should shape the next question rather than be tuned away.

Candidate directions already consistent with the project vision include:

- alternative demographic, household or mobility assumptions where they are required by a concrete comparison;
- richer kinship/social-interaction mechanisms;
- settlement formation and persistence mechanisms;
- exchange or cultural-transmission models;
- calibration and comparison against independent evidence;
- archaeological observation models that transform simulated past behaviour into material remains, preservation, detection and sampling;
- reproduction of published models on an independent implementation for validation.

The ordering should be justified by experimental need rather than by which mechanism is easiest or most visually impressive to implement.

## Scientific interpretation boundary

A progression from synthetic worlds to real spatial data increases empirical relevance but does not automatically increase explanatory validity.

At every stage, AnthroSim should distinguish:

1. source observations or reconstructed inputs;
2. transformations from evidence into model inputs;
3. authoritative simulated past state and behaviour;
4. derived metrics and classifications;
5. archaeological observation/preservation/detection processes where modelled;
6. downstream interpretation.

Strong archaeological or anthropological claims require question-specific validation, sensitivity analysis, comparison with relevant evidence and, ultimately, domain review. The software should make those steps possible and auditable rather than imply that simulation alone supplies the answer.
