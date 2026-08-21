use rand::Rng;
use thiserror::Error;

use crate::{
    config::{
        AgeProbabilityBand, DemographyConfig, PROBABILITY_PER_MILLION,
    },
    time::DAYS_PER_YEAR,
    world::PERMILLE_MAX,
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
        .find(|band| {
            age_years >= band.start_age_years && age_years < band.end_age_years_exclusive
        })
        .map_or(0, |band| band.annual_probability_per_million)
}

/// Stable integer probability draw used by demographic schedules.
///
/// Rejection sampling avoids modulo bias while keeping the authoritative draw
/// integer-only. The RNG stream itself is version-pinned by AnthroSim's build
/// provenance.
pub(crate) fn draw_per_million<R: Rng + ?Sized>(rng: &mut R, probability: u32) -> bool {
    if probability == 0 {
        return false;
    }
    if probability >= PROBABILITY_PER_MILLION {
        return true;
    }

    let scale = u64::from(PROBABILITY_PER_MILLION);
    let acceptance_limit = u64::MAX - (u64::MAX % scale);
    loop {
        let draw = rng.next_u64();
        if draw < acceptance_limit {
            return draw % scale < u64::from(probability);
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DemographyConfigError {
    #[error("demography schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("demography schedule ID must not be empty")]
    EmptyScheduleId,
    #[error("{schedule} schedule must contain at least one age band")]
    EmptySchedule { schedule: &'static str },
    #[error("{schedule} schedule has a gap: expected next band at age {expected_start}, found {actual_start}")]
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
    #[error("{schedule} schedule ends at age {final_end_exclusive} instead of covering open-ended old age")]
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

    #[test]
    fn default_synthetic_schedule_is_structurally_valid() {
        validate_demography_config(&DemographyConfig::synthetic_validation_v1()).unwrap();
    }

    #[test]
    fn mortality_lookup_respects_half_open_age_bands() {
        let config = DemographyConfig::synthetic_validation_v1();
        assert_eq!(annual_probability_for_age(&config.mortality_bands, 0), 180_000);
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
    fn probability_extremes_do_not_consume_semantic_uncertainty() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert!(!draw_per_million(&mut rng, 0));
        assert!(draw_per_million(&mut rng, PROBABILITY_PER_MILLION));
    }
}
