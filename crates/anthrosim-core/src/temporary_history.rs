use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    EventKind, Population, PopulationInitialization, SimulationCheckpoint,
    TemporaryJourneyObservability, TemporaryTriggerTiming, World,
    derive_temporary_mobility_observability,
    ids::{HouseholdId, TemporaryJourneyId},
    rng::RngFactory,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{0}")]
pub struct TemporaryMobilityHistoryError(String);

fn invalid(message: impl Into<String>) -> TemporaryMobilityHistoryError {
    TemporaryMobilityHistoryError(message.into())
}

/// Validate authoritative M9 history without requiring a derived report artifact on disk.
///
/// The existing temporary-observability replay is the single household-by-household authority for
/// reconstructing residence, living membership, physical presence and active journeys. This
/// validator runs that replay in memory, then adds historical guarantees that are intentionally
/// stronger than report generation alone: program/schedule identity on every trigger outcome,
/// canonical journey identifiers, exact transition days, and completeness for trigger days that
/// have passed the checkpoint boundary.
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

    let population_config = checkpoint.experiment.population;
    let initial_population = match population_config.initialization {
        PopulationInitialization::SyntheticValidationV1 => Population::initialize(
            population_config,
            world,
            RngFactory::new(checkpoint.experiment.seed),
        ),
        PopulationInitialization::DeclaredFounderStateV1 => {
            let definition = checkpoint
                .experiment
                .founder_population
                .as_ref()
                .ok_or_else(|| {
                    invalid(
                        "could not reconstruct founder population: declared founder initialization is missing founderPopulation in checkpoint experiment",
                    )
                })?;
            Population::initialize_declared_founder_state_v1(
                population_config,
                definition,
                world,
                &checkpoint.experiment.demography,
            )
        }
    }
    .map_err(|error| invalid(format!("could not reconstruct founder population: {error}")))?;
    let report = derive_temporary_mobility_observability(world, &initial_population, checkpoint)
        .map_err(|error| invalid(format!("temporary event replay failed: {error}")))?;
    let mut household_creation_days = BTreeMap::<u64, u64>::new();
    let mut first_dynamic_household = None::<u64>;
    for record in &checkpoint.events.events {
        if let EventKind::HouseholdFission { new_household, .. } = &record.event {
            if household_creation_days
                .insert(new_household.0, record.day)
                .is_some()
            {
                return Err(invalid(format!(
                    "duplicate household-fission creation event for household {}",
                    new_household.0
                )));
            }
            first_dynamic_household = Some(
                first_dynamic_household
                    .map_or(new_household.0, |current| current.min(new_household.0)),
            );
        }
    }
    let founder_household_count = first_dynamic_household
        .map(|raw| raw.saturating_sub(1))
        .unwrap_or_else(|| initial_population.household_count() as u64);

    let mut journeys = BTreeMap::<u64, &TemporaryJourneyObservability>::new();
    let mut next_journey_id = 1_u64;
    for journey in &report.journeys {
        if journey.journey.0 != next_journey_id {
            return Err(invalid(format!(
                "temporary journey id {} is not the next canonical id {next_journey_id}",
                journey.journey.0
            )));
        }
        if journeys.insert(journey.journey.0, journey).is_some() {
            return Err(invalid(format!(
                "duplicate temporary journey id {}",
                journey.journey.0
            )));
        }
        next_journey_id = next_journey_id
            .checked_add(1)
            .ok_or_else(|| invalid("temporary journey id sequence overflow"))?;
    }

    let expected_region_id = program.region.region_id.as_str();
    let expected_region_identity = program.region.identity();
    let mut trigger_outcomes = BTreeSet::<(u32, u64)>::new();

    for record in &checkpoint.events.events {
        match &record.event {
            EventKind::TemporaryJourneyNotStarted {
                household,
                region_id,
                region_identity,
                trigger_index,
                trigger_day,
                ..
            } => {
                validate_region(
                    region_id,
                    region_identity,
                    expected_region_id,
                    &expected_region_identity,
                )?;
                validate_trigger(
                    program.schedule.trigger_days.as_slice(),
                    *trigger_index,
                    *trigger_day,
                )?;
                record_trigger_outcome(&mut trigger_outcomes, *trigger_index, *household)?;
            }
            EventKind::TemporaryJourneyDeparted {
                household,
                journey,
                region_id,
                region_identity,
                trigger_index,
                trigger_day,
                departure_day,
                arrival_day,
                ..
            } => {
                validate_region(
                    region_id,
                    region_identity,
                    expected_region_id,
                    &expected_region_identity,
                )?;
                validate_trigger(
                    program.schedule.trigger_days.as_slice(),
                    *trigger_index,
                    *trigger_day,
                )?;
                record_trigger_outcome(&mut trigger_outcomes, *trigger_index, *household)?;
                if record.day != *departure_day {
                    return Err(invalid(
                        "temporary departure event day does not match its declared departure day",
                    ));
                }
                match program.schedule.trigger_timing {
                    TemporaryTriggerTiming::DepartureDay if departure_day != trigger_day => {
                        return Err(invalid(
                            "departure-timed temporary journey did not depart on its trigger day",
                        ));
                    }
                    TemporaryTriggerTiming::TargetArrivalDay if arrival_day != trigger_day => {
                        return Err(invalid(
                            "target-arrival temporary journey did not arrive on its trigger day",
                        ));
                    }
                    _ => {}
                }
                require_journey(&journeys, *journey)?;
            }
            EventKind::TemporaryJourneyArrived { journey, .. } => {
                let journey = require_journey(&journeys, *journey)?;
                if record.day != journey.arrival_day {
                    return Err(invalid(
                        "temporary arrival event day does not match the planned arrival day",
                    ));
                }
            }
            EventKind::TemporaryReturnDeparted { journey, .. } => {
                let journey = require_journey(&journeys, *journey)?;
                if record.day != journey.return_departure_day {
                    return Err(invalid(
                        "temporary return-departure event day does not match the planned return day",
                    ));
                }
            }
            EventKind::TemporaryJourneyCompleted { journey, .. } => {
                let journey = require_journey(&journeys, *journey)?;
                if record.day != journey.completion_day {
                    return Err(invalid(
                        "temporary completion event day does not match the planned completion day",
                    ));
                }
            }
            EventKind::Birth { .. }
            | EventKind::Death { .. }
            | EventKind::HouseholdMigration { .. }
            | EventKind::HouseholdFission { .. } => {}
        }
    }

    for (trigger_index, &trigger_day) in program.schedule.trigger_days.iter().enumerate() {
        if trigger_day > checkpoint.time.days() {
            continue;
        }
        let trigger_index = u32::try_from(trigger_index)
            .map_err(|_| invalid("temporary trigger index exceeds u32"))?;
        for raw in 1..=checkpoint.population.household_count() as u64 {
            let existed_for_trigger = if raw <= founder_household_count {
                true
            } else {
                let creation_day = household_creation_days.get(&raw).ok_or_else(|| {
                    invalid(format!(
                        "dynamic household {raw} has no household-fission creation event"
                    ))
                })?;
                trigger_day > *creation_day
            };
            if existed_for_trigger && !trigger_outcomes.contains(&(trigger_index, raw)) {
                return Err(invalid(format!(
                    "missing temporary trigger outcome for trigger {trigger_index}, household {raw}"
                )));
            }
        }
    }

    Ok(())
}

