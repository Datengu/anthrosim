use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{MigrationConfig, ParameterProvenance},
    evidence::{EvidenceCatalog, EvidenceError},
    ids::CellId,
    landscape::{GridGeometry, LandscapeBundle, LandscapeError},
    migration::{bounded_candidate_cells, candidate_count_upper_bound, validate_migration_config},
    world::World,
};

/// Executable spatial boundary semantics currently supported by AnthroSim.
///
/// The finite rectangular `World` is a closed graph: M4 never sees candidate cells beyond the
/// grid and M9 cannot route through space outside the grid before re-entering it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialBoundarySemantics {
    ClosedFiniteGridV1,
}

/// Scientific interpretation of the closed simulation edge.
///
/// The executable boundary is always closed in the current model. This declaration says whether
/// that closed edge is still unresolved, is known to be an analyst-selected crop that requires
/// extent sensitivity, or is intentionally being used as an evidence-supported closed barrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialBoundaryInterpretation {
    UnresolvedExtent,
    AnalystDefinedCrop,
    DeclaredClosedBarrier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialBoundaryDeclaration {
    pub schema_version: u32,
    pub declaration_id: String,
    pub interpretation: SpatialBoundaryInterpretation,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence_ids: Vec<String>,
}

impl SpatialBoundaryDeclaration {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn unresolved() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            declaration_id: "unresolved-spatial-extent".to_owned(),
            interpretation: SpatialBoundaryInterpretation::UnresolvedExtent,
            rationale: "The finite raster edge has not been justified as a historical barrier."
                .to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    #[must_use]
    pub fn analyst_defined_crop(
        declaration_id: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            declaration_id: declaration_id.into(),
            interpretation: SpatialBoundaryInterpretation::AnalystDefinedCrop,
            rationale: rationale.into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    #[must_use]
    pub fn declared_closed_barrier(
        declaration_id: impl Into<String>,
        rationale: impl Into<String>,
        supporting_evidence_ids: Vec<String>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            declaration_id: declaration_id.into(),
            interpretation: SpatialBoundaryInterpretation::DeclaredClosedBarrier,
            rationale: rationale.into(),
            supporting_evidence_ids,
        }
    }

