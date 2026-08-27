use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    config::{ExperimentConfig, ParameterProvenance},
    evidence::{EvidenceCatalog, EvidenceRecord},
    focal_region::FocalRegionSource,
    landscape::LandscapeBundle,
    spatial_mechanisms::SpatialMechanismConfig,
};

const DEMOGRAPHY_PARAMETER_PATHS: &[&str] = &["demography.scheduleId"];
const FOUNDER_PARAMETER_PATHS: &[&str] = &["founderPopulation.contentDigest64"];

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

const M9_TRAVEL_PARAMETER_PATHS: &[&str] = &[
    "temporaryMobility.travelModel.travelCapacityCostUnitsPerDay",
    "temporaryMobility.travelModel.maximumTraversableMovementCost",
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
    MissingExternalInputSupport,
    MissingEvidenceSupport,
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
    /// v2 extends closure to every provenance-bearing core input currently represented in the
    /// experiment schema (declared founder state and M9 travel assumptions) and assesses only
    /// external inputs that are actually referenced by the run.
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;
}

/// Assess research evidence closure without changing ordinary execution validity.
///
/// This is intentionally stricter than `ExperimentConfig` validation. Synthetic configurations
/// remain executable and return `not_applicable_synthetic`. A mixed experiment can be `closed`
/// when every empirical/evidence-informed claim is supported while its remaining assumptions are
/// explicitly synthetic/null-model inputs.
#[must_use]
pub fn assess_evidence_closure(experiment: &ExperimentConfig) -> EvidenceClosureAssessment {
    let claims = declared_core_claims(experiment);
    let external_inputs = referenced_core_external_inputs(experiment);
    let has_claim = claims
        .iter()
        .any(|(_, provenance)| *provenance != ParameterProvenance::SyntheticValidation)
        || !external_inputs.is_empty();

    if !has_claim {
        return EvidenceClosureAssessment {
            schema_version: EvidenceClosureAssessment::CURRENT_SCHEMA_VERSION,
            status: EvidenceClosureStatus::NotApplicableSynthetic,
            failures: Vec::new(),
        };
    }

    let mut failures = Vec::new();
    for (subject, provenance) in &claims {
        if *provenance == ParameterProvenance::Unresolved {
            push_failure(
                &mut failures,
                *subject,
                EvidenceClosureFailureClass::UnresolvedAssumption,
            );
        }
    }

    let evidence_claims = claims
        .iter()
        .copied()
        .filter(|(_, provenance)| is_evidence_claim(*provenance))
        .collect::<Vec<_>>();
    let needs_catalog = !evidence_claims.is_empty() || !external_inputs.is_empty();

    if needs_catalog {
        match &experiment.evidence {
            None => {
                for (subject, _) in evidence_claims {
                    push_failure(
                        &mut failures,
                        subject,
                        EvidenceClosureFailureClass::MissingCatalog,
                    );
                }
                for (subject, _) in &external_inputs {
                    push_failure(
                        &mut failures,
                        subject,
                        EvidenceClosureFailureClass::MissingCatalog,
                    );
                }
            }
            Some(catalog) => {
                if catalog.validate_against_experiment(experiment).is_err() {
                    push_failure(
                        &mut failures,
                        "evidence",
                        EvidenceClosureFailureClass::InvalidCatalog,
                    );
                } else {
                    assess_empirical_claims(experiment, catalog, &mut failures);
                    for (subject, input_id) in &external_inputs {
                        assess_external_input_reference(catalog, input_id, subject, &mut failures);
                    }
                }
            }
        }
    }

    finish_assessment(true, failures)
}

