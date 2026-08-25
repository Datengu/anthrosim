use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::config::{ExperimentConfig, ParameterProvenance};

/// Versioned evidence catalogue attached to an experiment when parameters or
/// external inputs are grounded in empirical or evidence-informed sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceCatalog {
    pub schema_version: u32,
    pub records: Vec<EvidenceRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_links: Vec<ParameterEvidenceLink>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_inputs: Vec<ExternalInputEvidence>,
}

impl EvidenceCatalog {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn new(records: Vec<EvidenceRecord>) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            records,
            parameter_links: Vec::new(),
            external_inputs: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_parameter_links(mut self, links: Vec<ParameterEvidenceLink>) -> Self {
        self.parameter_links = links;
        self
    }

    #[must_use]
    pub fn with_external_inputs(mut self, inputs: Vec<ExternalInputEvidence>) -> Self {
        self.external_inputs = inputs;
        self
    }

    /// Validate the catalogue as a self-contained evidence artifact.
    ///
    /// Parameter paths are experiment-schema references, so their existence is
    /// checked separately by [`Self::validate_against_experiment`].
    pub fn validate(&self) -> Result<(), EvidenceError> {
        validate_evidence_catalog(self)
    }

    /// Validate this catalogue and resolve every parameter link against the
    /// actual camelCase serialized shape of the supplied experiment schema.
    pub fn validate_against_experiment(
        &self,
        experiment: &ExperimentConfig,
    ) -> Result<(), EvidenceError> {
        validate_evidence_catalog(self)?;
        validate_parameter_evidence_links(self, experiment)
    }
}

/// One evidence record describing the origin and interpretation of a value or
/// input. Text fields intentionally preserve source wording/units without
/// introducing floating-point authoritative simulation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRecord {
    pub schema_version: u32,
    pub evidence_id: String,
    pub provenance: ParameterProvenance,
    pub source: EvidenceSource,
    pub original_variable: String,
    pub original_units: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformation: Option<EvidenceTransformation>,
    pub simulation_units: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<EvidenceUncertainty>,
    pub applicability: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub competing_estimates: Vec<String>,
}

impl EvidenceRecord {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSource {
    pub source_id: String,
    pub citation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_coverage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_coverage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceTransformation {
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_units: Option<String>,
    pub simulation_units: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceUncertainty {
    /// Machine-readable label such as `range`, `distribution`, `standard_error`
    /// or `qualitative`.
    pub representation: String,
    /// Source-preserving textual value, for example `20-35 km`, `N(4.1,0.6)`
    /// or `low confidence`. This avoids falsely converting evidence into model
    /// units before an explicit transformation is documented.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterEvidenceLink {
    /// Stable dotted path in the serialized experiment configuration, for
    /// example `resources.annualNeedUnitsPerPerson`.
    ///
    /// Paths traverse object keys only. Collection indices are deliberately
    /// excluded because an array position is not a stable parameter identity.
    pub parameter_path: String,
    pub evidence_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInputEvidence {
    /// Stable experiment-local identifier such as `dem_5m` or
    /// `landcover_reconstruction_v2`.
    pub input_id: String,
    pub evidence_id: String,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    #[error("evidence catalogue schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedCatalogSchema { found: u32, supported: u32 },
    #[error("evidence record schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedRecordSchema { found: u32, supported: u32 },
    #[error("evidence identifier is empty")]
    EmptyEvidenceId,
    #[error("duplicate evidence identifier {0}")]
    DuplicateEvidenceId(String),
    #[error("evidence record {evidence_id} is marked synthetic_validation")]
    SyntheticEvidenceRecord { evidence_id: String },
    #[error("evidence record {evidence_id} has an empty required field {field}")]
    EmptyRecordField {
        evidence_id: String,
        field: &'static str,
    },
    #[error("parameter evidence link has an empty parameter path")]
    EmptyParameterPath,
    #[error(
        "experiment schema {found} is unsupported for evidence parameter-path validation; supported schema is {supported}"
    )]
    UnsupportedExperimentSchema { found: u32, supported: u32 },
    #[error("failed to serialize experiment configuration for evidence parameter-path validation")]
    ExperimentSerialization,
    #[error(
        "parameter evidence path {parameter_path} does not resolve in serialized experiment schema {schema_version}"
    )]
    UnknownParameterPath {
        parameter_path: String,
        schema_version: u32,
    },
    #[error(
        "parameter evidence path {parameter_path} traverses a collection; collection indices are not stable parameter paths"
    )]
    CollectionParameterPath { parameter_path: String },
    #[error("parameter evidence path {parameter_path} resolves to an unconfigured optional value")]
    UnconfiguredParameterPath { parameter_path: String },
    #[error(
        "parameter evidence path {parameter_path} resolves to a container rather than a parameter leaf"
    )]
    ParameterPathNotLeaf { parameter_path: String },
    #[error(
        "parameter evidence path {parameter_path} may not target the evidence catalogue itself"
    )]
    EvidenceSelfReference { parameter_path: String },
    #[error("external input has an empty input identifier")]
    EmptyExternalInputId,
    #[error("duplicate external input identifier {0}")]
    DuplicateExternalInputId(String),
    #[error("evidence reference {evidence_id} does not exist in the catalogue")]
    UnknownEvidenceReference { evidence_id: String },
}

