use std::{cmp::Ordering, collections::BinaryHeap};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::ParameterProvenance,
    focal_region::{FocalRegion, FocalRegionError},
    ids::CellId,
    temporary_mobility::{
        TemporaryMobilityProgramError, TemporaryTravelResolution, TemporaryTravelTable,
    },
    world::{BASE_MOVEMENT_COST, World},
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Versioned M9.4 route-cost assumptions.
///
/// The cost unit is the abstract model-facing `movement_cost` unit. Capacity therefore converts
/// those abstract cost units into integer simulation days; it is not an empirical walking speed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryTravelModel {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    pub travel_capacity_cost_units_per_day: u32,
    pub maximum_traversable_movement_cost: u16,
}

impl TemporaryTravelModel {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        model_id: impl Into<String>,
        provenance: ParameterProvenance,
        travel_capacity_cost_units_per_day: u32,
        maximum_traversable_movement_cost: u16,
    ) -> Result<Self, TemporaryTravelModelError> {
        let model = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: model_id.into(),
            provenance,
            travel_capacity_cost_units_per_day,
            maximum_traversable_movement_cost,
        };
        model.validate()?;
        Ok(model)
    }

    /// Synthetic engine-validation defaults: about three baseline-cost edges per simulation day.
    /// This is deliberately not presented as a calibrated human travel rate.
    #[must_use]
    pub fn synthetic_validation_v1() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: "temporary_travel_synthetic_validation_v1".to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            travel_capacity_cost_units_per_day: 3_000,
            maximum_traversable_movement_cost: u16::MAX,
        }
    }

    pub fn validate(&self) -> Result<(), TemporaryTravelModelError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryTravelModelError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.model_id.trim().is_empty() {
            return Err(TemporaryTravelModelError::EmptyModelId);
        }
        if self.travel_capacity_cost_units_per_day == 0 {
            return Err(TemporaryTravelModelError::ZeroTravelCapacity);
        }
        if self.maximum_traversable_movement_cost < BASE_MOVEMENT_COST {
            return Err(TemporaryTravelModelError::InvalidTraversableCostCeiling {
                ceiling: self.maximum_traversable_movement_cost,
                minimum: BASE_MOVEMENT_COST,
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn identity(&self) -> String {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_str(&mut hash, &self.model_id);
        digest_u64(
            &mut hash,
            match self.provenance {
                ParameterProvenance::EmpiricalDirect => 0,
                ParameterProvenance::EmpiricalDerived => 1,
                ParameterProvenance::EvidenceInformed => 2,
                ParameterProvenance::SyntheticValidation => 3,
                ParameterProvenance::Unresolved => 4,
            },
        );
        digest_u64(
            &mut hash,
            u64::from(self.travel_capacity_cost_units_per_day),
        );
        digest_u64(&mut hash, u64::from(self.maximum_traversable_movement_cost));
        format!(
            "temporary-travel-model-v{}-{:016x}",
            self.schema_version, hash
        )
    }

    #[must_use]
    pub fn is_traversable(&self, world: &World, cell: CellId) -> bool {
        world
            .cell(cell)
            .is_some_and(|cell| cell.movement_cost <= self.maximum_traversable_movement_cost)
    }

    pub fn travel_days(&self, accumulated_cost: u64) -> Result<u32, TemporaryTravelModelError> {
        self.validate()?;
        let capacity = u64::from(self.travel_capacity_cost_units_per_day);
        let days = accumulated_cost.div_ceil(capacity);
        u32::try_from(days).map_err(|_| TemporaryTravelModelError::TravelDurationOverflow {
            accumulated_cost,
            capacity_per_day: self.travel_capacity_cost_units_per_day,
        })
    }

    /// Derive one indexed M9.4 travel table for every authoritative world origin.
    ///
    /// One multi-source search is seeded by all focal-region cells, avoiding a global search per
    /// household. Equal-cost destinations choose the lower authoritative `CellId`.
    pub fn derive_table(
        &self,
        region: &FocalRegion,
        world: &World,
    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {
        self.validate()?;
        region.validate(world)?;
        for &cell in region.member_cells() {
            if !self.is_traversable(world, cell) {
                return Err(TemporaryTravelModelError::RegionCellImpassable { cell });
            }
        }

        let labels = minimum_cost_labels(self, region, world)?;
        let mut resolutions = Vec::with_capacity(world.cell_count());
        let mut accumulated_costs = Vec::with_capacity(world.cell_count());

        for label in labels {
            let Some(label) = label else {
                resolutions.push(TemporaryTravelResolution::Unreachable);
                accumulated_costs.push(None);
                continue;
            };
            let travel_days = self.travel_days(label.cost)?;
            resolutions.push(TemporaryTravelResolution::Reachable {
                destination: label.destination,
                outbound_travel_days: travel_days,
                return_travel_days: travel_days,
            });
            accumulated_costs.push(Some(label.cost));
        }

        TemporaryTravelTable::new_m9_4(resolutions, accumulated_costs, self.clone(), region, world)
            .map_err(TemporaryTravelModelError::TravelTable)
    }
}

impl Default for TemporaryTravelModel {
    fn default() -> Self {
        Self::synthetic_validation_v1()
    }
}

/// Frozen M9.4 symmetric edge formula for two adjacent cells.
pub fn temporary_travel_edge_cost(
    world: &World,
    a: CellId,
    b: CellId,
) -> Result<u64, TemporaryTravelModelError> {
    let a_cell = world
        .cell(a)
        .ok_or(TemporaryTravelModelError::InvalidCell { cell: a })?;
    let b_cell = world
        .cell(b)
        .ok_or(TemporaryTravelModelError::InvalidCell { cell: b })?;
    if !world
        .neighbours4(a)
        .into_iter()
        .flatten()
        .any(|cell| cell == b)
    {
        return Err(TemporaryTravelModelError::CellsNotAdjacent { a, b });
    }
    let sum = u64::from(a_cell.movement_cost) + u64::from(b_cell.movement_cost);
    Ok(sum.div_ceil(2))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RouteLabel {
    cost: u64,
    destination: CellId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueueState {
    label: RouteLabel,
    cell: CellId,
}

impl Ord for QueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        // `BinaryHeap` is a max-heap; reverse every key so the smallest stable tuple wins.
        other
            .label
            .cmp(&self.label)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for QueueState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn minimum_cost_labels(
    model: &TemporaryTravelModel,
    region: &FocalRegion,
    world: &World,
) -> Result<Vec<Option<RouteLabel>>, TemporaryTravelModelError> {
    let mut labels = vec![None; world.cell_count()];
    let mut queue = BinaryHeap::new();

    for &destination in region.member_cells() {
        let index = cell_index(destination, world)?;
        let label = RouteLabel {
            cost: 0,
            destination,
        };
        labels[index] = Some(label);
        queue.push(QueueState {
            label,
            cell: destination,
        });
    }

    while let Some(current) = queue.pop() {
        let current_index = cell_index(current.cell, world)?;
        if labels[current_index] != Some(current.label) {
            continue;
        }

        for neighbour in world.neighbours4(current.cell).into_iter().flatten() {
            if !model.is_traversable(world, neighbour) {
                continue;
            }
            let edge = temporary_travel_edge_cost(world, current.cell, neighbour)?;
            let candidate = RouteLabel {
                cost: current
                    .label
                    .cost
                    .checked_add(edge)
                    .ok_or(TemporaryTravelModelError::AccumulatedCostOverflow)?,
                destination: current.label.destination,
            };
            let neighbour_index = cell_index(neighbour, world)?;
            if labels[neighbour_index].is_none_or(|existing| candidate < existing) {
                labels[neighbour_index] = Some(candidate);
                queue.push(QueueState {
                    label: candidate,
                    cell: neighbour,
                });
            }
        }
    }

    Ok(labels)
}

fn cell_index(cell: CellId, world: &World) -> Result<usize, TemporaryTravelModelError> {
    let index = usize::try_from(
        cell.0
            .checked_sub(1)
            .ok_or(TemporaryTravelModelError::InvalidCell { cell })?,
    )
    .map_err(|_| TemporaryTravelModelError::InvalidCell { cell })?;
    (index < world.cell_count())
        .then_some(index)
        .ok_or(TemporaryTravelModelError::InvalidCell { cell })
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
pub enum TemporaryTravelModelError {
    #[error(
        "temporary travel model schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("temporary travel model identifier is empty")]
    EmptyModelId,
    #[error("temporary travel capacity must be greater than zero")]
    ZeroTravelCapacity,
    #[error(
        "temporary travel maximum traversable movement cost {ceiling} is below world baseline {minimum}"
    )]
    InvalidTraversableCostCeiling { ceiling: u16, minimum: u16 },
    #[error("temporary travel references invalid world cell {cell:?}")]
    InvalidCell { cell: CellId },
    #[error("temporary travel edge requires adjacent cells, got {a:?} and {b:?}")]
    CellsNotAdjacent { a: CellId, b: CellId },
    #[error("focal-region cell {cell:?} is impassable under the temporary travel model")]
    RegionCellImpassable { cell: CellId },
    #[error("temporary travel accumulated cost overflowed u64")]
    AccumulatedCostOverflow,
    #[error(
        "temporary travel duration for cost {accumulated_cost} at capacity {capacity_per_day} exceeds u32 days"
    )]
    TravelDurationOverflow {
        accumulated_cost: u64,
        capacity_per_day: u32,
    },
    #[error(transparent)]
    Region(#[from] FocalRegionError),
    #[error(transparent)]
    TravelTable(#[from] TemporaryMobilityProgramError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::WorldConfig, focal_region::FocalRegionSource, rng::RngFactory};

    fn world(width: u32, height: u32, movement_cost: &[u16]) -> World {
        World::generate(WorldConfig::new(width, height), RngFactory::new(77))
            .unwrap()
            .with_model_field_overlay(Some(movement_cost), None, None)
            .unwrap()
    }

    fn region(world: &World, cells: Vec<CellId>) -> FocalRegion {
        FocalRegion::new("travel-test-region", FocalRegionSource::Synthetic, cells)
            .and_then(|region| {
                region.validate(world)?;
                Ok(region)
            })
            .unwrap()
    }

    #[test]
    fn symmetric_edge_formula_uses_ceil_of_endpoint_mean() {
        let world = world(2, 1, &[1_001, 2_000]);
        let a = CellId::new(1);
        let b = CellId::new(2);
        assert_eq!(temporary_travel_edge_cost(&world, a, b).unwrap(), 1_501);
        assert_eq!(temporary_travel_edge_cost(&world, b, a).unwrap(), 1_501);
    }

    #[test]
    fn route_cost_and_duration_follow_transformed_movement_cost() {
        let baseline = world(3, 1, &[1_000, 1_000, 1_000]);
        let transformed = world(3, 1, &[1_000, 5_000, 1_000]);
        let target = region(&baseline, vec![CellId::new(3)]);
        let transformed_target = region(&transformed, vec![CellId::new(3)]);
        let model = TemporaryTravelModel::new(
            "test-capacity",
            ParameterProvenance::SyntheticValidation,
            2_000,
            u16::MAX,
        )
        .unwrap();

        let baseline_table = model.derive_table(&target, &baseline).unwrap();
        let transformed_table = model
            .derive_table(&transformed_target, &transformed)
            .unwrap();

        assert_eq!(
            baseline_table.accumulated_cost_units(CellId::new(1)),
            Some(2_000)
        );
        assert_eq!(
            transformed_table.accumulated_cost_units(CellId::new(1)),
            Some(6_000)
        );
        assert!(matches!(
            baseline_table.resolution(CellId::new(1)),
            Some(TemporaryTravelResolution::Reachable {
                outbound_travel_days: 1,
                return_travel_days: 1,
                ..
            })
        ));
        assert!(matches!(
            transformed_table.resolution(CellId::new(1)),
            Some(TemporaryTravelResolution::Reachable {
                outbound_travel_days: 3,
                return_travel_days: 3,
                ..
            })
        ));
    }

    #[test]
    fn equal_cost_destinations_choose_lower_cell_id() {
        let world = world(3, 1, &[1_000, 1_000, 1_000]);
        let region = region(&world, vec![CellId::new(1), CellId::new(3)]);
        let table = TemporaryTravelModel::default()
            .derive_table(&region, &world)
            .unwrap();
        assert!(matches!(
            table.resolution(CellId::new(2)),
            Some(TemporaryTravelResolution::Reachable {
                destination,
                ..
            }) if destination == CellId::new(1)
        ));
    }

    #[test]
    fn traversal_ceiling_makes_disconnected_origins_explicitly_unreachable() {
        let world = world(
            3,
            3,
            &[
                1_000, 2_000, 1_000, 1_000, 2_000, 1_000, 1_000, 2_000, 1_000,
            ],
        );
        let region = region(&world, vec![CellId::new(3)]);
        let model = TemporaryTravelModel::new(
            "barrier-test",
            ParameterProvenance::SyntheticValidation,
            3_000,
            1_500,
        )
        .unwrap();
        let table = model.derive_table(&region, &world).unwrap();

        assert_eq!(
            table.resolution(CellId::new(1)),
            Some(TemporaryTravelResolution::Unreachable)
        );
        assert_eq!(table.accumulated_cost_units(CellId::new(1)), None);
        assert!(matches!(
            table.resolution(CellId::new(6)),
            Some(TemporaryTravelResolution::Reachable { .. })
        ));
    }

    #[test]
    fn duration_uses_integer_ceiling() {
        let model = TemporaryTravelModel::new(
            "duration-test",
            ParameterProvenance::SyntheticValidation,
            1_500,
            u16::MAX,
        )
        .unwrap();
        assert_eq!(model.travel_days(0).unwrap(), 0);
        assert_eq!(model.travel_days(1).unwrap(), 1);
        assert_eq!(model.travel_days(1_500).unwrap(), 1);
        assert_eq!(model.travel_days(1_501).unwrap(), 2);
    }
}
