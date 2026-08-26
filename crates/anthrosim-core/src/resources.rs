use std::sync::OnceLock;

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{PROBABILITY_PER_MILLION, ResourceConfig},
    events::{DeathCause, EventKind, EventLog},
    ids::HouseholdId,
    population::{Population, PopulationError},
    rng::{RngFactory, RngStreamPosition},
    temporary_resource::{
        TemporaryResourceAccountingError, TemporaryResourcePeriod, TemporaryResourcePresenceDays,
    },
    world::{PERMILLE_MAX, World},
};

const DAYS_PER_YEAR: u64 = 365;
const REFERENCE_RESPONSE_PERIODS_PER_YEAR: u16 = 4;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
static SEASONAL_PREFIX_BY_AMPLITUDE: OnceLock<Vec<Vec<u64>>> = OnceLock::new();

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

#[derive(Debug, Clone, Copy)]
struct ResourceDemandClaim {
    household_index: usize,
    cell_index: usize,
    need: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbabilityFraction {
    numerator: u128,
    denominator: u128,
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

    #[cfg(test)]
    pub(crate) fn process_period_recorded(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        events: &mut EventLog,
    ) -> Result<ResourceStepOutcome, ResourceError> {
        self.process_period_recorded_with_presence(population, context, scarcity_rng, events, None)
    }

    pub(crate) fn process_period_recorded_with_presence(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        events: &mut EventLog,
        temporary_presence: Option<&TemporaryResourcePeriod>,
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
        let (period_start, period_end) =
            resource_period_day_bounds(period_index_in_year, config.periods_per_year).ok_or(
                ResourceError::InvalidPeriodIndex {
                    index: period_index_in_year,
                    periods_per_year: config.periods_per_year,
                },
            )?;
        let interval_recovery = u16::try_from(reference_quarter_quantity_for_interval(
            u64::from(config.condition_recovery_per_period),
            period_start,
            period_end,
        )?)
        .map_err(|_| ResourceError::AccountingOverflow)?;
        let interval_max_loss = u16::try_from(reference_quarter_quantity_for_interval(
            u64::from(config.max_condition_loss_per_period),
            period_start,
            period_end,
        )?)
        .map_err(|_| ResourceError::AccountingOverflow)?;

        let stock_before = self.total_food_stock();
        let regenerated = self.regenerate(world, config, period_index_in_year)?;

        let household_count = population.household_count();
        if let Some(period) = temporary_presence {
            validate_temporary_resource_period(
                period,
                household_count,
                world,
                config.periods_per_year,
                period_index_in_year,
                day,
            )?;
        }
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

        let per_person_need = fixed_annual_quantity_for_period(
            u64::from(config.annual_need_units_per_person),
            period_index_in_year,
            config.periods_per_year,
        )
        .ok_or(ResourceError::InternalInvariant(
            "resource period need could not be allocated",
        ))?;

        let mut household_need = vec![0_u64; household_count];
        let mut cell_need = vec![0_u64; world.cell_count()];
        let mut total_need = 0_u64;
        let mut claims = Vec::with_capacity(household_count.saturating_mul(2));
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
            let residence = population.household_location(household).ok_or(
                ResourceError::InternalInvariant("household has no location"),
            )?;
            let residence_index = cell_index_for(world, residence)?;

            if let Some(period) = temporary_presence {
                let presence = period.households.get(household_index_value).ok_or(
                    ResourceError::InternalInvariant("temporary period household is missing"),
                )?;
                let (home_need, visiting_need) = duration_weighted_needs(need, presence)?;
                if home_need > 0 {
                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: residence_index,
                        need: home_need,
                    });
                }
                if visiting_need > 0 {
                    let destination =
                        presence
                            .visitor_destination
                            .ok_or(ResourceError::InternalInvariant(
                                "visiting demand has no destination",
                            ))?;
                    let destination_index = cell_index_for(world, destination)?;
                    if destination_index == residence_index {
                        return Err(ResourceError::InternalInvariant(
                            "temporary visitor destination equals residence",
                        ));
                    }
                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: destination_index,
                        need: visiting_need,
                    });
                }
            } else {
                claims.push(ResourceDemandClaim {
                    household_index: household_index_value,
                    cell_index: residence_index,
                    need,
                });
            }
        }

        for claim in &claims {
            cell_need[claim.cell_index] = cell_need[claim.cell_index]
                .checked_add(claim.need)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        let mut cell_target = vec![0_u64; world.cell_count()];
        for index in 0..world.cell_count() {
            cell_target[index] = self.cell_food_stock[index].min(cell_need[index]);
        }

        let mut household_harvest = vec![0_u64; household_count];
        let mut claim_harvest = vec![0_u64; claims.len()];
        let mut cell_allocated = vec![0_u64; world.cell_count()];
        for (claim_index, claim) in claims.iter().enumerate() {
            let demand = cell_need[claim.cell_index];
            let target = cell_target[claim.cell_index];
            let allocation = if demand == 0 {
                0
            } else {
                u64::try_from(u128::from(target) * u128::from(claim.need) / u128::from(demand))
                    .map_err(|_| ResourceError::AccountingOverflow)?
            };
            claim_harvest[claim_index] = allocation;
            household_harvest[claim.household_index] = household_harvest[claim.household_index]
                .checked_add(allocation)
                .ok_or(ResourceError::AccountingOverflow)?;
            cell_allocated[claim.cell_index] = cell_allocated[claim.cell_index]
                .checked_add(allocation)
                .ok_or(ResourceError::AccountingOverflow)?;
        }

        for (claim_index, claim) in claims.iter().enumerate() {
            if claim_harvest[claim_index] >= claim.need {
                continue;
            }
            if cell_allocated[claim.cell_index] < cell_target[claim.cell_index] {
                claim_harvest[claim_index] += 1;
                household_harvest[claim.household_index] = household_harvest[claim.household_index]
                    .checked_add(1)
                    .ok_or(ResourceError::AccountingOverflow)?;
                cell_allocated[claim.cell_index] += 1;
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
            let current = population.condition_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no condition state"),
            )?;
            let updated = if need == 0 {
                current
            } else {
                let supplied_permille = harvest
                    .saturating_mul(u64::from(PERMILLE_MAX))
                    .checked_div(need)
                    .ok_or(ResourceError::InternalInvariant(
                        "positive resource need produced no supply fraction",
                    ))?
                    .min(u64::from(PERMILLE_MAX));
                if supplied_permille >= u64::from(PERMILLE_MAX) {
                    current.saturating_add(interval_recovery).min(PERMILLE_MAX)
                } else {
                    let deficit = u64::from(PERMILLE_MAX) - supplied_permille;
                    let loss_numerator = deficit * u64::from(interval_max_loss);
                    let loss = if loss_numerator == 0 {
                        0
                    } else {
                        loss_numerator.div_ceil(u64::from(PERMILLE_MAX))
                    };
                    current.saturating_sub(u16::try_from(loss).unwrap_or(u16::MAX))
                }
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
            let reference_probability = u32::try_from(
                deficit * u64::from(config.max_scarcity_mortality_probability_per_million)
                    / u64::from(PERMILLE_MAX),
            )
            .map_err(|_| ResourceError::AccountingOverflow)?;
            let interval_probability = reference_quarter_probability_for_interval(
                reference_probability,
                period_start,
                period_end,
            )?;
            if draw_probability_fraction(scarcity_rng, interval_probability) {
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
                            probability_per_million: probability_fraction_per_million_ceil(
                                interval_probability,
                            )?,
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
        period_index_in_year: u16,
    ) -> Result<u64, ResourceError> {
        let mut regenerated = 0_u64;

        for (index, cell) in world.cells().iter().enumerate() {
            let annual_base = u64::from(cell.base_productivity)
                .checked_mul(u64::from(config.annual_regeneration_units_per_productivity))
                .ok_or(ResourceError::AccountingOverflow)?;
            let annual_base = scale_permille(annual_base, config.productivity_scale_permille);
            let stress_factor = PERMILLE_MAX.saturating_sub(cell.environmental_stress);
            let annual_effective = scale_permille(annual_base, stress_factor);
            let scaled_amplitude = u16::try_from(scale_permille(
                u64::from(cell.season_amplitude),
                config.seasonality_scale_permille,
            ))
            .unwrap_or(PERMILLE_MAX);
            let potential = seasonal_annual_quantity_for_period(
                annual_effective,
                period_index_in_year,
                config.periods_per_year,
                cell.season_phase_days,
                scaled_amplitude,
            )
            .ok_or(ResourceError::InternalInvariant(
                "seasonal resource period could not be allocated",
            ))?;
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
    if config.seasonality_scale_permille > PERMILLE_MAX {
        return Err(ResourceConfigError::InvalidSeasonalityScale {
            value: config.seasonality_scale_permille,
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

/// Exact half-open day offsets for one resource period within a 365-day model year.
pub(crate) fn resource_period_day_bounds(
    period_index_in_year: u16,
    periods_per_year: u16,
) -> Option<(u64, u64)> {
    if periods_per_year == 0 || period_index_in_year >= periods_per_year {
        return None;
    }
    let periods = u64::from(periods_per_year);
    let index = u64::from(period_index_in_year);
    let start = index.checked_mul(DAYS_PER_YEAR)? / periods;
    let end = index.checked_add(1)?.checked_mul(DAYS_PER_YEAR)? / periods;
    Some((start, end))
}

/// Allocate a fixed annual integer quantity over the scheduler's actual elapsed-day periods.
pub(crate) fn fixed_annual_quantity_for_period(
    annual: u64,
    period_index_in_year: u16,
    periods_per_year: u16,
) -> Option<u64> {
    let (start, end) = resource_period_day_bounds(period_index_in_year, periods_per_year)?;
    let before =
        u64::try_from(u128::from(annual) * u128::from(start) / u128::from(DAYS_PER_YEAR)).ok()?;
    let after =
        u64::try_from(u128::from(annual) * u128::from(end) / u128::from(DAYS_PER_YEAR)).ok()?;
    after.checked_sub(before)
}

/// Resolve the fixed annual share corresponding to an actual resource boundary day.
pub(crate) fn fixed_annual_quantity_at_resource_boundary(
    annual: u64,
    periods_per_year: u16,
    day: u64,
) -> Option<u64> {
    if periods_per_year == 0 {
        return None;
    }
    let offset = match day % DAYS_PER_YEAR {
        0 => DAYS_PER_YEAR,
        value => value,
    };
    let periods = u64::from(periods_per_year);
    let ordinal = offset
        .checked_mul(periods)?
        .checked_add(DAYS_PER_YEAR - 1)?
        / DAYS_PER_YEAR;
    let index = ordinal.checked_sub(1)?;
    let index = u16::try_from(index).ok()?;
    let (_, expected_end) = resource_period_day_bounds(index, periods_per_year)?;
    if expected_end != offset {
        return None;
    }
    fixed_annual_quantity_for_period(annual, index, periods_per_year)
}

/// Convert one reference-quarter response quantity into the amount attributable to any half-open
/// interval in the model year. The four reference intervals are exactly the canonical scheduler
/// quarters [0,91), [91,182), [182,273), [273,365). Linear cumulative allocation inside each
/// reference quarter preserves the configured amount at every quarter boundary and conserves four
/// times the reference quantity over a complete year, regardless of M3 partitioning.
fn reference_quarter_quantity_for_interval(
    reference_quantity: u64,
    start: u64,
    end: u64,
) -> Result<u64, ResourceError> {
    if start > end || end > DAYS_PER_YEAR {
        return Err(ResourceError::InternalInvariant(
            "response interval is outside the model year",
        ));
    }
    let mut total = 0_u64;
    for quarter in 0..REFERENCE_RESPONSE_PERIODS_PER_YEAR {
        let (quarter_start, quarter_end) =
            resource_period_day_bounds(quarter, REFERENCE_RESPONSE_PERIODS_PER_YEAR).ok_or(
                ResourceError::InternalInvariant("reference response quarter is invalid"),
            )?;
        let overlap_start = start.max(quarter_start);
        let overlap_end = end.min(quarter_end);
        if overlap_start >= overlap_end {
            continue;
        }
        let quarter_length = quarter_end - quarter_start;
        let local_start = overlap_start - quarter_start;
        let local_end = overlap_end - quarter_start;
        let before = u64::try_from(
            u128::from(reference_quantity) * u128::from(local_start) / u128::from(quarter_length),
        )
        .map_err(|_| ResourceError::AccountingOverflow)?;
        let after = u64::try_from(
            u128::from(reference_quantity) * u128::from(local_end) / u128::from(quarter_length),
        )
        .map_err(|_| ResourceError::AccountingOverflow)?;
        total = total
            .checked_add(
                after
                    .checked_sub(before)
                    .ok_or(ResourceError::AccountingOverflow)?,
            )
            .ok_or(ResourceError::AccountingOverflow)?;
    }
    Ok(total)
}

/// Exact conditional death probability for an arbitrary interval when `reference_probability` is
/// the conditional probability over each canonical reference quarter at fixed condition.
///
/// Within a reference quarter, cumulative incidence is linear in elapsed days. Conditional
/// survival over a sub-interval is therefore an exact rational ratio. Multiplying at most four
/// overlapping quarter ratios makes the complete-year survival `(1-q)^4` independent of how M3
/// partitions the year, while P=4 reproduces q exactly at every reference-quarter boundary.
fn reference_quarter_probability_for_interval(
    reference_probability: u32,
    start: u64,
    end: u64,
) -> Result<ProbabilityFraction, ResourceError> {
    if reference_probability > PROBABILITY_PER_MILLION || start > end || end > DAYS_PER_YEAR {
        return Err(ResourceError::InternalInvariant(
            "scarcity probability interval is invalid",
        ));
    }
    if start == end || reference_probability == 0 {
        return Ok(ProbabilityFraction {
            numerator: 0,
            denominator: 1,
        });
    }

    let probability = u128::from(reference_probability);
    let scale = u128::from(PROBABILITY_PER_MILLION);
    let mut survival_numerator = 1_u128;
    let mut survival_denominator = 1_u128;

    for quarter in 0..REFERENCE_RESPONSE_PERIODS_PER_YEAR {
        let (quarter_start, quarter_end) =
            resource_period_day_bounds(quarter, REFERENCE_RESPONSE_PERIODS_PER_YEAR).ok_or(
                ResourceError::InternalInvariant("reference mortality quarter is invalid"),
            )?;
        let overlap_start = start.max(quarter_start);
        let overlap_end = end.min(quarter_end);
        if overlap_start >= overlap_end {
            continue;
        }
        let quarter_length = quarter_end - quarter_start;
        let local_start = overlap_start - quarter_start;
        let local_end = overlap_end - quarter_start;
        let base = scale
            .checked_mul(u128::from(quarter_length))
            .ok_or(ResourceError::AccountingOverflow)?;
        let segment_numerator = base
            .checked_sub(
                probability
                    .checked_mul(u128::from(local_end))
                    .ok_or(ResourceError::AccountingOverflow)?,
            )
            .ok_or(ResourceError::AccountingOverflow)?;
        let segment_denominator = base
            .checked_sub(
                probability
                    .checked_mul(u128::from(local_start))
                    .ok_or(ResourceError::AccountingOverflow)?,
            )
            .ok_or(ResourceError::AccountingOverflow)?;
        if segment_denominator == 0 {
            return Err(ResourceError::InternalInvariant(
                "scarcity survival denominator is zero",
            ));
        }
        survival_numerator = survival_numerator
            .checked_mul(segment_numerator)
            .ok_or(ResourceError::AccountingOverflow)?;
        survival_denominator = survival_denominator
            .checked_mul(segment_denominator)
            .ok_or(ResourceError::AccountingOverflow)?;
    }

    Ok(ProbabilityFraction {
        numerator: survival_denominator
            .checked_sub(survival_numerator)
            .ok_or(ResourceError::AccountingOverflow)?,
        denominator: survival_denominator,
    })
}

fn draw_probability_fraction(rng: &mut ChaCha8Rng, probability: ProbabilityFraction) -> bool {
    if probability.numerator == 0 {
        return false;
    }
    if probability.numerator >= probability.denominator {
        return true;
    }
    draw_bounded_u128(rng, probability.denominator) < probability.numerator
}

fn draw_bounded_u128<R: Rng + ?Sized>(rng: &mut R, upper_exclusive: u128) -> u128 {
    debug_assert!(upper_exclusive > 0);
    let acceptance_limit = u128::MAX - (u128::MAX % upper_exclusive);
    loop {
        let draw = (u128::from(rng.next_u64()) << 64) | u128::from(rng.next_u64());
        if draw < acceptance_limit {
            return draw % upper_exclusive;
        }
    }
}

fn probability_fraction_per_million_ceil(
    probability: ProbabilityFraction,
) -> Result<u32, ResourceError> {
    if probability.numerator == 0 {
        return Ok(0);
    }
    let scaled = probability
        .numerator
        .checked_mul(u128::from(PROBABILITY_PER_MILLION))
        .ok_or(ResourceError::AccountingOverflow)?
        .div_ceil(probability.denominator)
        .min(u128::from(PROBABILITY_PER_MILLION));
    u32::try_from(scaled).map_err(|_| ResourceError::AccountingOverflow)
}

fn seasonal_prefix_table() -> &'static [Vec<u64>] {
    SEASONAL_PREFIX_BY_AMPLITUDE
        .get_or_init(|| {
            (0_u16..=PERMILLE_MAX)
                .map(|amplitude| {
                    let mut prefix = Vec::with_capacity(DAYS_PER_YEAR as usize + 1);
                    prefix.push(0_u64);
                    for day in 0..DAYS_PER_YEAR {
                        let factor = seasonal_factor_permille(
                            u16::try_from(day).expect("model-year day fits u16"),
                            0,
                            amplitude,
                        );
                        let next = prefix
                            .last()
                            .copied()
                            .unwrap_or(0)
                            .saturating_add(u64::from(factor));
                        prefix.push(next);
                    }
                    prefix
                })
                .collect()
        })
        .as_slice()
}

fn seasonal_cumulative_weight(offset: u64, phase: u16, amplitude: u16) -> Option<u64> {
    if offset > DAYS_PER_YEAR || amplitude > PERMILLE_MAX {
        return None;
    }
    let table = seasonal_prefix_table();
    let prefix = table.get(usize::from(amplitude))?;
    let phase = u64::from(phase) % DAYS_PER_YEAR;
    let start = (DAYS_PER_YEAR - phase) % DAYS_PER_YEAR;
    let end = start.checked_add(offset)?;
    let year_total = *prefix.get(DAYS_PER_YEAR as usize)?;
    if end <= DAYS_PER_YEAR {
        prefix
            .get(end as usize)?
            .checked_sub(*prefix.get(start as usize)?)
    } else {
        let first = year_total.checked_sub(*prefix.get(start as usize)?)?;
        first.checked_add(*prefix.get((end - DAYS_PER_YEAR) as usize)?)
    }
}

fn seasonal_annual_quantity_for_period(
    annual: u64,
    period_index_in_year: u16,
    periods_per_year: u16,
    phase: u16,
    amplitude: u16,
) -> Option<u64> {
    let (start, end) = resource_period_day_bounds(period_index_in_year, periods_per_year)?;
    let denominator = seasonal_cumulative_weight(DAYS_PER_YEAR, phase, amplitude)?;
    if denominator == 0 {
        return None;
    }
    let before_weight = seasonal_cumulative_weight(start, phase, amplitude)?;
    let after_weight = seasonal_cumulative_weight(end, phase, amplitude)?;
    let before =
        u64::try_from(u128::from(annual) * u128::from(before_weight) / u128::from(denominator))
            .ok()?;
    let after =
        u64::try_from(u128::from(annual) * u128::from(after_weight) / u128::from(denominator))
            .ok()?;
    after.checked_sub(before)
}

fn validate_temporary_resource_period(
    period: &TemporaryResourcePeriod,
    household_count: usize,
    world: &World,
    periods_per_year: u16,
    period_index_in_year: u16,
    day: u64,
) -> Result<(), ResourceError> {
    period.validate(household_count, world)?;
    let (previous_offset, current_offset) =
        resource_period_day_bounds(period_index_in_year, periods_per_year).ok_or(
            ResourceError::InvalidPeriodIndex {
                index: period_index_in_year,
                periods_per_year,
            },
        )?;
    let year_start =
        day.checked_sub(current_offset)
            .ok_or(ResourceError::TemporaryPeriodBoundaryMismatch {
                expected_start: 0,
                expected_end: current_offset,
                actual_start: period.start_day,
                actual_end: period.end_day,
            })?;
    let expected_start = year_start
        .checked_add(previous_offset)
        .ok_or(ResourceError::AccountingOverflow)?;
    if period.start_day != expected_start || period.end_day != day {
        return Err(ResourceError::TemporaryPeriodBoundaryMismatch {
            expected_start,
            expected_end: day,
            actual_start: period.start_day,
            actual_end: period.end_day,
        });
    }
    Ok(())
}

pub(crate) fn duration_weighted_needs(
    need: u64,
    presence: &TemporaryResourcePresenceDays,
) -> Result<(u64, u64), ResourceError> {
    let duration = presence.total_days()?;
    if duration == 0 {
        return Err(ResourceError::InternalInvariant(
            "temporary resource duration is zero",
        ));
    }
    let home_days = presence.home_provisioning_days()?;
    let visiting_days = presence.visiting_days;
    if home_days
        .checked_add(visiting_days)
        .ok_or(ResourceError::AccountingOverflow)?
        != duration
    {
        return Err(ResourceError::InternalInvariant(
            "temporary resource presence days do not reconcile",
        ));
    }

    let denominator = u128::from(duration);
    let home_numerator = u128::from(need)
        .checked_mul(u128::from(home_days))
        .ok_or(ResourceError::AccountingOverflow)?;
    let visiting_numerator = u128::from(need)
        .checked_mul(u128::from(visiting_days))
        .ok_or(ResourceError::AccountingOverflow)?;
    let mut home_need = u64::try_from(home_numerator / denominator)
        .map_err(|_| ResourceError::AccountingOverflow)?;
    let mut visiting_need = u64::try_from(visiting_numerator / denominator)
        .map_err(|_| ResourceError::AccountingOverflow)?;
    let assigned = home_need
        .checked_add(visiting_need)
        .ok_or(ResourceError::AccountingOverflow)?;
    let remainder = need
        .checked_sub(assigned)
        .ok_or(ResourceError::AccountingOverflow)?;
    if remainder > 1 {
        return Err(ResourceError::InternalInvariant(
            "duration-weighted need left more than one remainder unit",
        ));
    }
    if remainder == 1 {
        let home_fraction = home_numerator % denominator;
        let visiting_fraction = visiting_numerator % denominator;
        if visiting_fraction > home_fraction {
            visiting_need = visiting_need
                .checked_add(1)
                .ok_or(ResourceError::AccountingOverflow)?;
        } else {
            home_need = home_need
                .checked_add(1)
                .ok_or(ResourceError::AccountingOverflow)?;
        }
    }
    if home_need
        .checked_add(visiting_need)
        .ok_or(ResourceError::AccountingOverflow)?
        != need
    {
        return Err(ResourceError::InternalInvariant(
            "duration-weighted household need did not conserve exactly",
        ));
    }
    Ok((home_need, visiting_need))
}

fn cell_index_for(world: &World, cell: crate::ids::CellId) -> Result<usize, ResourceError> {
    let index = usize::try_from(
        cell.0
            .checked_sub(1)
            .ok_or(ResourceError::InternalInvariant("invalid cell ID"))?,
    )
    .map_err(|_| ResourceError::InternalInvariant("cell index does not fit usize"))?;
    if index >= world.cell_count() {
        return Err(ResourceError::InternalInvariant("cell is outside world"));
    }
    Ok(index)
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

#[cfg(test)]
fn scaled_seasonal_factor_permille(
    day_of_year: u16,
    phase: u16,
    cell_amplitude: u16,
    seasonality_scale_permille: u16,
) -> u16 {
    let scaled_amplitude = scale_permille(u64::from(cell_amplitude), seasonality_scale_permille);
    seasonal_factor_permille(
        day_of_year,
        phase,
        u16::try_from(scaled_amplitude).unwrap_or(PERMILLE_MAX),
    )
}

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
    #[error(transparent)]
    TemporaryResource(#[from] TemporaryResourceAccountingError),
    #[error("resource accounting overflowed")]
    AccountingOverflow,
    #[error("resource state does not match world cell count")]
    StateShapeMismatch,
    #[error("resource period {index} is invalid for {periods_per_year} periods per year")]
    InvalidPeriodIndex { index: u16, periods_per_year: u16 },
    #[error("resource accounting mismatch: expected stock {expected}, found {actual}")]
    ResourceAccountingMismatch { expected: u64, actual: u64 },
    #[error(
        "temporary resource period boundary mismatch: expected {expected_start}..{expected_end}, found {actual_start}..{actual_end}"
    )]
    TemporaryPeriodBoundaryMismatch {
        expected_start: u64,
        expected_end: u64,
        actual_start: u64,
        actual_end: u64,
    },
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
    #[error("resource seasonality scale {value} permille is outside 0..=1000")]
    InvalidSeasonalityScale { value: u16 },
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
    fn elapsed_day_fixed_allocation_conserves_annual_quantities() {
        assert_eq!(resource_period_day_bounds(0, 4), Some((0, 91)));
        assert_eq!(resource_period_day_bounds(1, 4), Some((91, 182)));
        assert_eq!(resource_period_day_bounds(2, 4), Some((182, 273)));
        assert_eq!(resource_period_day_bounds(3, 4), Some((273, 365)));

        let one = (0..4)
            .map(|index| fixed_annual_quantity_for_period(1, index, 4).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(one, vec![0, 0, 0, 1]);
        assert_eq!(one.iter().sum::<u64>(), 1);

        let hundred = (0..4)
            .map(|index| fixed_annual_quantity_for_period(100, index, 4).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(hundred, vec![24, 25, 25, 26]);
        assert_eq!(hundred.iter().sum::<u64>(), 100);

        for periods in [1_u16, 3, 4, 5, 12, 365] {
            for annual in [0_u64, 1, 4, 100, 365, 1_001] {
                let total = (0..periods)
                    .map(|index| fixed_annual_quantity_for_period(annual, index, periods).unwrap())
                    .sum::<u64>();
                assert_eq!(total, annual, "periods={periods}, annual={annual}");
            }
        }
    }

    #[test]
    fn reference_quarter_condition_budget_is_partition_invariant() {
        for periods in [1_u16, 4, 12, 365] {
            let total = (0..periods)
                .map(|index| {
                    let (start, end) = resource_period_day_bounds(index, periods).unwrap();
                    reference_quarter_quantity_for_interval(25, start, end).unwrap()
                })
                .sum::<u64>();
            assert_eq!(total, 100, "periods={periods}");
        }
        let quarterly = (0..4)
            .map(|index| {
                let (start, end) = resource_period_day_bounds(index, 4).unwrap();
                reference_quarter_quantity_for_interval(25, start, end).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(quarterly, vec![25, 25, 25, 25]);
    }

    fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let remainder = a % b;
            a = b;
            b = remainder;
        }
        a
    }

    fn composed_survival_for_partition(reference_probability: u32, periods: u16) -> (u128, u128) {
        let mut numerator = 1_u128;
        let mut denominator = 1_u128;
        for index in 0..periods {
            let (start, end) = resource_period_day_bounds(index, periods).unwrap();
            let death =
                reference_quarter_probability_for_interval(reference_probability, start, end)
                    .unwrap();
            let mut segment_numerator = death.denominator - death.numerator;
            let mut segment_denominator = death.denominator;
            let cross_a = gcd_u128(segment_numerator, denominator);
            segment_numerator /= cross_a;
            denominator /= cross_a;
            let cross_b = gcd_u128(segment_denominator, numerator);
            segment_denominator /= cross_b;
            numerator /= cross_b;
            numerator *= segment_numerator;
            denominator *= segment_denominator;
            let common = gcd_u128(numerator, denominator);
            numerator /= common;
            denominator /= common;
        }
        (numerator, denominator)
    }

    #[test]
    fn fixed_condition_scarcity_survival_is_partition_invariant() {
        for reference_probability in [0_u32, 200_000, 500_000, 1_000_000] {
            let baseline = composed_survival_for_partition(reference_probability, 4);
            for periods in [1_u16, 4, 12, 365] {
                assert_eq!(
                    composed_survival_for_partition(reference_probability, periods),
                    baseline,
                    "q={reference_probability}, periods={periods}"
                );
            }
        }
        let q = 200_000_u32;
        for quarter in 0..4_u16 {
            let (start, end) = resource_period_day_bounds(quarter, 4).unwrap();
            let probability = reference_quarter_probability_for_interval(q, start, end).unwrap();
            assert_eq!(
                probability_fraction_per_million_ceil(probability).unwrap(),
                q
            );
        }
    }

    #[test]
    fn boundary_lookup_returns_the_same_fixed_share_as_period_index() {
        for periods in [1_u16, 3, 4, 5, 12, 365] {
            for index in 0..periods {
                let (_, end) = resource_period_day_bounds(index, periods).unwrap();
                let indexed = fixed_annual_quantity_for_period(101, index, periods).unwrap();
                let by_boundary =
                    fixed_annual_quantity_at_resource_boundary(101, periods, end).unwrap();
                assert_eq!(indexed, by_boundary, "periods={periods}, index={index}");
            }
        }
        assert!(fixed_annual_quantity_at_resource_boundary(100, 4, 90).is_none());
    }

    #[test]
    fn integrated_seasonality_preserves_annual_total_across_phase_and_resolution() {
        for periods in [1_u16, 3, 4, 5, 12, 365] {
            let mut phase_zero = 0_u64;
            let mut phase_opposite = 0_u64;
            for index in 0..periods {
                phase_zero +=
                    seasonal_annual_quantity_for_period(10_000, index, periods, 0, 1_000).unwrap();
                phase_opposite +=
                    seasonal_annual_quantity_for_period(10_000, index, periods, 182, 1_000)
                        .unwrap();
            }
            assert_eq!(phase_zero, 10_000, "phase 0, periods={periods}");
            assert_eq!(phase_opposite, 10_000, "phase 182, periods={periods}");
        }

        let phase_zero_first = seasonal_annual_quantity_for_period(10_000, 0, 4, 0, 1_000).unwrap();
        let phase_opposite_first =
            seasonal_annual_quantity_for_period(10_000, 0, 4, 182, 1_000).unwrap();
        assert_ne!(phase_zero_first, phase_opposite_first);
    }

    #[test]
    fn zero_seasonality_reduces_exactly_to_fixed_elapsed_day_allocation() {
        for periods in [1_u16, 3, 4, 5, 12, 365] {
            for index in 0..periods {
                assert_eq!(
                    seasonal_annual_quantity_for_period(1_001, index, periods, 173, 0),
                    fixed_annual_quantity_for_period(1_001, index, periods)
                );
            }
        }
    }

    fn one_person_condition_run(periods: u16, fully_supplied: bool) -> u16 {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(211)).unwrap();
        let mut population = Population::initialize(
            PopulationConfig::new(1).with_target_household_size(1),
            &world,
            RngFactory::new(211),
        )
        .unwrap();
        assert!(population.set_condition_at_index(0, if fully_supplied { 500 } else { 1_000 }));
        let mut config = ResourceConfig::synthetic_validation_v1();
        config.periods_per_year = periods;
        config.annual_need_units_per_person = 365;
        config.annual_regeneration_units_per_productivity = 0;
        config.condition_recovery_per_period = 25;
        config.max_condition_loss_per_period = 100;
        config.max_scarcity_mortality_probability_per_million = 0;
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        system
            .cell_food_stock
            .fill(if fully_supplied { 1_000 } else { 0 });
        system.initial_food_stock = if fully_supplied { 1_000 } else { 0 };
        let mut rngs = ResourceRngs::new(RngFactory::new(211));
        for index in 0..periods {
            let (_, day) = resource_period_day_bounds(index, periods).unwrap();
            system
                .process_period(
                    &mut population,
                    &world,
                    &config,
                    index,
                    day,
                    &mut rngs.scarcity_mortality,
                )
                .unwrap();
        }
        population.condition_at_index(0).unwrap()
    }

    #[test]
    fn condition_response_does_not_multiply_with_resource_partition() {
        for periods in [1_u16, 4, 12, 365] {
            assert_eq!(
                one_person_condition_run(periods, true),
                600,
                "recovery P={periods}"
            );
            assert_eq!(
                one_person_condition_run(periods, false),
                600,
                "loss P={periods}"
            );
        }
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
        assert_eq!(system.harvested_food + system.unmet_need, 200 * 24);
    }

    #[test]
    fn zero_need_period_is_condition_neutral() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(13)).unwrap();
        let mut population = Population::initialize(
            PopulationConfig::new(1).with_target_household_size(1),
            &world,
            RngFactory::new(13),
        )
        .unwrap();
        assert!(population.set_condition_at_index(0, 400));

        let mut config = ResourceConfig::synthetic_validation_v1();
        config.periods_per_year = 4;
        config.annual_need_units_per_person = 1;
        config.annual_regeneration_units_per_productivity = 0;
        config.condition_recovery_per_period = 250;
        config.max_scarcity_mortality_probability_per_million = 0;
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        let mut rngs = ResourceRngs::new(RngFactory::new(13));

        for index in 0..3_u16 {
            let (_, day) = resource_period_day_bounds(index, 4).unwrap();
            system
                .process_period(
                    &mut population,
                    &world,
                    &config,
                    index,
                    day,
                    &mut rngs.scarcity_mortality,
                )
                .unwrap();
            assert_eq!(population.condition_at_index(0), Some(400));
        }

        assert_eq!(system.harvested_food, 0);
        assert_eq!(system.unmet_need, 0);
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
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 0), 1_000);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 500), 1_400);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 1_000), 1_800);

        let invalid =
            ResourceConfig::synthetic_validation_v1().with_seasonality_scale_permille(1_001);
        assert!(matches!(
            validate_resource_config(&invalid),
            Err(ResourceConfigError::InvalidSeasonalityScale { .. })
        ));
    }

    #[test]
    fn duration_weighting_conserves_one_and_five_day_visits_and_treats_transit_as_home() {
        let one_day = TemporaryResourcePresenceDays {
            at_residence_days: 88,
            outbound_transit_days: 1,
            visiting_days: 1,
            return_transit_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
        };
        assert_eq!(one_day.total_days().unwrap(), 91);
        assert_eq!(duration_weighted_needs(91, &one_day).unwrap(), (90, 1));

        let five_days = TemporaryResourcePresenceDays {
            at_residence_days: 84,
            outbound_transit_days: 1,
            visiting_days: 5,
            return_transit_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
        };
        assert_eq!(five_days.home_provisioning_days().unwrap(), 86);
        assert_eq!(five_days.total_days().unwrap(), 91);
        assert_eq!(duration_weighted_needs(91, &five_days).unwrap(), (86, 5));

        let tie = TemporaryResourcePresenceDays {
            at_residence_days: 1,
            visiting_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
            ..TemporaryResourcePresenceDays::default()
        };
        assert_eq!(duration_weighted_needs(1, &tie).unwrap(), (1, 0));
    }

    #[test]
    fn mixed_cell_supply_reconciles_to_one_household_satisfaction_without_losing_need() {
        let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(119)).unwrap();
        let mut population = Population::initialize(
            PopulationConfig::new(1).with_target_household_size(1),
            &world,
            RngFactory::new(119),
        )
        .unwrap();
        let household = HouseholdId::new(1);
        let residence = population.household_location(household).unwrap();
        let destination = if residence == crate::ids::CellId::new(1) {
            crate::ids::CellId::new(2)
        } else {
            crate::ids::CellId::new(1)
        };
        let residence_index = cell_index_for(&world, residence).unwrap();
        let destination_index = cell_index_for(&world, destination).unwrap();

        let mut config = ResourceConfig::synthetic_validation_v1();
        config.periods_per_year = 1;
        config.annual_need_units_per_person = 100;
        config.annual_regeneration_units_per_productivity = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        system.cell_food_stock.fill(0);
        system.cell_food_stock[destination_index] = 100;
        system.initial_food_stock = 100;

        let presence = TemporaryResourcePresenceDays {
            at_residence_days: 183,
            visiting_days: 182,
            visitor_destination: Some(destination),
            ..TemporaryResourcePresenceDays::default()
        };
        assert_eq!(duration_weighted_needs(100, &presence).unwrap(), (50, 50));
        let period = TemporaryResourcePeriod {
            schema_version: TemporaryResourcePeriod::CURRENT_SCHEMA_VERSION,
            start_day: 0,
            end_day: 365,
            households: vec![presence],
        };
        let before_condition = population.condition_at_index(0).unwrap();
        let mut rngs = ResourceRngs::new(RngFactory::new(119));
        let mut events = EventLog::new();

        system
            .process_period_recorded_with_presence(
                &mut population,
                &ResourcePeriodContext {
                    world: &world,
                    config: &config,
                    period_index_in_year: 0,
                    day: 365,
                },
                &mut rngs.scarcity_mortality,
                &mut events,
                Some(&period),
            )
            .unwrap();

        assert_eq!(system.cell_food_stock[residence_index], 0);
        assert_eq!(system.cell_food_stock[destination_index], 50);
        assert_eq!(system.harvested_food, 50);
        assert_eq!(system.unmet_need, 50);
        assert_eq!(system.harvested_food + system.unmet_need, 100);
        assert!(population.condition_at_index(0).unwrap() < before_condition);
        system.validate_accounting().unwrap();
    }
}
