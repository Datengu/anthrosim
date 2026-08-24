use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ids::CellId, world::World};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Provenance for the model-facing cell set used as an M9 focal region.
///
/// `LandscapeMask` records the normalized source layer that produced the cell set. It does not
/// make AnthroSim responsible for drawing, rasterizing, reprojecting or otherwise editing GIS
/// geometry; those operations remain outside the simulation engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FocalRegionSource {
    Synthetic,
    LandscapeMask {
        layer_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        evidence_input_id: Option<String>,
    },
}

/// Immutable identity-bearing set of world cells that a temporary-mobility experiment may target.
///
/// Member cells are stored in strictly increasing `CellId` order. The constructor canonicalizes an
/// ordinary cell list, while `validate` rejects non-canonical serialized state so content identity
/// cannot depend on input ordering or duplicate cells.
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
        member_cells.dedup();
        let region = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            region_id: region_id.into(),
            source,
            member_cells,
        };
        region.validate_structure()?;
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
                match evidence_input_id {
                    Some(input_id) => {
                        digest_u64(&mut hash, 1);
                        digest_str(&mut hash, input_id);
                    }
                    None => digest_u64(&mut hash, 0),
                }
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
        format!("focal-region-v{}-{:016x}", self.schema_version, self.digest64())
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
        if self
            .member_cells
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(FocalRegionError::NonCanonicalCells);
        }
        match &self.source {
            FocalRegionSource::Synthetic => {}
            FocalRegionSource::LandscapeMask {
                layer_id,
                evidence_input_id,
            } => {
                if layer_id.trim().is_empty() {
                    return Err(FocalRegionError::EmptyLayerId);
                }
                if evidence_input_id
                    .as_deref()
                    .is_some_and(|input_id| input_id.trim().is_empty())
                {
                    return Err(FocalRegionError::EmptyEvidenceInputId);
                }
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
    #[error("focal region member cells are not strictly increasing and duplicate-free")]
    NonCanonicalCells,
    #[error("focal region cell {cell:?} is outside the authoritative world")]
    CellOutsideWorld { cell: CellId },
    #[error("focal region landscape-mask source has an empty layer identifier")]
    EmptyLayerId,
    #[error("focal region landscape-mask source has an empty evidence input identifier")]
    EmptyEvidenceInputId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::WorldConfig, rng::RngFactory};

    fn world() -> World {
        World::generate(WorldConfig::new(4, 4), RngFactory::new(7)).unwrap()
    }

    #[test]
    fn constructor_canonicalizes_input_cells() {
        let region = FocalRegion::new(
            "gathering-area",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4), CellId::new(2), CellId::new(4), CellId::new(3)],
        )
        .unwrap();

        assert_eq!(
            region.member_cells(),
            &[CellId::new(2), CellId::new(3), CellId::new(4)]
        );
        region.validate(&world()).unwrap();
    }

    #[test]
    fn identity_is_independent_of_input_order_and_duplicates() {
        let first = FocalRegion::new(
            "region-a",
            FocalRegionSource::Synthetic,
            vec![CellId::new(1), CellId::new(4), CellId::new(2)],
        )
        .unwrap();
        let second = FocalRegion::new(
            "region-a",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4), CellId::new(2), CellId::new(1), CellId::new(2)],
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.digest64(), second.digest64());
        assert_eq!(first.identity(), second.identity());
    }

    #[test]
    fn different_provenance_changes_identity() {
        let cells = vec![CellId::new(1), CellId::new(2)];
        let synthetic = FocalRegion::new(
            "region-a",
            FocalRegionSource::Synthetic,
            cells.clone(),
        )
        .unwrap();
        let evidence_bound = FocalRegion::new(
            "region-a",
            FocalRegionSource::LandscapeMask {
                layer_id: "aggregation-mask".to_owned(),
                evidence_input_id: Some("survey-derived-mask".to_owned()),
            },
            cells,
        )
        .unwrap();

        assert_ne!(synthetic.identity(), evidence_bound.identity());
    }

    #[test]
    fn rejects_empty_or_out_of_world_regions() {
        assert_eq!(
            FocalRegion::new("empty", FocalRegionSource::Synthetic, vec![]),
            Err(FocalRegionError::EmptyRegion)
        );

        let region = FocalRegion::new(
            "outside",
            FocalRegionSource::Synthetic,
            vec![CellId::new(99)],
        )
        .unwrap();
        assert_eq!(
            region.validate(&world()),
            Err(FocalRegionError::CellOutsideWorld {
                cell: CellId::new(99)
            })
        );
    }

    #[test]
    fn contains_uses_canonical_binary_search() {
        let region = FocalRegion::new(
            "region-a",
            FocalRegionSource::Synthetic,
            vec![CellId::new(7), CellId::new(3), CellId::new(5)],
        )
        .unwrap();

        assert!(region.contains(CellId::new(5)));
        assert!(!region.contains(CellId::new(6)));
    }
}
