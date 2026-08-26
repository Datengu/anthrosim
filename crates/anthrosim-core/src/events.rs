use serde::{Deserialize, Serialize};

use crate::{
    ids::{CellId, HouseholdId, PersonId, TemporaryJourneyId},
    migration::MigrationUtilityBreakdown,
    population::ReproductiveSex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventProvenance {
    Authoritative,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    DemographicMortality,
    /// Historical Rust variant name retained to minimize internal churn. In v10 this serializes as
    /// `condition_mediated`: the shared condition may reflect multiple explicit upstream causes,
    /// so the event must not be interpreted as proof of resource scarcity.
    #[serde(rename = "condition_mediated")]
    ResourceScarcity,
}

/// Explicit M9 reasons why a scheduled temporary journey did not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryJourneyIneligibility {
    NoLivingMembers,
    ActiveJourney,
    ResidenceInRegion,
    Unreachable,
    DepartureBeforeSimulationStart,
    DepartureWindowMissed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EventKind {
    Birth {
        person: PersonId,
        female_parent: PersonId,
        male_parent: PersonId,
        household: HouseholdId,
        /// Persistent residence cell inherited from the household at birth.
        cell: CellId,
        reproductive_sex: ReproductiveSex,
    },
    Death {
        person: PersonId,
        household: HouseholdId,
        /// Persistent residence cell used for demographic/spatial attribution. Under M9 this is
        /// not necessarily the person's physical location at death while the household is away.
        cell: CellId,
        cause: DeathCause,
        condition_permille: u16,
        probability_per_million: u32,
    },
    HouseholdMigration {
        household: HouseholdId,
        people_moved: u32,
        origin: CellId,
        destination: CellId,
        distance_cells: u16,
        pressure_permille: u16,
        origin_utility: MigrationUtilityBreakdown,
        destination_utility: MigrationUtilityBreakdown,
        best_candidate: CellId,
        best_candidate_utility: i32,
        selected_weight: u64,
        total_move_weight: u64,
        choice_draw: u64,
        travel_condition_cost_per_person: u16,
    },
    TemporaryJourneyNotStarted {
        event_schema_version: u32,
        household: HouseholdId,
        region_id: String,
        region_identity: String,
        trigger_index: u32,
        trigger_day: u64,
        reason: TemporaryJourneyIneligibility,
    },
    TemporaryJourneyDeparted {
        event_schema_version: u32,
        household: HouseholdId,
        journey: TemporaryJourneyId,
        region_id: String,
        region_identity: String,
        residence: CellId,
        destination: CellId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        travel_model_identity: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        accumulated_travel_cost_units: Option<u64>,
        people_affected: u32,
        trigger_index: u32,
        trigger_day: u64,
        departure_day: u64,
        arrival_day: u64,
        return_departure_day: u64,
        completion_day: u64,
        outbound_travel_days: u32,
        return_travel_days: u32,
    },
    TemporaryJourneyArrived {
        event_schema_version: u32,
        household: HouseholdId,
        journey: TemporaryJourneyId,
        region_id: String,
        region_identity: String,
        destination: CellId,
        people_affected: u32,
    },
    TemporaryReturnDeparted {
        event_schema_version: u32,
        household: HouseholdId,
        journey: TemporaryJourneyId,
        region_id: String,
        region_identity: String,
        destination: CellId,
        residence: CellId,
        people_affected: u32,
    },
    TemporaryJourneyCompleted {
        event_schema_version: u32,
        household: HouseholdId,
        journey: TemporaryJourneyId,
        region_id: String,
        region_identity: String,
        residence: CellId,
        people_affected: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    pub sequence: u64,
    pub day: u64,
    pub provenance: EventProvenance,
    pub event: EventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    pub schema_version: u32,
    pub events: Vec<EventRecord>,
}

impl EventLog {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            events: Vec::new(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub(crate) fn push_authoritative(&mut self, day: u64, event: EventKind) {
        let sequence = u64::try_from(self.events.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        self.events.push(EventRecord {
            sequence,
            day,
            provenance: EventProvenance::Authoritative,
            event,
        });
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
