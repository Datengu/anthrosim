from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:140]!r}")
    p.write_text(text.replace(old, new, 1))


# Public module boundary.
path = "crates/anthrosim-core/src/lib.rs"
replace_once(
    path,
    "pub mod temporary_mobility;\npub mod temporary_travel;\npub mod time;",
    "pub mod temporary_mobility;\npub mod temporary_resource;\npub mod temporary_travel;\npub mod time;",
)
replace_once(
    path,
    "pub use temporary_travel::{\n    TemporaryTravelModel, TemporaryTravelModelError, temporary_travel_edge_cost,\n};\npub use time::SimTime;",
    "pub use temporary_resource::{\n    TemporaryResourceAccountingError, TemporaryResourcePeriod, TemporaryResourcePresenceDays,\n};\npub use temporary_travel::{\n    TemporaryTravelModel, TemporaryTravelModelError, temporary_travel_edge_cost,\n};\npub use time::SimTime;",
)

# Temporary mobility owns the authoritative duration ledger.
path = "crates/anthrosim-core/src/temporary_mobility.rs"
replace_once(
    path,
    "    population::Population,\n    temporary_travel::TemporaryTravelModel,\n    world::World,",
    "    population::Population,\n    temporary_resource::{\n        TemporaryResourceAccountingError, TemporaryResourceLedger, TemporaryResourcePeriod,\n    },\n    temporary_travel::TemporaryTravelModel,\n    world::World,",
)
replace_once(
    path,
    "    processed_triggers: Vec<ProcessedTemporaryTrigger>,\n    next_journey_id: u64,\n}\n\nimpl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 3;",
    "    processed_triggers: Vec<ProcessedTemporaryTrigger>,\n    next_journey_id: u64,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    resource_ledger: Option<TemporaryResourceLedger>,\n}\n\nimpl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 4;",
)
replace_once(
    path,
    "            processed_triggers: Vec::new(),\n            next_journey_id: 1,\n        }",
    "            processed_triggers: Vec::new(),\n            next_journey_id: 1,\n            resource_ledger: None,\n        }",
)
replace_once(
    path,
    "        let mut state = Self::at_residence(population);\n        state.program = Some(program);\n        Ok(state)",
    "        let mut state = Self::at_residence(population);\n        state.program = Some(program);\n        state.resource_ledger = Some(TemporaryResourceLedger::new(population.household_count(), 0));\n        Ok(state)",
)
replace_once(
    path,
    "        program.validate(world)?;\n        self.program = Some(program);\n        Ok(())",
    "        program.validate(world)?;\n        self.program = Some(program);\n        self.resource_ledger = Some(TemporaryResourceLedger::new(population.household_count(), 0));\n        Ok(())",
)
replace_once(
    path,
    "            && self.processed_triggers.is_empty()\n            && self.next_journey_id == 1",
    "            && self.processed_triggers.is_empty()\n            && self.next_journey_id == 1\n            && self.resource_ledger.is_none()",
)
replace_once(
    path,
    "    /// Remove active temporary state for households with no living members.\n    pub(crate) fn reconcile_after_population_change",
    "    pub(crate) fn resource_period_snapshot(\n        &mut self,\n        day: u64,\n        world: &World,\n    ) -> Result<Option<TemporaryResourcePeriod>, TemporaryMobilityExecutionError> {\n        let Some(ledger) = self.resource_ledger.as_mut() else {\n            return Ok(None);\n        };\n        Ok(Some(ledger.snapshot_period(\n            day,\n            &self.household_presence,\n            world,\n        )?))\n    }\n\n    pub(crate) fn complete_resource_period(\n        &mut self,\n        day: u64,\n    ) -> Result<(), TemporaryMobilityExecutionError> {\n        if let Some(ledger) = self.resource_ledger.as_mut() {\n            ledger.reset_after_settlement(day)?;\n        }\n        Ok(())\n    }\n\n    /// Remove active temporary state for households with no living members.\n    pub(crate) fn reconcile_after_population_change",
)
replace_once(
    path,
    "    ) -> Result<TemporaryMobilityDayOutcome, TemporaryMobilityExecutionError> {\n        let mut outcome = TemporaryMobilityDayOutcome::default();",
    "    ) -> Result<TemporaryMobilityDayOutcome, TemporaryMobilityExecutionError> {\n        if let Some(ledger) = self.resource_ledger.as_mut() {\n            ledger.accrue_until(day, &self.household_presence)?;\n        }\n        let mut outcome = TemporaryMobilityDayOutcome::default();",
)
replace_once(
    path,
    "        if self.next_journey_id == 0 {\n            return Err(TemporaryMobilityValidationError::InvalidNextJourneyId);\n        }\n        if let Some(program) = &self.program {",
    "        if self.next_journey_id == 0 {\n            return Err(TemporaryMobilityValidationError::InvalidNextJourneyId);\n        }\n        if self.program.is_some() != self.resource_ledger.is_some() {\n            return Err(TemporaryMobilityValidationError::ResourceLedgerProgramMismatch);\n        }\n        if let Some(program) = &self.program {",
)
replace_once(
    path,
    "    ) -> Result<(), TemporaryMobilityValidationError> {\n        self.validate(population, world)?;\n        for index in 0..self.household_count() {",
    "    ) -> Result<(), TemporaryMobilityValidationError> {\n        self.validate(population, world)?;\n        if let Some(ledger) = &self.resource_ledger {\n            ledger\n                .validate(self.household_count(), world, day)\n                .map_err(|error| TemporaryMobilityValidationError::InvalidResourceLedger {\n                    reason: error.to_string(),\n                })?;\n        }\n        for index in 0..self.household_count() {",
)
replace_once(
    path,
    "        digest_u64(&mut hash, self.next_journey_id);\n        hash",
    "        digest_u64(&mut hash, self.next_journey_id);\n        match &self.resource_ledger {\n            None => digest_u64(&mut hash, 0),\n            Some(ledger) => {\n                digest_u64(&mut hash, 1);\n                ledger.digest_into(&mut hash);\n            }\n        }\n        hash",
)
replace_once(
    path,
    "    #[error(\"temporary mobility next journey ID is invalid\")]\n    InvalidNextJourneyId,",
    "    #[error(\"temporary mobility next journey ID is invalid\")]\n    InvalidNextJourneyId,\n    #[error(\"temporary mobility program and M9.5 resource ledger are not enabled together\")]\n    ResourceLedgerProgramMismatch,\n    #[error(\"temporary mobility resource ledger is invalid: {reason}\")]\n    InvalidResourceLedger { reason: String },",
)
replace_once(
    path,
    "pub enum TemporaryMobilityExecutionError {\n    #[error(transparent)]\n    InvalidState(TemporaryMobilityValidationError),",
    "pub enum TemporaryMobilityExecutionError {\n    #[error(transparent)]\n    InvalidState(TemporaryMobilityValidationError),\n    #[error(transparent)]\n    ResourceAccounting(#[from] TemporaryResourceAccountingError),",
)

