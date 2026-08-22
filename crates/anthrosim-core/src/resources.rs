use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{PROBABILITY_PER_MILLION, ResourceConfig},
    demography::draw_per_million,
    events::{DeathCause, EventKind, EventLog},
    ids::HouseholdId,
    population::{Population, PopulationError},
    rng::{RngFactory, RngStreamPosition},
    world::{PERMILLE_MAX, World},
};

const DAYS_PER_YEAR: u64 = 365;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    pub schema_version: u32,
    pub model_id: String,
    pub initial_world_digest64: String,
    pub periods_processed: u64,
    pub initial_food_stock: u64,
    pub regenerated_food: u64,
    pub harvested_food: u64,
    pub consumed_food: u64,
    pub unmet_need: u64,
    pub final_food_stock: u64,
    pub household_periods_with_unmet_need: u64,
    pub scarcity_deaths: u64,
    pub mean_living_condition_permille: u16,
    pub living_below_half_condition: u64,
    pub digest64: u64,
}

impl ResourceSummary {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

/// Dynamic M3 resource state.
///
/// The environmental `World` remains an immutable description of baseline
/// geography. Renewable food stock is kept here as a compact contiguous array
/// so resource accounting can change without rewriting the synthetic terrain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSystem {
    schema_version: u32,
    model_id: String,
    initial_world_digest64: String,
    cell_food_stock: Vec<u64>,
    initial_food_stock: u64,
    regenerated_food: u64,
    harvested_food: u64,
    unmet_need: u64,
    periods_processed: u64,
    household_periods_with_unmet_need: u64,
    scarcity_deaths: u64,
}

pub(crate) struct ResourcePeriodContext<'a> {
    pub world: &'a World,
    pub config: &'a ResourceConfig,
    pub period_index_in_year: u16,
    pub day: u64,
}

impl ResourceSystem {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn initialize(world: &World, config: &ResourceConfig) -> Result<Self, ResourceError> {
        validate_resource_config(config)?;

        let mut cell_food_stock = Vec::with_capacity(world.cell_count());
        let mut initial_food_stock = 0_u64;
        for cell in world.cells() {
            let scaled_initial = scale_permille(
                u64::from(cell.food_stock),
                config.productivity_scale_permille,
            );
            let capacity = cell_capacity(cell.base_productivity, config);
            let stock = scaled_initial.min(capacity);
            cell_food_stock.push(stock);
            initial_food_stock = initial_food_stock
                .checked_add(stock)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        Ok(Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: config.model_id.clone(),
            initial_world_digest64: format!("{:016x}", world.digest64()),
            cell_food_stock,
            initial_food_stock,
            regenerated_food: 0,
            harvested_food: 0,
            unmet_need: 0,
            periods_processed: 0,
            household_periods_with_unmet_need: 0,
            scarcity_deaths: 0,
        })
    }

    #[must_use]
    pub fn total_food_stock(&self) -> u64 {
        self.cell_food_stock.iter().copied().sum()
    }

    #[must_use]
    pub fn cell_food_stock(&self, cell: crate::ids::CellId) -> Option<u64> {
        let index = usize::try_from(cell.0.checked_sub(1)?).ok()?;
        self.cell_food_stock.get(index).copied()
    }

