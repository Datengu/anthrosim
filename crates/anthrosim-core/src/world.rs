use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::WorldConfig, ids::CellId, rng::RngFactory};

pub const PERMILLE_MAX: u16 = 1_000;
pub const BASE_MOVEMENT_COST: u16 = 1_000;
pub const INITIAL_FOOD_STOCK_MULTIPLIER: u32 = 10;

/// Compact authoritative state for one synthetic world cell.
///
/// Dimensionless environmental quantities are stored as permille integers
/// (0..=1000) rather than floats to make replay and state digests exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    /// Synthetic relative elevation in the range -500..=500.
    pub elevation: i16,
    /// Relative traversal cost where 1000 is the cheapest baseline.
    pub movement_cost: u16,
    /// Synthetic water accessibility, 0..=1000.
    pub water_access: u16,
    /// Synthetic renewable food productivity, 0..=1000.
    pub base_productivity: u16,
    /// Abstract resource units available at initialization.
    pub food_stock: u32,
    /// Day of year at which seasonal productivity is conceptually centred.
    pub season_phase_days: u16,
    /// Seasonal variation strength, 0..=1000.
    pub season_amplitude: u16,
    /// Temporary environmental stress, 0..=1000. Starts at zero in M1.
    pub environmental_stress: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    schema_version: u32,
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl World {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn generate(config: WorldConfig, rng_factory: RngFactory) -> Result<Self, WorldError> {
        validate_config(config)?;

        let cell_count_u64 = u64::from(config.width) * u64::from(config.height);
        let cell_count = usize::try_from(cell_count_u64)
            .map_err(|_| WorldError::CellCountTooLarge { cell_count: cell_count_u64 })?;

        let mut world_rng = rng_factory.stream("world");
        let elevation_seed = world_rng.next_u64();
        let wetness_seed = world_rng.next_u64();
        let fertility_seed = world_rng.next_u64();
        let ruggedness_seed = world_rng.next_u64();
        let climate_seed = world_rng.next_u64();

        let mut cells = Vec::new();
        cells
            .try_reserve_exact(cell_count)
            .map_err(|_| WorldError::AllocationFailed { cell_count: cell_count_u64 })?;

        for y in 0..config.height {
            for x in 0..config.width {
                let elevation_raw = coherent_field(elevation_seed, x, y);
                let elevation = i16::try_from(i32::from(elevation_raw) - 500)
                    .expect("coherent field is constrained to 0..=1000");
                let abs_elevation = i32::from(elevation).unsigned_abs();
                let lowland_favourability = 1_000_u32.saturating_sub(abs_elevation * 2);

                let wetness = u32::from(coherent_field(wetness_seed, x, y));
                let water_access = ((wetness * 3 + lowland_favourability) / 4) as u16;

                let fertility = u32::from(coherent_field(fertility_seed, x, y));
                let productivity =
                    ((u32::from(water_access) * 5 + fertility * 3 + lowland_favourability * 2)
                        / 10) as u16;

                let ruggedness = u32::from(coherent_field(ruggedness_seed, x, y));
                let movement_cost = (u32::from(BASE_MOVEMENT_COST)
                    + ruggedness * 2
                    + abs_elevation)
                    .min(u32::from(u16::MAX)) as u16;

                let climate = u32::from(coherent_field(climate_seed, x, y));
                let latitude = synthetic_latitude_permille(y, config.height);
                let season_amplitude =
                    (100 + latitude * 700 / 1_000 + climate * 200 / 1_000).min(1_000) as u16;
                let season_phase_days = if y.saturating_mul(2) < config.height {
                    0
                } else {
                    182
                };

                cells.push(Cell {
                    elevation,
                    movement_cost,
                    water_access,
                    base_productivity: productivity,
                    food_stock: u32::from(productivity) * INITIAL_FOOD_STOCK_MULTIPLIER,
                    season_phase_days,
                    season_amplitude,
                    environmental_stress: 0,
                });
            }
        }

        let world = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            width: config.width,
            height: config.height,
            cells,
        };
        world.validate()?;
        Ok(world)
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[must_use]
    pub fn cell_id(&self, x: u32, y: u32) -> Option<CellId> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = u64::from(y) * u64::from(self.width) + u64::from(x);
        Some(CellId::new(index + 1))
    }

    #[must_use]
    pub fn coordinates(&self, id: CellId) -> Option<(u32, u32)> {
        let index = id.0.checked_sub(1)?;
        if index >= self.cells.len() as u64 {
            return None;
        }
        let width = u64::from(self.width);
        let x = u32::try_from(index % width).ok()?;
        let y = u32::try_from(index / width).ok()?;
        Some((x, y))
    }

    #[must_use]
    pub fn cell(&self, id: CellId) -> Option<&Cell> {
        let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
        self.cells.get(index)
    }

    /// Four-neighbour lookup in stable north, east, south, west order.
    #[must_use]
    pub fn neighbours4(&self, id: CellId) -> [Option<CellId>; 4] {
        let Some((x, y)) = self.coordinates(id) else {
            return [None; 4];
        };

        [
            y.checked_sub(1).and_then(|north| self.cell_id(x, north)),
            x.checked_add(1).and_then(|east| self.cell_id(east, y)),
            y.checked_add(1).and_then(|south| self.cell_id(x, south)),
            x.checked_sub(1).and_then(|west| self.cell_id(west, y)),
        ]
    }

    pub fn validate(&self) -> Result<(), WorldValidationError> {
        let expected = u64::from(self.width) * u64::from(self.height);
        if self.cells.len() as u64 != expected {
            return Err(WorldValidationError::CellCountMismatch {
                expected,
                actual: self.cells.len() as u64,
            });
        }

        for (index, cell) in self.cells.iter().enumerate() {
            let id = CellId::new(index as u64 + 1);
            if !(-500..=500).contains(&cell.elevation) {
                return Err(WorldValidationError::CellFieldOutOfRange {
                    cell: id,
                    field: "elevation",
                });
            }
            if cell.movement_cost < BASE_MOVEMENT_COST {
                return Err(WorldValidationError::CellFieldOutOfRange {
                    cell: id,
                    field: "movement_cost",
                });
            }
            for (field, value) in [
                ("water_access", cell.water_access),
                ("base_productivity", cell.base_productivity),
                ("season_amplitude", cell.season_amplitude),
                ("environmental_stress", cell.environmental_stress),
            ] {
                if value > PERMILLE_MAX {
                    return Err(WorldValidationError::CellFieldOutOfRange { cell: id, field });
                }
            }
            if cell.season_phase_days >= 365 {
                return Err(WorldValidationError::CellFieldOutOfRange {
                    cell: id,
                    field: "season_phase_days",
                });
            }
        }
        Ok(())
    }

    /// Stable non-cryptographic digest for exact replay/regression comparisons.
    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut digest = StableDigest::new();
        digest.write_u32(self.schema_version);
        digest.write_u32(self.width);
        digest.write_u32(self.height);
        for cell in &self.cells {
            digest.write_i16(cell.elevation);
            digest.write_u16(cell.movement_cost);
            digest.write_u16(cell.water_access);
            digest.write_u16(cell.base_productivity);
            digest.write_u32(cell.food_stock);
            digest.write_u16(cell.season_phase_days);
            digest.write_u16(cell.season_amplitude);
            digest.write_u16(cell.environmental_stress);
        }
        digest.finish()
    }

    #[must_use]
    pub fn summary(&self) -> WorldSummary {
        let count = self.cells.len() as u64;
        let mut elevation_sum = 0_i64;
        let mut water_sum = 0_u64;
        let mut productivity_sum = 0_u64;
        let mut movement_sum = 0_u64;

        for cell in &self.cells {
            elevation_sum += i64::from(cell.elevation);
            water_sum += u64::from(cell.water_access);
            productivity_sum += u64::from(cell.base_productivity);
            movement_sum += u64::from(cell.movement_cost);
        }

        WorldSummary {
            schema_version: WorldSummary::CURRENT_SCHEMA_VERSION,
            width: self.width,
            height: self.height,
            cell_count: count,
            mean_elevation: mean_signed(elevation_sum, count),
            mean_water_access: mean_unsigned(water_sum, count),
            mean_productivity: mean_unsigned(productivity_sum, count),
            mean_movement_cost: mean_unsigned(movement_sum, count),
            digest64: format!("{:016x}", self.digest64()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummary {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub cell_count: u64,
    pub mean_elevation: i16,
    pub mean_water_access: u16,
    pub mean_productivity: u16,
    pub mean_movement_cost: u16,
    pub digest64: String,
}

impl WorldSummary {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("world schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("world width and height must both be greater than zero")]
    InvalidDimensions,
    #[error("world cell count {cell_count} cannot be represented on this platform")]
    CellCountTooLarge { cell_count: u64 },
    #[error("unable to reserve memory for {cell_count} world cells")]
    AllocationFailed { cell_count: u64 },
    #[error(transparent)]
    Validation(#[from] WorldValidationError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorldValidationError {
    #[error("world cell count mismatch: expected {expected}, found {actual}")]
    CellCountMismatch { expected: u64, actual: u64 },
    #[error("cell {cell:?} has out-of-range field {field}")]
    CellFieldOutOfRange { cell: CellId, field: &'static str },
}

fn validate_config(config: WorldConfig) -> Result<(), WorldError> {
    if config.schema_version != WorldConfig::CURRENT_SCHEMA_VERSION {
        return Err(WorldError::UnsupportedSchema {
            found: config.schema_version,
            supported: WorldConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.width == 0 || config.height == 0 {
        return Err(WorldError::InvalidDimensions);
    }
    Ok(())
}

fn coherent_field(seed: u64, x: u32, y: u32) -> u16 {
    let large = u32::from(value_noise(seed ^ 0x4f1b_bcdd_19a2_41d7, x, y, 32));
    let medium = u32::from(value_noise(seed ^ 0x8c67_8f31_c8aa_52f1, x, y, 12));
    let small = u32::from(value_noise(seed ^ 0xd1b5_4a32_d192_ed03, x, y, 4));
    ((large * 5 + medium * 3 + small) / 9) as u16
}

fn value_noise(seed: u64, x: u32, y: u32, scale: u32) -> u16 {
    let x0 = x / scale;
    let y0 = y / scale;
    let tx = (x % scale) * 1_000 / scale;
    let ty = (y % scale) * 1_000 / scale;

    let north_west = hash_to_permille(seed, x0, y0);
    let north_east = hash_to_permille(seed, x0 + 1, y0);
    let south_west = hash_to_permille(seed, x0, y0 + 1);
    let south_east = hash_to_permille(seed, x0 + 1, y0 + 1);

    let north = lerp_permille(north_west, north_east, tx);
    let south = lerp_permille(south_west, south_east, tx);
    lerp_permille(north, south, ty) as u16
}

fn hash_to_permille(seed: u64, x: u32, y: u32) -> u32 {
    let mut value = seed
        ^ u64::from(x).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(y).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    (value % 1_001) as u32
}

const fn lerp_permille(a: u32, b: u32, t: u32) -> u32 {
    (a * (1_000 - t) + b * t) / 1_000
}

fn synthetic_latitude_permille(y: u32, height: u32) -> u32 {
    if height <= 1 {
        return 0;
    }
    let doubled_y = u64::from(y) * 2;
    let max = u64::from(height - 1);
    let distance_from_equator = doubled_y.abs_diff(max);
    u32::try_from(distance_from_equator * 1_000 / max).unwrap_or(1_000)
}

fn mean_signed(sum: i64, count: u64) -> i16 {
    if count == 0 {
        return 0;
    }
    i16::try_from(sum / count as i64).unwrap_or_default()
}

fn mean_unsigned(sum: u64, count: u64) -> u16 {
    if count == 0 {
        return 0;
    }
    u16::try_from(sum / count).unwrap_or(u16::MAX)
}

struct StableDigest(u64);

impl StableDigest {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }

    fn write_i16(&mut self, value: i16) {
        self.write(&value.to_le_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(seed: u64, width: u32, height: u32) -> World {
        World::generate(WorldConfig::new(width, height), RngFactory::new(seed)).unwrap()
    }

    #[test]
    fn same_seed_produces_identical_world() {
        let a = world(42, 64, 48);
        let b = world(42, 64, 48);
        assert_eq!(a, b);
        assert_eq!(a.digest64(), b.digest64());
    }

    #[test]
    fn different_seed_changes_world_digest() {
        let a = world(42, 64, 48);
        let b = world(43, 64, 48);
        assert_ne!(a.digest64(), b.digest64());
    }

    #[test]
    fn coordinates_and_ids_round_trip() {
        let world = world(7, 5, 4);
        for y in 0..world.height() {
            for x in 0..world.width() {
                let id = world.cell_id(x, y).unwrap();
                assert_eq!(world.coordinates(id), Some((x, y)));
                assert!(world.cell(id).is_some());
            }
        }
        assert_eq!(world.cell_id(5, 0), None);
        assert_eq!(world.coordinates(CellId::INVALID), None);
    }

    #[test]
    fn neighbours_are_stable_at_corner_edge_and_interior() {
        let world = world(7, 3, 3);
        let top_left = world.cell_id(0, 0).unwrap();
        assert_eq!(
            world.neighbours4(top_left),
            [None, world.cell_id(1, 0), world.cell_id(0, 1), None]
        );

        let centre = world.cell_id(1, 1).unwrap();
        assert_eq!(
            world.neighbours4(centre),
            [
                world.cell_id(1, 0),
                world.cell_id(2, 1),
                world.cell_id(1, 2),
                world.cell_id(0, 1),
            ]
        );
    }

    #[test]
    fn generated_world_passes_invariants() {
        let world = world(9_001, 128, 128);
        world.validate().unwrap();
        assert_eq!(world.cell_count(), 128 * 128);
    }

    #[test]
    fn rejects_invalid_dimensions() {
        let result = World::generate(WorldConfig::new(0, 10), RngFactory::new(1));
        assert!(matches!(result, Err(WorldError::InvalidDimensions)));
    }
}
