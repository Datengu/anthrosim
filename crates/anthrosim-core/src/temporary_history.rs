use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    EventKind, EventProvenance, HouseholdPresence, Population, SimulationCheckpoint,
    TemporaryJourneyIneligibility, TemporaryMobilityProgram, TemporaryTravelResolution,
    TemporaryTriggerTiming, World,
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    rng::RngFactory,
};

const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{0}")]
pub struct TemporaryMobilityHistoryError(String);

fn invalid(message: impl Into<String>) -> TemporaryMobilityHistoryError {
    TemporaryMobilityHistoryError(message.into())
}

#[derive(Debug, Clone)]
struct HouseholdHistory {
    residence: CellId,
    living: u64,
    presence: HouseholdPresence,
    active_journey: Option<TemporaryJourneyId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JourneyStage {
    Outbound,
    Visiting,
    Returning,
    Completed,
    TerminatedNoLivingMembers,
}

#[derive(Debug, Clone)]
struct JourneyHistory {
    household: HouseholdId,
    trigger_index: u32,
    trigger_day: u64,
    region_id: String,
    region_identity: String,
    residence: CellId,
    destination: CellId,
    travel_model_identity: Option<String>,
    accumulated_travel_cost_units: Option<u64>,
    people_at_departure: u32,
    departure_day: u64,
    arrival_day: u64,
    return_departure_day: u64,
    completion_day: u64,
    outbound_travel_days: u32,
    return_travel_days: u32,
    stage: JourneyStage,
}

/// Validate the authoritative M9 event history independently of any derived report artifact.
///
/// The validator reconstructs founder household state from immutable experiment identity, replays
/// births, deaths, permanent migrations and temporary transitions, validates each temporary event
/// against the configured/resolved program, and reconciles the replay with terminal temporary
/// state. This makes the event history part of ordinary scientific-integrity validation rather
/// than relying on `temporary-observability.json` being present.
pub fn validate_temporary_mobility_history(
    world: &World,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), TemporaryMobilityHistoryError> {
    let has_temporary_events = checkpoint.events.events.iter().any(|record| {
        matches!(
            record.event,
            EventKind::TemporaryJourneyNotStarted { .. }
                | EventKind::TemporaryJourneyDeparted { .. }
                | EventKind::TemporaryJourneyArrived { .. }
                | EventKind::TemporaryReturnDeparted { .. }
                | EventKind::TemporaryJourneyCompleted { .. }
        )
    });

    let Some(program) = checkpoint.temporary_mobility.program() else {
        if has_temporary_events {
            return Err(invalid(
                "temporary events exist even though the checkpoint has no temporary-mobility program",
            ));
        }
        return Ok(());
    };
    program
        .validate(world)
        .map_err(|error| invalid(format!("temporary-mobility program is invalid: {error}")))?;
    if let Some(config) = checkpoint.experiment.temporary_mobility.as_ref() {
        let expected = config.derive_program(world).map_err(|error| {
            invalid(format!(
                "temporary-mobility experiment definition cannot derive its program: {error}"
            ))
        })?;
        if &expected != program {
            return Err(invalid(
                "temporary-mobility checkpoint program does not match the experiment definition",
            ));
        }
    }

    let initial_population = Population::initialize(
        checkpoint.experiment.population,
        world,
        RngFactory::new(checkpoint.experiment.seed),
    )
    .map_err(|error| invalid(format!("could not reconstruct founder population: {error}")))?;
    if initial_population.household_count() != checkpoint.population.household_count() {
        return Err(invalid(
            "founder and terminal population household counts differ",
        ));
    }

    let mut households = Vec::with_capacity(initial_population.household_count());
    for raw in 1..=initial_population.household_count() as u64 {
        let household = HouseholdId::new(raw);
        let residence = initial_population
            .household_location(household)
            .ok_or_else(|| invalid(format!("missing founder household {raw}")))?;
        households.push(HouseholdHistory {
            residence,
            living: 0,
            presence: HouseholdPresence::AtResidence,
            active_journey: None,
        });
    }
    for raw in 1..=initial_population.person_count() as u64 {
        let person = initial_population
            .person(crate::ids::PersonId::new(raw))
            .ok_or_else(|| invalid(format!("missing founder person {raw}")))?;
        if !person.is_alive() {
            return Err(invalid("founder population contains a dead person"));
        }
        let index = household_index(person.household, households.len())?;
        households[index].living = households[index]
            .living
            .checked_add(1)
            .ok_or_else(|| invalid("founder household living count overflow"))?;
    }

    let mut outcomes = BTreeSet::<(u32, u64)>::new();
    let mut journeys = BTreeMap::<u64, JourneyHistory>::new();
    let mut next_expected_journey_id = 1_u64;
    let mut previous_day = 0_u64;

    for (index, record) in checkpoint.events.events.iter().enumerate() {
        let expected_sequence = u64::try_from(index)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if record.sequence != expected_sequence
            || record.provenance != EventProvenance::Authoritative
            || record.day < previous_day
            || record.day > checkpoint.time.days()
        {
            return Err(invalid(
                "event sequence, provenance or chronological ordering is invalid during temporary-history replay",
            ));
        }
        previous_day = record.day;

        match &record.event {
            EventKind::Birth { household, .. } => {
                let index = household_index(*household, households.len())?;
                households[index].living = households[index]
                    .living
                    .checked_add(1)
                    .ok_or_else(|| invalid("household living count overflow after birth"))?;
            }
            EventKind::Death { household, .. } => {
                let index = household_index(*household, households.len())?;
                households[index].living = households[index]
                    .living
                    .checked_sub(1)
                    .ok_or_else(|| invalid("death would make household living count negative"))?;
                if households[index].living == 0 && !households[index].presence.is_at_residence() {
                    let journey = households[index]
                        .active_journey
                        .ok_or_else(|| invalid("away extinct household has no active journey"))?;
                    let history = journeys
                        .get_mut(&journey.0)
                        .ok_or_else(|| invalid("away extinct household references an unknown journey"))?;
                    history.stage = JourneyStage::TerminatedNoLivingMembers;
                    households[index].presence = HouseholdPresence::AtResidence;
                    households[index].active_journey = None;
                }
            }
            EventKind::HouseholdMigration {
                household,
                origin,
                destination,
                people_moved,
                ..
            } => {
                let index = household_index(*household, households.len())?;
                if !households[index].presence.is_at_residence()
                    || households[index].active_journey.is_some()
                {
                    return Err(invalid(format!(
                        "permanent migration occurred while household {} was temporarily away",
                        household.0
                    )));
                }
                if households[index].residence != *origin
                    || u64::from(*people_moved) != households[index].living
                    || world.cell(*destination).is_none()
                {
                    return Err(invalid(
                        "permanent migration does not reconcile with temporary-history household state",
                    ));
                }
                households[index].residence = *destination;
            }
            EventKind::TemporaryJourneyNotStarted {
                event_schema_version,
                household,
                region_id,
                region_identity,
                trigger_index,
                trigger_day,
                reason,
            } => {
                validate_event_schema(*event_schema_version)?;
                validate_region(program, region_id, region_identity)?;
                validate_trigger(program, *trigger_index, *trigger_day)?;
                record_outcome(&mut outcomes, *trigger_index, *household)?;
                validate_skip(
                    program,
                    &households,
                    *household,
                    *trigger_day,
                    record.day,
                    *reason,
                )?;
            }
            EventKind::TemporaryJourneyDeparted {
                event_schema_version,
                household,
                journey,
                region_id,
                region_identity,
                residence,
                destination,
                travel_model_identity,
                accumulated_travel_cost_units,
                people_affected,
                trigger_index,
                trigger_day,
                departure_day,
                arrival_day,
                return_departure_day,
                completion_day,
                outbound_travel_days,
                return_travel_days,
            } => {
                validate_event_schema(*event_schema_version)?;
                validate_region(program, region_id, region_identity)?;
                validate_trigger(program, *trigger_index, *trigger_day)?;
                record_outcome(&mut outcomes, *trigger_index, *household)?;
                let household_index = household_index(*household, households.len())?;
                let state = &households[household_index];
                if !state.presence.is_at_residence() || state.active_journey.is_some() {
                    return Err(invalid("temporary departure started while household was already away"));
                }
                if state.living == 0
                    || state.residence != *residence
                    || program.region.contains(*residence)
                    || u64::from(*people_affected) != state.living
                {
                    return Err(invalid(
                        "temporary departure does not reconcile with household eligibility/state",
                    ));
                }
                if journey.0 != next_expected_journey_id {
                    return Err(invalid(format!(
                        "temporary journey id {} is not the next canonical id {}",
                        journey.0, next_expected_journey_id
                    )));
                }
                next_expected_journey_id = next_expected_journey_id
                    .checked_add(1)
                    .ok_or_else(|| invalid("temporary journey id sequence overflow"))?;
                if record.day != *departure_day
                    || *arrival_day
                        != departure_day
                            .checked_add(u64::from(*outbound_travel_days))
                            .ok_or_else(|| invalid("temporary journey arrival day overflow"))?
                    || *return_departure_day
                        != arrival_day
                            .checked_add(u64::from(program.schedule.stay_duration_days))
                            .ok_or_else(|| invalid("temporary journey return day overflow"))?
                    || *completion_day
                        != return_departure_day
                            .checked_add(u64::from(*return_travel_days))
                            .ok_or_else(|| invalid("temporary journey completion day overflow"))?
                {
                    return Err(invalid("temporary departure timing is inconsistent"));
                }
                match program.schedule.trigger_timing {
                    TemporaryTriggerTiming::DepartureDay if *departure_day != *trigger_day => {
                        return Err(invalid(
                            "departure-timed temporary trigger did not depart on its trigger day",
                        ));
                    }
                    TemporaryTriggerTiming::TargetArrivalDay if *arrival_day != *trigger_day => {
                        return Err(invalid(
                            "target-arrival temporary trigger did not arrive on its trigger day",
                        ));
                    }
                    _ => {}
                }
                validate_departure_travel(
                    program,
                    *residence,
                    *destination,
                    *outbound_travel_days,
                    *return_travel_days,
                    travel_model_identity.as_deref(),
                    *accumulated_travel_cost_units,
                )?;
                if journeys.contains_key(&journey.0) {
                    return Err(invalid(format!("duplicate temporary journey id {}", journey.0)));
                }
                journeys.insert(
                    journey.0,
                    JourneyHistory {
                        household: *household,
                        trigger_index: *trigger_index,
                        trigger_day: *trigger_day,
                        region_id: region_id.clone(),
                        region_identity: region_identity.clone(),
                        residence: *residence,
                        destination: *destination,
                        travel_model_identity: travel_model_identity.clone(),
                        accumulated_travel_cost_units: *accumulated_travel_cost_units,
                        people_at_departure: *people_affected,
                        departure_day: *departure_day,
                        arrival_day: *arrival_day,
                        return_departure_day: *return_departure_day,
                        completion_day: *completion_day,
                        outbound_travel_days: *outbound_travel_days,
                        return_travel_days: *return_travel_days,
                        stage: JourneyStage::Outbound,
                    },
                );
                households[household_index].presence = HouseholdPresence::OutboundTransit {
                    journey: *journey,
                    destination: *destination,
                };
                households[household_index].active_journey = Some(*journey);
            }
            EventKind::TemporaryJourneyArrived {
                event_schema_version,
                household,
                journey,
                region_id,
                region_identity,
                destination,
                people_affected,
            } => {
                validate_event_schema(*event_schema_version)?;
                validate_region(program, region_id, region_identity)?;
                let household_index = household_index(*household, households.len())?;
                require_active(
                    &households[household_index],
                    *journey,
                    JourneyStage::Outbound,
                    &journeys,
                )?;
                let history = journeys
                    .get_mut(&journey.0)
                    .ok_or_else(|| invalid("arrival references an unknown temporary journey"))?;
                if history.household != *household
                    || history.region_id != *region_id
                    || history.region_identity != *region_identity
                    || history.destination != *destination
                    || history.arrival_day != record.day
                    || u64::from(*people_affected) != households[household_index].living
                {
                    return Err(invalid("temporary arrival does not reconcile with departure history"));
                }
                history.stage = JourneyStage::Visiting;
                households[household_index].presence = HouseholdPresence::Visiting {
                    journey: *journey,
                    destination: *destination,
                };
            }
            EventKind::TemporaryReturnDeparted {
                event_schema_version,
                household,
                journey,
                region_id,
                region_identity,
                destination,
                residence,
                people_affected,
            } => {
                validate_event_schema(*event_schema_version)?;
                validate_region(program, region_id, region_identity)?;
                let household_index = household_index(*household, households.len())?;
                require_active(
                    &households[household_index],
                    *journey,
                    JourneyStage::Visiting,
                    &journeys,
                )?;
                let history = journeys
                    .get_mut(&journey.0)
                    .ok_or_else(|| invalid("return departure references an unknown temporary journey"))?;
                if history.household != *household
                    || history.region_id != *region_id
                    || history.region_identity != *region_identity
                    || history.destination != *destination
                    || history.residence != *residence
                    || history.return_departure_day != record.day
                    || u64::from(*people_affected) != households[household_index].living
                {
                    return Err(invalid(
                        "temporary return departure does not reconcile with journey history",
                    ));
                }
                history.stage = JourneyStage::Returning;
                households[household_index].presence = HouseholdPresence::ReturnTransit {
                    journey: *journey,
                    destination: *destination,
                };
            }
            EventKind::TemporaryJourneyCompleted {
                event_schema_version,
                household,
                journey,
                region_id,
                region_identity,
                residence,
                people_affected,
            } => {
                validate_event_schema(*event_schema_version)?;
                validate_region(program, region_id, region_identity)?;
                let household_index = household_index(*household, households.len())?;
                require_active(
                    &households[household_index],
                    *journey,
                    JourneyStage::Returning,
                    &journeys,
                )?;
                let history = journeys
                    .get_mut(&journey.0)
                    .ok_or_else(|| invalid("completion references an unknown temporary journey"))?;
                if history.household != *household
                    || history.region_id != *region_id
                    || history.region_identity != *region_identity
                    || history.residence != *residence
                    || history.completion_day != record.day
                    || u64::from(*people_affected) != households[household_index].living
                {
                    return Err(invalid(
                        "temporary completion does not reconcile with journey history",
                    ));
                }
                history.stage = JourneyStage::Completed;
                households[household_index].presence = HouseholdPresence::AtResidence;
                households[household_index].active_journey = None;
            }
        }
    }

    require_completed_trigger_outcomes(program, checkpoint.time.days(), households.len(), &outcomes)?;
    reconcile_terminal(checkpoint, &households, &journeys)?;
    Ok(())
}

fn validate_event_schema(schema: u32) -> Result<(), TemporaryMobilityHistoryError> {
    if schema != TEMPORARY_EVENT_SCHEMA_VERSION {
        return Err(invalid(format!(
            "temporary event schema {schema} is unsupported; expected {TEMPORARY_EVENT_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

fn validate_region(
    program: &TemporaryMobilityProgram,
    region_id: &str,
    region_identity: &str,
) -> Result<(), TemporaryMobilityHistoryError> {
    if region_id != program.region.region_id || region_identity != program.region.identity() {
        return Err(invalid(
            "temporary event focal-region identity does not match the configured program",
        ));
    }
    Ok(())
}

fn validate_trigger(
    program: &TemporaryMobilityProgram,
    trigger_index: u32,
    trigger_day: u64,
) -> Result<(), TemporaryMobilityHistoryError> {
    let index = usize::try_from(trigger_index)
        .map_err(|_| invalid("temporary trigger index does not fit usize"))?;
    if program.schedule.trigger_days.get(index).copied() != Some(trigger_day) {
        return Err(invalid(
            "temporary event trigger identity does not match the configured schedule",
        ));
    }
    Ok(())
}

fn record_outcome(
    outcomes: &mut BTreeSet<(u32, u64)>,
    trigger_index: u32,
    household: HouseholdId,
) -> Result<(), TemporaryMobilityHistoryError> {
    if !outcomes.insert((trigger_index, household.0)) {
        return Err(invalid(format!(
            "duplicate temporary trigger outcome for trigger {trigger_index}, household {}",
            household.0
        )));
    }
    Ok(())
}

fn validate_skip(
    program: &TemporaryMobilityProgram,
    households: &[HouseholdHistory],
    household: HouseholdId,
    trigger_day: u64,
    event_day: u64,
    reason: TemporaryJourneyIneligibility,
) -> Result<(), TemporaryMobilityHistoryError> {
    let index = household_index(household, households.len())?;
    let state = &households[index];
    let first_reason = if state.living == 0 {
        Some(TemporaryJourneyIneligibility::NoLivingMembers)
    } else if !state.presence.is_at_residence() || state.active_journey.is_some() {
        Some(TemporaryJourneyIneligibility::ActiveJourney)
    } else if program.region.contains(state.residence) {
        Some(TemporaryJourneyIneligibility::ResidenceInRegion)
    } else {
        None
    };
    if let Some(expected) = first_reason {
        if reason != expected {
            return Err(invalid(format!(
                "temporary skip reason {reason:?} does not match household state {expected:?}"
            )));
        }
        if matches!(expected, TemporaryJourneyIneligibility::NoLivingMembers | TemporaryJourneyIneligibility::ResidenceInRegion)
            && event_day != trigger_day
        {
            return Err(invalid(
                "residence/no-living temporary skip did not occur on its trigger day",
            ));
        }
        if program.schedule.trigger_timing == TemporaryTriggerTiming::DepartureDay
            && event_day != trigger_day
        {
            return Err(invalid(
                "departure-timed temporary skip did not occur on its trigger day",
            ));
        }
        return Ok(());
    }

    let resolution = program
        .travel
        .resolution(state.residence)
        .ok_or_else(|| invalid("temporary skip has no travel resolution for residence"))?;
    let TemporaryTravelResolution::Reachable {
        outbound_travel_days,
        ..
    } = resolution
    else {
        if reason != TemporaryJourneyIneligibility::Unreachable || event_day != trigger_day {
            return Err(invalid(
                "unreachable temporary skip reason/day does not match travel resolution",
            ));
        }
        return Ok(());
    };

    match program.schedule.trigger_timing {
        TemporaryTriggerTiming::DepartureDay => Err(invalid(
            "eligible reachable departure-timed trigger was skipped for an invalid reason",
        )),
        TemporaryTriggerTiming::TargetArrivalDay => {
            let Some(departure_day) = trigger_day.checked_sub(u64::from(outbound_travel_days)) else {
                if reason != TemporaryJourneyIneligibility::DepartureBeforeSimulationStart
                    || event_day != 0
                {
                    return Err(invalid(
                        "pre-start temporary skip reason/day does not match travel timing",
                    ));
                }
                return Ok(());
            };
            if reason != TemporaryJourneyIneligibility::DepartureWindowMissed
                || event_day != trigger_day
                || departure_day >= event_day
            {
                return Err(invalid(
                    "missed-window temporary skip reason/day does not match travel timing",
                ));
            }
            Ok(())
        }
    }
}

fn validate_departure_travel(
    program: &TemporaryMobilityProgram,
    residence: CellId,
    destination: CellId,
    outbound_travel_days: u32,
    return_travel_days: u32,
    travel_model_identity: Option<&str>,
    accumulated_travel_cost_units: Option<u64>,
) -> Result<(), TemporaryMobilityHistoryError> {
    let Some(TemporaryTravelResolution::Reachable {
        destination: expected_destination,
        outbound_travel_days: expected_outbound,
        return_travel_days: expected_return,
    }) = program.travel.resolution(residence)
    else {
        return Err(invalid(
            "temporary departure occurred from a residence that is not reachable",
        ));
    };
    if destination != expected_destination
        || outbound_travel_days != expected_outbound
        || return_travel_days != expected_return
    {
        return Err(invalid(
            "temporary departure destination/travel duration does not match the resolved program",
        ));
    }
    let expected_model_identity = program.travel.travel_model().map(|model| model.identity());
    let expected_cost = program.travel.accumulated_cost_units(residence);
    if travel_model_identity != expected_model_identity.as_deref()
        || accumulated_travel_cost_units != expected_cost
    {
        return Err(invalid(
            "temporary departure travel metadata does not match the resolved program",
        ));
    }
    Ok(())
}

fn require_active(
    household: &HouseholdHistory,
    journey: TemporaryJourneyId,
    expected_stage: JourneyStage,
    journeys: &BTreeMap<u64, JourneyHistory>,
) -> Result<(), TemporaryMobilityHistoryError> {
    if household.active_journey != Some(journey) {
        return Err(invalid(
            "temporary transition references the wrong active journey",
        ));
    }
    let history = journeys
        .get(&journey.0)
        .ok_or_else(|| invalid("temporary transition references an unknown journey"))?;
    if history.stage != expected_stage {
        return Err(invalid(
            "temporary transition occurred from an invalid lifecycle stage",
        ));
    }
    Ok(())
}

fn require_completed_trigger_outcomes(
    program: &TemporaryMobilityProgram,
    end_day: u64,
    household_count: usize,
    outcomes: &BTreeSet<(u32, u64)>,
) -> Result<(), TemporaryMobilityHistoryError> {
    for (trigger_index, &trigger_day) in program.schedule.trigger_days.iter().enumerate() {
        if trigger_day > end_day {
            continue;
        }
        let trigger_index = u32::try_from(trigger_index)
            .map_err(|_| invalid("temporary trigger index exceeds u32"))?;
        for raw in 1..=household_count as u64 {
            if !outcomes.contains(&(trigger_index, raw)) {
                return Err(invalid(format!(
                    "missing temporary trigger outcome for trigger {trigger_index}, household {raw}"
                )));
            }
        }
    }
    Ok(())
}

fn reconcile_terminal(
    checkpoint: &SimulationCheckpoint,
    households: &[HouseholdHistory],
    journeys: &BTreeMap<u64, JourneyHistory>,
) -> Result<(), TemporaryMobilityHistoryError> {
    let mut final_living = vec![0_u64; households.len()];
    for raw in 1..=checkpoint.population.person_count() as u64 {
        let person = checkpoint
            .population
            .person(crate::ids::PersonId::new(raw))
            .ok_or_else(|| invalid(format!("missing terminal person {raw}")))?;
        if person.is_alive() {
            let index = household_index(person.household, households.len())?;
            final_living[index] = final_living[index]
                .checked_add(1)
                .ok_or_else(|| invalid("terminal household living count overflow"))?;
        }
    }

    for raw in 1..=households.len() as u64 {
        let household = HouseholdId::new(raw);
        let index = household_index(household, households.len())?;
        let replay = &households[index];
        if replay.living != final_living[index]
            || checkpoint.population.household_location(household) != Some(replay.residence)
            || checkpoint.temporary_mobility.presence(household) != Some(replay.presence)
        {
            return Err(invalid(format!(
                "temporary-history terminal household state mismatch for household {raw}"
            )));
        }
        let terminal_active = checkpoint.temporary_mobility.active_journey(household);
        match (replay.active_journey, terminal_active) {
            (None, None) => {}
            (Some(journey), Some(active)) if active.journey == journey => {
                let history = journeys.get(&journey.0).ok_or_else(|| {
                    invalid("terminal active journey is missing from replay history")
                })?;
                let expected_stage = match replay.presence {
                    HouseholdPresence::OutboundTransit { .. } => JourneyStage::Outbound,
                    HouseholdPresence::Visiting { .. } => JourneyStage::Visiting,
                    HouseholdPresence::ReturnTransit { .. } => JourneyStage::Returning,
                    HouseholdPresence::AtResidence => {
                        return Err(invalid(
                            "terminal active journey exists while replay is at residence",
                        ));
                    }
                };
                if history.stage != expected_stage
                    || history.household != active.household
                    || history.trigger_index != active.trigger_index.unwrap_or(u32::MAX)
                    || history.trigger_day != active.trigger_day
                    || history.region_id != active.region_id
                    || history.region_identity != active.region_identity
                    || history.residence != active.residence
                    || history.destination != active.destination
                    || history.travel_model_identity != active.travel_model_identity
                    || history.accumulated_travel_cost_units
                        != active.accumulated_travel_cost_units
                    || history.departure_day != active.departure_day
                    || history.arrival_day != active.arrival_day
                    || history.return_departure_day != active.return_departure_day
                    || history.completion_day != active.completion_day
                    || history.outbound_travel_days != active.outbound_travel_days
                    || history.return_travel_days != active.return_travel_days
                {
                    return Err(invalid(
                        "terminal active journey metadata does not match replay history",
                    ));
                }
            }
            _ => {
                return Err(invalid(format!(
                    "temporary-history terminal active journey mismatch for household {raw}"
                )));
            }
        }
    }

    for history in journeys.values() {
        if matches!(history.stage, JourneyStage::Completed | JourneyStage::TerminatedNoLivingMembers)
        {
            continue;
        }
        let index = household_index(history.household, households.len())?;
        if households[index].active_journey != Some(TemporaryJourneyId::new(
            journeys
                .iter()
                .find_map(|(id, candidate)| std::ptr::eq(candidate, history).then_some(*id))
                .unwrap_or(0),
        )) {
            return Err(invalid(
                "non-terminal temporary journey is not the household's terminal active journey",
            ));
        }
    }
    Ok(())
}

fn household_index(
    household: HouseholdId,
    household_count: usize,
) -> Result<usize, TemporaryMobilityHistoryError> {
    let index = usize::try_from(
        household
            .0
            .checked_sub(1)
            .ok_or_else(|| invalid("temporary history references household zero"))?,
    )
    .map_err(|_| invalid("temporary history household id does not fit usize"))?;
    if index >= household_count {
        return Err(invalid(format!(
            "temporary history references household {} outside the population",
            household.0
        )));
    }
    Ok(index)
}
