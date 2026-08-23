# M8.1 normalized landscape contract v1

## Purpose

M8.1 introduces a simulation-independent normalized landscape bundle. It is the boundary between external GIS/scientific preprocessing and later authoritative AnthroSim landscape loading.

The bundle does **not** replace the existing synthetic `World` and M8.1 does not change simulation semantics. M8.3 will deliberately connect validated normalized landscapes to authoritative experiment execution.

## Design rules

- the simulation runtime consumes normalized values rather than arbitrary GIS formats;
- geometry and layer metadata are explicit and versioned;
- spatial values are integer-valued or explicit nodata, avoiding hidden floating-point conversion at the authoritative boundary;
- row-major ordering is part of the contract;
- source/evidence identifiers link to the existing experiment `EvidenceCatalog` without embedding a GIS database in the core;
- a stable non-cryptographic identity detects semantic changes to normalized inputs for deterministic experiment wiring;
- cryptographic research-archive integrity remains a separate concern;
- public fixtures remain generic and do not disclose a private case study.

## `LandscapeBundle`

Schema version 1 contains:

- `width` and `height`;
- `GridGeometry`;
- an ordered list of `LandscapeLayer` records.

The cell count is `width * height`. Every layer contains exactly that many row-major values.

## Grid geometry

`GridGeometry` records:

- integer `originX` / `originY`;
- positive integer `cellSizeX` / `cellSizeY`;
- explicit `coordinateUnit`;
- explicit `spatialReference` string.

M8.1 deliberately does not parse or reproject the spatial reference. Those are external preprocessing responsibilities. The metadata must nevertheless be preserved so the normalized coordinates remain interpretable.

## Layers

Each `LandscapeLayer` has:

- stable `layerId`;
- semantic `role`;
- explicit `unit`;
- optional inclusive integer `valueDomain`;
- optional `evidenceInputId` linking to `EvidenceCatalog.externalInputs`;
- row-major `values`, where JSON `null`/Rust `None` is explicit nodata.

The initial semantic roles are:

- `terrain_traversal`;
- `water_accessibility`;
- `resource_opportunity`;
- `auxiliary`.

These roles describe normalized scientific inputs. They do not yet define the exact behavioural transformation used by migration/resources; that belongs to M8.4.

## Validation

`LandscapeBundle::validate()` rejects:

- unsupported schema versions;
- zero dimensions;
- zero cell sizes;
- empty coordinate units or spatial reference metadata;
- empty or duplicate layer identifiers;
- empty layer units/evidence-input identifiers;
- layer lengths inconsistent with grid dimensions;
- invalid declared value domains;
- non-nodata values outside their declared domains.

`validate_evidence_links()` separately checks that any `evidenceInputId` exists in the supplied experiment `EvidenceCatalog`.

This separation allows synthetic fixtures to stay lightweight while evidence-grounded experiments can enforce traceability.

## Identity

`LandscapeBundle::identity()` returns a stable schema-qualified fingerprint derived from all normalized geometry, layer metadata, evidence-input links, nodata positions and values.

It is intentionally described as **non-cryptographic**. Its purpose is deterministic experiment identity/regression wiring, analogous to existing state fingerprints. Research-archive tamper detection should use the separate cryptographic integrity layer tracked independently.

Changing a single normalized value changes the landscape identity. Filesystem iteration order cannot change identity because the normalized bundle itself has an explicit ordered representation.

## Nodata

Nodata is explicit per cell as `None`/JSON `null`; there is no magic numeric sentinel in the core contract.

M8.2 preprocessing must decide how source nodata is propagated, filled or excluded and record that transformation. M8.3/M8.4 must not silently reinterpret nodata as zero productivity, zero water, or an impassable cell without an explicit rule.

## Relationship to evidence provenance

A layer may reference an experiment-local `ExternalInputEvidence.inputId`.

That external input can preserve source citation/version/licence, spatial reference and content digest through its linked `EvidenceRecord`. M8.2 is responsible for recording the preprocessing transformation from source GIS data to normalized landscape values.

A layer without an evidence link is allowed for synthetic fixtures and method validation. Such a layer must not later be described as evidence-grounded merely because it uses the same schema.

## M8.1 scientific boundary

This schema provides deterministic, inspectable spatial inputs. It does not establish that:

- the source dataset is historically appropriate;
- a reconstruction is correct;
- the transformation into normalized values is scientifically justified;
- a model-facing movement/resource relationship is valid;
- a resulting simulation is archaeologically validated.

Those questions remain visible for M8.2-M8.6 and later case-study work rather than being hidden by the data format.