fn validate_region(
    region_id: &str,
    region_identity: &str,
    expected_id: &str,
    expected_identity: &str,
) -> Result<(), TemporaryMobilityHistoryError> {
    if region_id != expected_id || region_identity != expected_identity {
        return Err(invalid(
            "temporary event focal-region identity does not match the configured program",
        ));
    }
    Ok(())
}

fn validate_trigger(
    trigger_days: &[u64],
    trigger_index: u32,
    trigger_day: u64,
) -> Result<(), TemporaryMobilityHistoryError> {
    let index = usize::try_from(trigger_index)
        .map_err(|_| invalid("temporary trigger index does not fit usize"))?;
    if trigger_days.get(index).copied() != Some(trigger_day) {
        return Err(invalid(
            "temporary event trigger identity does not match the configured schedule",
        ));
    }
    Ok(())
}

fn record_trigger_outcome(
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

fn require_journey<'a>(
    journeys: &BTreeMap<u64, &'a TemporaryJourneyObservability>,
    journey: TemporaryJourneyId,
) -> Result<&'a TemporaryJourneyObservability, TemporaryMobilityHistoryError> {
    journeys.get(&journey.0).copied().ok_or_else(|| {
        invalid(format!(
            "temporary event references unknown journey {}",
            journey.0
        ))
    })
}
