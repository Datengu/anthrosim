use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    checkpoint::SimulationCheckpoint,
    config::{AgeProbabilityBand, DemographyConfig},
    demography::{annual_probability_for_age, draw_per_million, effective_birth_spacing_days},
    events::{DeathCause, EventKind, EventProvenance, EventRecord},
    ids::{CellId, HouseholdId, PersonId},
    manifest::StopReason,
    population::{Population, ReproductiveSex},
    provenance::MODEL_SEMANTICS_ID,
    rng::RngFactory,
    time::DAYS_PER_YEAR,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographicMortalityBandObservability {
    pub start_age_years: u32,
    pub end_age_years_exclusive: u32,
    pub configured_probability_per_million: u32,
    pub exposures: u64,
    pub deaths: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographicFertilityBandObservability {
    pub start_age_years: u32,
    pub end_age_years_exclusive: u32,
    pub configured_probability_per_million: u32,
    pub surviving_female_exposures: u64,
    pub age_schedule_eligible: u64,
    pub spacing_eligible: u64,
    pub local_male_eligible: u64,
    pub fertility_draws_attempted: u64,
    pub fertility_draw_successes: u64,
    pub successful_births: u64,
    pub stochastic_draw_failures: u64,
    pub record_limit_blocked_births: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterbirthIntervalObservability {
    pub interval_days: u64,
    pub occurrences: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedFertilityObservability {
    pub model_period_births: u32,
    pub females: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographyObservabilitySummary {
    pub initial_living_population: u64,
    pub final_living_population: u64,
    pub mortality_exposures: u64,
    pub demographic_deaths: u64,
    pub surviving_females_entering_fertility: u64,
    pub age_schedule_eligible: u64,
    pub spacing_eligible: u64,
    pub local_male_eligible: u64,
    pub fertility_draws_attempted: u64,
    pub fertility_draw_successes: u64,
    pub successful_births: u64,
    pub stochastic_draw_failures: u64,
    pub record_limit_blocked_births: u64,
    pub uncensored_completed_fertility_females: u64,
    pub censored_completed_fertility_females: u64,
}

/// Versioned M2 analysis derived from authoritative initial state, events and checkpoint state.
///
/// The derivation replays the v7 structural opportunity rules and independent fertility RNG stream.
/// It does not add causal state or one event per rejected opportunity to the simulation hot loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographyObservabilityReport {
    pub schema_version: u32,
    pub model_semantics_id: String,
    pub schedule_id: String,
    pub simulated_days: u64,
    pub annual_boundaries_observed: u64,
    pub requested_birth_spacing_days: u32,
    pub effective_birth_spacing_days: u64,
    pub fertility_probability_is_conditional_on_m2_survival: bool,
    pub parentage_uses_pre_same_day_m4_residence: bool,
    pub summary: DemographyObservabilitySummary,
    pub mortality_bands: Vec<DemographicMortalityBandObservability>,
    pub fertility_bands: Vec<DemographicFertilityBandObservability>,
    pub model_period_interbirth_intervals: Vec<InterbirthIntervalObservability>,
    pub declared_prerun_to_first_birth_intervals: Vec<InterbirthIntervalObservability>,
    pub completed_fertility_distribution: Vec<CompletedFertilityObservability>,
    pub fertility_stage_truncated_by_record_limit: bool,
}

impl DemographyObservabilityReport {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DemographyObservabilityError {
    #[error("demography observability supports model semantics {supported}, found {found}")]
    UnsupportedModelSemantics { found: String, supported: String },
    #[error("invalid demographic observability replay: {0}")]
    InvalidReplay(String),
}

#[derive(Debug, Clone)]
struct ReplayPerson {
    id: PersonId,
    birth_day: i64,
    death_day: Option<u64>,
    last_birth_day: Option<u64>,
    reproductive_sex: ReproductiveSex,
    location: CellId,
    household: HouseholdId,
    model_period_births: u32,
}

impl ReplayPerson {
    fn alive(&self) -> bool {
        self.death_day.is_none()
    }

    fn age_days_at(&self, day: u64) -> Option<u64> {
        let day = i64::try_from(day).ok()?;
        u64::try_from(day.checked_sub(self.birth_day)?).ok()
    }
}

#[derive(Debug, Clone)]
struct BirthRecord {
    person: PersonId,
    male_parent: PersonId,
    household: HouseholdId,
    cell: CellId,
    reproductive_sex: ReproductiveSex,
}

/// Rebuild the M2 opportunity funnel for an exact run artifact set.
pub fn derive_demography_observability(
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
) -> Result<DemographyObservabilityReport, DemographyObservabilityError> {
    if checkpoint.model_semantics_id != MODEL_SEMANTICS_ID {
        return Err(DemographyObservabilityError::UnsupportedModelSemantics {
            found: checkpoint.model_semantics_id.clone(),
            supported: MODEL_SEMANTICS_ID.to_owned(),
        });
    }
    let end_day = checkpoint.time.days();
    if !end_day.is_multiple_of(DAYS_PER_YEAR) {
        return Err(invalid(format!(
            "checkpoint day {end_day} is not an annual M2 boundary"
        )));
    }
    let expected_initial = checkpoint.experiment.population.initial_population;
    if initial_population.person_count() != expected_initial as usize {
        return Err(invalid(format!(
            "initial population has {} records, experiment declares {expected_initial}",
            initial_population.person_count()
        )));
    }
    validate_event_order(&checkpoint.events.events, end_day)?;

    let config = &checkpoint.experiment.demography;
    let mut people = replay_people_from_initial(initial_population)?;
    let mut mortality_bands = config
        .mortality_bands
        .iter()
        .map(mortality_band_row)
        .collect::<Vec<_>>();
    let mut fertility_bands = config
        .fertility_bands
        .iter()
        .map(fertility_band_row)
        .collect::<Vec<_>>();
    let mut summary = empty_summary(initial_population, &checkpoint.population);
    let mut model_intervals = BTreeMap::<u64, u64>::new();
    let mut declared_intervals = BTreeMap::<u64, u64>::new();
    let mut fertility_rng =
        RngFactory::new(checkpoint.experiment.seed).stream("demography/fertility");
    let effective_spacing = effective_birth_spacing_days(config);
    let annual_boundaries = end_day / DAYS_PER_YEAR;
    let mut event_cursor = 0_usize;
    let mut truncated_by_limit = false;

    for boundary_index in 1..=annual_boundaries {
        let day = boundary_index.saturating_mul(DAYS_PER_YEAR);
        while event_cursor < checkpoint.events.events.len()
            && checkpoint.events.events[event_cursor].day < day
        {
            apply_interboundary_event(&mut people, &checkpoint.events.events[event_cursor])?;
            event_cursor += 1;
        }

        let day_start = event_cursor;
        while event_cursor < checkpoint.events.events.len()
            && checkpoint.events.events[event_cursor].day == day
        {
            event_cursor += 1;
        }
        let day_events = &checkpoint.events.events[day_start..event_cursor];
        let same_day_origins = apply_pre_m2_boundary_events(&mut people, day_events, day)?;
        let interval_start_day = day - DAYS_PER_YEAR;

        count_mortality_exposures(
            &people,
            config,
            interval_start_day,
            &mut mortality_bands,
            &mut summary,
        )?;
        apply_demographic_deaths(
            &mut people,
            day_events,
            config,
            interval_start_day,
            &mut mortality_bands,
            &mut summary,
            day,
        )?;

        if people.iter().all(|person| !person.alive()) {
            continue;
        }

        let mut births = births_by_female(day_events, day)?;
        let records_at_boundary_start = people.len();
        let max_records = checkpoint.experiment.population.max_person_records;
        let mut stop_boundary = false;

        for female_index in 0..records_at_boundary_start {
            if !people[female_index].alive()
                || people[female_index].reproductive_sex != ReproductiveSex::Female
            {
                continue;
            }
            let female_id = people[female_index].id;
            let age_days = replay_age(&people[female_index], interval_start_day)?;
            let band_index = schedule_band_index(&config.fertility_bands, age_days)?;
            fertility_bands[band_index].surviving_female_exposures = fertility_bands[band_index]
                .surviving_female_exposures
                .saturating_add(1);
            summary.surviving_females_entering_fertility = summary
                .surviving_females_entering_fertility
                .saturating_add(1);

            let probability = annual_probability_for_age(&config.fertility_bands, age_days);
            if probability == 0 {
                reject_recorded_birth(&births, female_id, day, "zero fertility schedule")?;
                continue;
            }
            fertility_bands[band_index].age_schedule_eligible = fertility_bands[band_index]
                .age_schedule_eligible
                .saturating_add(1);
            summary.age_schedule_eligible = summary.age_schedule_eligible.saturating_add(1);

            if prior_birth_elapsed_days(
                &people[female_index],
                day,
                checkpoint.experiment.founder_population.as_ref(),
            )
            .is_some_and(|elapsed| elapsed < effective_spacing)
            {
                reject_recorded_birth(&births, female_id, day, "birth spacing")?;
                continue;
            }
            fertility_bands[band_index].spacing_eligible = fertility_bands[band_index]
                .spacing_eligible
                .saturating_add(1);
            summary.spacing_eligible = summary.spacing_eligible.saturating_add(1);

            let location = exposure_location(&people[female_index], &same_day_origins);
            let eligible = eligible_males(
                &people,
                location,
                &same_day_origins,
                interval_start_day,
                config,
            )?;
            if eligible.is_empty() {
                reject_recorded_birth(&births, female_id, day, "local male availability")?;
                continue;
            }
            fertility_bands[band_index].local_male_eligible = fertility_bands[band_index]
                .local_male_eligible
                .saturating_add(1);
            summary.local_male_eligible = summary.local_male_eligible.saturating_add(1);
            fertility_bands[band_index].fertility_draws_attempted = fertility_bands[band_index]
                .fertility_draws_attempted
                .saturating_add(1);
            summary.fertility_draws_attempted = summary.fertility_draws_attempted.saturating_add(1);

            let draw_success = draw_per_million(&mut fertility_rng, probability);
            let recorded_birth = births.remove(&female_id);
            if !draw_success {
                fertility_bands[band_index].stochastic_draw_failures = fertility_bands[band_index]
                    .stochastic_draw_failures
                    .saturating_add(1);
                summary.stochastic_draw_failures =
                    summary.stochastic_draw_failures.saturating_add(1);
                if recorded_birth.is_some() {
                    return Err(invalid(format!(
                        "fertility RNG rejected {female_id:?} at day {day}, but a birth exists"
                    )));
                }
                continue;
            }

            fertility_bands[band_index].fertility_draw_successes = fertility_bands[band_index]
                .fertility_draw_successes
                .saturating_add(1);
            summary.fertility_draw_successes = summary.fertility_draw_successes.saturating_add(1);

            if people.len() as u64 >= max_records {
                fertility_bands[band_index].record_limit_blocked_births = fertility_bands
                    [band_index]
                    .record_limit_blocked_births
                    .saturating_add(1);
                summary.record_limit_blocked_births =
                    summary.record_limit_blocked_births.saturating_add(1);
                if recorded_birth.is_some() {
                    return Err(invalid(format!(
                        "birth recorded for {female_id:?} at full record limit on day {day}"
                    )));
                }
                truncated_by_limit = true;
                stop_boundary = true;
                break;
            }

            let birth = recorded_birth.ok_or_else(|| {
                invalid(format!(
                    "fertility RNG expected birth for {female_id:?} at day {day}, but none exists"
                ))
            })?;
            if !eligible.contains(&birth.male_parent) {
                return Err(invalid(format!(
                    "birth for {female_id:?} at day {day} has ineligible male parent {:?}",
                    birth.male_parent
                )));
            }
            if birth.household != people[female_index].household
                || birth.cell != people[female_index].location
            {
                return Err(invalid(format!(
                    "birth for {female_id:?} at day {day} does not match current residence"
                )));
            }

            record_interbirth_interval(
                &people[female_index],
                checkpoint,
                day,
                &mut model_intervals,
                &mut declared_intervals,
            )?;
            people[female_index].last_birth_day = Some(day);
            people[female_index].model_period_births =
                people[female_index].model_period_births.saturating_add(1);
            apply_birth(&mut people, &birth, day)?;
            fertility_bands[band_index].successful_births = fertility_bands[band_index]
                .successful_births
                .saturating_add(1);
            summary.successful_births = summary.successful_births.saturating_add(1);

            if people.len() as u64 >= max_records {
                truncated_by_limit = true;
                stop_boundary = true;
                break;
            }
        }

        if let Some((&female_parent, _)) = births.iter().next() {
            return Err(invalid(format!(
                "unexplained birth for {female_parent:?} at day {day}"
            )));
        }
        if stop_boundary {
            if checkpoint.terminal_stop_reason != Some(StopReason::PersonRecordLimitReached)
                || day != end_day
            {
                return Err(invalid(format!(
                    "record-limit stop reconstructed at day {day}, checkpoint does not match"
                )));
            }
            break;
        }
    }

    if checkpoint.terminal_stop_reason == Some(StopReason::PersonRecordLimitReached)
        && !truncated_by_limit
    {
        return Err(invalid(
            "checkpoint reports person-record-limit stop but M2 replay does not".to_owned(),
        ));
    }

    reconcile_final_population(&people, &checkpoint.population)?;
    let (completed_fertility_distribution, uncensored, censored) =
        completed_fertility_distribution(&people, config, end_day)?;
    summary.uncensored_completed_fertility_females = uncensored;
    summary.censored_completed_fertility_females = censored;

    Ok(DemographyObservabilityReport {
        schema_version: DemographyObservabilityReport::CURRENT_SCHEMA_VERSION,
        model_semantics_id: checkpoint.model_semantics_id.clone(),
        schedule_id: config.schedule_id.clone(),
        simulated_days: end_day,
        annual_boundaries_observed: annual_boundaries,
        requested_birth_spacing_days: config.minimum_birth_spacing_days,
        effective_birth_spacing_days: effective_spacing,
        fertility_probability_is_conditional_on_m2_survival: true,
        parentage_uses_pre_same_day_m4_residence: true,
        summary,
        mortality_bands,
        fertility_bands,
        model_period_interbirth_intervals: distribution_rows(model_intervals),
        declared_prerun_to_first_birth_intervals: distribution_rows(declared_intervals),
        completed_fertility_distribution,
        fertility_stage_truncated_by_record_limit: truncated_by_limit,
    })
}

fn empty_summary(
    initial_population: &Population,
    final_population: &Population,
) -> DemographyObservabilitySummary {
    DemographyObservabilitySummary {
        initial_living_population: initial_population.living_count(),
        final_living_population: final_population.living_count(),
        mortality_exposures: 0,
        demographic_deaths: 0,
        surviving_females_entering_fertility: 0,
        age_schedule_eligible: 0,
        spacing_eligible: 0,
        local_male_eligible: 0,
        fertility_draws_attempted: 0,
        fertility_draw_successes: 0,
        successful_births: 0,
        stochastic_draw_failures: 0,
        record_limit_blocked_births: 0,
        uncensored_completed_fertility_females: 0,
        censored_completed_fertility_females: 0,
    }
}

fn validate_event_order(
    events: &[EventRecord],
    checkpoint_day: u64,
) -> Result<(), DemographyObservabilityError> {
    let mut previous_sequence = 0_u64;
    let mut previous_day = 0_u64;
    for event in events {
        if event.provenance != EventProvenance::Authoritative {
            continue;
        }
        if event.sequence <= previous_sequence || event.day < previous_day {
            return Err(invalid(format!(
                "event order is invalid at sequence {}",
                event.sequence
            )));
        }
        if event.day > checkpoint_day {
            return Err(invalid(format!(
                "event day {} lies beyond checkpoint day {checkpoint_day}",
                event.day
            )));
        }
        previous_sequence = event.sequence;
        previous_day = event.day;
    }
    Ok(())
}

fn replay_people_from_initial(
    population: &Population,
) -> Result<Vec<ReplayPerson>, DemographyObservabilityError> {
    let mut people = Vec::with_capacity(population.person_count());
    for index in 0..population.person_count() {
        let id = PersonId::new(index as u64 + 1);
        let person = population
            .person(id)
            .ok_or_else(|| invalid(format!("initial population is missing {id:?}")))?;
        people.push(ReplayPerson {
            id,
            birth_day: person.birth_day,
            death_day: person.death_day,
            last_birth_day: person.last_birth_day,
            reproductive_sex: person.reproductive_sex,
            location: person.location,
            household: person.household,
            model_period_births: 0,
        });
    }
    Ok(people)
}

fn apply_interboundary_event(
    people: &mut [ReplayPerson],
    record: &EventRecord,
) -> Result<(), DemographyObservabilityError> {
    match &record.event {
        EventKind::HouseholdMigration {
            household,
            destination,
            ..
        } => {
            apply_household_migration(people, *household, *destination);
            Ok(())
        }
        EventKind::Death { person, .. } => mark_replay_death(people, *person, record.day),
        EventKind::Birth { female_parent, .. } => Err(invalid(format!(
            "birth for {female_parent:?} occurred off an annual M2 boundary at day {}",
            record.day
        ))),
        _ => Ok(()),
    }
}

fn apply_pre_m2_boundary_events(
    people: &mut [ReplayPerson],
    records: &[EventRecord],
    day: u64,
) -> Result<BTreeMap<HouseholdId, CellId>, DemographyObservabilityError> {
    let mut origins = BTreeMap::new();
    for record in records {
        match &record.event {
            EventKind::HouseholdMigration {
                household,
                origin,
                destination,
                ..
            } => {
                origins.entry(*household).or_insert(*origin);
                apply_household_migration(people, *household, *destination);
            }
            EventKind::Death {
                person,
                cause: DeathCause::ResourceScarcity,
                ..
            } => mark_replay_death(people, *person, day)?,
            _ => {}
        }
    }
    Ok(origins)
}

fn apply_household_migration(
    people: &mut [ReplayPerson],
    household: HouseholdId,
    destination: CellId,
) {
    for person in people
        .iter_mut()
        .filter(|person| person.alive() && person.household == household)
    {
        person.location = destination;
    }
}

fn mark_replay_death(
    people: &mut [ReplayPerson],
    person: PersonId,
    day: u64,
) -> Result<(), DemographyObservabilityError> {
    let index = replay_index(person, people.len())
        .ok_or_else(|| invalid(format!("death references unknown {person:?} at day {day}")))?;
    if !people[index].alive() {
        return Err(invalid(format!(
            "death references already-dead {person:?} at day {day}"
        )));
    }
    people[index].death_day = Some(day);
    Ok(())
}

fn count_mortality_exposures(
    people: &[ReplayPerson],
    config: &DemographyConfig,
    interval_start_day: u64,
    bands: &mut [DemographicMortalityBandObservability],
    summary: &mut DemographyObservabilitySummary,
) -> Result<(), DemographyObservabilityError> {
    for person in people.iter().filter(|person| person.alive()) {
        let age_days = replay_age(person, interval_start_day)?;
        let index = schedule_band_index(&config.mortality_bands, age_days)?;
        bands[index].exposures = bands[index].exposures.saturating_add(1);
        summary.mortality_exposures = summary.mortality_exposures.saturating_add(1);
    }
    Ok(())
}

fn apply_demographic_deaths(
    people: &mut [ReplayPerson],
    records: &[EventRecord],
    config: &DemographyConfig,
    interval_start_day: u64,
    bands: &mut [DemographicMortalityBandObservability],
    summary: &mut DemographyObservabilitySummary,
    day: u64,
) -> Result<(), DemographyObservabilityError> {
    for record in records {
        let EventKind::Death {
            person,
            cause: DeathCause::DemographicMortality,
            probability_per_million,
            ..
        } = &record.event
        else {
            continue;
        };
        let index = replay_index(*person, people.len()).ok_or_else(|| {
            invalid(format!(
                "M2 death references unknown {person:?} at day {day}"
            ))
        })?;
        if !people[index].alive() {
            return Err(invalid(format!(
                "M2 death references already-dead {person:?} at day {day}"
            )));
        }
        let age_days = replay_age(&people[index], interval_start_day)?;
        let band = schedule_band_index(&config.mortality_bands, age_days)?;
        let expected = config.mortality_bands[band].annual_probability_per_million;
        if *probability_per_million != expected {
            return Err(invalid(format!(
                "M2 death probability for {person:?} at day {day} is {probability_per_million}, expected {expected}"
            )));
        }
        people[index].death_day = Some(day);
        bands[band].deaths = bands[band].deaths.saturating_add(1);
        summary.demographic_deaths = summary.demographic_deaths.saturating_add(1);
    }
    Ok(())
}

fn births_by_female(
    records: &[EventRecord],
    day: u64,
) -> Result<BTreeMap<PersonId, BirthRecord>, DemographyObservabilityError> {
    let mut births = BTreeMap::new();
    for record in records {
        let EventKind::Birth {
            person,
            female_parent,
            male_parent,
            household,
            cell,
            reproductive_sex,
        } = &record.event
        else {
            continue;
        };
        let birth = BirthRecord {
            person: *person,
            male_parent: *male_parent,
            household: *household,
            cell: *cell,
            reproductive_sex: *reproductive_sex,
        };
        if births.insert(*female_parent, birth).is_some() {
            return Err(invalid(format!(
                "multiple births for {female_parent:?} at day {day}"
            )));
        }
    }
    Ok(births)
}

fn reject_recorded_birth(
    births: &BTreeMap<PersonId, BirthRecord>,
    female: PersonId,
    day: u64,
    reason: &str,
) -> Result<(), DemographyObservabilityError> {
    if births.contains_key(&female) {
        return Err(invalid(format!(
            "birth exists for {female:?} at day {day} despite {reason}"
        )));
    }
    Ok(())
}

fn apply_birth(
    people: &mut Vec<ReplayPerson>,
    birth: &BirthRecord,
    day: u64,
) -> Result<(), DemographyObservabilityError> {
    let expected = PersonId::new(people.len() as u64 + 1);
    if birth.person != expected {
        return Err(invalid(format!(
            "birth ID {:?} is not next canonical ID {expected:?} at day {day}",
            birth.person
        )));
    }
    let birth_day =
        i64::try_from(day).map_err(|_| invalid("birth day does not fit i64".to_owned()))?;
    people.push(ReplayPerson {
        id: birth.person,
        birth_day,
        death_day: None,
        last_birth_day: None,
        reproductive_sex: birth.reproductive_sex,
        location: birth.cell,
        household: birth.household,
        model_period_births: 0,
    });
    Ok(())
}

fn prior_birth_elapsed_days(
    female: &ReplayPerson,
    day: u64,
    founders: Option<&crate::founder_initialization::FounderPopulationDefinition>,
) -> Option<u64> {
    if let Some(last_birth_day) = female.last_birth_day {
        return day.checked_sub(last_birth_day);
    }
    let previous = founders?.last_birth_day(female.id)?;
    let day = i64::try_from(day).ok()?;
    u64::try_from(day.checked_sub(previous)?).ok()
}

fn exposure_location(
    person: &ReplayPerson,
    same_day_origins: &BTreeMap<HouseholdId, CellId>,
) -> CellId {
    same_day_origins
        .get(&person.household)
        .copied()
        .unwrap_or(person.location)
}

fn eligible_males(
    people: &[ReplayPerson],
    location: CellId,
    same_day_origins: &BTreeMap<HouseholdId, CellId>,
    interval_start_day: u64,
    config: &DemographyConfig,
) -> Result<BTreeSet<PersonId>, DemographyObservabilityError> {
    let mut eligible = BTreeSet::new();
    for person in people {
        if !person.alive() || person.reproductive_sex != ReproductiveSex::Male {
            continue;
        }
        if exposure_location(person, same_day_origins) != location {
            continue;
        }
        let age_years = replay_age(person, interval_start_day)? / DAYS_PER_YEAR;
        if age_years < u64::from(config.male_parent_min_age_years)
            || age_years >= u64::from(config.male_parent_max_age_years_exclusive)
        {
            continue;
        }
        eligible.insert(person.id);
    }
    Ok(eligible)
}

fn record_interbirth_interval(
    female: &ReplayPerson,
    checkpoint: &SimulationCheckpoint,
    day: u64,
    model_intervals: &mut BTreeMap<u64, u64>,
    declared_intervals: &mut BTreeMap<u64, u64>,
) -> Result<(), DemographyObservabilityError> {
    if let Some(previous) = female.last_birth_day {
        let interval = day
            .checked_sub(previous)
            .ok_or_else(|| invalid("model interbirth interval underflow".to_owned()))?;
        increment_distribution(model_intervals, interval);
    } else if let Some(previous) = checkpoint
        .experiment
        .founder_population
        .as_ref()
        .and_then(|definition| definition.last_birth_day(female.id))
    {
        let day = i64::try_from(day)
            .map_err(|_| invalid("declared interbirth day does not fit i64".to_owned()))?;
        let interval = u64::try_from(
            day.checked_sub(previous)
                .ok_or_else(|| invalid("declared interbirth interval underflow".to_owned()))?,
        )
        .map_err(|_| invalid("declared interbirth interval is negative".to_owned()))?;
        increment_distribution(declared_intervals, interval);
    }
    Ok(())
}

fn completed_fertility_distribution(
    people: &[ReplayPerson],
    config: &DemographyConfig,
    end_day: u64,
) -> Result<(Vec<CompletedFertilityObservability>, u64, u64), DemographyObservabilityError> {
    let reproductive_end_year = config
        .fertility_bands
        .iter()
        .filter(|band| band.annual_probability_per_million > 0)
        .map(|band| band.end_age_years_exclusive)
        .max();
    let mut distribution = BTreeMap::<u32, u64>::new();
    let mut uncensored = 0_u64;
    let mut censored = 0_u64;

    for person in people
        .iter()
        .filter(|person| person.reproductive_sex == ReproductiveSex::Female)
    {
        let complete = reproductive_window_complete(person, reproductive_end_year, end_day)?;
        if complete {
            uncensored = uncensored.saturating_add(1);
            let count = distribution.entry(person.model_period_births).or_default();
            *count = count.saturating_add(1);
        } else {
            censored = censored.saturating_add(1);
        }
    }

    let rows = distribution
        .into_iter()
        .map(
            |(model_period_births, females)| CompletedFertilityObservability {
                model_period_births,
                females,
            },
        )
        .collect();
    Ok((rows, uncensored, censored))
}

fn reproductive_window_complete(
    person: &ReplayPerson,
    reproductive_end_year: Option<u32>,
    end_day: u64,
) -> Result<bool, DemographyObservabilityError> {
    if person.birth_day < 0 {
        return Ok(false);
    }
    let Some(end_year) = reproductive_end_year else {
        return Ok(true);
    };
    if end_year == u32::MAX {
        return Ok(false);
    }
    let end_age_days = i64::from(end_year)
        .checked_mul(i64::try_from(DAYS_PER_YEAR).expect("DAYS_PER_YEAR fits i64"))
        .ok_or_else(|| invalid("reproductive age overflow".to_owned()))?;
    let reproductive_end_day = person
        .birth_day
        .checked_add(end_age_days)
        .ok_or_else(|| invalid("reproductive end-day overflow".to_owned()))?;
    let observation_end = i64::try_from(person.death_day.unwrap_or(end_day))
        .map_err(|_| invalid("observation end does not fit i64".to_owned()))?;
    Ok(observation_end >= reproductive_end_day)
}

fn reconcile_final_population(
    replay: &[ReplayPerson],
    final_population: &Population,
) -> Result<(), DemographyObservabilityError> {
    if replay.len() != final_population.person_count() {
        return Err(invalid(format!(
            "replayed {} people, final population contains {}",
            replay.len(),
            final_population.person_count()
        )));
    }
    for person in replay {
        let final_person = final_population
            .person(person.id)
            .ok_or_else(|| invalid(format!("final population is missing {:?}", person.id)))?;
        let checks = [
            (final_person.birth_day == person.birth_day, "birthDay"),
            (final_person.death_day == person.death_day, "deathDay"),
            (
                final_person.last_birth_day == person.last_birth_day,
                "lastBirthDay",
            ),
            (
                final_person.reproductive_sex == person.reproductive_sex,
                "reproductiveSex",
            ),
            (final_person.location == person.location, "location"),
            (final_person.household == person.household, "household"),
        ];
        if let Some((_, field)) = checks.into_iter().find(|(matches, _)| !matches) {
            return Err(invalid(format!(
                "replayed {field} does not match final population for {:?}",
                person.id
            )));
        }
    }
    Ok(())
}

fn replay_age(person: &ReplayPerson, day: u64) -> Result<u64, DemographyObservabilityError> {
    person.age_days_at(day).ok_or_else(|| {
        invalid(format!(
            "age is not representable for {:?} at day {day}",
            person.id
        ))
    })
}

fn schedule_band_index(
    bands: &[AgeProbabilityBand],
    age_days: u64,
) -> Result<usize, DemographyObservabilityError> {
    let age_years = u32::try_from(age_days / DAYS_PER_YEAR).unwrap_or(u32::MAX - 1);
    bands
        .iter()
        .position(|band| {
            age_years >= band.start_age_years && age_years < band.end_age_years_exclusive
        })
        .ok_or_else(|| invalid(format!("age {age_years} is outside configured schedule")))
}

fn mortality_band_row(band: &AgeProbabilityBand) -> DemographicMortalityBandObservability {
    DemographicMortalityBandObservability {
        start_age_years: band.start_age_years,
        end_age_years_exclusive: band.end_age_years_exclusive,
        configured_probability_per_million: band.annual_probability_per_million,
        exposures: 0,
        deaths: 0,
    }
}

fn fertility_band_row(band: &AgeProbabilityBand) -> DemographicFertilityBandObservability {
    DemographicFertilityBandObservability {
        start_age_years: band.start_age_years,
        end_age_years_exclusive: band.end_age_years_exclusive,
        configured_probability_per_million: band.annual_probability_per_million,
        surviving_female_exposures: 0,
        age_schedule_eligible: 0,
        spacing_eligible: 0,
        local_male_eligible: 0,
        fertility_draws_attempted: 0,
        fertility_draw_successes: 0,
        successful_births: 0,
        stochastic_draw_failures: 0,
        record_limit_blocked_births: 0,
    }
}

fn replay_index(person: PersonId, len: usize) -> Option<usize> {
    let index = usize::try_from(person.0.checked_sub(1)?).ok()?;
    (index < len).then_some(index)
}

fn increment_distribution(distribution: &mut BTreeMap<u64, u64>, value: u64) {
    let count = distribution.entry(value).or_default();
    *count = count.saturating_add(1);
}

fn distribution_rows(distribution: BTreeMap<u64, u64>) -> Vec<InterbirthIntervalObservability> {
    distribution
        .into_iter()
        .map(
            |(interval_days, occurrences)| InterbirthIntervalObservability {
                interval_days,
                occurrences,
            },
        )
        .collect()
}

fn invalid(reason: String) -> DemographyObservabilityError {
    DemographyObservabilityError::InvalidReplay(reason)
}
