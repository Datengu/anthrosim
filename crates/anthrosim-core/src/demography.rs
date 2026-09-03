use std::collections::BTreeMap;

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::{
    config::{AgeProbabilityBand, DemographyConfig, PROBABILITY_PER_MILLION},
    events::{DeathCause, EventKind, EventLog},
    founder_initialization::FounderPopulationDefinition,
    ids::{CellId, HouseholdId, PersonId},
    population::{Population, PopulationError, ReproductiveSex},
    rng::{RngFactory, RngStreamPosition},
    time::{DAYS_PER_YEAR, SimTime},
    world::{PERMILLE_MAX, World},
};

pub fn validate_demography_config(config: &DemographyConfig) -> Result<(), DemographyConfigError> {
    if config.schema_version != DemographyConfig::CURRENT_SCHEMA_VERSION {
        return Err(DemographyConfigError::UnsupportedSchema {
            found: config.schema_version,
            supported: DemographyConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.schedule_id.trim().is_empty() {
        return Err(DemographyConfigError::EmptyScheduleId);
    }
    validate_complete_schedule("mortality", &config.mortality_bands)?;
    validate_complete_schedule("fertility", &config.fertility_bands)?;
    if config.male_birth_permille > PERMILLE_MAX {
        return Err(DemographyConfigError::InvalidMaleBirthPermille {
            value: config.male_birth_permille,
        });
    }
    if config.male_parent_min_age_years >= config.male_parent_max_age_years_exclusive {
        return Err(DemographyConfigError::InvalidMaleParentAgeRange {
            minimum: config.male_parent_min_age_years,
            maximum_exclusive: config.male_parent_max_age_years_exclusive,
        });
    }
    Ok(())
}

fn validate_complete_schedule(
    schedule: &'static str,
    bands: &[AgeProbabilityBand],
) -> Result<(), DemographyConfigError> {
    if bands.is_empty() {
        return Err(DemographyConfigError::EmptySchedule { schedule });
    }
    if bands[0].start_age_years != 0 {
        return Err(DemographyConfigError::ScheduleGap {
            schedule,
            expected_start: 0,
            actual_start: bands[0].start_age_years,
        });
    }

    let mut expected_start = 0;
    for (index, band) in bands.iter().enumerate() {
        if band.start_age_years != expected_start {
            return Err(DemographyConfigError::ScheduleGap {
                schedule,
                expected_start,
                actual_start: band.start_age_years,
            });
        }
        if band.start_age_years >= band.end_age_years_exclusive {
            return Err(DemographyConfigError::InvalidBand {
                schedule,
                index,
                start: band.start_age_years,
                end_exclusive: band.end_age_years_exclusive,
            });
        }
        if band.annual_probability_per_million > PROBABILITY_PER_MILLION {
            return Err(DemographyConfigError::ProbabilityOutOfRange {
                schedule,
                index,
                value: band.annual_probability_per_million,
            });
        }
        expected_start = band.end_age_years_exclusive;
    }

    if expected_start != u32::MAX {
        return Err(DemographyConfigError::ScheduleDoesNotCoverOldAge {
            schedule,
            final_end_exclusive: expected_start,
        });
    }
    Ok(())
}

#[must_use]
pub fn annual_probability_for_age(bands: &[AgeProbabilityBand], age_days: u64) -> u32 {
    let age_years = age_days / DAYS_PER_YEAR;
    let age_years = u32::try_from(age_years).unwrap_or(u32::MAX - 1);
    bands
        .iter()
        .find(|band| age_years >= band.start_age_years && age_years < band.end_age_years_exclusive)
        .map_or(0, |band| band.annual_probability_per_million)
}

/// Executable birth-spacing lower bound under the current annual M2 scheduler.
///
/// `minimum_birth_spacing_days` is a requested day-valued lower bound, but M2 births are only
/// created at 365-day annual boundaries. The executable lower bound is therefore the smallest
/// whole number of annual boundaries that is at least the requested duration. Exposing this
/// normalization prevents the raw day value from being mistaken for subannual execution.
#[must_use]
pub fn effective_birth_spacing_days(config: &DemographyConfig) -> u64 {
    let requested = u64::from(config.minimum_birth_spacing_days);
    if requested == 0 {
        return 0;
    }
    requested
        .saturating_add(DAYS_PER_YEAR - 1)
        .div_euclid(DAYS_PER_YEAR)
        .saturating_mul(DAYS_PER_YEAR)
}

/// Stable integer probability draw used by demographic schedules.
///
/// Rejection sampling avoids modulo bias while keeping authoritative draws
/// integer-only. The RNG stream itself is version-pinned by AnthroSim's build
/// provenance.
pub(crate) fn draw_per_million<R: Rng + ?Sized>(rng: &mut R, probability: u32) -> bool {
    if probability == 0 {
        return false;
    }
    if probability >= PROBABILITY_PER_MILLION {
        return true;
    }

    draw_bounded(rng, u64::from(PROBABILITY_PER_MILLION)) < u64::from(probability)
}

fn draw_bounded<R: Rng + ?Sized>(rng: &mut R, upper_exclusive: u64) -> u64 {
    debug_assert!(upper_exclusive > 0);
    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);
    loop {
        let draw = rng.next_u64();
        if draw < acceptance_limit {
            return draw % upper_exclusive;
        }
    }
}

/// Independently seeded deterministic streams owned by the M2 demographic system.
///
/// Keeping the streams together avoids a broad lifecycle function signature
/// without coupling mortality, fertility, parent selection, or newborn sex to
/// each other's draw counts.
#[derive(Debug)]
pub(crate) struct DemographyRngs {
    mortality: ChaCha8Rng,
    fertility: ChaCha8Rng,
    parentage: ChaCha8Rng,
    newborn_sex: ChaCha8Rng,
}

impl DemographyRngs {
    #[must_use]
    pub(crate) fn new(factory: RngFactory) -> Self {
        Self {
            mortality: factory.stream("demography/mortality"),
            fertility: factory.stream("demography/fertility"),
            parentage: factory.stream("demography/parentage"),
            newborn_sex: factory.stream("demography/newborn_sex"),
        }
    }

    pub(crate) fn positions(&self) -> [RngStreamPosition; 4] {
        [
            RngStreamPosition::capture(&self.mortality),
            RngStreamPosition::capture(&self.fertility),
            RngStreamPosition::capture(&self.parentage),
            RngStreamPosition::capture(&self.newborn_sex),
        ]
    }

    pub(crate) fn restore_positions(&mut self, positions: [RngStreamPosition; 4]) {
        positions[0].restore(&mut self.mortality);
        positions[1].restore(&mut self.fertility);
        positions[2].restore(&mut self.parentage);
        positions[3].restore(&mut self.newborn_sex);
    }

    pub(crate) fn mortality_rng_mut(&mut self) -> &mut ChaCha8Rng {
        &mut self.mortality
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DemographyStepOutcome {
    Continue,
    PopulationExtinct,
    PersonRecordLimitReached,
}

/// Advance one annual M2 demographic transition for `[day - 365, day)`.
///
/// The schedule probability is selected from age at the **start** of the elapsed interval.
/// The test-only standalone transition can still draw the annual background-mortality risk in
/// one step. Authoritative simulation hosts instead partition that same annual risk over elapsed
/// M3 intervals and resolve it jointly with condition-mediated mortality before calling the annual
/// fertility/parentage finalizer below. Fertility therefore remains conditional on survival through
/// all mortality processes in the elapsed year.
///
/// Parentage locality uses the persistent-residence snapshot immediately before any M4 relocation
/// recorded on the same day. A zero-duration destination therefore cannot redefine exposure over
/// the preceding demographic interval. The newborn itself inherits the mother's current household
/// residence after that boundary's M4 state transition. M9 temporary presence remains excluded.
#[cfg(test)]
pub(crate) fn process_demographic_year(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
) -> Result<DemographyStepOutcome, PopulationError> {
    let mut events = EventLog::new();
    process_demographic_year_recorded(population, world, config, day, rngs, &mut events)
}

#[cfg(test)]
pub(crate) fn process_demographic_year_recorded(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_recorded_with_founder_history(
        population, world, config, day, rngs, events, None,
    )
}

#[cfg(test)]
pub(crate) fn process_demographic_year_recorded_with_founder_history(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_recorded_internal(
        population,
        world,
        config,
        day,
        rngs,
        events,
        true,
        founder_population,
    )
}

pub(crate) fn process_demographic_year_after_competing_mortality_recorded_with_founder_history(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_recorded_internal(
        population,
        world,
        config,
        day,
        rngs,
        events,
        false,
        founder_population,
    )
}

#[allow(clippy::too_many_arguments)]
fn process_demographic_year_recorded_internal(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    apply_background_mortality: bool,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
    if day < DAYS_PER_YEAR || !day.is_multiple_of(DAYS_PER_YEAR) {
        return Err(PopulationError::InternalInvariant {
            reason: "M2 demographic transition must run at a positive annual boundary",
        });
    }

    let interval_start_day = day - DAYS_PER_YEAR;
    let records_at_boundary_start = population.person_count();
    let same_day_migration_origins = same_day_migration_origins(events, day);

    if apply_background_mortality {
        for index in 0..records_at_boundary_start {
            if !population.is_alive_index(index) {
                continue;
            }
            let age_days = population
                .age_days_at_index(index, interval_start_day)
                .ok_or(PopulationError::InternalInvariant {
                    reason: "living person has no representable age at demographic interval start",
                })?;
            let probability = annual_probability_for_age(&config.mortality_bands, age_days);
            if draw_per_million(&mut rngs.mortality, probability) {
                let person = population.person_id_at_index(index).ok_or(
                    PopulationError::InternalInvariant {
                        reason: "living person is missing a stable ID at mortality boundary",
                    },
                )?;
                let household = population.household_at_index(index).ok_or(
                    PopulationError::InternalInvariant {
                        reason: "living person is missing a household at mortality boundary",
                    },
                )?;
                let cell = population.location_at_index(index).ok_or(
                    PopulationError::InternalInvariant {
                        reason: "living person is missing a current residence at mortality boundary",
                    },
                )?;
                let condition = population.condition_at_index(index).ok_or(
                    PopulationError::InternalInvariant {
                        reason: "living person is missing condition at mortality boundary",
                    },
                )?;
                if population.mark_death(index, day) {
                    events.push_authoritative(
                        day,
                        EventKind::Death {
                            person,
                            household,
                            cell,
                            cause: DeathCause::DemographicMortality,
                            condition_permille: condition,
                            probability_per_million: probability,
                        },
                    );
                }
            }
        }
    }
    if population.living_count() == 0 {
        return Ok(DemographyStepOutcome::PopulationExtinct);
    }

    let parentage_occupancy = build_parentage_occupancy(
        population,
        records_at_boundary_start,
        &same_day_migration_origins,
    )?;
    let executable_birth_spacing_days = effective_birth_spacing_days(config);
    let role_ranks = demographic_role_ranks(
        population,
        records_at_boundary_start,
        day,
        &same_day_migration_origins,
        founder_population,
    )?;

    // Freeze the eligible-female set before any birth is appended, matching the historical annual
    // boundary semantics, but assign the shared fertility stream in a scientific-state order rather
    // than packed-record/PersonId order. The final PersonId tie-break is reached only for records
    // that remain indistinguishable under the complete relabelling-invariant role refinement.
    let mut fertility_candidates = Vec::new();
    for female_index in 0..records_at_boundary_start {
        if !population.is_alive_index(female_index)
            || population.reproductive_sex_at_index(female_index) != Some(ReproductiveSex::Female)
        {
            continue;
        }

        let age_days = population
            .age_days_at_index(female_index, interval_start_day)
            .ok_or(PopulationError::InternalInvariant {
                reason: "living female has no representable age at fertility interval start",
            })?;
        let fertility_probability = annual_probability_for_age(&config.fertility_bands, age_days);
        if fertility_probability == 0 {
            continue;
        }

        if prior_birth_elapsed_days(population, female_index, day, founder_population)
            .is_some_and(|elapsed| elapsed < executable_birth_spacing_days)
        {
            continue;
        }

        let parentage_location =
            demographic_exposure_location(population, female_index, &same_day_migration_origins)
                .ok_or(PopulationError::InternalInvariant {
                    reason: "living female is missing a demographic exposure residence",
                })?;
        let eligible_males = parentage_occupancy
            .get(&parentage_location)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if !has_eligible_male(eligible_males, population, interval_start_day, config) {
            continue;
        }
        let female = population.person_id_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a stable person ID while ordering fertility candidates",
            },
        )?;
        fertility_candidates.push((
            role_ranks[female_index],
            female,
            female_index,
            fertility_probability,
            parentage_location,
        ));
    }
    fertility_candidates.sort_by_key(|candidate| (candidate.0, candidate.1));

    let mut births_added = false;
    for (_, female_parent, female_index, fertility_probability, parentage_location) in
        fertility_candidates
    {
        if !draw_per_million(&mut rngs.fertility, fertility_probability) {
            continue;
        }

        if population.record_limit_reached() {
            if births_added {
                population.rebuild_occupancy(world)?;
            }
            return Ok(DemographyStepOutcome::PersonRecordLimitReached);
        }

        let eligible_males = parentage_occupancy
            .get(&parentage_location)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let male_parent = select_male_parent(
            eligible_males,
            population,
            interval_start_day,
            config,
            &mut rngs.parentage,
        )
        .ok_or(PopulationError::InternalInvariant {
            reason: "eligible pre-boundary local male disappeared during a demographic boundary",
        })?;
        let household = population.household_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a household",
            },
        )?;
        let birth_residence = population.location_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a current birth residence",
            },
        )?;
        let newborn_condition = population.condition_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing condition at fertility boundary",
            },
        )?;

        let male_probability = u32::from(config.male_birth_permille) * 1_000;
        let newborn_sex = if draw_per_million(&mut rngs.newborn_sex, male_probability) {
            ReproductiveSex::Male
        } else {
            ReproductiveSex::Female
        };

        let newborn = population.append_birth(
            day,
            newborn_sex,
            birth_residence,
            household,
            female_parent,
            male_parent,
        )?;
        population.note_successful_birth(female_index, day);
        let newborn_index = population.person_count().saturating_sub(1);
        if population.person_id_at_index(newborn_index) != Some(newborn)
            || !population.set_condition_at_index(newborn_index, newborn_condition)
        {
            return Err(PopulationError::InternalInvariant {
                reason: "newborn condition could not be initialized from the female parent",
            });
        }
        events.push_authoritative(
            day,
            EventKind::Birth {
                person: newborn,
                female_parent,
                male_parent,
                household,
                cell: birth_residence,
                reproductive_sex: newborn_sex,
            },
        );
        births_added = true;

        if population.record_limit_reached() {
            population.rebuild_occupancy(world)?;
            return Ok(DemographyStepOutcome::PersonRecordLimitReached);
        }
    }

    if births_added {
        population.rebuild_occupancy(world)?;
    }

    Ok(DemographyStepOutcome::Continue)
}