/// Compose core closure with the evidence claims that are causally used by one spatial run.
///
/// Evidence-bound source layers and transforms are claims. Spatial layers not consumed by a
/// configured transform (or by an M9 landscape-mask region) do not affect this run and therefore do
/// not block it merely because they are present in the bundle. A transform/layer with no evidence
/// reference remains an explicit synthetic/null-model assumption rather than being silently
/// promoted to empirical status.
#[must_use]
pub fn assess_spatial_evidence_closure(
    experiment: &ExperimentConfig,
    landscape: &LandscapeBundle,
    mechanisms: &SpatialMechanismConfig,
) -> EvidenceClosureAssessment {
    let core = assess_evidence_closure(experiment);
    let mut failures = core.failures;
    let mut has_claim = core.status != EvidenceClosureStatus::NotApplicableSynthetic;

    let mut used_layer_ids = mechanisms
        .transforms
        .iter()
        .map(|transform| transform.source_layer_id.as_str())
        .collect::<BTreeSet<_>>();
    if let Some(temporary_mobility) = &experiment.temporary_mobility
        && let FocalRegionSource::LandscapeMask { layer_id, .. } = &temporary_mobility.region.source
    {
        used_layer_ids.insert(layer_id.as_str());
    }

    let mut external_references = Vec::new();
    for layer_id in used_layer_ids {
        let Some(layer) = landscape.layer(layer_id) else {
            continue;
        };
        if let Some(input_id) = layer.evidence_input_id.as_deref() {
            has_claim = true;
            external_references.push((format!("landscape.layers.{layer_id}"), input_id));
        }
    }

    let transform_references = mechanisms
        .transforms
        .iter()
        .filter_map(|transform| {
            transform.evidence_id.as_deref().map(|evidence_id| {
                (
                    format!("spatialTransforms.{:?}", transform.target),
                    evidence_id,
                )
            })
        })
        .collect::<Vec<_>>();
    if !transform_references.is_empty() {
        has_claim = true;
    }

    if !external_references.is_empty() || !transform_references.is_empty() {
        match &experiment.evidence {
            None => {
                for (subject, _) in &external_references {
                    push_failure(
                        &mut failures,
                        subject,
                        EvidenceClosureFailureClass::MissingCatalog,
                    );
                }
                for (subject, _) in &transform_references {
                    push_failure(
                        &mut failures,
                        subject,
                        EvidenceClosureFailureClass::MissingCatalog,
                    );
                }
            }
            Some(catalog) => {
                if catalog.validate_against_experiment(experiment).is_err() {
                    push_failure(
                        &mut failures,
                        "evidence",
                        EvidenceClosureFailureClass::InvalidCatalog,
                    );
                } else {
                    for (subject, input_id) in &external_references {
                        assess_external_input_reference(catalog, input_id, subject, &mut failures);
                    }
                    for (subject, evidence_id) in &transform_references {
                        assess_evidence_record_reference(
                            catalog,
                            evidence_id,
                            subject,
                            &mut failures,
                        );
                    }
                }
            }
        }
    }

    finish_assessment(has_claim, failures)
}

fn declared_core_claims(experiment: &ExperimentConfig) -> Vec<(&'static str, ParameterProvenance)> {
    let mut claims = vec![
        ("demography", experiment.demography.provenance),
        ("resources", experiment.resources.provenance),
        ("migration", experiment.migration.provenance),
    ];
    if let Some(founders) = &experiment.founder_population {
        claims.push(("founderPopulation", founders.provenance));
    }
    if let Some(temporary_mobility) = &experiment.temporary_mobility {
        claims.push((
            "temporaryMobility.travelModel",
            temporary_mobility.travel_model.provenance,
        ));
    }
    claims
}

fn referenced_core_external_inputs(experiment: &ExperimentConfig) -> Vec<(String, String)> {
    let Some(temporary_mobility) = &experiment.temporary_mobility else {
        return Vec::new();
    };
    let FocalRegionSource::LandscapeMask {
        evidence_input_id, ..
    } = &temporary_mobility.region.source
    else {
        return Vec::new();
    };
    vec![(
        "temporaryMobility.region".to_owned(),
        evidence_input_id.clone(),
    )]
}

fn assess_empirical_claims(
    experiment: &ExperimentConfig,
    catalog: &EvidenceCatalog,
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    if is_evidence_claim(experiment.demography.provenance) {
        if experiment.demography.has_content_bound_schedule_id() {
            assess_parameter_paths(
                catalog,
                experiment.demography.provenance,
                DEMOGRAPHY_PARAMETER_PATHS,
                failures,
            );
        } else {
            push_failure(
                failures,
                "demography.scheduleId",
                EvidenceClosureFailureClass::UnsupportedScheduleIdentity,
            );
        }
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

    if let Some(founders) = &experiment.founder_population
        && is_evidence_claim(founders.provenance)
    {
        assess_parameter_paths(
            catalog,
            founders.provenance,
            FOUNDER_PARAMETER_PATHS,
            failures,
        );
    }

    if let Some(temporary_mobility) = &experiment.temporary_mobility
        && is_evidence_claim(temporary_mobility.travel_model.provenance)
    {
        assess_parameter_paths(
            catalog,
            temporary_mobility.travel_model.provenance,
            M9_TRAVEL_PARAMETER_PATHS,
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
            push_failure(
                failures,
                *path,
                EvidenceClosureFailureClass::MissingParameterSupport,
            );
            continue;
        }

        if linked_records
            .iter()
            .all(|record| record.provenance != claim)
        {
            push_failure(
                failures,
                *path,
                EvidenceClosureFailureClass::ProvenanceMismatch,
            );
            continue;
        }

        let compatible = linked_records
            .iter()
            .copied()
            .filter(|record| record.provenance == claim)
            .collect::<Vec<_>>();
        assess_compatible_records(*path, claim, &compatible, failures);
    }
}

fn assess_external_input_reference(
    catalog: &EvidenceCatalog,
    input_id: &str,
    subject: &str,
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    let Some(input) = catalog
        .external_inputs
        .iter()
        .find(|input| input.input_id.trim() == input_id.trim())
    else {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingExternalInputSupport,
        );
        return;
    };

    if !input
        .content_digest
        .as_deref()
        .is_some_and(|digest| !digest.trim().is_empty())
    {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::ExternalInputMissingContentIdentity,
        );
    }

    let Some(record) = catalog
        .records
        .iter()
        .find(|record| record.evidence_id.trim() == input.evidence_id.trim())
    else {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingEvidenceSupport,
        );
        return;
    };
    assess_record_support(subject, record, failures);
}

