use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::EvidenceCatalog;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeBundle {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub geometry: GridGeometry,
    pub layers: Vec<LandscapeLayer>,
}

impl LandscapeBundle {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

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
        self.geometry.validate()?;

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

    pub fn validate_evidence_links(&self, catalog: &EvidenceCatalog) -> Result<(), LandscapeError> {
        let external_inputs: BTreeSet<&str> = catalog
            .external_inputs
            .iter()
            .map(|input| input.input_id.as_str())
            .collect();

        for layer in &self.layers {
            if let Some(input_id) = layer.evidence_input_id.as_deref() {
                if !external_inputs.contains(input_id) {
                    return Err(LandscapeError::UnknownEvidenceInput {
                        layer_id: layer.layer_id.clone(),
                        input_id: input_id.to_owned(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Stable non-cryptographic identity for deterministic experiment wiring.
    /// This is distinct from cryptographic research-archive integrity.
    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut digest = StableDigest::new();
        digest.write_u32(self.schema_version);
        digest.write_u32(self.width);
        digest.write_u32(self.height);
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridGeometry {
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

    fn write_digest(&self, digest: &mut StableDigest) {
        digest.write_i64(self.origin_x);
        digest.write_i64(self.origin_y);
        digest.write_u64(self.cell_size_x);
        digest.write_u64(self.cell_size_y);
        digest.write_str(&self.coordinate_unit);
        digest.write_str(&self.spatial_reference);
    }
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
                if let Some(value) = value {
                    if *value < domain.min || *value > domain.max {
                        return Err(LandscapeError::ValueOutOfDomain {
                            layer_id: self.layer_id.clone(),
                            cell_index: index as u64,
                            value: *value,
                        });
                    }
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
    #[error("landscape cell sizes must both be greater than zero")]
    InvalidCellSize,
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