fn prior_birth_elapsed_days(
    population: &Population,
    female_index: usize,
    day: u64,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Option<u64> {
    if let Some(last_birth_day) = population.last_birth_day_at_index(female_index) {
        return day.checked_sub(last_birth_day);
    }

    let founder_population = founder_population?;
    let person = population.person_id_at_index(female_index)?;
    let last_birth_day = founder_population.last_birth_day(person)?;
    let current_day = i64::try_from(day).ok()?;
    let elapsed = current_day.checked_sub(last_birth_day)?;
    u64::try_from(elapsed).ok()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DemographicRoleBaseKey {
    alive: bool,
    birth_day: i64,
    death_day: Option<u64>,
    reproductive_sex_rank: u8,
    exposure_location: CellId,
    current_location: CellId,
    condition_permille: u16,
    condition_loss_remainder_thousandths: u16,
    prior_birth_elapsed_days: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DemographicRoleSignature {
    prior_rank: u64,
    female_parent_rank: Option<u64>,
    male_parent_rank: Option<u64>,
    children: Vec<(u8, u64)>,
    living_household_members: Vec<u64>,
}

fn reproductive_sex_rank(sex: ReproductiveSex) -> u8 {
    match sex {
        ReproductiveSex::Female => 0,
        ReproductiveSex::Male => 1,
    }
}

fn demographic_person_index(person: PersonId, records_at_boundary_start: usize) -> Option<usize> {
    if person == PersonId::INVALID {
        return None;
    }
    usize::try_from(person.0.checked_sub(1)?)
        .ok()
        .filter(|&index| index < records_at_boundary_start)
}

/// Derive stable, relabelling-invariant scientific role classes for the complete pre-birth
/// demographic state. The refinement includes local scalar state, spatial exposure, parent/child
/// roles, and the multiset of living household-member roles. Because each iteration includes the
/// prior rank, the partition only refines and converges after at most one split per represented
/// record. Canonical PersonId is intentionally absent from every signature.
fn demographic_role_ranks(
    population: &Population,
    records_at_boundary_start: usize,
    day: u64,
    same_day_migration_origins: &BTreeMap<HouseholdId, CellId>,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<Vec<u64>, PopulationError> {
    let mut female_parent_indices = vec![None; records_at_boundary_start];
    let mut male_parent_indices = vec![None; records_at_boundary_start];
    let mut child_links = vec![Vec::<(u8, usize)>::new(); records_at_boundary_start];
    let mut living_households = BTreeMap::<HouseholdId, Vec<usize>>::new();
    let mut base_keys = Vec::with_capacity(records_at_boundary_start);

    for index in 0..records_at_boundary_start {
        let person_id = population.person_id_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement found a record without stable PersonId",
            },
        )?;
        let person = population
            .person(person_id)
            .ok_or(PopulationError::InternalInvariant {
                reason: "demographic role refinement could not materialize a person record",
            })?;
        let household = population.household_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement found a record without household",
            },
        )?;
        if person.is_alive() {
            living_households.entry(household).or_default().push(index);
        }

        let female_parent = population.female_parent_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement could not read female-parent state",
            },
        )?;
        let male_parent = population.male_parent_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement could not read male-parent state",
            },
        )?;
        female_parent_indices[index] =
            demographic_person_index(female_parent, records_at_boundary_start);
        male_parent_indices[index] = demographic_person_index(male_parent, records_at_boundary_start);
        if female_parent != PersonId::INVALID && female_parent_indices[index].is_none() {
            return Err(PopulationError::InternalInvariant {
                reason: "female-parent reference is outside the demographic boundary record set",
            });
        }
        if male_parent != PersonId::INVALID && male_parent_indices[index].is_none() {
            return Err(PopulationError::InternalInvariant {
                reason: "male-parent reference is outside the demographic boundary record set",
            });
        }

        let current_location = population.location_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement found a record without residence",
            },
        )?;
        let exposure_location =
            demographic_exposure_location(population, index, same_day_migration_origins).ok_or(
                PopulationError::InternalInvariant {
                    reason: "demographic role refinement found a record without exposure residence",
                },
            )?;
        let condition_permille = population.condition_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "demographic role refinement could not read condition",
            },
        )?;
        let condition_loss_remainder_thousandths = population
            .condition_loss_remainder_thousandths_at_index(index)
            .ok_or(PopulationError::InternalInvariant {
                reason: "demographic role refinement could not read condition remainder",
            })?;
        let prior_birth = (person.reproductive_sex == ReproductiveSex::Female)
            .then(|| prior_birth_elapsed_days(population, index, day, founder_population))
            .flatten();
        base_keys.push(DemographicRoleBaseKey {
            alive: person.is_alive(),
            birth_day: person.birth_day,
            death_day: person.death_day,
            reproductive_sex_rank: reproductive_sex_rank(person.reproductive_sex),
            exposure_location,
            current_location,
            condition_permille,
            condition_loss_remainder_thousandths,
            prior_birth_elapsed_days: prior_birth,
        });
    }

    for child_index in 0..records_at_boundary_start {
        if let Some(parent_index) = female_parent_indices[child_index] {
            child_links[parent_index].push((0, child_index));
        }
        if let Some(parent_index) = male_parent_indices[child_index] {
            child_links[parent_index].push((1, child_index));
        }
    }

    let mut unique_base_keys = base_keys.clone();
    unique_base_keys.sort();
    unique_base_keys.dedup();
    let mut ranks = base_keys
        .iter()
        .map(|key| {
            let position = unique_base_keys
                .binary_search(key)
                .expect("demographic base role must be represented");
            u64::try_from(position).expect("demographic role-rank space must fit u64")
        })
        .collect::<Vec<_>>();

    for _ in 0..records_at_boundary_start {
        let mut household_rank_multisets = BTreeMap::<HouseholdId, Vec<u64>>::new();
        for (&household, members) in &living_households {
            let mut member_ranks = members.iter().map(|&index| ranks[index]).collect::<Vec<_>>();
            member_ranks.sort_unstable();
            household_rank_multisets.insert(household, member_ranks);
        }

        let mut signatures = Vec::with_capacity(records_at_boundary_start);
        for index in 0..records_at_boundary_start {
            let mut children = child_links[index]
                .iter()
                .map(|&(role, child_index)| (role, ranks[child_index]))
                .collect::<Vec<_>>();
            children.sort_unstable();
            let household = population.household_at_index(index).ok_or(
                PopulationError::InternalInvariant {
                    reason: "demographic role refinement lost household state",
                },
            )?;
            let living_household_members = household_rank_multisets
                .get(&household)
                .cloned()
                .unwrap_or_default();
            signatures.push(DemographicRoleSignature {
                prior_rank: ranks[index],
                female_parent_rank: female_parent_indices[index].map(|parent| ranks[parent]),
                male_parent_rank: male_parent_indices[index].map(|parent| ranks[parent]),
                children,
                living_household_members,
            });
        }

        let mut unique_signatures = signatures.clone();
        unique_signatures.sort();
        unique_signatures.dedup();
        let next_ranks = signatures
            .iter()
            .map(|signature| {
                let position = unique_signatures
                    .binary_search(signature)
                    .expect("demographic role signature must be represented");
                u64::try_from(position).expect("demographic role-rank space must fit u64")
            })
            .collect::<Vec<_>>();
        if next_ranks == ranks {
            return Ok(ranks);
        }
        ranks = next_ranks;
    }

    Ok(ranks)
}