    #[must_use]
    pub fn summary(&self, population: &Population) -> ResourceSummary {
        ResourceSummary {
            schema_version: ResourceSummary::CURRENT_SCHEMA_VERSION,
            model_id: self.model_id.clone(),
            initial_world_digest64: self.initial_world_digest64.clone(),
            periods_processed: self.periods_processed,
            initial_food_stock: self.initial_food_stock,
            regenerated_food: self.regenerated_food,
            harvested_food: self.harvested_food,
            consumed_food: self.harvested_food,
            unmet_need: self.unmet_need,
            final_food_stock: self.total_food_stock(),
            household_periods_with_unmet_need: self.household_periods_with_unmet_need,
            scarcity_deaths: self.scarcity_deaths,
            mean_living_condition_permille: population.mean_living_condition_permille(),
            living_below_half_condition: population.living_below_condition(500),
            digest64: self.digest64(),
        }
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        for &byte in self.model_id.as_bytes() {
            digest_byte(&mut hash, byte);
        }
        for &stock in &self.cell_food_stock {
            digest_u64(&mut hash, stock);
        }
        digest_u64(&mut hash, self.initial_food_stock);
        digest_u64(&mut hash, self.regenerated_food);
        digest_u64(&mut hash, self.harvested_food);
        digest_u64(&mut hash, self.unmet_need);
        digest_u64(&mut hash, self.periods_processed);
        digest_u64(&mut hash, self.household_periods_with_unmet_need);
        digest_u64(&mut hash, self.scarcity_deaths);
        hash
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(crate) fn process_period(
        &mut self,
        population: &mut Population,
        world: &World,
        config: &ResourceConfig,
        period_index_in_year: u16,
        day: u64,
        scarcity_rng: &mut ChaCha8Rng,
    ) -> Result<ResourceStepOutcome, ResourceError> {
        let mut events = EventLog::new();
        self.process_period_recorded(
            population,
            &ResourcePeriodContext {
                world,
                config,
                period_index_in_year,
                day,
            },
            scarcity_rng,
            &mut events,
        )
    }

    pub(crate) fn process_period_recorded(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        events: &mut EventLog,
    ) -> Result<ResourceStepOutcome, ResourceError> {
        let ResourcePeriodContext {
            world,
            config,
            period_index_in_year,
            day,
        } = *context;
        if self.cell_food_stock.len() != world.cell_count() {
            return Err(ResourceError::StateShapeMismatch);
        }
        let periods = u64::from(config.periods_per_year);
        if u64::from(period_index_in_year) >= periods {
            return Err(ResourceError::InvalidPeriodIndex {
                index: period_index_in_year,
                periods_per_year: config.periods_per_year,
            });
        }

        let stock_before = self.total_food_stock();
        let regenerated = self.regenerate(world, config, day)?;

        let household_count = population.household_count();
        let mut living_members = vec![0_u64; household_count];
        for person_index in 0..population.person_count() {
            if !population.is_alive_index(person_index) {
                continue;
            }
            let household = population.household_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no household"),
            )?;
            let household_index = household_index(household, household_count).ok_or(
                ResourceError::InternalInvariant("person has invalid household"),
            )?;
            living_members[household_index] = living_members[household_index]
                .checked_add(1)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        let annual_need = u64::from(config.annual_need_units_per_person);
        let base_period_need = annual_need / periods;
        let remainder = annual_need % periods;
        let per_person_need =
            base_period_need + u64::from(u64::from(period_index_in_year) < remainder);

        let mut household_need = vec![0_u64; household_count];
        let mut cell_need = vec![0_u64; world.cell_count()];
        let mut total_need = 0_u64;
        for household_index_value in 0..household_count {
            let need = living_members[household_index_value]
                .checked_mul(per_person_need)
                .ok_or(ResourceError::AccountingOverflow)?;
            household_need[household_index_value] = need;
            total_need = total_need
                .checked_add(need)
                .ok_or(ResourceError::AccountingOverflow)?;
            if need == 0 {
                continue;
            }
            let household = HouseholdId::new(household_index_value as u64 + 1);
            let location = population.household_location(household).ok_or(
                ResourceError::InternalInvariant("household has no location"),
            )?;
            let cell_index = usize::try_from(
                location
                    .0
                    .checked_sub(1)
                    .ok_or(ResourceError::InternalInvariant("invalid cell ID"))?,
            )
            .map_err(|_| ResourceError::InternalInvariant("cell index does not fit usize"))?;
            let slot = cell_need
                .get_mut(cell_index)
                .ok_or(ResourceError::InternalInvariant(
                    "household location is outside world",
                ))?;
            *slot = slot
                .checked_add(need)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        let mut cell_target = vec![0_u64; world.cell_count()];
        for index in 0..world.cell_count() {
            cell_target[index] = self.cell_food_stock[index].min(cell_need[index]);
        }

        let mut household_harvest = vec![0_u64; household_count];
        let mut cell_allocated = vec![0_u64; world.cell_count()];
        for household_index_value in 0..household_count {
            let need = household_need[household_index_value];
            if need == 0 {
                continue;
            }
            let household = HouseholdId::new(household_index_value as u64 + 1);
            let location = population.household_location(household).ok_or(
                ResourceError::InternalInvariant("household has no location"),
            )?;
            let cell_index = usize::try_from(
                location
                    .0
                    .checked_sub(1)
                    .ok_or(ResourceError::InternalInvariant("invalid cell ID"))?,
            )
            .map_err(|_| ResourceError::InternalInvariant("cell index does not fit usize"))?;
            let demand = cell_need[cell_index];
            let target = cell_target[cell_index];
            let allocation = if demand == 0 {
                0
            } else {
                u64::try_from(u128::from(target) * u128::from(need) / u128::from(demand))
                    .map_err(|_| ResourceError::AccountingOverflow)?
            };
            household_harvest[household_index_value] = allocation;
            cell_allocated[cell_index] = cell_allocated[cell_index]
                .checked_add(allocation)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        // Integer proportional allocation can leave fewer than one unit per
        // competing household undistributed. Resolve that bounded remainder in
        // stable household-ID order without creating a cell->household graph.
        for household_index_value in 0..household_count {
            if household_harvest[household_index_value] >= household_need[household_index_value] {
                continue;
            }
            let household = HouseholdId::new(household_index_value as u64 + 1);
            let location = population.household_location(household).ok_or(
                ResourceError::InternalInvariant("household has no location"),
            )?;
            let cell_index = usize::try_from(
                location
                    .0
                    .checked_sub(1)
                    .ok_or(ResourceError::InternalInvariant("invalid cell ID"))?,
            )
            .map_err(|_| ResourceError::InternalInvariant("cell index does not fit usize"))?;
            if cell_allocated[cell_index] < cell_target[cell_index] {
                household_harvest[household_index_value] += 1;
                cell_allocated[cell_index] += 1;
            }
        }

        let mut harvested = 0_u64;
        for cell_index in 0..world.cell_count() {
            if cell_allocated[cell_index] != cell_target[cell_index] {
                return Err(ResourceError::InternalInvariant(
                    "proportional cell allocation did not reconcile",
                ));
            }
            self.cell_food_stock[cell_index] = self.cell_food_stock[cell_index]
                .checked_sub(cell_allocated[cell_index])
                .ok_or(ResourceError::AccountingOverflow)?;
            harvested = harvested
                .checked_add(cell_allocated[cell_index])
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        let unmet = total_need
            .checked_sub(harvested)
            .ok_or(ResourceError::AccountingOverflow)?;
        self.regenerated_food = self
            .regenerated_food
            .checked_add(regenerated)
            .ok_or(ResourceError::AccountingOverflow)?;
        self.harvested_food = self
            .harvested_food
            .checked_add(harvested)
            .ok_or(ResourceError::AccountingOverflow)?;
        self.unmet_need = self
            .unmet_need
            .checked_add(unmet)
            .ok_or(ResourceError::AccountingOverflow)?;
        self.periods_processed = self
            .periods_processed
            .checked_add(1)
            .ok_or(ResourceError::AccountingOverflow)?;

        for household_index_value in 0..household_count {
            if household_harvest[household_index_value] < household_need[household_index_value] {
                self.household_periods_with_unmet_need = self
                    .household_periods_with_unmet_need
                    .checked_add(1)
                    .ok_or(ResourceError::AccountingOverflow)?;
            }
        }

        for person_index in 0..population.person_count() {
            if !population.is_alive_index(person_index) {
                continue;
            }
            let household = population.household_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no household"),
            )?;
            let household_index_value = household_index(household, household_count).ok_or(
                ResourceError::InternalInvariant("person has invalid household"),
            )?;
            let need = household_need[household_index_value];
            let harvest = household_harvest[household_index_value];
            let supplied_permille = harvest
                .saturating_mul(u64::from(PERMILLE_MAX))
                .checked_div(need)
                .unwrap_or(u64::from(PERMILLE_MAX))
                .min(u64::from(PERMILLE_MAX));
            let current = population.condition_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no condition state"),
            )?;
            let updated = if supplied_permille >= u64::from(PERMILLE_MAX) {
                current
                    .saturating_add(config.condition_recovery_per_period)
                    .min(PERMILLE_MAX)
            } else {
                let deficit = u64::from(PERMILLE_MAX) - supplied_permille;
                let loss_numerator = deficit * u64::from(config.max_condition_loss_per_period);
                let loss = if loss_numerator == 0 {
                    0
                } else {
                    loss_numerator.div_ceil(u64::from(PERMILLE_MAX))
                };
                current.saturating_sub(u16::try_from(loss).unwrap_or(u16::MAX))
            };
            if !population.set_condition_at_index(person_index, updated) {
                return Err(ResourceError::InternalInvariant(
                    "unable to update living person's condition",
                ));
            }
        }

        let people_at_mortality_boundary = population.person_count();
        for person_index in 0..people_at_mortality_boundary {
            if !population.is_alive_index(person_index) {
                continue;
            }
            let condition = population.condition_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no condition state"),
            )?;
            let deficit = u64::from(PERMILLE_MAX - condition);
            let probability = u32::try_from(
                deficit * u64::from(config.max_scarcity_mortality_probability_per_million)
                    / u64::from(PERMILLE_MAX),
            )
            .map_err(|_| ResourceError::AccountingOverflow)?;
            if draw_per_million(scarcity_rng, probability) {
                let person = population.person_id_at_index(person_index).ok_or(
                    ResourceError::InternalInvariant("living person has no stable ID"),
                )?;
                let household = population.household_at_index(person_index).ok_or(
                    ResourceError::InternalInvariant("living person has no household"),
                )?;
                let cell = population.location_at_index(person_index).ok_or(
                    ResourceError::InternalInvariant("living person has no location"),
                )?;
                if population.mark_death(person_index, day) {
                    self.scarcity_deaths = self
                        .scarcity_deaths
                        .checked_add(1)
                        .ok_or(ResourceError::AccountingOverflow)?;
                    events.push_authoritative(
                        day,
                        EventKind::Death {
                            person,
                            household,
                            cell,
                            cause: DeathCause::ResourceScarcity,
                            condition_permille: condition,
                            probability_per_million: probability,
                        },
                    );
                }
            }
        }

        let stock_after = self.total_food_stock();
        let expected_after = stock_before
            .checked_add(regenerated)
            .and_then(|value| value.checked_sub(harvested))
            .ok_or(ResourceError::AccountingOverflow)?;
        if stock_after != expected_after {
            return Err(ResourceError::ResourceAccountingMismatch {
                expected: expected_after,
                actual: stock_after,
            });
        }
        self.validate_accounting()?;

        if population.living_count() == 0 {
            Ok(ResourceStepOutcome::PopulationExtinct)
        } else {
            Ok(ResourceStepOutcome::Continue)
        }
    }

    fn regenerate(
        &mut self,
        world: &World,
        config: &ResourceConfig,
        day: u64,
    ) -> Result<u64, ResourceError> {
        let mut regenerated = 0_u64;
        let periods = u64::from(config.periods_per_year);
        let day_of_year = u16::try_from(day % DAYS_PER_YEAR).unwrap_or(0);

        for (index, cell) in world.cells().iter().enumerate() {
            let annual_base = u64::from(cell.base_productivity)
                .checked_mul(u64::from(config.annual_regeneration_units_per_productivity))
                .ok_or(ResourceError::AccountingOverflow)?;
            let annual_base = scale_permille(annual_base, config.productivity_scale_permille);
            let seasonal = seasonal_factor_permille(
                day_of_year,
                cell.season_phase_days,
                cell.season_amplitude,
            );
            let stress_factor = PERMILLE_MAX.saturating_sub(cell.environmental_stress);
            let effective = scale_permille(scale_permille(annual_base, seasonal), stress_factor);
            let potential = effective / periods;
            let capacity = cell_capacity(cell.base_productivity, config);
            let available_space = capacity.saturating_sub(self.cell_food_stock[index]);
            let actual = potential.min(available_space);
            self.cell_food_stock[index] = self.cell_food_stock[index]
                .checked_add(actual)
                .ok_or(ResourceError::AccountingOverflow)?;
            regenerated = regenerated
                .checked_add(actual)
                .ok_or(ResourceError::AccountingOverflow)?;
        }
        Ok(regenerated)
    }

    pub(crate) fn validate_checkpoint_state(
        &self,
        world: &World,
        config: &ResourceConfig,
    ) -> Result<(), ResourceError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION
            || self.model_id != config.model_id
            || self.cell_food_stock.len() != world.cell_count()
            || self.initial_world_digest64 != format!("{:016x}", world.digest64())
        {
            return Err(ResourceError::StateShapeMismatch);
        }
        self.validate_accounting()
    }