pub fn validate_evidence_catalog(catalog: &EvidenceCatalog) -> Result<(), EvidenceError> {
    if catalog.schema_version != EvidenceCatalog::CURRENT_SCHEMA_VERSION {
        return Err(EvidenceError::UnsupportedCatalogSchema {
            found: catalog.schema_version,
            supported: EvidenceCatalog::CURRENT_SCHEMA_VERSION,
        });
    }

    let mut evidence_ids = BTreeSet::new();
    for record in &catalog.records {
        if record.schema_version != EvidenceRecord::CURRENT_SCHEMA_VERSION {
            return Err(EvidenceError::UnsupportedRecordSchema {
                found: record.schema_version,
                supported: EvidenceRecord::CURRENT_SCHEMA_VERSION,
            });
        }
        let evidence_id = record.evidence_id.trim();
        if evidence_id.is_empty() {
            return Err(EvidenceError::EmptyEvidenceId);
        }
        if !evidence_ids.insert(evidence_id.to_owned()) {
            return Err(EvidenceError::DuplicateEvidenceId(evidence_id.to_owned()));
        }
        if record.provenance == ParameterProvenance::SyntheticValidation {
            return Err(EvidenceError::SyntheticEvidenceRecord {
                evidence_id: evidence_id.to_owned(),
            });
        }
        for (field, value) in [
            ("source.sourceId", record.source.source_id.as_str()),
            ("source.citation", record.source.citation.as_str()),
            ("originalVariable", record.original_variable.as_str()),
            ("originalUnits", record.original_units.as_str()),
            ("simulationUnits", record.simulation_units.as_str()),
            ("applicability", record.applicability.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(EvidenceError::EmptyRecordField {
                    evidence_id: evidence_id.to_owned(),
                    field,
                });
            }
        }
        if let Some(transformation) = &record.transformation {
            for (field, value) in [
                ("transformation.method", transformation.method.as_str()),
                (
                    "transformation.simulationUnits",
                    transformation.simulation_units.as_str(),
                ),
            ] {
                if value.trim().is_empty() {
                    return Err(EvidenceError::EmptyRecordField {
                        evidence_id: evidence_id.to_owned(),
                        field,
                    });
                }
            }
        }
        if let Some(uncertainty) = &record.uncertainty {
            for (field, value) in [
                (
                    "uncertainty.representation",
                    uncertainty.representation.as_str(),
                ),
                ("uncertainty.value", uncertainty.value.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(EvidenceError::EmptyRecordField {
                        evidence_id: evidence_id.to_owned(),
                        field,
                    });
                }
            }
        }
    }

    for link in &catalog.parameter_links {
        if link.parameter_path.trim().is_empty() {
            return Err(EvidenceError::EmptyParameterPath);
        }
        if !evidence_ids.contains(link.evidence_id.trim()) {
            return Err(EvidenceError::UnknownEvidenceReference {
                evidence_id: link.evidence_id.clone(),
            });
        }
    }

    let mut input_ids = BTreeSet::new();
    for input in &catalog.external_inputs {
        let input_id = input.input_id.trim();
        if input_id.is_empty() {
            return Err(EvidenceError::EmptyExternalInputId);
        }
        if !input_ids.insert(input_id.to_owned()) {
            return Err(EvidenceError::DuplicateExternalInputId(input_id.to_owned()));
        }
        if !evidence_ids.contains(input.evidence_id.trim()) {
            return Err(EvidenceError::UnknownEvidenceReference {
                evidence_id: input.evidence_id.clone(),
            });
        }
        if input.format.trim().is_empty() {
            return Err(EvidenceError::EmptyRecordField {
                evidence_id: input.evidence_id.clone(),
                field: "externalInputs.format",
            });
        }
    }

    Ok(())
}