fn same_day_migration_origins(events: &EventLog, day: u64) -> BTreeMap<HouseholdId, CellId> {
    let mut origins = BTreeMap::new();
    for record in &events.events {
        if record.day != day {
            continue;
        }
        if let EventKind::HouseholdMigration {
            household, origin, ..
        } = &record.event
        {
            // The first same-day migration event is the residence immediately before any M4 move
            // on this boundary. M4 normally permits at most one move per household per boundary;
            // preserving the first origin also makes this helper robust to malformed duplicate
            // events without allowing a later zero-duration location to redefine M2 exposure.
            origins.entry(*household).or_insert(*origin);
        }
    }
    origins
}

fn demographic_exposure_location(
    population: &Population,
    index: usize,
    same_day_migration_origins: &BTreeMap<HouseholdId, CellId>,
) -> Option<CellId> {
    let household = population.household_at_index(index)?;
    same_day_migration_origins
        .get(&household)
        .copied()
        .or_else(|| population.location_at_index(index))
}

fn build_parentage_occupancy(
    population: &Population,
    records_at_boundary_start: usize,
    same_day_migration_origins: &BTreeMap<HouseholdId, CellId>,
) -> Result<BTreeMap<CellId, Vec<PersonId>>, PopulationError> {
    let mut occupancy: BTreeMap<CellId, Vec<PersonId>> = BTreeMap::new();
    for index in 0..records_at_boundary_start {
        if !population.is_alive_index(index) {
            continue;
        }
        let person = population.person_id_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living person is missing a stable ID while building parentage occupancy",
            },
        )?;
        let location = demographic_exposure_location(population, index, same_day_migration_origins)
            .ok_or(PopulationError::InternalInvariant {
                reason: "living person is missing a residence while building parentage occupancy",
            })?;
        occupancy.entry(location).or_default().push(person);
    }
    Ok(occupancy)
}

