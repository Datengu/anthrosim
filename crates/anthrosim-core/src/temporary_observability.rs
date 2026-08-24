use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    EventKind, EventProvenance, HouseholdPresence, MetricProvenance, Population,
    SimulationCheckpoint, TemporaryJourneyIneligibility, TemporaryMobilityProgram,
    TemporaryTravelModel, TemporaryTravelResolution, World,
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    temporary_travel::temporary_travel_edge_cost,
};

const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityObservabilityReport {
    pub schema_version: u32,
    pub provenance: MetricProvenance,
    pub source: TemporaryMobilityObservabilitySource,
    pub summary: TemporaryMobilityObservabilitySummary,
    pub cells: Vec<TemporaryMobilityCellObservability>,
    pub journeys: Vec<TemporaryJourneyObservability>,
    pub origin_catchment: Vec<TemporaryOriginCatchment>,
    pub visit_duration_distribution: Vec<TemporaryVisitDurationBin>,
    pub unavailable_observables: Vec<String>,
}

impl TemporaryMobilityObservabilityReport {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityObservabilitySource {
    pub model_version: String,
    pub model_semantics_id: String,
    pub git_commit: Option<String>,
    pub seed: u64,
    pub end_day: u64,
    pub run_state_digest64: u64,
    pub world_digest64: u64,
    pub temporary_mobility_config_identity: Option<String>,
    pub temporary_mobility_program_identity: String,
    pub region_id: String,
    pub region_identity: String,
    pub travel_model_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityObservabilitySummary {
    pub provenance: MetricProvenance,
    pub observation_duration_days: u64,
    pub trigger_outcomes: u64,
    pub journeys_started: u64,
    pub arrivals: u64,
    pub return_departures: u64,
    pub journeys_completed: u64,
    pub journeys_active_at_end: u64,
    pub journeys_terminated_no_living_members: u64,
    pub not_started_total: u64,
    pub not_started_no_living_members: u64,
    pub not_started_active_journey: u64,
    pub not_started_residence_in_region: u64,
    pub not_started_unreachable: u64,
    pub not_started_departure_before_simulation_start: u64,
    pub not_started_departure_window_missed: u64,
    pub total_living_person_days: u64,
    pub persistent_residence_person_days: u64,
    pub focal_region_resident_person_days: u64,
    pub at_residence_person_days: u64,
    pub visitor_person_days: u64,
    pub visitor_household_days: u64,
    pub outbound_transit_person_days: u64,
    pub return_transit_person_days: u64,
    pub transit_person_days: u64,
    pub peak_visitors: u64,
    pub mean_visitors_millipersons: Option<u64>,
    pub people_at_departure: u64,
    pub total_outbound_travel_days: u64,
    pub total_return_travel_days: u64,
    pub total_travel_days: u64,
    pub total_round_trip_travel_cost_units: u64,
    pub total_round_trip_route_distance_edges: u64,
    pub route_distance_unavailable_journeys: u64,
}

impl Default for TemporaryMobilityObservabilitySummary {
    fn default() -> Self {
        Self {
            provenance: MetricProvenance::Derived,
            observation_duration_days: 0,
            trigger_outcomes: 0,
            journeys_started: 0,
            arrivals: 0,
            return_departures: 0,
            journeys_completed: 0,
            journeys_active_at_end: 0,
            journeys_terminated_no_living_members: 0,
            not_started_total: 0,
            not_started_no_living_members: 0,
            not_started_active_journey: 0,
            not_started_residence_in_region: 0,
            not_started_unreachable: 0,
            not_started_departure_before_simulation_start: 0,
            not_started_departure_window_missed: 0,
            total_living_person_days: 0,
            persistent_residence_person_days: 0,
            focal_region_resident_person_days: 0,
            at_residence_person_days: 0,
            visitor_person_days: 0,
            visitor_household_days: 0,
            outbound_transit_person_days: 0,
            return_transit_person_days: 0,
            transit_person_days: 0,
            peak_visitors: 0,
            mean_visitors_millipersons: None,
            people_at_departure: 0,
            total_outbound_travel_days: 0,
            total_return_travel_days: 0,
            total_travel_days: 0,
            total_round_trip_travel_cost_units: 0,
            total_round_trip_route_distance_edges: 0,
            route_distance_unavailable_journeys: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityCellObservability {
    pub provenance: MetricProvenance,
    pub cell: CellId,
    pub persistent_residence_person_days: u64,
    pub at_residence_person_days: u64,
    pub visitor_person_days: u64,
    pub visitor_household_days: u64,
    pub arrivals: u64,
    pub return_departures: u64,
    pub peak_visitors: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryJourneyObservedStatus {
    Completed,
    ActiveOutboundTransit,
    ActiveVisiting,
    ActiveReturnTransit,
    TerminatedNoLivingMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryJourneyObservability {
    pub provenance: MetricProvenance,
    pub journey: TemporaryJourneyId,
    pub household: HouseholdId,
    pub trigger_index: u32,
    pub trigger_day: u64,
    pub residence: CellId,
    pub destination: CellId,
    pub region_id: String,
    pub region_identity: String,
    pub travel_model_identity: Option<String>,
    pub one_way_accumulated_travel_cost_units: Option<u64>,
    pub one_way_route_distance_edges: Option<u32>,
    pub people_at_departure: u32,
    pub departure_day: u64,
    pub arrival_day: u64,
    pub return_departure_day: u64,
    pub completion_day: u64,
    pub outbound_travel_days: u32,
    pub planned_visit_duration_days: u32,
    pub return_travel_days: u32,
    pub observed_outbound_transit_person_days: u64,
    pub observed_visitor_person_days: u64,
    pub observed_visitor_household_days: u64,
    pub observed_return_transit_person_days: u64,
    pub status: TemporaryJourneyObservedStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryOriginCatchment {
    pub provenance: MetricProvenance,
    pub origin: CellId,
    pub trigger_outcomes: u64,
    pub journeys_started: u64,
    pub arrivals: u64,
    pub return_departures: u64,
    pub journeys_completed: u64,
    pub journeys_terminated_no_living_members: u64,
    pub not_started_total: u64,
    pub not_started_unreachable: u64,
    pub people_at_departure: u64,
    pub visitor_person_days: u64,
    pub transit_person_days: u64,
    pub total_travel_days: u64,
    pub total_round_trip_travel_cost_units: u64,
    pub total_round_trip_route_distance_edges: u64,
    pub route_distance_unavailable_journeys: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryVisitDurationBin {
    pub provenance: MetricProvenance,
    pub duration_days: u32,
    pub journeys: u64,
    pub completed_journeys: u64,
    pub terminated_journeys: u64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{0}")]
pub struct TemporaryMobilityObservabilityError(String);

fn invalid(message: impl Into<String>) -> TemporaryMobilityObservabilityError {
    TemporaryMobilityObservabilityError(message.into())
}

#[derive(Debug, Clone)]
struct HouseholdReplay {
    residence: CellId,
    living: u64,
    presence: HouseholdPresence,
    active_journey: Option<TemporaryJourneyId>,
    last_day: u64,
}

#[derive(Debug, Clone, Default)]
struct CellAccumulator {
    persistent_residence_person_days: u64,
    at_residence_person_days: u64,
    visitor_person_days: u64,
    visitor_household_days: u64,
    arrivals: u64,
    return_departures: u64,
    current_visitors: u64,
    peak_visitors: u64,
}

#[derive(Debug, Clone, Default)]
struct OriginAccumulator {
    trigger_outcomes: u64,
    journeys_started: u64,
    arrivals: u64,
    return_departures: u64,
    journeys_completed: u64,
    journeys_terminated_no_living_members: u64,
    not_started_total: u64,
    not_started_unreachable: u64,
    people_at_departure: u64,
    visitor_person_days: u64,
    transit_person_days: u64,
    total_travel_days: u64,
    total_round_trip_travel_cost_units: u64,
    total_round_trip_route_distance_edges: u64,
    route_distance_unavailable_journeys: u64,
}

struct Replay<'a> {
    program: &'a TemporaryMobilityProgram,
    households: Vec<HouseholdReplay>,
    cells: Vec<CellAccumulator>,
    journeys: Vec<TemporaryJourneyObservability>,
    journey_index: BTreeMap<u64, usize>,
    origins: BTreeMap<u64, OriginAccumulator>,
    trigger_outcomes: BTreeSet<(u32, u64)>,
    summary: TemporaryMobilityObservabilitySummary,
    current_visitors: u64,
    route_distance_edges: Vec<Option<u32>>,
}

pub fn derive_temporary_mobility_observability(
    world: &World,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
) -> Result<TemporaryMobilityObservabilityReport, TemporaryMobilityObservabilityError> {
    initial_population
        .validate(world)
        .map_err(|error| invalid(format!("initial population failed validation: {error}")))?;
    checkpoint
        .population
        .validate(world)
        .map_err(|error| invalid(format!("checkpoint population failed validation: {error}")))?;
    let world_digest64 = world.digest64();
    if checkpoint.world_digest64 != world_digest64 {
        return Err(invalid(format!(
            "world digest mismatch: checkpoint {}, supplied world {}",
            checkpoint.world_digest64, world_digest64
        )));
    }
    if initial_population.household_count() != checkpoint.population.household_count() {
        return Err(invalid("initial and checkpoint household counts differ"));
    }
    let end_day = checkpoint.time.days();
    checkpoint
        .temporary_mobility
        .validate_at_day(end_day, &checkpoint.population, world)
        .map_err(|error| {
            invalid(format!(
                "temporary mobility checkpoint state is invalid: {error}"
            ))
        })?;
    let program = checkpoint
        .temporary_mobility
        .program()
        .ok_or_else(|| invalid("temporary mobility observability requires a configured program"))?;
    program
        .validate(world)
        .map_err(|error| invalid(format!("temporary mobility program is invalid: {error}")))?;
    if let Some(config) = checkpoint.experiment.temporary_mobility.as_ref() {
        let expected = config.derive_program(world).map_err(|error| {
            invalid(format!(
                "temporary mobility config cannot derive program: {error}"
            ))
        })?;
        if &expected != program {
            return Err(invalid(
                "temporary mobility checkpoint program does not match the experiment definition",
            ));
        }
    }

    let route_distance_edges = derive_route_distances(program, world)?;
    let mut households = Vec::with_capacity(initial_population.household_count());
    for raw in 1..=initial_population.household_count() as u64 {
        let household = HouseholdId::new(raw);
        let residence = initial_population
            .household_location(household)
            .ok_or_else(|| invalid(format!("missing initial household {raw}")))?;
        households.push(HouseholdReplay {
            residence,
            living: 0,
            presence: HouseholdPresence::AtResidence,
            active_journey: None,
            last_day: 0,
        });
    }
    for raw in 1..=initial_population.person_count() as u64 {
        let person = initial_population
            .person(crate::ids::PersonId::new(raw))
            .ok_or_else(|| invalid(format!("missing initial person {raw}")))?;
        if !person.is_alive() {
            return Err(invalid("initial population contains a dead person"));
        }
        let index = household_index(person.household, households.len())?;
        households[index].living = add(households[index].living, 1)?;
    }

    let mut replay = Replay {
        program,
        households,
        cells: vec![CellAccumulator::default(); world.cell_count()],
        journeys: Vec::new(),
        journey_index: BTreeMap::new(),
        origins: BTreeMap::new(),
        trigger_outcomes: BTreeSet::new(),
        summary: TemporaryMobilityObservabilitySummary {
            provenance: MetricProvenance::Derived,
            observation_duration_days: end_day,
            ..TemporaryMobilityObservabilitySummary::default()
        },
        current_visitors: 0,
        route_distance_edges,
    };

    replay_events(&mut replay, checkpoint, world)?;
    for index in 0..replay.households.len() {
        accrue_household(&mut replay, index, end_day)?;
    }
    reconcile_terminal_state(&mut replay, checkpoint)?;
    finalize_summary(&mut replay)?;

    let cells = replay
        .cells
        .into_iter()
        .enumerate()
        .map(|(index, cell)| TemporaryMobilityCellObservability {
            provenance: MetricProvenance::Derived,
            cell: CellId::new(index as u64 + 1),
            persistent_residence_person_days: cell.persistent_residence_person_days,
            at_residence_person_days: cell.at_residence_person_days,
            visitor_person_days: cell.visitor_person_days,
            visitor_household_days: cell.visitor_household_days,
            arrivals: cell.arrivals,
            return_departures: cell.return_departures,
            peak_visitors: cell.peak_visitors,
        })
        .collect::<Vec<_>>();
    let origin_catchment = replay
        .origins
        .into_iter()
        .map(|(origin, row)| TemporaryOriginCatchment {
            provenance: MetricProvenance::Derived,
            origin: CellId::new(origin),
            trigger_outcomes: row.trigger_outcomes,
            journeys_started: row.journeys_started,
            arrivals: row.arrivals,
            return_departures: row.return_departures,
            journeys_completed: row.journeys_completed,
            journeys_terminated_no_living_members: row.journeys_terminated_no_living_members,
            not_started_total: row.not_started_total,
            not_started_unreachable: row.not_started_unreachable,
            people_at_departure: row.people_at_departure,
            visitor_person_days: row.visitor_person_days,
            transit_person_days: row.transit_person_days,
            total_travel_days: row.total_travel_days,
            total_round_trip_travel_cost_units: row.total_round_trip_travel_cost_units,
            total_round_trip_route_distance_edges: row.total_round_trip_route_distance_edges,
            route_distance_unavailable_journeys: row.route_distance_unavailable_journeys,
        })
        .collect::<Vec<_>>();
    let visit_duration_distribution = duration_distribution(&replay.journeys)?;

    Ok(TemporaryMobilityObservabilityReport {
        schema_version: TemporaryMobilityObservabilityReport::CURRENT_SCHEMA_VERSION,
        provenance: MetricProvenance::Derived,
        source: TemporaryMobilityObservabilitySource {
            model_version: checkpoint.model_version.clone(),
            model_semantics_id: checkpoint.model_semantics_id.clone(),
            git_commit: checkpoint.git_commit.clone(),
            seed: checkpoint.experiment.seed,
            end_day,
            run_state_digest64: checkpoint.state_digest64,
            world_digest64,
            temporary_mobility_config_identity: checkpoint
                .experiment
                .temporary_mobility
                .as_ref()
                .map(|config| config.identity()),
            temporary_mobility_program_identity: program.identity(),
            region_id: program.region.region_id.clone(),
            region_identity: program.region.identity(),
            travel_model_identity: program.travel.travel_model().map(TemporaryTravelModel::identity),
        },
        summary: replay.summary,
        cells,
        journeys: replay.journeys,
        origin_catchment,
        visit_duration_distribution,
        unavailable_observables: vec![
            "M9 v1 transit has no authoritative per-day world cell; transit person-days are intentionally non-spatial"
                .to_owned(),
            "temporary travel purpose or social motive is not represented by the M9 v1 lifecycle"
                .to_owned(),
        ],
    })
}

fn replay_events(
    replay: &mut Replay<'_>,
    checkpoint: &SimulationCheckpoint,
    world: &World,
) -> Result<(), TemporaryMobilityObservabilityError> {
    let mut previous_sequence = 0_u64;
    let mut previous_day = 0_u64;
    for record in &checkpoint.events.events {
        if record.provenance != EventProvenance::Authoritative {
            return Err(invalid(format!(
                "event {} is not authoritative",
                record.sequence
            )));
        }
        if record.sequence != previous_sequence.saturating_add(1) {
            return Err(invalid(format!(
                "event sequence mismatch: expected {}, found {}",
                previous_sequence.saturating_add(1),
                record.sequence
            )));
        }
        if record.day < previous_day || record.day > checkpoint.time.days() {
            return Err(invalid(format!(
                "event {} day {} is outside replay range",
                record.sequence, record.day
            )));
        }
        previous_sequence = record.sequence;
        previous_day = record.day;

        let household = event_household(&record.event);
        let index = household_index(household, replay.households.len())?;
        accrue_household(replay, index, record.day)?;

        match &record.event {
            EventKind::Birth { .. } => {
                replay.households[index].living = add(replay.households[index].living, 1)?;
                if let HouseholdPresence::Visiting { destination, .. } =
                    replay.households[index].presence
                {
                    add_current_visitors(replay, destination, 1)?;
                }
            }
            EventKind::Death { .. } => {
                if replay.households[index].living == 0 {
                    return Err(invalid(format!(
                        "death for household {:?} would make living count negative",
                        household
                    )));
                }
                if let HouseholdPresence::Visiting { destination, .. } =
                    replay.households[index].presence
                {
                    remove_current_visitors(replay, destination, 1)?;
                }
                replay.households[index].living -= 1;
                if replay.households[index].living == 0
                    && !replay.households[index].presence.is_at_residence()
                {
                    terminate_extinct_journey(replay, index)?;
                }
            }
            EventKind::HouseholdMigration {
                origin,
                destination,
                people_moved,
                ..
            } => {
                if !replay.households[index].presence.is_at_residence()
                    || replay.households[index].active_journey.is_some()
                {
                    return Err(invalid(format!(
                        "permanent migration occurred while household {:?} was temporarily away",
                        household
                    )));
                }
                if replay.households[index].residence != *origin {
                    return Err(invalid(format!(
                        "migration origin {:?} does not match replay residence {:?}",
                        origin, replay.households[index].residence
                    )));
                }
                if u64::from(*people_moved) != replay.households[index].living {
                    return Err(invalid(format!(
                        "migration people count for household {:?} does not match replay living count",
                        household
                    )));
                }
                if world.cell(*destination).is_none() {
                    return Err(invalid("migration destination is outside world"));
                }
                replay.households[index].residence = *destination;
            }
            EventKind::TemporaryJourneyNotStarted {
                event_schema_version,
                trigger_index,
                reason,
                ..
            } => {
                validate_temp_event_schema(*event_schema_version)?;
                record_trigger_outcome(replay, household, *trigger_index)?;
                replay.summary.not_started_total = add(replay.summary.not_started_total, 1)?;
                count_not_started(&mut replay.summary, *reason)?;
                let origin = replay.households[index].residence;
                let row = replay.origins.entry(origin.0).or_default();
                row.trigger_outcomes = add(row.trigger_outcomes, 1)?;
                row.not_started_total = add(row.not_started_total, 1)?;
                if *reason == TemporaryJourneyIneligibility::Unreachable {
                    row.not_started_unreachable = add(row.not_started_unreachable, 1)?;
                }
            }
            EventKind::TemporaryJourneyDeparted {
                event_schema_version,
                journey,
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
                region_id,
                region_identity,
                ..
            } => {
                validate_temp_event_schema(*event_schema_version)?;
                record_trigger_outcome(replay, household, *trigger_index)?;
                if !replay.households[index].presence.is_at_residence()
                    || replay.households[index].active_journey.is_some()
                {
                    return Err(invalid(
                        "temporary departure started while household already away",
                    ));
                }
                if replay.households[index].residence != *residence {
                    return Err(invalid(
                        "temporary departure residence does not match replay residence",
                    ));
                }
                if u64::from(*people_affected) != replay.households[index].living {
                    return Err(invalid(
                        "temporary departure people count does not match replay living count",
                    ));
                }
                validate_departure_against_program(
                    replay,
                    *residence,
                    *destination,
                    travel_model_identity.as_deref(),
                    *accumulated_travel_cost_units,
                    *outbound_travel_days,
                    *return_travel_days,
                    region_id,
                    region_identity,
                )?;
                let route_distance =
                    replay.route_distance_edges[cell_index(*residence, replay.cells.len())?];
                let planned_visit = return_departure_day
                    .checked_sub(*arrival_day)
                    .ok_or_else(|| invalid("temporary journey has invalid visit interval"))?;
                let planned_visit = u32::try_from(planned_visit)
                    .map_err(|_| invalid("temporary visit duration exceeds u32"))?;
                if replay.journey_index.contains_key(&journey.0) {
                    return Err(invalid(format!(
                        "duplicate temporary journey id {}",
                        journey.0
                    )));
                }
                let row_index = replay.journeys.len();
                replay.journeys.push(TemporaryJourneyObservability {
                    provenance: MetricProvenance::Derived,
                    journey: *journey,
                    household,
                    trigger_index: *trigger_index,
                    trigger_day: *trigger_day,
                    residence: *residence,
                    destination: *destination,
                    region_id: region_id.clone(),
                    region_identity: region_identity.clone(),
                    travel_model_identity: travel_model_identity.clone(),
                    one_way_accumulated_travel_cost_units: *accumulated_travel_cost_units,
                    one_way_route_distance_edges: route_distance,
                    people_at_departure: *people_affected,
                    departure_day: *departure_day,
                    arrival_day: *arrival_day,
                    return_departure_day: *return_departure_day,
                    completion_day: *completion_day,
                    outbound_travel_days: *outbound_travel_days,
                    planned_visit_duration_days: planned_visit,
                    return_travel_days: *return_travel_days,
                    observed_outbound_transit_person_days: 0,
                    observed_visitor_person_days: 0,
                    observed_visitor_household_days: 0,
                    observed_return_transit_person_days: 0,
                    status: TemporaryJourneyObservedStatus::ActiveOutboundTransit,
                });
                replay.journey_index.insert(journey.0, row_index);
                replay.households[index].presence = HouseholdPresence::OutboundTransit {
                    journey: *journey,
                    destination: *destination,
                };
                replay.households[index].active_journey = Some(*journey);
                replay.summary.journeys_started = add(replay.summary.journeys_started, 1)?;
                replay.summary.people_at_departure = add(
                    replay.summary.people_at_departure,
                    u64::from(*people_affected),
                )?;
                replay.summary.total_outbound_travel_days = add(
                    replay.summary.total_outbound_travel_days,
                    u64::from(*outbound_travel_days),
                )?;
                replay.summary.total_return_travel_days = add(
                    replay.summary.total_return_travel_days,
                    u64::from(*return_travel_days),
                )?;
                if let Some(cost) = accumulated_travel_cost_units {
                    replay.summary.total_round_trip_travel_cost_units = add(
                        replay.summary.total_round_trip_travel_cost_units,
                        mul(*cost, 2)?,
                    )?;
                }
                if let Some(distance) = route_distance {
                    replay.summary.total_round_trip_route_distance_edges = add(
                        replay.summary.total_round_trip_route_distance_edges,
                        mul(u64::from(distance), 2)?,
                    )?;
                } else {
                    replay.summary.route_distance_unavailable_journeys =
                        add(replay.summary.route_distance_unavailable_journeys, 1)?;
                }
                let origin = replay.origins.entry(residence.0).or_default();
                origin.trigger_outcomes = add(origin.trigger_outcomes, 1)?;
                origin.journeys_started = add(origin.journeys_started, 1)?;
                origin.people_at_departure =
                    add(origin.people_at_departure, u64::from(*people_affected))?;
                origin.total_travel_days = add(
                    origin.total_travel_days,
                    u64::from(*outbound_travel_days) + u64::from(*return_travel_days),
                )?;
                if let Some(cost) = accumulated_travel_cost_units {
                    origin.total_round_trip_travel_cost_units =
                        add(origin.total_round_trip_travel_cost_units, mul(*cost, 2)?)?;
                }
                if let Some(distance) = route_distance {
                    origin.total_round_trip_route_distance_edges = add(
                        origin.total_round_trip_route_distance_edges,
                        mul(u64::from(distance), 2)?,
                    )?;
                } else {
                    origin.route_distance_unavailable_journeys =
                        add(origin.route_distance_unavailable_journeys, 1)?;
                }
            }
            EventKind::TemporaryJourneyArrived {
                event_schema_version,
                journey,
                destination,
                people_affected,
                ..
            } => {
                validate_temp_event_schema(*event_schema_version)?;
                require_active(replay, index, *journey, HouseholdPresenceKind::Outbound)?;
                if u64::from(*people_affected) != replay.households[index].living {
                    return Err(invalid(
                        "temporary arrival people count does not match replay living count",
                    ));
                }
                replay.households[index].presence = HouseholdPresence::Visiting {
                    journey: *journey,
                    destination: *destination,
                };
                let row_index = journey_row_index(replay, *journey)?;
                let residence = replay.journeys[row_index].residence;
                if replay.journeys[row_index].destination != *destination {
                    return Err(invalid(
                        "temporary arrival destination does not match departure row",
                    ));
                }
                replay.journeys[row_index].status = TemporaryJourneyObservedStatus::ActiveVisiting;
                replay.summary.arrivals = add(replay.summary.arrivals, 1)?;
                let cell_index = cell_index(*destination, replay.cells.len())?;
                replay.cells[cell_index].arrivals = add(replay.cells[cell_index].arrivals, 1)?;
                let origin = replay.origins.entry(residence.0).or_default();
                origin.arrivals = add(origin.arrivals, 1)?;
                add_current_visitors(replay, *destination, replay.households[index].living)?;
            }
            EventKind::TemporaryReturnDeparted {
                event_schema_version,
                journey,
                destination,
                residence,
                people_affected,
                ..
            } => {
                validate_temp_event_schema(*event_schema_version)?;
                require_active(replay, index, *journey, HouseholdPresenceKind::Visiting)?;
                if replay.households[index].residence != *residence
                    || u64::from(*people_affected) != replay.households[index].living
                {
                    return Err(invalid(
                        "temporary return departure does not reconcile with replay household",
                    ));
                }
                remove_current_visitors(replay, *destination, replay.households[index].living)?;
                replay.households[index].presence = HouseholdPresence::ReturnTransit {
                    journey: *journey,
                    destination: *destination,
                };
                let row_index = journey_row_index(replay, *journey)?;
                replay.journeys[row_index].status =
                    TemporaryJourneyObservedStatus::ActiveReturnTransit;
                replay.summary.return_departures = add(replay.summary.return_departures, 1)?;
                let cell_index = cell_index(*destination, replay.cells.len())?;
                replay.cells[cell_index].return_departures =
                    add(replay.cells[cell_index].return_departures, 1)?;
                let origin = replay.origins.entry(residence.0).or_default();
                origin.return_departures = add(origin.return_departures, 1)?;
            }
            EventKind::TemporaryJourneyCompleted {
                event_schema_version,
                journey,
                residence,
                people_affected,
                ..
            } => {
                validate_temp_event_schema(*event_schema_version)?;
                require_active(replay, index, *journey, HouseholdPresenceKind::Return)?;
                if replay.households[index].residence != *residence
                    || u64::from(*people_affected) != replay.households[index].living
                {
                    return Err(invalid(
                        "temporary completion does not reconcile with replay household",
                    ));
                }
                replay.households[index].presence = HouseholdPresence::AtResidence;
                replay.households[index].active_journey = None;
                let row_index = journey_row_index(replay, *journey)?;
                replay.journeys[row_index].status = TemporaryJourneyObservedStatus::Completed;
                replay.summary.journeys_completed = add(replay.summary.journeys_completed, 1)?;
                let origin = replay.origins.entry(residence.0).or_default();
                origin.journeys_completed = add(origin.journeys_completed, 1)?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum HouseholdPresenceKind {
    Outbound,
    Visiting,
    Return,
}

fn require_active(
    replay: &Replay<'_>,
    household_index: usize,
    journey: TemporaryJourneyId,
    expected: HouseholdPresenceKind,
) -> Result<(), TemporaryMobilityObservabilityError> {
    if replay.households[household_index].active_journey != Some(journey) {
        return Err(invalid(
            "temporary event references the wrong active journey",
        ));
    }
    let matches = match (expected, replay.households[household_index].presence) {
        (
            HouseholdPresenceKind::Outbound,
            HouseholdPresence::OutboundTransit {
                journey: active, ..
            },
        )
        | (
            HouseholdPresenceKind::Visiting,
            HouseholdPresence::Visiting {
                journey: active, ..
            },
        )
        | (
            HouseholdPresenceKind::Return,
            HouseholdPresence::ReturnTransit {
                journey: active, ..
            },
        ) => active == journey,
        _ => false,
    };
    matches
        .then_some(())
        .ok_or_else(|| invalid("temporary event occurred from an invalid replay presence state"))
}

fn accrue_household(
    replay: &mut Replay<'_>,
    index: usize,
    day: u64,
) -> Result<(), TemporaryMobilityObservabilityError> {
    let household = replay
        .households
        .get(index)
        .ok_or_else(|| invalid("invalid household replay index"))?
        .clone();
    let duration = day
        .checked_sub(household.last_day)
        .ok_or_else(|| invalid("household replay moved backwards in time"))?;
    if duration == 0 {
        replay.households[index].last_day = day;
        return Ok(());
    }
    let person_days = mul(household.living, duration)?;
    replay.summary.total_living_person_days =
        add(replay.summary.total_living_person_days, person_days)?;
    replay.summary.persistent_residence_person_days =
        add(replay.summary.persistent_residence_person_days, person_days)?;
    if replay.program.region.contains(household.residence) {
        replay.summary.focal_region_resident_person_days = add(
            replay.summary.focal_region_resident_person_days,
            person_days,
        )?;
    }
    let residence_index = cell_index(household.residence, replay.cells.len())?;
    replay.cells[residence_index].persistent_residence_person_days = add(
        replay.cells[residence_index].persistent_residence_person_days,
        person_days,
    )?;

    match household.presence {
        HouseholdPresence::AtResidence => {
            replay.summary.at_residence_person_days =
                add(replay.summary.at_residence_person_days, person_days)?;
            replay.cells[residence_index].at_residence_person_days = add(
                replay.cells[residence_index].at_residence_person_days,
                person_days,
            )?;
        }
        HouseholdPresence::OutboundTransit { journey, .. } => {
            replay.summary.outbound_transit_person_days =
                add(replay.summary.outbound_transit_person_days, person_days)?;
            let row_index = journey_row_index(replay, journey)?;
            replay.journeys[row_index].observed_outbound_transit_person_days = add(
                replay.journeys[row_index].observed_outbound_transit_person_days,
                person_days,
            )?;
            let origin = replay
                .origins
                .entry(replay.journeys[row_index].residence.0)
                .or_default();
            origin.transit_person_days = add(origin.transit_person_days, person_days)?;
        }
        HouseholdPresence::Visiting {
            journey,
            destination,
        } => {
            replay.summary.visitor_person_days =
                add(replay.summary.visitor_person_days, person_days)?;
            let household_days = if household.living > 0 { duration } else { 0 };
            replay.summary.visitor_household_days =
                add(replay.summary.visitor_household_days, household_days)?;
            let destination_index = cell_index(destination, replay.cells.len())?;
            replay.cells[destination_index].visitor_person_days = add(
                replay.cells[destination_index].visitor_person_days,
                person_days,
            )?;
            replay.cells[destination_index].visitor_household_days = add(
                replay.cells[destination_index].visitor_household_days,
                household_days,
            )?;
            let row_index = journey_row_index(replay, journey)?;
            replay.journeys[row_index].observed_visitor_person_days = add(
                replay.journeys[row_index].observed_visitor_person_days,
                person_days,
            )?;
            replay.journeys[row_index].observed_visitor_household_days = add(
                replay.journeys[row_index].observed_visitor_household_days,
                household_days,
            )?;
            let origin = replay
                .origins
                .entry(replay.journeys[row_index].residence.0)
                .or_default();
            origin.visitor_person_days = add(origin.visitor_person_days, person_days)?;
        }
        HouseholdPresence::ReturnTransit { journey, .. } => {
            replay.summary.return_transit_person_days =
                add(replay.summary.return_transit_person_days, person_days)?;
            let row_index = journey_row_index(replay, journey)?;
            replay.journeys[row_index].observed_return_transit_person_days = add(
                replay.journeys[row_index].observed_return_transit_person_days,
                person_days,
            )?;
            let origin = replay
                .origins
                .entry(replay.journeys[row_index].residence.0)
                .or_default();
            origin.transit_person_days = add(origin.transit_person_days, person_days)?;
        }
    }
    replay.households[index].last_day = day;
    Ok(())
}

fn terminate_extinct_journey(
    replay: &mut Replay<'_>,
    household_index: usize,
) -> Result<(), TemporaryMobilityObservabilityError> {
    let journey = replay.households[household_index]
        .active_journey
        .ok_or_else(|| invalid("away extinct household has no active journey"))?;
    let row_index = journey_row_index(replay, journey)?;
    replay.journeys[row_index].status = TemporaryJourneyObservedStatus::TerminatedNoLivingMembers;
    let origin_cell = replay.journeys[row_index].residence;
    replay.summary.journeys_terminated_no_living_members =
        add(replay.summary.journeys_terminated_no_living_members, 1)?;
    let origin = replay.origins.entry(origin_cell.0).or_default();
    origin.journeys_terminated_no_living_members =
        add(origin.journeys_terminated_no_living_members, 1)?;
    replay.households[household_index].presence = HouseholdPresence::AtResidence;
    replay.households[household_index].active_journey = None;
    Ok(())
}

fn add_current_visitors(
    replay: &mut Replay<'_>,
    destination: CellId,
    count: u64,
) -> Result<(), TemporaryMobilityObservabilityError> {
    replay.current_visitors = add(replay.current_visitors, count)?;
    replay.summary.peak_visitors = replay.summary.peak_visitors.max(replay.current_visitors);
    let index = cell_index(destination, replay.cells.len())?;
    replay.cells[index].current_visitors = add(replay.cells[index].current_visitors, count)?;
    replay.cells[index].peak_visitors = replay.cells[index]
        .peak_visitors
        .max(replay.cells[index].current_visitors);
    Ok(())
}

fn remove_current_visitors(
    replay: &mut Replay<'_>,
    destination: CellId,
    count: u64,
) -> Result<(), TemporaryMobilityObservabilityError> {
    replay.current_visitors = replay
        .current_visitors
        .checked_sub(count)
        .ok_or_else(|| invalid("global visitor count became negative"))?;
    let index = cell_index(destination, replay.cells.len())?;
    replay.cells[index].current_visitors = replay.cells[index]
        .current_visitors
        .checked_sub(count)
        .ok_or_else(|| invalid("cell visitor count became negative"))?;
    Ok(())
}

fn reconcile_terminal_state(
    replay: &mut Replay<'_>,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), TemporaryMobilityObservabilityError> {
    let mut final_living = vec![0_u64; replay.households.len()];
    for raw in 1..=checkpoint.population.person_count() as u64 {
        let person = checkpoint
            .population
            .person(crate::ids::PersonId::new(raw))
            .ok_or_else(|| invalid(format!("missing checkpoint person {raw}")))?;
        if person.is_alive() {
            let index = household_index(person.household, final_living.len())?;
            final_living[index] = add(final_living[index], 1)?;
        }
    }
    for raw in 1..=replay.households.len() as u64 {
        let household = HouseholdId::new(raw);
        let index = household_index(household, replay.households.len())?;
        if replay.households[index].living != final_living[index] {
            return Err(invalid(format!(
                "terminal living count mismatch for household {raw}"
            )));
        }
        let final_residence = checkpoint
            .population
            .household_location(household)
            .ok_or_else(|| invalid(format!("missing terminal household {raw}")))?;
        if replay.households[index].residence != final_residence {
            return Err(invalid(format!(
                "terminal residence mismatch for household {raw}"
            )));
        }
        let final_presence = checkpoint
            .temporary_mobility
            .presence(household)
            .ok_or_else(|| {
                invalid(format!(
                    "missing terminal temporary presence for household {raw}"
                ))
            })?;
        if replay.households[index].presence != final_presence {
            return Err(invalid(format!(
                "terminal temporary presence mismatch for household {raw}: replay {:?}, checkpoint {:?}",
                replay.households[index].presence, final_presence
            )));
        }
        let final_active = checkpoint
            .temporary_mobility
            .active_journey(household)
            .map(|journey| journey.journey);
        if replay.households[index].active_journey != final_active {
            return Err(invalid(format!(
                "terminal active journey mismatch for household {raw}"
            )));
        }
    }
    if replay.current_visitors
        != replay
            .cells
            .iter()
            .map(|cell| cell.current_visitors)
            .try_fold(0_u64, add)?
    {
        return Err(invalid(
            "terminal global/cell visitor counts do not reconcile",
        ));
    }
    Ok(())
}

fn finalize_summary(replay: &mut Replay<'_>) -> Result<(), TemporaryMobilityObservabilityError> {
    replay.summary.trigger_outcomes = u64::try_from(replay.trigger_outcomes.len())
        .map_err(|_| invalid("trigger outcome count exceeds u64"))?;
    replay.summary.transit_person_days = add(
        replay.summary.outbound_transit_person_days,
        replay.summary.return_transit_person_days,
    )?;
    replay.summary.total_travel_days = add(
        replay.summary.total_outbound_travel_days,
        replay.summary.total_return_travel_days,
    )?;
    replay.summary.journeys_active_at_end = replay
        .journeys
        .iter()
        .filter(|journey| {
            matches!(
                journey.status,
                TemporaryJourneyObservedStatus::ActiveOutboundTransit
                    | TemporaryJourneyObservedStatus::ActiveVisiting
                    | TemporaryJourneyObservedStatus::ActiveReturnTransit
            )
        })
        .count()
        .try_into()
        .map_err(|_| invalid("active journey count exceeds u64"))?;
    let physical = add(
        add(
            replay.summary.at_residence_person_days,
            replay.summary.visitor_person_days,
        )?,
        replay.summary.transit_person_days,
    )?;
    if physical != replay.summary.total_living_person_days {
        return Err(invalid(format!(
            "physical person-day partition mismatch: physical {physical}, total {}",
            replay.summary.total_living_person_days
        )));
    }
    if replay.summary.persistent_residence_person_days != replay.summary.total_living_person_days {
        return Err(invalid(
            "persistent-residence person-days do not equal total living person-days",
        ));
    }
    let not_started_by_reason = [
        replay.summary.not_started_no_living_members,
        replay.summary.not_started_active_journey,
        replay.summary.not_started_residence_in_region,
        replay.summary.not_started_unreachable,
        replay.summary.not_started_departure_before_simulation_start,
        replay.summary.not_started_departure_window_missed,
    ]
    .into_iter()
    .try_fold(0_u64, add)?;
    if not_started_by_reason != replay.summary.not_started_total {
        return Err(invalid(
            "not-started reason counts do not reconcile with total",
        ));
    }
    if add(
        replay.summary.journeys_started,
        replay.summary.not_started_total,
    )? != replay.summary.trigger_outcomes
    {
        return Err(invalid(
            "trigger outcomes do not reconcile with starts plus not-started outcomes",
        ));
    }
    let journey_states = add(
        add(
            replay.summary.journeys_completed,
            replay.summary.journeys_active_at_end,
        )?,
        replay.summary.journeys_terminated_no_living_members,
    )?;
    if journey_states != replay.summary.journeys_started {
        return Err(invalid(
            "started journeys do not reconcile with terminal journey statuses",
        ));
    }
    replay.summary.mean_visitors_millipersons = mul(replay.summary.visitor_person_days, 1_000)?
        .checked_div(replay.summary.observation_duration_days);
    Ok(())
}

fn duration_distribution(
    journeys: &[TemporaryJourneyObservability],
) -> Result<Vec<TemporaryVisitDurationBin>, TemporaryMobilityObservabilityError> {
    let mut bins = BTreeMap::<u32, (u64, u64, u64)>::new();
    for journey in journeys {
        let entry = bins.entry(journey.planned_visit_duration_days).or_default();
        entry.0 = add(entry.0, 1)?;
        if journey.status == TemporaryJourneyObservedStatus::Completed {
            entry.1 = add(entry.1, 1)?;
        }
        if journey.status == TemporaryJourneyObservedStatus::TerminatedNoLivingMembers {
            entry.2 = add(entry.2, 1)?;
        }
    }
    Ok(bins
        .into_iter()
        .map(
            |(duration_days, (journeys, completed_journeys, terminated_journeys))| {
                TemporaryVisitDurationBin {
                    provenance: MetricProvenance::Derived,
                    duration_days,
                    journeys,
                    completed_journeys,
                    terminated_journeys,
                }
            },
        )
        .collect())
}

fn event_household(event: &EventKind) -> HouseholdId {
    match event {
        EventKind::Birth { household, .. }
        | EventKind::Death { household, .. }
        | EventKind::HouseholdMigration { household, .. }
        | EventKind::TemporaryJourneyNotStarted { household, .. }
        | EventKind::TemporaryJourneyDeparted { household, .. }
        | EventKind::TemporaryJourneyArrived { household, .. }
        | EventKind::TemporaryReturnDeparted { household, .. }
        | EventKind::TemporaryJourneyCompleted { household, .. } => *household,
    }
}

fn validate_temp_event_schema(version: u32) -> Result<(), TemporaryMobilityObservabilityError> {
    if version != TEMPORARY_EVENT_SCHEMA_VERSION {
        return Err(invalid(format!(
            "temporary event schema {version} is unsupported by observability v1; expected {TEMPORARY_EVENT_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

fn record_trigger_outcome(
    replay: &mut Replay<'_>,
    household: HouseholdId,
    trigger_index: u32,
) -> Result<(), TemporaryMobilityObservabilityError> {
    if !replay.trigger_outcomes.insert((trigger_index, household.0)) {
        return Err(invalid(format!(
            "duplicate trigger outcome for trigger {trigger_index}, household {}",
            household.0
        )));
    }
    Ok(())
}

fn count_not_started(
    summary: &mut TemporaryMobilityObservabilitySummary,
    reason: TemporaryJourneyIneligibility,
) -> Result<(), TemporaryMobilityObservabilityError> {
    let slot = match reason {
        TemporaryJourneyIneligibility::NoLivingMembers => {
            &mut summary.not_started_no_living_members
        }
        TemporaryJourneyIneligibility::ActiveJourney => &mut summary.not_started_active_journey,
        TemporaryJourneyIneligibility::ResidenceInRegion => {
            &mut summary.not_started_residence_in_region
        }
        TemporaryJourneyIneligibility::Unreachable => &mut summary.not_started_unreachable,
        TemporaryJourneyIneligibility::DepartureBeforeSimulationStart => {
            &mut summary.not_started_departure_before_simulation_start
        }
        TemporaryJourneyIneligibility::DepartureWindowMissed => {
            &mut summary.not_started_departure_window_missed
        }
    };
    *slot = add(*slot, 1)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_departure_against_program(
    replay: &Replay<'_>,
    residence: CellId,
    destination: CellId,
    travel_model_identity: Option<&str>,
    accumulated_travel_cost_units: Option<u64>,
    outbound_travel_days: u32,
    return_travel_days: u32,
    region_id: &str,
    region_identity: &str,
) -> Result<(), TemporaryMobilityObservabilityError> {
    if region_id != replay.program.region.region_id
        || region_identity != replay.program.region.identity()
    {
        return Err(invalid(
            "temporary departure focal-region identity does not match program",
        ));
    }
    let expected_model = replay
        .program
        .travel
        .travel_model()
        .map(TemporaryTravelModel::identity);
    if travel_model_identity != expected_model.as_deref()
        || accumulated_travel_cost_units != replay.program.travel.accumulated_cost_units(residence)
    {
        return Err(invalid(
            "temporary departure travel metadata does not match program",
        ));
    }
    match replay.program.travel.resolution(residence) {
        Some(TemporaryTravelResolution::Reachable {
            destination: expected_destination,
            outbound_travel_days: expected_outbound,
            return_travel_days: expected_return,
        }) if expected_destination == destination
            && expected_outbound == outbound_travel_days
            && expected_return == return_travel_days =>
        {
            Ok(())
        }
        _ => Err(invalid(
            "temporary departure route resolution does not match program",
        )),
    }
}

fn journey_row_index(
    replay: &Replay<'_>,
    journey: TemporaryJourneyId,
) -> Result<usize, TemporaryMobilityObservabilityError> {
    replay
        .journey_index
        .get(&journey.0)
        .copied()
        .ok_or_else(|| {
            invalid(format!(
                "temporary journey {} has no departure row",
                journey.0
            ))
        })
}

fn household_index(
    household: HouseholdId,
    household_count: usize,
) -> Result<usize, TemporaryMobilityObservabilityError> {
    let index = usize::try_from(
        household
            .0
            .checked_sub(1)
            .ok_or_else(|| invalid("invalid household id"))?,
    )
    .map_err(|_| invalid("household id exceeds usize"))?;
    (index < household_count)
        .then_some(index)
        .ok_or_else(|| invalid(format!("household {} is outside replay state", household.0)))
}

fn cell_index(
    cell: CellId,
    cell_count: usize,
) -> Result<usize, TemporaryMobilityObservabilityError> {
    let index = usize::try_from(
        cell.0
            .checked_sub(1)
            .ok_or_else(|| invalid("invalid cell id"))?,
    )
    .map_err(|_| invalid("cell id exceeds usize"))?;
    (index < cell_count)
        .then_some(index)
        .ok_or_else(|| invalid(format!("cell {} is outside world", cell.0)))
}

fn add(a: u64, b: u64) -> Result<u64, TemporaryMobilityObservabilityError> {
    a.checked_add(b)
        .ok_or_else(|| invalid("temporary observability accounting overflow"))
}

fn mul(a: u64, b: u64) -> Result<u64, TemporaryMobilityObservabilityError> {
    a.checked_mul(b)
        .ok_or_else(|| invalid("temporary observability accounting overflow"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RouteLabel {
    cost: u64,
    destination: CellId,
    hops: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RouteQueueState {
    label: RouteLabel,
    cell: CellId,
}

impl Ord for RouteQueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .label
            .cmp(&self.label)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for RouteQueueState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn derive_route_distances(
    program: &TemporaryMobilityProgram,
    world: &World,
) -> Result<Vec<Option<u32>>, TemporaryMobilityObservabilityError> {
    let Some(model) = program.travel.travel_model() else {
        return Ok(vec![None; world.cell_count()]);
    };
    let mut labels = vec![None; world.cell_count()];
    let mut queue = BinaryHeap::new();
    for &destination in program.region.member_cells() {
        let index = cell_index(destination, world.cell_count())?;
        let label = RouteLabel {
            cost: 0,
            destination,
            hops: 0,
        };
        labels[index] = Some(label);
        queue.push(RouteQueueState {
            label,
            cell: destination,
        });
    }
    while let Some(current) = queue.pop() {
        let current_index = cell_index(current.cell, world.cell_count())?;
        if labels[current_index] != Some(current.label) {
            continue;
        }
        for neighbour in world.neighbours4(current.cell).into_iter().flatten() {
            if !model.is_traversable(world, neighbour) {
                continue;
            }
            let edge = temporary_travel_edge_cost(world, current.cell, neighbour)
                .map_err(|error| invalid(format!("route-distance replay failed: {error}")))?;
            let candidate = RouteLabel {
                cost: add(current.label.cost, edge)?,
                destination: current.label.destination,
                hops: current
                    .label
                    .hops
                    .checked_add(1)
                    .ok_or_else(|| invalid("route hop count exceeds u32"))?,
            };
            let neighbour_index = cell_index(neighbour, world.cell_count())?;
            if labels[neighbour_index].is_none_or(|existing| candidate < existing) {
                labels[neighbour_index] = Some(candidate);
                queue.push(RouteQueueState {
                    label: candidate,
                    cell: neighbour,
                });
            }
        }
    }

    let mut distances = vec![None; world.cell_count()];
    for (index, label) in labels.into_iter().enumerate() {
        let origin = CellId::new(index as u64 + 1);
        match (program.travel.resolution(origin), label) {
            (Some(TemporaryTravelResolution::Unreachable), None) => {}
            (Some(TemporaryTravelResolution::Reachable { destination, .. }), Some(label)) => {
                if label.destination != destination
                    || program.travel.accumulated_cost_units(origin) != Some(label.cost)
                {
                    return Err(invalid(format!(
                        "derived route distance for origin {} does not reconcile with authoritative travel table",
                        origin.0
                    )));
                }
                distances[index] = Some(label.hops);
            }
            _ => {
                return Err(invalid(format!(
                    "derived route reachability for origin {} does not reconcile with authoritative travel table",
                    origin.0
                )));
            }
        }
    }
    Ok(distances)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig, PopulationConfig,
        ResourceConfig, Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule,
        TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
    };

    fn configured_run(
        years: u64,
        trigger_day: u64,
        stay_days: u32,
    ) -> (ExperimentConfig, Population) {
        let seed = 91_607;
        let base = ExperimentConfig::new(seed, years)
            .with_world(WorldConfig::new(4, 1))
            .with_population(
                PopulationConfig::new(24)
                    .with_target_household_size(2)
                    .with_max_person_records(256),
            )
            .with_resources(ResourceConfig::synthetic_validation_v1())
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
        let base_simulation = Simulation::new(base.clone()).expect("base simulation");
        let first_residence = base_simulation
            .population()
            .household_location(HouseholdId::new(1))
            .expect("first household");
        let destination = if first_residence == CellId::new(1) {
            CellId::new(4)
        } else {
            CellId::new(1)
        };
        let region = FocalRegion::new(
            "temporary-observability-test-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .expect("region");
        let schedule = TemporaryMobilitySchedule::new(
            "temporary-observability-test-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![trigger_day],
            stay_days,
        )
        .expect("schedule");
        let temporary = TemporaryMobilityConfig::new(
            region,
            schedule,
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .expect("temporary config");
        let config = base.with_temporary_mobility(temporary);
        let simulation = Simulation::new(config.clone()).expect("configured simulation");
        (config, simulation.population().clone())
    }

    #[test]
    fn replay_partitions_person_days_and_is_deterministic() {
        let (config, initial_population) = configured_run(1, 10, 5);
        let simulation = Simulation::new(config).expect("simulation");
        let world = simulation.world().clone();
        let checkpoint = simulation.run_recorded().expect("run").checkpoint;
        let first =
            derive_temporary_mobility_observability(&world, &initial_population, &checkpoint)
                .expect("report");
        let second =
            derive_temporary_mobility_observability(&world, &initial_population, &checkpoint)
                .expect("report");
        assert_eq!(first, second);
        assert!(first.summary.journeys_started > 0);
        assert_eq!(
            first.summary.journeys_started,
            first.summary.journeys_completed
        );
        assert!(first.summary.visitor_person_days > 0);
        assert_eq!(
            first.summary.persistent_residence_person_days,
            first.summary.total_living_person_days
        );
        assert_eq!(
            first.summary.at_residence_person_days
                + first.summary.visitor_person_days
                + first.summary.transit_person_days,
            first.summary.total_living_person_days
        );
        let spatial_physical: u64 = first
            .cells
            .iter()
            .map(|cell| cell.at_residence_person_days + cell.visitor_person_days)
            .sum();
        assert_eq!(
            spatial_physical + first.summary.transit_person_days,
            first.summary.total_living_person_days
        );
        assert!(
            first
                .journeys
                .iter()
                .all(|journey| journey.one_way_route_distance_edges.is_some())
        );
    }

    #[test]
    fn uninterrupted_and_resumed_reports_reconcile_exactly() {
        let (config, initial_population) = configured_run(2, 360, 10);
        let uninterrupted = Simulation::new(config.clone()).expect("uninterrupted simulation");
        let world = uninterrupted.world().clone();
        let uninterrupted_checkpoint = uninterrupted.run_recorded().expect("run").checkpoint;

        let paused = Simulation::new(config).expect("paused simulation");
        let annual_checkpoint = paused.checkpoint_at_year(1).expect("annual checkpoint");
        assert!(
            (1..=annual_checkpoint.population.household_count() as u64).any(|raw| {
                annual_checkpoint
                    .temporary_mobility
                    .presence(HouseholdId::new(raw))
                    .is_some_and(|presence| !presence.is_at_residence())
            })
        );
        let resumed_checkpoint = Simulation::from_checkpoint(annual_checkpoint)
            .expect("resume")
            .run_recorded()
            .expect("resumed run")
            .checkpoint;
        assert_eq!(
            uninterrupted_checkpoint.state_digest64,
            resumed_checkpoint.state_digest64
        );
        assert_eq!(
            uninterrupted_checkpoint.experiment,
            resumed_checkpoint.experiment
        );
        assert_eq!(uninterrupted_checkpoint.time, resumed_checkpoint.time);
        assert_eq!(
            uninterrupted_checkpoint.completed_years,
            resumed_checkpoint.completed_years
        );
        assert_eq!(
            uninterrupted_checkpoint.terminal_stop_reason,
            resumed_checkpoint.terminal_stop_reason
        );
        assert_eq!(
            uninterrupted_checkpoint.world_digest64,
            resumed_checkpoint.world_digest64
        );
        assert_eq!(
            uninterrupted_checkpoint.population,
            resumed_checkpoint.population
        );
        assert_eq!(
            uninterrupted_checkpoint.temporary_mobility,
            resumed_checkpoint.temporary_mobility
        );
        assert_eq!(
            uninterrupted_checkpoint.resources,
            resumed_checkpoint.resources
        );
        assert_eq!(
            uninterrupted_checkpoint.migration,
            resumed_checkpoint.migration
        );
        assert_eq!(uninterrupted_checkpoint.rng, resumed_checkpoint.rng);
        assert_eq!(uninterrupted_checkpoint.events, resumed_checkpoint.events);
        assert_eq!(uninterrupted_checkpoint.metrics, resumed_checkpoint.metrics);
        assert!(
            uninterrupted_checkpoint
                .resume_lineage
                .boundaries
                .is_empty()
        );
        assert_eq!(resumed_checkpoint.resume_lineage.boundaries.len(), 1);

        let uninterrupted_report = derive_temporary_mobility_observability(
            &world,
            &initial_population,
            &uninterrupted_checkpoint,
        )
        .expect("uninterrupted report");
        let resumed_report = derive_temporary_mobility_observability(
            &world,
            &initial_population,
            &resumed_checkpoint,
        )
        .expect("resumed report");
        assert_eq!(uninterrupted_report, resumed_report);
    }
}
