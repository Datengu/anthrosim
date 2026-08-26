# AnthroSim roadmap

## Purpose

AnthroSim's roadmap is driven by research capability rather than feature count. The project should add new mechanisms or infrastructure when they are needed to ask a clearer question, distinguish competing explanations, expose uncertainty, or make an experiment more reproducible and inspectable.

The long-term direction remains the one described in `docs/vision.md`: build enough defensible lower-level rules that history-like structure can emerge without scripting historical outcomes.

This roadmap is intentionally case-study-neutral. Public core documentation should describe reusable scientific capabilities and validation boundaries without depending on any particular site, locality, dataset or project-specific research question. Specific case studies, when published, should live in explicitly scoped research artifacts rather than define the core engine roadmap.

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

The benchmark contract remains usable without naming a particular archaeological site or requiring case-study-specific information in the public core repository.

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

## M9 — Temporary mobility and aggregation experiments

**Status:** completed. M9.0-M9.7 establish the generic temporary-mobility and controlled aggregation capability. The authoritative M9 semantics contract is `docs/research/temporary-mobility-v1.md`; the capability audit that motivated it is `docs/research/m9-temporary-mobility-capability-audit.md`.

### Goal

Allow households with persistent residences to undertake reproducible, bounded temporary journeys to declared focal regions, remain away for explicit durations and return home, while keeping temporary presence scientifically distinct from permanent migration.

M9 makes controlled experiments possible that compare continuous residence with intermittent aggregation on synthetic or evidence-grounded landscapes. It does not assume why people aggregate; the initial mechanism is a null-model mobility capability, not a model of trade, ritual, refuge, politics or any named archaeological interpretation.

### Why M9 was the next missing capability

The v0.2.0 model could not represent the required experiment without changing the meaning of existing state:

- a household had one authoritative location that functioned simultaneously as residence and current presence;
- every living household member was required to occupy that same location;
- M4 migration permanently overwrote the household and living-member locations;
- migration journeys were atomic and had no journey duration, arrival/stay/return lifecycle or en-route state;
- no model-facing focal-region binding told a temporary-mobility mechanism where a declared aggregation area was;
- resource demand was charged to the household's single location for an entire resource period, which would misrepresent short visits;
- M8.5 could reconstruct person-days and occupancy from authoritative events, but only understood permanent migration as a movement event.

M9 addresses these as model/software capability gaps rather than treating them as missing archaeological data or missing GIS functionality.

### Architectural boundary

M9 extends AnthroSim's human-mobility semantics; it does not make AnthroSim a GIS or general routing application.

External GIS/scientific tooling should continue to own real-world polygon/raster editing, reprojection and preparation. AnthroSim may consume an externally prepared normalized mask or other declared region representation and give that input an explicit model-facing identity and role.

Permanent M4 migration remains a separate causal process. M9 must not silently reinterpret `HouseholdMigration` as temporary travel or overload a single location field with incompatible meanings.

### M9 implementation slices

#### M9.0 — Temporary-mobility research and semantics contract — complete

`docs/research/temporary-mobility-v1.md` freezes the minimum generic experiment semantics before authoritative implementation, including:

- persistent residence versus current physical presence;
- permanent relocation versus temporary travel;
- focal-region identity and provenance;
- departure, arrival, stay and return semantics;
- travel-time/cost assumptions and routing boundaries;
- duration-aware resource-accounting assumptions during temporary absence/presence;
- interaction with M4 migration and M2 demography;
- authoritative events, checkpoint/resume and observability requirements;
- the M9.7 benchmark acceptance criteria;
- explicit interpretation limits.

#### M9.1 — Residence/presence state separation — complete

Introduce authoritative state that can preserve a household's persistent residence while its living members are temporarily elsewhere or in transit. Invariants must make the relationship between household membership, residence and physical presence explicit rather than ambiguous.

Existing permanent migration must update residence under its own semantics; temporary travel must not.

#### M9.2 — Generic focal-region binding — complete

Add an identity-bearing experimental region contract that temporary mobility can target. Real region geometry should normally be prepared outside AnthroSim and supplied through the existing normalized landscape boundary, for example through a declared auxiliary mask or equivalent versioned representation.

The engine owns validation and scientific meaning of the binding, not GIS editing.

#### M9.3 — Deterministic temporary journey lifecycle — complete

Add a temporary-mobility process capable of deterministic/reproducible:

- departure from residence;
- travel/arrival timing;
- bounded stay duration;
- return travel;
- restoration of presence at the persistent residence.

Triggers should initially be generic and experiment-configured. A temporary journey need not claim a real social motive.

#### M9.4 — Travel-time and cost semantics — complete

`docs/research/m9-temporary-travel-semantics-v1.md` freezes the M9.4 integer edge-cost, reachability, destination tie-break and travel-capacity semantics before authoritative implementation is merged.