# M3 accepts an optional completed presence-duration period while preserving the legacy wrapper.
path = "crates/anthrosim-core/src/resources.rs"
replace_once(
    path,
    "    rng::{RngFactory, RngStreamPosition},\n    world::{PERMILLE_MAX, World},",
    "    rng::{RngFactory, RngStreamPosition},\n    temporary_resource::{\n        TemporaryResourceAccountingError, TemporaryResourcePeriod, TemporaryResourcePresenceDays,\n    },\n    world::{PERMILLE_MAX, World},",
)
replace_once(
    path,
    "pub(crate) struct ResourcePeriodContext<'a> {\n    pub world: &'a World,\n    pub config: &'a ResourceConfig,\n    pub period_index_in_year: u16,\n    pub day: u64,\n}\n",
    "pub(crate) struct ResourcePeriodContext<'a> {\n    pub world: &'a World,\n    pub config: &'a ResourceConfig,\n    pub period_index_in_year: u16,\n    pub day: u64,\n}\n\n#[derive(Debug, Clone, Copy)]\nstruct ResourceDemandClaim {\n    household_index: usize,\n    cell_index: usize,\n    need: u64,\n}\n",
)
replace_once(
    path,
    "    pub(crate) fn process_period_recorded(\n        &mut self,\n        population: &mut Population,\n        context: &ResourcePeriodContext<'_>,\n        scarcity_rng: &mut ChaCha8Rng,\n        events: &mut EventLog,\n    ) -> Result<ResourceStepOutcome, ResourceError> {\n        let ResourcePeriodContext {",
    "    pub(crate) fn process_period_recorded(\n        &mut self,\n        population: &mut Population,\n        context: &ResourcePeriodContext<'_>,\n        scarcity_rng: &mut ChaCha8Rng,\n        events: &mut EventLog,\n    ) -> Result<ResourceStepOutcome, ResourceError> {\n        self.process_period_recorded_with_presence(\n            population,\n            context,\n            scarcity_rng,\n            events,\n            None,\n        )\n    }\n\n    pub(crate) fn process_period_recorded_with_presence(\n        &mut self,\n        population: &mut Population,\n        context: &ResourcePeriodContext<'_>,\n        scarcity_rng: &mut ChaCha8Rng,\n        events: &mut EventLog,\n        temporary_presence: Option<&TemporaryResourcePeriod>,\n    ) -> Result<ResourceStepOutcome, ResourceError> {\n        let ResourcePeriodContext {",
)
replace_once(
    path,
    "        let household_count = population.household_count();\n        let mut living_members = vec![0_u64; household_count];",
    "        let household_count = population.household_count();\n        if let Some(period) = temporary_presence {\n            validate_temporary_resource_period(\n                period,\n                household_count,\n                world,\n                config.periods_per_year,\n                period_index_in_year,\n                day,\n            )?;\n        }\n        let mut living_members = vec![0_u64; household_count];",
)
old_block = '''        let mut household_need = vec![0_u64; household_count];
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
'''
new_block = '''        let mut household_need = vec![0_u64; household_count];
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
                    let destination = presence.visitor_destination.ok_or(
                        ResourceError::InternalInvariant("visiting demand has no destination"),
                    )?;
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
                u64::try_from(
                    u128::from(target) * u128::from(claim.need) / u128::from(demand),
                )
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

        // Integer proportional allocation can leave fewer than one unit per competing claim
        // undistributed. Claims are stable by household ID, then home before visitor, so the
        // bounded remainder remains deterministic while disabled M9 retains the legacy order.
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
'''
replace_once(path, old_block, new_block)
replace_once(
    path,
    "fn household_index(id: HouseholdId, household_count: usize) -> Option<usize> {",
    '''fn validate_temporary_resource_period(
    period: &TemporaryResourcePeriod,
    household_count: usize,
    world: &World,
    periods_per_year: u16,
    period_index_in_year: u16,
    day: u64,
) -> Result<(), ResourceError> {
    period.validate(household_count, world)?;
    let periods = u64::from(periods_per_year);
    let current_offset = (u64::from(period_index_in_year) + 1)
        .saturating_mul(DAYS_PER_YEAR)
        / periods;
    let previous_offset = u64::from(period_index_in_year).saturating_mul(DAYS_PER_YEAR) / periods;
    let year_start = day.checked_sub(current_offset).ok_or(
        ResourceError::TemporaryPeriodBoundaryMismatch {
            expected_start: 0,
            expected_end: current_offset,
            actual_start: period.start_day,
            actual_end: period.end_day,
        },
    )?;
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
            // Exact fractional ties resolve to home provisioning first.
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

fn household_index(id: HouseholdId, household_count: usize) -> Option<usize> {''',
)
replace_once(
    path,
    "    #[error(transparent)]\n    Population(#[from] PopulationError),",
    "    #[error(transparent)]\n    Population(#[from] PopulationError),\n    #[error(transparent)]\n    TemporaryResource(#[from] TemporaryResourceAccountingError),",
)
replace_once(
    path,
    "    #[error(\"resource accounting mismatch: expected stock {expected}, found {actual}\")]\n    ResourceAccountingMismatch { expected: u64, actual: u64 },",
    "    #[error(\"resource accounting mismatch: expected stock {expected}, found {actual}\")]\n    ResourceAccountingMismatch { expected: u64, actual: u64 },\n    #[error(\"temporary resource period boundary mismatch: expected {expected_start}..{expected_end}, found {actual_start}..{actual_end}\")]\n    TemporaryPeriodBoundaryMismatch {\n        expected_start: u64,\n        expected_end: u64,\n        actual_start: u64,\n        actual_end: u64,\n    },",
)