fn assess_evidence_record_reference(
    catalog: &EvidenceCatalog,
    evidence_id: &str,
    subject: &str,
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    let Some(record) = catalog
        .records
        .iter()
        .find(|record| record.evidence_id.trim() == evidence_id.trim())
    else {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingEvidenceSupport,
        );
        return;
    };
    assess_record_support(subject, record, failures);
}

fn assess_record_support(
    subject: &str,
    record: &EvidenceRecord,
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    if record.provenance == ParameterProvenance::Unresolved {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::UnresolvedAssumption,
        );
        return;
    }
    if !is_evidence_claim(record.provenance) {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::ProvenanceMismatch,
        );
        return;
    }
    if !has_reproducible_source_identity(record) {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingReproducibleSourceIdentity,
        );
    }
    if record.provenance == ParameterProvenance::EmpiricalDerived && record.transformation.is_none()
    {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingDerivedTransformation,
        );
    }
}

fn assess_compatible_records(
    subject: &str,
    claim: ParameterProvenance,
    records: &[&EvidenceRecord],
    failures: &mut Vec<EvidenceClosureFailure>,
) {
    if records
        .iter()
        .all(|record| !has_reproducible_source_identity(record))
    {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingReproducibleSourceIdentity,
        );
        return;
    }

    if claim == ParameterProvenance::EmpiricalDerived
        && records.iter().all(|record| record.transformation.is_none())
    {
        push_failure(
            failures,
            subject,
            EvidenceClosureFailureClass::MissingDerivedTransformation,
        );
    }
}

fn finish_assessment(
    has_claim: bool,
    failures: Vec<EvidenceClosureFailure>,
) -> EvidenceClosureAssessment {
    EvidenceClosureAssessment {
        schema_version: EvidenceClosureAssessment::CURRENT_SCHEMA_VERSION,
        status: if !failures.is_empty() {
            EvidenceClosureStatus::NotClosed
        } else if has_claim {
            EvidenceClosureStatus::Closed
        } else {
            EvidenceClosureStatus::NotApplicableSynthetic
        },
        failures,
    }
}