Define the minimum deterministic travel-duration/cost calculation required for temporary journeys, using existing model-facing movement-cost information where appropriate.

This capability must remain inspectable and provenance-bearing. It should not grow into a general GIS route-planning product; mature external GIS remains responsible for generic routing/cartographic analysis not required by authoritative simulated behaviour.

#### M9.5 — Duration-aware resource/presence accounting — complete

Ensure short visits cannot be silently treated as whole resource periods at the destination or disappear entirely when they occur between current resource boundaries.

The implemented approximation must explicitly account for how household need/resource pressure is attributed across residence, travel and temporary presence, preserve exact deterministic accounting, and expose the assumption as model semantics rather than hiding it in scheduling code.

#### M9.6 — Temporary-mobility observability and experiment integration — complete

M9 temporary mobility now participates in ordinary transformed-spatial execution, immutable experiment identity, run/ensemble/sweep inputs, checkpoint/resume, completed/paused artifact workflows and deterministic downstream observability.

The world-independent experiment definition preserves focal region, schedule and travel-model assumptions, while every run derives its resolved travel table from that run's own authoritative world. `anthrosim-temporary-observability` regenerates a separate machine-readable report from preserved authoritative artifacts rather than changing the meaning of M8 spatial observability.

The implemented report distinguishes:

- persistent residence from physical temporary presence;
- temporary visitors from focal-region residents;
- outbound and return transit without assigning transit to arbitrary cells;
- starts, explicit non-start outcomes, arrivals, return departures and completions;
- visit-duration distributions and peak/mean visitor presence;
- persistent-residence, at-residence, visitor and transit person-days with exact accounting identities;
- journey time/cost, derived route edge distance where it reconciles to M9.4 routing, and origin catchment;
- permanent M4 migration from temporary movement.

Completed bundles can carry the derived report and fail closed if it cannot be regenerated exactly. Paused runs with resume-boundary population provenance can reconstruct the day-zero founder state deterministically and derive/verify the same report. The read-only Explorer can surface the derived summary and M9 event family without changing residence maps or inventing transit locations.

See `docs/research/temporary-mobility-observability-v1.md` and `docs/research/m9-6-integration-audit.md`.

#### M9.7 — Controlled aggregation benchmark — complete

The frozen M9.7 benchmark compares paired continuous-residence and intermittent-aggregation regimes across seeds 9701-9708 in the same controlled 10×10 synthetic worlds and 70-cell focal region. Its assumptions and acceptance thresholds were committed before first result inspection.

The first execution classified **`capability_distinguished`**. All 8/8 paired seeds met the predeclared criteria: total focal-region person-days remained within 5% between arms while the intermittent arm produced a materially concentrated visitor signal above the declared peak threshold. Authoritative event replay reconciled with machine-readable M9.6 observability, duplicate execution was exact, and an annual checkpoint captured active journeys and resumed to the same terminal authoritative state and observability as uninterrupted execution.

The first-observation result is preserved as a machine-readable reference and protected by a tamper-rejecting CI verifier. See `docs/research/m9-controlled-aggregation-benchmark-v1.md`, `docs/research/m9-controlled-aggregation-benchmark-result.md` and `examples/m9-controlled-aggregation-benchmark/reference-result.json`.

This benchmark validates the M9 capability and its observability only. It does not validate any archaeological interpretation or claim that intermittent aggregation or continuous residence explains a real site.

### M9 non-goals

M9 does not by itself add or establish:

- a named archaeological site or case-study-specific rules in the public core;
- trade, ritual, feasting, religion or political institutions as causes of aggregation;
- detailed combat, attackers or warfare;
- livestock/herd simulation;
- settlement formation as a higher-level institution;
- archaeological preservation, detection or observation models;
- empirical calibration of temporary mobility for a real prehistoric population;
- a general-purpose GIS or route-planning suite.

Those capabilities should be considered only when a later controlled research question demonstrates that they are required.

## Direction after M9

No fixed M10 feature list is declared yet.

M9 is complete, its post-milestone audit/hardening has been resolved, and the audited capability is preserved as the `v0.3.0` release baseline. The next project step is therefore research-led rather than a predetermined M10 feature package: concrete comparisons using the M8/M9 capabilities should identify which missing mechanism, uncertainty treatment, observation layer or validation target is actually needed next.

Candidate directions remain valid only when justified by experimental need, including:

- evidence-grounded or alternative demographic/household initialization where a concrete comparison requires it;
- settlement formation and persistence mechanisms;
- livestock or managed-herd behaviour;
- richer kinship/social-interaction mechanisms;
- collective labour and construction costs;
- exchange or cultural-transmission models;
- archaeological observation models that transform simulated past behaviour into material remains, preservation, detection and sampling;
- calibration and comparison against independent evidence;
- reproduction of published models on an independent implementation for validation.

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