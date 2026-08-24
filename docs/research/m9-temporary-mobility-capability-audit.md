# M9 capability audit — temporary mobility and aggregation

**Status:** roadmap decision input  
**Baseline audited:** post-v0.2.0 `main` at `030c7299f26dea7edb0cea7f3aee66b34d30781f`

## Purpose

This audit asks one narrow software-capability question:

> Can the current AnthroSim model represent dispersed households leaving persistent residences, temporarily aggregating in a declared spatial region for a bounded duration, and then returning home, while preserving reproducible causal observability?

The motivating research class is intentionally generic. The public engine must not encode a named archaeological site, known historical destination, or preferred interpretation.

## Finding

**No.** AnthroSim v0.2.0 can model permanent household relocation through M4 migration, but it cannot yet represent reversible temporary mobility as a scientifically distinct process.

The missing capability is substantial enough to justify the next roadmap milestone because it blocks controlled comparison of continuous residence against intermittent aggregation while reusing the existing M1–M8 demographic, household, resource, experiment, provenance, and evidence-grounded spatial foundations.

## What already exists and should be reused

- Persistent people and households with stable identity.
- Integer-day authoritative simulation time.
- Household-level permanent migration with deterministic stochastic choice.
- Evidence-grounded movement-cost, water-access and resource-opportunity fields from M8.
- Versioned normalized landscape layers, including `Auxiliary` layers that can carry externally prepared masks without turning AnthroSim into GIS software.
- Ordered day-indexed authoritative events.
- Checkpoint/resume, run bundles, provenance, ensembles and parameter sweeps.
- M8.5 spatial observability that reconstructs occupancy duration, living person-days, migration flows and concentration from preserved authoritative state/events.

These foundations mean M9 should extend mobility semantics rather than create a parallel simulation stack.

## Genuine capability gaps

### 1. Residence and physical presence are the same state

`Population` stores one household location plus per-person locations. Validation requires every living person's location to equal the household location. There is no persistent residence/home field distinct from current physical presence.

Consequently, a temporary visit cannot be represented without changing what the model currently means by household location.

### 2. Existing movement is permanent relocation

`MigrationSystem` selects a destination and `Population::apply_household_relocations` overwrites the household location and all living member locations. The move is recorded as `HouseholdMigration` and there is no retained origin residence to return to.

Permanent migration must remain scientifically distinct rather than being overloaded to mean visits, refuge, seasonal aggregation, exchange trips, or other temporary movement.

### 3. Journeys are atomic

The scientific model explicitly states that M4 has no en-route state, journey duration, camp sequence, route choice or movement mortality. Selected moves complete atomically at a resource/migration boundary.

This is sufficient for the M4 relocation null model but not for experiments where travel time, bounded absence, arrival timing, stay duration and return matter.

### 4. No model-facing focal-region binding exists

The landscape contract can preserve arbitrary `Auxiliary` layers, but current model mechanisms do not consume a named destination/region mask. M9 needs a generic binding from an externally prepared or synthetic region definition to temporary-mobility semantics.

GIS remains external: QGIS/GDAL should prepare real polygon/raster masks. AnthroSim should only validate, identify and consume the resulting normalized model-facing region.

### 5. Current resource accounting cannot interpret a short visit correctly

Resource processing occurs at configured subannual boundaries and allocates a household's whole period need to its single household location. A five-day visit that happens to cross a resource boundary would therefore be silently treated as if the household occupied that destination for the whole resource period; a visit entirely between boundaries would exert no destination resource demand at all.

M9 must introduce an explicit duration-aware resource/presence boundary rather than inherit either artefact.

### 6. Existing observability is close but migration-specific

M8.5 already derives occupied duration and living person-days by replaying authoritative events. That architecture is suitable for temporary mobility, but it currently knows only births, deaths and permanent household migrations.

M9 should extend authoritative events and derived observability so temporary departures, arrivals, stays and returns can be distinguished from permanent relocation and measured independently.

## Capability decision

The minimum coherent next milestone is:

> **M9 — Temporary mobility and aggregation experiments:** allow households with persistent residences to undertake reproducible, bounded temporary journeys to declared focal regions, remain away for explicit durations, and return, while keeping permanent migration separate and making temporary presence, travel and resource consequences observable.

This is a reusable anthropological simulation capability rather than a site-specific feature. It is required by a real class of archaeological questions but does not assume why aggregation occurs.

## Required M9 boundaries

M9 should establish, at minimum:

1. distinct authoritative **residence** and **current-presence / mobility** semantics;
2. generic, identity-bearing **focal-region bindings** that can consume externally prepared landscape masks;
3. a deterministic **temporary journey lifecycle** with departure, travel/arrival, bounded stay and return;
4. explicit **travel-time/cost semantics** that can use existing movement-cost information without becoming a general GIS routing package;
5. duration-aware **resource/presence accounting** so short visits are not silently charged as whole resource periods or ignored between boundaries;
6. checkpoint/resume, invariant, RNG/provenance and ensemble integration;
7. authoritative temporary-mobility events plus derived observables including visits, arrivals, returns, peak presence, person-days, travel catchment/distance/cost and occupancy by region;
8. a controlled validation benchmark that distinguishes an intermittent-aggregation regime from a continuous-residence regime even when aggregate person-days are deliberately similar.

## Non-goals

M9 does not require or justify adding:

- a named archaeological site or case-study-specific rules to the public engine;
- GIS editing, reprojection, polygon drawing, viewsheds or generic cartography;
- trade, ritual, feasting, religion or political institutions as reasons for aggregation;
- combat simulation or an attacker model;
- livestock/herd simulation;
- archaeological preservation/detection/observation modelling;
- settlement formation as an emergent social institution;
- calibrated claims about any real prehistoric population;
- a general-purpose pathfinding/GIS routing product beyond the minimum deterministic journey-cost semantics needed by the experiment.

Those capabilities should be added only if later experiments demonstrate that they are genuinely required.

## Acceptance principle

M9 is complete when AnthroSim can run a reproducible ensemble in which otherwise comparable households can be assigned persistent residences, temporarily aggregate at a declared region for bounded durations, return home, interact with time/resource accounting under explicit assumptions, and produce machine-readable outputs that distinguish temporary use from permanent residence and permanent migration.

The first benchmark should be synthetic or otherwise controlled. Passing it demonstrates software/model capability, not archaeological validation.
