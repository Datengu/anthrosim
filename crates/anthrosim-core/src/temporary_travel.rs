use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::ParameterProvenance,
    focal_region::{FocalRegion, FocalRegionError},
    ids::CellId,
    temporary_mobility::{
        TemporaryMobilityProgramError, TemporaryTravelDestinationCandidate,
        TemporaryTravelResolution, TemporaryTravelTable,
    },
    world::{BASE_MOVEMENT_COST, World},
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Versioned spatial state used only to couple otherwise exchangeable M9 destination ties.
///
/// Movement cost is always present because it is authoritative M9 travel state. Spatial-host
/// runs additionally opt in model-facing fields that were explicitly supplied by M8 transforms;
/// residual synthetic fields are deliberately excluded from that host-level correspondence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TemporaryDestinationCouplingContextKind {
    CoreWorldMovementV1,
    SpatialDeclaredFieldsV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TemporaryDestinationCouplingContext {
    pub schema_version: u32,
    pub kind: TemporaryDestinationCouplingContextKind,
    pub include_water_access: bool,
    pub include_base_productivity: bool,
}

impl TemporaryDestinationCouplingContext {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub(crate) const fn core_world_movement_v1() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            kind: TemporaryDestinationCouplingContextKind::CoreWorldMovementV1,
            include_water_access: false,
            include_base_productivity: false,
        }
    }

    #[must_use]
    pub(crate) const fn spatial_declared_fields_v1(
        include_water_access: bool,
        include_base_productivity: bool,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            kind: TemporaryDestinationCouplingContextKind::SpatialDeclaredFieldsV1,
            include_water_access,
            include_base_productivity,
        }
    }

    #[must_use]
    pub(crate) const fn is_valid(self) -> bool {
        self.schema_version == Self::CURRENT_SCHEMA_VERSION
            && match self.kind {
                TemporaryDestinationCouplingContextKind::CoreWorldMovementV1 => {
                    !self.include_water_access && !self.include_base_productivity
                }
                TemporaryDestinationCouplingContextKind::SpatialDeclaredFieldsV1 => true,
            }
    }

    #[must_use]
    pub(crate) const fn kind_rank(self) -> u64 {
        match self.kind {
            TemporaryDestinationCouplingContextKind::CoreWorldMovementV1 => 0,
            TemporaryDestinationCouplingContextKind::SpatialDeclaredFieldsV1 => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridReflection {
    Identity,
    Horizontal,
    Vertical,
    Both,
}

const GRID_REFLECTIONS: [GridReflection; 4] = [
    GridReflection::Identity,
    GridReflection::Horizontal,
    GridReflection::Vertical,
    GridReflection::Both,
];

/// All grid-reflection frames that realize the exact lexicographically minimal declared
/// model-facing spatial state. More than one frame means the declared state itself has an
/// automorphism; destinations related inside that automorphism remain one exchangeable class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporaryDestinationCanonicalFrame {
    reflections: Vec<GridReflection>,
}

pub(crate) fn destination_canonical_frame(
    world: &World,
    context: TemporaryDestinationCouplingContext,
) -> Option<TemporaryDestinationCanonicalFrame> {
    if !context.is_valid() || world.width() == 0 || world.height() == 0 {
        return None;
    }
    let mut best_signature: Option<Vec<u16>> = None;
    let mut reflections = Vec::new();
    for reflection in GRID_REFLECTIONS {
        let signature = reflected_context_signature(world, context, reflection)?;
        match best_signature.as_ref() {
            None => {
                best_signature = Some(signature);
                reflections.push(reflection);
            }
            Some(best) => match signature.cmp(best) {
                Ordering::Less => {
                    best_signature = Some(signature);
                    reflections.clear();
                    reflections.push(reflection);
                }
                Ordering::Equal => reflections.push(reflection),
                Ordering::Greater => {}
            },
        }
    }
    (!reflections.is_empty()).then_some(TemporaryDestinationCanonicalFrame { reflections })
}

pub(crate) fn destination_coupling_classes(
    world: &World,
    frame: &TemporaryDestinationCanonicalFrame,
    origin: CellId,
    candidates: &[TemporaryTravelDestinationCandidate],
) -> Option<Vec<u32>> {
    let mut keyed = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.iter().enumerate() {
        let mut best_pair: Option<(u64, u64, u32)> = None;
        for &reflection in &frame.reflections {
            let mapped_origin = reflected_cell(world, reflection, origin)?;
            let mapped_destination = reflected_cell(world, reflection, candidate.destination)?;
            let key = (
                mapped_origin.0,
                mapped_destination.0,
                candidate.route_distance_edges,
            );
            best_pair = Some(best_pair.map_or(key, |prior| prior.min(key)));
        }
        keyed.push((best_pair?, index));
    }
    keyed.sort_unstable_by_key(|(key, index)| (*key, *index));

    let mut classes = vec![0_u32; candidates.len()];
    let mut previous = None;
    let mut class = 0_u32;
    for (key, index) in keyed {
        if previous.is_some_and(|prior| prior != key) {
            class = class.checked_add(1)?;
        }
        classes[index] = class;
        previous = Some(key);
    }
    Some(classes)
}

fn reflected_context_signature(
    world: &World,
    context: TemporaryDestinationCouplingContext,
    reflection: GridReflection,
) -> Option<Vec<u16>> {
    let fields_per_cell = 1
        + if context.include_water_access { 1 } else { 0 }
        + if context.include_base_productivity {
            1
        } else {
            0
        };
    let capacity = world.cell_count().checked_mul(fields_per_cell)?;
    let mut signature = Vec::with_capacity(capacity);
    for y in 0..world.height() {
        for x in 0..world.width() {
            let (source_x, source_y) =
                reflected_coordinates(reflection, x, y, world.width(), world.height());
            let source = world.cell_id(source_x, source_y)?;
            let cell = world.cell(source)?;
            signature.push(cell.movement_cost);
            if context.include_water_access {
                signature.push(cell.water_access);
            }
            if context.include_base_productivity {
                signature.push(cell.base_productivity);
            }
        }
    }
    Some(signature)
}

fn reflected_cell(world: &World, reflection: GridReflection, cell: CellId) -> Option<CellId> {
    let (x, y) = world.coordinates(cell)?;
    let (x, y) = reflected_coordinates(reflection, x, y, world.width(), world.height());
    world.cell_id(x, y)
}

fn reflected_coordinates(
    reflection: GridReflection,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> (u32, u32) {
    match reflection {
        GridReflection::Identity => (x, y),
        GridReflection::Horizontal => (width - 1 - x, y),
        GridReflection::Vertical => (x, height - 1 - y),
        GridReflection::Both => (width - 1 - x, height - 1 - y),
    }
}

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
    /// The public helper uses a zero tie seed for callers that only need static travel geometry.
    /// Authoritative simulations call `derive_table_with_tie_seed` with the experiment seed.
    pub fn derive_table(
        &self,
        region: &FocalRegion,
        world: &World,
    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {
        self.derive_table_with_tie_seed(region, world, 0)
    }

    /// Derive M9.4 travel geometry while preserving every exactly equal minimum destination.
    ///
    /// `destination_tie_seed` does not affect route cost or reachability. It is retained in the
    /// table solely so M9 execution can resolve an equal-cost destination with the declared keyed
    /// tie policy without consuming any sequential RNG stream.
    pub fn derive_table_with_tie_seed(
        &self,
        region: &FocalRegion,
        world: &World,
        destination_tie_seed: u64,
    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {
        self.derive_table_with_tie_seed_and_coupling_context(
            region,
            world,
            destination_tie_seed,
            TemporaryDestinationCouplingContext::core_world_movement_v1(),
        )
    }

    pub(crate) fn derive_table_with_tie_seed_and_coupling_context(
        &self,
        region: &FocalRegion,
        world: &World,
        destination_tie_seed: u64,
        destination_coupling_context: TemporaryDestinationCouplingContext,
    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {
        self.validate()?;
        region.validate(world)?;
        if !destination_coupling_context.is_valid() {
            return Err(TemporaryTravelModelError::InvalidDestinationCouplingContext);
        }
        for &cell in region.member_cells() {
            if !self.is_traversable(world, cell) {
                return Err(TemporaryTravelModelError::RegionCellImpassable { cell });
            }
        }
        let canonical_frame = destination_canonical_frame(world, destination_coupling_context)
            .ok_or(TemporaryTravelModelError::InvalidDestinationCouplingContext)?;

        let labels = minimum_cost_labels(self, region, world)?;
        let mut resolutions = Vec::with_capacity(world.cell_count());
        let mut accumulated_costs = Vec::with_capacity(world.cell_count());
        let mut equal_cost_destinations = Vec::with_capacity(world.cell_count());

        for (origin_index, label) in labels.into_iter().enumerate() {
            let origin = CellId::new(origin_index as u64 + 1);
            let Some(label) = label else {
                resolutions.push(TemporaryTravelResolution::Unreachable);
                accumulated_costs.push(None);
                equal_cost_destinations.push(Vec::new());
                continue;
            };
            let mut candidates = label
                .destinations
                .into_iter()
                .map(
                    |(destination, route_distance_edges)| TemporaryTravelDestinationCandidate {
                        destination,
                        route_distance_edges,
                        destination_coupling_class: 0,
                    },
                )
                .collect::<Vec<_>>();
            let classes =
                destination_coupling_classes(world, &canonical_frame, origin, &candidates).ok_or(
                    TemporaryTravelModelError::DestinationCouplingClassificationFailed { origin },
                )?;
            for (candidate, class) in candidates.iter_mut().zip(classes) {
                candidate.destination_coupling_class = class;
            }
            let destination = candidates
                .first()
                .expect("reachable M9.4 label must retain at least one destination")
                .destination;
            let travel_days = self.travel_days(label.cost)?;
            resolutions.push(TemporaryTravelResolution::Reachable {
                destination,
                outbound_travel_days: travel_days,
                return_travel_days: travel_days,
            });
            accumulated_costs.push(Some(label.cost));
            equal_cost_destinations.push(candidates);
        }

        TemporaryTravelTable::new_m9_4(
            resolutions,
            accumulated_costs,
            equal_cost_destinations,
            destination_tie_seed,
            destination_coupling_context,
            self.clone(),
            region,
            world,
        )
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteLabel {
    cost: u64,
    destinations: BTreeMap<CellId, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueueState {
    cost: u64,
    cell: CellId,
}

impl Ord for QueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        // `BinaryHeap` is a max-heap; reverse cost and cell for deterministic minimum-first work.
        other
            .cost
            .cmp(&self.cost)
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
        let mut destinations = BTreeMap::new();
        destinations.insert(destination, 0);
        labels[index] = Some(RouteLabel {
            cost: 0,
            destinations,
        });
        queue.push(QueueState {
            cost: 0,
            cell: destination,
        });
    }

    while let Some(current) = queue.pop() {
        let current_index = cell_index(current.cell, world)?;
        let Some(current_label) = labels[current_index].as_ref() else {
            continue;
        };
        if current_label.cost != current.cost {
            continue;
        }
        let current_destinations = current_label.destinations.clone();

        for neighbour in world.neighbours4(current.cell).into_iter().flatten() {
            if !model.is_traversable(world, neighbour) {
                continue;
            }
            let edge = temporary_travel_edge_cost(world, current.cell, neighbour)?;
            let candidate_cost = current
                .cost
                .checked_add(edge)
                .ok_or(TemporaryTravelModelError::AccumulatedCostOverflow)?;
            let mut candidate_destinations = BTreeMap::new();
            for (destination, hops) in &current_destinations {
                candidate_destinations.insert(
                    *destination,
                    hops.checked_add(1)
                        .ok_or(TemporaryTravelModelError::RouteDistanceOverflow)?,
                );
            }
            let neighbour_index = cell_index(neighbour, world)?;
            let mut changed = false;
            match labels[neighbour_index].as_mut() {
                None => {
                    labels[neighbour_index] = Some(RouteLabel {
                        cost: candidate_cost,
                        destinations: candidate_destinations,
                    });
                    changed = true;
                }
                Some(existing) if candidate_cost < existing.cost => {
                    *existing = RouteLabel {
                        cost: candidate_cost,
                        destinations: candidate_destinations,
                    };
                    changed = true;
                }
                Some(existing) if candidate_cost == existing.cost => {
                    for (destination, candidate_hops) in candidate_destinations {
                        match existing.destinations.get_mut(&destination) {
                            None => {
                                existing.destinations.insert(destination, candidate_hops);
                                changed = true;
                            }
                            Some(existing_hops) if candidate_hops < *existing_hops => {
                                *existing_hops = candidate_hops;
                                changed = true;
                            }
                            _ => {}
                        }
                    }
                }
                Some(_) => {}
            }
            if changed {
                queue.push(QueueState {
                    cost: candidate_cost,
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
    #[error("temporary travel destination coupling context is invalid")]
    InvalidDestinationCouplingContext,
    #[error("temporary travel destination coupling classification failed for origin {origin:?}")]
    DestinationCouplingClassificationFailed { origin: CellId },
    #[error("temporary travel references invalid world cell {cell:?}")]
    InvalidCell { cell: CellId },
    #[error("temporary travel edge requires adjacent cells, got {a:?} and {b:?}")]
    CellsNotAdjacent { a: CellId, b: CellId },
    #[error("focal-region cell {cell:?} is impassable under the temporary travel model")]
    RegionCellImpassable { cell: CellId },
    #[error("temporary travel accumulated cost overflowed u64")]
    AccumulatedCostOverflow,
    #[error("temporary travel minimum-cost route distance exceeds u32 edges")]
    RouteDistanceOverflow,
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
    fn equal_cost_destinations_preserve_all_minima() {
        let world = world(3, 1, &[1_000, 1_000, 1_000]);
        let region = region(&world, vec![CellId::new(1), CellId::new(3)]);
        let table = TemporaryTravelModel::default()
            .derive_table(&region, &world)
            .unwrap();
        let candidates = table.equal_cost_destinations(CellId::new(2)).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].destination, CellId::new(1));
        assert_eq!(candidates[1].destination, CellId::new(3));
        assert_eq!(candidates[0].route_distance_edges, 1);
        assert_eq!(candidates[1].route_distance_edges, 1);
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
