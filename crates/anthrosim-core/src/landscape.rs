use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{evidence::EvidenceCatalog, ids::CellId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeBundle {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub grid_convention: GridConvention,
    pub geometry: GridGeometry,
    pub layers: Vec<LandscapeLayer>,
}

impl LandscapeBundle {
    /// v2 makes the normalized row/column-to-CRS convention explicit and machine-readable.
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub fn new(
        width: u32,
        height: u32,
        geometry: GridGeometry,
        layers: Vec<LandscapeLayer>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            width,
            height,
            grid_convention: GridConvention::north_up_area_v1(),
            geometry,
            layers,
        }
    }

    #[must_use]
    pub fn cell_count(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    #[must_use]
    pub fn layer(&self, id: &str) -> Option<&LandscapeLayer> {
        self.layers.iter().find(|layer| layer.layer_id == id)
    }

    /// Convert an authoritative row-major `CellId` into zero-based `(x, y)` grid coordinates.
    ///
    /// Under the v2 convention, `x` increases with columns from left to right and `y` increases
    /// with rows from top to bottom. CRS X therefore increases with `x`, while CRS Y decreases
    /// with `y`.
    #[must_use]
    pub fn grid_coordinates(&self, cell: CellId) -> Option<(u32, u32)> {
        let zero_based = cell.0.checked_sub(1)?;
        if zero_based >= self.cell_count() || self.width == 0 {
            return None;
        }
        let width = u64::from(self.width);
        let x = u32::try_from(zero_based % width).ok()?;
        let y = u32::try_from(zero_based / width).ok()?;
        Some((x, y))
    }

    /// Exact CRS-aligned outer extent for an authoritative cell.
    pub fn cell_extent(&self, cell: CellId) -> Result<GridCellExtent, LandscapeError> {
        let (x, y) = self
            .grid_coordinates(cell)
            .ok_or(LandscapeError::InvalidCellId { cell })?;
        self.geometry.cell_extent(x, y)
    }

    /// Exact cell centre represented in doubled coordinate units.
    ///
    /// Divide each returned component by two to obtain the CRS coordinate. This avoids floating
    /// point and still represents half-unit centres when an odd integer cell size is used.
    pub fn cell_centre_2x(&self, cell: CellId) -> Result<GridCellCentre2x, LandscapeError> {
        let extent = self.cell_extent(cell)?;
        Ok(extent.centre_2x())
    }

    pub fn validate(&self) -> Result<(), LandscapeError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(LandscapeError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.width == 0 || self.height == 0 {
            return Err(LandscapeError::InvalidDimensions);
        }
        self.grid_convention.validate()?;
        self.geometry.validate()?;
        self.geometry.validate_extent(self.width, self.height)?;

        let expected = self.cell_count();
        let _ = usize::try_from(expected).map_err(|_| LandscapeError::CellCountTooLarge {
            cell_count: expected,
        })?;

        let mut layer_ids = BTreeSet::new();
        for layer in &self.layers {
            layer.validate(expected)?;
            if !layer_ids.insert(layer.layer_id.as_str()) {
                return Err(LandscapeError::DuplicateLayerId(layer.layer_id.clone()));
            }
        }
        Ok(())
    }

    /// Validate all landscape evidence links under one shared contract.
    ///
    /// Landscapes with no evidence links may omit a catalogue. Once a layer
    /// declares `evidenceInputId`, a valid catalogue is mandatory and the
    /// referenced external input must exist exactly.
    pub fn validate_evidence_context(
        &self,
        catalog: Option<&EvidenceCatalog>,
    ) -> Result<(), LandscapeError> {
        self.validate()?;
        if let Some(catalog) = catalog {
            catalog
                .validate()
                .map_err(|error| LandscapeError::InvalidEvidenceCatalog(error.to_string()))?;
        }

        for layer in &self.layers {
            let Some(input_id) = layer.evidence_input_id.as_deref() else {
                continue;
            };
            let Some(catalog) = catalog else {
                return Err(LandscapeError::MissingEvidenceCatalog {
                    layer_id: layer.layer_id.clone(),
                    input_id: input_id.to_owned(),
                });
            };
            if !catalog
                .external_inputs
                .iter()
                .any(|input| input.input_id == input_id)
            {
                return Err(LandscapeError::UnknownEvidenceInput {
                    layer_id: layer.layer_id.clone(),
                    input_id: input_id.to_owned(),
                });
            }
        }
        Ok(())
    }

    pub fn validate_evidence_links(&self, catalog: &EvidenceCatalog) -> Result<(), LandscapeError> {
        self.validate_evidence_context(Some(catalog))
    }

    /// Stable non-cryptographic identity for deterministic experiment wiring.
    /// This is distinct from cryptographic research-archive integrity.
    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut digest = StableDigest::new();
        digest.write_u32(self.schema_version);
        digest.write_u32(self.width);
        digest.write_u32(self.height);
        self.grid_convention.write_digest(&mut digest);
        self.geometry.write_digest(&mut digest);
        digest.write_u64(self.layers.len() as u64);
        for layer in &self.layers {
            layer.write_digest(&mut digest);
        }
        digest.finish()
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "landscape-v{}-{:016x}",
            self.schema_version,
            self.digest64()
        )
    }
}

