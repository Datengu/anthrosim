use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Versioned research-governance object for exploratory or confirmatory studies.
///
/// This object is deliberately separate from `ExperimentConfig`: it governs what a study claims,
/// measures, excludes and treats as evidence, but it never changes authoritative simulation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyProtocol {
    pub schema_version: u32,
    pub protocol_revision: u32,
    pub study_id: String,
    pub status: StudyScientificStatus,
    pub research_question: String,
    pub applicability_domain: String,
    pub hypotheses: Vec<StudyHypothesis>,
    pub analysis_windows: Vec<StudyAnalysisWindow>,
    pub observables: Vec<StudyObservable>,
    pub comparisons: Vec<StudyComparison>,
    pub evidence_roles: Vec<StudyEvidenceAssignment>,
    pub uncertainty: StudyUncertaintyPlan,
    pub ensemble_policy: StudyEnsemblePolicy,
    pub run_handling: StudyRunHandling,
    pub sensitivity_plan: Vec<String>,
    pub equifinality_plan: Vec<String>,
    pub manipulation_checks: Vec<StudyManipulationCheck>,
    pub analysis_method: String,
    pub multiplicity_policy: String,
    pub held_out_corroboration: Vec<StudyCorroborationTarget>,
    pub permitted_interpretations: Vec<String>,
    pub prohibited_interpretations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment: Option<StudyProtocolAmendment>,
}

