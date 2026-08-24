use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    evidence::EvidenceCatalog,
    ids::{CellId, HouseholdId},
    landscape::{LandscapeBundle, LandscapeError, LandscapeLayerRole, LandscapeValueDomain},
    population::Population,
    world::World,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FocalRegionSource {
    Synthetic,
    LandscapeMask {
        layer_id: String,
        evidence_input_id: String,
    },
}

/// Immutable identity-bearing world-cell set for an M9 temporary-mobility destination region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocalRegion {
    pub schema_version: u32,
    pub region_id: String,
    pub source: FocalRegionSource,
    member_cells: Vec<CellId>,
}

impl FocalRegion {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        region_id: impl Into<String>,
        source: FocalRegionSource,
        mut member_cells: Vec<CellId>,
    ) -> Result<Self, FocalRegionError> {
        member_cells.sort_unstable();
        if let Some(pair) = member_cells.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(FocalRegionError::DuplicateCell { cell: pair[0] });
        }
        let region = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            region_id: region_id.into(),
            source,
            member_cells,
        };
        region.validate_structure()?;
        Ok(region)
    }

    /// Bind a normalized binary auxiliary landscape mask to the core region contract.
    ///
    /// GIS preparation remains external. AnthroSim accepts only an aligned `0`/`1` auxiliary
    /// layer with explicit evidence provenance and rejects nodata or malformed bindings.
    pub fn from_landscape_mask(
        region_id: impl Into<String>,
        landscape: &LandscapeBundle,
        layer_id: &str,
        evidence: &EvidenceCatalog,
        world: &World,
    ) -> Result<Self, FocalRegionBindingError> {
        landscape.validate_evidence_context(Some(evidence))?;
        if landscape.width != world.width() || landscape.height != world.height() {
            return Err(FocalRegionBindingError::GridWorldDimensionMismatch {
                landscape_width: landscape.width,
                landscape_height: landscape.height,
                world_width: world.width(),
                world_height: world.height(),
            });
        }

        let layer =
            landscape
                .layer(layer_id)
                .ok_or_else(|| FocalRegionBindingError::MissingLayer {
                    layer_id: layer_id.to_owned(),
                })?;
        if layer.role != LandscapeLayerRole::Auxiliary {
            return Err(FocalRegionBindingError::MaskLayerNotAuxiliary {
                layer_id: layer.layer_id.clone(),
                role: layer.role,
            });
        }
        let binary_domain = LandscapeValueDomain { min: 0, max: 1 };
        if layer.value_domain != Some(binary_domain) {
            return Err(FocalRegionBindingError::InvalidBinaryDomain {
                layer_id: layer.layer_id.clone(),
                found: layer.value_domain,
            });
        }
        let evidence_input_id = layer.evidence_input_id.clone().ok_or_else(|| {
            FocalRegionBindingError::MissingEvidenceInput {
                layer_id: layer.layer_id.clone(),
            }
        })?;

        let mut member_cells = Vec::new();
        for (index, value) in layer.values.iter().enumerate() {
            match value {
                Some(0) => {}
                Some(1) => member_cells.push(CellId::new(index as u64 + 1)),
                Some(value) => {
                    return Err(FocalRegionBindingError::NonBinaryMaskValue {
                        layer_id: layer.layer_id.clone(),
                        cell_index: index as u64,
                        value: *value,
                    });
                }
                None => {
                    return Err(FocalRegionBindingError::MaskContainsNoData {
                        layer_id: layer.layer_id.clone(),
                        cell_index: index as u64,
                    });
                }
            }
        }

        let region = Self::new(
            region_id,
            FocalRegionSource::LandscapeMask {
                layer_id: layer.layer_id.clone(),
                evidence_input_id,
            },
            member_cells,
        )?;
        region.validate(world)?;
        Ok(region)
    }

    #[must_use]
    pub fn member_cells(&self) -> &[CellId] {
        &self.member_cells
    }

    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.member_cells.len()
    }

    #[must_use]
    pub fn contains(&self, cell: CellId) -> bool {
        self.member_cells.binary_search(&cell).is_ok()
    }

    #[must_use]
    pub fn contains_residence(
        &self,
        household: HouseholdId,
        population: &Population,
    ) -> Option<bool> {
        population
            .household_location(household)
            .map(|residence| self.contains(residence))
    }

    pub fn validate(&self, world: &World) -> Result<(), FocalRegionError> {
        self.validate_structure()?;
        for &cell in &self.member_cells {
            if world.cell(cell).is_none() {
                return Err(FocalRegionError::CellOutsideWorld { cell });
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_str(&mut hash, &self.region_id);
        match &self.source {
            FocalRegionSource::Synthetic => digest_u64(&mut hash, 0),
            FocalRegionSource::LandscapeMask {
                layer_id,
                evidence_input_id,
            } => {
                digest_u64(&mut hash, 1);
                digest_str(&mut hash, layer_id);
                digest_str(&mut hash, evidence_input_id);
            }
        }
        digest_u64(&mut hash, self.member_cells.len() as u64);
        for cell in &self.member_cells {
            digest_u64(&mut hash, cell.0);
        }
        hash
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "focal-region-v{}-{:016x}",
            self.schema_version,
            self.digest64()
        )
    }

    fn validate_structure(&self) -> Result<(), FocalRegionError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(FocalRegionError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.region_id.trim().is_empty() {
            return Err(FocalRegionError::EmptyRegionId);
        }
        if self.member_cells.is_empty() {
            return Err(FocalRegionError::EmptyRegion);
        }
        if self.member_cells.contains(&CellId::INVALID) {
            return Err(FocalRegionError::InvalidCellId);
        }
        if self.member_cells.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(FocalRegionError::NonCanonicalCells);
        }
        if let FocalRegionSource::LandscapeMask {
            layer_id,
            evidence_input_id,
        } = &self.source
        {
            if layer_id.trim().is_empty() {
                return Err(FocalRegionError::EmptyLayerId);
            }
            if evidence_input_id.trim().is_empty() {
                return Err(FocalRegionError::EmptyEvidenceInputId);
            }
        }
        Ok(())
    }
}