# Scheduler settles duration ledger before same-day temporary transitions.
path = "crates/anthrosim-core/src/simulation.rs"
replace_once(
    path,
    "                self.time = SimTime::from_days(day);\n                let outcome = self.resources.process_period_recorded(\n                    &mut self.population,",
    "                self.time = SimTime::from_days(day);\n                let temporary_resource_period = self\n                    .temporary_mobility\n                    .resource_period_snapshot(day, &self.world)?;\n                let outcome = self.resources.process_period_recorded_with_presence(\n                    &mut self.population,",
)
replace_once(
    path,
    "                    &mut self.resource_rngs.scarcity_mortality,\n                    &mut self.events,\n                )?;\n                self.temporary_mobility\n                    .reconcile_after_population_change(&self.population);",
    "                    &mut self.resource_rngs.scarcity_mortality,\n                    &mut self.events,\n                    temporary_resource_period.as_ref(),\n                )?;\n                self.temporary_mobility.complete_resource_period(day)?;\n                self.temporary_mobility\n                    .reconcile_after_population_change(&self.population);",
)

# Authoritative meaning and checkpoint schema advance for M9.5.
replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v4";',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v5";',
)
replace_once(
    "crates/anthrosim-core/src/checkpoint.rs",
    "    pub const PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION: u32 = 7;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 8;",
    "    pub const PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION: u32 = 7;\n    pub const PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION: u32 = 8;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 9;",
)

print("M9.5 integration patch applied")