pub fn validate_parameter_evidence_links(
    catalog: &EvidenceCatalog,
    experiment: &ExperimentConfig,
) -> Result<(), EvidenceError> {
    if experiment.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
        return Err(EvidenceError::UnsupportedExperimentSchema {
            found: experiment.schema_version,
            supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    let serialized =
        serde_json::to_value(experiment).map_err(|_| EvidenceError::ExperimentSerialization)?;

    for link in &catalog.parameter_links {
        validate_parameter_path(
            &serialized,
            link.parameter_path.trim(),
            experiment.schema_version,
        )?;
    }
    Ok(())
}

fn validate_parameter_path(
    experiment: &Value,
    parameter_path: &str,
    schema_version: u32,
) -> Result<(), EvidenceError> {
    let segments = parameter_path.split('.').collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(EvidenceError::UnknownParameterPath {
            parameter_path: parameter_path.to_owned(),
            schema_version,
        });
    }
    if segments.first() == Some(&"evidence") {
        return Err(EvidenceError::EvidenceSelfReference {
            parameter_path: parameter_path.to_owned(),
        });
    }

    let mut current = experiment;
    for segment in &segments {
        current = match current {
            Value::Object(object) => {
                object
                    .get(*segment)
                    .ok_or_else(|| EvidenceError::UnknownParameterPath {
                        parameter_path: parameter_path.to_owned(),
                        schema_version,
                    })?
            }
            Value::Array(_) => {
                return Err(EvidenceError::CollectionParameterPath {
                    parameter_path: parameter_path.to_owned(),
                });
            }
            Value::Null => {
                return Err(EvidenceError::UnconfiguredParameterPath {
                    parameter_path: parameter_path.to_owned(),
                });
            }
            _ => {
                return Err(EvidenceError::UnknownParameterPath {
                    parameter_path: parameter_path.to_owned(),
                    schema_version,
                });
            }
        };
    }

    match current {
        Value::Object(_) | Value::Array(_) => Err(EvidenceError::ParameterPathNotLeaf {
            parameter_path: parameter_path.to_owned(),
        }),
        Value::Null => Err(EvidenceError::UnconfiguredParameterPath {
            parameter_path: parameter_path.to_owned(),
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> EvidenceRecord {
        EvidenceRecord {
            schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
            evidence_id: "source-1".to_owned(),
            provenance: ParameterProvenance::EmpiricalDerived,
            source: EvidenceSource {
                source_id: "dataset-1".to_owned(),
                citation: "Example source".to_owned(),
                persistent_id: Some("doi:10.example/example".to_owned()),
                dataset_version: Some("v1".to_owned()),
                licence: Some("example-licence".to_owned()),
                spatial_coverage: Some("example area".to_owned()),
                temporal_coverage: Some("example period".to_owned()),
            },
            original_variable: "measured value".to_owned(),
            original_units: "source units".to_owned(),
            transformation: Some(EvidenceTransformation {
                method: "documented aggregation".to_owned(),
                source_units: Some("source units".to_owned()),
                simulation_units: "simulation units".to_owned(),
                notes: None,
            }),
            simulation_units: "simulation units".to_owned(),
            uncertainty: Some(EvidenceUncertainty {
                representation: "range".to_owned(),
                value: "10-20".to_owned(),
            }),
            applicability: "used only for the declared experiment".to_owned(),
            competing_estimates: vec!["alternative estimate A".to_owned()],
        }
    }

    fn linked(path: &str) -> EvidenceCatalog {
        EvidenceCatalog::new(vec![record()]).with_parameter_links(vec![ParameterEvidenceLink {
            parameter_path: path.to_owned(),
            evidence_id: "source-1".to_owned(),
            note: None,
        }])
    }

    #[test]
    fn valid_catalog_reconciles_parameter_and_external_input_references() {
        let catalog = EvidenceCatalog::new(vec![record()])
            .with_parameter_links(vec![ParameterEvidenceLink {
                parameter_path: "resources.annualNeedUnitsPerPerson".to_owned(),
                evidence_id: "source-1".to_owned(),
                note: None,
            }])
            .with_external_inputs(vec![ExternalInputEvidence {
                input_id: "terrain".to_owned(),
                evidence_id: "source-1".to_owned(),
                format: "GeoTIFF".to_owned(),
                spatial_reference: Some("EPSG:27700".to_owned()),
                content_digest: Some("sha256:example".to_owned()),
            }]);

        assert_eq!(catalog.validate(), Ok(()));
        assert_eq!(
            catalog.validate_against_experiment(&ExperimentConfig::new(1, 1)),
            Ok(())
        );
    }

    #[test]
    fn unknown_evidence_reference_is_rejected() {
        let catalog = EvidenceCatalog::new(vec![record()]).with_parameter_links(vec![
            ParameterEvidenceLink {
                parameter_path: "resources.annualNeedUnitsPerPerson".to_owned(),
                evidence_id: "missing".to_owned(),
                note: None,
            },
        ]);

        assert!(matches!(
            catalog.validate(),
            Err(EvidenceError::UnknownEvidenceReference { evidence_id }) if evidence_id == "missing"
        ));
    }

    #[test]
    fn typo_can_be_catalog_valid_but_is_rejected_against_experiment_schema() {
        let catalog = linked("resources.annualNeedsUnitsPerPerson");
        assert_eq!(catalog.validate(), Ok(()));
        assert!(matches!(
            catalog.validate_against_experiment(&ExperimentConfig::new(2, 1)),
            Err(EvidenceError::UnknownParameterPath { parameter_path, .. })
                if parameter_path == "resources.annualNeedsUnitsPerPerson"
        ));
    }

    #[test]
    fn container_and_collection_paths_are_not_stable_parameter_leaves() {
        assert!(matches!(
            linked("resources").validate_against_experiment(&ExperimentConfig::new(3, 1)),
            Err(EvidenceError::ParameterPathNotLeaf { parameter_path })
                if parameter_path == "resources"
        ));
        assert!(matches!(
            linked("demography.mortalityBands.0.annualProbabilityPerMillion")
                .validate_against_experiment(&ExperimentConfig::new(4, 1)),
            Err(EvidenceError::CollectionParameterPath { parameter_path })
                if parameter_path == "demography.mortalityBands.0.annualProbabilityPerMillion"
        ));
    }

    #[test]
    fn evidence_catalog_cannot_link_to_itself() {
        let experiment =
            ExperimentConfig::new(5, 1).with_evidence(linked("evidence.schemaVersion"));
        let catalog = experiment.evidence.as_ref().unwrap();
        assert!(matches!(
            catalog.validate_against_experiment(&experiment),
            Err(EvidenceError::EvidenceSelfReference { parameter_path })
                if parameter_path == "evidence.schemaVersion"
        ));
    }
}