fn digest_str(hash: &mut u64, value: &str) {
    digest_u64(hash, value.len() as u64);
    for byte in value.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FocalRegionError {
    #[error("focal region schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("focal region identifier is empty")]
    EmptyRegionId,
    #[error("focal region contains no cells")]
    EmptyRegion,
    #[error("focal region contains invalid cell ID 0")]
    InvalidCellId,
    #[error("focal region contains duplicate cell {cell:?}")]
    DuplicateCell { cell: CellId },
    #[error("focal region member cells are not strictly increasing and duplicate-free")]
    NonCanonicalCells,
    #[error("focal region cell {cell:?} is outside the authoritative world")]
    CellOutsideWorld { cell: CellId },
    #[error("focal region landscape-mask source has an empty layer identifier")]
    EmptyLayerId,
    #[error("focal region landscape-mask source has an empty evidence input identifier")]
    EmptyEvidenceInputId,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FocalRegionBindingError {
    #[error(transparent)]
    Landscape(#[from] LandscapeError),
    #[error(transparent)]
    Region(#[from] FocalRegionError),
    #[error("focal-region landscape mask layer {layer_id} does not exist")]
    MissingLayer { layer_id: String },
    #[error(
        "focal-region landscape dimensions {landscape_width}x{landscape_height} do not match world dimensions {world_width}x{world_height}"
    )]
    GridWorldDimensionMismatch {
        landscape_width: u32,
        landscape_height: u32,
        world_width: u32,
        world_height: u32,
    },
    #[error("focal-region mask layer {layer_id} must have auxiliary role, found {role:?}")]
    MaskLayerNotAuxiliary {
        layer_id: String,
        role: LandscapeLayerRole,
    },
    #[error("focal-region mask layer {layer_id} must declare valueDomain 0..=1, found {found:?}")]
    InvalidBinaryDomain {
        layer_id: String,
        found: Option<LandscapeValueDomain>,
    },
    #[error("focal-region mask layer {layer_id} has no evidenceInputId")]
    MissingEvidenceInput { layer_id: String },
    #[error("focal-region mask layer {layer_id} has nodata at cell index {cell_index}")]
    MaskContainsNoData { layer_id: String, cell_index: u64 },
    #[error(
        "focal-region mask layer {layer_id} has non-binary value {value} at cell index {cell_index}"
    )]
    NonBinaryMaskValue {
        layer_id: String,
        cell_index: u64,
        value: i32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{ParameterProvenance, PopulationConfig, WorldConfig},
        evidence::{EvidenceRecord, EvidenceSource, ExternalInputEvidence},
        landscape::{GridGeometry, LandscapeLayer},
        rng::RngFactory,
    };

    fn world() -> World {
        World::generate(WorldConfig::new(4, 4), RngFactory::new(7)).unwrap()
    }

    fn evidence_catalog() -> EvidenceCatalog {
        EvidenceCatalog::new(vec![EvidenceRecord {
            schema_version: EvidenceRecord::CURRENT_SCHEMA_VERSION,
            evidence_id: "mask-source".to_owned(),
            provenance: ParameterProvenance::EmpiricalDerived,
            source: EvidenceSource {
                source_id: "mask-dataset".to_owned(),
                citation: "Example focal-region evidence".to_owned(),
                persistent_id: None,
                dataset_version: Some("v1".to_owned()),
                licence: Some("example".to_owned()),
                spatial_coverage: Some("validation grid".to_owned()),
                temporal_coverage: None,
            },
            original_variable: "region membership".to_owned(),
            original_units: "binary".to_owned(),
            transformation: None,
            simulation_units: "binary mask".to_owned(),
            uncertainty: None,
            applicability: "M9 focal-region validation".to_owned(),
            competing_estimates: Vec::new(),
        }])
        .with_external_inputs(vec![ExternalInputEvidence {
            input_id: "region-mask-input".to_owned(),
            evidence_id: "mask-source".to_owned(),
            format: "normalized-json-grid".to_owned(),
            spatial_reference: Some("EPSG:27700".to_owned()),
            content_digest: Some("sha256:example".to_owned()),
        }])
    }

    fn landscape(values: Vec<Option<i32>>) -> LandscapeBundle {
        LandscapeBundle::new(
            4,
            4,
            GridGeometry {
                origin_x: 0,
                origin_y: 0,
                cell_size_x: 10,
                cell_size_y: 10,
                coordinate_unit: "metre".to_owned(),
                spatial_reference: "EPSG:27700".to_owned(),
            },
            vec![LandscapeLayer {
                layer_id: "region-mask".to_owned(),
                role: LandscapeLayerRole::Auxiliary,
                unit: "binary".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 1 }),
                evidence_input_id: Some("region-mask-input".to_owned()),
                values,
            }],
        )
    }

    #[test]
    fn synthetic_region_is_canonical_and_duplicate_fail_closed() {
        let region = FocalRegion::new(
            "region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4), CellId::new(2), CellId::new(3)],
        )
        .unwrap();
        assert_eq!(
            region.member_cells(),
            &[CellId::new(2), CellId::new(3), CellId::new(4)]
        );
        assert_eq!(
            FocalRegion::new(
                "duplicate",
                FocalRegionSource::Synthetic,
                vec![CellId::new(4), CellId::new(2), CellId::new(4)]
            ),
            Err(FocalRegionError::DuplicateCell {
                cell: CellId::new(4)
            })
        );
    }

    #[test]
    fn identity_is_order_independent_and_serialization_stable() {
        let first = FocalRegion::new(
            "region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(1), CellId::new(4), CellId::new(2)],
        )
        .unwrap();
        let second = FocalRegion::new(
            "region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4), CellId::new(2), CellId::new(1)],
        )
        .unwrap();
        assert_eq!(first.identity(), second.identity());
        let restored: FocalRegion =
            serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
        assert_eq!(restored.identity(), first.identity());
    }

    #[test]
    fn evidence_bound_binary_mask_derives_region() {
        let mut values = vec![Some(0); 16];
        values[1] = Some(1);
        values[5] = Some(1);
        values[14] = Some(1);
        let region = FocalRegion::from_landscape_mask(
            "region",
            &landscape(values),
            "region-mask",
            &evidence_catalog(),
            &world(),
        )
        .unwrap();
        assert_eq!(
            region.member_cells(),
            &[CellId::new(2), CellId::new(6), CellId::new(15)]
        );
        assert_eq!(
            region.source,
            FocalRegionSource::LandscapeMask {
                layer_id: "region-mask".to_owned(),
                evidence_input_id: "region-mask-input".to_owned(),
            }
        );
    }

    #[test]
    fn mask_nodata_and_empty_region_are_rejected() {
        let mut nodata = vec![Some(0); 16];
        nodata[3] = None;
        assert!(matches!(
            FocalRegion::from_landscape_mask(
                "region",
                &landscape(nodata),
                "region-mask",
                &evidence_catalog(),
                &world(),
            ),
            Err(FocalRegionBindingError::MaskContainsNoData { cell_index: 3, .. })
        ));
        assert_eq!(
            FocalRegion::from_landscape_mask(
                "empty",
                &landscape(vec![Some(0); 16]),
                "region-mask",
                &evidence_catalog(),
                &world(),
            ),
            Err(FocalRegionBindingError::Region(
                FocalRegionError::EmptyRegion
            ))
        );
    }

    #[test]
    fn missing_evidence_link_is_rejected() {
        let mut ungrounded = landscape(vec![Some(1); 16]);
        ungrounded.layers[0].evidence_input_id = None;
        assert_eq!(
            FocalRegion::from_landscape_mask(
                "region",
                &ungrounded,
                "region-mask",
                &evidence_catalog(),
                &world(),
            ),
            Err(FocalRegionBindingError::MissingEvidenceInput {
                layer_id: "region-mask".to_owned()
            })
        );
    }

    #[test]
    fn residents_are_identifiable_without_degenerate_visits() {
        let world = world();
        let population = Population::initialize(
            PopulationConfig::new(20).with_target_household_size(5),
            &world,
            RngFactory::new(11),
        )
        .unwrap();
        let household = HouseholdId::new(1);
        let residence = population.household_location(household).unwrap();
        let region =
            FocalRegion::new("home", FocalRegionSource::Synthetic, vec![residence]).unwrap();
        assert_eq!(
            region.contains_residence(household, &population),
            Some(true)
        );
    }

    #[test]
    fn invalid_world_cells_fail_closed() {
        assert_eq!(
            FocalRegion::new("invalid", FocalRegionSource::Synthetic, vec![CellId::INVALID]),
            Err(FocalRegionError::InvalidCellId)
        );
        let outside = FocalRegion::new(
            "outside",
            FocalRegionSource::Synthetic,
            vec![CellId::new(99)],
        )
        .unwrap();
        assert_eq!(
            outside.validate(&world()),
            Err(FocalRegionError::CellOutsideWorld {
                cell: CellId::new(99)
            })
        );
    }
}