    pub fn validate(&self, evidence: Option<&EvidenceCatalog>) -> Result<(), SpatialBoundaryError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(SpatialBoundaryError::UnsupportedDeclarationSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.declaration_id.trim().is_empty() {
            return Err(SpatialBoundaryError::EmptyDeclarationId);
        }
        if self.rationale.trim().is_empty() {
            return Err(SpatialBoundaryError::EmptyRationale);
        }

        let mut evidence_ids = BTreeSet::new();
        for evidence_id in &self.supporting_evidence_ids {
            if evidence_id.trim().is_empty() {
                return Err(SpatialBoundaryError::EmptySupportingEvidenceId);
            }
            if !evidence_ids.insert(evidence_id.as_str()) {
                return Err(SpatialBoundaryError::DuplicateSupportingEvidenceId(
                    evidence_id.clone(),
                ));
            }
        }

        match self.interpretation {
            SpatialBoundaryInterpretation::UnresolvedExtent
            | SpatialBoundaryInterpretation::AnalystDefinedCrop => {
                if !self.supporting_evidence_ids.is_empty() {
                    return Err(SpatialBoundaryError::EvidenceOnNonBarrierDeclaration);
                }
            }
            SpatialBoundaryInterpretation::DeclaredClosedBarrier => {
                if self.supporting_evidence_ids.is_empty() {
                    return Err(SpatialBoundaryError::BarrierWithoutEvidence);
                }
                let evidence = evidence.ok_or(SpatialBoundaryError::MissingEvidenceCatalog)?;
                evidence.validate()?;
                for evidence_id in &self.supporting_evidence_ids {
                    let record = evidence
                        .records
                        .iter()
                        .find(|record| record.evidence_id == *evidence_id)
                        .ok_or_else(|| {
                            SpatialBoundaryError::UnknownSupportingEvidence(evidence_id.clone())
                        })?;
                    if matches!(record.provenance, ParameterProvenance::Unresolved) {
                        return Err(SpatialBoundaryError::UnresolvedBarrierEvidence(
                            evidence_id.clone(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub const fn requires_extent_sensitivity(&self) -> bool {
        !matches!(
            self.interpretation,
            SpatialBoundaryInterpretation::DeclaredClosedBarrier
        )
    }
}

/// CRS-space rectangular reporting/analysis domain inside the larger simulation domain.
///
/// Boundaries must align exactly to whole landscape cells. This keeps membership deterministic and
/// lets the same physical inner domain be embedded in successively larger simulation buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialAnalysisExtent {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialAnalysisDomain {
    pub schema_version: u32,
    pub domain_id: String,
    pub extent: SpatialAnalysisExtent,
}

impl SpatialAnalysisDomain {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn new(domain_id: impl Into<String>, extent: SpatialAnalysisExtent) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            domain_id: domain_id.into(),
            extent,
        }
    }

    pub fn whole_landscape(
        domain_id: impl Into<String>,
        landscape: &LandscapeBundle,
    ) -> Result<Self, SpatialBoundaryError> {
        Ok(Self::new(domain_id, simulation_extent(landscape)?))
    }

    fn resolve(
        &self,
        landscape: &LandscapeBundle,
    ) -> Result<ResolvedAnalysisDomain, SpatialBoundaryError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(SpatialBoundaryError::UnsupportedAnalysisDomainSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.domain_id.trim().is_empty() {
            return Err(SpatialBoundaryError::EmptyAnalysisDomainId);
        }
        if self.extent.min_x >= self.extent.max_x || self.extent.min_y >= self.extent.max_y {
            return Err(SpatialBoundaryError::InvalidAnalysisExtent);
        }

        let simulation = simulation_extent(landscape)?;
        if self.extent.min_x < simulation.min_x
            || self.extent.max_x > simulation.max_x
            || self.extent.min_y < simulation.min_y
            || self.extent.max_y > simulation.max_y
        {
            return Err(SpatialBoundaryError::AnalysisDomainOutsideSimulation);
        }

        let size_x = i128::from(landscape.geometry.cell_size_x);
        let size_y = i128::from(landscape.geometry.cell_size_y);
        let simulation_min_x = i128::from(simulation.min_x);
        let simulation_max_y = i128::from(simulation.max_y);
        let min_x = i128::from(self.extent.min_x);
        let max_x = i128::from(self.extent.max_x);
        let min_y = i128::from(self.extent.min_y);
        let max_y = i128::from(self.extent.max_y);

        let left = min_x - simulation_min_x;
        let right_edge = max_x - simulation_min_x;
        let top = simulation_max_y - max_y;
        let bottom_edge = simulation_max_y - min_y;
        if left % size_x != 0
            || right_edge % size_x != 0
            || top % size_y != 0
            || bottom_edge % size_y != 0
        {
            return Err(SpatialBoundaryError::AnalysisDomainNotCellAligned);
        }

        let min_grid_x = u32::try_from(left / size_x)
            .map_err(|_| SpatialBoundaryError::AnalysisDomainIndexOverflow)?;
        let max_grid_x_exclusive = u32::try_from(right_edge / size_x)
            .map_err(|_| SpatialBoundaryError::AnalysisDomainIndexOverflow)?;
        let min_grid_y = u32::try_from(top / size_y)
            .map_err(|_| SpatialBoundaryError::AnalysisDomainIndexOverflow)?;
        let max_grid_y_exclusive = u32::try_from(bottom_edge / size_y)
            .map_err(|_| SpatialBoundaryError::AnalysisDomainIndexOverflow)?;
        if min_grid_x >= max_grid_x_exclusive || min_grid_y >= max_grid_y_exclusive {
            return Err(SpatialBoundaryError::InvalidAnalysisExtent);
        }

        Ok(ResolvedAnalysisDomain {
            min_grid_x,
            max_grid_x_exclusive,
            min_grid_y,
            max_grid_y_exclusive,
        })
    }

    pub fn member_cells(
        &self,
        landscape: &LandscapeBundle,
    ) -> Result<Vec<CellId>, SpatialBoundaryError> {
        let resolved = self.resolve(landscape)?;
        let width = u64::from(landscape.width);
        let mut cells = Vec::with_capacity(resolved.cell_count()?);
        for y in resolved.min_grid_y..resolved.max_grid_y_exclusive {
            for x in resolved.min_grid_x..resolved.max_grid_x_exclusive {
                let zero_based = u64::from(y)
                    .checked_mul(width)
                    .and_then(|row| row.checked_add(u64::from(x)))
                    .ok_or(SpatialBoundaryError::AnalysisDomainIndexOverflow)?;
                cells.push(CellId::new(
                    zero_based
                        .checked_add(1)
                        .ok_or(SpatialBoundaryError::AnalysisDomainIndexOverflow)?,
                ));
            }
        }
        Ok(cells)
    }

    pub fn minimum_buffer_cells(
        &self,
        landscape: &LandscapeBundle,
    ) -> Result<u32, SpatialBoundaryError> {
        let resolved = self.resolve(landscape)?;
        Ok(resolved
            .min_grid_x
            .min(landscape.width - resolved.max_grid_x_exclusive)
            .min(resolved.min_grid_y)
            .min(landscape.height - resolved.max_grid_y_exclusive))
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedAnalysisDomain {
    min_grid_x: u32,
    max_grid_x_exclusive: u32,
    min_grid_y: u32,
    max_grid_y_exclusive: u32,
}

impl ResolvedAnalysisDomain {
    fn cell_count(self) -> Result<usize, SpatialBoundaryError> {
        let width = u64::from(self.max_grid_x_exclusive - self.min_grid_x);
        let height = u64::from(self.max_grid_y_exclusive - self.min_grid_y);
        usize::try_from(width.saturating_mul(height))
            .map_err(|_| SpatialBoundaryError::AnalysisDomainIndexOverflow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialBoundaryCellAssessment {
    pub cell: CellId,
    pub grid_x: u32,
    pub grid_y: u32,
    pub distance_to_simulation_boundary_cells: u32,
    pub m4_candidate_count: u32,
    pub m4_full_interior_candidate_count: u32,
    pub m4_missing_candidate_count: u32,
    pub m4_candidate_set_truncated: bool,
}

/// Derived, run-independent assessment of how the finite simulation edge can affect a declared
/// inner analysis domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialBoundaryAssessment {
    pub schema_version: u32,
    pub landscape_identity: String,
    pub landscape_digest64: u64,
    pub geometry: GridGeometry,
    pub simulation_width: u32,
    pub simulation_height: u32,
    pub semantics: SpatialBoundarySemantics,
    pub declaration: SpatialBoundaryDeclaration,
    pub analysis_domain: SpatialAnalysisDomain,
    pub analysis_cell_count: u64,
    pub minimum_analysis_buffer_cells: u32,
    pub m4_enabled: bool,
    pub m4_candidate_radius_cells: u16,
    pub m4_full_interior_candidate_count: u32,
    pub analysis_cells_with_truncated_m4_candidates: u64,
    pub m4_analysis_horizon_clear_of_boundary: bool,
    pub m9_routes_confined_to_simulation_domain: bool,
    pub m9_routes_may_leave_and_reenter_simulation_domain: bool,
    pub requires_extent_sensitivity: bool,
    pub requires_study_specific_convergence_criterion: bool,
    pub cells: Vec<SpatialBoundaryCellAssessment>,
}

impl SpatialBoundaryAssessment {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
    pub const CURRENT_SEMANTICS: SpatialBoundarySemantics =
        SpatialBoundarySemantics::ClosedFiniteGridV1;
}

pub fn assess_spatial_boundary(
    landscape: &LandscapeBundle,
    world: &World,
    migration: &MigrationConfig,
    declaration: SpatialBoundaryDeclaration,
    analysis_domain: SpatialAnalysisDomain,
    evidence: Option<&EvidenceCatalog>,
) -> Result<SpatialBoundaryAssessment, SpatialBoundaryError> {
    landscape.validate()?;
    validate_migration_config(migration)
        .map_err(|error| SpatialBoundaryError::InvalidMigrationConfig(error.to_string()))?;
    declaration.validate(evidence)?;
    if landscape.width != world.width() || landscape.height != world.height() {
        return Err(SpatialBoundaryError::GridMismatch);
    }

    let member_cells = analysis_domain.member_cells(landscape)?;
    let minimum_analysis_buffer_cells = analysis_domain.minimum_buffer_cells(landscape)?;
    let full_candidate_count = candidate_count_upper_bound(migration.candidate_radius_cells);
    let full_candidate_count_u32 = u32::try_from(full_candidate_count)
        .map_err(|_| SpatialBoundaryError::CandidateCountOverflow)?;
    let mut truncated_cells = 0_u64;
    let mut cells = Vec::with_capacity(member_cells.len());

    for cell in member_cells {
        let (grid_x, grid_y) = world
            .coordinates(cell)
            .ok_or(SpatialBoundaryError::InvalidCell(cell))?;
        let distance_to_boundary = grid_x
            .min(world.width() - 1 - grid_x)
            .min(grid_y)
            .min(world.height() - 1 - grid_y);
        let actual_candidate_count =
            bounded_candidate_cells(world, cell, migration.candidate_radius_cells).len();
        let missing_candidate_count = full_candidate_count.saturating_sub(actual_candidate_count);
        let truncated = missing_candidate_count > 0;
        if truncated {
            truncated_cells = truncated_cells
                .checked_add(1)
                .ok_or(SpatialBoundaryError::AccountingOverflow)?;
        }
        cells.push(SpatialBoundaryCellAssessment {
            cell,
            grid_x,
            grid_y,
            distance_to_simulation_boundary_cells: distance_to_boundary,
            m4_candidate_count: u32::try_from(actual_candidate_count)
                .map_err(|_| SpatialBoundaryError::CandidateCountOverflow)?,
            m4_full_interior_candidate_count: full_candidate_count_u32,
            m4_missing_candidate_count: u32::try_from(missing_candidate_count)
                .map_err(|_| SpatialBoundaryError::CandidateCountOverflow)?,
            m4_candidate_set_truncated: truncated,
        });
    }

    let requires_extent_sensitivity = declaration.requires_extent_sensitivity();
    Ok(SpatialBoundaryAssessment {
        schema_version: SpatialBoundaryAssessment::CURRENT_SCHEMA_VERSION,
        landscape_identity: landscape.identity(),
        landscape_digest64: landscape.digest64(),
        geometry: landscape.geometry.clone(),
        simulation_width: landscape.width,
        simulation_height: landscape.height,
        semantics: SpatialBoundaryAssessment::CURRENT_SEMANTICS,
        declaration,
        analysis_domain,
        analysis_cell_count: u64::try_from(cells.len())
            .map_err(|_| SpatialBoundaryError::AccountingOverflow)?,
        minimum_analysis_buffer_cells,
        m4_enabled: migration.enabled,
        m4_candidate_radius_cells: migration.candidate_radius_cells,
        m4_full_interior_candidate_count: full_candidate_count_u32,
        analysis_cells_with_truncated_m4_candidates: truncated_cells,
        m4_analysis_horizon_clear_of_boundary: truncated_cells == 0,
        m9_routes_confined_to_simulation_domain: true,
        m9_routes_may_leave_and_reenter_simulation_domain: false,
        requires_extent_sensitivity,
        requires_study_specific_convergence_criterion: requires_extent_sensitivity,
        cells,
    })
}

/// Versioned declaration of what a study will accept as sufficient extent/buffer convergence.
///
/// The engine deliberately does not supply universal tolerances. Each study must name the metrics
/// relevant to its claim and predeclare an acceptable absolute and/or relative difference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentMetricTolerance {
    pub metric_id: String,
    pub max_absolute_difference: Option<u64>,
    pub max_relative_difference_permille: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentAdequacyCriterion {
    pub schema_version: u32,
    pub criterion_id: String,
    pub required_consecutive_stable_extensions: u16,
    pub minimum_buffer_cells: Option<u32>,
    pub metric_tolerances: Vec<SpatialExtentMetricTolerance>,
}

impl SpatialExtentAdequacyCriterion {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn validate(&self) -> Result<(), SpatialBoundaryError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(SpatialBoundaryError::UnsupportedAdequacyCriterionSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.criterion_id.trim().is_empty() {
            return Err(SpatialBoundaryError::EmptyAdequacyCriterionId);
        }
        if self.required_consecutive_stable_extensions == 0 {
            return Err(SpatialBoundaryError::ZeroStableExtensionRequirement);
        }
        if self.metric_tolerances.is_empty() {
            return Err(SpatialBoundaryError::NoExtentMetrics);
        }
        let mut metric_ids = BTreeSet::new();
        for tolerance in &self.metric_tolerances {
            if tolerance.metric_id.trim().is_empty() {
                return Err(SpatialBoundaryError::EmptyExtentMetricId);
            }
            if !metric_ids.insert(tolerance.metric_id.as_str()) {
                return Err(SpatialBoundaryError::DuplicateExtentMetricId(
                    tolerance.metric_id.clone(),
                ));
            }
            if tolerance.max_absolute_difference.is_none()
                && tolerance.max_relative_difference_permille.is_none()
            {
                return Err(SpatialBoundaryError::ExtentMetricWithoutTolerance(
                    tolerance.metric_id.clone(),
                ));
            }
            if tolerance
                .max_relative_difference_permille
                .is_some_and(|value| value > 1_000)
            {
                return Err(SpatialBoundaryError::RelativeToleranceOutOfRange {
                    metric_id: tolerance.metric_id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentMetricObservation {
    pub metric_id: String,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentObservation {
    pub buffer_cells: u32,
    pub metrics: Vec<SpatialExtentMetricObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentMetricComparison {
    pub metric_id: String,
    pub previous_value: u64,
    pub current_value: u64,
    pub absolute_difference: u64,
    pub relative_difference_permille: u16,
    pub absolute_within_tolerance: Option<bool>,
    pub relative_within_tolerance: Option<bool>,
    pub within_tolerance: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentExtensionComparison {
    pub previous_buffer_cells: u32,
    pub current_buffer_cells: u32,
    pub eligible_for_stability_sequence: bool,
    pub all_metrics_within_tolerance: bool,
    pub metrics: Vec<SpatialExtentMetricComparison>,
}

/// Machine-readable evaluation of a predeclared extent-adequacy criterion.
///
/// This compares adjacent enlargements of one fixed physical analysis domain. `adequate` is true
/// only when the required number of *trailing* eligible enlargements all satisfy every declared
/// metric tolerance. It does not imply that unmeasured outputs are boundary-insensitive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialExtentConvergenceAssessment {
    pub schema_version: u32,
    pub criterion: SpatialExtentAdequacyCriterion,
    pub observations: Vec<SpatialExtentObservation>,
    pub comparisons: Vec<SpatialExtentExtensionComparison>,
    pub trailing_stable_extensions: u16,
    pub latest_extension_within_tolerance: bool,
    pub latest_extension_eligible_for_stability_sequence: bool,
    pub material_boundary_dependence_at_latest_extension: bool,
    pub adequate: bool,
}

impl SpatialExtentConvergenceAssessment {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

/// Evaluate observed inner-domain metrics from progressively enlarged simulation domains.
///
/// Buffers must be strictly increasing and every observation must contain exactly the metrics
/// named by the criterion. Relative differences use `max(previous, current)` as a symmetric
/// denominator and round upward to whole permille so a small non-zero change cannot disappear by
/// integer truncation. If both absolute and relative tolerances are declared, both must pass.
pub fn assess_spatial_extent_convergence(
    criterion: SpatialExtentAdequacyCriterion,
    observations: Vec<SpatialExtentObservation>,
) -> Result<SpatialExtentConvergenceAssessment, SpatialBoundaryError> {
    criterion.validate()?;
    if observations.len() < 2 {
        return Err(SpatialBoundaryError::TooFewExtentObservations);
    }

    let expected_metric_ids = criterion
        .metric_tolerances
        .iter()
        .map(|tolerance| tolerance.metric_id.as_str())
        .collect::<BTreeSet<_>>();
    for (index, observation) in observations.iter().enumerate() {
        if index > 0 && observation.buffer_cells <= observations[index - 1].buffer_cells {
            return Err(SpatialBoundaryError::NonIncreasingExtentBuffers {
                previous: observations[index - 1].buffer_cells,
                current: observation.buffer_cells,
            });
        }
        let mut observed_ids = BTreeSet::new();
        for metric in &observation.metrics {
            if metric.metric_id.trim().is_empty() {
                return Err(SpatialBoundaryError::EmptyExtentObservationMetricId);
            }
            if !observed_ids.insert(metric.metric_id.as_str()) {
                return Err(SpatialBoundaryError::DuplicateExtentObservationMetricId(
                    metric.metric_id.clone(),
                ));
            }
            if !expected_metric_ids.contains(metric.metric_id.as_str()) {
                return Err(SpatialBoundaryError::UnexpectedExtentObservationMetric(
                    metric.metric_id.clone(),
                ));
            }
        }
        for expected in &expected_metric_ids {
            if !observed_ids.contains(expected) {
                return Err(SpatialBoundaryError::MissingExtentObservationMetric(
                    (*expected).to_owned(),
                ));
            }
        }
    }

    let mut comparisons = Vec::with_capacity(observations.len() - 1);
    for pair in observations.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        let mut metric_comparisons = Vec::with_capacity(criterion.metric_tolerances.len());
        let mut all_metrics_within_tolerance = true;
        for tolerance in &criterion.metric_tolerances {
            let previous_value = previous
                .metrics
                .iter()
                .find(|metric| metric.metric_id == tolerance.metric_id)
                .expect("validated extent observation metric must exist")
                .value;
            let current_value = current
                .metrics
                .iter()
                .find(|metric| metric.metric_id == tolerance.metric_id)
                .expect("validated extent observation metric must exist")
                .value;
            let absolute_difference = previous_value.abs_diff(current_value);
            let denominator = previous_value.max(current_value);
            let relative_difference_permille = if denominator == 0 {
                0
            } else {
                let numerator = u128::from(absolute_difference) * 1_000;
                let denominator = u128::from(denominator);
                let rounded_up = numerator.div_ceil(denominator).min(1_000);
                u16::try_from(rounded_up).map_err(|_| SpatialBoundaryError::AccountingOverflow)?
            };
            let absolute_within_tolerance = tolerance
                .max_absolute_difference
                .map(|limit| absolute_difference <= limit);
            let relative_within_tolerance = tolerance
                .max_relative_difference_permille
                .map(|limit| relative_difference_permille <= limit);
            let within_tolerance = absolute_within_tolerance.unwrap_or(true)
                && relative_within_tolerance.unwrap_or(true);
            all_metrics_within_tolerance &= within_tolerance;
            metric_comparisons.push(SpatialExtentMetricComparison {
                metric_id: tolerance.metric_id.clone(),
                previous_value,
                current_value,
                absolute_difference,
                relative_difference_permille,
                absolute_within_tolerance,
                relative_within_tolerance,
                within_tolerance,
            });
        }
        let eligible_for_stability_sequence = criterion
            .minimum_buffer_cells
            .is_none_or(|minimum| previous.buffer_cells >= minimum);
        comparisons.push(SpatialExtentExtensionComparison {
            previous_buffer_cells: previous.buffer_cells,
            current_buffer_cells: current.buffer_cells,
            eligible_for_stability_sequence,
            all_metrics_within_tolerance,
            metrics: metric_comparisons,
        });
    }

    let trailing_stable_extensions_usize = comparisons
        .iter()
        .rev()
        .take_while(|comparison| {
            comparison.eligible_for_stability_sequence && comparison.all_metrics_within_tolerance
        })
        .count();
    let trailing_stable_extensions =
        u16::try_from(trailing_stable_extensions_usize).unwrap_or(u16::MAX);
    let latest = comparisons
        .last()
        .expect("two observations must produce one comparison");
    let latest_extension_within_tolerance = latest.all_metrics_within_tolerance;
    let latest_extension_eligible_for_stability_sequence = latest.eligible_for_stability_sequence;
    let material_boundary_dependence_at_latest_extension = !latest_extension_within_tolerance;
    let latest_extension_within_tolerance = latest.all_metrics_within_tolerance;
    let latest_extension_eligible_for_stability_sequence = latest.eligible_for_stability_sequence;
    let material_boundary_dependence_at_latest_extension = !latest_extension_within_tolerance;
    let latest_extension_within_tolerance = latest.all_metrics_within_tolerance;
    let latest_extension_eligible_for_stability_sequence = latest.eligible_for_stability_sequence;
    let material_boundary_dependence_at_latest_extension = !latest_extension_within_tolerance;
    let latest_extension_within_tolerance = latest.all_metrics_within_tolerance;
    let latest_extension_eligible_for_stability_sequence = latest.eligible_for_stability_sequence;
    let material_boundary_dependence_at_latest_extension = !latest_extension_within_tolerance;
    let adequate = trailing_stable_extensions >= criterion.required_consecutive_stable_extensions;

    Ok(SpatialExtentConvergenceAssessment {
        schema_version: SpatialExtentConvergenceAssessment::CURRENT_SCHEMA_VERSION,
        criterion,
        observations,
        comparisons,
        trailing_stable_extensions,
        latest_extension_within_tolerance,
        latest_extension_eligible_for_stability_sequence,
        material_boundary_dependence_at_latest_extension,
        adequate,
    })
}

fn simulation_extent(
    landscape: &LandscapeBundle,
) -> Result<SpatialAnalysisExtent, SpatialBoundaryError> {
    landscape.validate()?;
    let width = i128::from(landscape.width)
        .checked_mul(i128::from(landscape.geometry.cell_size_x))
        .ok_or(SpatialBoundaryError::CoordinateExtentOverflow)?;
    let height = i128::from(landscape.height)
        .checked_mul(i128::from(landscape.geometry.cell_size_y))
        .ok_or(SpatialBoundaryError::CoordinateExtentOverflow)?;
    let min_x = i128::from(landscape.geometry.origin_x);
    let max_y = i128::from(landscape.geometry.origin_y);
    let max_x = min_x
        .checked_add(width)
        .ok_or(SpatialBoundaryError::CoordinateExtentOverflow)?;
    let min_y = max_y
        .checked_sub(height)
        .ok_or(SpatialBoundaryError::CoordinateExtentOverflow)?;
    Ok(SpatialAnalysisExtent {
        min_x: i64::try_from(min_x).map_err(|_| SpatialBoundaryError::CoordinateExtentOverflow)?,
        min_y: i64::try_from(min_y).map_err(|_| SpatialBoundaryError::CoordinateExtentOverflow)?,
        max_x: i64::try_from(max_x).map_err(|_| SpatialBoundaryError::CoordinateExtentOverflow)?,
        max_y: i64::try_from(max_y).map_err(|_| SpatialBoundaryError::CoordinateExtentOverflow)?,
    })
}

#[derive(Debug, Error)]
pub enum SpatialBoundaryError {
    #[error(transparent)]
    Landscape(#[from] LandscapeError),
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
    #[error(
        "spatial boundary declaration schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedDeclarationSchema { found: u32, supported: u32 },
    #[error(
        "spatial analysis-domain schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedAnalysisDomainSchema { found: u32, supported: u32 },
    #[error(
        "spatial extent-adequacy schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedAdequacyCriterionSchema { found: u32, supported: u32 },
    #[error("spatial boundary declaration identifier is empty")]
    EmptyDeclarationId,
    #[error("spatial boundary declaration rationale is empty")]
    EmptyRationale,
    #[error("spatial boundary supporting evidence identifier is empty")]
    EmptySupportingEvidenceId,
    #[error("duplicate spatial boundary supporting evidence identifier {0}")]
    DuplicateSupportingEvidenceId(String),
    #[error("non-barrier boundary declarations must not claim supporting barrier evidence")]
    EvidenceOnNonBarrierDeclaration,
    #[error("a declared closed barrier requires at least one supporting evidence record")]
    BarrierWithoutEvidence,
    #[error("a declared closed barrier requires an evidence catalogue")]
    MissingEvidenceCatalog,
    #[error("unknown supporting spatial-boundary evidence record {0}")]
    UnknownSupportingEvidence(String),
    #[error("supporting spatial-boundary evidence record {0} remains unresolved")]
    UnresolvedBarrierEvidence(String),
    #[error("spatial analysis-domain identifier is empty")]
    EmptyAnalysisDomainId,
    #[error("spatial analysis-domain extent is empty or inverted")]
    InvalidAnalysisExtent,
    #[error("spatial analysis domain extends outside the simulation domain")]
    AnalysisDomainOutsideSimulation,
    #[error("spatial analysis domain must align exactly to whole landscape cells")]
    AnalysisDomainNotCellAligned,
    #[error("spatial analysis-domain grid index overflowed")]
    AnalysisDomainIndexOverflow,
    #[error("landscape and world grids do not match")]
    GridMismatch,
    #[error("spatial boundary assessment references invalid cell {0:?}")]
    InvalidCell(CellId),
    #[error("M4 candidate count does not fit the boundary-assessment schema")]
    CandidateCountOverflow,
    #[error("invalid M4 configuration for spatial boundary assessment: {0}")]
    InvalidMigrationConfig(String),
    #[error("spatial boundary assessment accounting overflow")]
    AccountingOverflow,
    #[error("spatial extent-adequacy criterion identifier is empty")]
    EmptyAdequacyCriterionId,
    #[error("spatial extent-adequacy criterion requires at least one stable extension")]
    ZeroStableExtensionRequirement,
    #[error("spatial extent-adequacy criterion contains no metrics")]
    NoExtentMetrics,
    #[error("spatial extent metric identifier is empty")]
    EmptyExtentMetricId,
    #[error("duplicate spatial extent metric identifier {0}")]
    DuplicateExtentMetricId(String),
    #[error("spatial extent metric {0} has neither an absolute nor relative tolerance")]
    ExtentMetricWithoutTolerance(String),
    #[error("spatial extent metric {metric_id} relative tolerance exceeds 1000 permille")]
    RelativeToleranceOutOfRange { metric_id: String },
    #[error("spatial extent convergence requires at least two observations")]
    TooFewExtentObservations,
    #[error(
        "spatial extent buffers must increase strictly; previous {previous}, current {current}"
    )]
    NonIncreasingExtentBuffers { previous: u32, current: u32 },
    #[error("spatial extent observation metric identifier is empty")]
    EmptyExtentObservationMetricId,
    #[error("duplicate spatial extent observation metric identifier {0}")]
    DuplicateExtentObservationMetricId(String),
    #[error("unexpected spatial extent observation metric {0}")]
    UnexpectedExtentObservationMetric(String),
    #[error("missing spatial extent observation metric {0}")]
    MissingExtentObservationMetric(String),
    #[error("spatial coordinate extent overflowed")]
    CoordinateExtentOverflow,
}