fn has_eligible_male(
    people: &[PersonId],
    population: &Population,
    exposure_start_day: u64,
    config: &DemographyConfig,
) -> bool {
    people
        .iter()
        .copied()
        .any(|candidate| male_is_eligible(population, candidate, exposure_start_day, config))
}

fn select_male_parent<R: Rng + ?Sized>(
    people: &[PersonId],
    population: &Population,
    exposure_start_day: u64,
    config: &DemographyConfig,
    rng: &mut R,
) -> Option<PersonId> {
    let mut selected = None;
    let mut eligible_seen = 0_u64;

    for &candidate in people {
        if !male_is_eligible(population, candidate, exposure_start_day, config) {
            continue;
        }
        eligible_seen = eligible_seen.saturating_add(1);
        if draw_bounded(rng, eligible_seen) == 0 {
            selected = Some(candidate);
        }
    }
    selected
}

fn male_is_eligible(
    population: &Population,
    candidate: PersonId,
    exposure_start_day: u64,
    config: &DemographyConfig,
) -> bool {
    let Some(person) = population.person(candidate) else {
        return false;
    };
    if !person.is_alive() || person.reproductive_sex != ReproductiveSex::Male {
        return false;
    }

    let Some(age_days) = person.age_days_at(SimTime::from_days(exposure_start_day)) else {
        return false;
    };
    let age_years = age_days / DAYS_PER_YEAR;
    age_years >= u64::from(config.male_parent_min_age_years)
        && age_years < u64::from(config.male_parent_max_age_years_exclusive)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DemographyConfigError {
    #[error("demography schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("demography schedule ID must not be empty")]
    EmptyScheduleId,
    #[error("{schedule} schedule must contain at least one age band")]
    EmptySchedule { schedule: &'static str },
    #[error(
        "{schedule} schedule has a gap: expected next band at age {expected_start}, found {actual_start}"
    )]
    ScheduleGap {
        schedule: &'static str,
        expected_start: u32,
        actual_start: u32,
    },
    #[error("{schedule} schedule band {index} has invalid range {start}..{end_exclusive}")]
    InvalidBand {
        schedule: &'static str,
        index: usize,
        start: u32,
        end_exclusive: u32,
    },
    #[error("{schedule} schedule band {index} probability {value} exceeds one million")]
    ProbabilityOutOfRange {
        schedule: &'static str,
        index: usize,
        value: u32,
    },
    #[error(
        "{schedule} schedule ends at age {final_end_exclusive} instead of covering open-ended old age"
    )]
    ScheduleDoesNotCoverOldAge {
        schedule: &'static str,
        final_end_exclusive: u32,
    },
    #[error("male live-birth share {value} permille is outside 0..=1000")]
    InvalidMaleBirthPermille { value: u16 },
    #[error("male parent age range is invalid: {minimum}..{maximum_exclusive}")]
    InvalidMaleParentAgeRange {
        minimum: u32,
        maximum_exclusive: u32,
    },
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        config::{ParameterProvenance, PopulationConfig, PopulationInitialization, WorldConfig},
        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
        },
        migration::MigrationUtilityBreakdown,
    };

    #[test]
    fn default_synthetic_schedule_is_structurally_valid() {
        validate_demography_config(&DemographyConfig::synthetic_validation_v1()).unwrap();
    }

    #[test]
    fn mortality_lookup_respects_half_open_age_bands() {
        let config = DemographyConfig::synthetic_validation_v1();
        assert_eq!(
            annual_probability_for_age(&config.mortality_bands, 0),
            180_000
        );
        assert_eq!(
            annual_probability_for_age(&config.mortality_bands, DAYS_PER_YEAR - 1),
            180_000
        );
        assert_eq!(
            annual_probability_for_age(&config.mortality_bands, DAYS_PER_YEAR),
            50_000
        );
        assert_eq!(
            annual_probability_for_age(&config.mortality_bands, 75 * DAYS_PER_YEAR),
            300_000
        );
    }

    #[test]
    fn annual_spacing_normalization_is_explicit_at_boundaries() {
        let mut config = DemographyConfig::synthetic_validation_v1();
        for (requested, effective) in [
            (0, 0),
            (365, 365),
            (366, 730),
            (730, 730),
            (731, 1_095),
            (1_278, 1_460),
            (1_460, 1_460),
        ] {
            config.minimum_birth_spacing_days = requested;
            assert_eq!(effective_birth_spacing_days(&config), effective);
        }
    }

    #[test]
    fn probability_extremes_are_exact() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert!(!draw_per_million(&mut rng, 0));
        assert!(draw_per_million(&mut rng, PROBABILITY_PER_MILLION));
    }

    #[test]
    fn certain_mortality_can_extinguish_a_population() {
        let world = World::generate(WorldConfig::new(4, 4), RngFactory::new(9)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(9)).unwrap();
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }

        let mut rngs = DemographyRngs::new(RngFactory::new(9));
        let outcome =
            process_demographic_year(&mut population, &world, &config, DAYS_PER_YEAR, &mut rngs)
                .unwrap();

        assert_eq!(outcome, DemographyStepOutcome::PopulationExtinct);
        assert_eq!(population.living_count(), 0);
        assert_eq!(population.summary().deaths_since_start, 100);
    }

    #[test]
    fn high_mortality_and_high_fertility_follow_conditional_survival_contract() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(90)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(200), &world, RngFactory::new(90))
                .unwrap();
        let mut config = DemographyConfig::synthetic_validation_v1();
        config.minimum_birth_spacing_days = 0;
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }

        let mut rngs = DemographyRngs::new(RngFactory::new(90));
        let outcome =
            process_demographic_year(&mut population, &world, &config, DAYS_PER_YEAR, &mut rngs)
                .unwrap();

        assert_eq!(outcome, DemographyStepOutcome::PopulationExtinct);
        assert_eq!(population.summary().births_since_start, 0);
        assert_eq!(population.summary().deaths_since_start, 200);
    }

    #[test]
    fn model_born_child_receives_age_zero_mortality_interval() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(91)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(91)).unwrap();
        let female_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Female)
            })
            .unwrap();
        let male_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Male)
            })
            .unwrap();
        let female = population.person_id_at_index(female_index).unwrap();
        let male = population.person_id_at_index(male_index).unwrap();
        let household = population.household_at_index(female_index).unwrap();
        let location = population.location_at_index(female_index).unwrap();
        let child = population
            .append_birth(
                DAYS_PER_YEAR,
                ReproductiveSex::Male,
                location,
                household,
                female,
                male,
            )
            .unwrap();
        population.note_successful_birth(female_index, DAYS_PER_YEAR);
        population.rebuild_occupancy(&world).unwrap();

        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        config.mortality_bands[0].annual_probability_per_million = PROBABILITY_PER_MILLION;
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }

        let mut rngs = DemographyRngs::new(RngFactory::new(91));
        process_demographic_year(
            &mut population,
            &world,
            &config,
            2 * DAYS_PER_YEAR,
            &mut rngs,
        )
        .unwrap();

        assert_eq!(
            population.person(child).and_then(|person| person.death_day),
            Some(2 * DAYS_PER_YEAR)
        );
    }

    #[test]
    fn declared_recent_pre_run_birth_blocks_first_boundary_fertility() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(94)).unwrap();
        let mut definition = FounderPopulationDefinition::new(
            "spacing-history-test-v1",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -(25 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: Some(-100),
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -(30 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
            ],
        );
        let population_config = PopulationConfig::new(2)
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }

        let mut recent = Population::initialize_declared_founder_state_v1(
            population_config,
            &definition,
            &world,
            &config,
        )
        .unwrap();
        let mut recent_rngs = DemographyRngs::new(RngFactory::new(94));
        let mut recent_events = EventLog::new();
        process_demographic_year_recorded_with_founder_history(
            &mut recent,
            &world,
            &config,
            DAYS_PER_YEAR,
            &mut recent_rngs,
            &mut recent_events,
            Some(&definition),
        )
        .unwrap();
        assert_eq!(recent.summary().births_since_start, 0);

        definition.people[0].last_birth_day = Some(-2_000);
        let mut distant = Population::initialize_declared_founder_state_v1(
            population_config,
            &definition,
            &world,
            &config,
        )
        .unwrap();
        let mut distant_rngs = DemographyRngs::new(RngFactory::new(94));
        let mut distant_events = EventLog::new();
        process_demographic_year_recorded_with_founder_history(
            &mut distant,
            &world,
            &config,
            DAYS_PER_YEAR,
            &mut distant_rngs,
            &mut distant_events,
            Some(&definition),
        )
        .unwrap();
        assert_eq!(distant.summary().births_since_start, 1);
    }

    #[test]
    fn newborn_condition_inherits_female_parent_condition() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(92)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(300), &world, RngFactory::new(92))
                .unwrap();
        let config = DemographyConfig::synthetic_validation_v1();
        let interval_start_day = 0;
        let female_index = (0..population.person_count())
            .find(|&index| {
                if population.reproductive_sex_at_index(index) != Some(ReproductiveSex::Female) {
                    return false;
                }
                let age_days = population
                    .age_days_at_index(index, interval_start_day)
                    .unwrap();
                let probability = annual_probability_for_age(&config.fertility_bands, age_days);
                probability > 0
            })
            .unwrap();
        assert!(population.set_condition_at_index(female_index, 275));
        let female = population.person_id_at_index(female_index).unwrap();

        let female_age_years = population
            .age_days_at_index(female_index, interval_start_day)
            .unwrap()
            / DAYS_PER_YEAR;
        let mut config = config;
        config.minimum_birth_spacing_days = 0;
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = if female_age_years
                >= u64::from(band.start_age_years)
                && female_age_years < u64::from(band.end_age_years_exclusive)
            {
                PROBABILITY_PER_MILLION
            } else {
                0
            };
        }

        let mut rngs = DemographyRngs::new(RngFactory::new(92));
        let mut events = EventLog::new();
        process_demographic_year_recorded(
            &mut population,
            &world,
            &config,
            DAYS_PER_YEAR,
            &mut rngs,
            &mut events,
        )
        .unwrap();

        let newborn = events
            .events
            .iter()
            .find_map(|record| match &record.event {
                EventKind::Birth {
                    person,
                    female_parent,
                    ..
                } if *female_parent == female => Some(*person),
                _ => None,
            })
            .expect("selected female should have a certain fertility opportunity");
        assert_eq!(population.person(newborn).unwrap().condition_permille, 275);
    }

    #[test]
    fn same_day_migration_destination_does_not_redefine_parentage_locality() {
        let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(93)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(600), &world, RngFactory::new(93))
                .unwrap();
        let base_config = DemographyConfig::synthetic_validation_v1();
        let interval_start_day = 0;

        let mut selected = None;
        for female_index in 0..population.person_count() {
            if population.reproductive_sex_at_index(female_index) != Some(ReproductiveSex::Female) {
                continue;
            }
            let female_age_days = population
                .age_days_at_index(female_index, interval_start_day)
                .unwrap();
            if annual_probability_for_age(&base_config.fertility_bands, female_age_days) == 0 {
                continue;
            }
            let origin = population.location_at_index(female_index).unwrap();
            let destination = if origin == CellId::new(1) {
                CellId::new(2)
            } else {
                CellId::new(1)
            };
            let female_household = population.household_at_index(female_index).unwrap();

            let eligible_in_household = (0..population.person_count()).any(|index| {
                population.household_at_index(index) == Some(female_household)
                    && population.person_id_at_index(index).is_some_and(|person| {
                        male_is_eligible(&population, person, interval_start_day, &base_config)
                    })
            });
            if eligible_in_household {
                continue;
            }

            let origin_males: Vec<_> = (0..population.person_count())
                .filter(|&index| population.location_at_index(index) == Some(origin))
                .filter_map(|index| population.person_id_at_index(index))
                .filter(|&person| {
                    male_is_eligible(&population, person, interval_start_day, &base_config)
                })
                .collect();
            let destination_males: Vec<_> = (0..population.person_count())
                .filter(|&index| population.location_at_index(index) == Some(destination))
                .filter_map(|index| population.person_id_at_index(index))
                .filter(|&person| {
                    male_is_eligible(&population, person, interval_start_day, &base_config)
                })
                .collect();
            if !origin_males.is_empty() && !destination_males.is_empty() {
                selected = Some((
                    female_index,
                    female_household,
                    origin,
                    destination,
                    origin_males,
                    destination_males,
                ));
                break;
            }
        }

        let (female_index, female_household, origin, destination, origin_males, destination_males) =
            selected.expect("fixture should contain a suitable separated parentage case");
        let female = population.person_id_at_index(female_index).unwrap();

        let mut destinations = vec![CellId::INVALID; population.household_count()];
        let household_index = usize::try_from(female_household.0 - 1).unwrap();
        destinations[household_index] = destination;
        let condition_costs = vec![0; population.household_count()];
        let relocation = population
            .apply_household_relocations(&destinations, &condition_costs, &world)
            .unwrap();
        assert_eq!(
            population.location_at_index(female_index),
            Some(destination)
        );

        let zero_utility = MigrationUtilityBreakdown {
            resource_score_permille: 0,
            water_security_score_permille: 0,
            kin_score_permille: 0,
            travel_penalty_permille: 0,
            uncertainty_penalty_permille: 0,
            relocation_risk_penalty_permille: 0,
            total_utility: 0,
        };
        let mut events = EventLog::new();
        events.push_authoritative(
            DAYS_PER_YEAR,
            EventKind::HouseholdMigration {
                household: female_household,
                people_moved: u32::try_from(relocation.people_moved).unwrap(),
                origin,
                destination,
                distance_cells: 1,
                pressure_permille: 0,
                origin_utility: zero_utility,
                destination_utility: zero_utility,
                best_candidate: destination,
                best_candidate_utility: 0,
                selected_weight: 1,
                total_move_weight: 1,
                choice_draw: 0,
                nominal_travel_condition_cost_per_person: 0,
                realized_travel_condition_loss_total: 0,
            },
        );

        let female_age_years = population
            .age_days_at_index(female_index, interval_start_day)
            .unwrap()
            / DAYS_PER_YEAR;
        let mut config = base_config;
        config.minimum_birth_spacing_days = 0;
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = if female_age_years
                >= u64::from(band.start_age_years)
                && female_age_years < u64::from(band.end_age_years_exclusive)
            {
                PROBABILITY_PER_MILLION
            } else {
                0
            };
        }

        let mut rngs = DemographyRngs::new(RngFactory::new(93));
        process_demographic_year_recorded(
            &mut population,
            &world,
            &config,
            DAYS_PER_YEAR,
            &mut rngs,
            &mut events,
        )
        .unwrap();

        let (newborn, male_parent, birth_cell) = events
            .events
            .iter()
            .find_map(|record| match &record.event {
                EventKind::Birth {
                    person,
                    female_parent,
                    male_parent,
                    cell,
                    ..
                } if *female_parent == female => Some((*person, *male_parent, *cell)),
                _ => None,
            })
            .expect("selected female should have a certain fertility opportunity");

        assert!(origin_males.contains(&male_parent));
        assert!(!destination_males.contains(&male_parent));
        assert_eq!(birth_cell, destination);
        assert_eq!(population.person(newborn).unwrap().location, destination);
    }
}
