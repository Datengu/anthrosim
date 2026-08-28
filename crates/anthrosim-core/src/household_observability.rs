use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    EventKind, EventLog, EventProvenance,
    config::{ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID},
    events::HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,
    household_lifecycle::household_lifecycle_model_id,
    ids::{HouseholdId, PersonId},
    population::Population,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdSizeBin {
    pub living_members: u32,
    pub household_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdAgeBin {
    pub age_days: u64,
    pub household_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdGenerationSpanBin {
    /// Number of genealogical generations represented among living members of one household.
    pub generations: u32,
    pub household_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdObservabilityReport {
    pub schema_version: u32,
    pub day: u64,
    pub lifecycle_model_id: String,
    pub total_household_records: u64,
    pub active_households: u64,
    pub extinct_households: u64,
    /// Under the historical fixed-founder baseline every household record was created at day
    /// zero, so its age is exactly the run day. Dynamic treatments retain this compatibility
    /// field as null and expose the complete distribution below instead.
    pub uniform_founder_household_age_days: Option<u64>,
    pub oldest_living_household_age_days: u64,
    pub living_household_age_distribution: Vec<HouseholdAgeBin>,
    pub largest_living_household_size: u32,
    pub living_household_size_distribution: Vec<HouseholdSizeBin>,
    pub maximum_living_generation_span: u32,
    pub multigenerational_households: u64,
    pub living_household_generation_span_distribution: Vec<HouseholdGenerationSpanBin>,
}

impl HouseholdObservabilityReport {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;
}

pub fn derive_household_observability(
    population: &Population,
    experiment: &ExperimentConfig,
    events: &EventLog,
    day: u64,
) -> Result<HouseholdObservabilityReport, HouseholdObservabilityError> {
    let household_count = population.household_count();
    let creation_days = household_creation_days(experiment, events, household_count, day)?;
    let mut living_sizes = vec![0_u32; household_count];
    let mut minimum_generation = vec![u32::MAX; household_count];
    let mut maximum_generation = vec![0_u32; household_count];
    let mut has_living_member = vec![false; household_count];
    let mut memo = vec![None; population.person_count()];
    let mut visiting = vec![false; population.person_count()];

    for person_index in 0..population.person_count() {
        let person = population
            .person(PersonId::new(
                u64::try_from(person_index)
                    .map_err(|_| HouseholdObservabilityError::PersonIdOverflow)?
                    .checked_add(1)
                    .ok_or(HouseholdObservabilityError::PersonIdOverflow)?,
            ))
            .ok_or(HouseholdObservabilityError::MissingPerson { person_index })?;
        if !person.is_alive() {
            continue;
        }
        let household_index = usize::try_from(person.household.0.checked_sub(1).ok_or(
            HouseholdObservabilityError::InvalidHousehold {
                household: person.household,
            },
        )?)
        .map_err(|_| HouseholdObservabilityError::InvalidHousehold {
            household: person.household,
        })?;
        if household_index >= household_count {
            return Err(HouseholdObservabilityError::InvalidHousehold {
                household: person.household,
            });
        }
        let generation = generation_depth(population, person.id, &mut memo, &mut visiting)?;
        living_sizes[household_index] = living_sizes[household_index].saturating_add(1);
        has_living_member[household_index] = true;
        minimum_generation[household_index] = minimum_generation[household_index].min(generation);
        maximum_generation[household_index] = maximum_generation[household_index].max(generation);
    }

    let mut age_bins = BTreeMap::<u64, u64>::new();
    let mut size_bins = BTreeMap::<u32, u64>::new();
    let mut span_bins = BTreeMap::<u32, u64>::new();
    let mut active_households = 0_u64;
    let mut oldest_age = 0_u64;
    let mut largest = 0_u32;
    let mut max_span = 0_u32;
    let mut multigenerational = 0_u64;
    for index in 0..household_count {
        if !has_living_member[index] {
            continue;
        }
        active_households = active_households.saturating_add(1);
        let age = day.checked_sub(creation_days[index]).ok_or(
            HouseholdObservabilityError::CreationDayAfterObservation {
                household: HouseholdId::new(index as u64 + 1),
                creation_day: creation_days[index],
                observation_day: day,
            },
        )?;
        oldest_age = oldest_age.max(age);
        *age_bins.entry(age).or_default() += 1;
        let size = living_sizes[index];
        largest = largest.max(size);
        *size_bins.entry(size).or_default() += 1;
        let span = maximum_generation[index]
            .saturating_sub(minimum_generation[index])
            .saturating_add(1);
        max_span = max_span.max(span);
        if span >= 2 {
            multigenerational = multigenerational.saturating_add(1);
        }
        *span_bins.entry(span).or_default() += 1;
    }

    let total_household_records = u64::try_from(household_count)
        .map_err(|_| HouseholdObservabilityError::HouseholdCountOverflow)?;
    Ok(HouseholdObservabilityReport {
        schema_version: HouseholdObservabilityReport::CURRENT_SCHEMA_VERSION,
        day,
        lifecycle_model_id: household_lifecycle_model_id(experiment.household_lifecycle.as_ref())
            .to_owned(),
        total_household_records,
        active_households,
        extinct_households: total_household_records.saturating_sub(active_households),
        uniform_founder_household_age_days: (experiment.household_lifecycle.is_none()
            && household_lifecycle_model_id(None) == FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID)
            .then_some(day),
        oldest_living_household_age_days: oldest_age,
        living_household_age_distribution: age_bins
            .into_iter()
            .map(|(age_days, household_count)| HouseholdAgeBin {
                age_days,
                household_count,
            })
            .collect(),
        largest_living_household_size: largest,
        living_household_size_distribution: size_bins
            .into_iter()
            .map(|(living_members, household_count)| HouseholdSizeBin {
                living_members,
                household_count,
            })
            .collect(),
        maximum_living_generation_span: max_span,
        multigenerational_households: multigenerational,
        living_household_generation_span_distribution: span_bins
            .into_iter()
            .map(
                |(generations, household_count)| HouseholdGenerationSpanBin {
                    generations,
                    household_count,
                },
            )
            .collect(),
    })
}

fn household_creation_days(
    experiment: &ExperimentConfig,
    events: &EventLog,
    household_count: usize,
    observation_day: u64,
) -> Result<Vec<u64>, HouseholdObservabilityError> {
    let mut creation_days = vec![0_u64; household_count];
    let mut first_dynamic_household = None::<u64>;
    let mut dynamic_count = 0_u64;
    for record in &events.events {
        let EventKind::HouseholdFission {
            event_schema_version,
            new_household,
            ..
        } = &record.event
        else {
            continue;
        };
        if experiment.household_lifecycle.is_none() {
            return Err(HouseholdObservabilityError::UnexpectedFissionEvent);
        }
        if record.provenance != EventProvenance::Authoritative {
            return Err(HouseholdObservabilityError::NonAuthoritativeFissionEvent);
        }
        if *event_schema_version != HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION {
            return Err(HouseholdObservabilityError::UnsupportedFissionEventSchema {
                found: *event_schema_version,
                supported: HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,
            });
        }
        if record.day > observation_day {
            return Err(HouseholdObservabilityError::CreationDayAfterObservation {
                household: *new_household,
                creation_day: record.day,
                observation_day,
            });
        }
        let raw = new_household.0;
        let first = *first_dynamic_household.get_or_insert(raw);
        let expected = first
            .checked_add(dynamic_count)
            .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
        if raw != expected {
            return Err(HouseholdObservabilityError::NonCanonicalFissionHousehold {
                expected: HouseholdId::new(expected),
                found: *new_household,
            });
        }
        let index = usize::try_from(raw.checked_sub(1).ok_or(
            HouseholdObservabilityError::InvalidHousehold {
                household: *new_household,
            },
        )?)
        .map_err(|_| HouseholdObservabilityError::InvalidHousehold {
            household: *new_household,
        })?;
        if index >= household_count {
            return Err(HouseholdObservabilityError::InvalidHousehold {
                household: *new_household,
            });
        }
        creation_days[index] = record.day;
        dynamic_count = dynamic_count.saturating_add(1);
    }
    if let Some(first) = first_dynamic_household {
        let founder_count = first
            .checked_sub(1)
            .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
        let observed_total = founder_count
            .checked_add(dynamic_count)
            .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
        if observed_total
            != u64::try_from(household_count)
                .map_err(|_| HouseholdObservabilityError::HouseholdCountOverflow)?
        {
            return Err(HouseholdObservabilityError::IncompleteFissionHistory {
                expected_households: household_count,
                history_households: observed_total,
            });
        }
    }
    Ok(creation_days)
}

fn generation_depth(
    population: &Population,
    person: PersonId,
    memo: &mut [Option<u32>],
    visiting: &mut [bool],
) -> Result<u32, HouseholdObservabilityError> {
    let index = usize::try_from(
        person
            .0
            .checked_sub(1)
            .ok_or(HouseholdObservabilityError::InvalidPerson { person })?,
    )
    .map_err(|_| HouseholdObservabilityError::InvalidPerson { person })?;
    if index >= memo.len() {
        return Err(HouseholdObservabilityError::InvalidPerson { person });
    }
    if let Some(value) = memo[index] {
        return Ok(value);
    }
    if visiting[index] {
        return Err(HouseholdObservabilityError::GenealogyCycle { person });
    }
    visiting[index] = true;
    let snapshot = population
        .person(person)
        .ok_or(HouseholdObservabilityError::InvalidPerson { person })?;
    let mut has_parent = false;
    let mut parent_depth = 0_u32;
    for parent in [snapshot.female_parent, snapshot.male_parent] {
        if parent == PersonId::INVALID {
            continue;
        }
        if population.person(parent).is_none() {
            return Err(HouseholdObservabilityError::InvalidParent { person, parent });
        }
        has_parent = true;
        parent_depth = parent_depth.max(generation_depth(population, parent, memo, visiting)?);
    }
    visiting[index] = false;
    let depth = if has_parent {
        parent_depth
            .checked_add(1)
            .ok_or(HouseholdObservabilityError::GenerationDepthOverflow)?
    } else {
        0
    };
    memo[index] = Some(depth);
    Ok(depth)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HouseholdObservabilityError {
    #[error("person index {person_index} has no persistent record")]
    MissingPerson { person_index: usize },
    #[error("person identity space overflowed while deriving household observability")]
    PersonIdOverflow,
    #[error("household count does not fit u64")]
    HouseholdCountOverflow,
    #[error("invalid household identity {household:?}")]
    InvalidHousehold { household: HouseholdId },
    #[error("fixed-founder experiment unexpectedly contains a household-fission event")]
    UnexpectedFissionEvent,
    #[error("household-fission event is not authoritative")]
    NonAuthoritativeFissionEvent,
    #[error(
        "household-fission event schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedFissionEventSchema { found: u32, supported: u32 },
    #[error("non-canonical household-fission identity: expected {expected:?}, found {found:?}")]
    NonCanonicalFissionHousehold {
        expected: HouseholdId,
        found: HouseholdId,
    },
    #[error(
        "household-fission history accounts for {history_households} households but terminal state has {expected_households}"
    )]
    IncompleteFissionHistory {
        expected_households: usize,
        history_households: u64,
    },
    #[error(
        "household {household:?} was created on day {creation_day}, after observation day {observation_day}"
    )]
    CreationDayAfterObservation {
        household: HouseholdId,
        creation_day: u64,
        observation_day: u64,
    },
    #[error("invalid person identity {person:?}")]
    InvalidPerson { person: PersonId },
    #[error("person {person:?} references invalid parent {parent:?}")]
    InvalidParent { person: PersonId, parent: PersonId },
    #[error("genealogy cycle encountered while deriving generation depth at {person:?}")]
    GenealogyCycle { person: PersonId },
    #[error("genealogical generation depth overflowed u32")]
    GenerationDepthOverflow,
}
