use rand::Rng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::{
    config::{AgeProbabilityBand, DemographyConfig, PROBABILITY_PER_MILLION},
    ids::{CellId, PersonId},
    population::{Population, PopulationError, ReproductiveSex},
    rng::RngFactory,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DemographyStepOutcome {
    Continue,
    PopulationExtinct,
    PersonRecordLimitReached,
}

/// Advance one annual M2 demographic boundary.
///
/// Scheduling is intentionally explicit: mortality is evaluated first for
/// records that existed at the start of the boundary, then fertility is
/// evaluated among surviving female records. Newborns are not exposed to a
/// mortality draw until a later annual boundary. M2 father selection is local
/// to the mother's current cell and uniform among eligible living male records;
/// it does not model marriage, pair bonds, kin avoidance, or social paternity.
pub(crate) fn process_demographic_year(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
) -> Result<DemographyStepOutcome, PopulationError> {
    let records_at_boundary_start = population.person_count();

    for index in 0..records_at_boundary_start {
        if !population.is_alive_index(index) {
            continue;
        }
        let age_days =
            population
                .age_days_at_index(index, day)
                .ok_or(PopulationError::InternalInvariant {
                    reason: "living person has no representable age at demographic boundary",
                })?;
        let probability = annual_probability_for_age(&config.mortality_bands, age_days);
        if draw_per_million(&mut rngs.mortality, probability) {
            population.mark_death(index, day);
        }
    }

    if population.living_count() == 0 {
        return Ok(DemographyStepOutcome::PopulationExtinct);
    }

    let mut births_added = false;
    for female_index in 0..records_at_boundary_start {
        if !population.is_alive_index(female_index)
            || population.reproductive_sex_at_index(female_index) != Some(ReproductiveSex::Female)
        {
            continue;
        }

        let age_days = population.age_days_at_index(female_index, day).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female has no representable age at fertility boundary",
            },
        )?;
        let fertility_probability = annual_probability_for_age(&config.fertility_bands, age_days);
        if fertility_probability == 0 {
            continue;
        }

        if let Some(last_birth_day) = population.last_birth_day_at_index(female_index)
            && day.saturating_sub(last_birth_day) < u64::from(config.minimum_birth_spacing_days)
        {
            continue;
        }

        let location = population.location_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a location",
            },
        )?;
        if !has_eligible_male_in_cell(population, location, day, config) {
            continue;
        }
        if !draw_per_million(&mut rngs.fertility, fertility_probability) {
            continue;
        }

        if population.record_limit_reached() {
            if births_added {
                population.rebuild_occupancy(world)?;
            }
            return Ok(DemographyStepOutcome::PersonRecordLimitReached);
        }

        let male_parent = select_male_parent(population, location, day, config, &mut rngs.parentage)
            .ok_or(PopulationError::InternalInvariant {
                reason: "eligible local male disappeared during a demographic boundary",
            })?;
        let female_parent = population.person_id_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a stable person ID",
            },
        )?;
        let household = population.household_at_index(female_index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a household",
            },
        )?;

        let male_probability = u32::from(config.male_birth_permille) * 1_000;
        let newborn_sex = if draw_per_million(&mut rngs.newborn_sex, male_probability) {
            ReproductiveSex::Male
        } else {
            ReproductiveSex::Female
        };

        population.append_birth(
            day,
            newborn_sex,
            location,
            household,
            female_parent,
            male_parent,
        )?;
        population.note_successful_birth(female_index, day);
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

fn has_eligible_male_in_cell(
    population: &Population,
    location: CellId,
    day: u64,
    config: &DemographyConfig,
) -> bool {
    population
        .occupancy()
        .people_in_cell(location)
        .is_some_and(|people| {
            people
                .iter()
                .copied()
                .any(|candidate| male_is_eligible(population, candidate, location, day, config))
        })
}

fn select_male_parent<R: Rng + ?Sized>(
    population: &Population,
    location: CellId,
    day: u64,
    config: &DemographyConfig,
    rng: &mut R,
) -> Option<PersonId> {
    let people = population.occupancy().people_in_cell(location)?;
    let mut selected = None;
    let mut eligible_seen = 0_u64;

    for &candidate in people {
        if !male_is_eligible(population, candidate, location, day, config) {
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
    location: CellId,
    day: u64,
    config: &DemographyConfig,
) -> bool {
    let Some(person) = population.person(candidate) else {
        return false;
    };
    if !person.is_alive()
        || person.reproductive_sex != ReproductiveSex::Male
        || person.location != location
    {
        return false;
    }

    let Some(age_days) = person.age_days_at(SimTime::from_days(day)) else {
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
    use crate::{config::PopulationConfig, world::WorldConfig};

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
            annual_probability_for_age(&config.mortality_bands, DAYS_PER_YEAR),
            50_000
        );
        assert_eq!(
            annual_probability_for_age(&config.mortality_bands, 75 * DAYS_PER_YEAR),
            300_000
        );
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
        let outcome = process_demographic_year(
            &mut population,
            &world,
            &config,
            DAYS_PER_YEAR,
            &mut rngs,
        )
        .unwrap();

        assert_eq!(outcome, DemographyStepOutcome::PopulationExtinct);
        assert_eq!(population.living_count(), 0);
        assert_eq!(population.summary().deaths_since_start, 100);
    }
}
