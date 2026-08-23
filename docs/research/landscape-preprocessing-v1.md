# M8.2 external landscape preprocessing workflow v1

## Purpose

M8.2 defines a reproducible boundary between general-purpose GIS tooling and AnthroSim's normalized M8.1 landscape contract.

AnthroSim does not attempt to replace QGIS, GDAL or equivalent geospatial tools. Reprojection, clipping, raster/vector operations, resampling and scientific derivation of spatial variables remain external preprocessing steps. The public simulator consumes the resulting normalized, aligned integer grids.

The workflow is deliberately case-study-neutral. Private or unpublished research can use the same recipe format without committing local source paths, data or hypotheses to the public repository.

## Workflow

A reproducible landscape preparation has three stages:

1. **Prepare source GIS layers externally.** Use QGIS/GDAL or another documented toolchain to reproject, clip, align and derive the required layers on one declared grid.
2. **Export one aligned integer CSV grid per model-facing layer.** Each CSV must have exactly `height` rows and `width` values per row. Use the recipe's nodata token for missing cells.
3. **Assemble the M8.1 bundle.** Run `scripts/build-landscape-bundle.py` with a versioned recipe. The script validates alignment/value domains, hashes sources and aligned layer exports, and writes deterministic normalized JSON plus a reproduction record.

The assembler performs no reprojection or scientific interpolation itself. This is intentional: those choices are scientifically meaningful and must remain visible rather than hidden in simulator runtime behaviour.

## Recipe

Recipe schema version 1 records:

- target grid geometry: dimensions, origin, cell size, coordinate unit and spatial reference;
- external toolchain names and pinned/reported versions;
- ordered preprocessing steps with human-readable descriptions and exact or templated commands;
- source files with citation/version/licence metadata and SHA-256 identity;
- explicit nodata policy;
- aligned layer CSV paths, roles, units, optional value domains, nodata tokens and optional `EvidenceCatalog.externalInputs` identifiers;
- optional notes.

See `examples/landscape-preprocess/recipe.json` for a synthetic publishable fixture.

## QGIS/GDAL usage

A real workflow should record concrete commands/settings rather than relying on a remembered GUI sequence. For example, a GDAL-based preparation may use commands equivalent to:

```text
gdalwarp -t_srs <target-crs> -te <xmin> <ymin> <xmax> <ymax> -tr <xres> <yres> -r <declared-resampling> <source> <aligned-output>
```

If QGIS is used interactively, the recipe or adjacent research record should preserve the processing algorithm, parameter values, QGIS/GDAL versions and any expressions used to derive model-facing values. QGIS processing history/model exports can be retained alongside the research archive when useful.

The placeholder commands in the public synthetic fixture demonstrate the fields only. They are not scientific recipes for any real landscape.

## Model-facing values

M8.1 expects integer values or explicit nodata. Therefore any source float raster, categorical layer or vector geometry must be transformed externally into a declared integer layer before assembly.

Examples of transformations that must be documented include:

- elevation/slope to normalized traversal opportunity;
- distance-to-water or hydrological classification to water accessibility;
- land-cover/soil/environmental reconstruction to resource opportunity;
- interpolation, aggregation, classification or uncertainty handling.

M8.2 records these transformations; it does not declare them scientifically valid. M8.4 is responsible for connecting normalized spatial values to behavioural/resource mechanisms and sensitivity-testing uncertain relationships.

## Nodata

Nodata policy is mandatory in the recipe.

The assembler converts a layer's declared nodata token (default `NA`) to JSON `null`. It never silently converts nodata to zero, minimum productivity, maximum movement cost or another scientific value.

If source nodata is filled or interpolated before export, that decision belongs in the preprocessing steps and uncertainty/provenance record.

## Determinism and integrity

`build-landscape-bundle.py` writes canonical JSON with stable key ordering and compact separators. Given byte-identical recipe, source files and aligned layer exports, it produces byte-identical landscape and reproduction-record files.

The reproduction record contains SHA-256 hashes for:

- the recipe bytes;
- each declared source file;
- each aligned layer input;
- the normalized landscape output.

These hashes establish exact preprocessing input/output identity. They do not replace the separate research-archive integrity manifest planned by issue #42, and they do not establish that the scientific transformations are correct.

## Example

From the repository root:

```text
python scripts/build-landscape-bundle.py examples/landscape-preprocess/recipe.json --output runs/example-landscape.json --record runs/example-landscape-preprocessing.json
```

The checked-in fixture is synthetic and exists only to test the contract. It is not empirical evidence and is not a hidden case-study reconstruction.

## Reproduction checklist

Before treating a prepared landscape as evidence-grounded, preserve:

- source dataset identity/citation/version/licence;
- source files or an authorized way to retrieve the exact version;
- target CRS, extent and resolution;
- QGIS/GDAL/tool versions;
- reprojection/resampling/interpolation choices;
- all derivation expressions/commands;
- nodata policy;
- aligned exported grids;
- recipe, normalized bundle and reproduction record;
- uncertainty/alternative reconstructions where relevant.

A collaborator with access to permitted source data should be able to repeat the external steps and obtain the same aligned inputs and normalized bundle, subject to explicitly documented tool/version determinism boundaries.

## Scientific boundary

A reproducible GIS transformation is not automatically a defensible archaeological or anthropological transformation. M8.2 provides traceability and repeatability; source suitability, palaeoenvironmental interpretation, uncertainty and behavioural meaning remain separate scientific questions.
