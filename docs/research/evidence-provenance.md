# Machine-readable evidence provenance

AnthroSim's synthetic v0.1 presets deliberately record only coarse `ParameterProvenance` classifications such as `synthetic_validation`. Evidence-grounded experiments need more detail before their parameters or spatial inputs can support archaeological interpretation.

The core therefore provides an optional, versioned `EvidenceCatalog` that can be attached directly to `ExperimentConfig`. Because the catalogue is serialized inside the ordinary experiment configuration, it is preserved by run/checkpoint/experiment provenance and contributes to the same reproducible experiment identity whenever it is present.

Synthetic-validation experiments remain lightweight: `ExperimentConfig::new(...)` sets `evidence` to `None`, and serialization omits the field entirely. Existing synthetic v0.1 configurations therefore do not acquire invented citations or unnecessary metadata.

## Evidence records

Each `EvidenceRecord` preserves:

- a stable experiment-local evidence identifier;
- provenance class (`empirical_direct`, `empirical_derived`, `evidence_informed`, or `unresolved`);
- source/dataset identity and a human-readable citation;
- optional persistent identifier, dataset version and licence;
- optional spatial and temporal coverage;
- original measured variable and source units;
- the transformation/aggregation/interpolation method used to derive simulation input, when applicable;
- resulting simulation units;
- a source-preserving uncertainty representation;
- an explicit applicability statement; and
- competing estimates where scientifically relevant.

The uncertainty value is stored as text rather than silently forcing evidence into floating-point authoritative simulation state. Any conversion into model units must be represented by an explicit transformation.

## Parameter links

`ParameterEvidenceLink` maps an evidence record to a stable dotted path in the serialized `ExperimentConfig`, for example:

`resources.annualRegenerationUnitsPerProductivity`

Multiple evidence records may support the same parameter. The link therefore does not imply that a single citation uniquely determines a model value; it makes the relationship inspectable.

## External spatial/data inputs

`ExternalInputEvidence` anticipates M8 and later GIS/data ingestion without coupling the simulation core to a particular GIS library or database. It records an experiment-local input identifier, source evidence record, file/data format, optional spatial reference and optional content digest.

A future terrain layer can therefore identify a source DEM, its CRS, source/version/licence and the transformation that produced the simulation layer while leaving loading/resampling implementation outside the provenance schema.

## Validation and scientific boundary

`EvidenceCatalog::validate()` rejects unsupported record/catalog versions, duplicate/empty identifiers, broken parameter/input references, empty required fields and records incorrectly labelled `synthetic_validation`. Synthetic presets should not manufacture heavyweight evidence records merely to fill the schema.

The evidence schema and its identity wiring are covered by the ordinary workspace tests. As M8 adds new evidence-backed spatial inputs or new cross-system invariants, representative cases should also be added to AnthroSim's generated invariant coverage rather than relying only on hand-written happy paths.

This structure establishes traceability; it does **not** establish that a source is correct, that a transformation is archaeologically justified, or that uncertainty has been propagated adequately. Those remain scientific modelling decisions that must be reviewed and tested for each evidence-grounded preset.

The catalogue is intended to be expanded alongside M8's real-landscape input architecture. It deliberately does not prescribe a database, GIS stack, raster format or archaeological ontology.