impl StudyProtocol {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn validate(&self) -> Result<(), StudyProtocolError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(StudyProtocolError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.protocol_revision == 0 {
            return Err(StudyProtocolError::ZeroRevision);
        }
        nonempty(&self.study_id, "studyId")?;
        nonempty(&self.research_question, "researchQuestion")?;
        nonempty(&self.applicability_domain, "applicabilityDomain")?;
        nonempty(&self.analysis_method, "analysisMethod")?;
        nonempty(&self.multiplicity_policy, "multiplicityPolicy")?;
        self.uncertainty.validate()?;
        self.ensemble_policy.validate()?;
        self.run_handling.validate()?;
        string_list(&self.sensitivity_plan, "sensitivityPlan")?;
        string_list(&self.equifinality_plan, "equifinalityPlan")?;
        string_list(&self.permitted_interpretations, "permittedInterpretations")?;
        string_list(&self.prohibited_interpretations, "prohibitedInterpretations")?;

        match (&self.amendment, self.protocol_revision) {
            (None, 1) => {}
            (Some(_), 1) => return Err(StudyProtocolError::AmendmentOnInitialRevision),
            (None, _) => return Err(StudyProtocolError::MissingAmendment),
            (Some(amendment), _) => amendment.validate()?,
        }

        let mut hypothesis_ids = BTreeSet::new();
        for hypothesis in &self.hypotheses {
            hypothesis.validate()?;
            if !hypothesis_ids.insert(hypothesis.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "hypothesis",
                    id: hypothesis.id.clone(),
                });
            }
        }

        let mut window_ids = BTreeSet::new();
        for window in &self.analysis_windows {
            window.validate()?;
            if !window_ids.insert(window.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "analysis window",
                    id: window.id.clone(),
                });
            }
        }

        let mut observable_ids = BTreeSet::new();
        let mut primary_observables = BTreeSet::new();
        for observable in &self.observables {
            observable.validate()?;
            if !observable_ids.insert(observable.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "observable",
                    id: observable.id.clone(),
                });
            }
            if !window_ids.contains(observable.analysis_window_id.as_str()) {
                return Err(StudyProtocolError::UnknownAnalysisWindow {
                    observable_id: observable.id.clone(),
                    window_id: observable.analysis_window_id.clone(),
                });
            }
            if observable.role == StudyObservableRole::Primary {
                primary_observables.insert(observable.id.as_str());
            }
        }

        let mut comparison_ids = BTreeSet::new();
        let mut referenced_hypotheses = BTreeSet::new();
        let mut referenced_observables = BTreeSet::new();
        for comparison in &self.comparisons {
            comparison.validate()?;
            if !comparison_ids.insert(comparison.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "comparison",
                    id: comparison.id.clone(),
                });
            }
            for hypothesis_id in &comparison.hypothesis_ids {
                if !hypothesis_ids.contains(hypothesis_id.as_str()) {
                    return Err(StudyProtocolError::UnknownHypothesisReference {
                        comparison_id: comparison.id.clone(),
                        hypothesis_id: hypothesis_id.clone(),
                    });
                }
                referenced_hypotheses.insert(hypothesis_id.as_str());
            }
            for observable_id in &comparison.observable_ids {
                if !observable_ids.contains(observable_id.as_str()) {
                    return Err(StudyProtocolError::UnknownObservableReference {
                        role: "comparison",
                        owner_id: comparison.id.clone(),
                        observable_id: observable_id.clone(),
                    });
                }
                referenced_observables.insert(observable_id.as_str());
            }
        }

        let mut manipulation_ids = BTreeSet::new();
        for check in &self.manipulation_checks {
            check.validate()?;
            if !manipulation_ids.insert(check.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "manipulation check",
                    id: check.id.clone(),
                });
            }
        }

        for (index, assignment) in self.evidence_roles.iter().enumerate() {
            assignment.validate(index)?;
        }

        let mut corroboration_ids = BTreeSet::new();
        for target in &self.held_out_corroboration {
            target.validate()?;
            if !corroboration_ids.insert(target.id.as_str()) {
                return Err(StudyProtocolError::DuplicateId {
                    role: "held-out corroboration target",
                    id: target.id.clone(),
                });
            }
            if !observable_ids.contains(target.observable_id.as_str()) {
                return Err(StudyProtocolError::UnknownObservableReference {
                    role: "held-out corroboration",
                    owner_id: target.id.clone(),
                    observable_id: target.observable_id.clone(),
                });
            }
        }

        if self.status == StudyScientificStatus::Confirmatory {
            if self.hypotheses.len() < 2 {
                return Err(StudyProtocolError::ConfirmatoryNeedsCompetingHypotheses);
            }
            if primary_observables.is_empty() {
                return Err(StudyProtocolError::ConfirmatoryNeedsPrimaryObservable);
            }
            if self.comparisons.is_empty() {
                return Err(StudyProtocolError::ConfirmatoryNeedsComparison);
            }
            for comparison in &self.comparisons {
                if comparison.hypothesis_ids.len() < 2 {
                    return Err(StudyProtocolError::ConfirmatoryComparisonNeedsTwoHypotheses(
                        comparison.id.clone(),
                    ));
                }
                if comparison.observable_ids.is_empty() {
                    return Err(StudyProtocolError::ConfirmatoryComparisonNeedsObservable(
                        comparison.id.clone(),
                    ));
                }
            }
            for hypothesis in &self.hypotheses {
                if !referenced_hypotheses.contains(hypothesis.id.as_str()) {
                    return Err(StudyProtocolError::UncomparedConfirmatoryHypothesis(
                        hypothesis.id.clone(),
                    ));
                }
            }
            for observable in primary_observables {
                if !referenced_observables.contains(observable) {
                    return Err(StudyProtocolError::UnusedPrimaryObservable(observable.to_owned()));
                }
            }
        }

        Ok(())
    }

    pub fn identity(&self) -> Result<String, StudyProtocolError> {
        self.validate()?;
        let encoded = serde_json::to_value(self)
            .map_err(|error| StudyProtocolError::Serialization(error.to_string()))?;
        let canonical = canonical_json_bytes(&encoded)?;
        Ok(format!("study-protocol-v1-{:016x}", fnv1a64(&canonical)))
    }

    /// Whether this exact protocol may be described as a pre-result confirmatory declaration.
    ///
    /// The runner separately records that the protocol was bound before executing the associated
    /// research root. A revision explicitly declared as occurring after prior result inspection
    /// remains reproducible, but must not silently retain a preregistration/predeclaration claim.
    #[must_use]
    pub fn confirmatory_pre_result_claim_eligible(&self) -> bool {
        self.status == StudyScientificStatus::Confirmatory
            && self.amendment.as_ref().is_none_or(|amendment| {
                amendment.timing == StudyAmendmentTiming::BeforeResultInspection
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyScientificStatus {
    Exploratory,
    Confirmatory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyHypothesisKind {
    NullModel,
    Alternative,
    CompetingModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyHypothesis {
    pub id: String,
    pub kind: StudyHypothesisKind,
    pub statement: String,
}

impl StudyHypothesis {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "hypothesis.id")?;
        nonempty(&self.statement, "hypothesis.statement")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyAnalysisWindowSelectionRule {
    PredeclaredFixedDuration,
    ConvergenceDiagnostic,
    ExternallyMeaningfulHistoricalStart,
    InitialStateInScope,
    OtherExplicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyAnalysisWindow {
    pub id: String,
    pub analysis_start_day: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_end_day_inclusive: Option<u64>,
    pub selection_rule: StudyAnalysisWindowSelectionRule,
    pub rationale: String,
}

impl StudyAnalysisWindow {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "analysisWindow.id")?;
        nonempty(&self.rationale, "analysisWindow.rationale")?;
        if self
            .analysis_end_day_inclusive
            .is_some_and(|end| end < self.analysis_start_day)
        {
            return Err(StudyProtocolError::InvalidAnalysisWindow(self.id.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyObservableRole {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyObservable {
    pub id: String,
    pub role: StudyObservableRole,
    pub source: String,
    pub analysis_window_id: String,
    pub interpretation: String,
}

impl StudyObservable {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "observable.id")?;
        nonempty(&self.source, "observable.source")?;
        nonempty(&self.analysis_window_id, "observable.analysisWindowId")?;
        nonempty(&self.interpretation, "observable.interpretation")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyComparison {
    pub id: String,
    pub hypothesis_ids: Vec<String>,
    pub observable_ids: Vec<String>,
    pub prediction: String,
    pub decision_criterion: String,
}

impl StudyComparison {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "comparison.id")?;
        unique_string_list(&self.hypothesis_ids, "comparison.hypothesisIds")?;
        unique_string_list(&self.observable_ids, "comparison.observableIds")?;
        nonempty(&self.prediction, "comparison.prediction")?;
        nonempty(&self.decision_criterion, "comparison.decisionCriterion")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyEvidenceRole {
    ModelConstruction,
    Parameterisation,
    Calibration,
    ModelOutputVerification,
    IndependentCorroboration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyEvidenceAssignment {
    pub evidence_id: String,
    pub role: StudyEvidenceRole,
    pub target: String,
    pub notes: String,
}

impl StudyEvidenceAssignment {
    fn validate(&self, index: usize) -> Result<(), StudyProtocolError> {
        nonempty(&self.evidence_id, &format!("evidenceRoles[{index}].evidenceId"))?;
        nonempty(&self.target, &format!("evidenceRoles[{index}].target"))?;
        nonempty(&self.notes, &format!("evidenceRoles[{index}].notes"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyUncertaintyPlan {
    pub parameter_uncertainty: Vec<String>,
    pub structural_uncertainty: Vec<String>,
}

impl StudyUncertaintyPlan {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        string_list(&self.parameter_uncertainty, "uncertainty.parameterUncertainty")?;
        string_list(&self.structural_uncertainty, "uncertainty.structuralUncertainty")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyEnsemblePolicy {
    pub seed_policy: String,
    pub pairing_policy: String,
    pub replication_policy: String,
}

impl StudyEnsemblePolicy {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.seed_policy, "ensemblePolicy.seedPolicy")?;
        nonempty(&self.pairing_policy, "ensemblePolicy.pairingPolicy")?;
        nonempty(&self.replication_policy, "ensemblePolicy.replicationPolicy")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyRunHandling {
    pub stopping_rules: Vec<String>,
    pub exclusion_rules: Vec<String>,
    pub censoring_rules: Vec<String>,
}

impl StudyRunHandling {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        string_list(&self.stopping_rules, "runHandling.stoppingRules")?;
        string_list(&self.exclusion_rules, "runHandling.exclusionRules")?;
        string_list(&self.censoring_rules, "runHandling.censoringRules")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyManipulationCheck {
    pub id: String,
    pub mechanism: String,
    pub criterion: String,
    pub failure_handling: String,
}

impl StudyManipulationCheck {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "manipulationCheck.id")?;
        nonempty(&self.mechanism, "manipulationCheck.mechanism")?;
        nonempty(&self.criterion, "manipulationCheck.criterion")?;
        nonempty(&self.failure_handling, "manipulationCheck.failureHandling")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyCorroborationTarget {
    pub id: String,
    pub evidence_id: String,
    pub observable_id: String,
    pub criterion: String,
}

impl StudyCorroborationTarget {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(&self.id, "heldOutCorroboration.id")?;
        nonempty(&self.evidence_id, "heldOutCorroboration.evidenceId")?;
        nonempty(&self.observable_id, "heldOutCorroboration.observableId")?;
        nonempty(&self.criterion, "heldOutCorroboration.criterion")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyAmendmentTiming {
    BeforeResultInspection,
    AfterResultInspection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyProtocolAmendment {
    pub previous_protocol_identity: String,
    pub timing: StudyAmendmentTiming,
    pub rationale: String,
}

impl StudyProtocolAmendment {
    fn validate(&self) -> Result<(), StudyProtocolError> {
        nonempty(
            &self.previous_protocol_identity,
            "amendment.previousProtocolIdentity",
        )?;
        nonempty(&self.rationale, "amendment.rationale")
    }
}

fn nonempty(value: &str, field: &str) -> Result<(), StudyProtocolError> {
    if value.trim().is_empty() {
        return Err(StudyProtocolError::EmptyField(field.to_owned()));
    }
    Ok(())
}

fn string_list(values: &[String], field: &str) -> Result<(), StudyProtocolError> {
    for (index, value) in values.iter().enumerate() {
        nonempty(value, &format!("{field}[{index}]"))?;
    }
    Ok(())
}

fn unique_string_list(values: &[String], field: &str) -> Result<(), StudyProtocolError> {
    string_list(values, field)?;
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(StudyProtocolError::DuplicateListValue {
                field: field.to_owned(),
                value: value.clone(),
            });
        }
    }
    Ok(())
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, StudyProtocolError> {
    fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
            Value::Object(values) => {
                let mut keys: Vec<_> = values.keys().collect();
                keys.sort_unstable();
                let mut output = serde_json::Map::new();
                for key in keys {
                    output.insert(key.clone(), canonicalize(&values[key]));
                }
                Value::Object(output)
            }
            _ => value.clone(),
        }
    }
    serde_json::to_vec(&canonicalize(value))
        .map_err(|error| StudyProtocolError::Serialization(error.to_string()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StudyProtocolError {
    #[error("unsupported study-protocol schema {found}; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("study protocol revision must be at least 1")]
    ZeroRevision,
    #[error("study protocol revision 1 must not declare an amendment")]
    AmendmentOnInitialRevision,
    #[error("study protocol revisions after 1 must declare the previous protocol and amendment timing")]
    MissingAmendment,
    #[error("study protocol field {0} must be non-empty")]
    EmptyField(String),
    #[error("duplicate {role} id {id}")]
    DuplicateId { role: &'static str, id: String },
    #[error("duplicate value {value} in study protocol field {field}")]
    DuplicateListValue { field: String, value: String },
    #[error("analysis window {0} ends before it starts")]
    InvalidAnalysisWindow(String),
    #[error("observable {observable_id} references unknown analysis window {window_id}")]
    UnknownAnalysisWindow {
        observable_id: String,
        window_id: String,
    },
    #[error("comparison {comparison_id} references unknown hypothesis {hypothesis_id}")]
    UnknownHypothesisReference {
        comparison_id: String,
        hypothesis_id: String,
    },
    #[error("{role} {owner_id} references unknown observable {observable_id}")]
    UnknownObservableReference {
        role: &'static str,
        owner_id: String,
        observable_id: String,
    },
    #[error("confirmatory study protocol must declare at least two competing hypotheses/models")]
    ConfirmatoryNeedsCompetingHypotheses,
    #[error("confirmatory study protocol must declare at least one primary observable")]
    ConfirmatoryNeedsPrimaryObservable,
    #[error("confirmatory study protocol must declare at least one comparison/decision rule")]
    ConfirmatoryNeedsComparison,
    #[error("confirmatory comparison {0} must reference at least two hypotheses/models")]
    ConfirmatoryComparisonNeedsTwoHypotheses(String),
    #[error("confirmatory comparison {0} must reference at least one observable")]
    ConfirmatoryComparisonNeedsObservable(String),
    #[error("confirmatory hypothesis {0} is not referenced by any comparison")]
    UncomparedConfirmatoryHypothesis(String),
    #[error("primary observable {0} is not referenced by any confirmatory comparison")]
    UnusedPrimaryObservable(String),
    #[error("study-protocol serialization failed: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmatory_protocol() -> StudyProtocol {
        StudyProtocol {
            schema_version: StudyProtocol::CURRENT_SCHEMA_VERSION,
            protocol_revision: 1,
            study_id: "synthetic-resource-study".to_owned(),
            status: StudyScientificStatus::Confirmatory,
            research_question: "Does lower resource productivity increase scarcity mortality?"
                .to_owned(),
            applicability_domain: "Synthetic verification landscape only.".to_owned(),
            hypotheses: vec![
                StudyHypothesis {
                    id: "null".to_owned(),
                    kind: StudyHypothesisKind::NullModel,
                    statement: "Resource productivity does not change scarcity mortality."
                        .to_owned(),
                },
                StudyHypothesis {
                    id: "resource_effect".to_owned(),
                    kind: StudyHypothesisKind::Alternative,
                    statement: "Lower productivity increases scarcity mortality.".to_owned(),
                },
            ],
            analysis_windows: vec![StudyAnalysisWindow {
                id: "primary".to_owned(),
                analysis_start_day: 365,
                analysis_end_day_inclusive: None,
                selection_rule: StudyAnalysisWindowSelectionRule::PredeclaredFixedDuration,
                rationale: "Exclude one fixed initialization year.".to_owned(),
            }],
            observables: vec![StudyObservable {
                id: "scarcity_deaths".to_owned(),
                role: StudyObservableRole::Primary,
                source: "metrics.population.resourceScarcityDeaths".to_owned(),
                analysis_window_id: "primary".to_owned(),
                interpretation: "Higher values indicate greater scarcity mortality.".to_owned(),
            }],
            comparisons: vec![StudyComparison {
                id: "primary_contrast".to_owned(),
                hypothesis_ids: vec!["null".to_owned(), "resource_effect".to_owned()],
                observable_ids: vec!["scarcity_deaths".to_owned()],
                prediction: "Lower productivity has higher scarcity mortality.".to_owned(),
                decision_criterion: "Support resource_effect only if the predeclared contrast is positive."
                    .to_owned(),
            }],
            evidence_roles: vec![],
            uncertainty: StudyUncertaintyPlan {
                parameter_uncertainty: vec!["Sweep declared resource productivity range.".to_owned()],
                structural_uncertainty: vec![],
            },
            ensemble_policy: StudyEnsemblePolicy {
                seed_policy: "Use the exact paired seeds in the bound research definition."
                    .to_owned(),
                pairing_policy: "Compare equal seeds across parameter points.".to_owned(),
                replication_policy: "No adaptive seed addition after result inspection.".to_owned(),
            },
            run_handling: StudyRunHandling {
                stopping_rules: vec!["Use simulator-declared scientific stop reasons.".to_owned()],
                exclusion_rules: vec!["Exclude only predeclared operational failures.".to_owned()],
                censoring_rules: vec!["Report operational censoring separately.".to_owned()],
            },
            sensitivity_plan: vec!["Repeat the contrast over declared uncertainty dimensions."
                .to_owned()],
            equifinality_plan: vec!["Report alternative parameter combinations with equivalent outputs."
                .to_owned()],
            manipulation_checks: vec![StudyManipulationCheck {
                id: "resource_realized".to_owned(),
                mechanism: "M3 resource productivity".to_owned(),
                criterion: "Realized initial/period resource stock differs between treatment arms."
                    .to_owned(),
                failure_handling: "Do not interpret a non-realized treatment as a causal contrast."
                    .to_owned(),
            }],
            analysis_method: "Paired comparison across the exact declared seeds.".to_owned(),
            multiplicity_policy: "One predeclared primary contrast; all others are secondary."
                .to_owned(),
            held_out_corroboration: vec![],
            permitted_interpretations: vec!["Synthetic mechanism comparison.".to_owned()],
            prohibited_interpretations: vec!["Empirical archaeological validation.".to_owned()],
            amendment: None,
        }
    }

    #[test]
    fn stable_identity_changes_when_decision_rule_changes() {
        let protocol = confirmatory_protocol();
        let first = protocol.identity().unwrap();
        assert_eq!(first, protocol.clone().identity().unwrap());

        let mut changed = protocol;
        changed.comparisons[0].decision_criterion.push_str(" Require two-sided robustness.");
        assert_ne!(first, changed.identity().unwrap());
    }

    #[test]
    fn confirmatory_protocol_requires_competing_hypotheses_and_primary_outcome() {
        let mut one_hypothesis = confirmatory_protocol();
        one_hypothesis.hypotheses.pop();
        one_hypothesis.comparisons[0].hypothesis_ids.pop();
        assert_eq!(
            one_hypothesis.validate(),
            Err(StudyProtocolError::ConfirmatoryNeedsCompetingHypotheses)
        );

        let mut no_primary = confirmatory_protocol();
        no_primary.observables[0].role = StudyObservableRole::Secondary;
        assert_eq!(
            no_primary.validate(),
            Err(StudyProtocolError::ConfirmatoryNeedsPrimaryObservable)
        );
    }

    #[test]
    fn references_fail_closed() {
        let mut bad_window = confirmatory_protocol();
        bad_window.observables[0].analysis_window_id = "missing".to_owned();
        assert!(matches!(
            bad_window.validate(),
            Err(StudyProtocolError::UnknownAnalysisWindow { .. })
        ));

        let mut bad_hypothesis = confirmatory_protocol();
        bad_hypothesis.comparisons[0].hypothesis_ids[1] = "missing".to_owned();
        assert!(matches!(
            bad_hypothesis.validate(),
            Err(StudyProtocolError::UnknownHypothesisReference { .. })
        ));
    }

    #[test]
    fn amendments_create_visible_revision_and_post_result_status() {
        let original = confirmatory_protocol();
        let original_identity = original.identity().unwrap();
        let mut amended = original;
        amended.protocol_revision = 2;
        amended.analysis_method = "Revised paired comparison after inspecting prior outputs.".to_owned();
        amended.amendment = Some(StudyProtocolAmendment {
            previous_protocol_identity: original_identity.clone(),
            timing: StudyAmendmentTiming::AfterResultInspection,
            rationale: "The original analysis exposed an unplanned diagnostic requirement."
                .to_owned(),
        });
        let amended_identity = amended.identity().unwrap();
        assert_ne!(original_identity, amended_identity);
        assert!(!amended.confirmatory_pre_result_claim_eligible());

        amended.amendment.as_mut().unwrap().timing = StudyAmendmentTiming::BeforeResultInspection;
        assert!(amended.confirmatory_pre_result_claim_eligible());
    }

    #[test]
    fn later_revision_without_amendment_is_rejected() {
        let mut protocol = confirmatory_protocol();
        protocol.protocol_revision = 2;
        assert_eq!(protocol.validate(), Err(StudyProtocolError::MissingAmendment));
    }
}