    fn validate_accounting(&self) -> Result<(), ResourceError> {
        let expected = self
            .initial_food_stock
            .checked_add(self.regenerated_food)
            .and_then(|value| value.checked_sub(self.harvested_food))
            .ok_or(ResourceError::AccountingOverflow)?;
        let actual = self.total_food_stock();
        if expected != actual {
            return Err(ResourceError::ResourceAccountingMismatch { expected, actual });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceStepOutcome {
    Continue,
    PopulationExtinct,
}

#[derive(Debug)]
pub(crate) struct ResourceRngs {
    pub scarcity_mortality: ChaCha8Rng,
}

impl ResourceRngs {
    #[must_use]
    pub(crate) fn new(factory: RngFactory) -> Self {
        Self {
            scarcity_mortality: factory.stream("resources/scarcity_mortality"),
        }
    }

    pub(crate) fn position(&self) -> RngStreamPosition {
        RngStreamPosition::capture(&self.scarcity_mortality)
    }

    pub(crate) fn restore_position(&mut self, position: RngStreamPosition) {
        position.restore(&mut self.scarcity_mortality);
    }
}

pub fn validate_resource_config(config: &ResourceConfig) -> Result<(), ResourceConfigError> {
    if config.schema_version != ResourceConfig::CURRENT_SCHEMA_VERSION {
        return Err(ResourceConfigError::UnsupportedSchema {
            found: config.schema_version,
            supported: ResourceConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.model_id.trim().is_empty() {
        return Err(ResourceConfigError::EmptyModelId);
    }
    if config.periods_per_year == 0 || config.periods_per_year > 365 {
        return Err(ResourceConfigError::InvalidPeriodsPerYear {
            value: config.periods_per_year,
        });
    }
    if config.productivity_scale_permille > PERMILLE_MAX {
        return Err(ResourceConfigError::InvalidProductivityScale {
            value: config.productivity_scale_permille,
        });
    }
    if config.condition_recovery_per_period > PERMILLE_MAX {
        return Err(ResourceConfigError::InvalidConditionRecovery {
            value: config.condition_recovery_per_period,
        });
    }
    if config.max_condition_loss_per_period > PERMILLE_MAX {
        return Err(ResourceConfigError::InvalidConditionLoss {
            value: config.max_condition_loss_per_period,
        });
    }
    if config.max_scarcity_mortality_probability_per_million > PROBABILITY_PER_MILLION {
        return Err(ResourceConfigError::InvalidScarcityMortalityProbability {
            value: config.max_scarcity_mortality_probability_per_million,
        });
    }
    Ok(())
}

fn household_index(id: HouseholdId, household_count: usize) -> Option<usize> {
    let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
    (index < household_count).then_some(index)
}

fn cell_capacity(base_productivity: u16, config: &ResourceConfig) -> u64 {
    let annual = u64::from(base_productivity)
        .saturating_mul(u64::from(config.annual_regeneration_units_per_productivity));
    scale_permille(annual, config.productivity_scale_permille)
        .saturating_mul(u64::from(config.cell_stock_capacity_years))
}

fn scale_permille(value: u64, scale: u16) -> u64 {
    u64::try_from(u128::from(value) * u128::from(scale) / u128::from(PERMILLE_MAX))
        .unwrap_or(u64::MAX)
}

/// Integer triangular seasonal factor centred on a cell's phase.
///
/// A zero amplitude returns 1000. At amplitude 1000 the conceptual peak is
/// 2000 and the opposite point in the year reaches zero. This is deliberately
/// synthetic and exists to exercise temporal renewable-resource variation.
fn seasonal_factor_permille(day_of_year: u16, phase: u16, amplitude: u16) -> u16 {
    let direct = u16::abs_diff(day_of_year, phase);
    let wrapped = 365_u16.saturating_sub(direct);
    let distance = direct.min(wrapped).min(182);
    let wave = 1_000_i64 - (2_000_i64 * i64::from(distance) / 182_i64);
    let factor = 1_000_i64 + i64::from(amplitude) * wave / 1_000_i64;
    u16::try_from(factor.clamp(0, 2_000)).unwrap_or(0)
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        digest_byte(hash, byte);
    }
}

fn digest_byte(hash: &mut u64, byte: u8) {
    *hash ^= u64::from(byte);
    *hash = (*hash).wrapping_mul(FNV_PRIME);
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error(transparent)]
    Config(#[from] ResourceConfigError),
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error("resource accounting overflowed")]
    AccountingOverflow,
    #[error("resource state does not match world cell count")]
    StateShapeMismatch,
    #[error("resource period {index} is invalid for {periods_per_year} periods per year")]
    InvalidPeriodIndex { index: u16, periods_per_year: u16 },
    #[error("resource accounting mismatch: expected stock {expected}, found {actual}")]
    ResourceAccountingMismatch { expected: u64, actual: u64 },
    #[error("internal resource invariant failed: {0}")]
    InternalInvariant(&'static str),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResourceConfigError {
    #[error("resource schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("resource model ID must not be empty")]
    EmptyModelId,
    #[error("resource periods per year must be within 1..=365, found {value}")]
    InvalidPeriodsPerYear { value: u16 },
    #[error("resource productivity scale {value} permille is outside 0..=1000")]
    InvalidProductivityScale { value: u16 },
    #[error("condition recovery {value} permille is outside 0..=1000")]
    InvalidConditionRecovery { value: u16 },
    #[error("maximum condition loss {value} permille is outside 0..=1000")]
    InvalidConditionLoss { value: u16 },
    #[error("scarcity mortality probability {value} exceeds one million")]
    InvalidScarcityMortalityProbability { value: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorldConfig, config::PopulationConfig, rng::RngFactory, world::World};

    #[test]
    fn synthetic_resource_config_is_valid() {
        validate_resource_config(&ResourceConfig::synthetic_validation_v1()).unwrap();
    }

    #[test]
    fn resource_accounting_reconciles_after_a_period() {
        let world = World::generate(WorldConfig::new(8, 8), RngFactory::new(7)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(200), &world, RngFactory::new(7)).unwrap();
        let config = ResourceConfig::synthetic_validation_v1();
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        let before = system.total_food_stock();
        let mut rngs = ResourceRngs::new(RngFactory::new(7));

        system
            .process_period(
                &mut population,
                &world,
                &config,
                0,
                91,
                &mut rngs.scarcity_mortality,
            )
            .unwrap();

        assert_eq!(
            before + system.regenerated_food - system.harvested_food,
            system.total_food_stock()
        );
        assert_eq!(system.harvested_food + system.unmet_need, 200 * 25);
    }

    #[test]
    fn sustained_zero_resources_reduce_condition_and_survival() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(19)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(19))
                .unwrap();
        let mut config = ResourceConfig::synthetic_validation_v1()
            .with_productivity_scale_permille(0)
            .with_annual_need_units_per_person(100);
        config.periods_per_year = 1;
        config.max_condition_loss_per_period = 1_000;
        config.max_scarcity_mortality_probability_per_million = PROBABILITY_PER_MILLION;
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        let mut rngs = ResourceRngs::new(RngFactory::new(19));

        let outcome = system
            .process_period(
                &mut population,
                &world,
                &config,
                0,
                365,
                &mut rngs.scarcity_mortality,
            )
            .unwrap();

        assert_eq!(outcome, ResourceStepOutcome::PopulationExtinct);
        assert_eq!(population.living_count(), 0);
        assert_eq!(system.scarcity_deaths, 100);
        assert!(system.unmet_need > 0);
    }

    #[test]
    fn richer_environment_supports_more_survivors_than_zero_productivity() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(31)).unwrap();
        assert!(world.cells()[0].base_productivity > 0);
        let population_config = PopulationConfig::new(5);

        let mut poor_population =
            Population::initialize(population_config, &world, RngFactory::new(31)).unwrap();
        let mut rich_population =
            Population::initialize(population_config, &world, RngFactory::new(31)).unwrap();

        let mut poor = ResourceConfig::synthetic_validation_v1()
            .with_productivity_scale_permille(0)
            .with_annual_need_units_per_person(1);
        poor.periods_per_year = 1;
        poor.max_condition_loss_per_period = 1_000;
        poor.max_scarcity_mortality_probability_per_million = PROBABILITY_PER_MILLION;
        let mut rich = poor.clone();
        rich.productivity_scale_permille = 1_000;

        let mut poor_system = ResourceSystem::initialize(&world, &poor).unwrap();
        let mut rich_system = ResourceSystem::initialize(&world, &rich).unwrap();
        let mut poor_rngs = ResourceRngs::new(RngFactory::new(31));
        let mut rich_rngs = ResourceRngs::new(RngFactory::new(31));

        poor_system
            .process_period(
                &mut poor_population,
                &world,
                &poor,
                0,
                365,
                &mut poor_rngs.scarcity_mortality,
            )
            .unwrap();
        rich_system
            .process_period(
                &mut rich_population,
                &world,
                &rich,
                0,
                365,
                &mut rich_rngs.scarcity_mortality,
            )
            .unwrap();

        assert!(rich_population.living_count() > poor_population.living_count());
        assert!(
            rich_population.mean_living_condition_permille()
                >= poor_population.mean_living_condition_permille()
        );
    }

    #[test]
    fn seasonal_factor_has_expected_peak_and_trough() {
        assert_eq!(seasonal_factor_permille(0, 0, 1_000), 2_000);
        assert_eq!(seasonal_factor_permille(182, 0, 1_000), 0);
        assert_eq!(seasonal_factor_permille(100, 100, 0), 1_000);
    }
}
