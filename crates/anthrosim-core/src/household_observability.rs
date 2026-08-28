use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID},
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
    /// zero, so its age is exactly the run day. Dynamic lifecycle variants deliberately return
    /// null because creation-day history is not persisted by this minimal sensitivity model.
    pub uniform_founder_household_age_days: Option<u64>,
    pub largest_living_household_size: u32,
    pub living_household_size_distribution: Vec<HouseholdSizeBin>,
    pub maximum_living_generation_span: u32,
    pub multigenerational_households: u64,
    pub living_household_generation_span_distribution: Vec<HouseholdGenerationSpanBin>,
}

impl HouseholdObservabilityReport {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

pub fn derive_household_observability(
    population: &Population,
    experiment: &ExperimentConfig,
    day: u64,
) -> Result<HouseholdObservabilityReport, HouseholdObservabilityError> {
    let household_count = population.household_count();
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

    let mut size_bins = BTreeMap::<u32, u64>::new();
    let mut span_bins = BTreeMap::<u32, u64>::new();
    let mut active_households = 0_u64;
    let mut largest = 0_u32;
    let mut max_span = 0_u32;
    let mut multigenerational = 0_u64;
    for index in 0..household_count {
        if !has_living_member[index] {
            continue;
        }
        active_households = active_households.saturating_add(1);
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
    #[error("invalid person identity {person:?}")]
    InvalidPerson { person: PersonId },
    #[error("person {person:?} references invalid parent {parent:?}")]
    InvalidParent { person: PersonId, parent: PersonId },
    #[error("genealogy cycle encountered while deriving generation depth at {person:?}")]
    GenealogyCycle { person: PersonId },
    #[error("genealogical generation depth overflowed u32")]
    GenerationDepthOverflow,
}