/// Machine-readable v2 relationship between row-major grid indices and CRS coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridConvention {
    pub origin_anchor: GridOriginAnchor,
    pub column_direction: GridColumnDirection,
    pub row_direction: GridRowDirection,
    pub cell_interpretation: GridCellInterpretation,
}

impl GridConvention {
    #[must_use]
    pub const fn north_up_area_v1() -> Self {
        Self {
            origin_anchor: GridOriginAnchor::UpperLeftOuterCorner,
            column_direction: GridColumnDirection::IncreasingX,
            row_direction: GridRowDirection::DecreasingY,
            cell_interpretation: GridCellInterpretation::Area,
        }
    }

    fn validate(self) -> Result<(), LandscapeError> {
        if self != Self::north_up_area_v1() {
            return Err(LandscapeError::UnsupportedGridConvention);
        }
        Ok(())
    }

    fn write_digest(self, digest: &mut StableDigest) {
        digest.write_u8(match self.origin_anchor {
            GridOriginAnchor::UpperLeftOuterCorner => 0,
        });
        digest.write_u8(match self.column_direction {
            GridColumnDirection::IncreasingX => 0,
        });
        digest.write_u8(match self.row_direction {
            GridRowDirection::DecreasingY => 0,
        });
        digest.write_u8(match self.cell_interpretation {
            GridCellInterpretation::Area => 0,
        });
    }
}

