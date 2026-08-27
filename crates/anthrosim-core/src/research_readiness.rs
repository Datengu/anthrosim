use serde::{Deserialize, Serialize};

use crate::{
    config::{ExperimentConfig, ParameterProvenance},
    evidence::{EvidenceCatalog, EvidenceRecord},
};

const RESOURCE_PARAMETER_PATHS: &[&str] = &[
    "resources.periodsPerYear",
    "resources.annualNeedUnitsPerPerson",
    "resources.annualRegenerationUnitsPerProductivity",
    "resources.productivityScalePermille",
    "resources.seasonalityScalePermille",
    "resources.cellStockCapacityYears",
    "resources.conditionRecoveryPerPeriod",
    "resources.maxConditionLossPerPeriod",
    "resources.maxConditionMortalityProbabilityPerMillion",
];

const MIGRATION_PARAMETER_PATHS: &[&str] = &[
    "migration.enabled",
    "migration.decisionPeriodsPerYear",
    "migration.candidateRadiusCells",
    "migration.conditionPressureThresholdPermille",
    "migration.resourcePressureThresholdPermille",
    "migration.minimumUtilityImprovement",
    "migration.resourceWeight",
    "migration.waterSecurityWeight",
    "migration.kinWeight",
    "migration.travelCostWeight",
    "migration.maxUncertaintyPenaltyPermille",
    "migration.relocationRiskBasePenaltyPermille",
    "migration.relocationRiskPerCellPermille",
    "migration.travelConditionCostPerCell",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClosureStatus {
    Closed,
    NotClosed,
    NotApplicableSynthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClosureFailureClass {
    MissingCatalog,
    InvalidCatalog,
    UnresolvedAssumption,
    MissingParameterSupport,
    ProvenanceMismatch,
    MissingReproducibleSourceIdentity,
    MissingDerivedTransformation,
    UnsupportedScheduleIdentity,
    ExternalInputMissingContentIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceClosureFailure {
    pub subject: String,
    pub class: EvidenceClosureFailureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceClosureAssessment {
    pub schema_version: u32,
    pub status: EvidenceClosureStatus,
    pub failures: Vec<EvidenceClosureFailure>,
}

impl EvidenceClosureAssessment {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

/// Assess research evidence closure without changing ordinary execution validity.
///
/// This is intentionally stricter than `ExperimentConfig` validation. Synthetic
/// configurations remain executable and return `not_applicable_synthetic`.
/// Empirical/evidence-informed claims return `closed` only where the current
/// schema can prove complete scalar support. Empirical demographic schedules
/// remain fail-closed until their collection payload has a stable, content-bound
/// scientific identity rather than positional array addressing.
#[must_use]
pub fn assess_evidence_closure(experiment: &ExperimentConfig) -> EvidenceClosureAssessment {
    let claims = [
        ("demography", experiment.demography.provenance),
        ("resources", experiment.resources.provenance),
        ("migration", experiment.migration.provenance),
    ];

    if claims
        .iter()
        .all(|(_, provenance)| *provenance == ParameterProvenance::SyntheticValidation)
    {
        return EvidenceClosureAssessment {
            schema_version: EvidenceClosureAssessment::CURRENT_SCHEMA_VERSION,
            status: EvidenceClosureStatus::NotApplicableSynthetic,
            failures: Vec::new(),
        };
    }

    let mut failures = Vec::new();
    for (subject, provenance) in claims {
        if provenance == ParameterProvenance::Unresolved {
            failures.push(EvidenceClosureFailure {
                subject: subject.to_owned(),
                class: EvidenceClosureFailureClass::UnresolvedAssumption,
            });
        }
    }

    let empirical_claims = claims
        .iter()
        .copied()
        .filter(|(_, provenance)| is_evidence_claim(*provenance))
        .collect::<Vec<_>>();

    if !empirical_claims.is_empty() {
        match &experiment.evidence {
            None => {
                for (subject, _) in empirical_claims {
                    failures.push(EvidenceClosureFailure {
                        subject: subject.to_owned(),
                        class: EvidenceClosureFailureClass::MissingCatalog,
                    });
                }
            }
            Some(catalog) => {
                if catalog.validate_against_experiment(experiment).is_err() {
                    failures.push(EvidenceClosureFailure {
                        subject: "evidence".to_owned(),
                        class: EvidenceClosureFailureClass::InvalidCatalog,
                    });
                } else {
                    assess_empirical_claims(experiment, catalog, &mut failures);
                    assess_external_inputs(catalog, &mut failures);
                }
            }
        }
    }

    EvidenceClosureAssessment {
        schema_version: EvidenceClosureAssessment::CURRENT_SCHEMA_VERSION,
        status: if failures.is_empty() {
            EvidenceClosureStatus::Closed
        } else {
            EvidenceClosureStatus::NotClosed
        },
        failures,
    }
}

fn assess_empirical_claims(
    experiment: &ExperimentConfig,
    catalog: &EvidenceCatalog,
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    if is_evidence_claim(experiment.demography.provenance) {
        failures.push(EvidenceClosureFailure {
            subject: "demography.mortalityBands/fertilityBands".to_owned(),
            class: EvidenceClosureFailureClass::UnsupportedScheduleIdentity,
        });
    }

    if is_evidence_claim(experiment.resources.provenance) {
        assess_parameter_paths(
            catalog,
            experiment.resources.provenance,
            RESOURCE_PARAMETER_PATHS,
            failures,
        );
    }

    if is_evidence_claim(experiment.migration.provenance) {
        assess_parameter_paths(
            catalog,
            experiment.migration.provenance,
            MIGRATION_PARAMETER_PATHS,
            failures,
        );
    }
}

fn assess_parameter_paths(
    catalog: &EvidenceCatalog,
    claim: ParameterProvenance,
    paths: &[&str],
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    for path in paths {
        let linked_records = catalog
            .parameter_links
            .iter()
            .filter(|link| link.parameter_path.trim() == *path)
            .filter_map(|link| {
                catalog
                    .records
                    .iter()
                    .find(|record| record.evidence_id.trim() == link.evidence_id.trim())
            })
            .collect::<Vec<_>>();

        if linked_records.is_empty() {
            failures.push(EvidenceClosureFailure {
                subject: (*path).to_owned(),
                class: EvidenceClosureFailureClass::MissingParameterSupport,
            });
            continue;
        }

        if linked_records.iter().all(|record| record.provenance != claim) {
            failures.push(EvidenceClosureFailure {
                subject: (*path).to_owned(),
                class: EvidenceClosureFailureClass::ProvenanceMismatch,
            });
            continue;
        }

        let compatible = linked_records
            .iter()
            .copied()
            .filter(|record| record.provenance == claim)
            .collect::<Vec<_>>();

        if compatible
            .iter()
            .all(|record| !has_reproducible_source_identity(record))
        {
            failures.push(EvidenceClosureFailure {
                subject: (*path).to_owned(),
                class: EvidenceClosureFailureClass::MissingReproducibleSourceIdentity,
            });
            continue;
        }

        if claim == ParameterProvenance::EmpiricalDerived
            && compatible.iter().all(|record| record.transformation.is_none())
        {
            failures.push(EvidenceClosureFailure {
                subject: (*path).to_owned(),
                class: EvidenceClosureFailureClass::MissingDerivedTransformation,
            });
        }
    }
}

fn assess_external_inputs(catalog: &EvidenceCatalog, failures: &mut Vec<EvidenceClosureFailure>) {
    for input in &catalog.external_inputs {
        let has_digest = input
            .content_digest
            .as_deref()
            .is_some_and(|digest| !digest.trim().is_empty());
        if !has_digest {
            failures.push(EvidenceClosureFailure {
                subject: format!("externalInputs.{}", input.input_id),
                class: EvidenceClosureFailureClass::ExternalInputMissingContentIdentity,
            });
        }
    }
}

const fn is_evidence_claim(provenance: ParameterProvenance) -> bool {
    matches!(
        provenance,
        ParameterProvenance::EmpiricalDirect
            | ParameterProvenance::EmpiricalDerived
            | ParameterProvenance::EvidenceInformed
    )
}

fn has_reproducible_source_identity(record: &EvidenceRecord) -> bool {
    record
        .source
        .persistent_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || record
            .source
            .dataset_version
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{EvidenceRecord, EvidenceSource, ParameterEvidenceLink};

    fn record(provenance: ParameterProvenance) -> EvidenceRecord {
        EvidenceRecord {
            schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
            evidence_id: "source-1".to_owned(),
            provenance,
            source: EvidenceSource {
                source_id: "dataset-1".to_owned(),
                citation: "Example source".to_owned(),
                persistent_id: Some("doi:10.example/example".to_owned()),
                dataset_version: None,
                licence: None,
                spatial_coverage: None,
                temporal_coverage: None,
            },
            original_variable: "measured value".to_owned(),
            original_units: "source units".to_owned(),
            transformation: None,
            simulation_units: "simulation units".to_owned(),
            uncertainty: None,
            applicability: "example".to_owned(),
            competing_estimates: Vec::new(),
        }
    }

    fn links(paths: &[&str]) -> Vec<ParameterEvidenceLink> {
        paths
            .iter()
            .map(|path| ParameterEvidenceLink {
                parameter_path: (*path).to_owned(),
                evidence_id: "source-1".to_owned(),
                note: None,
            })
            .collect()
    }

    #[test]
    fn synthetic_experiment_is_explicitly_not_applicable() {
        let assessment = assess_evidence_closure(&ExperimentConfig::default());
        assert_eq!(
            assessment.status,
            EvidenceClosureStatus::NotApplicableSynthetic
        );
        assert!(assessment.failures.is_empty());
    }

    #[test]
    fn empirical_claim_without_catalog_is_not_closed() {
        let mut experiment = ExperimentConfig::default();
        experiment.resources.provenance = ParameterProvenance::EmpiricalDirect;

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.subject == "resources"
                && failure.class == EvidenceClosureFailureClass::MissingCatalog
        }));
    }

    #[test]
    fn unrelated_catalog_does_not_close_resource_claim() {
        let mut experiment = ExperimentConfig::default();
        experiment.resources.provenance = ParameterProvenance::EvidenceInformed;
        experiment.evidence = Some(EvidenceCatalog::new(vec![record(
            ParameterProvenance::EvidenceInformed,
        )]));

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.class == EvidenceClosureFailureClass::MissingParameterSupport
        }));
    }

    #[test]
    fn fully_linked_resource_claim_can_close() {
        let mut experiment = ExperimentConfig::default();
        experiment.resources.provenance = ParameterProvenance::EvidenceInformed;
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(RESOURCE_PARAMETER_PATHS)),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
        assert!(assessment.failures.is_empty());
    }

    #[test]
    fn empirical_demography_fails_closed_until_schedule_identity_exists() {
        let mut experiment = ExperimentConfig::default();
        experiment.demography.provenance = ParameterProvenance::EmpiricalDirect;
        experiment.evidence = Some(EvidenceCatalog::new(vec![record(
            ParameterProvenance::EmpiricalDirect,
        )]));

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.class == EvidenceClosureFailureClass::UnsupportedScheduleIdentity
        }));
    }

    #[test]
    fn unresolved_assumption_blocks_research_closure() {
        let mut experiment = ExperimentConfig::default();
        experiment.resources.provenance = ParameterProvenance::Unresolved;

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.subject == "resources"
                && failure.class == EvidenceClosureFailureClass::UnresolvedAssumption
        }));
    }
}
