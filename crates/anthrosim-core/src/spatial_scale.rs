use serde::{Deserialize, Serialize};

use crate::LandscapeBundle;

/// Executable spatial scale contract currently supported by AnthroSim.
///
/// There is deliberately no physically normalized alternative yet. Exposing only the semantics
/// that actually exist prevents a configuration switch from implying unsupported physical
/// invariance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialScaleSemantics {
    CellSpaceResolutionDependentV1,
}

/// Machine-readable statement about whether one landscape-backed run may be interpreted as
/// independent of raster resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialScaleStatus {
    /// Current M2/M3/M4/M9 spatial quantities remain tied to cells/edges. Resolution must therefore
    /// be treated as a scientific sensitivity dimension rather than incidental GIS metadata.
    ResolutionDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialM2InteractionBasis {
    /// Reproductive-contact eligibility is defined by exact equality of persistent-residence
    /// `CellId`; there is no physical interaction radius or settlement-unit abstraction.
    ExactPersistentResidenceCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialResourceQuantityBasis {
    PerCellTotal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialM4DistanceBasis {
    GridSteps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialM9TravelCostBasis {
    GridEdges,
}

/// Preserved scale/readiness provenance derived from the exact landscape geometry and executable
/// spatial scale semantics.
///
/// This is deliberately separate from evidence closure. An input can be perfectly source-closed
/// while the causal model remains resolution-dependent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialScaleAssessment {
    pub schema_version: u32,
    pub semantics: SpatialScaleSemantics,
    pub status: SpatialScaleStatus,
    pub cell_size_x: u64,
    pub cell_size_y: u64,
    pub coordinate_unit: String,
    pub m2_interaction_basis: SpatialM2InteractionBasis,
    pub resource_quantity_basis: SpatialResourceQuantityBasis,
    pub m4_distance_basis: SpatialM4DistanceBasis,
    pub m9_travel_cost_basis: SpatialM9TravelCostBasis,
    /// `true` means raster resolution must be varied or otherwise justified before a result is
    /// described as scale-independent physical inference.
    pub requires_resolution_sensitivity: bool,
}

impl SpatialScaleAssessment {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
    pub const CURRENT_SEMANTICS: SpatialScaleSemantics =
        SpatialScaleSemantics::CellSpaceResolutionDependentV1;

    #[must_use]
    pub fn resolve(landscape: &LandscapeBundle) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            semantics: Self::CURRENT_SEMANTICS,
            status: SpatialScaleStatus::ResolutionDependent,
            cell_size_x: landscape.geometry.cell_size_x,
            cell_size_y: landscape.geometry.cell_size_y,
            coordinate_unit: landscape.geometry.coordinate_unit.clone(),
            m2_interaction_basis: SpatialM2InteractionBasis::ExactPersistentResidenceCell,
            resource_quantity_basis: SpatialResourceQuantityBasis::PerCellTotal,
            m4_distance_basis: SpatialM4DistanceBasis::GridSteps,
            m9_travel_cost_basis: SpatialM9TravelCostBasis::GridEdges,
            requires_resolution_sensitivity: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridGeometry;

    #[test]
    fn cell_space_assessment_preserves_exact_resolution_and_dependence() {
        let landscape = LandscapeBundle::new(
            2,
            2,
            GridGeometry {
                origin_x: 0,
                origin_y: 200,
                cell_size_x: 100,
                cell_size_y: 100,
                coordinate_unit: "metre".to_owned(),
                spatial_reference: "LOCAL:TEST".to_owned(),
            },
            Vec::new(),
        );
        let assessment = SpatialScaleAssessment::resolve(&landscape);
        assert_eq!(
            assessment.semantics,
            SpatialScaleSemantics::CellSpaceResolutionDependentV1
        );
        assert_eq!(assessment.status, SpatialScaleStatus::ResolutionDependent);
        assert_eq!(assessment.cell_size_x, 100);
        assert_eq!(assessment.cell_size_y, 100);
        assert_eq!(assessment.coordinate_unit, "metre");
        assert_eq!(
            assessment.m2_interaction_basis,
            SpatialM2InteractionBasis::ExactPersistentResidenceCell
        );
        assert!(assessment.requires_resolution_sensitivity);
    }
}