impl Default for GridConvention {
    fn default() -> Self {
        Self::north_up_area_v1()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridOriginAnchor {
    UpperLeftOuterCorner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridColumnDirection {
    IncreasingX,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridRowDirection {
    DecreasingY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridCellInterpretation {
    Area,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridGeometry {
    /// CRS coordinate of the upper-left outer corner of row 0 / column 0.
    pub origin_x: i64,
    pub origin_y: i64,
    pub cell_size_x: u64,
    pub cell_size_y: u64,
    pub coordinate_unit: String,
    pub spatial_reference: String,
}

impl GridGeometry {
    fn validate(&self) -> Result<(), LandscapeError> {
        if self.cell_size_x == 0 || self.cell_size_y == 0 {
            return Err(LandscapeError::InvalidCellSize);
        }
        if self.coordinate_unit.trim().is_empty() {
            return Err(LandscapeError::EmptyCoordinateUnit);
        }
        if self.spatial_reference.trim().is_empty() {
            return Err(LandscapeError::EmptySpatialReference);
        }
        Ok(())
    }

    fn validate_extent(&self, width: u32, height: u32) -> Result<(), LandscapeError> {
        self.cell_extent(width - 1, height - 1).map(|_| ())
    }

    /// Exact outer extent for zero-based grid coordinates under the v2 convention.
    pub fn cell_extent(&self, x: u32, y: u32) -> Result<GridCellExtent, LandscapeError> {
        let origin_x = i128::from(self.origin_x);
        let origin_y = i128::from(self.origin_y);
        let size_x = i128::from(self.cell_size_x);
        let size_y = i128::from(self.cell_size_y);
        let x_offset = i128::from(x)
            .checked_mul(size_x)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        let y_offset = i128::from(y)
            .checked_mul(size_y)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        let min_x = origin_x
            .checked_add(x_offset)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        let max_x = min_x
            .checked_add(size_x)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        let max_y = origin_y
            .checked_sub(y_offset)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        let min_y = max_y
            .checked_sub(size_y)
            .ok_or(LandscapeError::CoordinateExtentOverflow)?;
        Ok(GridCellExtent {
            min_x: i64::try_from(min_x).map_err(|_| LandscapeError::CoordinateExtentOverflow)?,
            min_y: i64::try_from(min_y).map_err(|_| LandscapeError::CoordinateExtentOverflow)?,
            max_x: i64::try_from(max_x).map_err(|_| LandscapeError::CoordinateExtentOverflow)?,
            max_y: i64::try_from(max_y).map_err(|_| LandscapeError::CoordinateExtentOverflow)?,
        })
    }

    #[must_use]
    pub const fn has_square_cells(&self) -> bool {
        self.cell_size_x == self.cell_size_y
    }

    fn write_digest(&self, digest: &mut StableDigest) {
        digest.write_i64(self.origin_x);
        digest.write_i64(self.origin_y);
        digest.write_u64(self.cell_size_x);
        digest.write_u64(self.cell_size_y);
        digest.write_str(&self.coordinate_unit);
        digest.write_str(&self.spatial_reference);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCellExtent {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

impl GridCellExtent {
    #[must_use]
    pub fn centre_2x(self) -> GridCellCentre2x {
        GridCellCentre2x {
            x_twice: i128::from(self.min_x) + i128::from(self.max_x),
            y_twice: i128::from(self.min_y) + i128::from(self.max_y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCellCentre2x {
    pub x_twice: i128,
    pub y_twice: i128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandscapeLayerRole {
    TerrainTraversal,
    WaterAccessibility,
    ResourceOpportunity,
    Auxiliary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeLayer {
    pub layer_id: String,
    pub role: LandscapeLayerRole,
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_domain: Option<LandscapeValueDomain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_input_id: Option<String>,
    /// Row-major normalized values. `None` is explicit nodata.
    pub values: Vec<Option<i32>>,
}

impl LandscapeLayer {
    fn validate(&self, expected_cell_count: u64) -> Result<(), LandscapeError> {
        if self.layer_id.trim().is_empty() {
            return Err(LandscapeError::EmptyLayerId);
        }
        if self.unit.trim().is_empty() {
            return Err(LandscapeError::EmptyLayerUnit {
                layer_id: self.layer_id.clone(),
            });
        }
        if matches!(self.evidence_input_id.as_deref(), Some(id) if id.trim().is_empty()) {
            return Err(LandscapeError::EmptyEvidenceInputId {
                layer_id: self.layer_id.clone(),
            });
        }
        if self.values.len() as u64 != expected_cell_count {
            return Err(LandscapeError::LayerLengthMismatch {
                layer_id: self.layer_id.clone(),
                expected: expected_cell_count,
                actual: self.values.len() as u64,
            });
        }
        if let Some(domain) = self.value_domain {
            if domain.min > domain.max {
                return Err(LandscapeError::InvalidValueDomain {
                    layer_id: self.layer_id.clone(),
                });
            }
            for (index, value) in self.values.iter().enumerate() {
                if let Some(value) = value
                    && (*value < domain.min || *value > domain.max)
                {
                    return Err(LandscapeError::ValueOutOfDomain {
                        layer_id: self.layer_id.clone(),
                        cell_index: index as u64,
                        value: *value,
                    });
                }
            }
        }
        Ok(())
    }

    fn write_digest(&self, digest: &mut StableDigest) {
        digest.write_str(&self.layer_id);
        digest.write_u8(match self.role {
            LandscapeLayerRole::TerrainTraversal => 0,
            LandscapeLayerRole::WaterAccessibility => 1,
            LandscapeLayerRole::ResourceOpportunity => 2,
            LandscapeLayerRole::Auxiliary => 3,
        });
        digest.write_str(&self.unit);
        match self.value_domain {
            Some(domain) => {
                digest.write_u8(1);
                digest.write_i32(domain.min);
                digest.write_i32(domain.max);
            }
            None => digest.write_u8(0),
        }
        match &self.evidence_input_id {
            Some(input_id) => {
                digest.write_u8(1);
                digest.write_str(input_id);
            }
            None => digest.write_u8(0),
        }
        digest.write_u64(self.values.len() as u64);
        for value in &self.values {
            match value {
                Some(value) => {
                    digest.write_u8(1);
                    digest.write_i32(*value);
                }
                None => digest.write_u8(0),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeValueDomain {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LandscapeError {
    #[error("landscape schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("landscape width and height must both be greater than zero")]
    InvalidDimensions,
    #[error("landscape cell count {cell_count} cannot be represented on this platform")]
    CellCountTooLarge { cell_count: u64 },
    #[error("landscape uses an unsupported grid convention")]
    UnsupportedGridConvention,
    #[error("landscape cell sizes must both be greater than zero")]
    InvalidCellSize,
    #[error("landscape CRS extent cannot be represented in signed 64-bit coordinate units")]
    CoordinateExtentOverflow,
    #[error("landscape references invalid cell id {cell:?}")]
    InvalidCellId { cell: CellId },
    #[error("landscape coordinate unit is empty")]
    EmptyCoordinateUnit,
    #[error("landscape spatial reference is empty")]
    EmptySpatialReference,
    #[error("landscape layer identifier is empty")]
    EmptyLayerId,
    #[error("duplicate landscape layer identifier {0}")]
    DuplicateLayerId(String),
    #[error("landscape layer {layer_id} has an empty unit")]
    EmptyLayerUnit { layer_id: String },
    #[error("landscape layer {layer_id} has an empty evidence input identifier")]
    EmptyEvidenceInputId { layer_id: String },
    #[error("landscape layer {layer_id} has {actual} values; expected {expected}")]
    LayerLengthMismatch {
        layer_id: String,
        expected: u64,
        actual: u64,
    },
    #[error("landscape layer {layer_id} has an invalid value domain")]
    InvalidValueDomain { layer_id: String },
    #[error(
        "landscape layer {layer_id} cell {cell_index} value {value} is outside its declared domain"
    )]
    ValueOutOfDomain {
        layer_id: String,
        cell_index: u64,
        value: i32,
    },
    #[error(
        "landscape layer {layer_id} references evidence external input {input_id}, but no evidence catalogue was supplied"
    )]
    MissingEvidenceCatalog { layer_id: String, input_id: String },
    #[error("landscape evidence catalogue is invalid: {0}")]
    InvalidEvidenceCatalog(String),
    #[error("landscape layer {layer_id} references unknown evidence external input {input_id}")]
    UnknownEvidenceInput { layer_id: String, input_id: String },
}

struct StableDigest(u64);

impl StableDigest {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_i32(&mut self, value: i32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_i64(&mut self, value: i64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write_bytes(value.as_bytes());
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EvidenceCatalog, EvidenceRecord, EvidenceSource, ExternalInputEvidence, ParameterProvenance,
    };

    use super::*;

    fn evidence_landscape(input_id: &str) -> LandscapeBundle {
        LandscapeBundle::new(
            1,
            1,
            GridGeometry {
                origin_x: 0,
                origin_y: 0,
                cell_size_x: 1,
                cell_size_y: 1,
                coordinate_unit: "m".to_owned(),
                spatial_reference: "EPSG:27700".to_owned(),
            },
            vec![LandscapeLayer {
                layer_id: "terrain".to_owned(),
                role: LandscapeLayerRole::TerrainTraversal,
                unit: "permille".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 1000 }),
                evidence_input_id: Some(input_id.to_owned()),
                values: vec![Some(500)],
            }],
        )
    }

    fn evidence_catalog(input_id: &str) -> EvidenceCatalog {
        EvidenceCatalog::new(vec![EvidenceRecord {
            schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
            evidence_id: "terrain-source".to_owned(),
            provenance: ParameterProvenance::EmpiricalDerived,
            source: EvidenceSource {
                source_id: "terrain-dataset".to_owned(),
                citation: "Example terrain dataset".to_owned(),
                persistent_id: None,
                dataset_version: None,
                licence: None,
                spatial_coverage: None,
                temporal_coverage: None,
            },
            original_variable: "terrain".to_owned(),
            original_units: "permille".to_owned(),
            transformation: None,
            simulation_units: "permille".to_owned(),
            uncertainty: None,
            applicability: "test".to_owned(),
            competing_estimates: Vec::new(),
        }])
        .with_external_inputs(vec![ExternalInputEvidence {
            input_id: input_id.to_owned(),
            evidence_id: "terrain-source".to_owned(),
            format: "test-grid".to_owned(),
            spatial_reference: Some("EPSG:27700".to_owned()),
            content_digest: None,
        }])
    }

    #[test]
    fn evidence_link_requires_catalogue() {
        let landscape = evidence_landscape("terrain-input");
        assert!(matches!(
            landscape.validate_evidence_context(None),
            Err(LandscapeError::MissingEvidenceCatalog { layer_id, input_id })
                if layer_id == "terrain" && input_id == "terrain-input"
        ));
    }

    #[test]
    fn unknown_evidence_input_is_rejected() {
        let landscape = evidence_landscape("missing-input");
        let catalog = evidence_catalog("other-input");
        assert!(matches!(
            landscape.validate_evidence_context(Some(&catalog)),
            Err(LandscapeError::UnknownEvidenceInput { layer_id, input_id })
                if layer_id == "terrain" && input_id == "missing-input"
        ));
    }

    #[test]
    fn valid_evidence_input_resolves() {
        let landscape = evidence_landscape("terrain-input");
        let catalog = evidence_catalog("terrain-input");
        assert_eq!(landscape.validate_evidence_context(Some(&catalog)), Ok(()));
    }

    #[test]
    fn asymmetric_fixture_has_unambiguous_row_major_crs_geometry() {
        let landscape = LandscapeBundle::new(
            3,
            2,
            GridGeometry {
                origin_x: 1_000,
                origin_y: 2_000,
                cell_size_x: 10,
                cell_size_y: 20,
                coordinate_unit: "metre".to_owned(),
                spatial_reference: "LOCAL_CS[test]".to_owned(),
            },
            vec![LandscapeLayer {
                layer_id: "labels".to_owned(),
                role: LandscapeLayerRole::Auxiliary,
                unit: "ordinal".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 1, max: 6 }),
                evidence_input_id: None,
                values: (1..=6).map(Some).collect(),
            }],
        );
        landscape.validate().expect("valid v2 landscape");

        assert_eq!(
            landscape.grid_convention,
            GridConvention::north_up_area_v1()
        );
        assert_eq!(landscape.grid_coordinates(CellId::new(1)), Some((0, 0)));
        assert_eq!(landscape.grid_coordinates(CellId::new(4)), Some((0, 1)));
        assert_eq!(landscape.grid_coordinates(CellId::new(6)), Some((2, 1)));
        assert_eq!(
            landscape.cell_extent(CellId::new(1)).unwrap(),
            GridCellExtent {
                min_x: 1_000,
                min_y: 1_980,
                max_x: 1_010,
                max_y: 2_000,
            }
        );
        assert_eq!(
            landscape.cell_extent(CellId::new(6)).unwrap(),
            GridCellExtent {
                min_x: 1_020,
                min_y: 1_960,
                max_x: 1_030,
                max_y: 1_980,
            }
        );
        assert_eq!(
            landscape.cell_centre_2x(CellId::new(6)).unwrap(),
            GridCellCentre2x {
                x_twice: 2_050,
                y_twice: 3_940,
            }
        );
    }

    #[test]
    fn ambiguous_v1_landscape_is_rejected() {
        let mut landscape = evidence_landscape("terrain-input");
        landscape.schema_version = 1;
        assert!(matches!(
            landscape.validate(),
            Err(LandscapeError::UnsupportedSchema {
                found: 1,
                supported: 2
            })
        ));
    }
}
