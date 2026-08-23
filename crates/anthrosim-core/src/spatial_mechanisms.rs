use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    LandscapeBundle, LandscapeError, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    evidence::EvidenceCatalog,
    world::{BASE_MOVEMENT_COST, PERMILLE_MAX},
};

/// Compatibility identity for the M8.4 landscape-to-model transformation semantics.
///
/// This remains separate from the pre-existing core `MODEL_SEMANTICS_ID`: synthetic execution
/// continues to use the unchanged M1-M7 world-generation semantics, while landscape-transformed
/// runs must preserve and verify this identity in their spatial provenance wrappers.
pub const SPATIAL_MODEL_SEMANTICS_ID: &str = "anthrosim-spatial-transform-semantics-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialMechanismConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub transforms: Vec<SpatialFieldTransform>,
}

impl SpatialMechanismConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn new(model_id: impl Into<String>, transforms: Vec<SpatialFieldTransform>) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: model_id.into(),
            transforms,
        }
    }

    pub fn validate(&self) -> Result<(), SpatialMechanismError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(SpatialMechanismError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.model_id.trim().is_empty() {
            return Err(SpatialMechanismError::EmptyModelId);
        }
        if self.transforms.is_empty() {
            return Err(SpatialMechanismError::NoTransforms);
        }

        let mut targets = BTreeSet::new();
        for transform in &self.transforms {
            transform.validate()?;
            if !targets.insert(transform.target) {
                return Err(SpatialMechanismError::DuplicateTarget(transform.target));
            }
        }
        Ok(())
    }

    /// Validate optional transformation-assumption evidence links against the experiment catalogue.
    ///
    /// A transform with no `evidenceId` remains valid for synthetic/null-model exploration. Once a
    /// link is declared it is strict: the experiment must carry a valid `EvidenceCatalog` containing
    /// that exact record identifier.
    pub fn validate_evidence_links(
        &self,
        catalog: Option<&EvidenceCatalog>,
    ) -> Result<(), SpatialMechanismError> {
        self.validate()?;
        if let Some(catalog) = catalog {
            catalog.validate()?;
        }
        for transform in &self.transforms {
            let Some(evidence_id) = transform.evidence_id.as_deref() else {
                continue;
            };
            let evidence_id = evidence_id.trim();
            let Some(catalog) = catalog else {
                return Err(SpatialMechanismError::MissingEvidenceCatalog {
                    evidence_id: evidence_id.to_owned(),
                });
            };
            if !catalog
                .records
                .iter()
                .any(|record| record.evidence_id == evidence_id)
            {
                return Err(SpatialMechanismError::UnknownEvidenceReference {
                    evidence_id: evidence_id.to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Stable deterministic identity for transformation configuration/provenance.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut digest = StableDigest::new();
        digest.write_u32(self.schema_version);
        digest.write_str(&self.model_id);
        digest.write_u64(self.transforms.len() as u64);
        for transform in &self.transforms {
            transform.write_digest(&mut digest);
        }
        format!(
            "spatial-mechanisms-v{}-{:016x}",
            self.schema_version,
            digest.finish()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialTargetField {
    MovementCost,
    WaterAccess,
    BaseProductivity,
}

impl SpatialTargetField {
    #[must_use]
    pub const fn expected_role(self) -> LandscapeLayerRole {
        match self {
            Self::MovementCost => LandscapeLayerRole::TerrainTraversal,
            Self::WaterAccess => LandscapeLayerRole::WaterAccessibility,
            Self::BaseProductivity => LandscapeLayerRole::ResourceOpportunity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformDirection {
    Direct,
    Inverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoDataPolicy {
    Reject,
    Constant { value: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialFieldTransform {
    pub target: SpatialTargetField,
    pub source_layer_id: String,
    pub expected_unit: String,
    pub source_domain: LandscapeValueDomain,
    pub target_min: u16,
    pub target_max: u16,
    pub direction: TransformDirection,
    pub nodata: NoDataPolicy,
    /// Optional evidence record supporting the scientific assumption represented by this mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
}

impl SpatialFieldTransform {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        target: SpatialTargetField,
        source_layer_id: impl Into<String>,
        expected_unit: impl Into<String>,
        source_domain: LandscapeValueDomain,
        target_min: u16,
        target_max: u16,
        direction: TransformDirection,
        nodata: NoDataPolicy,
    ) -> Self {
        Self {
            target,
            source_layer_id: source_layer_id.into(),
            expected_unit: expected_unit.into(),
            source_domain,
            target_min,
            target_max,
            direction,
            nodata,
            evidence_id: None,
        }
    }

    #[must_use]
    pub fn with_evidence_id(mut self, evidence_id: impl Into<String>) -> Self {
        self.evidence_id = Some(evidence_id.into());
        self
    }

    fn validate(&self) -> Result<(), SpatialMechanismError> {
        if self.source_layer_id.trim().is_empty() {
            return Err(SpatialMechanismError::EmptyLayerId);
        }
        if self.expected_unit.trim().is_empty() {
            return Err(SpatialMechanismError::EmptyExpectedUnit {
                layer_id: self.source_layer_id.clone(),
            });
        }
        if self
            .evidence_id
            .as_ref()
            .is_some_and(|evidence_id| evidence_id.trim().is_empty())
        {
            return Err(SpatialMechanismError::EmptyEvidenceId {
                layer_id: self.source_layer_id.clone(),
            });
        }
        if self.source_domain.min >= self.source_domain.max {
            return Err(SpatialMechanismError::InvalidSourceDomain {
                layer_id: self.source_layer_id.clone(),
            });
        }
        if self.target_min > self.target_max {
            return Err(SpatialMechanismError::InvalidTargetDomain {
                target: self.target,
                min: self.target_min,
                max: self.target_max,
            });
        }
        match self.target {
            SpatialTargetField::MovementCost => {
                if self.target_min < BASE_MOVEMENT_COST {
                    return Err(SpatialMechanismError::InvalidTargetDomain {
                        target: self.target,
                        min: self.target_min,
                        max: self.target_max,
                    });
                }
            }
            SpatialTargetField::WaterAccess | SpatialTargetField::BaseProductivity => {
                if self.target_max > PERMILLE_MAX {
                    return Err(SpatialMechanismError::InvalidTargetDomain {
                        target: self.target,
                        min: self.target_min,
                        max: self.target_max,
                    });
                }
            }
        }
        if let NoDataPolicy::Constant { value } = self.nodata
            && (value < self.source_domain.min || value > self.source_domain.max)
        {
            return Err(SpatialMechanismError::NoDataConstantOutsideSourceDomain {
                layer_id: self.source_layer_id.clone(),
                value,
            });
        }
        Ok(())
    }

    fn write_digest(&self, digest: &mut StableDigest) {
        digest.write_u8(match self.target {
            SpatialTargetField::MovementCost => 0,
            SpatialTargetField::WaterAccess => 1,
            SpatialTargetField::BaseProductivity => 2,
        });
        digest.write_str(&self.source_layer_id);
        digest.write_str(&self.expected_unit);
        digest.write_i32(self.source_domain.min);
        digest.write_i32(self.source_domain.max);
        digest.write_u16(self.target_min);
        digest.write_u16(self.target_max);
        digest.write_u8(match self.direction {
            TransformDirection::Direct => 0,
            TransformDirection::Inverse => 1,
        });
        match self.nodata {
            NoDataPolicy::Reject => digest.write_u8(0),
            NoDataPolicy::Constant { value } => {
                digest.write_u8(1);
                digest.write_i32(value);
            }
        }
        match &self.evidence_id {
            None => digest.write_u8(0),
            Some(evidence_id) => {
                digest.write_u8(1);
                digest.write_str(evidence_id);
            }
        }
    }
}

/// Deterministic, model-facing M8.4 outputs prior to their application to `World`.
///
/// Missing vectors mean the corresponding model-facing field remains on the existing synthetic
/// baseline. Present vectors contain one value per landscape/world cell in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialMechanismOverlay {
    pub config_identity: String,
    pub movement_cost: Option<Vec<u16>>,
    pub water_access: Option<Vec<u16>>,
    pub base_productivity: Option<Vec<u16>>,
}

pub fn transform_landscape(
    landscape: &LandscapeBundle,
    config: &SpatialMechanismConfig,
) -> Result<SpatialMechanismOverlay, SpatialMechanismError> {
    landscape.validate()?;
    config.validate()?;

    let mut overlay = SpatialMechanismOverlay {
        config_identity: config.identity(),
        movement_cost: None,
        water_access: None,
        base_productivity: None,
    };

    for transform in &config.transforms {
        let layer = landscape.layer(&transform.source_layer_id).ok_or_else(|| {
            SpatialMechanismError::UnknownLayer {
                layer_id: transform.source_layer_id.clone(),
            }
        })?;
        validate_layer_contract(layer, transform)?;
        let values = transform_values(layer, transform)?;
        match transform.target {
            SpatialTargetField::MovementCost => overlay.movement_cost = Some(values),
            SpatialTargetField::WaterAccess => overlay.water_access = Some(values),
            SpatialTargetField::BaseProductivity => overlay.base_productivity = Some(values),
        }
    }

    Ok(overlay)
}

fn validate_layer_contract(
    layer: &LandscapeLayer,
    transform: &SpatialFieldTransform,
) -> Result<(), SpatialMechanismError> {
    let expected_role = transform.target.expected_role();
    if layer.role != expected_role {
        return Err(SpatialMechanismError::UnexpectedLayerRole {
            layer_id: layer.layer_id.clone(),
            expected: expected_role,
            actual: layer.role,
        });
    }
    if layer.unit != transform.expected_unit {
        return Err(SpatialMechanismError::UnexpectedLayerUnit {
            layer_id: layer.layer_id.clone(),
            expected: transform.expected_unit.clone(),
            actual: layer.unit.clone(),
        });
    }
    if layer.value_domain != Some(transform.source_domain) {
        return Err(SpatialMechanismError::UnexpectedLayerDomain {
            layer_id: layer.layer_id.clone(),
            expected: transform.source_domain,
            actual: layer.value_domain,
        });
    }
    Ok(())
}

fn transform_values(
    layer: &LandscapeLayer,
    transform: &SpatialFieldTransform,
) -> Result<Vec<u16>, SpatialMechanismError> {
    let mut output = Vec::with_capacity(layer.values.len());
    for (cell_index, value) in layer.values.iter().enumerate() {
        let source = match value {
            Some(value) => *value,
            None => match transform.nodata {
                NoDataPolicy::Reject => {
                    return Err(SpatialMechanismError::NoDataRejected {
                        layer_id: layer.layer_id.clone(),
                        cell_index: cell_index as u64,
                    });
                }
                NoDataPolicy::Constant { value } => value,
            },
        };
        output.push(linear_map(source, transform)?);
    }
    Ok(output)
}

fn linear_map(value: i32, transform: &SpatialFieldTransform) -> Result<u16, SpatialMechanismError> {
    if value < transform.source_domain.min || value > transform.source_domain.max {
        return Err(SpatialMechanismError::SourceValueOutsideDomain {
            layer_id: transform.source_layer_id.clone(),
            value,
        });
    }

    let source_min = i64::from(transform.source_domain.min);
    let source_max = i64::from(transform.source_domain.max);
    let source_span = source_max - source_min;
    let direct_position = i64::from(value) - source_min;
    let position = match transform.direction {
        TransformDirection::Direct => direct_position,
        TransformDirection::Inverse => source_span - direct_position,
    };
    let target_min = u64::from(transform.target_min);
    let target_span = u64::from(transform.target_max - transform.target_min);
    let numerator = u64::try_from(position)
        .map_err(|_| SpatialMechanismError::ArithmeticOverflow)?
        .checked_mul(target_span)
        .ok_or(SpatialMechanismError::ArithmeticOverflow)?;
    let mapped = target_min
        .checked_add(
            numerator
                / u64::try_from(source_span)
                    .map_err(|_| SpatialMechanismError::ArithmeticOverflow)?,
        )
        .ok_or(SpatialMechanismError::ArithmeticOverflow)?;
    u16::try_from(mapped).map_err(|_| SpatialMechanismError::ArithmeticOverflow)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpatialMechanismError {
    #[error(transparent)]
    Landscape(#[from] LandscapeError),
    #[error(transparent)]
    Evidence(#[from] crate::EvidenceError),
    #[error("spatial mechanism schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("spatial mechanism model identifier is empty")]
    EmptyModelId,
    #[error("spatial mechanism configuration contains no transforms")]
    NoTransforms,
    #[error("spatial mechanism source layer identifier is empty")]
    EmptyLayerId,
    #[error("spatial mechanism source layer {layer_id} has an empty expected unit")]
    EmptyExpectedUnit { layer_id: String },
    #[error("spatial mechanism source layer {layer_id} has an empty evidence identifier")]
    EmptyEvidenceId { layer_id: String },
    #[error("spatial transform references evidence {evidence_id}, but experiment has no evidence catalogue")]
    MissingEvidenceCatalog { evidence_id: String },
    #[error("spatial transform references unknown evidence record {evidence_id}")]
    UnknownEvidenceReference { evidence_id: String },
    #[error("spatial mechanism source layer {layer_id} has an invalid source domain")]
    InvalidSourceDomain { layer_id: String },
    #[error("spatial target {target:?} has invalid target domain {min}..={max}")]
    InvalidTargetDomain {
        target: SpatialTargetField,
        min: u16,
        max: u16,
    },
    #[error("spatial mechanism defines target {0:?} more than once")]
    DuplicateTarget(SpatialTargetField),
    #[error("spatial mechanism references unknown landscape layer {layer_id}")]
    UnknownLayer { layer_id: String },
    #[error("landscape layer {layer_id} has role {actual:?}; expected {expected:?}")]
    UnexpectedLayerRole {
        layer_id: String,
        expected: LandscapeLayerRole,
        actual: LandscapeLayerRole,
    },
    #[error("landscape layer {layer_id} has unit {actual}; expected {expected}")]
    UnexpectedLayerUnit {
        layer_id: String,
        expected: String,
        actual: String,
    },
    #[error("landscape layer {layer_id} has domain {actual:?}; expected {expected:?}")]
    UnexpectedLayerDomain {
        layer_id: String,
        expected: LandscapeValueDomain,
        actual: Option<LandscapeValueDomain>,
    },
    #[error(
        "landscape layer {layer_id} contains nodata at cell {cell_index}, but policy is reject"
    )]
    NoDataRejected { layer_id: String, cell_index: u64 },
    #[error("nodata replacement {value} for layer {layer_id} is outside its source domain")]
    NoDataConstantOutsideSourceDomain { layer_id: String, value: i32 },
    #[error("source value {value} for layer {layer_id} is outside the declared transform domain")]
    SourceValueOutsideDomain { layer_id: String, value: i32 },
    #[error("spatial transformation arithmetic overflow")]
    ArithmeticOverflow,
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

    fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_i32(&mut self, value: i32) {
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