fn push_failure(
    failures: &mut Vec<EvidenceClosureFailure>,
    subject: impl Into<String>,
    class: EvidenceClosureFailureClass,
) {
    let failure = EvidenceClosureFailure {
        subject: subject.into(),
        class,
    };
    if !failures.contains(&failure) {
        failures.push(failure);
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
    use crate::{
        ExternalInputEvidence, FocalRegion, FounderGenealogyStatus, FounderHousehold,
        FounderPerson, FounderPopulationDefinition, PopulationConfig, ReproductiveSex,
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming,
        evidence::{EvidenceRecord, EvidenceSource, ParameterEvidenceLink},
        ids::{CellId, HouseholdId, PersonId},
    };

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

    fn founder_definition(provenance: ParameterProvenance) -> FounderPopulationDefinition {
        FounderPopulationDefinition::new(
            "founder-evidence-v1",
            provenance,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -9_125,
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -10_950,
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
            ],
        )
    }

    fn temporary_mobility(
        region: FocalRegion,
        travel_provenance: ParameterProvenance,
    ) -> TemporaryMobilityConfig {
        TemporaryMobilityConfig::new(
            region,
            TemporaryMobilitySchedule::new(
                "annual-visit-v1",
                TemporaryTriggerTiming::DepartureDay,
                vec![100],
                5,
            )
            .expect("valid schedule"),
            TemporaryTravelModel::new("travel-v1", travel_provenance, 3_000, u16::MAX)
                .expect("valid travel model"),
        )
        .expect("valid temporary mobility config")
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
    fn unused_external_input_does_not_block_an_otherwise_closed_claim() {
        let mut experiment = ExperimentConfig::default();
        experiment.resources.provenance = ParameterProvenance::EvidenceInformed;
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(RESOURCE_PARAMETER_PATHS))
                .with_external_inputs(vec![ExternalInputEvidence {
                    input_id: "unused-input".to_owned(),
                    evidence_id: "source-1".to_owned(),
                    format: "example".to_owned(),
                    spatial_reference: None,
                    content_digest: None,
                }]),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
    }

    #[test]
    fn empirical_demography_without_content_bound_identity_fails_closed() {
        let mut experiment = ExperimentConfig::default();
        experiment.demography.provenance = ParameterProvenance::EmpiricalDirect;
        experiment.evidence = Some(EvidenceCatalog::new(vec![record(
            ParameterProvenance::EmpiricalDirect,
        )]));

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.subject == "demography.scheduleId"
                && failure.class == EvidenceClosureFailureClass::UnsupportedScheduleIdentity
        }));
    }

    #[test]
    fn fully_linked_content_bound_demography_schedule_can_close() {
        let mut experiment = ExperimentConfig::default();
        experiment.demography.provenance = ParameterProvenance::EvidenceInformed;
        experiment.demography.schedule_id = experiment.demography.content_bound_schedule_id();
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(DEMOGRAPHY_PARAMETER_PATHS)),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
        assert!(assessment.failures.is_empty());
    }

    #[test]
    fn mutating_bound_demography_schedule_invalidates_evidence_closure() {
        let mut experiment = ExperimentConfig::default();
        experiment.demography.provenance = ParameterProvenance::EvidenceInformed;
        experiment.demography.schedule_id = experiment.demography.content_bound_schedule_id();
        experiment.demography.mortality_bands[0].annual_probability_per_million -= 1;
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(DEMOGRAPHY_PARAMETER_PATHS)),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.subject == "demography.scheduleId"
                && failure.class == EvidenceClosureFailureClass::UnsupportedScheduleIdentity
        }));
    }

    #[test]
    fn fully_linked_founder_state_can_close_as_one_content_bound_object() {
        let mut experiment = ExperimentConfig::default()
            .with_population(PopulationConfig::new(2))
            .with_founder_population(founder_definition(ParameterProvenance::EvidenceInformed));
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(FOUNDER_PARAMETER_PATHS)),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
        assert!(assessment.failures.is_empty());
    }

    #[test]
    fn empirical_m9_travel_model_requires_its_two_substantive_parameters() {
        let region = FocalRegion::new(
            "synthetic-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(1)],
        )
        .expect("valid region");
        let mut experiment = ExperimentConfig::default().with_temporary_mobility(
            temporary_mobility(region, ParameterProvenance::EvidenceInformed),
        );
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EvidenceInformed)])
                .with_parameter_links(links(M9_TRAVEL_PARAMETER_PATHS)),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
        assert!(assessment.failures.is_empty());
    }

    #[test]
    fn evidence_bound_m9_mask_is_not_misclassified_as_wholly_synthetic() {
        let region = FocalRegion::new(
            "mask-region",
            FocalRegionSource::LandscapeMask {
                layer_id: "mask".to_owned(),
                evidence_input_id: "mask-input".to_owned(),
            },
            vec![CellId::new(1)],
        )
        .expect("valid region");
        let experiment = ExperimentConfig::default().with_temporary_mobility(temporary_mobility(
            region,
            ParameterProvenance::SyntheticValidation,
        ));

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::NotClosed);
        assert!(assessment.failures.iter().any(|failure| {
            failure.subject == "temporaryMobility.region"
                && failure.class == EvidenceClosureFailureClass::MissingCatalog
        }));
    }

    #[test]
    fn evidence_bound_m9_mask_can_close_with_content_identity() {
        let region = FocalRegion::new(
            "mask-region",
            FocalRegionSource::LandscapeMask {
                layer_id: "mask".to_owned(),
                evidence_input_id: "mask-input".to_owned(),
            },
            vec![CellId::new(1)],
        )
        .expect("valid region");
        let mut experiment = ExperimentConfig::default().with_temporary_mobility(
            temporary_mobility(region, ParameterProvenance::SyntheticValidation),
        );
        experiment.evidence = Some(
            EvidenceCatalog::new(vec![record(ParameterProvenance::EmpiricalDirect)])
                .with_external_inputs(vec![ExternalInputEvidence {
                    input_id: "mask-input".to_owned(),
                    evidence_id: "source-1".to_owned(),
                    format: "landscape-mask-v1".to_owned(),
                    spatial_reference: Some("EPSG:27700".to_owned()),
                    content_digest: Some("sha256:example".to_owned()),
                }]),
        );

        let assessment = assess_evidence_closure(&experiment);
        assert_eq!(assessment.status, EvidenceClosureStatus::Closed);
        assert!(assessment.failures.is_empty());
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
